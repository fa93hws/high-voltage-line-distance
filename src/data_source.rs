use regex::Regex;
use std::collections::HashMap;

use crate::api::geocode::Address;
use crate::api::property_data_map::SelectSuburbResponse;
use crate::geometry;

// suburb id is for propertydatamap.com
pub fn init_postcode_to_suburb_id(
    raw_suburb_map: HashMap<String, [String; 4]>,
) -> HashMap<u16, u16> {
    let mut postcode_to_code = HashMap::<u16, u16>::new();
    for (code_str, raw_suburb_code_info) in raw_suburb_map {
        let code = match code_str.parse::<u16>() {
            Ok(val) => val,
            Err(e) => panic!(
                "failed to parse code in raw_suburb_map, expect u16 but got '{}'\n{}",
                code_str, e
            ),
        };
        // some suburb doesn't have a postcode. we don't care about them
        if raw_suburb_code_info[1] == "None" {
            continue;
        }
        let postcode = match raw_suburb_code_info[1].parse::<u16>() {
            Ok(val) => val,
            Err(e) => panic!(
                "failed to parse suburb_postcode in raw_suburb_map, expect u16 but got '{}'\n{}",
                raw_suburb_code_info[1], e
            ),
        };
        postcode_to_code.insert(postcode, code);
    }
    postcode_to_code
}

pub fn parse_address(address: Address) -> (u16, geometry::Point) {
    let re = Regex::new(r#"(\d{4})[, ]+Australia"#).expect("wrong regex pattern typed");
    let captures = re
        .captures(&address.full_address)
        .ok_or(&format!(
            "failed to capture post code from the address '{}'",
            address.full_address
        ))
        .unwrap();
    let postcode_str = match captures.get(1) {
        Some(val) => val.as_str(),
        None => panic!(
            "failed to capture post code from the address '{}' at idx=1",
            address.full_address
        ),
    };
    let postcode = match postcode_str.parse::<u16>() {
        Ok(val) => val,
        Err(e) => panic!("failed to parse post code '{}' to u16\n{}", postcode_str, e),
    };
    (
        postcode,
        geometry::GeoPosition {
            latitude_radius: address.latitude_degree.to_radians(),
            longitude_radius: address.longitude_degree.to_radians(),
        }
        .to_cartesian(),
    )
}

fn raw_position_to_point(latitude_degree: f64, longitude_degree: f64) -> geometry::Point {
    geometry::GeoPosition {
        latitude_radius: latitude_degree.to_radians(),
        longitude_radius: longitude_degree.to_radians(),
    }
    .to_cartesian()
}

pub fn parse_high_voltage_lines(
    raw: SelectSuburbResponse,
) -> HashMap<u16, Vec<geometry::PolyLine>> {
    let mut high_voltage_lines = HashMap::<u16, Vec<geometry::PolyLine>>::new();
    let lines_map = raw.selected_lat_lon;
    let voltages_map = raw.selected_popup_info;
    for (line_id, line) in lines_map {
        if line.r#type != "LineString" {
            panic!(
                "only LineString is supported for lines, but got '{}'",
                line.r#type
            );
        }
        let points = line
            .coordinates
            .into_iter()
            .map(|p| raw_position_to_point(p[1], p[0]))
            .collect::<Vec<geometry::Point>>();
        let voltage_str = match voltages_map.get(&line_id) {
            Some(val) => val,
            None => panic!(
                "can not find voltage for id='{line_id}' in the map {:?}",
                voltages_map
            ),
        };
        if voltage_str.len() > 1 {
            panic!(
                "only 1 voltage should be in the map, but got '{:?}'",
                voltage_str
            )
        }
        let voltage = match voltage_str[0].replace("kV", "").parse::<u16>() {
            Ok(val) => val,
            Err(e) => panic!(
                "failed to parsed voltage string '{}' to u8\n{}",
                voltage_str[0], e
            ),
        };
        match high_voltage_lines.get_mut(&voltage) {
            Some(arr) => arr.push(geometry::PolyLine::new(points)),
            None => {
                high_voltage_lines.insert(voltage, vec![geometry::PolyLine::new(points)]);
            }
        }
    }
    high_voltage_lines
}

#[cfg(test)]
mod tests_init_suburbs {
    use super::*;

