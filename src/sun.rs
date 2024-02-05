use chrono::naive::{
    NaiveDate, NaiveDateTime, NaiveTime,
};
use chrono::offset::Utc;
use chrono::{DateTime, Datelike};

use crate::constants::{
    ECCENTRICITY_OF_ORBIT,
    ECLIPTIC_LONGITUDE_AT_1990,
    ECLIPTIC_LONGITUDE_OF_PERIGEE,
};

use crate::coords::{
    equatorial_from_ecliptic_with_generic_date,
    Angle, EcliCoord, EquaCoord,
};

use crate::time::{
    angle_from_decimal_hours,
    day_number_from_generic_date, days_since_1990,
    decimal_hours_from_naive_time, utc_from_gst,
};

const KEPLER_ACCURACY: f64 = 1e-6; // (ε)

// Private recursive function for 'find_kepler()'.
fn _kepler_aux(
    mean_anom: f64,
    ecc: f64,
    counter: u32,
) -> f64 {
    if counter > 1000 {
        panic!("Dude, this is insane...");
    }
    let delta = ecc
        - (ECCENTRICITY_OF_ORBIT * ecc.sin())
        - mean_anom;
    if delta.abs() > KEPLER_ACCURACY {
        let delta_e = delta
            / (1.0
                - (ECCENTRICITY_OF_ORBIT
                    * ecc.cos()));
        _kepler_aux(
            mean_anom,
            ecc - delta_e,
            counter + 1,
        )
    } else {
        ecc
    }
}

pub fn find_kepler(mean_anom: f64) -> f64 {
    _kepler_aux(mean_anom, mean_anom, 0_u32)
}

/// See 'equatorial_position_of_the_sun_from_date'
/// for the specs.
#[allow(clippy::many_single_char_names)]
pub fn sun_longitude_and_mean_anomaly(
    days: f64,
) -> (f64, f64) {
    let mut n: f64 = (360.0 / 365.242_191) * days;
    n -= 360.0 * (n / 360.0).floor();

    // Mean anomaly (M)
    let mut mean_anom: f64 = n
        + ECLIPTIC_LONGITUDE_AT_1990
        - ECLIPTIC_LONGITUDE_OF_PERIGEE;

    if mean_anom < 0.0 {
        mean_anom += 360.0;
    }

    // Eccentric anomaly (E)
    let ecc: f64 =
        find_kepler(mean_anom.to_radians());

    // True anomaly (v)
    // (the true motion of the sun in an ellipse)
    let mut v: f64 = ((1.0 + ECCENTRICITY_OF_ORBIT)
        / (1.0 - ECCENTRICITY_OF_ORBIT))
        .sqrt()
        * (ecc / 2.0).tan();
    v = (v.atan() * 2.0).to_degrees();

    // Sun's longitude (λ)
    let mut lng: f64 =
        v + ECLIPTIC_LONGITUDE_OF_PERIGEE;

    if lng > 360.0 {
        lng -= 360.0;
    }

    if lng < 0.0 {
        lng += 360.0;
    }

    (lng, mean_anom)
}

pub fn ecliptic_position_of_the_sun_from_generic_date<
    T,
>(
    date: T,
) -> EcliCoord
where
    T: Datelike,
    T: std::marker::Copy,
    T: std::fmt::Debug,
    T: std::fmt::Display,
{
    let day_number =
        day_number_from_generic_date(date) as f64;
    let days: f64 = days_since_1990(date.year())
        as f64
        + day_number;

    let (lng, _mean_anom): (f64, f64) =
        sun_longitude_and_mean_anomaly(days);

    EcliCoord { lat: 0.0, lng }
}

/// Given a specific date, returns right ascension (α)
/// and declination (δ) for that of equatorial
/// coordinate. Greek letters are assigned in Duffet-
/// Smith's for the following:
/// ECLIPTIC_LONGITUDE_AT_1990 --> Epsilon G (ε g)
/// ECLIPTIC_LONGITUDE_OF_PERIGEE
///     --> Omega bar G (ω bar g)
///
/// * `date` - Datelike
///
/// Reference:
/// - (Peter Duffett-Smith, p.91)
///
/// Example:
/// ```rust
/// use approx_eq::assert_approx_eq;
/// use chrono::naive::NaiveDate;
/// use sowngwala::coords::{Angle, EquaCoord};
/// use sowngwala::sun::equatorial_position_of_the_sun_from_generic_date;
///
/// let date: NaiveDate = NaiveDate::from_ymd(1988, 7, 27);
/// let coord: EquaCoord =
///     equatorial_position_of_the_sun_from_generic_date(date);
/// let asc: Angle = coord.asc;
/// let dec: Angle = coord.dec;
///
/// assert_eq!(asc.hour(), 8);
/// assert_eq!(asc.minute(), 26);
/// assert_approx_eq!(
///     asc.second(), // 3.8050320752654443
///     4.0,
///     1e-1
/// );
///
/// assert_eq!(dec.hour(), 19);
/// assert_eq!(dec.minute(), 12);
/// assert_approx_eq!(
///     dec.second(), // 42.522657925921976
///     42.0,
///     5e-2
/// );
/// ```
pub fn equatorial_position_of_the_sun_from_generic_date<
    T,
