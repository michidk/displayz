use core::fmt;
use std::cell::{Cell, RefCell};

use thiserror::Error;
use winsafe::{EnumDisplayDevices, co};

use crate::{
    DisplayPropertiesError,
    properties::{DisplayProperties, DisplaySettings, Position},
};

/// Error type for the display module
#[derive(Error, Debug)]
pub enum DisplayError {
    #[error("Error in DisplayProperties")]
    Properties(#[from] DisplayPropertiesError),
    #[error("Error when calling the Windows API")]
    WinAPI(#[from] co::ERROR),
    #[error("Only active displays can used as a primary display")]
    PrimaryDisplay,
    #[error("Display {0} has no settings")]
    NoSettings(String),
    #[error("Failed to commit the changes; Returned flags: {0}")]
    FailedToCommit(co::DISP_CHANGE),
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

    pub fn is_primary(&self) -> bool {
        self.display_set.primary_display.get() == self.index
    }

    pub fn set_primary(&self) -> Result {
        self.display_set.set_primary(self)
    }

    pub fn apply(&self) -> Result {
        self.properties().apply().map_err(DisplayError::Properties)
    }
}

/// A struct that represents a set of displays
#[derive(Debug, Clone)]
pub struct DisplaySet {
    /// The displays in this set
    displays: Vec<DisplayProperties>,
    /// The primary display
    primary_display: Cell<usize>,
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

    /// Sets all changes on the displays
    pub fn apply(&self) -> Result {
        let primary_idx = self.primary_display.get();

        // Apply the primary display first (Windows API requirement)
        if primary_idx < self.displays.len() {
            let primary = &self.displays[primary_idx];
            if primary.active {
                primary.apply()?;
            }
        }

        // Then apply all other displays
        for (i, display) in self.displays.iter().enumerate() {
            if display.active && i != primary_idx {
                display.apply()?;
            }
        }

        Ok(())
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
    let mut result = Vec::<DisplayProperties>::new();
    let mut primary_index = 0;

    for (dev_num, display_device) in EnumDisplayDevices(None, None).enumerate() {
        let display_device = match display_device {
            Ok(dd) => dd,
            Err(e) => {
                // Windows 11 24H2 returns ERROR_INVALID_HANDLE after enumerating all displays
                // This is expected behavior when there are no more displays to enumerate
                if e == co::ERROR::INVALID_HANDLE {
                    log::debug!("No more displays to enumerate (got ERROR_INVALID_HANDLE)");
                    break;
                }
                return Err(e.into());
            }
        };

        log::debug!(
            "{}: {} - {}",
            dev_num,
            display_device.DeviceName(),
            display_device.DeviceString()
        );

        let properties = DisplayProperties::from_winsafe(display_device)?;
        if properties.primary.get() {
            primary_index = dev_num;
        }
        result.push(properties);
    }

    Ok(DisplaySet {
        displays: result,
        primary_display: Cell::new(primary_index),
    })
}

/// Refreshes the screen to apply the changes
pub fn refresh() -> Result {
    let result = winsafe::ChangeDisplaySettingsEx(None, None, winsafe::co::CDS::DYNAMICALLY);
    match result {
        Ok(_) => Ok(()),
        Err(err) => Err(DisplayError::FailedToCommit(err)),
    }
}
