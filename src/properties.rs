use core::fmt;
use std::cell::{Cell, RefCell};

use thiserror::Error;
use windows::Win32::Devices::Display::{
    DISPLAYCONFIG_DEVICE_INFO_GET_SOURCE_NAME, DISPLAYCONFIG_DEVICE_INFO_GET_TARGET_NAME,
    DISPLAYCONFIG_DEVICE_INFO_HEADER, DISPLAYCONFIG_MODE_INFO, DISPLAYCONFIG_PATH_INFO,
    DISPLAYCONFIG_SOURCE_DEVICE_NAME, DISPLAYCONFIG_TARGET_DEVICE_NAME, DisplayConfigGetDeviceInfo,
};
use windows::Win32::Foundation::POINTL;
use windows::Win32::Graphics::Gdi::{
    DEVMODEW, DISPLAY_DEVICEW, ENUM_CURRENT_SETTINGS, EnumDisplaySettingsW,
};
use windows::core::PCWSTR;

use crate::types::{
    BitDepth, ConnectorType, Frequency, Orientation, Position, Resolution, Scaling,
    ScanlineOrdering,
};

/// Error type for the display module
#[derive(Error, Debug)]
pub enum DisplayPropertiesError {
    #[error("Display {0} has no settings")]
    NoSettings(String),
    #[error("Error when calling the Windows API: {0}")]
    WinAPI(String),
}

type Result<T = ()> = std::result::Result<T, DisplayPropertiesError>;

/// Contains the properties of a display
#[derive(Clone, Debug)]
pub struct DisplayProperties {
    pub name: String,

    pub string: String,
    pub key: String,

    pub active: bool,
    pub primary: Cell<bool>,

    pub settings: Option<RefCell<DisplaySettings>>,

    // Additional display information (modern API only)
    pub connector_type: Option<ConnectorType>,
    pub target_available: bool,
}

impl fmt::Display for DisplayProperties {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Display {{ key: {}, name: {}, string: {}, active: {}, primary: {} }}",
            self.key,
            self.name,
            self.string,
            self.active,
            self.primary.get()
        )
    }
}

/// Contains the settings of a display
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct DisplaySettings {
    pub position: Position,
    pub resolution: Resolution,
    pub frequency: Frequency,
    pub orientation: Orientation,
    pub scaling: Scaling,
    pub bit_depth: BitDepth,
    pub scanline_ordering: ScanlineOrdering,
}

impl DisplayProperties {
    /// Create a display properties struct from a Windows display device
    pub fn from_windows(device: &DISPLAY_DEVICEW) -> Result<DisplayProperties> {
        use windows::Win32::Graphics::Gdi::DISPLAY_DEVICE_STATE_FLAGS;
        let active = (device.StateFlags & DISPLAY_DEVICE_STATE_FLAGS(0x00000001))
            != DISPLAY_DEVICE_STATE_FLAGS(0); // DISPLAY_DEVICE_ATTACHED_TO_DESKTOP

        // Convert device name from wide string
        let name = {
            let len = device
                .DeviceName
                .iter()
                .position(|&c| c == 0)
                .unwrap_or(device.DeviceName.len());
            String::from_utf16_lossy(&device.DeviceName[..len])
        };

        let settings = if active {
            Some(RefCell::new(Self::fetch_settings(&name)?))
        } else {
            None
        };

        // Convert device string from wide string
        let string = {
            let len = device
                .DeviceString
                .iter()
                .position(|&c| c == 0)
                .unwrap_or(device.DeviceString.len());
            String::from_utf16_lossy(&device.DeviceString[..len])
        };

        // Convert device key from wide string
        let key = {
            let len = device
                .DeviceKey
                .iter()
                .position(|&c| c == 0)
                .unwrap_or(device.DeviceKey.len());
            String::from_utf16_lossy(&device.DeviceKey[..len])
        };

        Ok(DisplayProperties {
            name,
            string,
            key,
            active,
            primary: Cell::new(
                (device.StateFlags & DISPLAY_DEVICE_STATE_FLAGS(0x00000004))
                    != DISPLAY_DEVICE_STATE_FLAGS(0),
            ), // DISPLAY_DEVICE_PRIMARY_DEVICE
            settings,
            connector_type: None,     // Not available in legacy API
            target_available: active, // Assume available if active
        })
    }

