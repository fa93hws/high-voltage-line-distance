pub const TOL: f64 = 1e-14;
use std::fmt::Display;

#[derive(Debug)]
pub struct Point {
    pub x: f64,
    pub y: f64,
}

impl Point {
    pub fn distance_to(&self, p: &Point) -> f64 {
        let delta_x = p.x - self.x;
        let delta_y = p.y - self.y;
        f64::sqrt(delta_x * delta_x + delta_y * delta_y)
    }
}

impl Clone for Point {
    fn clone(&self) -> Self {
        Point {
            x: self.x,
            y: self.y,
        }
    }
}

impl Display for Point {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}, {}]", self.x, self.y)
    }
}

#[cfg(test)]
impl Point {
    pub fn close_to(&self, p: &Point, delta: f64) -> bool {
        self.x - p.x < delta && p.x - self.x < delta && self.y - p.y < delta && p.y - self.y < delta
    }
    pub fn assert_close_to(&self, p: &Point, delta: f64) {
        if !self.close_to(p, delta) {
            panic!("left is not close to right with delta '{delta}'\nleft  = {self}\nright = {p}")
        }
    }
}

pub struct Vector {
    pub x: f64,
    pub y: f64,
}

impl Vector {
    pub fn from_points(from: &Point, to: &Point) -> Self {
        Vector {
            x: to.x - from.x,
            y: to.y - from.y,
        }
    }

    pub fn cross(&self, v: &Vector) -> f64 {
        self.x * v.y - self.y * v.x
    }

    pub fn det(&self) -> f64 {
        f64::sqrt(self.x * self.x + self.y * self.y)
    }
}

#[cfg(test)]
pub mod test_utils {
    pub fn assert_close_to(left: f64, right: f64, delta: f64) {
        if f64::is_nan(left) {
            panic!("left is nan")
        }
        if f64::is_nan(right) {
            panic!("right is nan")
        }
        if left - right > delta || right - left > delta {
            panic!(
                "left is not close to right with delta '{delta}'\nleft  = {left}\nright = {right}"
            )
        }
    }
}

#[cfg(test)]
mod test_point {
    use super::*;

    #[test]
    fn distance_to_a_point() {
        let point = Point { x: 1.0, y: 2.0 };
        let distance = point.distance_to(&Point { x: 0.0, y: 0.0 });
        test_utils::assert_close_to(distance, 2.23606797749979, TOL)
    }

    #[test]
    fn distance_to_itself() {
        let point = Point { x: 1.0, y: 2.0 };
        let distance = point.distance_to(&Point { x: 1.0, y: 2.0 });
        test_utils::assert_close_to(distance, 0.0, TOL)
    }
}

#[cfg(test)]
mod test_vector {
    use super::*;

    #[test]
    fn vector_cross_product() {
        let v1 = Vector { x: 13.0, y: 8.0 };
        let v2 = Vector { x: -1.0, y: 2.0 };
        let product = v1.cross(&v2);
        test_utils::assert_close_to(product, 34.0, TOL)
    }

    #[test]
    fn vector_det() {
        let v = Vector { x: 3.0, y: 4.0 };
        test_utils::assert_close_to(v.det(), 5.0, TOL)
    }
}
