use chrono::naive::{
    NaiveDate, NaiveDateTime, NaiveTime,
};
use chrono::offset::{FixedOffset, TimeZone, Utc};
use chrono::{
    DateTime, Datelike, Duration, Timelike,
};
use core::ops::Add;

use crate::constants::{
    J2000, NUM_OF_DAYS_IN_A_YEAR,
};
use crate::coords::{Angle, Direction};
use crate::sun::equation_of_time_from_utc;
use crate::utils::overflow;

/// A handy tool to build `DateTime<FixedOffset>`.
///
/// Example:
/// ```rust
/// use chrono::{
///     DateTime,
///     Datelike,
///     Timelike,
/// };
/// use chrono::offset::FixedOffset;
/// use sowngwala::time::build_fixed;
///
/// let zone: i32 = 4;
/// let result: DateTime<FixedOffset> =
///     build_fixed(2021, 1, 1, 22, 37, 0, 0, zone);
///
/// assert_eq!(result.hour(), 22);
/// assert_eq!(result.minute(), 37);
/// assert_eq!(result.second(), 0);
/// ```
#[allow(clippy::too_many_arguments)]
pub fn build_fixed(
    year: i32,
    month: u32,
    day: u32,
    hour: u32,
    min: u32,
    sec: u32,
    nano: u32,
    zone: i32,
) -> DateTime<FixedOffset> {
    FixedOffset::east(zone * 3600)
        .ymd(year, month, day)
        .and_hms_nano(hour, min, sec, nano)
}

/// A handy tool to build `DateTime<Utc>`.
///
/// Example:
/// ```rust
/// use chrono::{
///     DateTime,
///     Datelike,
///     Timelike,
/// };
/// use chrono::offset::Utc;
/// use sowngwala::time::build_utc;
///
/// let utc: DateTime<Utc> =
///     build_utc(2021, 1, 1, 22, 37, 0, 0);
///
/// assert_eq!(utc.hour(), 22);
/// assert_eq!(utc.minute(), 37);
/// assert_eq!(utc.second(), 0);
/// assert_eq!(utc.nanosecond(), 0);
/// ```
pub fn build_utc(
    year: i32,
    month: u32,
    day: u32,
    hour: u32,
    min: u32,
    sec: u32,
    nano: u32,
) -> DateTime<Utc> {
    Utc.ymd(year, month, day)
        .and_hms_nano(hour, min, sec, nano)
        .with_timezone(&Utc)
}

/// Given the specific date and time, returns right
/// ascension (α) and declination (δ) of equatorial
/// coordinate.
///
/// * `dt` - DateTime
///
/// Reference:
/// - (Peter Duffett-Smith, p.144)
///
/// Example:
/// ```rust
/// use chrono::Datelike;
/// use chrono::naive::{NaiveDate, NaiveDateTime};
/// use sowngwala::time::naive_date_from_generic_datetime;
///
/// let dt: NaiveDateTime =
///     NaiveDate::from_ymd(1979, 2, 26).and_hms(16, 0, 0);
///
/// let date: NaiveDate = naive_date_from_generic_datetime(dt);
///
/// assert_eq!(date.year(), 1979);
/// assert_eq!(date.month(), 2);
/// assert_eq!(date.day(), 26);
/// ```
pub fn naive_date_from_generic_datetime<T>(
    dt: T,
) -> NaiveDate
where
    T: Datelike,
    T: Timelike,
    T: std::marker::Copy,
    T: std::fmt::Debug,
    T: std::fmt::Display,
{
    NaiveDate::from_ymd(
        dt.year(),
        dt.month(),
        dt.day(),
    )
}

pub fn naive_time_from_generic_datetime<T>(
    dt: T,
) -> NaiveTime
where
    T: Datelike,
    T: Timelike,
    T: std::fmt::Display,
{
    NaiveTime::from_hms_nano(
        dt.hour(),
        dt.minute(),
        dt.second(),
        dt.nanosecond(),
    )
}

/// We define the decimal year "y" as follows:
///
///   y = year + (month - 0.5) / 12
///
/// This gives "y" for the middle of the month, which
/// is accurate enough given the precision in the known
/// values of ΔT. The following polynomial expressions
/// can be used calculate the value of ΔT (in seconds)
/// over the time period covered by of the Five
/// Millennium Canon of Solar Eclipses: -1999 to +3000.
pub fn decimal_year_from_generic_date<T>(
    date: T,
) -> f64
where
    T: Datelike,
    T: std::marker::Copy,
    T: std::fmt::Debug,
    T: std::fmt::Display,
{
    (date.year() as f64)
        + (date.month() as f64 - 0.5) / 12.0
}

/// Converts `NativeTime` into Decimal Hours.
///
/// Reference:
/// - (Peter Duffett-Smith, p.10)
/// - sowngwala::time::decimal_hours_from_time
///
/// Example:
/// ```rust
/// use approx_eq::assert_approx_eq;
/// use chrono::naive::NaiveTime;
/// use sowngwala::time::decimal_hours_from_naive_time;
///
/// let t = NaiveTime::from_hms_nano(18, 31, 27, 0);
/// let hours = decimal_hours_from_naive_time(t);
/// assert_approx_eq!(
///     hours, // 18.524166666666666
///     18.52417,
///     1e-6
/// );
/// ```
pub fn decimal_hours_from_naive_time(
    t: NaiveTime,
) -> f64 {
    let hour = t.hour() as f64;
    let min = t.minute() as f64;

    // A bit differs from Duffett-Smith's
    let sec_0 =
        (t.nanosecond() as f64) / 1_000_000_000_f64;

    let sec = (t.second() as f64) + sec_0;

    let dec: f64 =
        hour + ((min + (sec / 60.0)) / 60.0);

    if hour < 0.0 || min < 0.0 || sec < 0.0 {
        -dec
    } else {
        dec
    }
}

