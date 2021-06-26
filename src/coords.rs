#[cfg(test)]
extern crate approx_eq;

use std::f64::consts::PI;

use crate::time::{
    Date,
    DateTime,
    Month,
    Time,
    gst_from_ut,
    lst_from_gst,
    decimal_hours_from_time,
    time_from_decimal_hours,
};
use crate::utils::mean_obliquity_of_the_epliptic;

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
    pub asc: Time, // right ascension (α)
    pub dec: Time, // declination (δ)
}

// Equatorial Coordinate (with Hour-Angle)
#[derive(Debug)]
pub struct EquaCoord2 {
    pub ha: Time, // hour-angle (H)
    pub dec: Time, // declination (δ)
}

// Ecliptic coordinate
#[derive(Debug)]
pub struct HorizCoord {
    pub alt: Time, // altitude (a)
    pub azi: Time, // azimuth (A)
}

/// Given UT, right ascension (α), and longitude (along with its direction),
/// returns hour-angle (H).
/// (Peter Duffett-Smith, p.35)
/// * `ut` - UT
/// * `asc` - Right-ascension
/// * `lng` - Longitude
/// * `dir` - Direction for Longitude
pub fn hour_angle_from_ut(&ut: &DateTime, &asc: &Time, lng: f64, dir: Direction) -> Time {
    let gst = gst_from_ut(&ut);
    let gst_0 = DateTime {
        year: ut.year,
        month: ut.month,
        day: ut.day,
        hour: gst.hour,
        min: gst.min,
        sec: gst.sec,
    };
    let lst = lst_from_gst(&gst_0, lng, dir);
    let lst_decimal = decimal_hours_from_time(&lst);
    let asc_decimal = decimal_hours_from_time(&asc);

    let mut hour_angle = lst_decimal - asc_decimal;
    if hour_angle < 0.0 {
        hour_angle += 24.0;
    }
    time_from_decimal_hours(hour_angle)
}


/// Given UT, hour-angle (H), and longitude (along with its direction),
/// returns right ascension (α).
/// (Peter Duffett-Smith, p.35)
/// * `ut` - UT
/// * `ha` - Hour-angle (H)
/// * `lng` - Longitude
/// * `dir` - Direction for Longitude
pub fn right_ascension_from_ut(&ut: &DateTime, &ha: &Time, lng: f64, dir: Direction) -> Time {
    let gst = gst_from_ut(&ut);
    let gst_0 = DateTime {
        year: ut.year,
        month: ut.month,
        day: ut.day,
        hour: gst.hour,
        min: gst.min,
        sec: gst.sec,
    };

    let lst = lst_from_gst(&gst_0, lng, dir);
    let lst_decimal = decimal_hours_from_time(&lst);
    let ha_decimal = decimal_hours_from_time(&ha);

    let mut asc = lst_decimal - ha_decimal;
    if asc < 0.0 {
        asc += 24.0;
    }

    time_from_decimal_hours(asc)
}

/// Given equatorial coordinate with hour-angle (H), declination (δ),
/// and observer's latitude (φ), returns altitude (a) and azimuth (A)
/// for that of horizontal coordinate.
/// (Peter Duffett-Smith, pp.36-37)
/// * `coord` - Equatorial coordinate (with hour-angle)
/// * `coord.ha` - Hour-angle (H)
/// * `coord.dec` - Declination (δ)
/// * `lat` - Latitude (φ)
pub fn horizon_from_equatorial(coord: EquaCoord2, lat: f64) -> HorizCoord {
    let hour_angle = (decimal_hours_from_time(&coord.ha) * 15.0).to_radians();
    let decline = decimal_hours_from_time(&coord.dec).to_radians();
    let latitude = lat.to_radians();

    let altitude = (
        (decline.sin() * latitude.sin()) +
            (decline.cos() * latitude.cos() * hour_angle.cos())
    ).asin();

    let mut azimuth = (
        (decline.sin() - (latitude.sin() * altitude.sin())) /
            (latitude.cos() * altitude.cos())
    ).acos();

    azimuth = if hour_angle.sin() < 0.0 {
        azimuth
    } else {
        (2.0 * PI) - azimuth
    };

    HorizCoord {
        alt: time_from_decimal_hours(altitude.to_degrees()),
        azi: time_from_decimal_hours(azimuth.to_degrees()),
    }
}

