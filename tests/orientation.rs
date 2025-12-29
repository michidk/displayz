use displayz::{Orientation, query_displays, refresh};

#[test]
fn test_orientation_flip_functionality() -> Result<(), Box<dyn std::error::Error>> {
    let _ = env_logger::builder().is_test(true).try_init();

    let display_set = query_displays()?;
    println!("Discovered displays:\n{}", display_set);

    let primary = display_set.primary();
    let settings = primary
        .settings()
        .as_ref()
        .expect("Primary display should have settings");

    let original = settings.borrow().orientation;
    println!("Original orientation: {:?}", original);

    let flipped = flip_orientation(original);
    verify_flip_logic(original, flipped);

    // Apply flipped orientation
    settings.borrow_mut().orientation = flipped;
    display_set.apply()?;
    refresh()?;

    // Verify the change
    let current = get_primary_orientation()?;
    assert_eq!(current, flipped, "Orientation should be {:?}", flipped);
    println!("✓ Changed to: {:?}", current);

    // Restore original orientation
    settings.borrow_mut().orientation = original;
    display_set.apply()?;
    refresh()?;

    // Verify restoration
    let restored = get_primary_orientation()?;
    assert_eq!(restored, original, "Should restore to {:?}", original);
    println!("✓ Restored to: {:?}", restored);

    Ok(())
}

fn flip_orientation(orientation: Orientation) -> Orientation {
    match orientation {
        Orientation::Portrait => Orientation::PortraitFlipped,
        Orientation::PortraitFlipped => Orientation::Portrait,
        Orientation::Landscape => Orientation::LandscapeFlipped,
        Orientation::LandscapeFlipped => Orientation::Landscape,
    }
}

fn verify_flip_logic(original: Orientation, flipped: Orientation) {
    let expected = match original {
        Orientation::Portrait => Orientation::PortraitFlipped,
        Orientation::PortraitFlipped => Orientation::Portrait,
        Orientation::Landscape => Orientation::LandscapeFlipped,
        Orientation::LandscapeFlipped => Orientation::Landscape,
    };
    assert_eq!(flipped, expected, "Flip logic should be correct");
}

fn get_primary_orientation() -> Result<Orientation, Box<dyn std::error::Error>> {
    let display_set = query_displays()?;
    Ok(display_set
        .primary()
        .settings()
        .as_ref()
        .expect("Primary should have settings")
        .borrow()
        .orientation)
}
