use nom::{
    branch::alt,
    bytes::complete::{tag, take_while, take_while_m_n},
    character::complete::{one_of},
    error::Error, IResult
};

use nom::sequence::{terminated, preceded, tuple};
use nom::combinator::{opt, map};

use crate::eval::Type;
use crate::parser::parse_literal::is_digits;

pub fn string_to_date(input: &str) -> Result<Type, String> {
    match parse_date(input) {
        Ok((input, dt)) => {
            if input.is_empty() {
                Ok(dt)
            } else {
                Err(String::from(input))
            }
        },
        Err(e) => {
            Err(format!("{:?}", e))
        }
    }
}
pub fn string_to_duration(input: &str) -> Result<Type, String> {
    match parse_duration(input) {
        Ok((input, dt)) => {
            if input.is_empty() {
                Ok(dt)
            } else {
                Err(String::from(input))
            }
        },
        Err(e) => {
            Err(format!("{:?}", e))
        }
    }
}

pub fn string_to_ym_duration(input: &str) -> Result<Type, String> {
    match parse_year_month_duration(input) {
        Ok((input, dt)) => {
            if input.is_empty() {
                Ok(dt)
            } else {
                Err(String::from(input))
            }
        },
        Err(e) => {
            Err(format!("{:?}", e))
        }
    }
}

pub fn string_to_dt_duration(input: &str) -> Result<Type, String> {
    match parse_day_time_duration(input) {
        Ok((input, dt)) => {
            if input.is_empty() {
                Ok(dt)
            } else {
                Err(String::from(input))
            }
        },
        Err(e) => {
            Err(format!("{:?}", e))
        }
    }
}

pub fn parse_date(input: &str) -> IResult<&str, Type> {
    map(
        tuple((
            parse_year,
            tag("-"),
            parse_month,
            tag("-"),
            parse_day,
            opt(alt((timezone_hour, timezone_utc))),
        )),
        |(y, _, m, _, d, z)| {
            let (tz_h, tz_m) = z.unwrap_or((0, 0));
            Type::Date { y, m, d, tz_h, tz_m }
        }
    )(input)
}

fn timezone_hour(input: &str) -> IResult<&str, (i32, i32)> {
    map(
        tuple((
            opt(alt((tag("+"), tag("-")))),
            time_hour,
            opt(preceded(tag(":"), time_minute))
        )),
        |(sign, h, m)| {
            let s = if let Some(ch) = sign {
                match ch {
                    "+" => 1,
                    "-" => -1,
                    _ => 1
                }
            } else { 1 };

            (s * h as i32, s * m.unwrap_or(0) as i32)
        }
    )(input)
}

fn timezone_utc(input: &str) -> IResult<&str, (i32, i32)> {
    map(tag("Z"), |_| (0, 0))(input)
}

fn parse_duration(input: &str) -> IResult<&str, Type> {
    map(
        tuple((
            opt(tag("-")),
            preceded(
                tag("P"),
                tuple((
                    opt(terminated(take_digits, tag("Y"))),
                    opt(terminated(parse_month, tag("M"))),
                    opt(terminated(duration_day, tag("D"))),
                    opt(preceded(tag("T"), parse_duration_time)),
                ))
            )
        )),
        |(sign, (y, m, d, time))| {
            let positive = sign.is_none();

            let years = y.unwrap_or(0);
            let month = m.unwrap_or(0);
            let days = d.unwrap_or(0);

            let (hours, minutes, seconds, microseconds) = match time {
                Some(Type::Time { h, m, s, ms, ..}) => {
                    (h, m, s, ms)
                }
                _ => (0,0,0,0)
            };

            // normalization
            let (s, am) = norm(seconds, 60);
            let (m, ah) = norm(minutes + am, 60);
            let (h, ad) = norm(hours + ah, 24);

            let d = days + ad;

            let (mm, ay) = norm(month, 12);
            let y = years + ay;

            Type::Duration { positive, years: y, months: mm, days: d, hours: h, minutes: m, seconds: s, microseconds }
        }
    )(input)
}

pub fn parse_year_month_duration(input: &str) -> IResult<&str, Type> {
    map(
        tuple((
            opt(tag("-")),
            preceded(
                tag("P"),
                tuple((
                    opt(terminated(take_digits, tag("Y"))),
                    opt(terminated(parse_month, tag("M"))),
                ))
            )
        )),
        |(sign, (y, m))| {
            let positive = sign.is_none();
            let years = y.unwrap_or(0);
            let months = m.unwrap_or(0);

            let (m, ay) = norm(months, 12);
            let y = years + ay;

            Type::YearMonthDuration { positive, years: y, months: m }
        }
    )(input)
}

pub fn parse_day_time_duration(input: &str) -> IResult<&str, Type> {
    map(
        tuple((
            opt(tag("-")),
            preceded(
                tag("P"),
                tuple((
                    opt(terminated(duration_day, tag("D"))),
                    opt(preceded(tag("T"), parse_duration_time)),
                ))
            )
        )),
        |(sign, (d, time))| {
            let positive = sign.is_none();
            let days = d.unwrap_or(0);

            let (hours, minutes, seconds, microseconds) = match time {
                Some(Type::Time { h, m, s, ms, ..}) => {
                    (h, m, s, ms)
                }
                _ => (0,0,0,0)
            };

            // normalization
            let (s, am) = norm(seconds, 60);
            let (m, ah) = norm(minutes + am, 60);
            let (h, ad) = norm(hours + ah, 24);

            let d = days + ad;

            Type::DayTimeDuration { positive, days: d, hours: h, minutes: m, seconds: s, microseconds }
        }
    )(input)
}

fn parse_duration_time(input: &str) -> IResult<&str, Type> {
    map(
        tuple((
            opt(terminated(duration_hour, tag("H"))),
            opt(terminated(duration_minute, tag("M"))),
            opt(terminated(duration_second_and_ms, tag("S"))),
        )),
        |(h, m, s)| {
            let h = h.unwrap_or(0);
            let m = m.unwrap_or(0);

            let (s, ms) = s.unwrap_or((0,0));

            Type::Time { h, m, s, ms, tz_h: 0, tz_m: 0 }
        }
    )(input)
}

fn parse_year(input: &str) -> IResult<&str, u32> {
    digit_in_range(input, (1, 4), 0..=9999)
}

fn parse_month(input: &str) -> IResult<&str, u32> {
    digit_in_range(input, (1, 2), 0..=12)
}

fn parse_day(input: &str) -> IResult<&str, u32> {
    digit_in_range(input, (1, 2), 0..=31)
}

fn time_hour(input: &str) -> IResult<&str, u32> {
    digit_in_range(input, (2, 2), 0..=24)
}

fn time_minute(input: &str) -> IResult<&str, u32> {
    digit_in_range(input, (2, 2), 0..=59)
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

    let res = digits
        .parse()
        .expect("expected digits");

    Ok((input, res))
}

fn norm(value: u32, max: u32) -> (u32, u32) {
    let mut v = value;
    let mut count = 0;
    while v > max {
        v = v - max;
        count += 1;
    }
    (v, count)
}