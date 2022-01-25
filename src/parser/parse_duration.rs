use nom::{
    branch::alt,
    bytes::complete::{tag, take_while, take_while_m_n},
    character::complete::{one_of},
    error::Error, IResult
};

use nom::sequence::{terminated, preceded, tuple};
use nom::combinator::{opt, map, map_res, complete, all_consuming};

use crate::parser::parse_literal::is_digits;
use chrono::{Date, NaiveDate, FixedOffset, NaiveTime, DateTime, NaiveDateTime, TimeZone, LocalResult};
use nom::error::{ParseError, ErrorKind};
use nom::multi::many0;
use crate::values::Type;

pub fn string_to_date(input: &str) -> Result<Type, String> {
    match all_consuming(parse_date)(input.trim()) {
        Ok((_, date)) => Ok(date),
        Err(e) => Err(String::from("TODO"))
    }
}

pub fn string_to_time(input: &str) -> Result<Type, String> {
    match all_consuming(parse_time)(input.trim()) {
        Ok((_, time)) => Ok(time),
        Err(e) => Err(String::from("TODO"))
    }
}

pub fn string_to_date_time(input: &str) -> Result<Type, String> {
    match all_consuming(parse_date_time)(input.trim()) {
        Ok((_, dt)) => Ok(dt),
        Err(e) => Err(String::from("TODO"))
    }
}

pub fn string_to_duration(input: &str) -> Result<Type, String> {
    match all_consuming(parse_duration)(input.trim()) {
        Ok((_, duration)) => Ok(duration),
        Err(e) => Err(String::from("TODO"))
    }
}

pub fn string_to_ym_duration(input: &str) -> Result<Type, String> {
    match all_consuming(parse_year_month_duration)(input.trim()) {
        Ok((_, duration)) => Ok(duration),
        Err(e) => Err(String::from("TODO"))
    }
}

pub fn string_to_date_time_duration(input: &str) -> Result<Type, String> {
    match all_consuming(parse_day_time_duration)(input.trim()) {
        Ok((_, dt)) => Ok(dt),
        Err(e) => Err(String::from("TODO"))
    }
}

pub fn string_to_year_month(input: &str) -> Result<Type, String> {
    match all_consuming(parse_g_year_month)(input.trim()) {
        Ok((_, result)) => result,
        Err(e) => Err(format!("can't convert to GYearMonth: {:?}", input))
    }
}

pub fn parse_g_year_month(input: &str) -> IResult<&str, Result<Type, String>> {
    map(
        tuple((
            opt(tag("-")),
            take_digits, // parse_year,
            preceded(tag("-"), parse_month),
            opt(alt((timezone_hour, timezone_utc))),
        )),
        |(sign, year, month, tz_m)| {
            let year = if sign.is_some() {
                -(year as i32)
            } else {
                year as i32
            };

            new_g_year_month(year, month, tz_m)
        }
    )(input)
}

pub fn string_to_year(input: &str) -> Result<Type, String> {
    match all_consuming(parse_g_year)(input.trim()) {
        Ok((_, dt)) => Ok(dt),
        Err(e) => Err(format!("can't convert to GYear: {:?}", input))
    }
}

pub fn parse_g_year(input: &str) -> IResult<&str, Type> {
    map(
        tuple((
            opt(tag("-")),
            take_digits, // parse_year,
            opt(alt((timezone_hour, timezone_utc))),
        )),
        |(sign, year, tz_m)| {
            let year = if sign.is_some() {
                -(year as i32)
            } else {
                year as i32
            };

            Type::GYear { year, tz_m }
        }
    )(input)
}

pub fn string_to_month_day(input: &str) -> Result<Type, String> {
    match all_consuming(parse_g_month_day)(input.trim()) {
        Ok((_, result)) => result,
        Err(e) => Err(format!("can't convert to GMonthDay: {:?}", input))
    }
}

pub fn parse_g_month_day(input: &str) -> IResult<&str, Result<Type, String>> {
    map(
        tuple((
            tag("--"),
            take_digits, // parse_month,
            tag("-"),
            parse_day,
            opt(alt((timezone_hour, timezone_utc))),
        )),
        |(_, month, _, day, tz_m)| {
            new_g_month_day(month, day, tz_m)
        }
    )(input)
}

pub fn string_to_day(input: &str) -> Result<Type, String> {
    match all_consuming(parse_g_day)(input.trim()) {
        Ok((_, dt)) => Ok(dt),
        Err(e) => Err(format!("can't convert to GDay: {:?}", input))
    }
}