/// Given altitude (a), azimuth (A), and observer's latitude (φ),
/// returns hour-angle (H) and declination (δ)
/// for that of equatorial coordinate.
/// (Peter Duffett-Smith, pp.38-39)
/// * `coord` - Horizontal coordinate
/// * `coord.alt` - Altitude (a)
/// * `coord.azi` - Azimuth (A)
/// * `lat` - Latitude (φ)
pub fn equatorial_from_horizon(coord: HorizCoord, lat: f64) -> EquaCoord2 {
    let altitude = decimal_hours_from_time(&coord.alt).to_radians();
    let azimuth = decimal_hours_from_time(&coord.azi).to_radians();
    let latitude = lat.to_radians();

    let decline = (
        (altitude.sin() * latitude.sin()) +
            (altitude.cos() * latitude.cos() * azimuth.cos())
    ).asin();

    let mut hour_angle = (
        (altitude.sin() - (latitude.sin() * decline.sin())) /
            (latitude.cos() * decline.cos())
    ).acos();

    hour_angle = if azimuth.sin() < 0.0 {
        hour_angle
    } else {
        (2.0 * PI) - hour_angle
    };

    hour_angle /= PI / 12.0;

    EquaCoord2 {
        ha: time_from_decimal_hours(hour_angle),
        dec: time_from_decimal_hours(decline.to_degrees())
    }
}

/// Given LST and hour-angle (H), returns right ascension (α),
/// (Peter Duffett-Smith, p.39)
/// * `lst` - LST
/// * `ha` - Hour-angle (H)
pub fn right_ascension_from_lst_and_hour_angle(&lst: &DateTime, &ha: &Time) -> Time {
    let ha_decimal = decimal_hours_from_time(&ha);
    let time = Time {
        hour: lst.hour,
        min: lst.min,
        sec: lst.sec,
    };
    let lst_decimal = decimal_hours_from_time(&time);
    let mut asc = lst_decimal - ha_decimal;
    if asc < 0.0 {
        asc += 24.0;
    }
    time_from_decimal_hours(asc)
}

/// Given ecliptic ecliptic latitude (β) and longitude (λ)
/// (optionally takes date for specific obliquity of the ecliptic (ε)),
/// returns right ascension (α) and declination (δ)
/// for that of equatorial coordinate.
/// (Peter Duffett-Smith, pp.40-41)
/// * `coord` - Ecliptic coordinate
/// * `coord.lat` - Latitude (β)
/// * `coord.lng` - Longitude (λ)
/// * `date` - Date for specific obliquity of the eplictic (ε)
#[allow(clippy::many_single_char_names)]
pub fn equatorial_from_ecliptic_with_date(coord: EcliCoord, &date: &Date) -> EquaCoord {
    let oblique = mean_obliquity_of_the_epliptic(&date).to_radians();
    let oblique_cos = oblique.cos();
    let oblique_sin = oblique.sin();

    let lat = coord.lat; // latitude (β)
    let lng = coord.lng; // longitude (λ)

    let lat_cos = lat.to_radians().cos();
    let lat_sin = lat.to_radians().sin();
    let lat_tan = lat.to_radians().tan();
    let lng_cos = lng.to_radians().cos();
    let lng_sin = lng.to_radians().sin();

    let decline_0 = (lat_sin * oblique_cos) + (lat_cos * oblique_sin * lng_sin);
    let decline = decline_0.asin().to_degrees();

    let y = (lng_sin * oblique_cos) - (lat_tan * oblique_sin);
    let x = lng_cos;

    let mut asc = y.atan2(x).to_degrees();
    asc -= 360.0 * (asc / 360.0).floor();
    asc /= 15.0;

    EquaCoord {
        asc: time_from_decimal_hours(asc),
        dec: time_from_decimal_hours(decline),
    }
}

