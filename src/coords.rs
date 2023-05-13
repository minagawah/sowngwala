use crate::time::{
    angle_from_decimal_hours,
    decimal_hours_from_angle,
    decimal_hours_from_generic_time, gst_from_utc,
    lst_from_gst, nano_from_second, normalize_angle,
};
use crate::utils::mean_obliquity_of_the_epliptic;
use chrono::naive::{
    NaiveDate, NaiveDateTime, NaiveTime,
};
use chrono::offset::Utc;
use chrono::{DateTime, Datelike, Timelike};
use std::convert::From;
use std::f64::consts::PI;

#[derive(Debug, Copy, Clone)]
pub struct Angle {
    pub hour: i32,
    pub minute: i32,
    pub second: f64,
}

impl Angle {
    pub fn new(
        hour: i32,
        minute: i32,
        second: f64,
    ) -> Self {
        Angle {
            hour,
            minute,
            second,
        }
    }

    pub fn hour(&self) -> i32 {
        self.hour
    }
    pub fn minute(&self) -> i32 {
        self.minute
    }
    pub fn second(&self) -> f64 {
        self.second
    }

    pub fn to_naive_time(self) -> NaiveTime {
        self.into()
    }
}

impl From<Angle> for NaiveTime {
    fn from(angle: Angle) -> Self {
        let (angle_1, _day_excess) =
            normalize_angle(angle);

        let (sec, nano): (u32, u32) =
            nano_from_second(angle_1.second());

        NaiveTime::from_hms_nano(
            angle_1.hour() as u32,
            angle_1.minute() as u32,
            sec,
            nano,
        )
    }
}

pub enum Direction {
    North,
    East,
    South,
    West,
}

// Geometric Coordinate
#[derive(Debug)]
pub struct Coord {
    pub lat: f64,
    pub lng: f64,
}

// Ecliptic Coordinate
#[derive(Debug)]
pub struct EcliCoord {
    pub lat: f64,
    pub lng: f64,
}

// Galactic Coordinate
#[derive(Debug)]
pub struct GalacCoord {
    pub lat: f64,
    pub lng: f64,
}

// Equatorial Coordinate
#[derive(Debug)]
pub struct EquaCoord {
    pub asc: Angle, // right ascension (α)
    pub dec: Angle, // declination (δ)
}

// Equatorial Coordinate (with Hour-Angle)
#[derive(Debug)]
pub struct EquaCoord2 {
    pub ha: Angle,  // hour-angle (H)
    pub dec: Angle, // declination (δ)
}

// Ecliptic coordinate
#[derive(Debug)]
pub struct HorizCoord {
    pub alt: Angle, // altitude (a)
    pub azi: Angle, // azimuth (A)
}

/// Given UTC, right ascension (α), and longitude
/// (along with its direction), returns
/// hour-angle (H).
///
/// * `utc` - UTC
/// * `asc` - Right-ascension
/// * `lng` - Longitude
/// * `dir` - Direction for Longitude
///
/// Reference:
/// - (Peter Duffett-Smith, p.35)
///
/// Example:
/// ```rust
/// use approx_eq::assert_approx_eq;
/// use chrono::{DateTime, Timelike};
/// use chrono::offset::Utc;
/// use sowngwala::time::{
///   build_utc,
///   angle_from_decimal_hours,
///   decimal_hours_from_generic_time,
///   lst_from_gst,
/// };
/// use sowngwala::coords::{
///   Angle,
///   Direction,
///   hour_angle_from_utc
/// };
///
/// let dir = Direction::West;
/// let lng = 64.0;
/// let asc: Angle = Angle::new(18, 32, 21.0);
///
/// // TODO: Do we need `zone`? Originally, no zone.
/// let zone: i32 = 4;
/// let nanosecond: u32 = 670_000_000;
/// let utc: DateTime<Utc> =
///     build_utc(1980, 4, 22, 14, 36, 51, nanosecond);
///
/// let hour_angle: Angle =
///     hour_angle_from_utc(utc, asc, lng, dir);
///
/// assert_eq!(hour_angle.hour(), 5);
/// assert_eq!(hour_angle.minute(), 51);
/// assert_approx_eq!(
///     hour_angle.second(), // 44.22957675918951
///     44.0,
///     1e-2
/// );
/// ```
pub fn hour_angle_from_utc(
    utc: DateTime<Utc>,
    asc: Angle,
    lng: f64,
    dir: Direction,
) -> Angle {
    let gst: NaiveTime = gst_from_utc(utc);
    let gst_0: NaiveDateTime = NaiveDate::from_ymd(
        utc.year(),
        utc.month(),
        utc.day(),
    )
    .and_hms_nano(
        gst.hour(),
        gst.minute(),
        gst.second(),
        gst.nanosecond(),
    );

    let lst: NaiveTime =
        lst_from_gst(gst_0, lng, dir);
    let lst_decimal: f64 =
        decimal_hours_from_generic_time(lst);
    let asc_decimal: f64 =
        decimal_hours_from_angle(asc);

    let mut hour_angle = lst_decimal - asc_decimal;

    if hour_angle < 0.0 {
        hour_angle += 24.0;
    }

    angle_from_decimal_hours(hour_angle)
}

