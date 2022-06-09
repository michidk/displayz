use displayz::{query_displays, refresh};

/// Sets a display to be the new primary display
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let display_set = query_displays()?;
    println!("Discovered displays:\n{}", display_set);

    // find a display by filtering by name
    let display = display_set
        .displays()
        .find(|display| display.name() == "\\\\.\\DISPLAY2");

    // set display primary
    if let Some(display) = display {
        display.set_primary()?;
        // display_set.set_primary(&display)?; // this works, too
    }

    // apply the changed settings
    display_set.apply()?;
    refresh()?;

    Ok(())
}
