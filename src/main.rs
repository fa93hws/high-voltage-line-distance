use crate::data::get_data;

mod data;
mod geometry;

const DATA: &str = include_str!("./data/data.json");

fn main() {
    let suburb_data = get_data(DATA);
    println!("{}\n{}", suburb_data[0].name, suburb_data[0].catchment);
}