/// Given UT, hour-angle (H), and longitude
/// (along with its direction), returns right
/// ascension (α).
///
/// * `utc` - UTC
/// * `ha` - Hour-angle (H)
/// * `lng` - Longitude
/// * `dir` - Direction for Longitude
///
/// Reference:
/// - (Peter Duffett-Smith, p.35)
///
/// Example:
/// ```rust
/// use approx_eq::assert_approx_eq;
/// use chrono::{DateTime, Timelike};
/// use chrono::offset::Utc;
/// use sowngwala::time::build_utc;
/// use sowngwala::coords::{
///   Angle,
///   Direction,
///   right_ascension_from_utc
/// };
///
/// let dir = Direction::West;
/// let lng = 64.0;
///
/// // hour-angle
/// let ha = Angle::new(5, 51, 44.0);
///
/// // TODO: Do we need `zone`? Originally, no zone.
/// let zone: i32 = 4;
/// let nanosecond: u32 = 670_000_000;
/// let utc: DateTime<Utc> =
///     build_utc(1980, 4, 22, 14, 36, 51, nanosecond);
///
/// let asc = right_ascension_from_utc(
///     utc,
///     ha,
///     lng,
///     dir
/// );
/// assert_eq!(asc.hour(), 18);
/// assert_eq!(asc.minute(), 32);
/// assert_approx_eq!(
///     asc.second(), // 21.229576759189968
///     21.0,
///     1e-1
/// );
/// ```
pub fn right_ascension_from_utc(
    utc: DateTime<Utc>,
    ha: Angle,
    lng: f64,
    dir: Direction,
) -> Angle {
    let gst = gst_from_utc(utc);
    let gst_0: NaiveDateTime = NaiveDate::from_ymd(
        utc.year(),
        utc.month(),
        utc.day(),
    )
    .and_hms_nano(
        gst.hour(),
        gst.minute(),
        gst.second(),
        gst.nanosecond(),
    );

    let lst: NaiveTime =
        lst_from_gst(gst_0, lng, dir);
    let lst_decimal: f64 =
        decimal_hours_from_generic_time(lst);
    let ha_decimal: f64 =
        decimal_hours_from_angle(ha);

    let mut asc = lst_decimal - ha_decimal;
    if asc < 0.0 {
        asc += 24.0;
    }

    angle_from_decimal_hours(asc)
}

