#[cfg(test)]
extern crate approx_eq;

use crate::constants::{
    MOON_MEAN_LONGITUDE_AT_THE_EPOCH,
    MEAN_LONGITUDE_OF_PERIGEE_AT_THE_EPOCH,
    MEAN_LONGITUDE_OF_THE_NODE_AT_THE_EPOCH,
    INCLINATION_OF_THE_MOON_ORBIT,
};
use crate::coords::{
    EcliCoord,
    EquaCoord,
    equatorial_from_ecliptic_with_date,
};
use crate::delta_t::delta_t_from_date;
use crate::time::{
    Date,
    DateTime,
    Time,
    day_number_from_date,
    decimal_hours_from_time,
    days_since_1990,
};
use crate::sun::sun_longitude_and_mean_anomaly;

/// Given the specific date and time, returns right ascension (α)
/// and declination (δ) of equatorial coordinate.
/// (Peter Duffett-Smith, p.144)
/// * `dt` - DateTime
///
/// Example:
/// ```rust
/// use approx_eq::assert_approx_eq;
/// use sowngwala::time::{DateTime, Month};
/// use sowngwala::moon::equatorial_position_of_the_moon_from_datetime;
///
/// let dt = DateTime {
///     year: 1979,
///     month: Month::Feb,
///     day: 26.0,
///     hour: 16,
///     min: 0,
///     sec: 0.0,
/// };
///
/// let coord = equatorial_position_of_the_moon_from_datetime(&dt);
/// let asc = coord.asc;
/// let dec = coord.dec;

/// assert_eq!(asc.hour, 22);
/// assert_eq!(asc.min, 33);
/// assert_approx_eq!(
///     asc.sec, // 26.382007503326292
///     29.0,
///     1e-1
/// );
///
/// // dec:
/// //   Time { hour: -8, min: 1, sec: 1.8454925599195349 }
/// // where expected:
/// //   Time { hour: -8, min: 2, sec: 42.0 }
/// assert_eq!(dec.hour, -8);
/// assert_approx_eq!(
///     dec.min as f64, // 1.0
///     2.0,
///     2.0
/// );
/// ```
#[allow(clippy::many_single_char_names)]
pub fn equatorial_position_of_the_moon_from_datetime(&dt: &DateTime) -> EquaCoord {
    let date = Date::from(&dt);
    let day_number = day_number_from_date(&date) as f64;
    let delta_t: f64 = delta_t_from_date(&date);

    let time = Time {
        hour: dt.hour,
        min: dt.min,
        sec: dt.sec + delta_t,
    };

    let hours: f64 = decimal_hours_from_time(&time);
    let days_jan_0: f64 = day_number + (hours / 24.0);

    // Days since 1990 (d)
    let days: f64 = days_since_1990(date.year) + days_jan_0;

    // Sun's longitude (λ) and Sun's mean anomaly (M)
    let (sun_lng, sun_mean_anom): (f64, f64) = sun_longitude_and_mean_anomaly(days);

    // Moon's mean longitude (l)
    let mut l = 13.176_396_6 * days + MOON_MEAN_LONGITUDE_AT_THE_EPOCH;
    l -= 360.0 * (l / 360.0).floor();

    // Moon's mean anomaly (Mm)
    let mut mm = l - (0.111_404_1 * days) - MEAN_LONGITUDE_OF_PERIGEE_AT_THE_EPOCH;
    mm -= 360.0 * (mm / 360.0).floor();

    // Acending node's mean longitude (N).
    let mut n = MEAN_LONGITUDE_OF_THE_NODE_AT_THE_EPOCH - 0.052_953_9 * days;
    n -= 360.0 * (n / 360.0).floor();

    let c = l - sun_lng;

    // Corrections for evection (Ev)
    let ev = 1.2739 * ((2.0 * c) - mm).to_radians().sin();

    let sun_mean_anom_sin = sun_mean_anom.to_radians().sin();

    // The annual equation (Ae)
    let ae = 0.1858 * sun_mean_anom_sin;

    // The third correction (A3)
    let a3 = 0.37 * sun_mean_anom_sin;

    mm += ev - ae - a3;

    // Center of the eclipse
    let ec = 6.2886 * mm.to_radians().sin();

    // The fourth correction (A4)
    let a4 = 0.214 * (2.0 * mm).to_radians().sin();

    // Moon's corrected longitude (l)
    l += ev + ec - ae + a4;

    // Variation
    let v = 0.6583 * (2.0 * (l - sun_lng)).to_radians().sin();

    // Moon's true orbital longtude
    l += v;

    // Corrected longitude of the node
    n -= 0.16 * sun_mean_anom_sin;

    let l_minus_n = (l - n).to_radians();
    let y = l_minus_n.sin() * INCLINATION_OF_THE_MOON_ORBIT.to_radians().cos();
    let x = l_minus_n.cos();

    // Ecliptic longitude (λm)
    let lng = y.atan2(x).to_degrees() + n;

    // Ecliptic latitude (βm)
    let lat = (
        l_minus_n.sin() * INCLINATION_OF_THE_MOON_ORBIT.to_radians().sin()
    ).asin().to_degrees();

    equatorial_from_ecliptic_with_date(
        EcliCoord { lat, lng },
        &date
    )
}
