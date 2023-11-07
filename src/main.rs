#[macro_use]
extern crate log;
extern crate simplelog;
use clap::Parser;
use simplelog::{ColorChoice, ConfigBuilder, LevelFilter, TermLogger, TerminalMode};
use std::collections::HashMap;

mod api;
mod data_source;
mod geometry;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    address: String,

    #[arg(short, long, default_value_t = false)]
    verbose: bool,
}

fn init_logger(verbose: bool) {
    let log_level = if verbose {
        LevelFilter::Trace
    } else {
        LevelFilter::Info
    };
    let config = ConfigBuilder::new()
        .add_filter_ignore("reqwest".to_owned())
        .build();
    TermLogger::init(log_level, config, TerminalMode::Mixed, ColorChoice::Auto)
        .expect("failed to init logger");
}

fn print_results(distances: &HashMap<u16, f64>, mut voltages: Vec<u16>) {
    let mut min_distance = f64::INFINITY;
    voltages.sort_by(|a, b| b.cmp(a));
    for voltage in voltages {
        let distance = *distances.get(&voltage).unwrap();
        if distance < min_distance {
            min_distance = distance;
            info!("{:.0}m away from {}kV power line", distance, voltage);
        }
    }
}
fn main() {
    let args: Args = Args::parse();
    init_logger(args.verbose);
    let raw_suburb_map = api::property_data_map::server_init_init();
    let postcode_to_suburb_id = data_source::init_postcode_to_suburb_id(raw_suburb_map);
    debug!("postcode_to_suburb_id calculated");
    let address = api::geocode::find_address(&args.address);
    let (postcode, location) = data_source::parse_address(address);
    debug!("address parsed, postcode is '{postcode}'");
    let suburb_id = match postcode_to_suburb_id.get(&postcode) {
        Some(id) => *id,
        None => panic!("can not find suburb id from postcode '{postcode}'"),
    };
    debug!("suburb id found as '{}'", suburb_id);
    let raw = api::property_data_map::select_suburb(suburb_id);
    let high_voltage_lines = data_source::parse_high_voltage_lines(raw);
    debug!("suburb info parsed");
    //     if args.verbose {
    //         export_suburb_to_vtk(&Path::new(".").join("debug"), &suburb_data, &test_location);
    //     }
    // voltage -> distance
    let mut distances = HashMap::<u16, f64>::new();
    let mut voltages = Vec::<u16>::new();
    for (voltage, lines) in high_voltage_lines {
        let distance = lines
            .iter()
            .fold(f64::INFINITY, |a, l: &geometry::PolyLine| {
                a.min(l.distance_to(&location))
            });
        distances.insert(voltage, distance);
        voltages.push(voltage);
    }
    debug!("distances found {:?}", distances);
    print_results(&distances, voltages);
}