pub fn parse_g_day(input: &str) -> IResult<&str, Type> {
    map(
        tuple((
            tag("---"),
            // take_digits,
            parse_day,
            opt(alt((timezone_hour, timezone_utc))),
        )),
        |(_, day, tz_m)| {
            Type::GDay { day, tz_m }
        }
    )(input)
}

pub fn string_to_month(input: &str) -> Result<Type, String> {
    match all_consuming(parse_g_month)(input.trim()) {
        Ok((_, dt)) => Ok(dt),
        Err(e) => Err(format!("can't convert to GMonth: {:?}", input))
    }
}

pub fn parse_g_month_complete(input: &str) -> IResult<&str, Type> {
    all_consuming(parse_g_month)(input.trim())
}

pub fn parse_g_month(input: &str) -> IResult<&str, Type> {
    map(
        tuple((
            tag("--"),
            take_digits, // parse_day,
            opt(alt((timezone_hour, timezone_utc))),
        )),
        |(sign, month, tz_m)| {
            Type::GMonth { month, tz_m }
        }
    )(input)
}

pub fn parse_date_complete(input: &str) -> IResult<&str, Type> {
    all_consuming(parse_date)(input.trim())
}

pub fn parse_date(input: &str) -> IResult<&str, Type> {
    map_res(
        tuple((
            opt(tag("-")),
            parse_year,
            preceded(tag("-"), parse_month),
            preceded(tag("-"), parse_day),
            opt(alt((timezone_hour, timezone_utc))),
        )),
        |(sign, y, m, d, tz)| {
            let y = if sign.is_some() {
                -(y as i32)
            } else {
                y as i32
            };

            let tz_m = tz.unwrap_or(0);
            let offset = if tz_m >= 0 {
                FixedOffset::east(tz_m * 60)
            } else {
                FixedOffset::west(-tz_m * 60)
            };

            match offset.ymd_opt(y, m, d) {
                LocalResult::Single(date) => {
                    Ok(Type::Date { date, offset: tz.is_some() })
                }
                LocalResult::None |
                LocalResult::Ambiguous(_, _) => {
                    Err(nom::Err::Failure(Error::from_error_kind(input, ErrorKind::MapRes)))
                }
            }
        }
    )(input)
}

pub fn parse_time_complete(input: &str) -> IResult<&str, Type> {
    all_consuming(parse_time)(input.trim())
}

// 12:00:00-01:00
pub fn parse_time(input: &str) -> IResult<&str, Type> {
    map_res(
        tuple((
            parse_hour,
            preceded(tag(":"), parse_minute),
            preceded(tag(":"), parse_second_and_ms),
            opt(alt((timezone_hour, timezone_utc))),
        )),
        |(mut h, m, (s, ms), tz)| {
            // workaround for 24:00:00 case
            if h == 24 {
                if m != 0 && s != 0 && ms.unwrap_or(0) != 0 {
                    return Err(nom::Err::Failure(Error::from_error_kind(input, ErrorKind::MapRes)));
                }
                h = 0;
            }

            let time = if let Some(ms) = ms {
                if let Some(time) = NaiveTime::from_hms_milli_opt(h, m, s, ms) {
                    time
                } else {
                    return Err(nom::Err::Failure(Error::from_error_kind(input, ErrorKind::MapRes)));
                }
            } else {
                if let Some(time) = NaiveTime::from_hms_opt(h, m, s) {
                    time
                } else {
                    return Err(nom::Err::Failure(Error::from_error_kind(input, ErrorKind::MapRes)));
                }
            };

            if check_tz(tz) {
                Err(nom::Err::Failure(Error::from_error_kind(input, ErrorKind::MapRes)))
            } else {
                let tz_m = tz.unwrap_or(0);
                let offset = if tz_m > 0 {
                    FixedOffset::east(tz_m * 60)
                } else {
                    FixedOffset::west(-tz_m * 60)
                };
                Ok(Type::Time { time: crate::values::time::Time::from(time, offset), offset: tz.is_some() })
            }
        }
    )(input)
}

pub fn parse_date_time_complete(input: &str) -> IResult<&str, Type> {
    all_consuming(parse_date_time)(input.trim())
}

