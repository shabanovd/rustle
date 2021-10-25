use std::convert::TryFrom;
use std::fmt;
use std::cmp::Ordering;
use rand::prelude::*;
use chrono::prelude::*;

const LEVEL_UP: u8 = 0;

const BITS_PER_UNIT: u8 = 4;
const SHIFT: u8 = 3;

const MAX_SHIFTS: u8 = 21;
lazy_static!(
    // number_of_bits = unit_number * BITS_PER_UNIT - unit_number;
    // biggest[i] = 2^number_of_bits + biggest[i - 1];
    pub static ref PER_COMPONENT_SIZE: [usize; MAX_SHIFTS as usize] = {
        let mut nums: [usize; MAX_SHIFTS as usize] = [0; MAX_SHIFTS as usize];

        let mut number_of_bits = 3;
        let mut biggest: u128 = 7; // one bit for document id
        let mut components: u8 = 1;

        nums[(components - 1) as usize] = biggest as usize;
        loop {
            components += 1;
            number_of_bits = bits_offset(components);
            biggest = 2_u128.pow(number_of_bits as u32) + biggest;
            if biggest < usize::MAX as u128 {
                nums[(components - 1) as usize] = biggest as usize;
            } else {
                break
            }
        }
        nums
    };

    pub static ref BIT_MASK: [u8; 8] = {
        let mut masks: [u8; 8] = [0; 8];
        masks[0] = 1 << 0;
        for i in 1..8 {
            let mask = 1 << i;
            masks[i] = mask + masks[i - 1];
        }
        masks
    };
);

fn units_required(levelId: usize) -> Result<u8, String> {
    for i in 0..MAX_SHIFTS as usize {
        let biggest = PER_COMPONENT_SIZE[i];
        if biggest == 0 {
            return Err("Internal limit exceeds".to_string());
        }
        if levelId < biggest {
            return Ok((i + 1) as u8);
        }
    }
    Err("Internal limit exceeds".to_string())
}

fn bit(pos: usize) -> u8 {
    1 << (pos & 7)
}

fn bits_offset(units: u8) -> u8 {
    (units * BITS_PER_UNIT) - units
}

fn get_bit_at(input: &u8, n: usize) -> bool {
    if n < 8 {
        input & (1 << n) != 0
    } else {
        false
    }
}

fn bits(i: u8) -> String {
    let mut buf = String::new();
    for pos in 0..8 {
        let ch = if get_bit_at(&i, pos) {
            '1'
        } else {
            '0'
        };
        buf.push(ch);
    }
    buf
}

#[derive(Clone)]
pub struct DLN {
    bits: Vec<u8>,
    pos: usize,
}

impl fmt::Display for DLN {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut buf = String::with_capacity(42);

        let mut offset = 0;
        while offset <= self.pos {
            if offset > 0 {
                if (self.bits[offset >> SHIFT] & bit(offset)) == 0 {
                    buf.push('.');
                } else {
                    buf.push('/');
                }
                offset += 1;
            }
            let id = self.get_level_id(offset);
            buf.push_str(id.to_string().as_str());
            match units_required(id) {
                Ok(units) => {
                    offset += units as usize * BITS_PER_UNIT as usize;
                }
                Err(e) => {
                    return write!(f, "ERROR {}", e);
                }
            }
        }
        write!(f, "{}", buf)
    }
}

impl core::fmt::Debug for DLN {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{} [{}]", self.debug_bits(), self.pos+1)
    }
}

impl DLN {
    pub fn default() -> Self {
        DLN {
            bits: Vec::with_capacity(7),
            pos: 0,
        }
    }

    pub fn one() -> Self {
        DLN {
            bits: [255_u8].to_vec(),
            pos: 0
        }
    }

    pub fn new(bits: Vec<u8>, number_of_bits: usize) -> Self {
        let remainder = number_of_bits % 8;
        let mut len = number_of_bits / 8;
        if remainder > 0 {
            len += 1;
        }

        let mut new_bits = bits.split_at(len).0.to_vec();

        if remainder > 0 {
            let last = new_bits.remove(len - 1);
            let mut new_last = 0;
            for i in 0..remainder {
                if last & (1 << (i & 7)) != 0 {
                    new_last |= 1 << i;
                }
            }
            new_bits.push(new_last)
        }

        DLN {
            bits: new_bits,
            pos: number_of_bits - 1,
        }
    }

