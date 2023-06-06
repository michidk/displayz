#[cfg(windows)]
use displayz::{query_displays, refresh, Resolution};

/// Prints and changes the current resolution of the primary display
#[cfg(windows)]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let display_set = query_displays()?;
    println!("Discovered displays:\n{}", display_set);

    if let Some(settings) = display_set.primary().settings() {
        let res = (*settings).borrow().resolution;
        println!("Current resolution: {:?}", res);

        if res.height == 1080 {
            println!("Resolution is 1080p, changing to 720p");
            (*settings).borrow_mut().resolution = Resolution::new(1280, 720);
        } else {
            println!("Resolution is 720p, changing to 1080p");
            (*settings).borrow_mut().resolution = Resolution::new(1920, 1080);
        }
    } else {
        eprintln!("Primary display has no settings");
    }

    display_set.primary().apply()?;
    refresh()?;

    Ok(())
}

#[cfg(not(windows))]
fn main() {}
