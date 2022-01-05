use bigdecimal::{BigDecimal, Zero};
use chrono::{Date, DateTime, FixedOffset, SecondsFormat, Timelike};
use ordered_float::OrderedFloat;
use crate::eval::{Object, Type, RangeIterator, Environment};
use crate::parser::op::Representation;
use crate::values::{binary_base64_to_string, binary_hex_to_string};
use crate::values::time::Time;
use std::num;
use crate::values::string_to::decimal;

pub fn object_to_string_xml(env: &Box<Environment>, object: &Object) -> String {
    _object_to_string(env, object, false, " ")
}

pub fn object_to_string(env: &Box<Environment>, object: &Object) -> String {
    _object_to_string(env, object, true, " ")
}

pub fn _object_to_string(env: &Box<Environment>, object: &Object, ref_resolving: bool, sep: &str) -> String {
    match object {
        Object::Empty => String::new(),
        Object::Range { min, max } => {
            let (it, count) = RangeIterator::create(*min, *max);

            let mut buf = Vec::with_capacity(count);
            for item in it {
                buf.push(_object_to_string(env, &item, ref_resolving, sep));
            }

            buf.join(sep)
        }
        Object::CharRef { representation, reference } => {
            if ref_resolving {
                String::from(ref_to_char(*reference))
            } else {
                match representation {
                    Representation::Hexadecimal => {
                        format!("&#x{:X}", reference)
                    }
                    Representation::Decimal => {
                        format!("&#{}", reference)
                    }
                }
            }
        },
        Object::EntityRef(reference) => {
            match reference.as_str() {
                "lt" => String::from("<"),
                "gt" => String::from(">"),
                "amp" => String::from("&"),
                "quot" => String::from("\""),
                "apos" => String::from("'"),
                _ => panic!("unexpected {:?}", reference)
            }
        },
        Object::Atomic(t) => {
            match t {
                Type::Boolean(b) => b.to_string(),
                Type::AnyURI(uri) => uri.to_string(),

                Type::Untyped(str) |
                Type::String(str) |
                Type::NormalizedString(str) |

                Type::ID(str) |
                Type::IDREF(str) |
                Type::ENTITY(str) |

                Type::Token(str) |
                Type::Language(str) |
                Type::NMTOKEN(str) |
                Type::Name(str) |
                Type::NCName(str) => str.clone(),
                Type::QName { prefix, local_part, .. } => {
                    qname_to_string(prefix, local_part)
                }

                Type::Long(number) => number.to_string(),
                Type::Int(number) => number.to_string(),
                Type::Short(number) => number.to_string(),
                Type::Byte(number) => number.to_string(),

                Type::UnsignedByte(number) => number.to_string(),
                Type::UnsignedShort(number) => number.to_string(),
                Type::UnsignedInt(number) => number.to_string(),
                Type::UnsignedLong(number) => number.to_string(),

                Type::PositiveInteger(number) |
                Type::NonNegativeInteger(number) |
                Type::NonPositiveInteger(number) |
                Type::NegativeInteger(number) |
                Type::Integer(number) => number.to_string(),
                Type::Decimal(number) => decimal_to_string(number),
                Type::Float(number) => float_to_string(number, true),
                Type::Double(number) => double_to_string(number, true),

                Type::DateTimeStamp() => todo!(),
                Type::DateTime { dt, offset } => {
                    date_time_to_string(dt, offset)
                }
                Type::Date { date, offset } => {
                    date_to_string(date, offset)
                }
                Type::Time { time, offset } => {
                    time_to_string(time, offset)
                }
                Type::GYearMonth { year, month, tz_m } => {
                    g_year_month_to_string(*year, *month, *tz_m)
                }
                Type::GYear { year, tz_m } => {
                    g_year_to_string(*year, *tz_m)
                }
                Type::GMonthDay { month, day, tz_m } => {
                    g_month_day_to_string(*month, *day, *tz_m)
                }
                Type::GDay { day, tz_m } => {
                    g_day_to_string(*day, *tz_m)
                }
                Type::GMonth { month, tz_m } => {
                    g_month_to_string(*month, *tz_m)
                }
                Type::Duration { positive, years, months, days, hours, minutes, seconds, microseconds } => {
                    duration_to_string(*positive, *years, *months, *days, *hours, *minutes, *seconds, *microseconds)
                }
                Type::YearMonthDuration { positive, years, months } => {
                    year_month_duration_to_string(*positive, *years, *months)
                }
                Type::DayTimeDuration { positive, days, hours, minutes, seconds, microseconds } => {
                    day_time_duration_to_string(*positive, *days, *hours, *minutes, *seconds, *microseconds)
                }
                Type::Base64Binary(binary) => {
                    match binary_base64_to_string(binary) {
                        Ok(data) => data,
                        Err(code) => panic!("{:?}", code)
                    }
                }
                Type::HexBinary(binary) => {
                    match binary_hex_to_string(binary) {
                        Ok(data) => data,
                        Err(code) => panic!("{:?}", code)
                    }
                }

                Type::NOTATION() => todo!()
            }
        }
        Object::Array(items) |
        Object::Sequence(items) => {
            let mut buf = Vec::with_capacity(items.len());
            for item in items {
                let data = _object_to_string(env, item, ref_resolving, " ");
                buf.push(data);
            }
            let data = buf.join(sep);
            data
        },
        Object::Node(rf) => {
            match rf.to_typed_value() {
                Ok(data) => data,
                Err(msg) => panic!("{}", msg)
            }
        },
        _ => panic!("TODO object_to_string {:?}", object)
    }
}