/// Converts `NativeTime` into Decimal Hours.
///
/// Reference:
/// - (Peter Duffett-Smith, p.10)
/// - sowngwala::time::decimal_hours_from_time
///
/// Example:
/// ```rust
/// use approx_eq::assert_approx_eq;
/// use chrono::naive::NaiveTime;
/// use sowngwala::time::decimal_hours_from_generic_time;
///
/// let t = NaiveTime::from_hms_nano(18, 31, 27, 0);
/// let hours = decimal_hours_from_generic_time(t);
/// assert_approx_eq!(
///     hours, // 18.524166666666666
///     18.52417,
///     1e-6
/// );
/// ```
pub fn decimal_hours_from_generic_time<T>(t: T) -> f64
where
    T: Timelike,
    T: std::marker::Copy,
    T: std::fmt::Debug,
    T: std::fmt::Display,
{
    let hour = t.hour() as f64;
    let min = t.minute() as f64;

    // A bit differs from Duffett-Smith's
    let sec_0 =
        (t.nanosecond() as f64) / 1_000_000_000_f64;

    let sec = (t.second() as f64) + sec_0;

    let dec: f64 =
        hour + ((min + (sec / 60.0)) / 60.0);

    if hour < 0.0 || min < 0.0 || sec < 0.0 {
        -dec
    } else {
        dec
    }
}

pub fn decimal_hours_from_angle(angle: Angle) -> f64 {
    let hour = angle.hour().abs() as f64;
    let min = angle.minute().abs() as f64;

    let sec: f64 = angle.second().abs();

    let dec: f64 =
        hour + ((min + (sec / 60.0)) / 60.0);

    if angle.hour() < 0
        || angle.minute() < 0
        || angle.second() < 0.0
    {
        -dec
    } else {
        dec
    }
}

/// Not in use...
pub fn decimal_days_from_generic_datetime<T>(
    dt: T,
) -> f64
where
    T: Datelike,
    T: Timelike,
    T: std::marker::Copy,
    T: std::fmt::Debug,
    T: std::fmt::Display,
{
    let t: NaiveTime =
        naive_time_from_generic_datetime(dt);
    let dec: f64 = decimal_hours_from_generic_time(t);
    (dt.day() as f64) + (dec / 24.0)
}

// Carry-over utils (1)
pub fn hms_from_decimal_hours(
    dec: f64,
) -> (i32, i32, f64) {
    let hour = dec.floor() as i32;
    let base_0: f64 = dec.fract() * 60.0;
    let min = base_0.floor() as i32;
    let sec: f64 = base_0.fract() * 60.0;

    (hour, min, sec)
}

// Carry-over utils (2)
pub fn nano_from_second(sec_0: f64) -> (u32, u32) {
    let sec = sec_0.floor() as u32;
    let base_0: f64 = sec_0.fract() * 1_000_000_000.0;
    let nano = base_0.floor() as u32;
    (sec, nano)
}

/// Convert Decimal Hours into `NaiveTime`.
///
/// References:
/// - (Peter Duffett-Smith, p.11)
/// - sowngwala::time::time_from_decimal_hours;
///
/// Example:
/// ```rust
/// use chrono::Timelike;
/// use chrono::naive::NaiveTime;
/// use sowngwala::time::naive_time_from_decimal_hours;
///
/// let t: NaiveTime =
///     naive_time_from_decimal_hours(18.52417);
///
/// assert_eq!(t.hour(), 18);
/// assert_eq!(t.minute(), 31);
/// assert_eq!(t.second(), 27);
/// ```
#[allow(clippy::many_single_char_names)]
pub fn naive_time_from_decimal_hours(
    dec: f64,
) -> NaiveTime {
    let angle: Angle = angle_from_decimal_hours(dec);
    angle.to_naive_time()
}

#[allow(clippy::many_single_char_names)]
pub fn angle_from_decimal_hours(dec: f64) -> Angle {
    let sign: i16 = if dec < 0.0 { -1 } else { 1 };

    let (h, m, s): (i32, i32, f64) =
        hms_from_decimal_hours(dec.abs());

    let mut hour: i32 = h;
    let mut min: i32 = m;
    let mut sec: f64 = s;

    if hour != 0 {
        hour *= sign as i32;
    } else if min != 0 {
        min *= sign as i32;
    } else {
        sec *= sign as f64;
    }

    Angle::new(hour, min, sec)
}

/// Example:
/// ```rust
/// use chrono::naive::NaiveTime;
/// use chrono::Timelike;
/// use sowngwala::time::naive_time_from_decimal_days;
///
/// let (day, naive) =
///     naive_time_from_decimal_days(17.25);
///
/// assert_eq!(day, 17);
/// assert_eq!(naive.hour(), 6);
/// ```
pub fn naive_time_from_decimal_days(
    days: f64,
) -> (u32, NaiveTime) {
    let day = days.floor() as u32;
    let dec: f64 = days.fract() * 24.0;
    let naive = naive_time_from_decimal_hours(dec);
    (day, naive)
}