// 2002-04-02T12:00:00-01:00
pub fn parse_date_time(input: &str) -> IResult<&str, Type> {
    map_res(
        tuple((
            opt(tag("-")),
            parse_year,
            preceded(tag("-"), parse_month),
            preceded(tag("-"), parse_day),
            preceded(tag("T"), parse_hour),
            preceded(tag(":"), parse_minute),
            preceded(tag(":"), parse_second_and_ms),
            opt(alt((timezone_hour, timezone_utc))),
        )),
        |(sign, yy, mm, dd, h, m, (s, ms), tz)| {
            let yy = if sign.is_some() {
                -(yy as i32)
            } else {
                yy as i32
            };

            let tz_m = tz.unwrap_or(0);
            let offset = if tz_m >= 0 {
                FixedOffset::east(tz_m * 60)
            } else {
                FixedOffset::west(-tz_m * 60)
            };

            let ms = if let Some(ms) = ms { ms } else { 0 };

            match offset.ymd_opt(yy, mm, dd) {
                LocalResult::Single(date) => {
                    if h == 24 && m == 0 && s == 0 && ms == 0 {
                        match date.checked_add_signed(chrono::Duration::days(1)) {
                            Some(date) => {
                                match date.and_hms_opt(0, 0, 0) {
                                    Some(dt) => Ok(Type::DateTime { dt, offset: tz.is_some() }),
                                    None => Err(nom::Err::Failure(Error::from_error_kind(input, ErrorKind::MapRes)))
                                }
                            }
                            None => Err(nom::Err::Failure(Error::from_error_kind(input, ErrorKind::MapRes)))
                        }
                    } else {
                        match date.and_hms_milli_opt(h, m, s, ms) {
                            Some(dt) => Ok(Type::DateTime { dt, offset: tz.is_some() }),
                            None => Err(nom::Err::Failure(Error::from_error_kind(input, ErrorKind::MapRes)))
                        }
                    }
                }
                LocalResult::None |
                LocalResult::Ambiguous(_, _) => {
                    Err(nom::Err::Failure(Error::from_error_kind(input, ErrorKind::MapRes)))
                }
            }
        }
    )(input)
}

fn timezone_hour(input: &str) -> IResult<&str, i32> {
    map_res(
        tuple((
            opt(alt((tag("+"), tag("-")))),
            parse_hour,
            // opt(preceded(tag(":"), parse_minute))
            preceded(tag(":"), parse_minute)
        )),
        |(sign, h, m)| {
            let s: i32 = if let Some(ch) = sign {
                match ch {
                    "+" => 1,
                    "-" => -1,
                    _ => 1
                }
            } else { 1 };

            // if let Some(m) = m {
            if m >= 60 {
                return Err(nom::Err::Failure(Error::from_error_kind(input, ErrorKind::MapRes)))
            }
            // }

            let tz_m = s * ((h * 60) + m) as i32;
            Ok(tz_m)
        }
    )(input)
}

fn timezone_utc(input: &str) -> IResult<&str, i32> {
    map(tag("Z"), |_| 0)(input)
}

pub(crate) fn parse_duration_complete(input: &str) -> IResult<&str, Type> {
    all_consuming(parse_duration)(input.trim())
}

fn parse_duration(input: &str) -> IResult<&str, Type> {
    map_res(
        tuple((
            opt(tag("-")),
            preceded(
                tag("P"),
                tuple((
                    opt(terminated(take_digits, tag("Y"))),
                    opt(terminated(take_digits, tag("M"))),
                    opt(terminated(duration_day, tag("D"))),
                    opt(preceded(tag("T"), parse_duration_time_as_u32)),
                ))
            )
        )),
        |(sign, (y, m, d, time))| {
            if y.is_none() && m.is_none() && d.is_none() && time.is_none() {
                Err(nom::Err::Failure(Error::from_error_kind(input, ErrorKind::MapRes)))
            } else {
                let positive = sign.is_none();

                let years = y.unwrap_or(0);
                let month = m.unwrap_or(0);
                let days = d.unwrap_or(0);

                let (hours, minutes, seconds, microseconds) = time.unwrap_or((0, 0, 0, 0));

                // normalization
                // let (s, am) = norm(seconds, 60);
                // let (m, ah) = norm(minutes + am, 60);
                let (h, ad) = norm(hours, 24); // norm(hours + ah, 24);

                let d = days + ad;

                let (mm, ay) = norm(month, 12);
                let y = years + ay;

                Ok(Type::Duration { positive, years: y, months: mm, days: d, hours: h, minutes, seconds, microseconds })
            }
        }
    )(input)
}

pub fn parse_year_month_duration_complete(input: &str) -> IResult<&str, Type> {
    all_consuming(parse_year_month_duration)(input.trim())
}

