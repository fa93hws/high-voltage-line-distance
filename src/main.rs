#[macro_use]
extern crate log;
extern crate simplelog;
use clap::Parser;
use data_source::{HighVoltageLine, SuburbInfo};
use simplelog::{ColorChoice, ConfigBuilder, LevelFilter, TermLogger, TerminalMode};
use std::collections::{HashMap, HashSet};

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

fn filter_suburb(
    place: &geometry::Point,
    suburbs: Vec<SuburbInfo>,
    range_m: f64,
) -> Vec<SuburbInfo> {
    suburbs
        .into_iter()
        .filter(|s| s.location.distance_to(&place) < range_m)
        .collect()
}

fn aggregate_high_voltage_lines(
    mut acc: HashMap<u16, Vec<HighVoltageLine>>,
    map: HashMap<u16, Vec<HighVoltageLine>>,
    cached_line_id: &mut HashSet<String>,
) -> HashMap<u16, Vec<HighVoltageLine>> {
    {
        for (k, v) in map {
            let mut lines = Vec::<HighVoltageLine>::new();
            for line in v {
                if cached_line_id.contains(&line.id) {
                    continue;
                }
                cached_line_id.insert(line.id.to_owned());
                lines.push(line);
            }
            match acc.get_mut(&k) {
                Some(existing_lines) => {
                    existing_lines.extend(lines);
                }
                None => {
                    acc.insert(k, lines);
                }
            }
        }
        acc
    }
}

fn main() {
    let args: Args = Args::parse();
    init_logger(args.verbose);
    let raw_suburb_map = api::property_data_map::server_init_init();
    let address = api::geocode::find_address(&args.address);
    let location = data_source::parse_address(address);
    let suburbs_info: Vec<SuburbInfo> = data_source::get_all_suburbs(raw_suburb_map);
    debug!("postcode_to_suburb_id calculated");

    let filtered_suburb_infos = filter_suburb(&location, suburbs_info, 5_000.0);
    let surrounding_suburbs_name = filtered_suburb_infos
        .iter()
        .map(|s| s.name.to_owned())
        .collect::<Vec<String>>();
    debug!(
        "suburbs within 5km filtered: {:?}",
        surrounding_suburbs_name
    );

    let mut cached_line_id = HashSet::<String>::new();
    let high_voltage_lines = filtered_suburb_infos
        .iter()
        .map(|s| api::property_data_map::select_suburb(s.id, &s.name))
        .map(|raw| data_source::parse_high_voltage_lines(raw))
        .fold(HashMap::<u16, Vec<HighVoltageLine>>::new(), |acc, map| {
            aggregate_high_voltage_lines(acc, map, &mut cached_line_id)
        });
    debug!("suburb info parsed");
    // voltage -> distance
    let mut distances = HashMap::<u16, f64>::new();
    let mut voltages = Vec::<u16>::new();
    for (voltage, lines) in high_voltage_lines {
        let distance = lines.iter().fold(f64::INFINITY, |acc, l| {
            acc.min(l.line.distance_to(&location))
        });
        distances.insert(voltage, distance);
        voltages.push(voltage);
    }
    debug!("distances found {:?}", distances);
    print_results(&distances, voltages);
}
