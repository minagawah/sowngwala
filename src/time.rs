#[cfg(test)]
extern crate approx_eq;

use serde::{ Deserialize, Serialize };
use std::convert::{ TryFrom, From };

use crate::constants::{ NUM_OF_DAYS_IN_A_YEAR, J2000 };
use crate::coords::Direction;
use crate::sun::equation_of_time_from_ut;
use crate::utils::carry_over;

#[derive(Debug, Copy, Clone, PartialEq, Serialize, Deserialize)]
pub enum Month {
    Jan = 1,
    Feb = 2,
    Mar = 3,
    Apr = 4,
    May = 5,
    Jun = 6,
    Jul = 7,
    Aug = 8,
    Sep = 9,
    Oct = 10,
    Nov = 11,
    Dec = 12,
}

impl TryFrom<i32> for Month {
    type Error = ();

    fn try_from(n: i32) -> Result<Self, Self::Error> {
        match n {
            1 => Ok(Month::Jan),
            2 => Ok(Month::Feb),
            3 => Ok(Month::Mar),
            4 => Ok(Month::Apr),
            5 => Ok(Month::May),
            6 => Ok(Month::Jun),
            7 => Ok(Month::Jul),
            8 => Ok(Month::Aug),
            9 => Ok(Month::Sep),
            10 => Ok(Month::Oct),
            11 => Ok(Month::Nov),
            12 => Ok(Month::Dec),
            _ => Err(()),
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Serialize, Deserialize)]
pub enum Weekday {
    Sun = 1,
    Mon = 2,
    Tue = 3,
    Wed = 4,
    Thu = 5,
    Fri = 6,
    Sat = 7,
}

impl TryFrom<i32> for Weekday {
    type Error = ();

    fn try_from(n: i32) -> Result<Self, Self::Error> {
        match n {
            0 => Ok(Weekday::Sun),
            1 => Ok(Weekday::Mon),
            2 => Ok(Weekday::Tue),
            3 => Ok(Weekday::Wed),
            4 => Ok(Weekday::Thu),
            5 => Ok(Weekday::Fri),
            6 => Ok(Weekday::Sat),
            _ => Err(()),
        }
    }
}

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub struct Date {
    pub year: i16,
    pub month: Month,
    pub day: f64,
}

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub struct Time {
    pub hour: i16,
    pub min: i16,
    pub sec: f64,
}

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub struct DateTime {
    pub year: i16,
    pub month: Month,
    pub day: f64,
    pub hour: i16,
    pub min: i16,
    pub sec: f64,
}

impl From<&DateTime> for Date {
    fn from(&dt: &DateTime) -> Self {
        Date {
            year: dt.year,
            month: dt.month,
            day: dt.day,
        }
    }
}

impl From<&DateTime> for Time {
    fn from(&dt: &DateTime) -> Self {
        Time {
            hour: dt.hour,
            min: dt.min,
            sec: dt.sec,
        }
    }
}

impl DateTime {
    pub fn new(&d: &Date, &t: &Time) -> Self {
        DateTime {
            year: d.year,
            month: d.month,
            day: d.day,
            hour: t.hour,
            min: t.min,
            sec: t.sec,
        }
    }