/// Checks whether the given Date is julian day.
///
/// Example:
/// ```rust
/// use chrono::naive::NaiveDate;
/// use sowngwala::time::is_julian_date;
///
/// let date = NaiveDate::from_ymd(1582, 10, 14);
/// assert_eq!(is_julian_date(date), true);
///
/// let date = NaiveDate::from_ymd(1582, 10, 15);
/// assert_eq!(is_julian_date(date), false);
/// ```
pub fn is_julian_date<T>(date: T) -> bool
where
    T: Datelike,
    T: std::fmt::Display,
{
    if date.year() > 1582 {
        return false;
    }
    if date.year() < 1582 {
        return true;
    }
    if date.month() > 10 {
        return false;
    }
    if date.month() < 10 {
        return true;
    }
    if date.day() > 14 {
        return false;
    }
    true
}

/// Check if the given year is a leap year.
pub fn is_leap_year(year: i32) -> bool {
    if year % 4 == 0 {
        if year % 100 == 0 {
            year % 400 == 0
        } else {
            true
        }
    } else {
        false
    }
}

/// Finds the day number from date.
/// (Peter Duffett-Smith, p.5)
///
/// Example:
/// ```rust
/// use chrono::naive::NaiveDate;
/// use sowngwala::time::day_number_from_generic_date;
///
/// // ### Example 1
/// let date = NaiveDate::from_ymd(1985, 2, 17);
/// assert_eq!(day_number_from_generic_date(date), 48);
///
/// // ### Example 2
/// let date = NaiveDate::from_ymd(1988, 7, 27);
/// assert_eq!(day_number_from_generic_date(date), 209);
/// ```
#[allow(clippy::many_single_char_names)]
pub fn day_number_from_generic_date<T>(date: T) -> u32
where
    T: Datelike,
    T: std::fmt::Display,
{
    let tmp: f64 = if is_leap_year(date.year()) {
        62.0
    } else {
        63.0
    };

    let mut num = date.month() as f64;
    if num <= 2.0 {
        num = ((num - 1.0) * tmp / 2.0).floor();
    } else {
        num = ((num + 1.0) * 30.6).floor() - tmp;
    }

    (num as u32) + date.day()
}

/// Note:
/// Regardless of the month, the diff is of "Jan 0th".
/// Say, for "July 27th, 1988", it will be the diff
/// between:
///
///    Jan 0th, 1988
///
/// and
///
///    Jan 0th, 1990
///
/// which is obviously -2 years, and the result being:
///
/// -731
///
/// or
///
/// (365 * -2 years) - 1
///
/// where "-1" is for 1988, a leap year.
#[allow(clippy::many_single_char_names)]
pub fn days_since_1990(year: i32) -> i32 {
    let mut year_0: i32 = year;
    let mut days: i32 = 0;

    if year - 1990 < 0 {
        while year_0 < 1990 {
            let leap = is_leap_year(year_0);
            days -= 365;
            if leap {
                days -= 1;
            }
            year_0 += 1;
        }
    } else {
        while year_0 > 1990 {
            let leap = is_leap_year(year_0);
            days += 365;
            if leap {
                days += 1;
            }
            year_0 -= 1;
        }
    }

    days
}

/// Converts a generic datetime into Julian Day.
///
/// Example:
/// ```rust
/// use chrono::Timelike;
/// use chrono::{NaiveDate, NaiveTime, NaiveDateTime};
/// use sowngwala::time::{
///   julian_day_from_generic_datetime,
///   naive_time_from_decimal_hours,
/// };
///
/// let date = NaiveDate::from_ymd(1985, 2, 17);
/// let time: NaiveTime =
///     naive_time_from_decimal_hours(6.0);
/// let dt = NaiveDateTime::new(date, time);
///
/// assert_eq!(
///     julian_day_from_generic_datetime(dt),
///     2_446_113.75
/// );
/// ```
#[allow(clippy::many_single_char_names)]
pub fn julian_day_from_generic_datetime<T>(
    dt: T,
) -> f64
where
    T: Datelike,
    T: Timelike,
    T: std::marker::Copy,
    T: std::fmt::Debug,
    T: std::fmt::Display,
{
    julian_day(
        dt.year(),
        dt.month(),
        decimal_days_from_generic_datetime(dt),
    )
}

#[allow(clippy::many_single_char_names)]
pub fn julian_day_from_generic_date<T>(date: T) -> f64
where
    T: Datelike,
    T: std::marker::Copy,
    T: std::fmt::Debug,
    T: std::fmt::Display,
{
    julian_day(
        date.year(),
        date.month(),
        date.day() as f64,
    )
}

