use num::One;
use std::fmt;
use std::iter::Sum;
use std::ops::{Add, RangeInclusive, Sub};
use std::str::FromStr;

#[derive(Debug, Clone, PartialEq)]
pub struct RangeListInclusive<Idx> {
    pub ranges: Vec<RangeInclusive<Idx>>,
}

impl<Idx: Copy + PartialOrd<Idx> + Add<Output = Idx> + Sub<Output = Idx> + One + Sum>
    RangeListInclusive<Idx>
{
    pub fn calculate_total_port_count(&self) -> Idx {
        return self
            .ranges
            .iter()
            .map(|r| *r.end() - *r.start() + One::one())
            .sum();
    }
}

impl<Idx: Copy + FromStr> FromStr for RangeListInclusive<Idx> {
    type Err = String;

    fn from_str(input: &str) -> Result<RangeListInclusive<Idx>, Self::Err> {
        let mut parsed_ranges = Vec::new();

        if input.len() == 0 {
            return Ok(RangeListInclusive {
                ranges: parsed_ranges,
            });
        }

        for input_part in input.split(",") {
            let range_parts = input_part.split("-").collect::<Vec<&str>>();
            if range_parts.len() == 1 {
                let port = Idx::from_str(range_parts[0])
                    .map_err(|_| format!("Parse port \"{}\" failed.", range_parts[0]))?;

                parsed_ranges.push(port..=port);
            } else if range_parts.len() == 2 {
                let port_start = Idx::from_str(range_parts[0]).map_err(|_| {
                    format!("Parse port range start \"{}\" failed.", range_parts[0])
                })?;
                let port_end = Idx::from_str(range_parts[1])
                    .map_err(|_| format!("Parse port range end \"{}\" failed.", range_parts[1]))?;

                parsed_ranges.push(port_start..=port_end);
            } else {
                return Err(format!("Invalid port range \"{}\". Each port range should only contain 1 or 2 elements. Examples: 1024, 10000-11000", input_part));
            }
        }

        return Ok(RangeListInclusive {
            ranges: parsed_ranges,
        });
    }
}

impl<Idx: fmt::Display + PartialEq> fmt::Display for RangeListInclusive<Idx> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut is_first_range = true;
        for range in &self.ranges {
            if is_first_range {
                is_first_range = false;
            } else {
                write!(f, ",")?;
            }

            if range.start() == range.end() {
                write!(f, "{}", range.start())?;
            } else {
                write!(f, "{}-{}", range.start(), range.end())?;
            }
        }

        Ok(())
    }
}

pub type PortRangeList = RangeListInclusive<u16>;

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn parsing_range_list_should_work() {
        assert_eq!(RangeListInclusive { ranges: vec![] }, "".parse::<RangeListInclusive<i32>>().unwrap());
        assert_eq!(RangeListInclusive { ranges: vec![(1..=1)] }, "1".parse::<RangeListInclusive<i32>>().unwrap());
        assert_eq!(RangeListInclusive { ranges: vec![(1..=1),(2..=2)] }, "1,2".parse::<RangeListInclusive<i32>>().unwrap());
        assert_eq!(RangeListInclusive { ranges: vec![(1..=2)] }, "1-2".parse::<RangeListInclusive<i32>>().unwrap());
        assert_eq!(RangeListInclusive { ranges: vec![(1..=2),(5..=6)] }, "1-2,5-6".parse::<RangeListInclusive<i32>>().unwrap());
        assert_eq!(RangeListInclusive { ranges: vec![(1..=1),(2..=2),(5..=6)] }, "1,2,5-6".parse::<RangeListInclusive<i32>>().unwrap());
        assert_eq!(RangeListInclusive { ranges: vec![(1..=1),(2..=2),(5..=6),(100..=200)] }, "1,2,5-6,100-200".parse::<RangeListInclusive<i32>>().unwrap());
    }

    #[test]
    fn parsing_invalid_range_list_should_fail() {
        assert!("1-".parse::<RangeListInclusive<i32>>().is_err());
        assert!("-2".parse::<RangeListInclusive<i32>>().is_err());
        assert!("1,2,5-".parse::<RangeListInclusive<i32>>().is_err());
        assert!("-2".parse::<RangeListInclusive<i32>>().is_err());
    }
}
