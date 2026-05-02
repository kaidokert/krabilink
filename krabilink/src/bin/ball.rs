use clap::Parser;
use krabilink::{load_chart, run_simulation};
use log::LevelFilter;
use std::{fs::File, io::Read};

#[derive(Parser)]
#[clap(version = "1.0")]
struct Args {
    /// Verbosity level (-v, -vv, -vvv)
    #[clap(short, long, action = clap::ArgAction::Count)]
    verbose: u8,

    /// Log level (debug, trace, info, warning, error)
    #[clap(long, value_parser = ["error", "warning", "info", "debug", "trace"])]
    loglevel: Option<String>,

    /// Simulation timestep in seconds
    #[clap(long, default_value = "1.0")]
    dt: f32,

    /// Simulation duration in seconds
    #[clap(short, long, default_value = "10.0")]
    duration: f32,

    /// File to load
    #[clap(short, long)]
    file: Option<String>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    let log_level = if let Some(level) = args.loglevel {
        match level.as_str() {
            "error" => LevelFilter::Error,
            "warning" => LevelFilter::Warn,
            "info" => LevelFilter::Info,
            "debug" => LevelFilter::Debug,
            "trace" => LevelFilter::Trace,
            _ => LevelFilter::Info,
        }
    } else {
        match args.verbose {
            0 => LevelFilter::Error,
            1 => LevelFilter::Warn,
            2 => LevelFilter::Info,
            3 => LevelFilter::Debug,
            _ => LevelFilter::Trace,
        }
    };

    env_logger::Builder::new().filter_level(log_level).init();

    // Calculate number of steps based on duration and timestep
    let num_steps = (args.duration / args.dt).ceil() as usize;

    let file_name = args.file.unwrap_or("ball.json".to_string());
    let mut file = File::open(file_name).expect("Failed to open config file");
    let mut buf = String::new();
    file.read_to_string(&mut buf)?;

    let mut ports = Vec::new();
    let mut chart = load_chart(&buf, &mut ports).unwrap();

    run_simulation(&mut chart, args.dt, num_steps);

    Ok(())
}
