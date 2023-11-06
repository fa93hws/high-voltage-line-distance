use std::path::Path;

use crate::data::get_data;
use crate::geometry::geo_position::GeoPosition;
use crate::vtk::export_suburb_to_vtk;
use clap::Parser;
use colored::Colorize;
use geometry::basic::Point;
use serde::Deserialize;

mod data;
mod geometry;
mod vtk;

const DATA: &str = include_str!("./data/data.json");

#[derive(Deserialize, Debug)]
struct GeoCodeResponse {
    lat: String,
    lon: String,
    display_name: String,
}

fn address_to_location(address: &str) -> Point {
    let url = format!(
        "https://geocode.maps.co/search?q={}",
        address.replace(" ", "+")
    );
    let resp = reqwest::blocking::get(url)
        .unwrap()
        .json::<Vec<GeoCodeResponse>>()
        .unwrap();
    if resp.len() > 1 {
        println!(
            "{}",
            format!(
                "more than one results found for address '{}', results are: '{:#?}'.",
                address, resp
            )
            .yellow()
        );
        println!("{}", "The first address will be used, if it's not expected, please specify more specific address".yellow());
    } else if resp.is_empty() {
        panic!("no result found for address '{}'", address);
    } else {
        println!("address found as '{}'", resp[0].display_name);
    }
    let latitude = resp[0]
        .lat
        .parse::<f64>()
        .expect(&format!(
            "failed to parse latitude from the response to float, got '{}'",
            resp[0].lat
        ))
        .to_radians();
    let longitude = resp[0]
        .lon
        .parse::<f64>()
        .expect(&format!(
            "failed to parse longitude from the response to float, got '{}'",
            resp[0].lon
        ))
        .to_radians();
    GeoPosition {
        latitude_radius: latitude,
        longitude_radius: longitude,
    }
    .to_cartesian()
}

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    address: String,
}

fn main() {
    let args: Args = Args::parse();
    let test_location = address_to_location(&args.address);

    let suburb_data = get_data(DATA);

    export_suburb_to_vtk(&Path::new(".").join("debug"), &suburb_data, &test_location);
    let distance = suburb_data[0]
        .high_voltage_lines
        .iter()
        .fold(f64::INFINITY, |a, polyline| {
            a.min(polyline.distance_to(&test_location))
        });
    let colored_distance = if distance < 90.0 {
        format!("{:.1}", distance).red().bold()
    } else if distance < 200.0 {
        format!("{:.1}", distance).yellow().bold()
    } else {
        format!("{:.1}", distance).green().bold()
    };
    println!(
        "{} meters away from '{}' to high voltage power line",
        colored_distance, args.address
    )
}
