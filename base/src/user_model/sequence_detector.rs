use crate::locale::Locale;

pub(crate) struct NumericProgression {
    last: f64,
    step: f64,
}
impl NumericProgression {
    fn next(&self, i: usize) -> f64 {
        self.last + self.step * (i as f64 + 1.0)
    }
}

pub(crate) struct DateProgression {
    numeric_progression: NumericProgression,
    dates: Vec<String>,
}

impl DateProgression {
    fn next(&self, i: usize) -> String {
        let num_next_index = self.numeric_progression.next(i);
        let months_len = self.dates.len() as f64;
        let next_index = (num_next_index % months_len + months_len) % months_len;
        self.dates[next_index as usize].clone()
    }
}

pub(crate) enum Progression {
    Numeric(NumericProgression),
    SuffixedNumber {
        progression: NumericProgression,
        prefix: String,
    },
    Date(DateProgression),
}
impl Progression {
    pub(crate) fn next(&self, i: usize) -> String {
        match self {
            Progression::Numeric(num_prog) => NumericProgression::next(num_prog, i).to_string(),
            Progression::SuffixedNumber {
                progression,
                prefix,
            } => format!("{}{}", prefix, progression.next(i)),
            Progression::Date(date_prog) => DateProgression::next(date_prog, i),
        }
    }
}

trait SequenceDetector {
    fn detect(&self, values: &[String]) -> Option<Progression>;
}

struct NumericProgressionDetector<'a> {
    locale: &'a Locale,
}

impl<'a> NumericProgressionDetector<'a> {
    fn validate_group(part: &str, max_len: usize) -> Result<(), ()> {
        (!part.is_empty() && part.chars().all(|c| c.is_ascii_digit()) && part.len() <= max_len)
            .then_some(())
            .ok_or(())
    }

    fn validate_grouping(&self, value: &str, primary: usize, secondary: usize) -> Result<(), ()> {
        let numbers = &self.locale.numbers;
        let decimal_sep = numbers.symbols.decimal.chars().next().unwrap_or('.');
        let group_sep = numbers.symbols.group.chars().next().unwrap_or(',');

        if value.chars().filter(|&c| c == decimal_sep).count() > 1 {
            return Err(());
        }

        let mut groups = value
            .split_once(decimal_sep)
            .map_or(value, |(int, _)| int)
            .split(group_sep)
            .peekable();

        while let Some(group) = groups.next() {
            let max_len = if groups.peek().is_some() {
                secondary
            } else {
                primary
            };
            Self::validate_group(group, max_len)?;
        }

        Ok(())
    }
}

impl SequenceDetector for NumericProgressionDetector<'_> {
    fn detect(&self, values: &[String]) -> Option<Progression> {
        let numbers = &self.locale.numbers;

        let decimal_sep = numbers.symbols.decimal.chars().next().unwrap_or('.');
        let group_sep = numbers.symbols.group.chars().next().unwrap_or(',');
        let decimal_format = &numbers.decimal_formats.standard;

        let groups_len = decimal_format
            .split_once(decimal_sep)
            .map_or(decimal_format.as_str(), |(int, _)| int)
            .split(group_sep)
            .map(|group| group.len())
            .collect::<Vec<_>>();

        let primary = groups_len.last().unwrap_or(&3);
        let secondary = if groups_len.len() > 2 {
            groups_len
                .get(groups_len.len().saturating_sub(2)) // penultimate
                .unwrap_or(&3)
        } else {
            primary
        };

        values
            .iter()
            .map(|num| {
                self.validate_grouping(num, *primary, *secondary)?;

                num.chars()
                    .filter(|&c| c != group_sep)
                    .map(|c| if c == decimal_sep { '.' } else { c })
                    .collect::<String>()
                    .parse::<f64>()
                    .map_err(|_| ())
            })
            .collect::<Result<Vec<_>, _>>()
            .ok()
            .filter(|nums| nums.len() >= 2)
            .and_then(|nums| {
                let step = nums[1] - nums[0];
                if step.abs() < 1e-9 {
                    return None;
                }

                let is_progression = nums.windows(2).all(|w| (w[1] - w[0] - step).abs() < 1e-9);
                if !is_progression {
                    return None;
                }

                let last = nums[nums.len() - 1];

                Some(Progression::Numeric(NumericProgression { last, step }))
            })
    }
}

struct SuffixedNumberDetector<'a> {
    locale: &'a Locale,
}

impl SuffixedNumberDetector<'_> {
    fn suffix_index(value: &str) -> usize {
        let mut rev = String::new();

        let potential_numeric_suffixes = value
            .chars()
            .rev()
            .map_while(|x| {
                rev.push(x);
                rev.parse::<i64>().ok()
            })
            .collect::<Vec<_>>();

        if value.len() == potential_numeric_suffixes.len() {
            0
        } else {
            potential_numeric_suffixes.len()
        }
    }
}

impl SequenceDetector for SuffixedNumberDetector<'_> {
    fn detect(&self, values: &[String]) -> Option<Progression> {
        if values.len() < 2 {
            return None;
        }
        let value0 = &values[0];

        let suffix_indexes: Vec<_> = values.iter().map(|v| Self::suffix_index(v)).collect();

        let all_have_suffixes = suffix_indexes.iter().all(|i| *i != 0);
        if !all_have_suffixes {
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

        let all_have_same_prefix = prefixes.iter().all(|prefix| prefix.eq(prefix0));
        if !all_have_same_prefix {
            return None;
        }

        if let Some(Progression::Numeric(numeric_progression_from_suffixes)) =
            (NumericProgressionDetector {
                locale: self.locale,
            })
            .detect(&suffixes)
        {
            return Some(Progression::SuffixedNumber {
                progression: numeric_progression_from_suffixes,
                prefix: prefix0.to_string(),
            });
        }

        None
    }
}

struct DateProgressionDetector<'a> {
    locale: &'a Locale,
}

impl<'a> DateProgressionDetector<'a> {
    fn find_progression(&self, values: &[String], dates: &[String]) -> Option<Progression> {
        let indexes = values
            .iter()
            .map(|value| {
                dates
                    .iter()
                    .position(|date| date.eq_ignore_ascii_case(value))
                    .map(|idx| idx.to_string())
            })
            .collect::<Option<Vec<_>>>();

        if let Some(indices) = indexes {
            if let Some(Progression::Numeric(numeric_progression)) = (NumericProgressionDetector {
                locale: self.locale,
            })
            .detect(&indices)
            {
                let date_progression = DateProgression {
                    numeric_progression,
                    dates: dates.to_vec(),
                };
                return Some(Progression::Date(date_progression));
            }
        }
        None
    }
}

impl<'a> SequenceDetector for DateProgressionDetector<'a> {
    fn detect(&self, values: &[String]) -> Option<Progression> {
        if values.len() < 2 {
            return None;
        }

        let dates = &self.locale.dates;

        [
            &dates.day_names,
            &dates.day_names_short,
            &dates.months,
            &dates.months_short,
            &dates.months_letter,
        ]
        .iter()
        .find_map(|&names_vec| {
            (Self {
                locale: self.locale,
            })
            .find_progression(values, names_vec)
        })
    }
}

pub(crate) fn detect_progression(values: &[String], locale: &Locale) -> Option<Progression> {
    if let Some(progression) = (NumericProgressionDetector { locale }).detect(values) {
        return Some(progression);
    }
    if let Some(progression) = (SuffixedNumberDetector { locale }).detect(values) {
        return Some(progression);
    }
    if let Some(progression) = (DateProgressionDetector { locale }).detect(values) {
        return Some(progression);
    }
    None
}
