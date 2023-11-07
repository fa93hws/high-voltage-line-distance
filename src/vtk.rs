use std::fs;
use std::path::Path;

use crate::data::SuburbData;
use crate::geometry::*;

fn catchment_to_vtk(polygon: &Polygon) -> String {
    let mut vtk_content = "# vtk DataFile Version 1.0\n".to_owned();
    vtk_content.push_str("2D Unstructured Grid of Linear Triangles\n");
    vtk_content.push_str("ASCII\n\n");
    vtk_content.push_str("DATASET POLYDATA\n");
    let vertices = polygon.get_vertices();
    vtk_content.push_str(&format!("POINTS {} float\n", vertices.len()));
    for point in vertices.iter() {
        vtk_content.push_str(&format!("{}  {}  0.0\n", point.x, point.y))
    }
    vtk_content.push_str("\n");
    vtk_content.push_str(&format!("POLYGONS 1 {}\n", vertices.len() + 2));
    vtk_content.push_str(&format!("{}  ", vertices.len() + 1));
    for i in 0..vertices.len() {
        vtk_content.push_str(&format!("{}  ", i));
    }
    vtk_content.push_str("0\n");
    vtk_content
}

fn high_voltages_to_vtk(lines: &Vec<PolyLine>) -> String {
    let mut vtk_content = "# vtk DataFile Version 1.0\n".to_owned();
    vtk_content.push_str("2D Unstructured Grid of Linear Triangles\n");
    vtk_content.push_str("ASCII\n\n");
    vtk_content.push_str("DATASET POLYDATA\n");
    let vertices = lines
        .iter()
        .map(|line| line.get_vertices())
        .collect::<Vec<Vec<Point>>>();
    let vertices_count = vertices.iter().fold(0, |sum, v| sum + v.len());
    vtk_content.push_str(&format!("POINTS {} float\n", vertices_count));
    for vertices_for_polyline in vertices.iter() {
        for point in vertices_for_polyline.iter() {
            vtk_content.push_str(&format!("{}  {}  0.0\n", point.x, point.y))
        }
    }
    vtk_content.push_str("\n");

    let lines_count = vertices.len();
    vtk_content.push_str(&format!(
        "LINES {} {}\n",
        lines_count,
        vertices_count + lines_count
    ));
    let mut offset = 0_usize;
    for i in 0..vertices.len() {
        vtk_content.push_str(&format!("{}  ", vertices[i].len()));
        for _ in 0..vertices[i].len() {
            vtk_content.push_str(&format!("{}  ", offset));
            offset += 1;
        }
        vtk_content.push_str("\n");
    }
    vtk_content.push_str("\n");
    vtk_content
}

fn circle_to_vtk(origin: &Point, radius: f64) -> String {
    const SAMPLE_SIZE: usize = 64;
    let points: [Point; SAMPLE_SIZE] = core::array::from_fn(|idx| {
        2.0 * (idx as f64) * std::f64::consts::PI / (SAMPLE_SIZE as f64)
    })
    .map(|angle| Point {
        x: origin.x + angle.cos() * radius,
        y: origin.y + angle.sin() * radius,
    });
    catchment_to_vtk(&Polygon::new(Vec::from(points)))
}

pub fn export_suburb_to_vtk(dir: &Path, data: &Vec<SuburbData>, point_for_test: &Point) {
    fs::create_dir_all(dir).unwrap();
    for suburb in data {
        let catchment_content = catchment_to_vtk(&suburb.catchment);
        fs::write(
            dir.join(format!("{}_catchment.vtk", suburb.name)),
            catchment_content,
        )
        .expect("Unable to write file");
        let high_voltage_lines_content = high_voltages_to_vtk(&suburb.high_voltage_lines);
        fs::write(
            dir.join(format!("{}_high_voltage.vtk", suburb.name)),
            high_voltage_lines_content,
        )
        .expect("Unable to write file");
    }
    fs::write(
        dir.join("address_100m.vtk"),
        circle_to_vtk(&point_for_test, 100.0),
    )
    .expect("Unable to write file");
    fs::write(
        dir.join("address_200m.vtk"),
        circle_to_vtk(&point_for_test, 200.0),
    )
    .expect("Unable to write file");
}
