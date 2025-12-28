pub(crate) enum Progression {
    IntegerProgression {
        last: i64,
        step: i64,
    },
    FloatProgression {
        last: f64,
        step: f64,
        precision: usize,
    },
}

impl Progression {
    pub(crate) fn next(&self, i: usize) -> String {
        match self {
            Progression::IntegerProgression { last, step } => {
                (last + step * (i as i64 + 1)).to_string()
            }
            Progression::FloatProgression {
                last,
                step,
                precision,
            } => {
                format!("{:.precision$}", last + step * (i as f64 + 1.0),)
            }
        }
    }
}

pub(crate) trait SequenceDetector {
    fn detect(&self, values: &[String]) -> Option<Progression>;
}

pub(crate) struct IntegerProgressionDetector;

impl SequenceDetector for IntegerProgressionDetector {
    fn detect(&self, values: &[String]) -> Option<Progression> {
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
                    Some(Progression::IntegerProgression { last, step })
                } else {
                    None
                }
            })
    }
}

pub(crate) struct FloatProgressionDetector;

impl SequenceDetector for FloatProgressionDetector {
    fn detect(&self, values: &[String]) -> Option<Progression> {
        values
            .iter()
            .map(|s| s.parse::<f64>())
            .collect::<Result<Vec<_>, _>>()
            .ok()
            .filter(|nums| nums.len() >= 2)
            .and_then(|numbers| {
                let step = numbers[1] - numbers[0];
                if step.abs() < 1e-9 {
                    return None;
                }
                let is_progression = numbers
                    .windows(2)
                    .all(|w| (w[1] - w[0] - step).abs() < 1e-9);

                if is_progression {
                    let last = numbers[numbers.len() - 1];
                    let precision = values
                        .iter()
                        .map(|s| s.split('.').nth(1).map_or(0, |p| p.len()))
                        .max()
                        .unwrap_or(0);
                    Some(Progression::FloatProgression {
                        last,
                        step,
                        precision,
                    })
                } else {
                    None
                }
            })
    }
}
