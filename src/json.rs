use serde::Serialize;

use crate::Display;

/// Serializable display settings for JSON output
#[derive(Serialize)]
pub struct DisplaySettingsJson {
    pub position: PositionJson,
    pub resolution: ResolutionJson,
    pub frequency: u32,
    pub orientation: String,
    pub scaling: String,
    pub bit_depth: String,
    pub scanline_ordering: String,
}

/// Serializable position for JSON output
#[derive(Serialize)]
pub struct PositionJson {
    pub x: i32,
    pub y: i32,
}

/// Serializable resolution for JSON output
#[derive(Serialize)]
pub struct ResolutionJson {
    pub width: u32,
    pub height: u32,
}

/// Serializable display info for JSON output
#[derive(Serialize)]
pub struct DisplayInfoJson {
    pub id: usize,
    pub windows_display_number: usize,
    pub name: String,
    pub string: String,
    pub key: String,
    pub primary: bool,
    pub connector: Option<String>,
    pub available: bool,
    pub settings: Option<DisplaySettingsJson>,
}

/// Converts display data to JSON serializable format
pub fn display_to_json(display: &Display) -> DisplayInfoJson {
    let settings_json = display.settings().as_ref().map(|s| {
        let settings = s.borrow();
        DisplaySettingsJson {
            position: PositionJson {
                x: settings.position.0.x,
                y: settings.position.0.y,
            },
            resolution: ResolutionJson {
                width: settings.resolution.width,
                height: settings.resolution.height,
            },
            frequency: settings.frequency.0,
            orientation: settings.orientation.to_string(),
            scaling: settings.scaling.to_string(),
            bit_depth: settings.bit_depth.to_string(),
            scanline_ordering: settings.scanline_ordering.to_string(),
        }
    });

    DisplayInfoJson {
        id: display.index(),
        windows_display_number: display.index() + 1,
        name: display.name().to_string(),
        string: display.string().to_string(),
        key: display.key().to_string(),
        primary: display.is_primary(),
        connector: display.connector_type().as_ref().map(|c| c.to_string()),
        available: display.target_available(),
        settings: settings_json,
    }
}
