//     DateTimeStamp(),
//     GYearMonth(),
//     GYear(),
//     GMonthDay(),
//     GDay(),
//     GMonth(),

use crate::values::date_time_utils::ChronoTime;
use crate::values::Value;

#[derive(Debug, Clone)]
pub struct DateTime(pub(crate) chrono::DateTime<chrono::FixedOffset>);

impl DateTime {
    pub(crate) fn from_ymd_hms_tz(yy: u32, mm: u32, dd: u32, h: u32, m: u32, s: u32, tz_h: i32, tz_m: i32) -> Option<Self> {
        if let Some(date) = chrono::NaiveDate::from_ymd_opt(yy as i32, mm, dd) {
            if let Some(time) = chrono::NaiveTime::from_hms_opt(h, m, s) {
                let offset = if tz_h > 0 {
                    chrono::FixedOffset::east(((tz_h * 60) + tz_m) * 60)
                } else {
                    chrono::FixedOffset::west(((-tz_h * 60) + tz_m) * 60)
                };
                let dt = chrono::NaiveDateTime::new(date, time - offset);
                Some(DateTime(chrono::DateTime::from_utc(dt, offset)))
            } else {
                None
            }
        } else {
            None
        }
    }
}

impl Value for DateTime {

}

#[derive(Debug, Clone)]
pub struct Date(pub(crate) chrono::Date<chrono::FixedOffset>);

impl Date {
    pub(crate) fn now_boxed() -> Box<dyn Value> {
        let now = chrono::Local::now();
        let date = chrono::Date::from_utc(
            now.date().naive_utc(),
            chrono::TimeZone::from_offset(now.offset())
        );

        Box::new(Date(date))
    }

    pub(crate) fn from_ymd_tz(y: u32, m: u32, d: u32, tz_h: i32, tz_m: i32) -> Option<Self> {
        if let Some(date) = chrono::NaiveDate::from_ymd_opt(y as i32, m, d) {
            let offset = if tz_h > 0 {
                chrono::FixedOffset::east(((tz_h * 60) + tz_m) * 60)
            } else {
                chrono::FixedOffset::west(((tz_h * 60) + tz_m) * 60)
            };
            Some(Date(chrono::Date::from_utc(date, offset)))
        } else {
            // Err(nom::Err::Failure(Error::from_error_kind(input, ErrorKind::MapRes)))
            None
        }
    }
}

impl Value for Date {

}

#[derive(Debug, Clone)]
pub struct Time(pub(crate) ChronoTime<chrono::FixedOffset>);

impl Time {
    pub(crate) fn now_boxed() -> Box<dyn Value> {
        Box::new(Time(ChronoTime::now()))
    }

    pub(crate)  fn from_hms_ms(h: u32, m: u32, s: u32, ms: u32) -> Option<Self> {
        if let Some(time) = chrono::NaiveTime::from_hms_milli_opt(h, m, s, ms) {
            Some(Time(ChronoTime::from_utc(time)))
        } else {
            None
        }
    }
}

impl Value for Time {
    
}

#[derive(Debug, Clone)]
pub struct Duration {
    pub(crate) positive: bool,
    pub(crate) years: u32,
    pub(crate) months: u32,
    pub(crate) days: u32,
    pub(crate) hours: u32,
    pub(crate) minutes: u32,
    pub(crate) seconds: u32,
    pub(crate) microseconds: u32
}

impl Value for Duration {
    
}

#[derive(Debug, Clone)]
pub struct YearMonthDuration  {
    pub(crate) positive: bool,
    pub(crate) years: u32,
    pub(crate) months: u32
}

impl Value for YearMonthDuration {

}

#[derive(Debug, Clone)]
pub struct DayTimeDuration {
    pub(crate) positive: bool,
    pub(crate) days: u32,
    pub(crate) hours: u32,
    pub(crate) minutes: u32,
    pub(crate) seconds: u32,
    pub(crate) microseconds: u32
}

impl Value for DayTimeDuration {

}
