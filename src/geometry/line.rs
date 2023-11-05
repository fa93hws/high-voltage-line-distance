use std::fmt::Display;

use super::basic::{Point, Vector, TOL};

pub struct LineSegment {
    pub a: Point,
    pub b: Point,
}

impl Display for LineSegment {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({} -> {})", self.a, self.b)
    }
}

impl LineSegment {
    pub fn new(a: Point, b: Point) -> Self {
        if a.distance_to(&b) < TOL {
            panic!("can not form a line with two point at same coordinate.\npoint0={a}\npoint1={b}\neps={TOL}")
        }
        LineSegment { a, b }
    }

    fn find_projection(&self, point: &Point) -> Point {
        let dy = self.b.y - self.a.y;
        let dx = self.b.x - self.a.x;
        // slop is infinite. e.g. x=c
        if f64::abs(dx) < TOL {
            return Point {
                x: self.a.x,
                y: point.y,
            };
        }
        // y = kx + b
        let k = dy / dx;
        let b = self.a.y - k * self.a.x;
        let p_x = (point.x + k * point.y - k * b) / (1.0 + k * k);
        let p_y = (k * point.x + k * k * point.y + b) / (1.0 + k * k);
        Point { x: p_x, y: p_y }
    }

    fn find_closest_point(&self, point: &Point) -> Point {
        let projection = self.find_projection(point);
        let vec_point_to_projection = Vector::from_points(point, &projection);
        if vec_point_to_projection.det() < TOL {
            // point is on the line, cross product will be zero.
            return projection;
        }
        let vec_projection_to_a = Vector::from_points(&projection, &self.a);
        if vec_projection_to_a.det() < TOL {
            // projection is the end point, cross product will be zero.
            return self.a.clone();
        }
        let vec_projection_to_b = Vector::from_points(&projection, &self.b);
        if vec_projection_to_b.det() < TOL {
            // projection is the end point, cross product will be zero.
            return self.b.clone();
        }
        let cross_a = vec_point_to_projection.cross(&vec_projection_to_a);
        let cross_b = vec_point_to_projection.cross(&vec_projection_to_b);

        if (cross_a > 0.0 && cross_b > 0.0) || (cross_a < 0.0 && cross_b < 0.0) {
            // same sign, meaning the projection is not within the segment
            // closest point will be either a or b, whichever is closer.
            let p_to_a = point.distance_to(&self.a);
            let p_to_b = point.distance_to(&self.b);
            if p_to_a > p_to_b {
                self.b.clone()
            } else {
                self.a.clone()
            }
        } else {
            // different sign, meaning the projection is not within the segment
            projection
        }
    }

    pub fn distance_to_point(&self, p: &Point) -> f64 {
        let closest_point = self.find_closest_point(p);
        closest_point.distance_to(p)
    }
}

#[cfg(test)]
mod test_line_segment {
    use super::*;
    use crate::geometry::basic::test_utils;

    #[test]
    fn find_projection_point_outside() {
        let line_segment = LineSegment::new(Point { x: 0.0, y: 0.0 }, Point { x: 1.0, y: 1.0 });
        let projection = line_segment.find_projection(&Point { x: 0.0, y: 1.0 });
        projection.assert_close_to(&Point { x: 0.5, y: 0.5 }, TOL)
    }

    #[test]
    fn find_projection_vertical_line() {
        let line_segment = LineSegment::new(Point { x: 10.0, y: 0.0 }, Point { x: 10.0, y: 100.0 });
        let projection = line_segment.find_projection(&Point { x: 0.0, y: 100.0 });
        projection.assert_close_to(&Point { x: 10.0, y: 100.0 }, TOL)
    }

    #[test]
    fn find_projection_point_on_line() {
        let line_segment = LineSegment::new(Point { x: 0.0, y: 0.0 }, Point { x: 1.0, y: 1.0 });
        let projection = line_segment.find_projection(&Point { x: 0.5, y: 0.5 });
        projection.assert_close_to(&Point { x: 0.5, y: 0.5 }, TOL)
    }

    #[test]
    fn find_closest_point_projection_on_line() {
        let line_segment = LineSegment::new(Point { x: 0.0, y: 0.0 }, Point { x: 1.0, y: 1.0 });
        let closest_point = line_segment.find_closest_point(&Point { x: 0.0, y: 1.0 });
        closest_point.assert_close_to(&Point { x: 0.5, y: 0.5 }, TOL)
    }

    #[test]
    fn find_closest_point_projection_on_end_point_a() {
        let line_segment = LineSegment::new(Point { x: 0.0, y: 0.0 }, Point { x: 1.0, y: 1.0 });
        let closest_point = line_segment.find_closest_point(&Point { x: 0.0, y: 2.0 });
        closest_point.assert_close_to(&Point { x: 1.0, y: 1.0 }, TOL)
    }

    #[test]
    fn find_closest_point_projection_on_end_point_b() {
        let line_segment = LineSegment::new(Point { x: 0.0, y: 0.0 }, Point { x: 1.0, y: 1.0 });
        let closest_point = line_segment.find_closest_point(&Point { x: -1.0, y: 1.0 });
        closest_point.assert_close_to(&Point { x: 0.0, y: 0.0 }, TOL)
    }

    #[test]
    fn find_closest_point_projection_outside_close_to_point_a() {
        let line_segment = LineSegment::new(Point { x: 0.0, y: 0.0 }, Point { x: 1.0, y: 1.0 });
        let closest_point = line_segment.find_closest_point(&Point { x: 1.0, y: 100.0 });
        closest_point.assert_close_to(&Point { x: 1.0, y: 1.0 }, TOL)
    }

    #[test]
    fn find_closest_point_projection_outside_close_to_point_b() {
        let line_segment = LineSegment::new(Point { x: 0.0, y: 0.0 }, Point { x: 1.0, y: 1.0 });
        let closest_point = line_segment.find_closest_point(&Point { x: -100.0, y: 1.0 });
        closest_point.assert_close_to(&Point { x: 0.0, y: 0.0 }, TOL)
    }
}
