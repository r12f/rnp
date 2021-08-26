use crate::PortRangeList;
use contracts::requires;

pub struct PingPortPicker {
    remaining_ping_count: Option<u32>,

    port_ranges: PortRangeList,
    next_port: u16,
    next_port_range_index: usize,
}

impl PingPortPicker {
    #[allow(unreachable_code)]
    #[requires(port_ranges.ranges.len() > 0)]
    #[requires(port_ranges.ranges.iter().filter(|r| r.start() == &0 || r.end() == &0 || r.start() > r.end()).count() == 0)]
    pub fn new(ping_count: Option<u32>, mut port_ranges: PortRangeList, skip_port_count: u32) -> PingPortPicker {
        port_ranges.ranges.sort_by(|a, b| a.start().cmp(b.start()));

        let next_port = *port_ranges.ranges[0].start();

        let mut port_picker = PingPortPicker { remaining_ping_count: ping_count, port_ranges, next_port, next_port_range_index: 0 };

        for _ in 0..skip_port_count {
            port_picker.next();
        }

        return port_picker;
    }

    fn fetch_next_available_port(&mut self) -> Option<u16> {
        match self.remaining_ping_count {
            Some(remaining_ping_count) if remaining_ping_count == 0 => return None,
            Some(remaining_ping_count) => self.remaining_ping_count = Some(remaining_ping_count - 1),
            None => (),
        }

        return Some(self.fetch_next_available_port_from_port_ranges());
    }

    fn fetch_next_available_port_from_port_ranges(&mut self) -> u16 {
        let port = self.next_port;
        self.next_port = if self.next_port >= *(self.port_ranges.ranges[self.next_port_range_index].end()) {
            self.next_port_range_index += 1;
            if self.next_port_range_index >= self.port_ranges.ranges.len() {
                self.next_port_range_index = 0;
            }

            *self.port_ranges.ranges[self.next_port_range_index].start()
        } else {
            self.next_port + 1
        };

        return port;
    }
}

impl Iterator for PingPortPicker {
    type Item = u16;

    fn next(&mut self) -> Option<Self::Item> {
        return self.fetch_next_available_port();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ping_port_picker_should_work_with_port_range_1() {
        assert_eq!(vec![1024, 1024, 1024], PingPortPicker::new(Some(3), PortRangeList { ranges: vec![(1024..=1024)] }, 0).collect::<Vec<u16>>());
    }

    #[test]
    fn ping_port_picker_should_work_with_limited_ping_count() {
        assert_eq!(vec![1024, 1025], PingPortPicker::new(Some(2), PortRangeList { ranges: vec![(1024..=1027)] }, 0).collect::<Vec<u16>>());
    }

    #[test]
    fn ping_port_picker_should_work_with_ping_count_larger_than_range() {
        assert_eq!(
            vec![1024, 1025, 1026, 1027, 1024, 1025],
            PingPortPicker::new(Some(6), PortRangeList { ranges: vec![(1024..=1027)] }, 0).collect::<Vec<u16>>()
        );
    }

    #[test]
    #[should_panic]
    fn ping_port_picker_should_panic_on_zero_min_port() {
        PingPortPicker::new(Some(3), PortRangeList { ranges: vec![(0..=1024)] }, 0);
    }

    #[test]
    #[should_panic]
    fn ping_port_picker_should_panic_on_zero_max_port() {
        PingPortPicker::new(Some(3), PortRangeList { ranges: vec![(1024..=0)] }, 0);
    }

    #[test]
    #[should_panic]
    fn ping_port_picker_should_panic_when_min_port_is_larger_than_max_port() {
        PingPortPicker::new(Some(3), PortRangeList { ranges: vec![(1028..=1024)] }, 0);
    }

    #[test]
    fn ping_port_picker_should_work_with_port_list() {
        assert_eq!(
            vec![1024, 1025, 1026, 1024, 1025],
            PingPortPicker::new(Some(5), PortRangeList { ranges: vec![(1024..=1024), (1025..=1025), (1026..=1026)] }, 0).collect::<Vec<u16>>()
        );
    }

    #[test]
    #[should_panic]
    fn ping_port_picker_should_panic_when_port_list_is_empty() {
        PingPortPicker::new(Some(3), PortRangeList { ranges: vec![] }, 0);
    }
}
