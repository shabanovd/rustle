use core::borrow::Borrow;
use std::cmp::Ordering;
use chrono::{DateTime, FixedOffset, Local, NaiveTime, SecondsFormat, Timelike, TimeZone};
use chrono::format::{DelayedFormat, Fixed, Item, StrftimeItems};
use chrono::format::Numeric::{Hour, Minute, Second};
use chrono::format::Pad::Zero;
use num_integer::div_mod_floor;

impl PartialOrd for Time<FixedOffset> {
    fn partial_cmp(&self, other: &Time<FixedOffset>) -> Option<Ordering> {
        self.time.partial_cmp(&other.time)
    }
}

impl Ord for Time<FixedOffset> {
    fn cmp(&self, other: &Time<FixedOffset>) -> Ordering {
        self.time.cmp(&other.time)
    }
}

#[derive(Debug, Eq, PartialEq, Clone, Hash)]
pub struct Time<Tz: TimeZone> {
    pub time: NaiveTime,
    pub offset: Tz::Offset,
}

impl Time<FixedOffset> {
    #[inline]
    pub fn now() -> Time<FixedOffset> {
        let now = Local::now();
        Time { time: now.time(), offset: TimeZone::from_offset(now.offset()) }
    }

    // #[inline]
    // pub fn from(dt: DateTime<Local>) -> Time<FixedOffset> {
    //     Time { time: dt.time(), offset: TimeZone::from_offset(dt.offset()) }
    // }

    #[inline]
    pub fn from(time: NaiveTime, offset: FixedOffset) -> Time<FixedOffset> {
        Time { time, offset }
    }

    #[inline]
    pub fn from_utc(time: NaiveTime) -> Time<FixedOffset> {
        Time { time, offset: FixedOffset::east(0) }
    }

    pub fn hms(&self) -> (u32, u32, u32, u32) {
        let (min, sec) = div_mod_floor(self.time.num_seconds_from_midnight(), 60);
        let (hour, min) = div_mod_floor(min, 60);
        (hour, min, sec, self.time.nanosecond())
    }

    pub fn nanosecond(&self) -> u32 {
        self.time.nanosecond()
    }

    pub fn to_rfc3339_opts(&self, secform: SecondsFormat, use_z: bool) -> String {
        use chrono::format::Numeric::*;
        use chrono::format::Pad::Zero;
        use chrono::SecondsFormat::*;

        debug_assert!(secform != __NonExhaustive, "Do not use __NonExhaustive!");

        const PREFIX: &'static [Item<'static>] = &[
            Item::Numeric(Hour, Zero),
            Item::Literal(":"),
            Item::Numeric(Minute, Zero),
            Item::Literal(":"),
            Item::Numeric(Second, Zero),
        ];

        let ssitem = match secform {
            Secs => None,
            Millis => Some(Item::Fixed(Fixed::Nanosecond3)),
            Micros => Some(Item::Fixed(Fixed::Nanosecond6)),
            Nanos => Some(Item::Fixed(Fixed::Nanosecond9)),
            AutoSi => Some(Item::Fixed(Fixed::Nanosecond)),
            __NonExhaustive => unreachable!(),
        };

        let tzitem = Item::Fixed(if use_z {
            Fixed::TimezoneOffsetColonZ
        } else {
            Fixed::TimezoneOffsetColon
        });

        match ssitem {
            None => self.format_with_items(PREFIX.iter().chain([tzitem].iter())).to_string(),
            Some(s) => self.format_with_items(PREFIX.iter().chain([s, tzitem].iter())).to_string(),
        }
    }

    #[inline]
    pub fn format<'a>(&self, fmt: &'a str) -> DelayedFormat<StrftimeItems<'a>> {
        self.format_with_items(StrftimeItems::new(fmt))
    }

    #[inline]
    pub fn format_with_items<'a, I, B>(&self, items: I) -> DelayedFormat<I>
        where
            I: Iterator<Item = B> + Clone,
            B: Borrow<Item<'a>>,
    {
        DelayedFormat::new_with_offset(None, Some(self.time), &self.offset, items)
    }
}