/// Converts a generic datetime into Julian Day. It is
/// a bit different from that of Duffett-Smith.
/// For one of the function arguments `day`, Duffett-
/// Smith suggests a float (ex. 7.5). Whereas we want
/// `u32` because `NaiveDate` would not accept float
/// for `day`. So, the idea is to use `NaiveDateTime`,
/// and include the excess (which is 0.5) into
/// `NaiveTime` already.
///
/// References:
/// - (Peter Duffett-Smith, pp.6-7)
///
/// Example:
/// ```rust
/// use chrono::Timelike;
/// use chrono::{NaiveDate, NaiveTime, NaiveDateTime};
/// use sowngwala::time::{
///   julian_day,
///   naive_time_from_decimal_hours,
/// };
///
/// let year: i32 = 1985;
/// let month: u32 = 2;
/// let day: f64 = 17.25;
///
/// assert_eq!(
///     julian_day(year, month, day),
///     2_446_113.75
/// );
/// ```
#[allow(clippy::many_single_char_names)]
pub fn julian_day(
    year: i32,
    month: u32,
    day: f64,
) -> f64 {
    let (y, m) = if month == 1 || month == 2 {
        ((year - 1) as f64, (month + 12) as f64)
    } else {
        (year as f64, month as f64)
    };

    let b: f64 = if is_julian_date(
        NaiveDate::from_ymd(year, month, day as u32),
    ) {
        0.0
    } else {
        let a = (y / 100.0).floor();
        2.0 - a + (a / 4.0).floor()
    };

    let c: f64 = if y < 0.0 {
        ((NUM_OF_DAYS_IN_A_YEAR * y) - 0.75).floor()
    } else {
        (NUM_OF_DAYS_IN_A_YEAR * y).floor()
    };

    let d: f64 = (30.6001 * (m + 1.0)).floor();

    b + c + d + day + 1_720_994.5
}

/// Converts Julian Day into `NaiveDateTime`. Duffett-
/// Smith suggests a float value (ex. 17.5) for `day`
/// for the returned result, but we want the excess
/// (which is 0.5) being separate. That is why, instead
/// of returning `NaiveDate`, returning `NaiveDateTime`,
///
/// References:
/// - (Peter Duffett-Smith, p.8)
/// - sowngwala::time::date_from_julian_day
///
/// Example:
/// ```rust
/// use chrono::{Datelike, Timelike};
/// use chrono::naive::NaiveDateTime;
/// use sowngwala::time::naive_from_julian_day;
///
/// let naive: NaiveDateTime =
///     naive_from_julian_day(2_446_113.75);
///
/// assert_eq!(naive.year(), 1985);
/// assert_eq!(naive.month(), 2);
/// assert_eq!(naive.day(), 17);
/// assert_eq!(naive.hour(), 6);
/// ```
#[allow(clippy::many_single_char_names)]
pub fn naive_from_julian_day(
    mut jd: f64,
) -> NaiveDateTime {
    jd += 0.5;

    let i = jd.floor();
    let f = jd.abs().fract();
    let b = if i > 2_299_160.0 {
        let a =
            ((i - 1_867_216.25) / 36_524.25).floor();
        i + 1.0 + a - (a / 4.0).floor()
    } else {
        i
    };
    let c = b + 1524.0;
    let d =
        ((c - 122.1) / NUM_OF_DAYS_IN_A_YEAR).floor();
    let e = (d * NUM_OF_DAYS_IN_A_YEAR).floor();
    let g = ((c - e) / 30.6001).floor();

    let decimal_days =
        c - e + f - (g * 30.6001).floor();

    // This is where it differs from Duffett-Smith.
    let (day, naive_time) =
        naive_time_from_decimal_days(decimal_days);

    let month =
        if g < 13.5 { g - 1.0 } else { g - 13.0 };
    let year = if month > 2.5 {
        d - 4716.0
    } else {
        d - 4715.0
    };
    let naive_date = NaiveDate::from_ymd(
        year as i32,
        month as u32,
        day,
    );

    NaiveDateTime::new(naive_date, naive_time)
}

pub fn j2000_from_julian_day(jd: f64) -> f64 {
    jd - J2000
}

pub fn j2000_from_generic_datetime<T>(dt: T) -> f64
where
    T: Datelike,
    T: Timelike,
    T: std::marker::Copy,
    T: std::fmt::Debug,
    T: std::fmt::Display,
{
    j2000_from_julian_day(
        julian_day_from_generic_datetime(dt),
    )
}

pub fn modified_julian_day_from_julian_day(
    jd: f64,
) -> f64 {
    jd - 2_400_000.5
}

pub fn modified_julian_day_from_generic_datetime<T>(
    dt: T,
) -> f64
where
    T: Datelike,
    T: Timelike,
    T: std::marker::Copy,
    T: std::fmt::Debug,
    T: std::fmt::Display,
{
    modified_julian_day_from_julian_day(
        julian_day_from_generic_datetime(dt),
    )
}

/// Finds day of the week out of a generic datetime.
///
/// References:
/// - (Peter Duffett-Smith, p.9)
/// - sowngwala::time::day_of_the_week
///
/// Example:
/// ```rust
/// use chrono::naive::{NaiveDate, NaiveDateTime};
/// use sowngwala::time::day_of_the_week;
///
/// let naive: NaiveDate =
///     NaiveDate::from_ymd(1985, 2, 17);
///
/// assert_eq!(day_of_the_week(naive), 1);
/// ```
pub fn day_of_the_week<T>(dt: T) -> u32
where
    T: Datelike,
    T: std::marker::Copy,
    T: std::fmt::Display,
{
    // let jd = julian_day(dt);
    // let a = (jd + 1.5) / 7.0;
    // (a.abs().fract() * 7.0).round() as i32

    dt.weekday().number_from_sunday()
}