/// Given equatorial coordinate with hour-angle (H),
/// declination (δ), and observer's latitude (φ),
/// returns altitude (a) and azimuth (A) for that of
/// horizontal coordinate.
///
/// * `coord` - Equatorial coordinate (with hour-angle)
/// * `coord.ha` - Hour-angle (H)
/// * `coord.dec` - Declination (δ)
/// * `lat` - Latitude (φ)
///
/// Reference:
/// - (Peter Duffett-Smith, pp.36-37)
///
/// Example:
/// ```rust
/// use approx_eq::assert_approx_eq;
/// use chrono::Timelike;
/// use sowngwala::coords::{
///   Angle,
///   EquaCoord2,
///   HorizCoord,
///   horizon_from_equatorial
/// };
///
/// let lat = 52.0;
///
/// // hour-angle
/// let ha = Angle::new(5, 51, 44.0);
///
/// // declination
/// let dec = Angle::new(23, 13, 10.0);
///
/// let coord_0 = EquaCoord2 { ha, dec };
/// let coord = horizon_from_equatorial(coord_0, lat);
/// let alt: Angle = coord.alt;
/// let azi: Angle = coord.azi;
///
/// assert_eq!(alt.hour(), 19);
/// assert_eq!(alt.minute(), 20);
/// assert_approx_eq!(
///     alt.second(), // 3.6428077696939454
///     4.0,
///     1e-0
/// );
///
/// assert_eq!(azi.hour(), 283);
/// assert_eq!(azi.minute(), 16);
/// assert_approx_eq!(
///     azi.second(), // 15.698162189496543
///     16.0,
///     1e-0
/// );
/// ```
pub fn horizon_from_equatorial(
    coord: EquaCoord2,
    lat: f64,
) -> HorizCoord {
    let hour_angle: f64 =
        (decimal_hours_from_angle(coord.ha) * 15.0)
            .to_radians();
    let decline: f64 =
        decimal_hours_from_angle(coord.dec)
            .to_radians();
    let latitude: f64 = lat.to_radians();

    let altitude = ((decline.sin() * latitude.sin())
        + (decline.cos()
            * latitude.cos()
            * hour_angle.cos()))
    .asin();

    let mut azimuth = ((decline.sin()
        - (latitude.sin() * altitude.sin()))
        / (latitude.cos() * altitude.cos()))
    .acos();

    azimuth = if hour_angle.sin() < 0.0 {
        azimuth
    } else {
        (2.0 * PI) - azimuth
    };

    HorizCoord {
        alt: angle_from_decimal_hours(
            altitude.to_degrees(),
        ),
        azi: angle_from_decimal_hours(
            azimuth.to_degrees(),
        ),
    }
}

/// Given altitude (a), azimuth (A), and observer's
/// latitude (φ), returns hour-angle (H) and
/// declination (δ) for that of equatorial coordinate.
///
/// * `coord` - Horizontal coordinate
/// * `coord.alt` - Altitude (a)
/// * `coord.azi` - Azimuth (A)
/// * `lat` - Latitude (φ)
///
/// Reference:
/// - (Peter Duffett-Smith, pp.38-39)
///
/// Example:
/// ```rust
/// use approx_eq::assert_approx_eq;
/// use chrono::Timelike;
/// use sowngwala::coords::{
///   Angle,
///   EquaCoord2,
///   HorizCoord,
///   equatorial_from_horizon
/// };
///
/// let lat = 52.0;
///
/// // altitude
/// let alt = Angle::new(19, 20, 4.0);
///
/// // azimuth
/// let azi = Angle::new(283, 16, 16.0);
///
/// let coord_0 = HorizCoord { alt, azi };
/// let coord: EquaCoord2 =
///     equatorial_from_horizon(coord_0, lat);
/// let ha: Angle = coord.ha;
/// let dec: Angle = coord.dec;
///
/// assert_eq!(ha.hour(), 5);
/// assert_eq!(ha.minute(), 51);
/// assert_approx_eq!(
///     ha.second(), // 43.998769832229954
///     44.0,
///     1e-0
/// );
///
/// assert_eq!(dec.hour(), 23);
/// assert_eq!(dec.minute(), 13);
/// assert_approx_eq!(
///     dec.second(), // 10.456528456985552
///     10.0,
///     1e-1
/// );
/// ```
pub fn equatorial_from_horizon(
    coord: HorizCoord,
    lat: f64,
) -> EquaCoord2 {
    let altitude: f64 =
        decimal_hours_from_angle(coord.alt)
            .to_radians();
    let azimuth: f64 =
        decimal_hours_from_angle(coord.azi)
            .to_radians();
    let latitude: f64 = lat.to_radians();

    let decline = ((altitude.sin() * latitude.sin())
        + (altitude.cos()
            * latitude.cos()
            * azimuth.cos()))
    .asin();

    let mut hour_angle = ((altitude.sin()
        - (latitude.sin() * decline.sin()))
        / (latitude.cos() * decline.cos()))
    .acos();

    hour_angle = if azimuth.sin() < 0.0 {
        hour_angle
    } else {
        (2.0 * PI) - hour_angle
    };

    hour_angle /= PI / 12.0;

    EquaCoord2 {
        ha: angle_from_decimal_hours(hour_angle),
        dec: angle_from_decimal_hours(
            decline.to_degrees(),
        ),
    }
}

