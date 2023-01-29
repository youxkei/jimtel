pub struct DelayBuffer {
    delay: usize,
    buffer: Vec<(f32, f32)>,
    current_index: usize,
}

impl DelayBuffer {
    pub fn new(delay: usize) -> Self {
        Self {
            delay,
            buffer: vec![(0.0, 0.0); delay + 1],
            current_index: 0,
        }
    }

    #[inline(always)]
    pub fn add(&mut self, current_left_value: f32, current_right_value: f32) -> (f32, f32) {
        self.buffer[self.current_index] = (current_left_value, current_right_value);

        self.current_index += 1;
        if self.current_index >= self.buffer.len() {
            self.current_index = 0;
        }

        self.buffer[self.current_index]
    }

    #[inline(always)]
    pub fn set_delay(&mut self, delay: usize) {
        if delay != self.delay {
            self.current_index = 0;
            self.delay = delay;
            self.buffer = vec![(0.0, 0.0); delay + 1];
        }
    }
}

#[cfg(test)]
mod tests {
    use super::DelayBuffer;

    #[test]
    fn no_delay() {
        let mut buffer = DelayBuffer::new(0);

        assert_eq!(buffer.add(1.0, 2.0), (1.0, 2.0));
        assert_eq!(buffer.add(3.0, 4.0), (3.0, 4.0));
    }

    #[test]
    fn some_delay() {
        let mut buffer = DelayBuffer::new(3);

        assert_eq!(buffer.add(1.0, 2.0), (0.0, 0.0));
        assert_eq!(buffer.add(3.0, 4.0), (0.0, 0.0));
        assert_eq!(buffer.add(5.0, 6.0), (0.0, 0.0));
        assert_eq!(buffer.add(7.0, 8.0), (1.0, 2.0));
        assert_eq!(buffer.add(8.0, 9.0), (3.0, 4.0));
        assert_eq!(buffer.add(10.0, 11.0), (5.0, 6.0));
        assert_eq!(buffer.add(12.0, 13.0), (7.0, 8.0));
    }
}
