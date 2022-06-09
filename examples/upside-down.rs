use displayz::{query_displays, refresh, Orientation};

/// Turns the primary display upside-down
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let display_set = query_displays()?;
    println!("Discovered displays:\n{}", display_set);

    if let Some(settings) = display_set.primary().settings() {
        {
            let mut settings = &mut *settings.borrow_mut();
            println!("Current orientation: {:?}", settings.orientation);

            settings.orientation = match settings.orientation {
                Orientation::PortraitFlipped => Orientation::Portrait,
                Orientation::Portrait => Orientation::PortraitFlipped,
                Orientation::Landscape => Orientation::LandscapeFlipped,
                Orientation::LandscapeFlipped => Orientation::Landscape,
            };

            println!("New orientation: {:?}", settings.orientation);
        }
    } else {
        eprintln!("Primary display has no settings");
    }

    display_set.primary().apply()?;
    refresh()?;

    Ok(())
}