/// Given LST and hour-angle (H), returns right
/// ascension (α),
///
/// * `lst` - LST
/// * `ha` - Hour-angle (H)
///
/// Reference:
/// - (Peter Duffett-Smith, p.39)
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
///   right_ascension_from_lst_and_hour_angle,
/// };
///
/// let lst: NaiveDateTime =
///     NaiveDate::from_ymd(1980, 4, 22)
///         .and_hms(0, 24, 5);
///
/// // hour-angle
/// let ha: Angle = Angle::new(5, 51, 44.0);
///
/// let asc: Angle =
///     right_ascension_from_lst_and_hour_angle(
///         lst,
///         ha
///     );
///
/// assert_eq!(asc.hour(), 18);
/// assert_eq!(asc.minute(), 32);
/// assert_approx_eq!(
///     asc.second(), // 20.99999999999966
///     21.0,
///     1e-0
/// );
/// ```
pub fn right_ascension_from_lst_and_hour_angle<T>(
    lst: T,
    ha: Angle,
) -> Angle
where
    T: Datelike,
    T: Timelike,
    T: std::marker::Copy,
    T: std::fmt::Debug,
    T: std::fmt::Display,
{
    let ha_decimal: f64 =
        decimal_hours_from_angle(ha);
    let angle = Angle::new(
        lst.hour() as i32,
        lst.minute() as i32,
        lst.second() as f64,
    );
    let lst_decimal: f64 =
        decimal_hours_from_angle(angle);
    let mut asc = lst_decimal - ha_decimal;
    if asc < 0.0 {
        asc += 24.0;
    }

    angle_from_decimal_hours(asc)
}