    #[test]
    fn success() {
        let raw_suburb_map: HashMap<String, [String; 4]> = HashMap::from([
            (
                "3900".to_owned(),
                [
                    "CHERRYBROOK".to_owned(),
                    "2126".to_owned(),
                    "-33.72185040017101".to_owned(),
                    "151.04624440456263".to_owned(),
                ],
            ),
            (
                "371".to_owned(),
                [
                    "WEST RYDE".to_owned(),
                    "2114".to_owned(),
                    "33.80736158843438".to_owned(),
                    "151.08385175565996".to_owned(),
                ],
            ),
        ]);
        let map = init_postcode_to_suburb_id(raw_suburb_map);
        assert_eq!(map, HashMap::from([(2126, 3900), (2114, 371)]))
    }

    #[test]
    fn success_with_none_postcode() {
        let raw_suburb_map: HashMap<String, [String; 4]> = HashMap::from([
            (
                "3900".to_owned(),
                [
                    "CHERRYBROOK".to_owned(),
                    "2126".to_owned(),
                    "-33.72185040017101".to_owned(),
                    "151.04624440456263".to_owned(),
                ],
            ),
            (
                "3775".to_owned(),
                [
                    "BLUE MOUNTAINS NATIONAL PARK".to_owned(),
                    "None".to_owned(),
                    "-33.90908096333183".to_owned(),
                    "150.35316571753296".to_owned(),
                ],
            ),
        ]);
        let map = init_postcode_to_suburb_id(raw_suburb_map);
        assert_eq!(map, HashMap::from([(2126, 3900)]))
    }

    #[test]
    #[should_panic]
    fn throw_on_non_number_code() {
        let raw_suburb_map: HashMap<String, [String; 4]> = HashMap::from([(
            "abcd".to_owned(),
            [
                "CHERRYBROOK".to_owned(),
                "2126".to_owned(),
                "-33.72185040017101".to_owned(),
                "151.04624440456263".to_owned(),
            ],
        )]);
        init_postcode_to_suburb_id(raw_suburb_map);
    }

    #[test]
    #[should_panic]
    fn throw_on_non_number_postcode() {
        let raw_suburb_map: HashMap<String, [String; 4]> = HashMap::from([(
            "3900".to_owned(),
            [
                "CHERRYBROOK".to_owned(),
                "abcd".to_owned(),
                "-33.72185040017101".to_owned(),
                "151.04624440456263".to_owned(),
            ],
        )]);
        init_postcode_to_suburb_id(raw_suburb_map);
    }
}

#[cfg(test)]
mod test_parse_address {
    use super::*;

    #[test]
    fn success() {
        let address = Address {
            full_address: "30, Franklin Road, Cherrybrook, Sydney, The Council of the Shire of Hornsby, New South Wales, 2126, Australia".to_owned(),
            latitude_degree: -33.921119441679096,
            longitude_degree: 151.1984099658811,
        };
        let (postcode, location) = parse_address(address);
        assert_eq!(postcode, 2126);
        location.assert_close_to(
            &geometry::Point {
                x: -738.6767504988909,
                y: -4301.452856541296,
            },
            1.0,
        )
    }
}

#[cfg(test)]
mod test_parse_high_voltage_lines {
    use super::*;
    use crate::{
        api::property_data_map::SelectedLatLon,
        geometry::{Point, PolyLine},
    };