pub fn equatorial_from_ecliptic(coord: EcliCoord) -> EquaCoord {
    let date = Date {
        year: 2021,
        month: Month::Jan,
        day: 0.0,
    };
    equatorial_from_ecliptic_with_date(coord, &date)
}

/// Given right ascension (α) and declination (δ) of equatorial coordinate
/// (optionally takes date for specific obliquity of the ecliptic (ε)),
/// returns latitude (β) and longitude (λ)
/// (Peter Duffett-Smith, p.42)
/// * `coord` - Equatorial coordinate
/// * `coord.asc` - Right ascension (α)
/// * `coord.dec` - Declination (δ)
/// * `date` - Date for specific obliquity of the eplictic (ε)
#[allow(clippy::many_single_char_names)]
pub fn ecliptic_from_equatorial_with_date(coord: EquaCoord, &date: &Date) -> EcliCoord {
    let oblique = mean_obliquity_of_the_epliptic(&date).to_radians();
    let oblique_cos = oblique.cos();
    let oblique_sin = oblique.sin();

    let mut asc_decimal = decimal_hours_from_time(&coord.asc).to_radians(); // right ascension (α)
    let dec_decimal = decimal_hours_from_time(&coord.dec).to_radians(); // declination (δ)
    asc_decimal *= 15.0;

    let asc_sin = asc_decimal.sin();
    let asc_cos = asc_decimal.cos();
    let dec_sin = dec_decimal.sin();
    let dec_cos = dec_decimal.cos();
    let dec_tan = dec_decimal.tan();

    let lat_0 = (dec_sin * oblique_cos) - (dec_cos * oblique_sin * asc_sin);
    let lat = lat_0.asin().to_degrees();

    let y = (asc_sin * oblique_cos) + (dec_tan * oblique_sin);
    let x = asc_cos;

    let mut lng = y.atan2(x).to_degrees();
    lng -= 360.0 * (lng / 360.0).floor();

    EcliCoord { lat, lng }
}

/// Given right ascension (α) and declination (δ) of equatorial coordinate,
/// returns galactic latitude (b) and galactic longitude (l).
/// (Peter Duffett-Smith, p.43)
/// * `coord` - Equatorial coordinate
/// * `coord.asc` - Right ascension (α)
/// * `coord.dec` - Declination (δ)
#[allow(clippy::many_single_char_names)]
pub fn galactic_from_equatorial(coord: EquaCoord) -> GalacCoord {
    let mut asc_decimal = decimal_hours_from_time(&coord.asc).to_radians(); // right ascension (α)
    let dec_decimal = decimal_hours_from_time(&coord.dec).to_radians(); // declination (δ)
    asc_decimal *= 15.0;

    let dec_sin = dec_decimal.sin();
    let dec_cos = dec_decimal.cos();
    let asc_192 = (asc_decimal.to_degrees() - 192.25).to_radians();

    let r_27 = (27.4_f64).to_radians();
    let r_27_sin = r_27.sin();
    let r_27_cos = r_27.cos();

    let b_sin = dec_cos * r_27.cos() * asc_192.cos() + (dec_sin * r_27_sin);
    let b = b_sin.asin();

    let y = dec_sin - (b_sin * r_27_sin);
    let x = dec_cos * asc_192.sin() * r_27_cos;

    let mut l = y.atan2(x).to_degrees();
    l -= 360.0 * (l / 360.0).floor();
    l += 33.0;

    GalacCoord {
        lat: b.to_degrees(),
        lng: l,
    }
}