    pub fn document() -> Self {
        let mut id = DLN::default();
        id.set_current_level_id(0);
        id
    }

    pub fn root() -> Self {
        let mut id = DLN::default();
        id.set_current_level_id(1);
        id
    }

    pub fn level_id(level_id: usize) -> Self {
        let mut id = DLN::default();
        id.add_level_id(level_id, false);
        id
    }

    // generative
    pub fn parent(&self) -> Option<Self> {
        // is document?
        if self.bits.len() == 1 && self.bits.contains(&0) {
            None
        } else {
            let last = self.last_level_offset();
            if last == 0 {
                Some(DLN::document())
            } else {
                Some(DLN::new(self.bits.clone(), last - 1))
            }
        }
    }

    pub fn zero_child(&self) -> Self {
        let mut child = self.clone();
        child.add_level_id(0, false);
        child
    }

    pub fn first_child(&self) -> Self {
        let mut child = self.clone();
        child.add_level_id(1, false);
        child
    }

    pub fn next_sibling(&self) -> Self {
        let mut id = self.clone();
        id.increment_level_id();
        id
    }

    pub fn preceding_sibling(&self) -> Self {
        let mut id = self.clone();
        id.decrement_level_id();
        id
    }

    fn debug_bits(&self) -> String {
        let mut buf = String::with_capacity(self.bits.len()*8);
        for bits in &self.bits {
            for pos in 0..8 {
                let ch = if get_bit_at(bits, pos) {
                    '1'
                } else {
                    '0'
                };
                buf.push(ch);
            }
        }
        buf
    }

    fn level_id_from_string(&mut self, data: &String, is_sub: bool) -> Result<(), String> {
        match data.as_str().parse::<usize>() {
            Ok(level_id) => {
                self.add_level_id(level_id, is_sub);
                Ok(())
            },
            Err(..) => return Err(format!("can't get level id '{}'", data.as_str()))
        }
    }

    fn add_level_id(&mut self, level_id: usize, is_sub: bool) -> Result<(), String> {
        if self.bits.len() != 0 {
            self.set_next_bit(is_sub);
        }
        self.set_current_level_id(level_id)
    }

    fn set_level_id(&mut self, offset: usize, level_id: usize) -> Result<(), String> {
        self.pos = offset;
        if offset == 0 {
            self.bits.clear();
        }
        self.set_current_level_id(level_id)
    }

    fn increment_level_id(&mut self) {
        let last = self.last_field_position();
        let last_id = self.get_level_id(last);
        if last == 0 {
            self.bits.clear();
            self.pos = 0;
        } else {
            self.pos = last - 1;
        }
        self.set_current_level_id(last_id + 1);
    }

    fn decrement_level_id(&mut self) {
        let last = self.last_field_position();
        self.pos = last - 1;
        let mut level_id = self.get_level_id(last);
        // TODO: raise error?
        if level_id != 0 {
            level_id -= 1;
        }
        self.set_current_level_id(level_id);

        // compress to remove unused bits
        let len = self.pos + 1;
        let mut items = len / 8;
        if len % 8 > 0 {
            items += 1;
        }

        if items < self.bits.len() {
            self.bits.truncate(items);
        }
    }

    pub(crate) fn get_level_id(&self, offset: usize) -> usize {
        let units = self.units_used(offset);

        let mut start_bit = offset + units as usize;
        let number_of_bits = bits_offset(units);

        let mut id = 0;
        for i in (0..number_of_bits).rev() {
            if (self.bits[start_bit >> SHIFT] & bit(start_bit)) != 0 {
                id |= 1 << i;
            }
            start_bit += 1;
        }
        if units > 1 {
            id += PER_COMPONENT_SIZE[(units - 2) as usize];
        }
        id
    }

    pub(crate) fn count_levels(&self) -> usize {
        let mut count = 0;
        let mut pos = 0;
        while self.pos >= pos {
            if pos > 0 {
                if self.bits[pos >> SHIFT] & (1 << (pos & 7)) == LEVEL_UP {
                    count += 1;
                }
                pos += 1;
            }
            let units = self.units_used(pos);
            pos += units as usize * BITS_PER_UNIT as usize;
        }

        count
    }

