pub const BUFFER_SIZE: usize = 3;

pub struct Random {
    numbers: [Option<u8>; BUFFER_SIZE],
}

impl Default for Random {
    fn default() -> Self {
        Self {
            numbers: [None; BUFFER_SIZE],
        }
    }
}

impl Random {
    pub fn fill_entropy<T>(&mut self, t: &T, f: fn(&T) -> bool) {
        for slot in &mut self.numbers {
            if slot.is_none() {
                let mut b = 0;
                for _ in 0..8 {
                    b <<= 1;
                    b += f(t) as u8;
                }
                *slot = Some(b);
            }
        }
    }

    pub fn get(&mut self) -> u8 {
        for slot in &mut self.numbers {
            if let Some(num) = slot.take() {
                return num;
            }
        }
        0
    }
}
