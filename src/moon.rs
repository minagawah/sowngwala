use chrono::{
    Datelike,
    Timelike,
};
use crate::constants::{
    MOON_MEAN_LONGITUDE_AT_THE_EPOCH,
    MEAN_LONGITUDE_OF_PERIGEE_AT_THE_EPOCH,
    MEAN_LONGITUDE_OF_THE_NODE_AT_THE_EPOCH,
    INCLINATION_OF_THE_MOON_ORBIT,
};
use crate::coords::{
    EcliCoord,
    EquaCoord,
    equatorial_from_ecliptic_with_generic_date,
};
use crate::delta_t::delta_t_from_generic_date;
use crate::coords::Angle;
use crate::time::{
    day_number_from_generic_date,
    decimal_hours_from_angle,
    days_since_1990,
    naive_date_from_generic_datetime,
};
use crate::sun::sun_longitude_and_mean_anomaly;

/// Given the specific date and time, returns right ascension (α)
/// and declination (δ) of equatorial coordinate.
///
/// * `dt` - DateTime
///
/// Reference:
/// - (Peter Duffett-Smith, p.144)
///
/// Example:
/// ```rust
/// use approx_eq::assert_approx_eq;
/// use chrono::Timelike;
/// use chrono::naive::{
///     NaiveDate,
///     NaiveDateTime,
/// };
/// use sowngwala::coords::{
///   Angle,
///   EquaCoord,
/// };
/// use sowngwala::moon::equatorial_position_of_the_moon_from_generic_datetime;
///
/// let dt: NaiveDateTime = NaiveDate::from_ymd(1979, 2, 26)
///     .and_hms(16, 0, 0);
///
/// let coord: EquaCoord = equatorial_position_of_the_moon_from_generic_datetime(
///   dt
/// );
/// let asc: Angle = coord.asc;
/// let dec: Angle = coord.dec;
///
/// assert_eq!(asc.hour(), 22);
/// assert_eq!(asc.minute(), 33);
/// assert_approx_eq!(
///     asc.second(), // 26.382007503326292
///     29.0,
///     1e-1
/// );
///
/// // dec:
/// //   Angle { hour: -8, minute: 1, second: 1.8454925599195349 }
/// // where expected:
/// //   Angle { hour: -8, minute: 2, second: 42.0 }
///
/// assert_eq!(dec.hour(), -8);
/// assert_approx_eq!(
///     dec.minute() as f64, // 1.0
///     2.0,
///     2.0
/// );
/// ```
#[allow(clippy::many_single_char_names)]
pub fn equatorial_position_of_the_moon_from_generic_datetime<T>(
    dt: T
) -> EquaCoord
    where T: Datelike,
          T: Timelike,
          T: std::marker::Copy,
          T: std::fmt::Debug,
          T: std::fmt::Display
{
    let date = naive_date_from_generic_datetime(dt);
    let day_number = day_number_from_generic_date(date) as f64;
    let delta_t: f64 = delta_t_from_generic_date(date);

    let angle = Angle::new(
        dt.hour() as i32,
        dt.minute() as i32,
        (dt.second() as f64) + delta_t,
    );

    let hours: f64 = decimal_hours_from_angle(angle);
    let days_jan_0: f64 = day_number + (hours / 24.0);

    // Days since 1990 (d)
    let days: f64 = days_since_1990(date.year()) as f64 + days_jan_0;

    // Sun's longitude (λ) and Sun's mean anomaly (M)
    let (sun_lng, sun_mean_anom): (f64, f64) =
        sun_longitude_and_mean_anomaly(days);

    // Moon's mean longitude (l)
    let mut l: f64 = 13.176_396_6 * days + MOON_MEAN_LONGITUDE_AT_THE_EPOCH;
    l -= 360.0 * (l / 360.0).floor();

    // Moon's mean anomaly (Mm)
    let mut mm: f64 = l
        - (0.111_404_1 * days)
        - MEAN_LONGITUDE_OF_PERIGEE_AT_THE_EPOCH;

    mm -= 360.0 * (mm / 360.0).floor();

    // Acending node's mean longitude (N).
    let mut n: f64 = MEAN_LONGITUDE_OF_THE_NODE_AT_THE_EPOCH
        - (0.052_953_9 * days);

    n -= 360.0 * (n / 360.0).floor();

    let c: f64 = l - sun_lng;

    // Corrections for evection (Ev)
    let ev: f64 = 1.2739 * ((2.0 * c) - mm).to_radians().sin();

    let sun_mean_anom_sin: f64 = sun_mean_anom.to_radians().sin();

    // The annual equation (Ae)
    let ae: f64 = 0.1858 * sun_mean_anom_sin;

    // The third correction (A3)
    let a3: f64 = 0.37 * sun_mean_anom_sin;

    mm += ev - ae - a3;

    // Center of the eclipse
    let ec: f64 = 6.2886 * mm.to_radians().sin();

    // The fourth correction (A4)
    let a4: f64 = 0.214 * (2.0 * mm).to_radians().sin();

    // Moon's corrected longitude (l)
    l += ev + ec - ae + a4;

    // Variation
    let v: f64 = 0.6583 * (2.0 * (l - sun_lng)).to_radians().sin();

    // Moon's true orbital longtude
    l += v;

    // Corrected longitude of the node
    n -= 0.16 * sun_mean_anom_sin;

    let l_minus_n: f64 = (l - n).to_radians();

    let y: f64 = l_minus_n.sin()
        * INCLINATION_OF_THE_MOON_ORBIT.to_radians().cos();

    let x: f64 = l_minus_n.cos();

    // Ecliptic longitude (λm)
    let lng: f64 = y.atan2(x).to_degrees() + n;

    // Ecliptic latitude (βm)
    let lat: f64 = (
        l_minus_n.sin() * INCLINATION_OF_THE_MOON_ORBIT.to_radians().sin()
    ).asin().to_degrees();

    equatorial_from_ecliptic_with_generic_date(
        EcliCoord { lat, lng },
        date
    )
}
