use chrono::naive::NaiveDate;
use chrono::Datelike;

use crate::time::julian_day_from_generic_datetime;

/// Checks if the given value exceeds
/// the given target value.
///
/// Arguments
/// * `value` - Value you want to check.
/// * `base` - Max/min for 'value' to carry over when exceeded.
///
/// Results
/// * `result` - A tuple pair returned.
/// * `result.0` - Value after the calculation.
/// * `result.1` - Value denoting how much did the value exceed.
///
/// Example
/// ```rust
/// use approx_eq::assert_approx_eq;
/// use sowngwala::utils::overflow;
///
/// let (res, up) = overflow(59.0, 60.0);
/// assert_eq!(res, 59.0);
/// assert_eq!(up, 0.0);
///
/// let (res, up) = overflow(60.0, 60.0);
/// assert_eq!(res, 0.0);
/// assert_eq!(up, 1.0);
///
/// let (res, up) = overflow(120.0, 60.0);
/// assert_eq!(res, 0.0);
/// assert_eq!(up, 2.0);
///
/// let (res, up) = overflow(121.0, 60.0);
/// assert_eq!(res, 1.0);
/// assert_eq!(up, 2.0);
///
/// let (res, up) = overflow(120.1, 60.0);
/// assert_approx_eq!(res, 0.1, 1e-1);
/// assert_eq!(up, 2.0);
///
/// let (res, up) = overflow(-60.0, 60.0);
/// assert_eq!(res, 0.0);
/// assert_eq!(up, -1.0);
///
/// let (res, up) = overflow(-120.0, 60.0);
/// assert_eq!(res, 0.0);
/// assert_eq!(up, -2.0);
///
/// let (res, up) = overflow(-59.0, 60.0);
/// assert_eq!(res, -59.0);
/// assert_eq!(up, 0.0);
///
/// let (res, up) = overflow(-61.0, 60.0);
/// assert_eq!(res, -1.0);
/// assert_eq!(up, -1.0);
///
/// let (res, up) = overflow(-60.1, 60.0);
/// assert_approx_eq!(res, -0.1, 1e-1);
/// assert_eq!(up, -1.0);
/// ```
pub fn overflow(value: f64, base: f64) -> (f64, f64) {
    let remainder = value % base;
    let divisible = value - remainder;
    let quotient = divisible / base;

    (remainder, quotient)
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

/// Returns the obliquity of the ecliptic (ε),
/// the angle between the planes of the equator and
/// the ecliptic, from the given datetime.
///
/// References:
/// - (Peter Duffett-Smith, p.41)
///
/// Note:
/// Does not have to be datetime, but date...
///
/// Example:
/// ```rust
/// use approx_eq::assert_approx_eq;
/// use chrono::naive::NaiveDate;
/// use sowngwala::utils::mean_obliquity_of_the_epliptic;
///
/// // TODO:
/// // It was originally: (1980, 1, 0)
/// let date = NaiveDate::from_ymd(1979, 12, 31);
/// let oblique: f64 =
///     mean_obliquity_of_the_epliptic(date);
///
/// assert_approx_eq!(
///     oblique,
///     23.441893,
///     1e-6
/// );
/// ```
#[allow(clippy::many_single_char_names)]
pub fn mean_obliquity_of_the_epliptic<T>(
    date: T,
) -> f64
where
    T: Datelike,
    T: std::marker::Copy,
    T: std::fmt::Debug,
    T: std::fmt::Display,
{
    let mut jd = julian_day_from_generic_datetime(
        NaiveDate::from_ymd(
            date.year(),
            date.month(),
            date.day(),
        )
        .and_hms(0, 0, 0),
    );
    jd -= 2_451_545.0; // January 1.5, 2000

    let t = jd / 36_525.0;
    let mut delta = (46.815 * t) + (0.0006 * t * t)
        - (0.001_81 * t * t * t);
    delta /= 3600.0;
    23.439_292 - delta
}