    pub(crate) fn last_level_offset(&self) -> usize {
        let mut pos = 0;
        let mut offset = 0;
        while self.pos >= pos {
            if pos > 0 {
                if self.bits[pos >> SHIFT] & (1 << (pos & 7)) == LEVEL_UP {
                    offset = pos + 1;
                }
                pos += 1;
            }
            let units = self.units_used(pos);
            pos += units as usize * BITS_PER_UNIT as usize;
        }
        offset
    }

    fn last_field_position(&self) -> usize {
        let mut pos = 0;
        let mut offset = 0;
        while self.pos >= pos {
            if pos > 0 {
                pos += 1;
                offset = pos;
            }
            let units = self.units_used(pos);
            pos += units as usize * BITS_PER_UNIT as usize;
        }
        offset
    }

    fn set_next_bit(&mut self, is_sub: bool) {
        if self.bits.len() != 0 {
            self.pos += 1;
        }

        if (self.pos >> SHIFT) >= self.bits.len() {
            self.bits.push(0)
        }

        let pos = self.pos >> SHIFT;
        let bit = bit(self.pos);
        if is_sub {
            self.bits[pos] |= bit;
        } else {
            self.bits[pos] &= !bit;
        }
    }

    fn set_current_level_id(&mut self, level_id: usize) -> Result<(), String> {
        let units = units_required(level_id)?;
        let number_of_bits = bits_offset(units);

        let number = if units > 1 {
            level_id - PER_COMPONENT_SIZE[(units - 2) as usize]
        } else {
            level_id
        };

        for i in 1..units {
            self.set_next_bit(true);
        }
        self.set_next_bit(false);

        for i in (0..number_of_bits).rev() {
            self.set_next_bit(((number >> i) & 1) != 0);
        }

        Ok(())
    }

    fn units_used(&self, offset: usize) -> u8 {
        let mut offset = offset;
        let mut units = 1;
        while self.bits[offset >> SHIFT] & bit(offset) != LEVEL_UP {
            units += 1;
            offset += 1;
        }
        units
    }

    pub(crate) fn start_with(&self, other: &DLN) -> bool {
        if self.pos >= other.pos {
            let number_of_bytes = other.pos / 8;
            for i in 0..number_of_bytes {
                if self.bits[i] != other.bits[i] {
                    return false;
                }
            }
            let remainder = other.pos % 8;
            self.bits[number_of_bytes] & BIT_MASK[remainder] == other.bits[number_of_bytes] & BIT_MASK[remainder]
        } else {
            false
        }
    }
}

impl Eq for DLN {

}

impl PartialEq<Self> for DLN {
    fn eq(&self, other: &Self) -> bool {
        self.cmp(other) == Ordering::Equal
    }
}

impl PartialOrd<Self> for DLN {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for DLN {
    fn cmp(&self, other: &Self) -> Ordering {
        for (l, r) in self.bits.iter().zip(other.bits.iter()) {
            if l != r {
                for n in 0..8 {
                    let lb = l & (1 << n) != 0;
                    let rb = r & (1 << n) != 0;
                    if lb != rb {
                        return if lb { Ordering::Greater } else { Ordering::Less }
                    }
                }
            }
        }
        self.bits.len().cmp(&other.bits.len())
    }
}

impl TryFrom<&str> for DLN {
    type Error = String;

