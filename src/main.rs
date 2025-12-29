//! The CLI interface for displayz
//!
//! Use the `--help` flag to see the available options.
use std::cell::RefMut;

use color_eyre::eyre::{Result, eyre};
use displayz::{
    DisplaySettings, FixedOutput, Frequency, Orientation, Position, Resolution, query_displays,
    refresh,
};
use structopt::{StructOpt, clap::ArgGroup};

/// CLI arguments
#[derive(StructOpt, Debug)]
#[structopt(
    name = "display-cli",
    about = "Allows changing display settings on Windows using the CLI."
)]
struct Opts {
    /// Subcommand to run
    #[structopt(subcommand)]
    cmd: SubCommands,
    /// Output debug info
    #[structopt(short, long, global = true)]
    verbose: bool,
}

/// Subcommands to select the mode of operatiom
#[derive(StructOpt, Debug)]
enum SubCommands {
    /// Displays information about a specific display or all displays if no id is provided
    #[structopt(alias = "i")]
    Info {
        /// The id of the display (optional - if not provided, lists all displays)
        #[structopt(short, long)]
        id: Option<usize>,
    },
    /// Sets the primary display
    #[structopt(alias = "sp")]
    SetPrimary {
        #[structopt(short, long)]
        id: usize,
    },
    /// Changes settings of the primary display
    #[structopt(alias = "p")]
    Primary {
        /// The properties to change
        #[structopt(flatten)]
        properties: PropertiesOpt,
    },
    /// Changes settings of a display with a specified id
    #[structopt(alias = "props")]
    Properties {
        /// THe id of the display
        #[structopt(short, long)]
        id: usize,
        /// The properties to change
        #[structopt(flatten)]
        properties: PropertiesOpt,
    },
}

/// Describes the properties that can be changed on a display
#[derive(StructOpt, Debug)]
#[structopt(group = ArgGroup::with_name("prop").required(true).multiple(true))]
struct PropertiesOpt {
    /// Set the position of the display
    #[structopt(
        group = "prop",
        short,
        long,
        long_help = "Set the position of the display. Expected format: `<x>,<y>`"
    )]
    position: Option<Position>,
    /// Sets the resolution of the display
    #[structopt(
        group = "prop",
        short,
        long,
        long_help = "Sets the resolution of the display. Expected format: `<width>x<height>`."
    )]
    resolution: Option<Resolution>,
    /// Sets the orientation of the display
    #[structopt(
        group = "prop",
        short,
        long,
        long_help = "Sets the orientation of the display. One of: `Default`, `UpsideDown`, `Right`, `Left`"
    )]
    orientation: Option<Orientation>,
    /// Sets the fixed output of the display
    #[structopt(
        group = "prop",
        short,
        long,
        long_help = "Sets the fixed output of the display. One of: `Default`, `Stretch`, `Center`."
    )]
    fixed_output: Option<FixedOutput>,
    // Sets the refresh rate of the display
    #[structopt(
        group = "prop",
        short("t"),
        long,
        long_help = "Sets the refresh rate of the display. Expected format: `<n>`."
    )]
    frequency: Option<Frequency>,
}

/// Entry point for `displayz`.
fn main() -> Result<()> {
    color_eyre::install()?;

    let opts = Opts::from_args();

    let log_level = if opts.verbose {
        log::LevelFilter::Trace
    } else {
        log::LevelFilter::Info
    };

    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or(log_level.as_str()))
        .init();

    log::debug!("Parsed Opts:\n{:#?}", opts);

    let display_set = query_displays()?;
    log::debug!("Discovered displays:\n{}", display_set);

    match opts.cmd {
        SubCommands::Info { id } => {
            match id {
                Some(id) => {
                    // Display info for a specific display
                    let display = display_set
                        .get(id)
                        .ok_or_else(|| eyre!("Display with id {} not found", id))?;

                    println!("Display ID: {}", display.index());
                    println!("Name:       {}", display.name());
                    println!("String:     {}", display.string());
                    println!("Key:        {}", display.key());
                    println!("Primary:    {}", display.is_primary());

                    if let Some(settings) = display.settings() {
                        let settings = settings.borrow();
                        println!("\nSettings:");
                        println!("  Position:     {}", settings.position);
                        println!("  Resolution:   {}", settings.resolution);
                        println!("  Orientation:  {}", settings.orientation);
                        println!("  Fixed Output: {}", settings.fixed_output);
                        println!("  Frequency:    {} Hz", settings.frequency);
                    } else {
                        println!("\nSettings:   None (Inactive)");
                    }
                }
                None => {
                    // List all displays
                    println!("All Displays:");
                    println!();
                    for display in display_set.displays() {
                        println!("Display ID: {}", display.index());
                        println!("Name:       {}", display.name());
                        println!("String:     {}", display.string());
                        println!("Key:        {}", display.key());
                        println!("Primary:    {}", display.is_primary());

                        if let Some(settings) = display.settings() {
                            let settings = settings.borrow();
                            println!("Settings:");
                            println!("  Position:     {}", settings.position);
                            println!("  Resolution:   {}", settings.resolution);
                            println!("  Orientation:  {}", settings.orientation);
                            println!("  Fixed Output: {}", settings.fixed_output);
                            println!("  Frequency:    {} Hz", settings.frequency);
                        } else {
                            println!("Settings:   None (Inactive)");
                        }
                        println!();
                    }
                }
            }
        }
        SubCommands::SetPrimary { id } => {
            let display = display_set
                .get(id)
                .ok_or_else(|| eyre!("Display with id {} not found", id))?;

            display.set_primary()?;

            display_set.apply()?;
            refresh()?;
            log::info!("Display settings changed");
        }
        SubCommands::Primary { properties } => {
            let display = display_set.primary();

            if let Some(settings) = display.settings() {
                let mut settings = settings.borrow_mut();
                set_properties(&properties, &mut settings);
            } else {
                Err(eyre!("Primary display has no settings"))?;
            }

            display.apply()?;
            refresh()?;
            log::info!("Display settings changed");
        }
        SubCommands::Properties { id, properties } => {
            let display = display_set
                .get(id)
                .ok_or_else(|| eyre!("Display with id {} not found", id))?;

            if let Some(settings) = display.settings() {
                let mut settings = settings.borrow_mut();
                set_properties(&properties, &mut settings)
            } else {
                Err(eyre!("Display has no settings"))?;
            }

            display.apply()?;
            refresh()?;
            log::info!("Display settings changed");
        }
    }

    Ok(())
}

/// Sets a specific settings from the given properties
macro_rules! assign_if_ok {
    ($properties:expr_2021, $settings:expr_2021, $name:ident) => {
        if let Some(value) = $properties.$name {
            $settings.$name = value;
        }
    };
}

/// Sets all available properties
fn set_properties(properties: &PropertiesOpt, settings: &mut RefMut<DisplaySettings>) {
    assign_if_ok!(properties, settings, position);
    assign_if_ok!(properties, settings, resolution);
    assign_if_ok!(properties, settings, orientation);
    assign_if_ok!(properties, settings, fixed_output);
    assign_if_ok!(properties, settings, frequency);
}
