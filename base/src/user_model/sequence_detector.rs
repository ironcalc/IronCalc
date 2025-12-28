pub(crate) trait SequenceDetector {
    fn detect(&self, values: &[String]) -> Option<ArithmeticProgression>;
}

pub(crate) struct IntegerProgressionDetector;

#[derive(Copy, Clone)]
pub(crate) struct ArithmeticProgression {
    last: i64,
    step: i64,
}

impl ArithmeticProgression {
    pub(crate) fn next(&self, i: usize) -> String {
        (self.last + self.step * (i as i64 + 1)).to_string()
    }
}

impl SequenceDetector for IntegerProgressionDetector {
    fn detect(&self, values: &[String]) -> Option<ArithmeticProgression> {
        values
            .iter()
            .map(|s| s.parse::<i64>())
            .collect::<Result<Vec<_>, _>>()
            .ok()
            .filter(|nums| nums.len() >= 2)
            .and_then(|numbers| {
                let step = numbers[1] - numbers[0];
                if step == 0 {
                    return None;
                }
                let is_progression = numbers.windows(2).all(|w| w[1] - w[0] == step);

                if is_progression {
                    let last = numbers[numbers.len() - 1];
                    Some(ArithmeticProgression { last, step })
                } else {
                    None
                }
            })
    }
}