pub fn parse_year_month_duration(input: &str) -> IResult<&str, Type> {
    map_res(
        tuple((
            opt(tag("-")),
            preceded(
                tag("P"),
                tuple((
                    opt(terminated(take_digits, tag("Y"))),
                    opt(terminated(take_digits, tag("M"))),
                ))
            )
        )),
        |(sign, (y, m))| {
            if y.is_none() && m.is_none() {
                Err(nom::Err::Failure(Error::from_error_kind(input, ErrorKind::MapRes)))
            } else {
                let positive = sign.is_none();
                let years = y.unwrap_or(0);
                let months = m.unwrap_or(0);

                let (m, ay) = norm(months, 12);
                let y = years + ay;

                Ok(Type::YearMonthDuration { positive, years: y, months: m })
            }
        }
    )(input)
}
pub fn parse_day_time_duration_complete(input: &str) -> IResult<&str, Type> {
    all_consuming(parse_day_time_duration)(input.trim())
}

pub fn parse_day_time_duration(input: &str) -> IResult<&str, Type> {
    map_res(
        tuple((
            opt(tag("-")),
            preceded(
                tag("P"),
                tuple((
                    opt(terminated(duration_day, tag("D"))),
                    opt(preceded(tag("T"), parse_duration_time_as_u32)),
                ))
            )
        )),
        |(sign, (d, time))| {
            if d.is_none() && time.is_none() {
                Err(nom::Err::Failure(Error::from_error_kind(input, ErrorKind::MapRes)))
            } else {
                let positive = sign.is_none();
                let days = d.unwrap_or(0);

                let (hours, minutes, seconds, microseconds) = time.unwrap_or((0, 0, 0, 0));

                // normalization
                let (s, am) = norm(seconds, 60);
                let (m, ah) = norm(minutes + am, 60);
                let (h, ad) = norm(hours + ah, 24);

                let d = days + ad;

                Ok(Type::DayTimeDuration { positive, days: d, hours: h, minutes: m, seconds: s, microseconds })
            }
        }
    )(input)
}

fn parse_duration_time(input: &str) -> IResult<&str, Type> {
    map_res(
        tuple((
            opt(terminated(take_digits, tag("H"))), // duration_hour
            opt(terminated(take_digits, tag("M"))), // duration_minute
            opt(terminated(duration_second_and_ms, tag("S"))),
        )),
        |(h, m, s)| {
            if h.is_none() && m.is_none() && s.is_none() {
                Err(nom::Err::Failure(Error::from_error_kind(input, ErrorKind::MapRes)))
            } else {
                let h = h.unwrap_or(0);
                let m = m.unwrap_or(0);

                let (s, ms) = s.unwrap_or((0, 0));

                // normalization
                let (s, am) = norm(s, 60);
                let (m, ah) = norm(m + am, 60);
                let (h, ad) = norm(h + ah, 24);

                if let Some(time) = NaiveTime::from_hms_milli_opt(h, m, s, ms) {
                    Ok(Type::Time { time: crate::values::time::Time::from_utc(time), offset: false })
                } else {
                    Err(nom::Err::Failure(Error::from_error_kind(input, ErrorKind::MapRes)))
                }
            }
        }
    )(input)
}

fn parse_duration_time_as_u32(input: &str) -> IResult<&str, (u32,u32,u32,u32)> {
    map_res(
        tuple((
            opt(terminated(take_digits, tag("H"))), // duration_hour
            opt(terminated(take_digits, tag("M"))), // duration_minute
            opt(terminated(duration_second_and_ms, tag("S"))),
        )),
        |(h, m, s)| {
            if h.is_none() && m.is_none() && s.is_none() {
                Err(nom::Err::Failure(Error::from_error_kind(input, ErrorKind::MapRes)))
            } else {
                let h = h.unwrap_or(0);
                let m = m.unwrap_or(0);

                let (s, ms) = s.unwrap_or((0, 0));

                // normalization
                let (s, am) = norm(s, 60);
                let (m, ah) = norm(m + am, 60);
                let h = h + ah; // let (h, ad) = norm(h + ah, 24);

                Ok((h, m, s, ms))
            }
        }
    )(input)
}

fn parse_year(input: &str) -> IResult<&str, u32> {
    digit_in_range(input, (4, 4), 0..=9999)
}

fn parse_month(input: &str) -> IResult<&str, u32> {
    digit_in_range(input, (2, 2), 0..=12)
}

fn parse_month_12(input: &str) -> IResult<&str, u32> {
    digit_in_range(input, (1, 2), 0..=12)
}

fn parse_day(input: &str) -> IResult<&str, u32> {
    digit_in_range(input, (2, 2), 0..=31)
}