/// Given ecliptic ecliptic latitude (β) and
/// longitude (λ) (optionally takes date for specific
/// obliquity of the ecliptic (ε)), returns right
/// ascension (α) and declination (δ) for that of
/// equatorial coordinate.
///
/// * `coord` - Ecliptic coordinate
/// * `coord.lat` - Latitude (β)
/// * `coord.lng` - Longitude (λ)
/// * `date` - Date for specific obliquity of the eplictic (ε)
///
/// Reference:
/// - (Peter Duffett-Smith, pp.40-41)
///
/// Example:
/// ```rust
/// use approx_eq::assert_approx_eq;
/// use chrono::Timelike;
/// use chrono::naive::NaiveDate;
/// use sowngwala::time::decimal_hours_from_angle;
/// use sowngwala::coords::{
///   Angle,
///   EcliCoord,
///   equatorial_from_ecliptic_with_generic_date
/// };
///
/// let lat_0 = Angle::new(4, 52, 31.0);
/// let lng_0 = Angle::new(139, 41, 10.0);
///
/// let coord_0 = EcliCoord {
///     lat: decimal_hours_from_angle(lat_0),
///     lng: decimal_hours_from_angle(lng_0),
/// };
///
/// // To calculate a specific value
/// // for mean obliquity of the ecliptic.
/// let date = NaiveDate::from_ymd(1980, 4, 22);
/// let coord =
///     equatorial_from_ecliptic_with_generic_date(
///         coord_0,
///         date
///     );
/// let asc: Angle = coord.asc;
/// let dec: Angle = coord.dec;
///
/// assert_eq!(asc.hour(), 9);
/// assert_eq!(asc.minute(), 34);
/// assert_approx_eq!(
///     asc.second(), // 53.58216253599352
///     53.6,
///     1e-2
/// );
///
/// assert_eq!(dec.hour(), 19);
/// assert_eq!(dec.minute(), 32);
/// assert_approx_eq!(
///     dec.second(), // 14.100993558899972
///     14.2,
///     1e-2
/// );
/// ```
#[allow(clippy::many_single_char_names)]
pub fn equatorial_from_ecliptic_with_generic_date<T>(
    coord: EcliCoord,
    date: T,
) -> EquaCoord
where
    T: Datelike,
    T: std::marker::Copy,
    T: std::fmt::Debug,
    T: std::fmt::Display,
{
    let oblique =
        mean_obliquity_of_the_epliptic(date)
            .to_radians();

    let oblique_cos = oblique.cos();
    let oblique_sin = oblique.sin();

    let lat = coord.lat; // latitude (β)
    let lng = coord.lng; // longitude (λ)

    let lat_cos = lat.to_radians().cos();
    let lat_sin = lat.to_radians().sin();
    let lat_tan = lat.to_radians().tan();
    let lng_cos = lng.to_radians().cos();
    let lng_sin = lng.to_radians().sin();

    let decline_0 = (lat_sin * oblique_cos)
        + (lat_cos * oblique_sin * lng_sin);
    let decline = decline_0.asin().to_degrees();

    let y = (lng_sin * oblique_cos)
        - (lat_tan * oblique_sin);
    let x = lng_cos;

    let mut asc = y.atan2(x).to_degrees();
    asc -= 360.0 * (asc / 360.0).floor();
    asc /= 15.0;

    EquaCoord {
        asc: angle_from_decimal_hours(asc),
        dec: angle_from_decimal_hours(decline),
    }
}

pub fn equatorial_from_ecliptic(
    coord: EcliCoord,
) -> EquaCoord {
    equatorial_from_ecliptic_with_generic_date(
        coord,
        NaiveDate::from_ymd(2021, 1, 0),
    )
}