>(
    date: T,
) -> EquaCoord
where
    T: Datelike,
    T: std::marker::Copy,
    T: std::fmt::Debug,
    T: std::fmt::Display,
{
    equatorial_from_ecliptic_with_generic_date(
        ecliptic_position_of_the_sun_from_generic_date(date),
        date,
    )
}

/// Given the date in GST, returns the EOT.
/// (Peter Duffett-Smith, pp.98-99)
#[allow(clippy::many_single_char_names)]
pub fn equation_of_time_from_gst(
    gst: NaiveDateTime,
) -> (Angle, f64) {
    let date: NaiveDate = gst.date();
    let coord: EquaCoord =
        equatorial_position_of_the_sun_from_generic_date(date);
    let asc_0: Angle = coord.asc;
    let asc_1: NaiveTime = asc_0.into();
    let naivetime = NaiveDateTime::new(date, asc_1);
    let utc: NaiveTime = utc_from_gst(naivetime);
    let decimal = decimal_hours_from_naive_time(utc);
    let e = 12.0 - decimal;
    let mut angle_0 = angle_from_decimal_hours(e);
    let day_excess = angle_0.calibrate();
    (angle_0, day_excess)
}

#[allow(clippy::many_single_char_names)]
pub fn equation_of_time_from_utc(
    utc: DateTime<Utc>,
) -> (Angle, f64) {
    equation_of_time_from_gst(utc.naive_utc())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::coords::EcliCoord;
    use chrono::naive::NaiveDate;

    #[test]
    fn see_if_you_can_find_monthly_zhi() {
        // 立夏 (Li-xia) for 2022 starts on 5/5.
        // So, it should result in `3` for 5/6.

        let date: NaiveDate =
            NaiveDate::from_ymd(2022, 5, 6);

        let ecliptic: EcliCoord =
            ecliptic_position_of_the_sun_from_generic_date(date);

        let lng: f64 = ecliptic.lng;

        let branch: usize =
            if (315.0..345.0).contains(&lng) {
                0 // 立春 (lichun) + 雨水 (yushui) ---> 寅 (yin)
            } else if !(15.0..345.0).contains(&lng) {
                1 // 啓蟄 (jingzhe) + 春分 (chunfen) ---> 卯 (mao)
            } else if (15.0..45.0).contains(&lng) {
                2 // 清明 (qingming) + 穀雨 (guyu) ---> 辰 (chen)
            } else if (45.0..75.0).contains(&lng) {
                3 // 立夏 (lixia) + 小滿 (xiaoman) ---> 巳 (si)
            } else if (75.0..105.0).contains(&lng) {
                4 // 芒種 (mangzhong) + 夏至 (xiazhi) ---> 午 (wu)
            } else if (105.0..135.0).contains(&lng) {
                5 // 小暑 (xiaoshu) + 大暑 (dashu) ---> 未 (wei)
            } else if (135.0..165.0).contains(&lng) {
                6 // 立秋 (liqiu) + 處暑 (chushu) ---> 申 (shen)
            } else if (165.0..195.0).contains(&lng) {
                7 // 白露 (bailu) + 秋分 (qiufen) ---> 酉 (you)
            } else if (195.0..225.0).contains(&lng) {
                8 // 寒露 (hanlu) + 霜降 (shuangjiang) ---> 戌 (xu)
            } else if (225.0..255.0).contains(&lng) {
                9 // 立冬 (lidong) + 小雪 (xiaoxue) ---> 亥 (hai)
            } else if (255.0..285.0).contains(&lng) {
                10 // 大雪 (daxue) + 冬至 (dongzhi) ---> 子 (zi)
            } else {
                // lng >= 285.0 || lng < 315.0
                11 // 小寒 (xiaohan) + 大寒 (dahan) ---> 丑 (chou)
            };

        assert_eq!(branch, 3);
    }
}
