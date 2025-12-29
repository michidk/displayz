use core::fmt;
use std::cell::{Cell, RefCell};

use thiserror::Error;
use windows::Win32::Devices::Display::{
    DISPLAYCONFIG_MODE_INFO, DISPLAYCONFIG_PATH_INFO, GetDisplayConfigBufferSizes,
    QDC_ONLY_ACTIVE_PATHS, QueryDisplayConfig, SDC_ALLOW_CHANGES, SDC_APPLY, SDC_SAVE_TO_DATABASE,
    SDC_USE_SUPPLIED_DISPLAY_CONFIG, SetDisplayConfig,
};

use crate::{
    DisplayPropertiesError,
    properties::{DisplayProperties, DisplaySettings},
    types::Position,
};

/// Error type for the display module
#[derive(Error, Debug)]
pub enum DisplayError {
    #[error("Error in DisplayProperties")]
    Properties(#[from] DisplayPropertiesError),
    #[error("Error when calling the Windows API: {0}")]
    WinAPI(String),
    #[error("Only active displays can used as a primary display")]
    PrimaryDisplay,
    #[error("Display {0} has no settings")]
    NoSettings(String),
    #[error("Failed to commit the changes; Returned code: {0}")]
    FailedToCommit(i32),
}

type Result<T = ()> = std::result::Result<T, DisplayError>;

/// A struct that represents a display (index)
#[derive(Debug, Clone)]
pub struct Display<'a> {
    /// The index of the display in the display set
    index: usize,
    /// THe display set containing this display
    display_set: &'a DisplaySet,
}

/// Generates getter for properties of a display
macro_rules! get_properties_str {
    ($field:ident) => {
        pub fn $field(&self) -> &str {
            self.properties().$field.as_str()
        }
    };
}

impl Display<'_> {
    pub fn index(&self) -> usize {
        self.index
    }

    fn properties(&self) -> &DisplayProperties {
        &self.display_set.displays[self.index]
    }

    get_properties_str!(name);
    get_properties_str!(string);
    get_properties_str!(key);

    pub fn settings(&self) -> &Option<RefCell<DisplaySettings>> {
        &self.properties().settings
    }

    pub fn connector_type(&self) -> &Option<crate::types::ConnectorType> {
        &self.properties().connector_type
    }

    pub fn target_available(&self) -> bool {
        self.properties().target_available
    }

    pub fn is_primary(&self) -> bool {
        self.display_set.primary_display.get() == self.index
    }

    pub fn set_primary(&self) -> Result {
        self.display_set.set_primary(self)
    }
}

/// A struct that represents a set of displays
#[derive(Clone)]
pub struct DisplaySet {
    /// The displays in this set
    displays: Vec<DisplayProperties>,
    /// The primary display
    primary_display: Cell<usize>,
    /// The display configuration paths (for modern API)
    paths: RefCell<Vec<DISPLAYCONFIG_PATH_INFO>>,
    /// The display configuration modes (for modern API)
    modes: RefCell<Vec<DISPLAYCONFIG_MODE_INFO>>,
}

impl fmt::Debug for DisplaySet {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("DisplaySet")
            .field("displays", &self.displays)
            .field("primary_display", &self.primary_display)
            .field("paths", &format!("<{} paths>", self.paths.borrow().len()))
            .field("modes", &format!("<{} modes>", self.modes.borrow().len()))
            .finish()
    }
}

