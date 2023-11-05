mod geo_position;

use crate::geometry::{basic::Point, line::LineSegment, polygon::Polygon};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use self::geo_position::GeoPosition;

const DATA: &str = include_str!("./data.json");

#[derive(Serialize, Deserialize, Debug)]
struct RawSuburbData {
    suburb_catchment: Vec<[f64; 2]>,
    high_voltage_lines: Vec<Vec<[f64; 3]>>,
}

pub struct SuburbData {
    pub name: String,
    pub catchment: Polygon,
    pub high_voltage_lines: Vec<Vec<LineSegment>>,
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

fn parse_high_voltage_lines(raw_value: Vec<Vec<[f64; 3]>>) -> Vec<Vec<LineSegment>> {
    let mut high_voltage_lines = Vec::<Vec<LineSegment>>::new();
    for raw_line in raw_value {
        let mut lines = Vec::<LineSegment>::new();
        if raw_line.len() < 2 {
            panic!(
                "need at least two points in power line, got '{:?}'",
                raw_line
            )
        }
        let mut idx = 0;
        while idx < raw_line.len() - 1 {
            lines.push(LineSegment {
                a: raw_position_to_point(raw_line[idx][1], raw_line[idx][0]),
                b: raw_position_to_point(raw_line[idx + 1][1], raw_line[idx + 1][0]),
            });
            idx += 1;
        }
        high_voltage_lines.push(lines);
    }
    high_voltage_lines
}

pub fn get_data() -> Vec<SuburbData> {
    let raw_data: HashMap<String, RawSuburbData> = match serde_json::from_str(DATA) {
        Ok(val) => val,
        Err(e) => panic!("failed to parse raw data, {e}"),
    };
    let mut suburbData = Vec::<SuburbData>::new();
    for (k, v) in raw_data {
        suburbData.push(SuburbData {
            name: k,
            catchment: parse_polygon(v.suburb_catchment),
            high_voltage_lines: parse_high_voltage_lines(v.high_voltage_lines),
        });
    }
    suburbData
}