pub(crate) fn decimal_to_string(number: &BigDecimal) -> String {
    let str = number.to_string();
    if str.ends_with(".0") {
        str[..str.len()-2].to_string()
    } else {
        str
    }
}

pub(crate) fn float_to_string(number: &OrderedFloat<f32>, rules: bool) -> String {
    if number.is_nan() {
        String::from("NaN")
    } else if number.is_infinite() {
        if number.is_sign_positive() {
            String::from("INF")
        } else {
            String::from("-INF")
        }
    } else if number.is_zero() {
        number.to_string()
    } else if rules {
        // If SV has an absolute value that is greater than or equal to 0.000001 (one millionth)
        // and less than 1000000 (one million), then the value is converted to an xs:decimal
        // and the resulting xs:decimal is converted to an xs:string
        let abs = number.0.abs();
        if abs < 1e6 {
            if abs >= 1e-3 {
                number.to_string()
            } else if abs >= 1e-6 {
                number.to_string()
            } else {
                // ???
                format!("{:+E}", number.0)
            }
        } else if abs < 1e7 {
            //
            format!("{:+E}", number.0)
        } else {
            // science notation
            // format!("{:.precision$E}", number.0, precision = 1)
            let mut str = format!("{:E}", number.0);
            // workarounds
            str = if str.contains(".") {
                str
            } else {
                str.replace("E", ".0E")
            };

            str
        }
    } else {
        number.to_string()
    }
}

pub(crate) fn double_to_string(number: &OrderedFloat<f64>, rules: bool) -> String {
    if number.is_nan() {
        String::from("NaN")
    } else if number.is_infinite() {
        if number.is_sign_positive() {
            String::from("INF")
        } else {
            String::from("-INF")
        }
    } else if number.is_zero() {
        number.to_string()
    } else if rules {
        // If SV has an absolute value that is greater than or equal to 0.000001 (one millionth)
        // and less than 1000000 (one million), then the value is converted to an xs:decimal
        // and the resulting xs:decimal is converted to an xs:string
        let abs = number.0.abs();
        if abs < 1e6 {
            if abs >= 1e-3 {
                number.to_string()
            } else if abs >= 1e-6 {
                number.to_string()
            } else {
                // ???
                format!("{:+E}", number.0)
            }
        } else if abs < 1e7 {
            //
            format!("{:+E}", number.0)
        } else {
            // science notation
            // format!("{:.precision$E}", number.0, precision = 1)
            let mut str = format!("{:E}", number.0);
            // workarounds
            str = if str.contains(".") {
                str
            } else {
                str.replace("E", ".0E")
            };

            str
        }
    } else {
        number.to_string()
    }
}

