use std::fmt::Display;

use super::basic::{Point, TOL};
use super::line::LineSegment;

pub struct Polygon {
    lines: Vec<LineSegment>,
}

impl Display for Polygon {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut str_buf = String::new();
        str_buf.push_str("[");
        for line in &self.lines {
            str_buf.push_str(&format!("{line} ,"))
        }
        str_buf.push_str("]");
        write!(f, "{str_buf}")
    }
}

impl Polygon {
    pub fn new(points: Vec<Point>) -> Self {
        if points.len() < 3 || (points.len() == 3 && points[0].distance_to(&points[2]) < TOL) {
            panic!(
                "At least 3 points are necessary to form a polygon, but got {:?}",
                points
            )
        }
        let mut idx = 0;
        let mut lines = Vec::<LineSegment>::new();
        while idx < points.len() - 1 {
            lines.push(LineSegment::new(
                points[idx].clone(),
                points[idx + 1].clone(),
            ));
            idx += 1;
        }
        // we need to form a closed shape, if the last point is not same as the first point.
        if points[0].distance_to(&points[points.len() - 1]) > TOL {
            lines.push(LineSegment::new(
                points[points.len() - 1].clone(),
                points[0].clone(),
            ));
        }

        Polygon { lines }
    }
}

#[cfg(test)]
impl Polygon {
    pub fn assert_close_to(&self, p: &Polygon, delta: f64) {
        let mut idx_not_match = Vec::<usize>::new();
        for (idx, line) in self.lines.iter().enumerate() {
            if !line.close_to(&p.lines[idx], delta) {
                idx_not_match.push(idx);
            }
        }
        if idx_not_match.is_empty() {
            return;
        }
        let mut error_msg = "Polygon does not match\n".to_owned();
        for idx in idx_not_match {
            error_msg.push_str(&format!("line at idx={idx} does not match\n"));
            error_msg.push_str(&format!(
                "  left  = {}\n  right = {}\n",
                self.lines[idx], p.lines[idx]
            ));
        }
        panic!("{error_msg}");
    }
}

#[cfg(test)]
mod test {
    use super::super::basic::Point;
    use super::*;

    #[test]
    #[should_panic]
    fn polygon_failed_new_two_points() {
        Polygon::new(Vec::from([
            Point { x: 0.0, y: 0.0 },
            Point { x: 1.0, y: 1.0 },
        ]));
    }

    #[test]
    #[should_panic]
    fn polygon_failed_new_three_closed_points() {
        Polygon::new(Vec::from([
            Point { x: 0.0, y: 0.0 },
            Point { x: 1.0, y: 1.0 },
            Point { x: 0.0, y: 0.0 },
        ]));
    }

    #[test]
    fn polygon_new_points_not_closed() {
        let polygon = Polygon::new(Vec::from([
            Point { x: 0.0, y: 0.0 },
            Point { x: 1.0, y: 1.0 },
            Point { x: 1.0, y: 0.0 },
        ]));
        let expected_polygon = Polygon::new(Vec::from([
            Point { x: 0.0, y: 0.0 },
            Point { x: 1.0, y: 1.0 },
            Point { x: 1.0, y: 0.0 },
            Point { x: 0.0, y: 0.0 },
        ]));
        polygon.assert_close_to(&expected_polygon, 1e-10)
    }
}