    #[test]
    fn success() {
        let raw_response = SelectSuburbResponse {
            selected_lat_lon: HashMap::from([
                (
                    "512".to_owned(),
                    SelectedLatLon {
                        r#type: "LineString".to_owned(),
                        coordinates: vec![
                            [151.1984099658811, -33.921119441679096, 0.0],
                            [150.9600398224331, -33.71703513789143, 0.0],
                        ],
                    },
                ),
                (
                    "1024".to_owned(),
                    SelectedLatLon {
                        r#type: "LineString".to_owned(),
                        coordinates: vec![
                            [151.2877152046721, -33.79284455124619, 0.0],
                            [150.9600398224331, -33.71703513789143, 0.0],
                        ],
                    },
                ),
                (
                    "2048".to_owned(),
                    SelectedLatLon {
                        r#type: "LineString".to_owned(),
                        coordinates: vec![
                            [151.1984099658811, -33.921119441679096, 0.0],
                            [151.2877152046721, -33.79284455124619, 0.0],
                        ],
                    },
                ),
            ]),
            selected_popup_info: HashMap::from([
                ("512".to_owned(), vec!["123kV".to_owned()]),
                ("1024".to_owned(), vec!["123kV".to_owned()]),
                ("2048".to_owned(), vec!["66kV".to_owned()]),
            ]),
        };
        let high_voltage_lines = parse_high_voltage_lines(raw_response);
        let v66kv = high_voltage_lines.get(&66).unwrap();
        assert_eq!(v66kv.len(), 1);
        v66kv[0].assert_close_to(
            &PolyLine::new(vec![
                Point {
                    x: -738.6767504988909,
                    y: -4301.452856541296,
                },
                Point {
                    x: 7513.165838856347,
                    y: 9962.083813063693,
                },
            ]),
            1.0,
        );
        let v123kv = high_voltage_lines.get(&123).unwrap();
        assert_eq!(v123kv.len(), 2);
        // line 512
        let line_512_idx = if v123kv[0].get_vertices()[0].close_to(
            &Point {
                x: -738.6767504988909,
                y: -4301.452856541296,
            },
            1.0,
        ) {
            0
        } else {
            1
        };
        let line_1024_idx = 1 - line_512_idx;
        v123kv[line_512_idx].assert_close_to(
            &PolyLine::new(vec![
                Point {
                    x: -738.6767504988909,
                    y: -4301.452856541296,
                },
                Point {
                    x: -22787.159185405268,
                    y: 18391.717575661813,
                },
            ]),
            1.0,
        );
        v123kv[line_1024_idx].assert_close_to(
            &PolyLine::new(vec![
                Point {
                    x: 7513.165838856347,
                    y: 9962.083813063693,
                },
                Point {
                    x: -22787.159185405268,
                    y: 18391.717575661813,
                },
            ]),
            1.0,
        );
    }

    #[test]
    #[should_panic(expected = "failed to parsed voltage string '123KV' to u8")]
    fn failed_with_wrong_voltage_format() {
        let raw_response = SelectSuburbResponse {
            selected_lat_lon: HashMap::from([(
                "512".to_owned(),
                SelectedLatLon {
                    r#type: "LineString".to_owned(),
                    coordinates: vec![[0.0, 0.0, 0.0], [0.01, 0.01, 0.0]],
                },
            )]),
            selected_popup_info: HashMap::from([("512".to_owned(), vec!["123KV".to_owned()])]),
        };
        parse_high_voltage_lines(raw_response);
    }

    #[test]
    #[should_panic(expected = "only 1 voltage should be in the map, but got")]
    fn failed_with_multiple_voltage() {
        let raw_response = SelectSuburbResponse {
            selected_lat_lon: HashMap::from([(
                "512".to_owned(),
                SelectedLatLon {
                    r#type: "LineString".to_owned(),
                    coordinates: vec![[0.0, 0.0, 0.0], [0.01, 0.01, 0.0]],
                },
            )]),
            selected_popup_info: HashMap::from([(
                "512".to_owned(),
                vec!["123KV".to_owned(), "123KV".to_owned()],
            )]),
        };
        parse_high_voltage_lines(raw_response);
    }

    #[test]
    #[should_panic(expected = "only LineString is supported for lines, but got 'PolyLine'")]
    fn failed_with_unsupported_type() {
        let raw_response = SelectSuburbResponse {
            selected_lat_lon: HashMap::from([(
                "512".to_owned(),
                SelectedLatLon {
                    r#type: "PolyLine".to_owned(),
                    coordinates: vec![[0.0, 0.0, 0.0], [0.01, 0.01, 0.0]],
                },
            )]),
            selected_popup_info: HashMap::from([("512".to_owned(), vec!["123kV".to_owned()])]),
        };
        parse_high_voltage_lines(raw_response);
    }

    #[test]
    #[should_panic(expected = "can not find voltage for id='512' in the map")]
    fn failed_with_missing_line() {
        let raw_response = SelectSuburbResponse {
            selected_lat_lon: HashMap::from([(
                "512".to_owned(),
                SelectedLatLon {
                    r#type: "LineString".to_owned(),
                    coordinates: vec![[0.0, 0.0, 0.0], [0.01, 0.01, 0.0]],
                },
            )]),
            selected_popup_info: HashMap::from([("1024".to_owned(), vec!["123kV".to_owned()])]),
        };
        parse_high_voltage_lines(raw_response);
    }
}
