#[cfg(test)]
extern crate approx_eq;
#[cfg(test)]
use crate::time::Month;

use crate::time::{ Date, julian_day };

pub fn reduce_to(given: f64, target: f64) -> (f64, f64) {
    let mut factor: f64 = ((target - given) / target).abs();

    factor = if factor < 1.0 {
        if given < target {
            0.0
        } else {
            factor.ceil()
        }
    } else {
        factor.floor()
    };

    let largest = target * factor;

    let value = if given < 0.0 {
        given + largest
    } else {
        given - largest
    };

    factor = if given < 0.0 {
        - factor
    } else {
        factor
    };

    (value, factor)
}

#[allow(clippy::float_cmp)]
pub fn reduce_to_exclusive(given: f64, target: f64) -> (f64, f64) {
    let (mut value, mut factor) = reduce_to(given, target);

    // TODO: float_cmp
    if value == target {
        value = 0.0;
        factor += 1.0;
    }
    (value, factor)
}

pub fn normalize_angle(value: f64, max: f64) -> f64 {
    let half = max / 2.0;
    let mut angle = value;
    while angle <= -half {
        angle += max;
    }
    while angle > half {
        angle -= max;
    }
    angle
}

/// Returns the obliquity of the ecliptic (ε), the angle between
/// the planes of the equator and the ecliptic, from the given date.
/// (Peter Duffett-Smith, p.41)
#[allow(clippy::many_single_char_names)]
pub fn mean_obliquity_of_the_epliptic(&date: &Date) -> f64 {
    let mut jd = julian_day(&date);
    jd -= 2_451_545.0; // January 1.5, 2000

    let t = jd / 36_525.0;
    let mut delta = (46.815 * t) + (0.0006 * t * t) - (0.001_81 * t * t * t);
    delta /= 3600.0;
    23.439_292 - delta
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx_eq::assert_approx_eq;

    #[test]
    fn reduce_to_works() {
        let (value, _factor) = reduce_to(-465.986_246, 24.0);
        assert_approx_eq!(
            value,
            14.013_754,
            1e-3
        );
    }

    #[test]
    fn mean_obliquity_of_the_epliptic_works() {
        let date = Date {
            year: 1980,
            month: Month::Jan,
            day: 0.0,
        };

        let oblique = mean_obliquity_of_the_epliptic(&date);

        assert_approx_eq!(
            oblique,
            23.441893,
            1e-6
        );
    }
}