/// References:
/// - sowngwala::time::add_date
///
/// Example:
/// ```rust
/// use chrono::{DateTime, Datelike, Timelike};
/// use chrono::naive::{NaiveDate, NaiveDateTime};
/// use chrono::offset::{FixedOffset, Utc};
/// use sowngwala::time::{
///     build_fixed,
///     build_utc,
///     add_date
/// };
///
/// let days: i64 = 1;
/// let zone: i32 = 4;
///
/// let naive: NaiveDateTime =
///     NaiveDate::from_ymd(2021, 1, 1)
///         .and_hms(22, 37, 0);
/// let naive: NaiveDateTime = add_date(naive, days);
///
/// assert_eq!(naive.day(), 2);
/// assert_eq!(naive.hour(), 22);
/// assert_eq!(naive.minute(), 37);
/// assert_eq!(naive.second(), 0);
///
/// let fixed: DateTime<FixedOffset> =
///     build_fixed(2021, 1, 1, 22, 37, 0, 0, zone);
/// let fixed: DateTime<FixedOffset> =
///     add_date(fixed, days);
///
/// assert_eq!(fixed.day(), 2);
/// assert_eq!(fixed.hour(), 22);
/// assert_eq!(fixed.minute(), 37);
/// assert_eq!(fixed.second(), 0);
///
/// let utc: DateTime<Utc> =
///     build_utc(2021, 1, 1, 22, 37, 0, 0);
/// let utc: DateTime<Utc> = add_date(utc, 1);
///
/// assert_eq!(utc.day(), 2);
/// assert_eq!(utc.hour(), 22);
/// assert_eq!(utc.minute(), 37);
/// assert_eq!(utc.second(), 0);
/// ```
pub fn add_date<T>(dt: T, days: i64) -> T
where
    T: Add<Duration> + Add<Duration, Output = T>,
{
    dt + Duration::days(days)
}

/// Example
/// ```rust
/// use sowngwala::time::calibrate_hmsn;
///
/// let ((hour, min, sec), day_excess) = calibrate_hmsn(0, 0, 63.0);
/// assert_eq!(sec, 3.0);
/// assert_eq!(min, 1);
///
/// let ((hour, min, sec), day_excess) = calibrate_hmsn(0, 63, 0.0);
/// assert_eq!(min, 3);
/// assert_eq!(hour, 1);
///
/// let ((hour, min, sec), day_excess) = calibrate_hmsn(24, 0, 0.0);
/// assert_eq!(hour, 0);
/// assert_eq!(day_excess, 1.0);
///
/// let ((hour, min, sec), day_excess) = calibrate_hmsn(23, 59, 60.0);
/// assert_eq!(sec, 0.0);
/// assert_eq!(min, 0);
/// assert_eq!(hour, 0);
/// assert_eq!(day_excess, 1.0);
///
/// let ((hour, min, sec), day_excess) = calibrate_hmsn(0, 1, -1.0);
/// assert_eq!(sec, 59.0);
/// assert_eq!(min, 0);
///
/// let ((hour, min, sec), day_excess) = calibrate_hmsn(0, 0, -1.0);
/// assert_eq!(sec, 59.0);
/// assert_eq!(min, 59);
/// assert_eq!(hour, 23);
/// assert_eq!(day_excess, -1.0);
/// ```
pub fn calibrate_hmsn(
    hour: i32,
    min: i32,
    sec: f64,
) -> ((i32, i32, f64), f64) {
    let mut hour = hour as f64;
    let mut min = min as f64;
    let mut sec = sec;

    // Carry over the exceeded
    // values to the next place.
    // Say, we had 60 seconds.
    // It is too much for 'sec'
    // and we want to carry over
    // to 'min' by increasing
    // 'min' by 1. For 'sec'
    // will now become 0 second.
    //
    // Say, we had 23°59'60"
    // and 60 is too much for
    // 'sec'. So, we would
    // return 1 for 'day_excess'
    // and will make a new
    // angle being 0°0'0".

    let (sec_2, min_excess): (f64, f64) =
        overflow(sec, 60.0);

    sec = sec_2;
    min += min_excess;

    let (min_2, hour_excess): (f64, f64) =
        overflow(min, 60.0);

    min = min_2;
    hour += hour_excess;

    let (hour_2, day_excess_0): (f64, f64) =
        overflow(hour, 24.0);

    hour = hour_2;

    let mut day_excess: f64 = day_excess_0;

    // Say, we had -1.0 for
    // 'sec' which is invalid
    // for 'sec'. So, we want
    // to decrease 'min' by 1,
    // and will now have 59
    // for 'sec'.
    //
    // Say, we had 0°0'-1" for
    // an angle. Again, -1 is
    // invalid for 'sec'.
    // For this, we would return
    // -1 for 'day_access' and
    // the new angle will now
    // become 23°59'59".

    if sec < 0.0 {
        sec += 60.0;
        min -= 1.0;
    }

    if min < 0.0 {
        min += 60.0;
        hour -= 1.0;
    }

    if hour < 0.0 {
        hour += 24.0;
        day_excess -= 1.0;
    }

    ((hour as i32, min as i32, sec), day_excess)
}

/// Converts `NaiveDateTime` into
/// `DateTime<FixedOffset>`. Resulted `hour` should be
/// the same regardless of `zone` given. In another
/// word, it just attaches `zone` to the given.
///
/// Example:
/// ```rust
/// use chrono::{DateTime, Timelike};
/// use chrono::naive::{NaiveDate, NaiveDateTime};
/// use chrono::offset::FixedOffset;
/// use sowngwala::time::fixed_from_naive;
///
/// let zone: i32 = 4;
/// let naive: NaiveDateTime =
///     NaiveDate::from_ymd(2021, 1, 1)
///         .and_hms(22, 37, 0);
/// let fixed: DateTime<FixedOffset> =
///     fixed_from_naive(naive, zone);
///
/// assert_eq!(fixed.hour(), 22);
/// assert_eq!(fixed.minute(), 37);
/// assert_eq!(fixed.second(), 0);
/// ```
pub fn fixed_from_naive(
    naive: NaiveDateTime,
    zone: i32,
) -> DateTime<FixedOffset> {
    FixedOffset::east(zone * 3600)
        .from_local_datetime(&naive)
        .unwrap()
}