    /// Fetch the settings of a display
    fn fetch_settings(name: &str) -> Result<DisplaySettings> {
        let mut devmode: DEVMODEW = unsafe { std::mem::zeroed() };
        devmode.dmSize = std::mem::size_of::<DEVMODEW>() as u16;

        // Convert name to wide string
        let wide_name: Vec<u16> = name.encode_utf16().chain(std::iter::once(0)).collect();

        let result = unsafe {
            EnumDisplaySettingsW(
                PCWSTR(wide_name.as_ptr()),
                ENUM_CURRENT_SETTINGS,
                &mut devmode,
            )
        };

        if !result.as_bool() {
            return Err(DisplayPropertiesError::WinAPI(format!(
                "EnumDisplaySettingsW failed for display {}",
                name
            )));
        }

        Ok(DisplaySettings {
            position: Position(POINTL {
                x: unsafe { devmode.Anonymous1.Anonymous2.dmPosition.x },
                y: unsafe { devmode.Anonymous1.Anonymous2.dmPosition.y },
            }),
            resolution: Resolution::new(devmode.dmPelsWidth, devmode.dmPelsHeight),
            frequency: Frequency(devmode.dmDisplayFrequency),
            orientation: Orientation::from_rotation(unsafe {
                devmode.Anonymous1.Anonymous2.dmDisplayOrientation.0
            }),
            scaling: Scaling::default(), // Not available in legacy API
            bit_depth: BitDepth::Bpp32,  // Default assumption
            scanline_ordering: ScanlineOrdering::Progressive, // Default assumption
        })
    }

    /// Create a display properties struct from modern Windows Display Configuration API
    pub fn from_display_config(
        path: &DISPLAYCONFIG_PATH_INFO,
        modes: &[DISPLAYCONFIG_MODE_INFO],
    ) -> Result<DisplayProperties> {
        let active = (path.flags & 0x00000001) != 0; // DISPLAYCONFIG_PATH_ACTIVE

        // Get source and target mode indices
        let source_mode_idx = unsafe { path.sourceInfo.Anonymous.modeInfoIdx as usize };
        let target_mode_idx = unsafe { path.targetInfo.Anonymous.modeInfoIdx as usize };

        if source_mode_idx >= modes.len() || target_mode_idx >= modes.len() {
            return Err(DisplayPropertiesError::WinAPI(
                "Invalid mode index in display path".to_string(),
            ));
        }

        let source_mode = &modes[source_mode_idx];
        let target_mode = &modes[target_mode_idx];

        // Extract position from source mode
        let position = unsafe { source_mode.Anonymous.sourceMode.position };

        // Determine if primary (position == 0,0)
        let is_primary = position.x == 0 && position.y == 0;

        // Get device names via DisplayConfigGetDeviceInfo
        let name = Self::get_source_device_name(path)?;
        let (string, key) = Self::get_target_device_info(path)?;

        let settings = if active {
            Some(RefCell::new(Self::fetch_settings_from_mode(
                path,
                source_mode,
                target_mode,
            )?))
        } else {
            None
        };

        // Extract connector type
        let connector_type = Some(ConnectorType::from_value(
            path.targetInfo.outputTechnology.0,
        ));

        // Extract target availability
        let target_available = path.targetInfo.targetAvailable.as_bool();

        Ok(DisplayProperties {
            name,
            string,
            key,
            active,
            primary: Cell::new(is_primary),
            settings,
            connector_type,
            target_available,
        })
    }

