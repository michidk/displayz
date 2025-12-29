use displayz::query_displays;

#[test]
fn test_find_display_functionality() -> Result<(), Box<dyn std::error::Error>> {
    let _ = env_logger::builder().is_test(true).try_init();

    let display_set = query_displays()?;
    println!("Discovered displays:\n{}", display_set);

    assert!(
        display_set.displays().count() > 0,
        "Should have at least one display"
    );

    test_primary_display(&display_set);
    test_find_by_name(&display_set);
    test_get_by_index(&display_set);

    Ok(())
}

fn test_primary_display(display_set: &displayz::DisplaySet) {
    let primary = display_set.primary();
    println!(
        "Primary display: {} at index {}",
        primary.name(),
        primary.index()
    );
    assert!(primary.is_primary(), "Primary display flag should be set");
}

fn test_find_by_name(display_set: &displayz::DisplaySet) {
    let first_display = display_set.displays().next().unwrap();
    let name = first_display.name();

    let found = display_set.displays().find(|d| d.name() == name);
    assert!(found.is_some(), "Should find display by name");
}

fn test_get_by_index(display_set: &displayz::DisplaySet) {
    let Some(display) = display_set.get(0) else {
        return;
    };

    println!("Display at index 0: {}", display.name());

    if let Some(settings) = display.settings() {
        let position = settings.borrow().position;
        println!("  Position: {}", position);
    }
}
