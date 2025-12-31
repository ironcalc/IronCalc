pub(crate) enum Progression {
    Integer(IntegerProgression),
    Float {
        last: f64,
        step: f64,
        precision: usize,
    },
    SuffixedNumber {
        progression: IntegerProgression,
        prefix: String,
    },
}

pub(crate) struct IntegerProgression {
    last: i64,
    step: i64,
}
impl IntegerProgression {
    fn next(&self, i: usize) -> String {
        (self.last + self.step * (i as i64 + 1)).to_string()
    }
}

impl Progression {
    pub(crate) fn next(&self, i: usize) -> String {
        match self {
            Progression::Integer(int_p) => IntegerProgression::next(int_p, i),
            Progression::Float {
                last,
                step,
                precision,
            } => {
                format!("{:.precision$}", last + step * (i as f64 + 1.0))
            }
            Progression::SuffixedNumber {
                progression,
                prefix,
            } => format!("{}{}", prefix, progression.next(i)),
        }
    }
}

trait SequenceDetector {
    fn detect(&self, values: &[String]) -> Option<Progression>;
}

struct IntegerProgressionDetector;

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
                    Some(Progression::Integer(IntegerProgression { last, step }))
                } else {
                    None
                }
            })
    }
}

struct FloatProgressionDetector;

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
                    Some(Progression::Float {
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

struct SuffixedNumberDetector;

impl SuffixedNumberDetector {
    fn suffix_index(value: &str) -> usize {
        let mut rev = String::new();

        let a = value
            .chars()
            .rev()
            .map_while(|x| {
                rev.push(x);
                rev.parse::<f64>().ok()
            })
            .collect::<Vec<_>>();

        if value.len() == a.len() {
            0
        } else {
            a.len()
        }
    }
}

impl SequenceDetector for SuffixedNumberDetector {
    fn detect(&self, values: &[String]) -> Option<Progression> {
        if values.len() < 2 {
            return None;
        }
        let value0 = &values[0];

        let suffix_indexes: Vec<_> = values.iter().map(|v| Self::suffix_index(v)).collect();

        let is_contain_suffix = suffix_indexes.iter().all(|i| *i != 0);
        if !is_contain_suffix {
            return None;
        }

        let prefix = &value0[..value0.len() - suffix_indexes[0]];

        let is_contain_prefix = values.iter().all(|value| value.contains(prefix));
        if !is_contain_prefix {
            return None;
        }

        let suffixes = values
            .iter()
            .zip(suffix_indexes.iter())
            .map(|(value, &index)| value[value.len() - index..].to_string())
            .collect::<Vec<_>>();

        if let Some(Progression::Integer(c)) = IntegerProgressionDetector.detect(&suffixes) {
            return Some(Progression::SuffixedNumber {
                progression: c,
                prefix: prefix.to_string(),
            });
        }

        None
    }
}

pub(crate) fn detect_progression(values: &[String]) -> Option<Progression> {
    if let Some(progression) = IntegerProgressionDetector.detect(values) {
        return Some(progression);
    }
    if let Some(progression) = FloatProgressionDetector.detect(values) {
        return Some(progression);
    }
    if let Some(progression) = SuffixedNumberDetector.detect(values) {
        return Some(progression);
    }
    None
}