pub(crate) fn date_time_to_string(dt: &DateTime<FixedOffset>, offset: &bool) -> String {
    // let secform = if dt.time().nanosecond() != 0 { SecondsFormat::Secs } else { SecondsFormat::Secs };
    let str = dt.to_rfc3339_opts(SecondsFormat::AutoSi, true);
    if *offset {
        str
    } else {
        str[0..str.len() - 1].to_string()
    }
}

pub(crate) fn date_to_string(date: &Date<FixedOffset>, offset: &bool) -> String {
    if *offset {
        if date.offset().to_string() == "+00:00" {
            date.format("%Y-%m-%dZ").to_string()
        } else {
            date.format("%Y-%m-%d%:z").to_string()
        }
    } else {
        date.format("%Y-%m-%d").to_string()
    }
}

pub(crate) fn time_to_string(time: &Time<FixedOffset>, offset: &bool) -> String {
    // time.format("%H:%M:%S%Z").to_string()
    let str = if time.nanosecond() == 0 {
        time.to_rfc3339_opts(SecondsFormat::Secs, true)
    } else {
        time.to_rfc3339_opts(SecondsFormat::Millis, true)
    };
    if *offset {
        str
    } else {
        str[0..str.len() - 1].to_string()
    }
}

pub(crate) fn g_year_month_to_string(year: i32, month: u32, tz_m: Option<i32>) -> String {
    let (date_sign, year) = if year >= 0 { ("", year) } else { ("-", -year) };
    if let Some(tz) = tz_m {
        let (sign, mut tz) = if tz >= 0 { ("+", tz) } else { ("-", -tz) };
        let m = tz % 60;
        let h = tz / 60;
        if h == 0 && m == 0 {
            format!("{}{:04}-{:02}Z", date_sign, year, month)
        } else {
            format!("{}{:04}-{:02}{}{:02}:{:02}", date_sign, year, month, sign, h, m)
        }
    } else {
        format!("{}{:04}-{:02}", date_sign, year, month)
    }
}

pub(crate) fn g_year_to_string(year: i32, tz_m: Option<i32>) -> String {
    let (date_sign, year) = if year >= 0 { ("", year) } else { ("-", -year) };
    if let Some(tz) = tz_m {
        let (sign, mut tz) = if tz >= 0 { ("+", tz) } else { ("-", -tz) };
        let m = tz % 60;
        let h = tz / 60;
        if h == 0 && m == 0 {
            format!("{}{:04}Z", date_sign, year)
        } else {
            format!("{}{:04}{}{:02}:{:02}", date_sign, year, sign, h, m)
        }
    } else {
        format!("{}{:04}", date_sign, year)
    }
}

pub(crate) fn g_month_day_to_string(month: u32, day: u32, tz_m: Option<i32>) -> String {
    if let Some(tz) = tz_m {
        let (sign, mut tz) = if tz >= 0 { ("+", tz) } else { ("-", -tz) };
        let m = tz % 60;
        let h = tz / 60;
        if h == 0 && m == 0 {
            format!("--{:02}-{:02}Z", month, day)
        } else {
            format!("--{:02}-{:02}{}{:02}:{:02}", month, day, sign, h, m)
        }
    } else {
        format!("--{:02}-{:02}", month, day)
    }
}

pub(crate) fn g_day_to_string(day: u32, tz_m: Option<i32>) -> String {
    if let Some(tz) = tz_m {
        let (sign, mut tz) = if tz >= 0 { ("+", tz) } else { ("-", -tz) };
        let m = tz % 60;
        let h = tz / 60;
        if h == 0 && m == 0 {
            format!("---{:02}Z", day)
        } else {
            format!("---{:02}{}{:02}:{:02}", day, sign, h, m)
        }
    } else {
        format!("---{:02}", day)
    }
}

