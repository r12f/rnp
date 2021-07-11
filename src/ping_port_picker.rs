use contracts::requires;

pub struct PingPortPicker {
    remaining_ping_count: Option<u32>,
    remaining_warmup_count: u32,

    // Available port range
    min_port: u16,
    max_port: u16,
    next_port: u16,

    // Available port list
    port_list: Option<Vec<u16>>,
    next_port_index: usize,
}

impl PingPortPicker {
    #[requires(min_port > 0)]
    #[requires(max_port > 0)]
    #[requires(min_port <= max_port)]
    #[requires(port_list.is_some() -> port_list.as_ref().unwrap().len() > 0)]
    pub fn new(
        ping_count: Option<u32>,
        warmup_count: u32,
        min_port: u16,
        max_port: u16,
        port_list: &Option<Vec<u16>>,
    ) -> PingPortPicker {
        return PingPortPicker {
            remaining_ping_count: ping_count,
            remaining_warmup_count: warmup_count,
            min_port,
            max_port,
            next_port: min_port,
            port_list: port_list.clone(),
            next_port_index: 0,
        };
    }

    fn fetch_next_available_port(&mut self) -> Option<(u16, bool)> {
        let mut is_warmup = false;
        if self.remaining_warmup_count > 0 {
            self.remaining_warmup_count -= 1;
            is_warmup = true;
        } else {
            match self.remaining_ping_count {
                Some(remaining_ping_count) if remaining_ping_count == 0 => return None,
                Some(remaining_ping_count) => {
                    self.remaining_ping_count = Some(remaining_ping_count - 1)
                }
                None => (),
            }
        }

        if self.port_list.is_some() {
            return Some((self.fetch_next_available_port_from_port_list(), is_warmup));
        }

        return Some((self.fetch_next_available_port_from_port_range(), is_warmup));
    }

    #[requires(self.port_list.is_some())]
    fn fetch_next_available_port_from_port_list(&mut self) -> u16 {
        let port = self.port_list.as_ref().unwrap()[self.next_port_index];

        self.next_port_index += 1;
        if self.next_port_index >= self.port_list.as_ref().unwrap().len() {
            self.next_port_index = 0;
        }

        return port;
    }

    fn fetch_next_available_port_from_port_range(&mut self) -> u16 {
        let port = self.next_port;
        self.next_port = if self.next_port >= self.max_port {
            self.min_port
        } else {
            self.next_port + 1
        };

        return port;
    }
}

impl Iterator for PingPortPicker {
    type Item = (u16, bool);

    fn next(&mut self) -> Option<Self::Item> {
        return self.fetch_next_available_port();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ping_port_picker_should_work_with_port_range_1() {
        assert_eq!(
            vec![(1024, false), (1024, false), (1024, false)],
            PingPortPicker::new(Some(3), 0, 1024, 1024, &None).collect::<Vec<(u16, bool)>>()
        );

        assert_eq!(
            vec![(1024, true), (1024, false), (1024, false), (1024, false)],
            PingPortPicker::new(Some(3), 1, 1024, 1024, &None).collect::<Vec<(u16, bool)>>()
        );
    }

    #[test]
    fn ping_port_picker_should_work_with_limited_ping_count() {
        assert_eq!(
            vec![(1024, false), (1025, false)],
            PingPortPicker::new(Some(2), 0, 1024, 1027, &None).collect::<Vec<(u16, bool)>>()
        );

        assert_eq!(
            vec![(1024, true), (1025, false), (1026, false)],
            PingPortPicker::new(Some(2), 1, 1024, 1027, &None).collect::<Vec<(u16, bool)>>()
        );
    }

    #[test]
    fn ping_port_picker_should_work_with_ping_count_larger_than_range() {
        assert_eq!(
            vec![
                (1024, false),
                (1025, false),
                (1026, false),
                (1027, false),
                (1024, false),
                (1025, false)
            ],
            PingPortPicker::new(Some(6), 0, 1024, 1027, &None).collect::<Vec<(u16, bool)>>()
        );

        assert_eq!(
            vec![
                (1024, true),
                (1025, true),
                (1026, true),
                (1027, true),
                (1024, true),
                (1025, false),
                (1026, false),
                (1027, false),
                (1024, false),
            ],
            PingPortPicker::new(Some(4), 5, 1024, 1027, &None).collect::<Vec<(u16, bool)>>()
        );
    }

    #[test]
    #[should_panic]
    fn ping_port_picker_should_panic_on_zero_min_port() {
        PingPortPicker::new(Some(3), 0, 0, 1024, &None);
    }

    #[test]
    #[should_panic]
    fn ping_port_picker_should_panic_on_zero_max_port() {
        PingPortPicker::new(Some(3), 0, 1024, 0, &None);
    }

    #[test]
    #[should_panic]
    fn ping_port_picker_should_panic_when_min_port_is_larger_than_max_port() {
        PingPortPicker::new(Some(3), 0, 1028, 1024, &None);
    }

    #[test]
    fn ping_port_picker_should_work_with_port_list() {
        assert_eq!(
            vec![
                (1024, true),
                (1025, false),
                (1026, false),
                (1024, false),
                (1025, false),
                (1026, false)
            ],
            PingPortPicker::new(Some(5), 1, 1024, 1027, &Some(vec![1024, 1025, 1026]))
                .collect::<Vec<(u16, bool)>>()
        );
    }

    #[test]
    #[should_panic]
    fn ping_port_picker_should_panic_when_port_list_is_empty() {
        PingPortPicker::new(Some(3), 0, 1028, 1024, &Some(vec![]));
    }
}