/// Given right ascension (α) and declination (δ) of
/// equatorial coordinate (optionally takes date for
/// specific obliquity of the ecliptic (ε)),
/// returns latitude (β) and longitude (λ)
///
/// * `coord` - Equatorial coordinate
/// * `coord.asc` - Right ascension (α)
/// * `coord.dec` - Declination (δ)
/// * `date` - Date for specific obliquity of the eplictic (ε)
///
/// Reference:
/// - (Peter Duffett-Smith, p.42)
///
/// Example:
/// ```rust
/// use approx_eq::assert_approx_eq;
/// use chrono::Timelike;
/// use chrono::naive::{
///     NaiveDate,
///     NaiveDateTime,
/// };
/// use sowngwala::time::angle_from_decimal_hours;
/// use sowngwala::coords::{
///   Angle,
///   EcliCoord,
///   EquaCoord,
///   ecliptic_from_equatorial_with_generic_date,
/// };
///
/// // right-ascension
/// let asc: Angle = Angle::new(9, 34, 53.6);
///
/// // declination
/// let dec: Angle = Angle::new(19, 32, 14.2);
///
/// // To calculate a specific value for mean obliquity
/// // of the ecliptic.
/// let date = NaiveDate::from_ymd(1980, 4, 22);
/// let coord_0 = EquaCoord { asc, dec };
///
/// let coord: EcliCoord =
///     ecliptic_from_equatorial_with_generic_date(
///         coord_0,
///         date
///     );
/// let lat: Angle =
///     angle_from_decimal_hours(coord.lat);
/// let lng: Angle =
///     angle_from_decimal_hours(coord.lng);
///
/// assert_eq!(lat.hour(), 4);
/// assert_eq!(lat.minute(), 52);
/// assert_approx_eq!(
///     lat.second(), // 31.17490012745307
///     31.0,
///     1e-2
/// );
///
/// assert_eq!(lng.hour(), 139);
/// assert_eq!(lng.minute(), 41);
/// assert_approx_eq!(
///     lng.second(), // 10.207621429881328
///     10.0,
///     3e-2
/// );
/// ```
#[allow(clippy::many_single_char_names)]
pub fn ecliptic_from_equatorial_with_generic_date<T>(
    coord: EquaCoord,
    date: T,
) -> EcliCoord
where
    T: Datelike,
    T: std::marker::Copy,
    T: std::fmt::Debug,
    T: std::fmt::Display,
{
    let oblique: f64 =
        mean_obliquity_of_the_epliptic(date)
            .to_radians();
    let oblique_cos: f64 = oblique.cos();
    let oblique_sin: f64 = oblique.sin();

    // right ascension (α)
    let mut asc_decimal: f64 =
        decimal_hours_from_angle(coord.asc)
            .to_radians();

    // declination (δ)
    let dec_decimal: f64 =
        decimal_hours_from_angle(coord.dec)
            .to_radians();
    asc_decimal *= 15.0;

    let asc_sin: f64 = asc_decimal.sin();
    let asc_cos: f64 = asc_decimal.cos();
    let dec_sin: f64 = dec_decimal.sin();
    let dec_cos: f64 = dec_decimal.cos();
    let dec_tan: f64 = dec_decimal.tan();

    let lat_0: f64 = (dec_sin * oblique_cos)
        - (dec_cos * oblique_sin * asc_sin);
    let lat: f64 = lat_0.asin().to_degrees();

    let y: f64 = (asc_sin * oblique_cos)
        + (dec_tan * oblique_sin);
    let x: f64 = asc_cos;

    let mut lng: f64 = y.atan2(x).to_degrees();
    lng -= 360.0 * (lng / 360.0).floor();

    EcliCoord { lat, lng }
}

/// Given right ascension (α) and declination (δ) of
/// equatorial coordinate, returns galactic
/// latitude (b) and galactic longitude (l).
///
/// * `coord` - Equatorial coordinate
/// * `coord.asc` - Right ascension (α)
/// * `coord.dec` - Declination (δ)
///
/// Reference:
/// - (Peter Duffett-Smith, p.43)
///
/// Example:
/// ```rust
/// use approx_eq::assert_approx_eq;
/// use chrono::Timelike;
/// use chrono::naive::{
///     NaiveDate,
///     NaiveDateTime,
/// };
/// use sowngwala::time::angle_from_decimal_hours;
/// use sowngwala::coords::{
///   Angle,
///   EquaCoord,
///   GalacCoord,
///   galactic_from_equatorial,
/// };
///
/// // right-ascension
/// let asc: Angle = Angle::new(10, 21, 0.0);
///
/// // declination
/// let dec: Angle = Angle::new(10, 3, 11.0);
///
/// let coord_0 = EquaCoord { asc, dec };
///
/// let coord: GalacCoord =
///     galactic_from_equatorial(coord_0);
/// let lat: Angle =
///     angle_from_decimal_hours(coord.lat);
/// let lng: Angle =
///     angle_from_decimal_hours(coord.lng);
///
/// assert_eq!(lat.hour(), 51);
/// assert_eq!(lat.minute(), 7);
/// assert_approx_eq!(
///     lat.second(), // 20.16407768754391
///     20.0,
///     1e-2
/// );
///
/// assert_eq!(lng.hour(), 232);
/// assert_eq!(lng.minute(), 14);
/// assert_approx_eq!(
///     lng.second(), // 52.38055557683538
///     52.0,
///     1e-2
/// );
/// ```
#[allow(clippy::many_single_char_names)]
pub fn galactic_from_equatorial(
    coord: EquaCoord,
) -> GalacCoord {
    // right ascension (α)
    let mut asc_decimal: f64 =
        decimal_hours_from_angle(coord.asc)
            .to_radians();

    // declination (δ)
    let dec_decimal: f64 =
        decimal_hours_from_angle(coord.dec)
            .to_radians();
    asc_decimal *= 15.0;

    let dec_sin: f64 = dec_decimal.sin();
    let dec_cos: f64 = dec_decimal.cos();
    let asc_192: f64 = (asc_decimal.to_degrees()
        - 192.25)
        .to_radians();

    let r_27: f64 = (27.4_f64).to_radians();
    let r_27_sin: f64 = r_27.sin();
    let r_27_cos: f64 = r_27.cos();

    let b_sin: f64 =
        dec_cos * r_27.cos() * asc_192.cos()
            + (dec_sin * r_27_sin);
    let b: f64 = b_sin.asin();

    let y: f64 = dec_sin - (b_sin * r_27_sin);
    let x: f64 = dec_cos * asc_192.sin() * r_27_cos;

    let mut l: f64 = y.atan2(x).to_degrees();
    l -= 360.0 * (l / 360.0).floor();
    l += 33.0;

    GalacCoord {
        lat: b.to_degrees(),
        lng: l,
    }
}

