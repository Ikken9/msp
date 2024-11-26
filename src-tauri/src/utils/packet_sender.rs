use rand::Rng;

pub struct PacketSender {
    sent_packets: u32,
    confirmed_packets: u32,
}

impl PacketSender {
    pub fn send_packet(&mut self) -> bool {
        self.sent_packets += 1;
        let rate = self.calculate_error_rate();
        let mut rng = rand::thread_rng();
        let rnd: f64 = rng.gen_range(1..=100) as f64 / 100.0;

        if rate * 100.0 > rnd * 100.0 {
            for _ in 0..=2 {
                let retry_rnd: f64 = rng.gen_range(1..=100) as f64 / 100.0;
                if rate * 100.0 < retry_rnd * 100.0 {
                    self.confirmed_packets += 1;
                    return true
                }
            }
            return false;
        }
        self.confirmed_packets += 1;
        true
    }

    fn calculate_error_rate(&self) -> f64 {
        if self.sent_packets == 0 {
            return 0.0;
        }

        self.confirmed_packets as f64 / self.sent_packets as f64
    }
}