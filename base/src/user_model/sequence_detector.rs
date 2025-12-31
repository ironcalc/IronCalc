pub(crate) struct NumericProgression {
    last: f64,
    step: f64,
}
impl NumericProgression {
    fn next(&self, i: usize) -> String {
        (self.last + self.step * (i as f64 + 1.0)).to_string()
    }
}

pub(crate) enum Progression {
    Numeric(NumericProgression),
    SuffixedNumber {
        progression: NumericProgression,
        prefix: String,
    },
}
impl Progression {
    pub(crate) fn next(&self, i: usize) -> String {
        match self {
            Progression::Numeric(num_prog) => NumericProgression::next(num_prog, i),
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

struct NumericProgressionDetector;

impl SequenceDetector for NumericProgressionDetector {
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
                if !is_progression {
                    return None;
                }

                let last = numbers[numbers.len() - 1];

                Some(Progression::Numeric(NumericProgression { last, step }))
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

        let (prefixes, suffixes): (Vec<String>, Vec<String>) = values
            .iter()
            .zip(suffix_indexes.iter())
            .map(|(value, &suffix_len)| {
                let suffix_start = value.len() - suffix_len;
                (
                    value[..suffix_start].to_string(), // prefix
                    value[suffix_start..].to_string(), // suffix
                )
            })
            .unzip();

        let prefix0 = &value0[..value0.len() - suffix_indexes[0]];

        let is_contain_prefix = prefixes.iter().all(|prefix| prefix.eq(prefix0));
        if !is_contain_prefix {
            return None;
        }

        if let Some(Progression::Numeric(c)) = NumericProgressionDetector.detect(&suffixes) {
            return Some(Progression::SuffixedNumber {
                progression: c,
                prefix: prefix0.to_string(),
            });
        }

        None
    }
}

pub(crate) fn detect_progression(values: &[String]) -> Option<Progression> {
    if let Some(progression) = NumericProgressionDetector.detect(values) {
        return Some(progression);
    }
    if let Some(progression) = SuffixedNumberDetector.detect(values) {
        return Some(progression);
    }
    None
}
