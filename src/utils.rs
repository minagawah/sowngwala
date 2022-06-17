#[cfg(test)]
extern crate approx_eq;
#[cfg(test)]
use crate::time::Month;

use crate::time::{ Date, julian_day };

/// Example
/// ```rust
/// use approx_eq::assert_approx_eq;
/// use sowngwala::utils::carry_over;
///
/// let (res, up) = carry_over(59.0, 60.0);
/// assert_eq!(res, 59.0);
/// assert_eq!(up, 0.0);
///
/// let (res, up) = carry_over(60.0, 60.0);
/// assert_eq!(res, 0.0);
/// assert_eq!(up, 1.0);
///
/// let (res, up) = carry_over(120.0, 60.0);
/// assert_eq!(res, 0.0);
/// assert_eq!(up, 2.0);
///
/// let (res, up) = carry_over(121.0, 60.0);
/// assert_eq!(res, 1.0);
/// assert_eq!(up, 2.0);
///
/// let (res, up) = carry_over(120.1, 60.0);
/// assert_approx_eq!(res, 0.1, 1e-1);
/// assert_eq!(up, 2.0);
///
/// let (res, up) = carry_over(-60.0, 60.0);
/// assert_eq!(res, 0.0);
/// assert_eq!(up, -1.0);
///
/// let (res, up) = carry_over(-120.0, 60.0);
/// assert_eq!(res, 0.0);
/// assert_eq!(up, -2.0);
///
/// let (res, up) = carry_over(-59.0, 60.0);
/// assert_eq!(res, 1.0);
/// assert_eq!(up, -1.0);
///
/// let (res, up) = carry_over(-61.0, 60.0);
/// assert_eq!(res, 59.0);
/// assert_eq!(up, -2.0);
/// 
/// let (res, up) = carry_over(-60.1, 60.0);
/// assert_approx_eq!(res, 59.9, 1e-1);
/// assert_eq!(up, -2.0);
/// ```
pub fn carry_over(value: f64, target: f64) -> (f64, f64) {
    let mut quotient = value.abs() / target;

    quotient = if value < 0.0 {
        quotient.ceil()
    } else {
        quotient.floor()
    };

    let largest = target * quotient;

    let result = if value < 0.0 {
        value + largest
    } else {
        value - largest
    };

    if value < 0.0 && quotient != 0.0 {
        quotient = -quotient;
    }

    (result, quotient)
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
