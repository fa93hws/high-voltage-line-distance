use crate::{data::get_data, geometry::geo_position::GeoPosition};

mod data;
mod geometry;

const DATA: &str = include_str!("./data/data.json");

fn main() {
    let suburb_data = get_data(DATA);
    let test_location = GeoPosition {
        latitude_radius: -33.72796860923377_f64.to_radians(),
        longitude_radius: 151.03774087535038_f64.to_radians(),
    }
    .to_cartesian();

    let distances = suburb_data[0]
        .high_voltage_lines
        .iter()
        .fold(f64::INFINITY, |a, polyline| {
            a.min(polyline.distance_to(&test_location))
        });
    println!("{:#?}", distances)
}