    /// Gets the GDI device name for a display path
    pub fn get_source_device_name(path: &DISPLAYCONFIG_PATH_INFO) -> Result<String> {
        let mut source_name: DISPLAYCONFIG_SOURCE_DEVICE_NAME = unsafe { std::mem::zeroed() };
        source_name.header.r#type = DISPLAYCONFIG_DEVICE_INFO_GET_SOURCE_NAME;
        source_name.header.size = std::mem::size_of::<DISPLAYCONFIG_SOURCE_DEVICE_NAME>() as u32;
        source_name.header.adapterId = path.sourceInfo.adapterId;
        source_name.header.id = path.sourceInfo.id;

        let result = unsafe {
            DisplayConfigGetDeviceInfo(
                &mut source_name.header as *mut DISPLAYCONFIG_DEVICE_INFO_HEADER,
            )
        };

        if result != 0 {
            return Err(DisplayPropertiesError::WinAPI(format!(
                "DisplayConfigGetDeviceInfo (source) failed: {}",
                result
            )));
        }

        // Convert wide string to String
        let len = source_name
            .viewGdiDeviceName
            .iter()
            .position(|&c| c == 0)
            .unwrap_or(source_name.viewGdiDeviceName.len());
        Ok(String::from_utf16_lossy(
            &source_name.viewGdiDeviceName[..len],
        ))
    }

    /// Gets the friendly name and device path for a display target
    fn get_target_device_info(path: &DISPLAYCONFIG_PATH_INFO) -> Result<(String, String)> {
        let mut target_name: DISPLAYCONFIG_TARGET_DEVICE_NAME = unsafe { std::mem::zeroed() };
        target_name.header.r#type = DISPLAYCONFIG_DEVICE_INFO_GET_TARGET_NAME;
        target_name.header.size = std::mem::size_of::<DISPLAYCONFIG_TARGET_DEVICE_NAME>() as u32;
        target_name.header.adapterId = path.targetInfo.adapterId;
        target_name.header.id = path.targetInfo.id;

        let result = unsafe {
            DisplayConfigGetDeviceInfo(
                &mut target_name.header as *mut DISPLAYCONFIG_DEVICE_INFO_HEADER,
            )
        };

        if result != 0 {
            return Err(DisplayPropertiesError::WinAPI(format!(
                "DisplayConfigGetDeviceInfo (target) failed: {}",
                result
            )));
        }

        // Convert friendly name
        let friendly_len = target_name
            .monitorFriendlyDeviceName
            .iter()
            .position(|&c| c == 0)
            .unwrap_or(target_name.monitorFriendlyDeviceName.len());
        let friendly_name =
            String::from_utf16_lossy(&target_name.monitorFriendlyDeviceName[..friendly_len]);

        // Convert device path (key)
        let path_len = target_name
            .monitorDevicePath
            .iter()
            .position(|&c| c == 0)
            .unwrap_or(target_name.monitorDevicePath.len());
        let device_path = String::from_utf16_lossy(&target_name.monitorDevicePath[..path_len]);

        Ok((friendly_name, device_path))
    }

    /// Converts modern API mode info to DisplaySettings
    fn fetch_settings_from_mode(
        path: &DISPLAYCONFIG_PATH_INFO,
        source_mode: &DISPLAYCONFIG_MODE_INFO,
        target_mode: &DISPLAYCONFIG_MODE_INFO,
    ) -> Result<DisplaySettings> {
        let source = unsafe { &source_mode.Anonymous.sourceMode };
        let target = unsafe { &target_mode.Anonymous.targetMode };

        // Extract rotation from path.targetInfo.rotation
        let rotation = path.targetInfo.rotation;
        let orientation = Orientation::from_rotation(rotation.0 as u32);

        // Extract scaling from targetInfo
        let scaling = Scaling::from_value(path.targetInfo.scaling.0);

        // Extract bit depth from pixel format
        let bit_depth = BitDepth::from_pixel_format(source.pixelFormat.0);

        // Extract scanline ordering
        let scanline_ordering =
            ScanlineOrdering::from_value(target.targetVideoSignalInfo.scanLineOrdering.0);

        Ok(DisplaySettings {
            position: Position(source.position),
            resolution: Resolution::new(source.width, source.height),
            frequency: Frequency::new(Self::calculate_frequency(&target.targetVideoSignalInfo)),
            orientation,
            scaling,
            bit_depth,
            scanline_ordering,
        })
    }

    /// Calculates frequency from rational vSyncFreq
    fn calculate_frequency(
        signal_info: &windows::Win32::Devices::Display::DISPLAYCONFIG_VIDEO_SIGNAL_INFO,
    ) -> u32 {
        if signal_info.vSyncFreq.Denominator != 0 {
            signal_info.vSyncFreq.Numerator / signal_info.vSyncFreq.Denominator
        } else {
            60 // Default fallback
        }
    }
}