/// Given galactic latitude (b) and galactic longitude (l),
/// returns right ascension (α) and declination (δ) of equatorial coordinate,
/// (Peter Duffett-Smith, p.44)
/// * `coord` - Galactic coordinate
/// * `coord.lat` - Galactic latitude (b)
/// * `coord.lng` - Galactic longitude (l)
#[allow(clippy::many_single_char_names)]
pub fn equatorial_from_galactic(coord: GalacCoord) -> EquaCoord {
    let b = coord.lat.to_radians(); // // Galactic latitude (b)
    let l = coord.lng.to_radians(); // Galactic longitude (l)
    let b_sin = b.sin();
    let b_cos = b.cos();

    let l_minus_33 = (l.to_degrees() - 33.0).to_radians();
    let l_minus_33_sin = l_minus_33.sin();
    let l_minus_33_cos = l_minus_33.cos();

    let r_27 = (27.4_f64).to_radians();
    let r_27_sin = r_27.sin();
    let r_27_cos = r_27.cos();

    let dec_sin = (b_cos * r_27_cos * l_minus_33_sin) + (b_sin * r_27_sin);
    let dec = dec_sin.asin();

    let y = b_cos * l_minus_33_cos;
    let x = (b_sin * r_27_cos) - (b_cos * r_27_sin * l_minus_33_sin);

    let mut asc = y.atan2(x).to_degrees();
    asc += 192.25;
    asc -= 360.0 * (asc / 360.0).floor();
    asc /= 15.0;

    EquaCoord {
        asc: time_from_decimal_hours(asc),
        dec: time_from_decimal_hours(dec.to_degrees()),
    }
}

/// Given coordinates for two celestial objects expressed in
/// ecliptic coordinate system (latitude (β) and longitude (λ)),
/// returns the angle between them.
/// (Peter Duffett-Smith, p.51)
/// * `coord_0` - Equatorial coordinate
/// * `coord_0.asc` - Right ascension (α)
/// * `coord_0.dec` - Declination (δ)
/// * `coord_1` - Equatorial coordinate
/// * `coord_1.asc` - Right ascension (α)
/// * `coord_1.dec` - Declination (δ)
#[allow(clippy::many_single_char_names)]
pub fn angle_between_two_celestial_objects_for_equatorial(coord_0: EquaCoord, coord_1: EquaCoord) -> f64 {
    angle_between_two_celestial_objects(
        decimal_hours_from_time(&coord_0.asc),
        decimal_hours_from_time(&coord_0.dec),
        decimal_hours_from_time(&coord_1.asc),
        decimal_hours_from_time(&coord_1.dec)
    )
}

/// Given coordinates for two celestial objects expressed in
/// galactic coordinate system (latitude (b) and longitude (l)),
/// returns the angle between them.
/// (Peter Duffett-Smith, p.51)
/// * `coord_0` - Galactic coordinate
/// * `coord_0.lat` - Galactic latitude (b)
/// * `coord_0.lng` - Galactic longitude (l)
/// * `coord_1` - Galactic coordinate
/// * `coord_1.lat` - Galactic latitude (b)
/// * `coord_1.lng` - Galactic longitude (l)
#[allow(clippy::many_single_char_names)]
pub fn angle_between_two_celestial_objects_for_galactic(coord_0: GalacCoord, coord_1: GalacCoord) -> f64 {
    angle_between_two_celestial_objects(
        coord_0.lat,
        coord_0.lng,
        coord_1.lat,
        coord_1.lng
    )
}