/// Given galactic latitude (b) and galactic
/// longitude (l), returns right ascension (α)
/// and declination (δ) of equatorial coordinate,
///
/// * `coord` - Galactic coordinate
/// * `coord.lat` - Galactic latitude (b)
/// * `coord.lng` - Galactic longitude (l)
///
/// Reference:
/// - (Peter Duffett-Smith, p.44)
///
/// Example:
/// ```rust
/// use approx_eq::assert_approx_eq;
/// use chrono::Timelike;
/// use chrono::naive::{
///     NaiveDate,
///     NaiveDateTime,
/// };
/// use sowngwala::time::decimal_hours_from_angle;
/// use sowngwala::coords::{
///   Angle,
///   EquaCoord,
///   GalacCoord,
///   equatorial_from_galactic
/// };
///
/// // Galactic latitude (b)
/// let lat: Angle = Angle::new(51, 7, 20.0);
///
/// // Galactic latitude (l)
/// let lng: Angle = Angle::new(232, 14, 52.0);
///
/// let coord_0 = GalacCoord {
///     lat: decimal_hours_from_angle(lat),
///     lng: decimal_hours_from_angle(lng),
/// };
///
/// let coord: EquaCoord =
///     equatorial_from_galactic(coord_0);
/// let asc: Angle = coord.asc;
/// let dec: Angle = coord.dec;
///
/// assert_eq!(asc.hour(), 10);
/// // TODO:
/// // The book tells it should be 21 for 'asc.min'.
/// assert_eq!(asc.minute(), 20);
/// assert_approx_eq!(
///     asc.second(), // 59.98205693746215
///     59.9,
///     1e-2
/// );
///
/// assert_eq!(dec.hour(), 10);
/// assert_eq!(dec.minute(), 3);
/// assert_approx_eq!(
///     dec.second(), // 11.117231829019829
///     11.0,
///     2e-2
/// );
/// ```
#[allow(clippy::many_single_char_names)]
pub fn equatorial_from_galactic(
    coord: GalacCoord,
) -> EquaCoord {
    let b: f64 = coord.lat.to_radians(); // // Galactic latitude (b)
    let l: f64 = coord.lng.to_radians(); // Galactic longitude (l)
    let b_sin: f64 = b.sin();
    let b_cos: f64 = b.cos();

    let l_minus_33: f64 =
        (l.to_degrees() - 33.0).to_radians();
    let l_minus_33_sin: f64 = l_minus_33.sin();
    let l_minus_33_cos: f64 = l_minus_33.cos();

    let r_27: f64 = (27.4_f64).to_radians();
    let r_27_sin: f64 = r_27.sin();
    let r_27_cos: f64 = r_27.cos();

    let dec_sin: f64 =
        (b_cos * r_27_cos * l_minus_33_sin)
            + (b_sin * r_27_sin);
    let dec: f64 = dec_sin.asin();

    let y: f64 = b_cos * l_minus_33_cos;
    let x: f64 = (b_sin * r_27_cos)
        - (b_cos * r_27_sin * l_minus_33_sin);

    let mut asc: f64 = y.atan2(x).to_degrees();
    asc += 192.25;
    asc -= 360.0 * (asc / 360.0).floor();
    asc /= 15.0;

    EquaCoord {
        asc: angle_from_decimal_hours(asc),
        dec: angle_from_decimal_hours(
            dec.to_degrees(),
        ),
    }
}

