use displayz::{query_displays, refresh};
use windows::Win32::Graphics::Gdi::{
    DISPLAY_DEVICE_STATE_FLAGS, DISPLAY_DEVICEW, EnumDisplayDevicesW,
};
use windows::core::PCWSTR;

/// Helper function to query the current primary display from Windows API
fn get_primary_display_name() -> Result<String, Box<dyn std::error::Error>> {
    let mut dev_num = 0u32;
    loop {
        let mut display_device: DISPLAY_DEVICEW = unsafe { std::mem::zeroed() };
        display_device.cb = std::mem::size_of::<DISPLAY_DEVICEW>() as u32;

        let success =
            unsafe { EnumDisplayDevicesW(PCWSTR::null(), dev_num, &mut display_device, 0) };

        if !success.as_bool() {
            break;
        }

        // Check if this is the primary display (DISPLAY_DEVICE_PRIMARY_DEVICE = 0x00000004)
        if (display_device.StateFlags & DISPLAY_DEVICE_STATE_FLAGS(0x00000004))
            != DISPLAY_DEVICE_STATE_FLAGS(0)
        {
            let len = display_device
                .DeviceName
                .iter()
                .position(|&c| c == 0)
                .unwrap_or(display_device.DeviceName.len());
            let device_name = String::from_utf16_lossy(&display_device.DeviceName[..len]);
            return Ok(device_name);
        }

        dev_num += 1;
    }

    Err("No primary display found".into())
}

#[test]
fn test_change_primary_display_and_restore() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logger for debugging
    let _ = env_logger::builder().is_test(true).try_init();

    // Step 1: Get the current primary display from Windows API
    let original_primary = get_primary_display_name()?;
    println!("Original primary display: {}", original_primary);

    // Step 2: Query all displays
    let display_set = query_displays()?;
    println!("Discovered displays:\n{}", display_set);

    // Verify we have at least 2 displays for this test
    let display_count = display_set.displays().count();
    if display_count < 2 {
        println!(
            "Skipping test: Need at least 2 displays, found {}",
            display_count
        );
        return Ok(());
    }

    // Step 3: Find a different display to set as primary
    let new_primary_display = display_set
        .displays()
        .find(|d| d.name() != original_primary)
        .expect("Should find at least one other display");

    let new_primary_name = new_primary_display.name().to_string();
    println!("Changing primary to: {}", new_primary_name);

    // Step 4: Set the new primary display
    new_primary_display.set_primary()?;
    display_set.apply()?;
    refresh()?;

    // Step 5: Verify the change with Windows API
    let current_primary = get_primary_display_name()?;
    println!("Current primary after change: {}", current_primary);
    assert_eq!(
        current_primary, new_primary_name,
        "Primary display should have changed to {}",
        new_primary_name
    );

    // Step 6: Restore the original primary display
    println!("Restoring original primary: {}", original_primary);

    // Re-query displays to get fresh state
    let display_set = query_displays()?;

    let original_display = display_set
        .displays()
        .find(|d| d.name() == original_primary)
        .expect("Should find original primary display");

    original_display.set_primary()?;
    display_set.apply()?;
    refresh()?;

    // Step 7: Verify restoration with Windows API
    let restored_primary = get_primary_display_name()?;
    println!("Primary after restoration: {}", restored_primary);
    assert_eq!(
        restored_primary, original_primary,
        "Primary display should be restored to {}",
        original_primary
    );

    println!("Test passed: Successfully changed and restored primary display");
    Ok(())
}
