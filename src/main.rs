use crate::data::get_data;

mod data;
mod geometry;

fn main() {
    let suburb_data = get_data();
    println!("{}\n{}", suburb_data[0].name, suburb_data[0].catchment);
}