    fn try_from(data: &str) -> Result<Self, String> {
        let mut id = DLN::default();
        let mut buf = String::with_capacity(17);
        let mut is_sub = false;

        for ch in data.chars() {
            if ch == '.' || ch == '/' {
                id.level_id_from_string(&buf, is_sub)?;
                buf.clear();
                is_sub = ch == '/';
            } else if ch >= '0' && ch <= '9' {
                buf.push(ch)
            } else {
                return Err(format!("unexpected char '{}' at '{}'", ch, data));
            }
        }
        if buf.len() > 0 {
            id.level_id_from_string(&buf, is_sub)?;
        }
        Ok(id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compare() {
        let n1 = create("121310.203340");
        let n2 = create("203340.121310");

        assert_eq!(n1.cmp(&n2), Ordering::Less);
    }

    #[test]
    fn get_level() {
        let n1234 = create("1.2/3.4");

        assert_eq!(1, n1234.get_level_id(0));
        assert_eq!(2, n1234.get_level_id(1));
        assert_eq!(3, n1234.get_level_id(2));
        assert_eq!(4, n1234.get_level_id(3));
    }

    #[test]
    fn after() {
        let n1 = create("1");
        let n11 = n1.first_child();

        assert_eq!(true, n11.start_with(&n1));
        assert_eq!(true, n11.start_with(&n1));
    }

    #[test]
    fn experiment() {
        // 1 = 00010000 [4]
        let n1 = create("1");
        // 2 = 00100000 [4]
        let n2 = create("2");

        assert_eq!(n1.cmp(&n2), Ordering::Less);

        // 3 = 00110000 [4]
        let n3 = create("3");
        // 6 = 01100000 [4]
        let n6 = create("6");
        // 7 = 10000000 [8]
        let n7 = create("7");
        // 16 = 10001001 [8]
        let n16 = create("16");
        // 70 = 10111111 [8]
        let n70 = create("70");
        // 71 = 1100000000000000 [12]
        let n71 = create("71");

        assert_eq!(n70.cmp(&n71), Ordering::Less);
        assert_eq!(n70.cmp(&n71), Ordering::Less);

        // 1.1 = 0001000010000000 [9]
        let n11 = create("1.1");
        // 1.2 = 0001000100000000 [9]
        let n12 = create("1.2");
        // 1.2/1 = 0001000101000100 [14]
        let n121 = create("1.2/1");
        // 1.2/2 = 0001000101001000 [14]
        let n122 = create("1.2/2");
        // 1.2/2.1 = 000100010100100000100000 [19]
        let n1221 = create("1.2/2.1");

        assert_eq!(0, n1.count_levels());
        assert_eq!(1, n11.count_levels());
        assert_eq!(1, n121.count_levels());
        assert_eq!(2, n1221.count_levels());

        assert_eq!(n122.cmp(&n1221), Ordering::Less);
        assert_eq!(n1.cmp(&n1221), Ordering::Less);

        assert_eq!(1, n1221.get_level_id(0));
        assert_eq!(2, n1221.get_level_id(1 * SHIFT as usize));
        assert_eq!(2, n1221.get_level_id(2 * SHIFT as usize));
        assert_eq!(1, n1221.get_level_id(3 * SHIFT as usize));
    }

    const ITEMS_TO_TEST: usize = 1_000_000;

    struct TestItem {
        num: usize,
        dln: DLN,
    }

    // #[bench]
    // fn creation(b: &mut Bencher) {
    //     b.iter(|| {
    //         let mut dln = DLN::default();
    //         let num = rand::thread_rng().next_u32() as usize;
    //         dln.set_level_id(0, num).unwrap();
    //     })
    // }

    #[test]
    fn random_ordering() {
        let mut rng = rand::thread_rng();

        let mut items = Vec::with_capacity(ITEMS_TO_TEST);

        let start = Local::now();

        for i in 0..ITEMS_TO_TEST {
            let mut dln = DLN::default();
            let num = rng.next_u32() as usize;
            dln.set_level_id(0, num).unwrap();
            items.push(TestItem { num, dln })
        }

        println!("created in {}", Local::now() - start);

        let start = Local::now();

        items.sort_by(|l, r| l.dln.cmp(&r.dln));

        println!("sorted in {}", Local::now() - start);

        let ge = [Ordering::Greater, Ordering::Equal];

        let mut last: Option<TestItem> = None;
        for item in items {
            // println!("{} {:?}", item.num, item.dln);
            assert_eq!(item.dln.get_level_id(0), item.num);
            if let Some(last) = &last {
                assert_eq!(ge.contains(&item.dln.cmp(&last.dln)), true);
                assert_eq!(ge.contains(&item.num.cmp(&last.num)), true, "{} vs {} .. {} vs {}", item.num, last.num, item.dln.debug_bits(), last.dln.debug_bits());
            }
            last = Some(item)
        }
    }

    fn create(data: &str) -> DLN {
        let id = DLN::try_from(data).unwrap();
        println!("{} vs {:?}", data, id);
        assert_eq!(id.to_string(), data);
        id
    }
}