/// Given coordinates for two celestial objects
/// expressed in ecliptic coordinate system
/// (latitude (β) and longitude (λ)), returns
/// the angle between them.
///
/// * `coord_0` - Equatorial coordinate
/// * `coord_0.asc` - Right ascension (α)
/// * `coord_0.dec` - Declination (δ)
/// * `coord_1` - Equatorial coordinate
/// * `coord_1.asc` - Right ascension (α)
/// * `coord_1.dec` - Declination (δ)
///
/// Reference:
/// - (Peter Duffett-Smith, p.51)
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
///   angle_between_two_celestial_objects_for_equatorial
/// };
///
/// // right-ascension (for Beta Orionis)
/// let asc_0: Angle = Angle::new(5, 13, 31.7);
///
/// // declination (for Beta Orionis)
/// let dec_0: Angle = Angle::new(-8, 13, 30.0);
///
/// // right-ascension (for Canis Majoris)
/// let asc_1: Angle = Angle::new(6, 44, 13.4);
///
/// // declination (for Canis Majoris)
/// let dec_1: Angle = Angle::new(-16, 41, 11.0);
///
/// let angle: f64 =
///     angle_between_two_celestial_objects_for_equatorial(
///         EquaCoord { asc: asc_0, dec: dec_0 },
///         EquaCoord { asc: asc_1, dec: dec_1 }
///     );
///
/// assert_approx_eq!(
///     angle, // 23.67384942216419
///     23.673_850,
///     1e-6
/// );
/// ```
#[allow(clippy::many_single_char_names)]
pub fn angle_between_two_celestial_objects_for_equatorial(
    coord_0: EquaCoord,
    coord_1: EquaCoord,
) -> f64 {
    angle_between_two_celestial_objects(
        decimal_hours_from_angle(coord_0.asc),
        decimal_hours_from_angle(coord_0.dec),
        decimal_hours_from_angle(coord_1.asc),
        decimal_hours_from_angle(coord_1.dec),
    )
}

/// Given coordinates for two celestial objects
/// expressed in galactic coordinate system
/// (latitude (b) and longitude (l)), returns
/// the angle between them.
///
/// * `coord_0` - Galactic coordinate
/// * `coord_0.lat` - Galactic latitude (b)
/// * `coord_0.lng` - Galactic longitude (l)
/// * `coord_1` - Galactic coordinate
/// * `coord_1.lat` - Galactic latitude (b)
/// * `coord_1.lng` - Galactic longitude (l)
///
/// Reference:
/// - (Peter Duffett-Smith, p.51)
#[allow(clippy::many_single_char_names)]
pub fn angle_between_two_celestial_objects_for_galactic(
    coord_0: GalacCoord,
    coord_1: GalacCoord,
) -> f64 {
    angle_between_two_celestial_objects(
        coord_0.lat,
        coord_0.lng,
        coord_1.lat,
        coord_1.lng,
    )
}

#[allow(clippy::many_single_char_names)]
pub fn angle_between_two_celestial_objects(
    asc_0: f64,
    dec_0: f64,
    asc_1: f64,
    dec_1: f64,
) -> f64 {
    let tmp = ((asc_0 - asc_1) * 15.0).to_radians();
    let dec_0 = dec_0.to_radians();
    let dec_1 = dec_1.to_radians();
    let d_cos = (dec_0.sin() * dec_1.sin())
        + (dec_0.cos() * dec_1.cos() * tmp.cos());

    d_cos.acos().to_degrees()
}