pub(crate) fn g_month_to_string(month: u32, tz_m: Option<i32>) -> String {
    if let Some(tz) = tz_m {
        let (sign, mut tz) = if tz >= 0 { ("+", tz) } else { ("-", -tz) };
        let m = tz % 60;
        let h = tz / 60;
        if h == 0 && m == 0 {
            format!("--{:02}Z", month)
        } else {
            format!("--{:02}{}{:02}:{:02}", month, sign, h, m)
        }
    } else {
        format!("--{:02}", month)
    }
}

pub(crate) fn duration_to_string(positive: bool, years: u32, months: u32, days: u32, hours: u32, minutes: u32, seconds: u32, microseconds: u32) -> String {
    let mut buf = String::new();
    if !positive {
        buf.push_str("-")
    }
    buf.push_str("P");
    if years != 0 {
        buf.push_str(years.to_string().as_str());
        buf.push_str("Y");
    }
    if months != 0 {
        buf.push_str(months.to_string().as_str());
        buf.push_str("M");
    }
    if days != 0 {
        buf.push_str(days.to_string().as_str());
        buf.push_str("D");
    }
    if hours == 0 && minutes == 0 && seconds == 0 && microseconds == 0 {
        // nothing to do here
    } else {
        buf.push_str("T");
        if hours != 0 {
            buf.push_str(hours.to_string().as_str());
            buf.push_str("H");
        }
        if minutes != 0 {
            buf.push_str(minutes.to_string().as_str());
            buf.push_str("M");
        }
        if seconds != 0 || microseconds != 0 {
            buf.push_str(seconds.to_string().as_str());
            if microseconds != 0 {
                buf.push_str(".");
                buf.push_str(microseconds.to_string().as_str());
            }
            buf.push_str("S");
        }
    }
    buf
}

pub(crate) fn year_month_duration_to_string(positive: bool, years: u32, months: u32) -> String {
    let mut buf = String::new();
    if !positive {
        if years != 0 || months != 0 {
            buf.push_str("-")
        }
    }
    buf.push_str("P");
    if years == 0 && months == 0 {
        buf.push_str(months.to_string().as_str());
        buf.push_str("M");
    } else {
        if years != 0 {
            buf.push_str(years.to_string().as_str());
            buf.push_str("Y");
        }
        if months != 0 {
            buf.push_str(months.to_string().as_str());
            buf.push_str("M");
        }
    }
    buf
}

pub(crate) fn day_time_duration_to_string(positive: bool, days: u32, hours: u32, minutes: u32, seconds: u32, microseconds: u32) -> String {
    let mut buf = String::new();
    if !positive {
        if days != 0 || hours != 0 || minutes != 0 || seconds != 0 || microseconds != 0 {
            buf.push_str("-")
        }
    }
    buf.push_str("P");
    if days != 0 {
        buf.push_str(days.to_string().as_str());
        buf.push_str("D");
    }
    if days != 0 && (hours == 0 && minutes == 0 && seconds == 0 && microseconds == 0) {
        // nothing to do here
    } else {
        buf.push_str("T");
        if hours == 0 && minutes == 0 && seconds == 0 && microseconds == 0 {
            buf.push_str(seconds.to_string().as_str());
            buf.push_str("S");
        } else {
            if hours != 0 {
                buf.push_str(hours.to_string().as_str());
                buf.push_str("H");
            }
            if minutes != 0 {
                buf.push_str(minutes.to_string().as_str());
                buf.push_str("M");
            }
            if seconds != 0 || microseconds != 0 {
                buf.push_str(seconds.to_string().as_str());
                if microseconds != 0 {
                    buf.push_str(".");
                    buf.push_str(microseconds.to_string().as_str());
                }
                buf.push_str("S");
            }
        }
    }
    buf
}

pub(crate) fn qname_to_string(prefix: &Option<String>, local_part: &String) -> String {
    let mut str = if let Some(prefix) = prefix {
        let mut str = String::with_capacity(local_part.len() + 1 + prefix.len());
        str.push_str(prefix.as_str());
        str.push_str(":");
        str
    } else {
        String::with_capacity(local_part.len())
    };
    str.push_str(local_part.as_str());
    str
}

pub(crate) fn ref_to_char(code: u32) -> char {
    char::from_u32(code).unwrap()
}