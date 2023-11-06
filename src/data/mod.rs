use crate::geometry::{
    basic::Point, geo_position::GeoPosition, line::LineSegment, polygon::Polygon,
    polyline::PolyLine,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Debug)]
struct RawSuburbData {
    suburb_catchment: Vec<[f64; 2]>,
    high_voltage_lines: Vec<Vec<[f64; 3]>>,
}

pub struct SuburbData {
    pub name: String,
    pub catchment: Polygon,
    pub high_voltage_lines: Vec<PolyLine>,
}

fn raw_position_to_point(latitude_degree: f64, longitude_degree: f64) -> Point {
    GeoPosition {
        latitude_radius: latitude_degree.to_radians(),
        longitude_radius: longitude_degree.to_radians(),
    }
    .to_cartesian()
}

fn parse_polygon(raw_points: Vec<[f64; 2]>) -> Polygon {
    if raw_points.len() < 3 {
        panic!("need at least 3 points, but got '{:?}'", raw_points);
    }
    let mut idx = 1;
    let mut lines = Vec::<LineSegment>::new();
    while idx < raw_points.len() {
        let start = raw_points[idx - 1];
        let end = raw_points[idx];
        idx += 1;
        lines.push(LineSegment::new(
            raw_position_to_point(start[1], start[0]),
            raw_position_to_point(end[1], end[0]),
        ));
    }

    Polygon::new(lines)
}

fn parse_high_voltage_lines(raw_value: Vec<Vec<[f64; 3]>>) -> Vec<PolyLine> {
    raw_value
        .into_iter()
        .map(|raw_points| {
            let points = raw_points
                .into_iter()
                .map(|p| raw_position_to_point(p[1], p[0]))
                .collect();
            PolyLine::new(points)
        })
        .collect()
}

pub fn get_data(json_str: &str) -> Vec<SuburbData> {
    let raw_data: HashMap<String, RawSuburbData> = match serde_json::from_str(json_str) {
        Ok(val) => val,
        Err(e) => panic!("failed to parse raw data, {e}"),
    };
    let mut suburb_data = Vec::<SuburbData>::new();
    for (k, v) in raw_data {
        suburb_data.push(SuburbData {
            name: k,
            catchment: parse_polygon(v.suburb_catchment),
            high_voltage_lines: parse_high_voltage_lines(v.high_voltage_lines),
        });
    }
    suburb_data
}

#[cfg(test)]
mod test {
    const TEST_DATA: &str = include_str!("./fixtures/test_data.json");

    use super::*;

    #[test]
    fn parse_demo_data() {
        let points = [
            Point {
                x: -738.6767504988909,
                y: -4301.452856541296,
            },
            Point {
                x: -22787.159185405268,
                y: 18391.717575661813,
            },
            Point {
                x: 7513.165838856347,
                y: 9962.083813063693,
            },
        ];
        let expected_data_foo = SuburbData {
            name: "foo".to_owned(),
            catchment: Polygon::new(Vec::from([
                LineSegment {
                    a: points[0].clone(),
                    b: points[1].clone(),
                },
                LineSegment {
                    a: points[1].clone(),
                    b: points[2].clone(),
                },
                LineSegment {
                    a: points[2].clone(),
                    b: points[0].clone(),
                },
            ])),
            high_voltage_lines: Vec::from([
                PolyLine::new(Vec::from([points[0].clone(), points[1].clone()])),
                PolyLine::new(Vec::from([points[0].clone(), points[2].clone()])),
            ]),
        };
        let expected_data_bar = SuburbData {
            name: "bar".to_owned(),
            catchment: Polygon::new(Vec::from([
                LineSegment {
                    a: points[1].clone(),
                    b: points[2].clone(),
                },
                LineSegment {
                    a: points[2].clone(),
                    b: points[0].clone(),
                },
                LineSegment {
                    a: points[0].clone(),
                    b: points[1].clone(),
                },
            ])),
            high_voltage_lines: Vec::from([
                PolyLine::new(Vec::from([points[0].clone(), points[2].clone()])),
                PolyLine::new(Vec::from([points[0].clone(), points[1].clone()])),
            ]),
        };
        let mut suburb_data = get_data(TEST_DATA);
        assert_eq!(suburb_data.len(), 2);
        if suburb_data[0].name != "foo" {
            suburb_data.reverse();
        }

        let data_foo = &suburb_data[0];
        let data_bar = &suburb_data[1];
        assert_eq!(data_foo.name, "foo");
        assert_eq!(data_bar.name, "bar");
        data_foo
            .catchment
            .assert_close_to(&expected_data_foo.catchment, 1.0);
        data_bar
            .catchment
            .assert_close_to(&expected_data_bar.catchment, 1.0);

        for (i, high_voltage_line) in data_foo.high_voltage_lines.iter().enumerate() {
            high_voltage_line.assert_close_to(&expected_data_foo.high_voltage_lines[i], 1.0)
        }
        for (i, high_voltage_line) in data_bar.high_voltage_lines.iter().enumerate() {
            high_voltage_line.assert_close_to(&expected_data_bar.high_voltage_lines[i], 1.0)
        }
    }
}