    /// Example:
    /// ```rust
    /// use sowngwala::time::{Month, DateTime};
    ///
    /// // Marty McFly goes back in time after the doc gets shot.
    /// let dt = DateTime {
    ///     year: 1985,
    ///     month: Month::Nov,
    ///     day: 5.0,
    ///     hour: 1,
    ///     min: 35,
    ///     sec: 0.0,
    /// };
    ///
    /// assert_eq!(
    ///     dt.iso_8601(),
    ///     "1985-11-05T01:35:00"
    /// );
    /// ```
    pub fn iso_8601 (&self) -> String {
        format!(
            "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}",
            self.year as u16,
            self.month as u8,
            self.day as u8,
            self.hour as u8,
            self.min as u8,
            self.sec as u8
        )
    }
}

/// Checks whether the given Date is julian day.
pub fn is_julian_date(date: &Date) -> bool {
    if date.year > 1582 {
        return false;
    }
    if date.year < 1582 {
        return true;
    }
    if (date.month as u8) > 10 {
        return false;
    }
    if (date.month as u8) < 10 {
        return true;
    }
    if date.day > 14.0 {
        return false;
    }
    true
}

/// Check if the given year is a leap year.
pub fn is_leap_year(year: i16) -> bool {
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
/// use approx_eq::assert_approx_eq;
/// use sowngwala::time::{
///   Month,
///   Date,
///   day_number_from_date
/// };
///
/// // --------------------
/// // Example 1
/// // --------------------
/// let date = Date {
///     year: 1985,
///     month: Month::Feb,
///     day: 17.0,
/// };
/// assert_eq!(
///     day_number_from_date(&date),
///     48
/// );
///
/// // --------------------
/// // Example 2
/// // --------------------
/// let date = Date {
///     year: 1988,
///     month: Month::Jul,
///     day: 27.0,
/// };
/// assert_eq!(
///     day_number_from_date(&date),
///     209
/// );
/// ```
#[allow(clippy::many_single_char_names)]
pub fn day_number_from_date(&date: &Date) -> u16 {
    let tmp = if is_leap_year(date.year) {
        62.0
    } else {
        63.0
    };
    let mut num = (date.month as i32) as f64;
    if num <= 2.0 {
        num = ((num - 1.0) * tmp / 2.0).floor();
    } else {
        num = ((num + 1.0) * 30.6).floor() - tmp;
    }
    (num + date.day) as u16
}

/// Note:
/// Regardless of the month, the diff is of "Jan 0th".
/// Say, for "July 27th, 1988", it will be the diff between:
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
pub fn days_since_1990(year: i16) -> f64 {
    let mut year_0: i16 = year;
    let mut days: f64 = 0.0;

    if year - 1990 < 0 {
        while year_0 < 1990 {
            let leap = is_leap_year(year_0);
            days -= 365.0;
            if leap {
                days -= 1.0;
            }
            year_0 += 1;
        }
    } else {
        while year_0 > 1990 {
            let leap = is_leap_year(year_0);
            days += 365.0;
            if leap {
                days += 1.0;
            }
            year_0 -= 1;
        }
    }

    days
}

/// Converts Date into julian day.
/// (Peter Duffett-Smith, pp.6-7)
///
/// Example:
/// ```rust
/// use approx_eq::assert_approx_eq;
/// use sowngwala::time::{
///   Month,
///   Date,
///   julian_day
/// };
///
/// let date = Date {
///     year: 1985,
///     month: Month::Feb,
///     day: 17.25,
/// };
/// assert_eq!(
///     julian_day(&date),
///     2_446_113.75
/// );
/// ```
#[allow(clippy::many_single_char_names)]
pub fn julian_day(&date: &Date) -> f64 {
    let month = date.month as u8;

    let (y, m) = if month == 1 || month == 2 {
        ((date.year - 1) as f64, (month + 12) as f64)
    } else {
        (date.year as f64, month as f64)
    };

    let b = if is_julian_date(&date) {
        0.0
    } else {
        let a = (y / 100.0).floor();
        2.0 - a + (a / 4.0).floor()
    };

    let c = if y < 0.0 {
        ((NUM_OF_DAYS_IN_A_YEAR * y) - 0.75).floor()
    } else {
        (NUM_OF_DAYS_IN_A_YEAR * y).floor()
    };

    let d = (30.6001 * (m + 1.0)).floor();

    b + c + d + date.day + 1_720_994.5
}

/// Example:
/// ```rust
/// use approx_eq::assert_approx_eq;
/// use sowngwala::time::{
///   Month,
///   DateTime,
///   julian_day_from_ut
/// };
///
/// let ut = DateTime {
///     year: 2004,
///     month: Month::May,
///     day: 12.0,
///     hour: 14,
///     min: 45,
///     sec: 30.0,
/// };
/// assert_approx_eq!(
///     julian_day_from_ut(&ut), // 2453138.1149305557,
///     2_453_138.115,
///     1e-4
/// );
/// ```
#[allow(clippy::many_single_char_names)]
pub fn julian_day_from_ut(&ut: &DateTime) -> f64 {
    let jd: f64 = julian_day(&Date::from(&ut));
    let decimal_hours: f64 = decimal_hours_from_time(&Time::from(&ut));
    jd + (decimal_hours / 24.0)
}

/// Converts julian day into Date.
/// (Peter Duffett-Smith, p.8)
///
/// Example:
/// ```rust
/// use sowngwala::time::{Month, date_from_julian_day};
///
/// let date = date_from_julian_day(2_446_113.75);
/// assert_eq!(date.year, 1985);
/// assert_eq!(date.month, Month::Feb);
/// assert_eq!(date.day, 17.25);
/// ```
#[allow(clippy::many_single_char_names)]
pub fn date_from_julian_day(mut jd: f64) -> Date {
    jd += 0.5;

    let i = jd.floor();
    let f = jd.abs().fract();
    let b = if i > 2_299_160.0 {
        let a = ((i - 1_867_216.25) / 36_524.25).floor();
        i + 1.0 + a - (a / 4.0).floor()
    } else {
        i
    };
    let c = b + 1524.0;
    let d = ((c - 122.1) / NUM_OF_DAYS_IN_A_YEAR).floor();
    let e = (d * NUM_OF_DAYS_IN_A_YEAR).floor();
    let g = ((c - e) / 30.6001).floor();

    let day = c - e + f - (g * 30.6001).floor();
    let month = if g < 13.5 {
        g - 1.0
    } else {
        g - 13.0
    };

    let year = if month > 2.5 {
        d - 4716.0
    } else {
        d - 4715.0
    };

    Date {
        year: year as i16,
        month: Month::try_from(month as i32).unwrap(),
        day,
    }
}

pub fn j2000_from_julian_day(jd: f64) -> f64 {
    jd - J2000
}

pub fn j2000_from_ut(&ut: &DateTime) -> f64 {
    j2000_from_julian_day(
        julian_day_from_ut(&ut)
    )
}

pub fn modified_julian_day_from_julian_day(jd: f64) -> f64 {
    jd - 2_400_000.5
}

pub fn modified_julian_day_from_ut(&ut: &DateTime) -> f64 {
    modified_julian_day_from_julian_day(
        julian_day_from_ut(&ut)
    )
}

/// We define the decimal year "y" as follows:
///
///   y = year + (month - 0.5) / 12
///
/// This gives "y" for the middle of the month, which is accurate enough
/// given the precision in the known values of ΔT. The following
/// polynomial expressions can be used calculate the value of ΔT
/// (in seconds) over the time period covered by of the Five Millennium
/// Canon of Solar Eclipses: -1999 to +3000.
pub fn decimal_year_from_date(&date: &Date) -> f64 {
    (date.year as f64) + ((date.month as i16) as f64 - 0.5) / 12.0
}

/// Finds day of the week out of Date.
/// (Peter Duffett-Smith, p.9)
///
/// Example:
/// ```rust
/// use sowngwala::time::{
///   Month,
///   Date,
///   Weekday,
///   day_of_the_week
/// };
///
/// let date = Date {
///     year: 1985,
///     month: Month::Feb,
///     day: 17.0,
/// };
/// assert_eq!(day_of_the_week(&date), Weekday::Sun);
/// ```
#[allow(clippy::many_single_char_names)]
pub fn day_of_the_week(&date: &Date) -> Weekday {
    let jd = julian_day(&date);
    let a = (jd + 1.5) / 7.0;
    Weekday::try_from((a.abs().fract() * 7.0).round() as i32).unwrap()
}

/// Converts Time into decimal hours.
/// (Peter Duffett-Smith, p.10)
///
/// Example:
/// ```rust
/// use approx_eq::assert_approx_eq;
/// use sowngwala::time::{Time, decimal_hours_from_time};
///
/// let time = Time {
///     hour: 18,
///     min: 31,
///     sec: 27.0,
/// };
/// assert_approx_eq!(
///     decimal_hours_from_time(&time), // 18.524166666666666
///     18.52417,
///     1e-6
/// );
/// ```
pub fn decimal_hours_from_time(&time: &Time) -> f64 {
    let hour = (time.hour as f64).abs();
    let min = (time.min as f64).abs();
    let sec = time.sec.abs();
    let dhours = hour + ((min + (sec / 60.0)) / 60.0);
    if time.hour < 0 || time.min < 0 || time.sec < 0.0 {
        - dhours
    } else {
        dhours
    }
}

/// Convert decimal hours into Time.
/// (Peter Duffett-Smith, p.11)
///
/// Example:
/// ```rust
/// use approx_eq::assert_approx_eq;
/// use sowngwala::time::time_from_decimal_hours;
///
/// let time = time_from_decimal_hours(18.52417);
/// assert_eq!(time.hour, 18);
/// assert_eq!(time.min, 31);
/// assert_approx_eq!(
///     time.sec, // 27.012000000005685
///     27.0,
///     1e-3
/// );
/// ```
#[allow(clippy::many_single_char_names)]
pub fn time_from_decimal_hours(dec: f64) -> Time {
    let sign: i16 = if dec < 0.0 {
        -1
    } else {
        1
    };
    let dec2 = dec.abs();
    let lower = dec2.fract() * 60.0;

    let mut hour = dec2.floor() as i16;
    let mut min = lower.floor() as i16;
    let mut sec: f64 = lower.abs().fract() * 60.0;

    if hour != 0 {
        hour *= sign;
    } else if min != 0 {
        min *= sign;
    } else {
        sec *= sign as f64;
    }

    Time { hour, min, sec }
}

pub fn add_date(&date: &Date, adjust: f64) -> Date {
    date_from_julian_day(
        julian_day(&date) + adjust
    )
}

fn _ut_aux(&dt: &DateTime, zone: i8) -> DateTime {
    let t_0 = Time {
        hour: dt.hour,
        min: dt.min,
        sec: dt.sec,
    };

    let mut decimal: f64 = decimal_hours_from_time(&t_0);
    decimal += zone as f64;

    let mut day_adjust: f64 = 0.0;

    if decimal > 24.0 {
        decimal -= 24.0;
        day_adjust += 1.0;
    }

    if decimal < 0.0 {
        decimal += 24.0;
        day_adjust -= 1.0;
    }

    let t = time_from_decimal_hours(decimal);
    let d = add_date(&Date::from(&dt), day_adjust);

    DateTime::new(&d, &t)
}

/// Given local time and time zone, returns UT.
/// (Peter Duffett-Smith, pp.12-13)
///
/// Example:
/// ```rust
/// use approx_eq::assert_approx_eq;
/// use sowngwala::time::{
///   DateTime,
///   Month,
///   ut_from_local
/// };
///
/// let daylight: i16 = 1;
/// let dt = DateTime {
///     year: 2021,
///     month: Month::Jan,
///     day: 1.0,
///     hour: 3 - daylight,
///     min: 37,
///     sec: 0.0,
/// };
/// let ut: DateTime = ut_from_local(&dt, 4);
/// assert_eq!(ut.hour, 22);
/// assert_eq!(ut.min, 37);
/// assert_approx_eq!(
///     ut.sec, // 0.0000000000017053025658242404
///     0.0,
///     0.1
/// );
/// ```
pub fn ut_from_local(&dt: &DateTime, zone: i8) -> DateTime {
    _ut_aux(&dt, -zone)
}

/// Given UT and time zone, returns local time.
/// (Peter Duffett-Smith, p.14)
///
/// Example:
/// ```rust
/// use approx_eq::assert_approx_eq;
/// use sowngwala::time::{
///   DateTime,
///   Month,
///   local_from_ut
/// };
///
/// let daylight: i16 = 1;
/// let ut = DateTime {
///     year: 2021,
///     month: Month::Jan,
///     day: 1.0,
///     hour: 22,
///     min: 37,
///     sec: 0.0,
/// };
///
/// let dt: DateTime = local_from_ut(&ut, 4);
/// assert_eq!(dt.hour + daylight, 3);
/// assert_eq!(dt.min, 37);
/// assert_approx_eq!(
///     dt.sec, // 0.0000000000017053025658242404
///     0.0,
///     0.1
/// );
/// ```
pub fn local_from_ut(&dt: &DateTime, zone: i8) -> DateTime {
    _ut_aux(&dt, zone)
}

pub fn normalize_time(&t: &Time) -> (Time, f64) {
    let (sec, min_excess): (f64, f64) = carry_over(t.sec, 60.0);

    let min: f64 = (t.min as f64) + min_excess;
    let (min, hour_excess): (f64, f64) = carry_over(min, 60.0);

    let hour: f64 = (t.hour as f64) + hour_excess;
    let (hour, day_excess): (f64, f64) = carry_over(hour, 24.0);

    let time = Time {
        hour: hour as i16,
        min: min as i16,
        sec,
    };

    (time, day_excess)
}

/// Example:
/// ```rust
/// use sowngwala::time::{
///   Month,
///   DateTime,
///   normalize_datetime
/// };
///
/// let dt_0 = DateTime {
///     year: 2021,
///     month: Month::Jan,
///     day: 31.0,
///     hour: 23,
///     min: 61,
///     sec: -2.0,
/// };
/// let dt: DateTime = normalize_datetime(&dt_0);
///
/// assert_eq!(dt.year, 2021);
/// assert_eq!(dt.month, Month::Feb);
/// assert_eq!(dt.day, 1.0);
/// assert_eq!(dt.hour, 0);
/// assert_eq!(dt.min, 0);
/// assert_eq!(dt.sec, 58.0);
/// ```
#[allow(clippy::many_single_char_names)]
pub fn normalize_datetime(&dt: &DateTime) -> DateTime {
    let (t, day_excess): (Time, f64) = normalize_time(&Time::from(&dt));
    let d = add_date(&Date::from(&dt), day_excess);
    DateTime::new(&d, &t)
}

pub fn eot_decimal_from_ut(&ut: &DateTime) -> f64 {
    decimal_hours_from_time(
        &equation_of_time_from_ut(&ut)
    )
}

/// Example:
/// ```rust
/// use approx_eq::assert_approx_eq;
/// use sowngwala::time::{
///   Month,
///   DateTime,
///   eot_fortified_ut_from_local
/// };
///
/// let zone: i8 = 9;
/// let dt = DateTime {
///     year: 2021,
///     month: Month::Jan,
///     day: 1.0,
///     hour: 9,
///     min: 0,
///     sec: 0.0,
/// };
///
/// let ut: DateTime = eot_fortified_ut_from_local(&dt, zone);
///
/// assert_eq!(ut.year, 2020);
/// assert_eq!(ut.month, Month::Dec);
/// assert_eq!(ut.day, 31.0);
/// assert_eq!(ut.hour, 23);
/// assert_eq!(ut.min, 59);
/// assert_approx_eq!(
///     ut.sec, // 34.227691152289594
///     34.22769,
///     1e-6
/// );
///```
#[allow(clippy::many_single_char_names)]
pub fn eot_fortified_ut_from_local(&dt: &DateTime, zone: i8) -> DateTime {
    let ut: DateTime = ut_from_local(&dt, zone);
    let ut_decimal: f64 = decimal_hours_from_time(&Time::from(&ut));
    let eot_decimal: f64 = eot_decimal_from_ut(&ut);
    let d: Date = Date::from(&ut);
    let t: Time = time_from_decimal_hours(ut_decimal + eot_decimal);
    normalize_datetime(&DateTime::new(&d, &t))
}

/// Given UT, and retursn GST.
/// (Peter Duffett-Smith, p.17)
///
/// Example:
/// ```rust
/// use approx_eq::assert_approx_eq;
/// use sowngwala::time::{
///   Month,
///   DateTime,
///   gst_from_ut
/// };
///
/// let ut = DateTime {
///     year: 1980,
///     month: Month::Apr,
///     day: 22.0,
///     hour: 14,
///     min: 36,
///     sec: 51.67,
/// };
/// let gst = gst_from_ut(&ut);
/// assert_eq!(gst.hour, 4);
/// assert_eq!(gst.min, 40);
/// assert_approx_eq!(
///     gst.sec, // 5.229576759185761
///     5.23,
///     1e-3
/// );
/// ```
#[allow(clippy::many_single_char_names)]
pub fn gst_from_ut(&ut: &DateTime) -> Time {
    let jd = julian_day(&Date::from(&ut));
    let s = jd - 2_451_545.0;
    let t = s / 36_525.0;
    let t0 = 6.697_374_558 + (2_400.051_336 * t) + (0.000_025_862 * t * t);
    let (t0, _factor) = carry_over(t0, 24.0);

    let mut decimal = decimal_hours_from_time(&Time::from(&ut));
    decimal *= 1.002_737_909;
    decimal += t0;

    let (decimal, _factor): (f64, f64) = carry_over(decimal, 24.0);

    time_from_decimal_hours(decimal)
}

/// Given GST, returns UT.
/// (Peter Duffett-Smith, pp.18-19)
///
/// Example:
/// ```rust
/// use approx_eq::assert_approx_eq;
/// use sowngwala::time::{
///   Month,
///   DateTime,
///   ut_from_gst
/// };
///
/// let gst = DateTime {
///     year: 1980,
///     month: Month::Apr,
///     day: 22.0,
///     hour: 4,
///     min: 40,
///     sec: 5.23,
/// };
/// let ut = ut_from_gst(&gst);
/// assert_eq!(ut.hour, 14);
/// assert_eq!(ut.min, 36);
/// assert_approx_eq!(
///     ut.sec, // 51.67040214530175
///     51.67,
///     1e-4
/// );
/// ```
#[allow(clippy::many_single_char_names)]
pub fn ut_from_gst(&gst: &DateTime) -> Time {
    let jd = julian_day(&Date::from(&gst));
    let s = jd - 2_451_545.0;
    let t = s / 36_525.0;
    let t0 = 6.697_374_558 + (2_400.051_336 * t) + (0.000_025_862 * t * t);
    let (t0, _factor): (f64, f64) = carry_over(t0, 24.0);

    let decimal = decimal_hours_from_time(&Time::from(&gst));
    let (decimal, _factor2): (f64, f64) = carry_over(decimal - t0, 24.0);

    time_from_decimal_hours(decimal * 0.997_269_566_3)
}

/// Given GST and longitude, returns LST.
/// (Peter Duffett-Smith, p.20)
///
/// Example:
/// ```rust
/// use approx_eq::assert_approx_eq;
/// use sowngwala::coords::Direction;
/// use sowngwala::time::{
///   Month,
///   DateTime,
///   lst_from_gst
/// };
///
/// let dir = Direction::West;
/// let lng = 64.0;
///
/// let gst = DateTime {
///     year: 1980,
///     month: Month::Apr,
///     day: 22.0,
///     hour: 4,
///     min: 40,
///     sec: 5.23,
/// };
///
/// let lst = lst_from_gst(&gst, lng, dir);
///
/// assert_eq!(lst.hour, 0);
/// assert_eq!(lst.min, 24);
/// assert_approx_eq!(
///     lst.sec, // 5.230000000001169
///     5.23,
///     1e-3
/// );
/// ```
pub fn lst_from_gst(&gst: &DateTime, lng: f64, dir: Direction) -> Time {
    let decimal = decimal_hours_from_time(&Time::from(&gst));
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

    time_from_decimal_hours(lst)
}

/// Given LST and longitude, returns GST.
/// (Peter Duffett-Smith, p.21)
///
/// Example:
/// ```rust
/// use approx_eq::assert_approx_eq;
/// use sowngwala::coords::Direction;
/// use sowngwala::time::{
///   Month,
///   DateTime,
///   gst_from_lst
/// };
///
/// let dir = Direction::West;
/// let lng = 64.0;
///
/// let lst = DateTime {
///     year: 1980,
///     month: Month::Apr,
///     day: 22.0,
///     hour: 0,
///     min: 24,
///     sec: 5.23,
/// };
///
/// let gst = gst_from_lst(&lst, lng, dir);
///
/// assert_eq!(gst.hour, 4);
/// assert_eq!(gst.min, 40);
/// assert_approx_eq!(
///     gst.sec, // 5.230000000000956
///     5.23,
///     1e-3
/// );
/// ```
pub fn gst_from_lst(&lst: &DateTime, lng: f64, dir: Direction) -> Time {
    let decimal = decimal_hours_from_time(&Time::from(&lst));
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

    time_from_decimal_hours(gst)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn is_julian_date_returns_true() {
        let date = Date {
            year: 1582,
            month: Month::Oct,
            day: 10.0,
        };
        assert_eq!(
            is_julian_date(&date),
            true
        );
    }

    #[test]
    fn is_julian_date_returns_false() {
        let date = Date {
            year: 1582,
            month: Month::Oct,
            day: 9.0,
        };
        assert_eq!(
            is_julian_date(&date),
            true
        );
    }
}
