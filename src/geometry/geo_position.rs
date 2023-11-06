use lazy_static::lazy_static;

use crate::geometry::basic::Point;

const EARTH_RADIUS_KM: f64 = 6371.0710;

lazy_static! {
    static ref SYDNEY_CENTRAL_POSITION: GeoPosition = GeoPosition {
        latitude_radius: -33.88243560003056_f64.to_radians(),
        longitude_radius: 151.2064118987779_f64.to_radians(),
    };
}
pub struct GeoPosition {
    pub latitude_radius: f64,
    pub longitude_radius: f64,
}

impl GeoPosition {
    pub fn to_cartesian(&self) -> Point {
        // x, longitude
        let x = {
            let radius = EARTH_RADIUS_KM * 1000.0 * self.latitude_radius.cos();
            radius * (self.longitude_radius - SYDNEY_CENTRAL_POSITION.longitude_radius)
        };
        let y = EARTH_RADIUS_KM
            * 1000.0
            * (self.latitude_radius - SYDNEY_CENTRAL_POSITION.latitude_radius);

        Point { x, y }
    }
}

#[cfg(test)]
mod test {
    use crate::geometry::basic::Point;

    use super::GeoPosition;

    #[test]
    fn latitude_to_cartesian_rosebery() {
        // 919 botany
        let random_place = GeoPosition {
            latitude_radius: -33.921119441679096_f64.to_radians(),
            longitude_radius: 151.1984099658811_f64.to_radians(),
        };
        let cartesian = random_place.to_cartesian();
        println!("{cartesian}");
        cartesian.assert_close_to(
            // from https://www.lddgo.net/convert/distance
            &Point {
                x: -738.6767504988909,
                y: -4301.452856541296,
            },
            1.0,
        )
    }

    #[test]
    fn latitude_to_cartesian_hills() {
        let random_place = GeoPosition {
            latitude_radius: -33.71703513789143_f64.to_radians(),
            longitude_radius: 150.9600398224331_f64.to_radians(),
        };
        let cartesian = random_place.to_cartesian();
        println!("{cartesian}");
        cartesian.assert_close_to(
            // from https://www.lddgo.net/convert/distance
            &Point {
                x: -22787.159185405268,
                y: 18391.717575661813,
            },
            1.0,
        )
    }

    #[test]
    fn latitude_to_cartesian_beacon_mainly() {
        let random_place = GeoPosition {
            latitude_radius: -33.79284455124619_f64.to_radians(),
            longitude_radius: 151.2877152046721_f64.to_radians(),
        };
        let cartesian = random_place.to_cartesian();
        println!("{cartesian}");
        cartesian.assert_close_to(
            // from https://www.lddgo.net/convert/distance
            &Point {
                x: 7513.165838856347,
                y: 9962.083813063693,
            },
            1.0,
        )
    }
}