/// Converts `DateTime<Utc>` into `DateTime<FixedOffset>`.
/// Resulted `hour` differs depending on `zone` given.
/// Meaning, for `Utc` given, there will be
/// a calculation for `FixedOffset` returned.
///
/// Reference:
/// - (Peter Duffett-Smith, p.14)
/// - sowngwala::time::local_from_ut
///
/// Example:
/// ```rust
/// use chrono::{DateTime, Timelike};
/// use chrono::naive::NaiveDateTime;
/// use chrono::offset::{Utc, FixedOffset};
/// use sowngwala::time::{
///     build_utc,
///     fixed_from_utc,
/// };
///
/// let zone: i32 = 4;
/// let utc: DateTime<Utc> =
///     build_utc(2021, 1, 1, 22, 37, 0, 0);
/// let fixed: DateTime<FixedOffset> =
///     fixed_from_utc(utc, zone);
///
/// assert_eq!(fixed.hour(), 2);
/// assert_eq!(fixed.minute(), 37);
/// assert_eq!(fixed.second(), 0);
/// ```
pub fn fixed_from_utc(
    utc: DateTime<Utc>,
    zone: i32,
) -> DateTime<FixedOffset> {
    utc.with_timezone(&FixedOffset::east(zone * 3600))
}

/// Converts `NaiveDateTime` into `DateTime<Utc>`.
/// Resulted `hour` should be the same regardless of
/// `zone` given. In another word, it just attaches
/// `zone` to the given.
///
/// Example
/// ```rust
/// use chrono::{DateTime, Timelike};
/// use chrono::offset::Utc;
/// use chrono::naive::{NaiveDate, NaiveDateTime};
/// use sowngwala::time::utc_from_naive;
///
/// let naive: NaiveDateTime =
///     NaiveDate::from_ymd(2021, 1, 1)
///         .and_hms(22, 37, 0);
/// let utc: DateTime<Utc> = utc_from_naive(naive);
///
/// assert_eq!(utc.hour(), 22);
/// assert_eq!(utc.minute(), 37);
/// assert_eq!(utc.second(), 0);
/// ```
pub fn utc_from_naive(
    naive: NaiveDateTime,
) -> DateTime<Utc> {
    DateTime::<Utc>::from_utc(naive, Utc)
}

/// Converts `DateTime<FixedOffset>`
/// into `DateTime<Utc>`. Resulted `hour` should be
/// the same regardless of `zone` given. Meaning,
/// for `FixedOffset` given, there will be
/// a calculation for `Utc` returned.
///
/// References:
/// - (Peter Duffett-Smith, pp.12-13)
/// - sowngwala::time::ut_from_local
///
/// Example
/// ```rust
/// use chrono::{DateTime, Timelike};
/// use chrono::offset::{FixedOffset, Utc};
/// use sowngwala::time::{
///     build_fixed,
///     utc_from_fixed,
/// };
///
/// let zone: i32 = 4;
/// let fixed: DateTime<FixedOffset> =
///     build_fixed(2021, 1, 1, 2, 37, 0, 0, zone);
/// let utc: DateTime<Utc> = utc_from_fixed(fixed);
///
/// assert_eq!(utc.hour(), 22);
/// assert_eq!(utc.minute(), 37);
/// assert_eq!(utc.second(), 0);
/// ```
pub fn utc_from_fixed(
    fixed: DateTime<FixedOffset>,
) -> DateTime<Utc> {
    Utc.from_utc_datetime(&fixed.naive_utc())
}

/// Converts `DateTime<FixedOffset>` into
/// `NaiveDateTime`. Resulted `hour` should be the same
/// regardless of `zone` given. In another word, it
/// just removes `zone` from the given.
///
/// Example:
/// ```rust
/// use chrono::{DateTime, Timelike};
/// use chrono::naive::NaiveDateTime;
/// use chrono::offset::FixedOffset;
/// use sowngwala::time::{
///     build_fixed,
///     naive_from_fixed,
/// };
///
/// let zone: i32 = 4;
/// let fixed: DateTime<FixedOffset> =
///     build_fixed(2021, 1, 1, 22, 37, 0, 0, zone);
/// let naive: NaiveDateTime = naive_from_fixed(fixed);
///
/// assert_eq!(naive.hour(), 22);
/// assert_eq!(naive.minute(), 37);
/// assert_eq!(naive.second(), 0);
/// ```
pub fn naive_from_fixed(
    fixed: DateTime<FixedOffset>,
) -> NaiveDateTime {
    fixed.naive_local()
}

