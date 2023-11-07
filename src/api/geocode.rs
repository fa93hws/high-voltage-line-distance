use serde::Deserialize;

#[derive(Deserialize, Debug)]
struct GeoCodeResponse {
    lat: String,
    lon: String,
    display_name: String,
}

pub struct Address {
    pub full_address: String,
    pub latitude_degree: f64,
    pub longitude_degree: f64,
}

pub fn find_address(address: &str) -> Address {
    let url = format!(
        "https://geocode.maps.co/search?q={}",
        address.replace(" ", "+")
    );
    trace!("fetch '{}' to find geo location", url);
    let resp = reqwest::blocking::get(url)
        .unwrap()
        .json::<Vec<GeoCodeResponse>>()
        .unwrap();
    if resp.len() > 1 {
        warn!(
            "more than one results found for address '{}', results are: '{:#?}'.",
            address, resp
        );
        warn!("The first address will be used, if it's not expected, please specify more specific address");
    } else if resp.is_empty() {
        panic!("no result found for address '{}'", address);
    }
    let latitude = resp[0].lat.parse::<f64>().expect(&format!(
        "failed to parse latitude from the response to float, got '{}'",
        resp[0].lat
    ));
    let longitude = resp[0].lon.parse::<f64>().expect(&format!(
        "failed to parse longitude from the response to float, got '{}'",
        resp[0].lon
    ));
    trace!(
        "address found as '{}' at {}, {}",
        resp[0].display_name,
        latitude,
        longitude
    );
    Address {
        full_address: resp[0].display_name.to_owned(),
        latitude_degree: latitude,
        longitude_degree: longitude,
    }
}
