use contracts::requires;

pub struct PingPortPicker {
    min_port: u16,
    max_port: u16,
    next_port: u16,
    remaining_ping_count: Option<u32>,
}

impl PingPortPicker {
    #[requires(min_port > 0)]
    #[requires(max_port > 0)]
    #[requires(min_port <= max_port)]
    pub fn new(min_port: u16, max_port: u16, ping_count: Option<u32>) -> PingPortPicker {
        return PingPortPicker {
            min_port,
            max_port,
            next_port: min_port,
            remaining_ping_count: ping_count,
        };
    }
}

impl Iterator for PingPortPicker {
    type Item = u16;

    fn next(&mut self) -> Option<Self::Item> {
        match self.remaining_ping_count {
            Some(remaining_ping_count) if remaining_ping_count == 0 => return None,
            Some(remaining_ping_count) => self.remaining_ping_count = Some(remaining_ping_count - 1),
            None => (),
        }

        let port = self.next_port;
        self.next_port = if self.next_port >= self.max_port {
            self.min_port
        } else {
            self.next_port + 1
        };

        return Some(port);
    }
}

#[cfg(test)]
mod tests {
    use crate::ping_port_picker::PingPortPicker;

    #[test]
    fn ping_port_picker_should_work_with_port_range_1() {
        let picker = PingPortPicker::new(1024, 1024, Some(3));
        assert_eq!(vec![1024, 1024, 1024], picker.collect::<Vec<u16>>());
    }

    #[test]
    fn ping_port_picker_should_work_with_limited_ping_count() {
        let picker = PingPortPicker::new(1024, 1027, Some(2));
        assert_eq!(vec![1024, 1025], picker.collect::<Vec<u16>>());
    }

    #[test]
    fn ping_port_picker_should_work_with_ping_count_larger_than_range() {
        let picker = PingPortPicker::new(1024, 1027, Some(6));
        assert_eq!(vec![1024, 1025, 1026, 1027, 1024, 1025], picker.collect::<Vec<u16>>());
    }

    #[test]
    #[should_panic]
    fn ping_port_picker_should_panic_on_zero_min_port() {
        PingPortPicker::new(0, 1024, Some(3));
    }

    #[test]
    #[should_panic]
    fn ping_port_picker_should_panic_on_zero_max_port() {
        PingPortPicker::new(1024, 0, Some(3));
    }

    #[test]
    #[should_panic]
    fn ping_port_picker_should_panic_when_min_port_is_larger_than_max_port() {
        PingPortPicker::new(1028, 1024, Some(3));
    }
}