/// Converts `DateTime<Utc>` into `NaiveDateTime`.
/// Resulted `hour` should be the same regardless of
/// `zone` given. In another word, it just removes
/// `zone` from the given.
///
/// Example:
/// ```rust
/// use chrono::{DateTime, Timelike};
/// use chrono::naive::NaiveDateTime;
/// use chrono::offset::Utc;
/// use sowngwala::time::{
///     build_utc,
///     naive_from_utc,
/// };
///
/// let zone: i32 = 4;
/// let utc: DateTime<Utc> =
///     build_utc(2021, 1, 1, 22, 37, 0, 0);
/// let naive: NaiveDateTime = naive_from_utc(utc);
///
/// assert_eq!(naive.hour(), 22);
/// assert_eq!(naive.minute(), 37);
/// assert_eq!(naive.second(), 0);
/// ```
pub fn naive_from_utc(
    utc: DateTime<Utc>,
) -> NaiveDateTime {
    utc.naive_utc()
}

pub fn eot_decimal_from_utc(
    utc: DateTime<Utc>,
) -> (f64, f64) {
    let (eot, day_excess) =
        equation_of_time_from_utc(utc);
    let decimal = decimal_hours_from_angle(eot);
    (decimal, day_excess)
}

/// Example:
/// ```rust
/// use chrono::{
///   DateTime,
///   Datelike,
///   Timelike,
/// };
/// use chrono::naive::NaiveTime;
/// use chrono::offset::{FixedOffset, Utc};
/// use sowngwala::time::{
///     build_fixed,
///     eot_fortified_utc_from_fixed
/// };
///
/// let zone: i32 = 9;
/// let fixed: DateTime<FixedOffset> =
///     build_fixed(2021, 1, 1, 9, 0, 0, 0, zone);
/// let utc: DateTime<Utc> =
///     eot_fortified_utc_from_fixed(fixed);
///
/// assert_eq!(utc.year(), 2020);
/// assert_eq!(utc.month(), 12);
/// assert_eq!(utc.day(), 31);
/// assert_eq!(utc.hour(), 23);
/// assert_eq!(utc.minute(), 59);
/// assert_eq!(utc.second(), 34); // 34.227691152289594
/// assert_eq!(utc.nanosecond(), 227_691_152);
/// ```
#[allow(clippy::many_single_char_names)]
pub fn eot_fortified_utc_from_fixed(
    fixed: DateTime<FixedOffset>,
) -> DateTime<Utc> {
    let utc_0: DateTime<Utc> = utc_from_fixed(fixed);

    let utc_decimal: f64 =
        decimal_hours_from_generic_time(
            naive_from_utc(utc_0),
        );
    let (eot_decimal, day_excess): (f64, f64) =
        eot_decimal_from_utc(utc_0);

    let mut angle: Angle = angle_from_decimal_hours(
        utc_decimal + eot_decimal,
    );
    angle.calibrate();
    let t: NaiveTime = angle.to_naive_time();

    let utc_1: DateTime<Utc> =
        add_date(utc_0, day_excess as i64);

    build_utc(
        utc_1.year(),
        utc_1.month(),
        utc_1.day(),
        t.hour(),
        t.minute(),
        t.second(),
        t.nanosecond(),
    )
}

/// Given UT, and retursn GST.
///
/// References:
/// - (Peter Duffett-Smith, p.17)
/// - sowngwala::time::gst_from_ut
///
/// Example:
/// ```rust
/// use chrono::{DateTime, Timelike};
/// use chrono::naive::NaiveTime;
/// use chrono::offset::Utc;
/// use sowngwala::time::{
///     build_utc,
///     gst_from_utc,
/// };
///
/// let nanosecond: u32 = 670_000_000;
/// let utc: DateTime<Utc> =
///     build_utc(1980, 4, 22, 14, 36, 51, nanosecond);
/// let gst: NaiveTime = gst_from_utc(utc);
///
/// assert_eq!(gst.hour(), 4);
/// assert_eq!(gst.minute(), 40);
/// assert_eq!(gst.second(), 5); // 5.229576759185761
/// assert_eq!(gst.nanosecond(), 229_576_759);
/// ```
pub fn gst_from_utc(utc: DateTime<Utc>) -> NaiveTime {
    let jd = julian_day_from_generic_date(utc);

    let s = jd - 2_451_545.0;
    let t = s / 36_525.0;
    let t0 = 6.697_374_558
        + (2_400.051_336 * t)
        + (0.000_025_862 * t * t);

    let (t0, _factor) = overflow(t0, 24.0);

    let naive_time =
        naive_time_from_generic_datetime(utc);

    let mut decimal =
        decimal_hours_from_generic_time(naive_time);
    decimal *= 1.002_737_909;
    decimal += t0;

    let (decimal, _factor): (f64, f64) =
        overflow(decimal, 24.0);

    naive_time_from_decimal_hours(decimal)
}