fn parse_day_12(input: &str) -> IResult<&str, u32> {
    digit_in_range(input, (1, 2), 0..=31)
}

fn parse_hour(input: &str) -> IResult<&str, u32> {
    digit_in_range(input, (2, 2), 0..=24)
}

fn parse_minute(input: &str) -> IResult<&str, u32> {
    digit_in_range(input, (2, 2), 0..=59)
}

fn parse_second(input: &str) -> IResult<&str, u32> {
    digit_in_range(input, (2, 2), 0..=59)
}

fn parse_second_and_ms(input: &str) -> IResult<&str, (u32, Option<u32>)> {
    tuple((
        // take_digits,
        parse_second,
        opt(preceded(tag("."), take_digits))
    ))(input)
}

fn duration_day(input: &str) -> IResult<&str, u32> {
    digit_in_range(input, (1, 10), 0..=4_294_967_295)
}

fn duration_hour(input: &str) -> IResult<&str, u32> {
    digit_in_range(input, (1, 2), 0..=24)
}

fn duration_minute(input: &str) -> IResult<&str, u32> {
    digit_in_range(input, (1, 2), 0..=60)
}

fn duration_second_and_ms(input: &str) -> IResult<&str, (u32, u32)> {
    let (input, sec) = digit_in_range(input, (1, 2), 0..=60)?;

    let (input, ms) = opt(
        preceded(
            one_of(",."),
            duration_ms
        )
    )(input)?;

    Ok((input, (sec, ms.unwrap_or(0))))
}

fn duration_ms(input: &str) -> IResult<&str, u32> {
    let (input, digits) = take_while(is_digits)(input)?;

    let num: f32 = format!("0.{}", digits).parse().unwrap();
    let num = (num * 1000.0) as u32;

    Ok((input, num))
}

fn digit_in_range(
    input: &str,
    min_max: (usize, usize),
    range: impl core::ops::RangeBounds<u32>
) -> IResult<&str, u32> {
    let (input, number) = take_while_m_n(min_max.0, min_max.1, is_digits)(input)?;

    let num = number.parse().expect("number");
    Ok((input, num))
}

fn take_digits(input: &str) -> IResult<&str, u32> {
    let (input, digits) = take_while(is_digits)(input)?;

    if digits.is_empty() {
        return Err(nom::Err::Error(Error::new(input, nom::error::ErrorKind::Digit)));
    }

    let res = match digits.parse() {
        Ok(num) => num,
        Err(_) => return Err(nom::Err::Failure(Error::from_error_kind(input, ErrorKind::MapRes)))
    };

    Ok((input, res))
}

fn norm(value: u32, max: u32) -> (u32, u32) {
    let mut v = value;
    let mut count = 0;
    while v >= max {
        v = v - max;
        count += 1;
    }
    (v, count)
}

fn check_tz(tz_m: Option<i32>) -> bool {
    if let Some(tz) = tz_m {
        // Timezones are durations with (integer-valued) hour and minute properties
        // (with the hour magnitude limited to at most 14,
        // and the minute magnitude limited to at most 59,
        // except that if the hour magnitude is 14, the minute value must be 0);
        // they may be both positive or both negative.
        if tz < -840 || tz > 840 {
            return true;
        }
    }
    false
}

// // The ·recoverable timezone· of a date will always be a duration between '+12:00' and '11:59'.
// fn check_tz_12(tz_m: Option<i32>) -> bool {
//     if let Some(tz) = tz_m {
//         if tz < -720 || tz > 720 {
//             return true;
//         }
//     }
//     false
// }

pub(crate) fn new_g_year_month(year: i32, month: u32, tz_m: Option<i32>) -> Result<Type, String> {
    if month < 1 || month > 12 {
        Err(String::from("TODO"))
    } else if check_tz(tz_m) {
        Err(String::from("TODO"))
    } else {
        Ok(Type::GYearMonth { year, month, tz_m })
    }
}

pub(crate) fn new_g_month_day(month: u32, day: u32, tz_m: Option<i32>) -> Result<Type, String> {
    match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => {
            if day < 1 || day > 31 {
                return Err(String::from("TODO"))
            }
        }
        4 | 6 | 9 | 11 => {
            if day < 1 || day > 30 {
                return Err(String::from("TODO"))
            }
        }
        2 => {
            if day < 1 || day > 29 {
                return Err(String::from("TODO"))
            }
        }
        _ => return Err(String::from("TODO"))
    }

    if check_tz(tz_m) {
        return Err(String::from("TODO"))
    }

    Ok(Type::GMonthDay { month, day, tz_m })
}