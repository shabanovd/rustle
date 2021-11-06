use core::borrow::Borrow;
use std::cmp::Ordering;
use chrono::{DateTime, FixedOffset, Local, NaiveTime, Timelike, TimeZone};
use chrono::format::{DelayedFormat, Item, StrftimeItems};

use num_integer::div_mod_floor;

#[derive(Debug, Eq, PartialEq, Clone, Hash)]
pub struct ChronoTime<Tz: TimeZone> {
    pub time: NaiveTime,
    pub offset: Tz::Offset,
}

impl PartialOrd for ChronoTime<FixedOffset> {
    fn partial_cmp(&self, other: &ChronoTime<FixedOffset>) -> Option<Ordering> {
        self.time.partial_cmp(&other.time)
    }
}

impl Ord for ChronoTime<FixedOffset> {
    fn cmp(&self, other: &ChronoTime<FixedOffset>) -> Ordering {
        self.time.cmp(&other.time)
    }
}

impl ChronoTime<FixedOffset> {
    #[inline]
    pub fn now() -> ChronoTime<FixedOffset> {
        let now = Local::now();
        ChronoTime { time: now.time(), offset: TimeZone::from_offset(now.offset()) }
    }

    #[inline]
    pub fn from(dt: DateTime<Local>) -> ChronoTime<FixedOffset> {
        ChronoTime { time: dt.time(), offset: TimeZone::from_offset(dt.offset()) }
    }

    #[inline]
    pub fn from_utc(time: NaiveTime) -> ChronoTime<FixedOffset> {
        ChronoTime { time, offset: FixedOffset::east(0) }
    }

    pub fn hms(&self) -> (u32, u32, u32, u32) {
        let (mins, sec) = div_mod_floor(self.time.num_seconds_from_midnight(), 60);
        let (hour, min) = div_mod_floor(mins, 60);
        (hour, min, sec, 0)
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