impl DisplaySet {
    /// Iterates over the displays in this set
    pub fn displays(&self) -> impl ExactSizeIterator<Item = Display<'_>> {
        self.displays.iter().enumerate().map(|(index, _)| Display {
            index,
            display_set: self,
        })
    }

    /// Returns display for the given `index`
    pub fn get(&self, index: usize) -> Option<Display<'_>> {
        if index >= self.displays.len() {
            return None;
        }
        Some(Display {
            index,
            display_set: self,
        })
    }

    /// Returns the primary display
    pub fn primary(&self) -> Display<'_> {
        Display {
            index: self.primary_display.get(),
            display_set: self,
        }
    }

    /// Sets the given `display` as the primary display
    /// Requires a call to `display_set.apply` and `commit_changes` afterwards
    pub fn set_primary(&self, display: &Display) -> Result {
        let index = display.index;
        let new_primary = &self.displays[index];

        if !new_primary.active {
            return Err(DisplayError::PrimaryDisplay);
        }

        let old_position = new_primary
            .settings
            .as_ref()
            .ok_or_else(|| DisplayError::NoSettings(new_primary.name.to_string()))?
            .borrow()
            .position;

        // move all other displays to new position (because we set a new origin in the next step)
        for (i, display) in self.displays.iter().enumerate() {
            if display.active && i != index {
                let settings = display
                    .settings
                    .as_ref()
                    .ok_or_else(|| DisplayError::NoSettings(display.name.to_string()))?;
                let pos = settings.borrow().position;
                settings.borrow_mut().position = -old_position + pos;
                // unset primary flag on all other displays
                display.primary.set(false);
            }
        }

        // the new primary is the new origin
        let new_primary_mut = &self.displays[index];
        let new_settings = new_primary_mut
            .settings
            .as_ref()
            .ok_or_else(|| DisplayError::NoSettings(new_primary_mut.name.to_string()))?;

        new_settings.borrow_mut().position = Position::new(0, 0);
        // set primary flag on the new primary display
        new_primary_mut.primary.set(true);

        self.primary_display.set(index);

        Ok(())
    }

    /// Applies all pending display configuration changes
    ///
    /// This updates the Windows display configuration to match the current settings.
    /// Modified settings include: position, resolution, frequency, orientation, and scaling.
    /// Read-only properties (bit_depth, scanline_ordering) cannot be changed.
    pub fn apply(&self) -> Result {
        let mut paths = self.paths.borrow_mut();
        let mut modes = self.modes.borrow_mut();

        for display in self.displays.iter().filter(|d| d.active) {
            let Some(path_idx) = Self::find_path_for_display(&paths, &display.name) else {
                continue;
            };

            let path = &mut paths[path_idx];
            let source_idx = unsafe { path.sourceInfo.Anonymous.modeInfoIdx as usize };
            let target_idx = unsafe { path.targetInfo.Anonymous.modeInfoIdx as usize };

            if let Some(settings) = &display.settings {
                let settings = settings.borrow();
                Self::update_source_mode(&mut modes, source_idx, &settings);
                Self::update_target_mode(&mut modes, target_idx, &settings);
                Self::update_path_info(path, &settings);
            }
        }

        Self::commit_display_config(&paths, &modes)
    }

    fn find_path_for_display(
        paths: &[DISPLAYCONFIG_PATH_INFO],
        display_name: &str,
    ) -> Option<usize> {
        paths.iter().position(|p| {
            DisplayProperties::get_source_device_name(p)
                .map(|name| name == display_name)
                .unwrap_or(false)
        })
    }

    fn update_source_mode(
        modes: &mut [DISPLAYCONFIG_MODE_INFO],
        idx: usize,
        settings: &DisplaySettings,
    ) {
        if idx >= modes.len() {
            return;
        }

        unsafe {
            let mode = &mut modes[idx].Anonymous.sourceMode;
            mode.position = settings.position.0;
            mode.width = settings.resolution.width;
            mode.height = settings.resolution.height;
        }
    }

    fn update_target_mode(
        modes: &mut [DISPLAYCONFIG_MODE_INFO],
        idx: usize,
        settings: &DisplaySettings,
    ) {
        if idx >= modes.len() {
            return;
        }

        unsafe {
            let vsync = &mut modes[idx]
                .Anonymous
                .targetMode
                .targetVideoSignalInfo
                .vSyncFreq;
            vsync.Numerator = settings.frequency.0;
            vsync.Denominator = 1;
        }
    }

    fn update_path_info(path: &mut DISPLAYCONFIG_PATH_INFO, settings: &DisplaySettings) {
        use windows::Win32::Devices::Display::{DISPLAYCONFIG_ROTATION, DISPLAYCONFIG_SCALING};

        path.targetInfo.rotation =
            DISPLAYCONFIG_ROTATION(settings.orientation.to_rotation() as i32);
        path.targetInfo.scaling = DISPLAYCONFIG_SCALING(settings.scaling.to_value());
    }

    fn commit_display_config(
        paths: &[DISPLAYCONFIG_PATH_INFO],
        modes: &[DISPLAYCONFIG_MODE_INFO],
    ) -> Result {
        let result = unsafe {
            SetDisplayConfig(
                Some(paths),
                Some(modes),
                SDC_APPLY
                    | SDC_USE_SUPPLIED_DISPLAY_CONFIG
                    | SDC_ALLOW_CHANGES
                    | SDC_SAVE_TO_DATABASE,
            )
        };

        if result == 0 {
            log::debug!("Successfully applied display configuration");
            Ok(())
        } else {
            log::error!(
                "Failed to apply display configuration: error code {}",
                result
            );
            Err(DisplayError::FailedToCommit(result))
        }
    }
}

