use displayz::query_displays;

/// Finds displays by name and index
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let display_set = query_displays()?;
    println!("Discovered displays:\n{}", display_set);

    // find a display by filtering
    match display_set
        .displays()
        .find(|display| display.name() == "\\\\.\\DISPLAY3")
    {
        Some(display) => println!("Display3 found. Is primary? {}", display.is_primary()),
        None => println!("Display 3 not found"),
    }

    println!(
        "Primary is `{}` with index `{}`",
        display_set.primary().name(),
        display_set.primary().index()
    );

    // getting the displays properties by index
    if let Some(new_primary) = display_set.get(0)
        && let Some(settings) = new_primary.settings()
    {
        println!(
            "Position of display with index 0: {}",
            settings.borrow().position
        );
    }

    Ok(())
}