#[allow(clippy::many_single_char_names)]
pub fn angle_between_two_celestial_objects(asc_0: f64, dec_0: f64, asc_1: f64, dec_1: f64) -> f64 {
    let tmp = ((asc_0 - asc_1) * 15.0).to_radians();
    let dec_0 = dec_0.to_radians();
    let dec_1 = dec_1.to_radians();
    let d_cos = (dec_0.sin() * dec_1.sin()) +
        (
            dec_0.cos() * dec_1.cos() * tmp.cos()
        );
    d_cos.acos().to_degrees()
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx_eq::assert_approx_eq;

    #[test]
    fn hour_angle_from_ut_works() {
        let dir = Direction::West;
        let lng = 64.0;

        let asc = Time {
            hour: 18,
            min: 32,
            sec: 21.0,
        };

        let ut = DateTime {
            year: 1980,
            month: Month::Apr,
            day: 22.0,
            hour: 14,
            min: 36,
            sec: 51.67,
        };

        let hour_angle = hour_angle_from_ut(&ut, &asc, lng, dir);

        assert_eq!(hour_angle.hour, 5);
        assert_eq!(hour_angle.min, 51);
        assert_approx_eq!(
            hour_angle.sec, // 44.22957675918951
            44.0,
            1e-2
        );
    }

    #[test]
    fn right_ascension_from_ut_works() {
        let dir = Direction::West;
        let lng = 64.0;

        // hour-angle
        let ha = Time {
            hour: 5,
            min: 51,
            sec: 44.0,
        };

        let ut = DateTime {
            year: 1980,
            month: Month::Apr,
            day: 22.0,
            hour: 14,
            min: 36,
            sec: 51.67,
        };

        let asc = right_ascension_from_ut(&ut, &ha, lng, dir);

        assert_eq!(asc.hour, 18);
        assert_eq!(asc.min, 32);
        assert_approx_eq!(
            asc.sec, // 21.229576759189968
            21.0,
            1e-1
        );
    }

    #[test]
    fn horizon_from_equatorial_works() {
        let lat = 52.0;

        // hour-angle
        let ha = Time {
            hour: 5,
            min: 51,
            sec: 44.0,
        };

        // declination
        let dec = Time {
            hour: 23,
            min: 13,
            sec: 10.0,
        };

        let coord_0 = EquaCoord2 { ha, dec };
        let coord = horizon_from_equatorial(coord_0, lat);
        let alt = coord.alt;
        let azi = coord.azi;

        assert_eq!(alt.hour, 19);
        assert_eq!(alt.min, 20);
        assert_approx_eq!(
            alt.sec, // 3.6428077696939454
            4.0,
            1e-0
        );

        assert_eq!(azi.hour, 283);
        assert_eq!(azi.min, 16);
        assert_approx_eq!(
            azi.sec, // 15.698162189496543
            16.0,
            1e-0
        );
    }

    #[test]
    fn equatorial_from_horizon_works() {
        let lat = 52.0;

        // altitude
        let alt = Time {
            hour: 19,
            min: 20,
            sec: 4.0,
        };

        // azimuth
        let azi = Time {
            hour: 283,
            min: 16,
            sec: 16.0,
        };

        let coord_0 = HorizCoord { alt, azi };
        let coord = equatorial_from_horizon(coord_0, lat);
        let ha = coord.ha;
        let dec = coord.dec;

        assert_eq!(ha.hour, 5);
        assert_eq!(ha.min, 51);
        assert_approx_eq!(
            ha.sec, // 43.998769832229954
            44.0,
            1e-0
        );

        assert_eq!(dec.hour, 23);
        assert_eq!(dec.min, 13);
        assert_approx_eq!(
            dec.sec, // 10.456528456985552
            10.0,
            1e-1
        );
    }

    #[test]
    fn right_ascension_from_lst_and_hour_angle_works() {
        let lst = DateTime {
            year: 1980,
            month: Month::Apr,
            day: 22.0,
            hour: 0,
            min: 24,
            sec: 5.0,
        };

        // hour-angle
        let ha = Time {
            hour: 5,
            min: 51,
            sec: 44.0,
        };

        let asc = right_ascension_from_lst_and_hour_angle(&lst, &ha);

        assert_eq!(asc.hour, 18);
        assert_eq!(asc.min, 32);
        assert_approx_eq!(
            asc.sec, // 20.99999999999966
            21.0,
            1e-0
        );
    }

    #[test]
    fn equatorial_from_ecliptic_works() {
        let lat_0 = Time {
            hour: 4,
            min: 52,
            sec: 31.0,
        };

        let lng_0 = Time {
            hour: 139,
            min: 41,
            sec: 10.0,
        };

        let coord_0 = EcliCoord {
            lat: decimal_hours_from_time(&lat_0),
            lng: decimal_hours_from_time(&lng_0),
        };

        // To calculate a specific value
        // for mean obliquity of the ecliptic.
        let date = Date {
            year: 1980,
            month: Month::Apr,
            day: 22.0,
        };

        let coord = equatorial_from_ecliptic_with_date(coord_0, &date);
        let asc = coord.asc;
        let dec = coord.dec;

        assert_eq!(asc.hour, 9);
        assert_eq!(asc.min, 34);
        assert_approx_eq!(
            asc.sec, // 53.58216253599352
            53.6,
            1e-2
        );

        assert_eq!(dec.hour, 19);
        assert_eq!(dec.min, 32);
        assert_approx_eq!(
            dec.sec, // 14.100993558899972
            14.2,
            1e-2
        );
    }

    #[test]
    fn ecliptic_from_equatorial_works() {
        // right-ascension
        let asc = Time {
            hour: 9,
            min: 34,
            sec: 53.6,
        };
        // declination
        let dec = Time {
            hour: 19,
            min: 32,
            sec: 14.2,
        };

        // To calculate a specific value
        // for mean obliquity of the ecliptic.
        let date = Date {
            year: 1980,
            month: Month::Apr,
            day: 22.0,
        };

        let coord_0 = EquaCoord { asc, dec };
        let coord = ecliptic_from_equatorial_with_date(coord_0, &date);
        let lat = time_from_decimal_hours(coord.lat);
        let lng = time_from_decimal_hours(coord.lng);

        assert_eq!(lat.hour, 4);
        assert_eq!(lat.min, 52);
        assert_approx_eq!(
            lat.sec, // 31.17490012745307
            31.0,
            1e-2
        );

        assert_eq!(lng.hour, 139);
        assert_eq!(lng.min, 41);
        assert_approx_eq!(
            lng.sec, // 10.207621429881328
            10.0,
            3e-2
        );
    }

    #[test]
    fn galactic_from_equatorial_works() {
        // right-ascension
        let asc = Time {
            hour: 10,
            min: 21,
            sec: 0.0,
        };

        // declination
        let dec = Time {
            hour: 10,
            min: 3,
            sec: 11.0,
        };

        let coord_0 = EquaCoord { asc, dec };
        let coord = galactic_from_equatorial(coord_0);
        let lat = time_from_decimal_hours(coord.lat);
        let lng = time_from_decimal_hours(coord.lng);

        assert_eq!(lat.hour, 51);
        assert_eq!(lat.min, 7);
        assert_approx_eq!(
            lat.sec, // 20.16407768754391
            20.0,
            1e-2
        );

        assert_eq!(lng.hour, 232);
        assert_eq!(lng.min, 14);
        assert_approx_eq!(
            lng.sec, // 52.38055557683538
            52.0,
            1e-2
        );
    }

    #[test]
    fn equatorial_from_galactic_works() {
        // Galactic latitude (b)
        let lat = Time {
            hour: 51,
            min: 7,
            sec: 20.0,
        };

        // Galactic latitude (l)
        let lng = Time {
            hour: 232,
            min: 14,
            sec: 52.0,
        };

        let coord_0 = GalacCoord {
            lat: decimal_hours_from_time(&lat),
            lng: decimal_hours_from_time(&lng),
        };
        let coord = equatorial_from_galactic(coord_0);
        let asc = coord.asc;
        let dec = coord.dec;

        assert_eq!(asc.hour, 10);
        // TODO:
        // The book tells it should be 21 for 'asc.min'...
        assert_eq!(asc.min, 20);
        assert_approx_eq!(
            asc.sec, // 59.98205693746215
            59.9,
            1e-2
        );

        assert_eq!(dec.hour, 10);
        assert_eq!(dec.min, 3);
        assert_approx_eq!(
            dec.sec, // 11.117231829019829
            11.0,
            2e-2
        );
    }

    #[test]
    fn angle_between_two_celestial_objects_works() {
        // right-ascension (for Beta Orionis)
        let asc_0 = Time {
            hour: 5,
            min: 13,
            sec: 31.7,
        };

        // declination (for Beta Orionis)
        let dec_0 = Time {
            hour: -8,
            min: 13,
            sec: 30.0,
        };

        // right-ascension (for Canis Majoris)
        let asc_1 = Time {
            hour: 6,
            min: 44,
            sec: 13.4,
        };

        // declination (for Canis Majoris)
        let dec_1 = Time {
            hour: -16,
            min: 41,
            sec: 11.0,
        };

        let angle = angle_between_two_celestial_objects_for_equatorial(
            EquaCoord { asc: asc_0, dec: dec_0 },
            EquaCoord { asc: asc_1, dec: dec_1 }
        );

        assert_approx_eq!(
            angle, // 23.67384942216419
            23.673_850,
            1e-6
        );
    }
}