impl fmt::Display for DisplaySet {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "DisplaySet {{ displays: [")?;
        for (i, display) in self.displays.iter().enumerate() {
            if i > 0 {
                writeln!(f, ", ")?;
            }
            write!(f, "    {}", display)?;
        }
        write!(f, "\n] }}")
    }
}

/// Returns a list of all displays.
pub fn query_displays() -> Result<DisplaySet> {
    let mut num_paths: u32 = 0;
    let mut num_modes: u32 = 0;

    // Step 1: Get buffer sizes
    unsafe {
        GetDisplayConfigBufferSizes(QDC_ONLY_ACTIVE_PATHS, &mut num_paths, &mut num_modes)
            .ok()
            .map_err(|e| {
                DisplayError::WinAPI(format!("GetDisplayConfigBufferSizes failed: {:?}", e))
            })?;
    }

    log::debug!("Display config: {} paths, {} modes", num_paths, num_modes);

    // Step 2: Allocate and query paths/modes
    let mut paths = vec![DISPLAYCONFIG_PATH_INFO::default(); num_paths as usize];
    let mut modes = vec![DISPLAYCONFIG_MODE_INFO::default(); num_modes as usize];

    unsafe {
        QueryDisplayConfig(
            QDC_ONLY_ACTIVE_PATHS,
            &mut num_paths,
            paths.as_mut_ptr(),
            &mut num_modes,
            modes.as_mut_ptr(),
            None,
        )
        .ok()
        .map_err(|e| DisplayError::WinAPI(format!("QueryDisplayConfig failed: {:?}", e)))?;
    }

    // Truncate to actual returned counts
    paths.truncate(num_paths as usize);
    modes.truncate(num_modes as usize);

    // Step 3: Convert each path to DisplayProperties
    let mut result = Vec::<DisplayProperties>::new();
    let mut primary_index = 0;

    for (path_idx, path) in paths.iter().enumerate() {
        // Skip inactive paths
        if (path.flags & 0x00000001) == 0 {
            // DISPLAYCONFIG_PATH_ACTIVE
            continue;
        }

        let properties = DisplayProperties::from_display_config(path, &modes)?;

        log::debug!(
            "Display {}: {} - {} (primary={})",
            path_idx,
            properties.name,
            properties.string,
            properties.primary.get()
        );

        // Primary is at position (0, 0)
        if properties.primary.get() {
            primary_index = result.len();
        }

        result.push(properties);
    }

    Ok(DisplaySet {
        displays: result,
        primary_display: Cell::new(primary_index),
        paths: RefCell::new(paths),
        modes: RefCell::new(modes),
    })
}

/// Refreshes the screen to apply the changes
pub fn refresh() -> Result {
    // Re-query and re-apply current configuration
    let mut num_paths: u32 = 0;
    let mut num_modes: u32 = 0;

    unsafe {
        GetDisplayConfigBufferSizes(QDC_ONLY_ACTIVE_PATHS, &mut num_paths, &mut num_modes)
            .ok()
            .map_err(|e| {
                DisplayError::WinAPI(format!("GetDisplayConfigBufferSizes failed: {:?}", e))
            })?;

        let mut paths = vec![DISPLAYCONFIG_PATH_INFO::default(); num_paths as usize];
        let mut modes = vec![DISPLAYCONFIG_MODE_INFO::default(); num_modes as usize];

        QueryDisplayConfig(
            QDC_ONLY_ACTIVE_PATHS,
            &mut num_paths,
            paths.as_mut_ptr(),
            &mut num_modes,
            modes.as_mut_ptr(),
            None,
        )
        .ok()
        .map_err(|e| DisplayError::WinAPI(format!("QueryDisplayConfig failed: {:?}", e)))?;

        let result = SetDisplayConfig(
            Some(&paths[..num_paths as usize]),
            Some(&modes[..num_modes as usize]),
            SDC_APPLY | SDC_USE_SUPPLIED_DISPLAY_CONFIG | SDC_ALLOW_CHANGES,
        );

        if result == 0 {
            // ERROR_SUCCESS
            Ok(())
        } else {
            Err(DisplayError::FailedToCommit(result))
        }?;
    }

    Ok(())
}
