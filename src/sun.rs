#[cfg(test)]
extern crate approx_eq;
#[cfg(test)]
use crate::time::Month;

use crate::constants::{
    ECCENTRICITY_OF_ORBIT,
    ECLIPTIC_LONGITUDE_AT_1990,
    ECLIPTIC_LONGITUDE_OF_PERIGEE
};
use crate::coords::{
    EcliCoord,
    EquaCoord,
    equatorial_from_ecliptic_with_date
};
use crate::time::{
    DateTime,
    Date,
    Time,
    day_number_from_date,
    days_since_1990,
    decimal_hours_from_time,
    time_from_decimal_hours,
    ut_from_gst,
    gst_from_ut,
};

const KEPLER_ACCURACY: f64 = 1e-6; // (ε)

// Private recursive function for 'find_kepler()'.
fn _kepler_aux(mean_anom: f64, ecc: f64, counter: u32) -> f64 {
    if counter > 1000 {
        panic!("Dude, this is insane...");
    }
    let delta = ecc - (ECCENTRICITY_OF_ORBIT * ecc.sin()) - mean_anom;
    if delta.abs() > KEPLER_ACCURACY {
        let delta_e = delta / (1.0 - (ECCENTRICITY_OF_ORBIT * ecc.cos()));
        _kepler_aux(mean_anom, ecc - delta_e, counter + 1)
    } else {
        ecc
    }
}

/// Takes mean anomaly, and returns the eccentric anomaly.
/// (Peter Duffett-Smith, p.90)
/// * `mean_anom` - Mean anomaly (M) (in radians)
pub fn find_kepler(mean_anom: f64) -> f64 {
    _kepler_aux(mean_anom, mean_anom, 0_u32)
}

/// See 'equatorial_position_of_the_sun_from_date' for the specs.
#[allow(clippy::many_single_char_names)]
pub fn sun_longitude_and_mean_anomaly(days: f64) -> (f64, f64) {
    let mut n: f64 = (360.0 / 365.242_191) * days;
    n -= 360.0 * (n / 360.0).floor();

    // Mean anomaly (M)
    let mut mean_anom: f64 =
        n + ECLIPTIC_LONGITUDE_AT_1990 - ECLIPTIC_LONGITUDE_OF_PERIGEE;

    if mean_anom < 0.0 {
        mean_anom += 360.0;
    }

    // Eccentric anomaly (E)
    let ecc: f64 = find_kepler(mean_anom.to_radians());

    // True anomaly (v)
    // (the true motion of the sun in an ellipse)
    let mut v: f64 = (
        (1.0 + ECCENTRICITY_OF_ORBIT) / (1.0 - ECCENTRICITY_OF_ORBIT)
    ).sqrt() *
        (ecc / 2.0).tan();
    v = (v.atan() * 2.0).to_degrees();

    // Sun's longitude (λ)
    let mut lng: f64 = v + ECLIPTIC_LONGITUDE_OF_PERIGEE;

    if lng > 360.0 {
        lng -= 360.0;
    }

    if lng < 0.0 {
        lng += 360.0;
    }

    (lng, mean_anom)
}

pub fn ecliptic_position_of_the_sun_from_date(&date: &Date) -> EcliCoord {
    let day_number = day_number_from_date(&date) as f64;
    let days: f64 = days_since_1990(date.year) + day_number;
    let (lng, _mean_anom): (f64, f64) = sun_longitude_and_mean_anomaly(days);
    EcliCoord { lat: 0.0, lng }
}

/// Given a specific date, returns right ascension (α) and declination (δ)
/// for that of equatorial coordinate.
/// Greek letters are assigned in Duffet-Smith's for the following:
/// ECLIPTIC_LONGITUDE_AT_1990 --> Epsilon G (ε g)
/// ECLIPTIC_LONGITUDE_OF_PERIGEE --> Omega bar G (ω bar g)
/// (Peter Duffett-Smith, p.91)
/// * `date` - Date
pub fn equatorial_position_of_the_sun_from_date(&date: &Date) -> EquaCoord {
    equatorial_from_ecliptic_with_date(
        ecliptic_position_of_the_sun_from_date(&date),
        &date
    )
}

/// Given the date in GST, returns the EOT.
/// (Peter Duffett-Smith, pp.98-99)
#[allow(clippy::many_single_char_names)]
pub fn equation_of_time_from_gst(&gst: &DateTime) -> Time {
    let date = Date {
        year: gst.year,
        month: gst.month,
        day: gst.day,
    };
    let coord = equatorial_position_of_the_sun_from_date(&date);
    let asc = coord.asc;
    let ut = ut_from_gst(&DateTime::from(&date, &asc));
    let decimal = decimal_hours_from_time(&ut);
    let e = 12.0 - decimal;

    time_from_decimal_hours(e)
}

#[allow(clippy::many_single_char_names)]
pub fn equation_of_time_from_ut(&ut: &DateTime) -> Time {
    equation_of_time_from_gst(
        &DateTime::from(
            &Date::from(&ut),
            &gst_from_ut(&ut)
        )
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx_eq::assert_approx_eq;

    #[test]
    fn find_kepler_works() {
        let mean_anom = 3.527_781;
        let ecc_anom = find_kepler(mean_anom);
        assert_approx_eq!(
            ecc_anom, // 3.521581853477305
            3.521_581,
            1e-6
        );
    }

    #[test]
    fn equatorial_position_of_the_sun_from_date_works() {
        let date = Date {
            year: 1988,
            month: Month::Jul,
            day: 27.0,
        };
        let coord = equatorial_position_of_the_sun_from_date(&date);
        let asc = coord.asc;
        let dec = coord.dec;

        assert_eq!(asc.hour, 8);
        assert_eq!(asc.min, 26);
        assert_approx_eq!(
            asc.sec, // 3.8050320752654443
            4.0,
            1e-1
        );

        assert_eq!(dec.hour, 19);
        assert_eq!(dec.min, 12);
        assert_approx_eq!(
            dec.sec, // 42.522657925921976
            42.0,
            5e-2
        );
    }

    #[test]
    fn equation_of_time_from_gst_works() {
        let gst = DateTime {
            year: 1980,
            month: Month::Jul,
            day: 27.5,
            hour: 0,
            min: 0,
            sec: 0.0,
        };

        let eot = equation_of_time_from_gst(&gst);
        let decimal = decimal_hours_from_time(&eot);

        // dec:
        //   Time { hour: 0, min: -2, sec: 33.3100561387684 }
        // where expected:
        //   Time { hour: 0, min: -6, sec: 25.0 }
        assert_approx_eq!(
            decimal, // -0.042586126705213445
            -0.10694444444444445,
            2.0
        );
    }
}