/// Given GST, returns UTC.
///
/// Reference:
/// - (Peter Duffett-Smith, pp.18-19)
/// - sowngwala::time::utc_from_gst
///
/// Example:
/// ```rust
/// use chrono::Timelike;
/// use chrono::naive::{NaiveDateTime, NaiveDate, NaiveTime};
/// use sowngwala::time::utc_from_gst;
///
/// let nanosecond: u32 = 230_000_000;
/// let gst: NaiveDateTime =
///     NaiveDate::from_ymd(1980, 4, 22)
///         .and_hms_nano(4, 40, 5, nanosecond);
///
/// let utc = utc_from_gst(gst);
/// assert_eq!(utc.hour(), 14);
/// assert_eq!(utc.minute(), 36);
/// assert_eq!(utc.second(), 51); // 51.67040214530175
/// assert_eq!(
///     utc.nanosecond(),
///     670_402_145
/// );
/// ```
#[allow(clippy::many_single_char_names)]
pub fn utc_from_gst<T>(gst: T) -> NaiveTime
where
    T: Datelike,
    T: Timelike,
    T: std::marker::Copy,
    T: std::fmt::Debug,
    T: std::fmt::Display,
{
    // Luckily, we only need date, not datetime.
    let jd = julian_day_from_generic_date(gst);

    let s = jd - 2_451_545.0;
    let t = s / 36_525.0;
    let t0 = 6.697_374_558
        + (2_400.051_336 * t)
        + (0.000_025_862 * t * t);
    let (t0, _factor): (f64, f64) =
        overflow(t0, 24.0);

    let decimal = decimal_hours_from_generic_time(
        NaiveTime::from_hms_nano(
            gst.hour(),
            gst.minute(),
            gst.second(),
            gst.nanosecond(),
        ),
    );

    let (decimal, _factor2): (f64, f64) =
        overflow(decimal - t0, 24.0);

    naive_time_from_decimal_hours(
        decimal * 0.997_269_566_3,
    )
}

/// Given GST and longitude, returns LST.
///
/// Reference:
/// - (Peter Duffett-Smith, p.20)
/// - sowngwala::time::lst_from_gst
///
/// Example:
/// ```rust
/// use chrono::Timelike;
/// use chrono::naive::{NaiveDateTime, NaiveDate};
/// use sowngwala::coords::Direction;
/// use sowngwala::time::lst_from_gst;
///
/// let dir = Direction::West;
/// let lng = 64.0;
/// let nanosecond: u32 = 230_000_000;
/// let gst: NaiveDateTime =
///     NaiveDate::from_ymd(1980, 4, 22)
///         .and_hms_nano(4, 40, 5, nanosecond);
///
/// let lst = lst_from_gst(gst, lng, dir);
///
/// assert_eq!(lst.hour(), 0);
/// assert_eq!(lst.minute(), 24);
/// assert_eq!(lst.second(), 5); // 5.230000000001169
/// assert_eq!(lst.nanosecond(), 230_000_000);
/// ```
pub fn lst_from_gst<T>(
    gst: T,
    lng: f64,
    dir: Direction,
) -> NaiveTime
where
    T: Datelike,
    T: Timelike,
    T: std::marker::Copy,
    T: std::fmt::Debug,
    T: std::fmt::Display,
{
    let decimal = decimal_hours_from_generic_time(
        naive_time_from_generic_datetime(gst),
    );
    let diff = lng / 15.0;

    let mut lst = match dir {
        Direction::West => decimal - diff,
        Direction::East => decimal + diff,
        _ => decimal,
    };

    if lst > 24.0 {
        lst -= 24.0;
    };

    if lst < 0.0 {
        lst += 24.0;
    };

    naive_time_from_decimal_hours(lst)
}

/// Given LST and longitude, returns GST.
///
/// Reference:
/// - (Peter Duffett-Smith, p.21)
/// - sowngwala::time::gst_lst_from
///
/// Example:
/// ```rust
/// use chrono::Timelike;
/// use chrono::naive::{NaiveDateTime, NaiveDate};
/// use sowngwala::coords::Direction;
/// use sowngwala::time::gst_from_lst;
///
/// let dir = Direction::West;
/// let lng = 64.0;
/// let nanosecond: u32 = 230_000_000;
/// let lst: NaiveDateTime =
///     NaiveDate::from_ymd(1980, 4, 22)
///         .and_hms_nano(0, 24, 5, nanosecond);
///
/// let gst = gst_from_lst(lst, lng, dir);
///
/// assert_eq!(gst.hour(), 4);
/// assert_eq!(gst.minute(), 40);
/// assert_eq!(gst.second(), 5); // 5.230000000000956
/// assert_eq!(gst.nanosecond(), 230_000_000);
/// ```
pub fn gst_from_lst<T>(
    lst: T,
    lng: f64,
    dir: Direction,
) -> NaiveTime
where
    T: Datelike,
    T: Timelike,
    T: std::marker::Copy,
    T: std::fmt::Debug,
    T: std::fmt::Display,
{
    let decimal = decimal_hours_from_generic_time(
        naive_time_from_generic_datetime(lst),
    );
    let diff = lng / 15.0;

    let mut gst = match dir {
        Direction::West => decimal + diff,
        Direction::East => decimal - diff,
        _ => decimal,
    };

    if gst > 24.0 {
        gst -= 24.0;
    };

    if gst < 0.0 {
        gst += 24.0;
    };

    naive_time_from_decimal_hours(gst)
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx_eq::assert_approx_eq;
    use chrono::naive::{NaiveDate, NaiveDateTime};
    // use crate::time::julian_day_from_generic_datetime;

    #[test]
    fn julian_day_for_marty_mcfly_goes_back() {
        // On Saturday, October 26, 1985, 1:35 AM,
        // Marty McFly goes back in time,
        // and arrives at future Peabody Farm
        // on Saturday, November 5, 1955, 6:15 am.
        let datetime: NaiveDateTime =
            NaiveDate::from_ymd(1985, 10, 26)
                .and_hms(1, 35, 0);

        let jd: f64 =
            julian_day_from_generic_datetime(
                datetime,
            );

        assert_approx_eq!(
            jd, // 2446364.565972222
            2446364.56597,
            1e-11
        );
    }
}
