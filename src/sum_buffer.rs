#[derive(Clone, Debug, PartialEq)]
struct ValueWithSum {
    value: f32,
    sum: f32,
}

pub struct SumBuffer {
    buffer: Vec<ValueWithSum>,
    size: usize,

    current: usize,
    prev: usize,

    residue: f32,
}

impl SumBuffer {
    pub fn new(size: usize) -> SumBuffer {
        SumBuffer {
            buffer: vec![
                ValueWithSum {
                    value: 0.0,
                    sum: 0.0,
                };
                size
            ],
            size,

            current: 0,
            prev: 0,

            residue: 0.0,
        }
    }

    #[inline(always)]
    pub fn add(&mut self, current_value: f32) -> f32 {
        self.prev = self.current;
        self.current += 1;
        if self.current >= self.size {
            self.current = 0;
        }

        let last_value = self.buffer[self.current].value;
        let prev_sum = self.buffer[self.prev].sum;

        let current_sum = self.add_with_residue(prev_sum, -last_value);
        let current_sum = self.add_with_residue(current_sum, current_value);

        self.buffer[self.current] = ValueWithSum {
            value: current_value,
            sum: current_sum,
        };

        current_sum
    }

    #[inline(always)]
    fn add_with_residue(&mut self, lhs: f32, rhs: f32) -> f32 {
        let result = lhs + (self.residue + rhs);
        self.residue = (self.residue + rhs) - (result - lhs);

        result
    }
}

#[cfg(test)]
mod tests {
    use super::SumBuffer;

    #[test]
    fn sum() {
        let mut buffer = SumBuffer::new(3);

        assert_eq!(buffer.add(1.0), 1.0);
        assert_eq!(buffer.add(2.0), 1.0 + 2.0);
        assert_eq!(buffer.add(3.0), 1.0 + 2.0 + 3.0);
        assert_eq!(buffer.add(4.0), 2.0 + 3.0 + 4.0);
        assert_eq!(buffer.add(5.0), 3.0 + 4.0 + 5.0);
        assert_eq!(buffer.add(6.0), 4.0 + 5.0 + 6.0);
    }
}
