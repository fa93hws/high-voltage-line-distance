use std::fmt::Display;

use super::basic::TOL;
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
    pub fn new(lines: Vec<LineSegment>) -> Self {
        if lines.len() < 3 {
            panic!(
                "At least 3 lines is necessary to form a polygon, but got {}",
                lines.len()
            )
        }

        let mut idx = 1;
        while idx < lines.len() {
            let end = &lines[idx - 1].b;
            let start = &lines[idx].a;
            if end.distance_to(start) > TOL {
                panic!("End point of the previous line must be the start point of the current line\nprevious = {}\ncurrent  = {}", lines[idx-1], lines[idx])
            }
            idx += 1;
        }
        let start = &lines[0].a;
        let end = &lines[lines.len() - 1].b;
        if start.distance_to(end) > TOL {
            panic!("End point of the last line must be the start point of the first line\nfirst = {}\nlast  = {}", lines[0], lines[lines.len() - 1]);
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
    #[should_panic(expected = "At least 3 lines is necessary to form a polygon, but got 2")]
    fn polygon_new_2_lines() {
        Polygon::new(Vec::from([
            LineSegment::new(Point { x: 0.0, y: 0.0 }, Point { x: 1.0, y: 1.0 }),
            LineSegment::new(Point { x: 1.0, y: 1.0 }, Point { x: 2.0, y: 2.0 }),
        ]));
    }

    #[test]
    #[should_panic]
    fn polygon_new_disconnected_middle() {
        Polygon::new(Vec::from([
            LineSegment::new(Point { x: 0.0, y: 0.0 }, Point { x: 1.0, y: 1.0 }),
            LineSegment::new(Point { x: 1.0, y: 1.0 }, Point { x: 1.5, y: 1.5 }),
            LineSegment::new(Point { x: 2.0, y: 1.0 }, Point { x: 0.0, y: 0.0 }),
        ]));
    }

    #[test]
    #[should_panic]
    fn polygon_new_disconnected_end() {
        Polygon::new(Vec::from([
            LineSegment::new(Point { x: 0.0, y: 0.0 }, Point { x: 1.0, y: 1.0 }),
            LineSegment::new(Point { x: 1.0, y: 1.0 }, Point { x: 2.0, y: 2.0 }),
            LineSegment::new(Point { x: 2.0, y: 1.0 }, Point { x: 1.0, y: 0.0 }),
        ]));
    }
}
