use super::{basic::Point, line::LineSegment};

pub struct PolyLine {
    lines: Vec<LineSegment>,
}

impl PolyLine {
    pub fn new(points: Vec<Point>) -> Self {
        if points.len() < 2 {
            panic!(
                "need at least two points to form a polyline, got '{:?}'",
                points
            )
        }
        let mut lines = Vec::<LineSegment>::new();
        let mut idx = 0;
        while idx < points.len() - 1 {
            lines.push(LineSegment {
                a: points[idx].clone(),
                b: points[idx + 1].clone(),
            });
            idx += 1;
        }
        PolyLine { lines }
    }

    pub fn distance_to(&self, point: &Point) -> f64 {
        self.lines.iter().fold(f64::INFINITY, |a, line| {
            a.min(line.distance_to_point(&point))
        })
    }

    pub fn get_vertices(&self) -> Vec<Point> {
        let mut points = self
            .lines
            .iter()
            .map(|line| line.a.clone())
            .collect::<Vec<Point>>();
        points.push(self.lines.last().unwrap().b.clone());
        points
    }
}

#[cfg(test)]
impl PolyLine {
    pub fn assert_close_to(&self, p: &PolyLine, delta: f64) {
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
    use crate::geometry::basic::test_utils::assert_close_to;

    use super::*;

    #[test]
    fn polyline_distance_to_point() {
        let polyline = PolyLine::new(Vec::from([
            Point { x: 0.0, y: 1.0 },
            Point { x: 0.0, y: 0.0 },
            Point { x: 1.0, y: 0.0 },
        ]));
        let min_distance = polyline.distance_to(&Point { x: -0.5, y: 0.5 });
        assert_close_to(min_distance, 0.5, 1e-10);
    }

    #[test]
    #[should_panic]
    fn polyline_failed_one_point() {
        PolyLine::new(Vec::from([Point { x: 0.0, y: 1.0 }]));
    }

    #[test]
    fn polyline_success_two_points() {
        PolyLine::new(Vec::from([
            Point { x: 0.0, y: 1.0 },
            Point { x: 0.0, y: 0.0 },
        ]));
    }
}
