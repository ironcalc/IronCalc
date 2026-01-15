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

#[cfg(test)]
mod tests {
    use crate::locale::get_locale;

    use super::*;

    #[test]
    fn numeric_progression_basic() {
        let p = NumericProgression {
            last: 3.0,
            step: 2.0,
        };
        assert_eq!(p.next(0), 5.0);
        assert_eq!(p.next(1), 7.0);
    }

    #[test]
    fn suffixed_progression_basic() {
        let locale = get_locale("en").unwrap();

        let values = vec!["A1", "A2", "A3"]
            .into_iter()
            .map(String::from)
            .collect::<Vec<_>>();

        let prog = (SuffixedNumberDetector { locale }).detect(&values).unwrap();
        assert_eq!(prog.next(0), "A4");
        assert_eq!(prog.next(1), "A5");
    }

    #[test]
    fn numeric_detector_rejects_non_progression() {
        let locale = get_locale("en").unwrap();

        let values = vec!["1", "2", "4"]
            .into_iter()
            .map(String::from)
            .collect::<Vec<_>>();
        assert!((NumericProgressionDetector { locale })
            .detect(&values)
            .is_none());
    }

    // New tests below

    #[test]
    fn test_numeric_progression_detector() {
        let locale = get_locale("en").unwrap();
        let detector = NumericProgressionDetector { locale };

        // Simple integers
        let values = vec!["1".to_string(), "2".to_string(), "3".to_string()];
        let progression = detector.detect(&values).unwrap();
        assert_eq!(progression.next(0), "4");
        assert_eq!(progression.next(1), "5");

        // Floating point numbers
        let values = vec!["1.5".to_string(), "2.0".to_string(), "2.5".to_string()];
        let progression = detector.detect(&values).unwrap();
        assert_eq!(progression.next(0), "3");
        assert_eq!(progression.next(1), "3.5");

        // Negative step
        let values = vec!["10".to_string(), "8".to_string(), "6".to_string()];
        let progression = detector.detect(&values).unwrap();
        assert_eq!(progression.next(0), "4");
        assert_eq!(progression.next(1), "2");

        // Negative numbers
        let values = vec!["-10".to_string(), "-8".to_string(), "-6".to_string()];
        let progression = detector.detect(&values).unwrap();
        assert_eq!(progression.next(0), "-4");
        assert_eq!(progression.next(1), "-2");

        // Single value is not a progression
        let values = vec!["1".to_string()];
        assert!(detector.detect(&values).is_none());

        // No progression
        let values = vec!["1".to_string(), "3".to_string(), "4".to_string()];
        assert!(detector.detect(&values).is_none());
    }

    #[test]
    fn test_numeric_progression_detector_locale_de() {
        let locale = get_locale("de").unwrap();
        let detector = NumericProgressionDetector { locale };

        // "de" uses "," as decimal separator and "." as grouping separator
        let values = vec!["1,5".to_string(), "2,0".to_string(), "2,5".to_string()];
        let progression = detector.detect(&values).unwrap();
        // NB: The output format is not localized
        assert_eq!(progression.next(0), "3");
        assert_eq!(progression.next(1), "3.5");

        // With grouping
        let values = vec!["1.000".to_string(), "2.000".to_string(), "3.000".to_string()];
        let progression = detector.detect(&values).unwrap();
        assert_eq!(progression.next(0), "4000");
        assert_eq!(progression.next(1), "5000");

        // Grouping and decimal separator
        let values = vec!["1.000,5".to_string(), "2.000,5".to_string()];
        let progression = detector.detect(&values).unwrap();
        assert!((progression.next(0).parse::<f64>().unwrap() - 3000.5).abs() < 1e-9);
    }

    #[test]
    fn test_numeric_progression_detector_weird_grouping() {
        let locale = get_locale("de").unwrap(); // group: '.', decimal: ','
        let detector = NumericProgressionDetector { locale };

        // The grouping validator is a bit loose and allows this.
        // "1.00" -> 100
        // "2.000" -> 2000
        // step = 1900, last = 2000
        let values = vec!["1.00".to_string(), "2.000".to_string()];
        let progression = detector.detect(&values).unwrap();
        assert_eq!(progression.next(0), "3900");
    }

    #[test]
    fn test_suffixed_progression_detector() {
        let locale = get_locale("en").unwrap();
        let detector = SuffixedNumberDetector { locale };

        // Basic case
        let values = vec!["A1".to_string(), "A2".to_string()];
        let progression = detector.detect(&values).unwrap();
        assert_eq!(progression.next(0), "A3");
        assert_eq!(progression.next(1), "A4");

        // With space
        let values = vec!["Product 1".to_string(), "Product 2".to_string()];
        let progression = detector.detect(&values).unwrap();
        assert_eq!(progression.next(0), "Product 3");

        // Decreasing
        let values = vec!["Q10".to_string(), "Q9".to_string()];
        let progression = detector.detect(&values).unwrap();
        assert_eq!(progression.next(0), "Q8");

        // Different prefixes
        let values = vec!["A1".to_string(), "B2".to_string()];
        assert!(detector.detect(&values).is_none());

        // Suffix is not a number
        let values = vec!["Test-A".to_string(), "Test-B".to_string()];
        assert!(detector.detect(&values).is_none());

        // No numeric suffix
        let values = vec!["Test".to_string(), "Test".to_string()];
        assert!(detector.detect(&values).is_none());
    }

    #[test]
    fn test_suffixed_progression_float_like_suffix() {
        let locale = get_locale("en").unwrap();
        let detector = SuffixedNumberDetector { locale };

        // The suffix detector does not support float numbers, it will find the last group of digits
        // "V1.0" -> prefix "V1.", suffix "0"
        // "V1.5" -> prefix "V1.", suffix "5"
        // It detects a numeric progression on "0", "5" (step 5)
        let values = vec!["V1.0".to_string(), "V1.5".to_string()];
        let progression = detector.detect(&values).unwrap();
        assert_eq!(progression.next(0), "V1.10");
    }

    #[test]
    fn test_date_progression_detector_en() {
        let locale = get_locale("en").unwrap();
        let detector = DateProgressionDetector { locale };

        // Day names
        let values = vec!["Monday".to_string(), "Tuesday".to_string()];
        let progression = detector.detect(&values).unwrap();
        assert_eq!(progression.next(0), "Wednesday");

        // Day names short
        let values = vec!["Mon".to_string(), "Tue".to_string()];
        let progression = detector.detect(&values).unwrap();
        assert_eq!(progression.next(0), "Wed");

        // Month names
        let values = vec!["January".to_string(), "February".to_string()];
        let progression = detector.detect(&values).unwrap();
        assert_eq!(progression.next(0), "March");

        // Month names short
        let values = vec!["Jan".to_string(), "Feb".to_string()];
        let progression = detector.detect(&values).unwrap();
        assert_eq!(progression.next(0), "Mar");

        // Month letters
        let values = vec!["J".to_string(), "F".to_string(), "M".to_string()];
        let progression = detector.detect(&values).unwrap();
        assert_eq!(progression.next(0), "A");
        assert_eq!(progression.next(1), "M");

        // Wrap-around days
        let values = vec!["Saturday".to_string(), "Sunday".to_string()];
        let progression = detector.detect(&values).unwrap();
        assert_eq!(progression.next(0), "Monday");

        // Case-insensitivity
        let values = vec!["saturday".to_string(), "SUNDAY".to_string()];
        let progression = detector.detect(&values).unwrap();
        assert_eq!(progression.next(0), "Monday");

        // Step > 1
        let values = vec!["Jan".to_string(), "Mar".to_string()];
        let progression = detector.detect(&values).unwrap();
        assert_eq!(progression.next(0), "May");
    }

    #[test]
    fn test_date_progression_detector_fr() {
        let locale = get_locale("fr").unwrap();
        let detector = DateProgressionDetector { locale };

        // Day names
        let values = vec!["lundi".to_string(), "mardi".to_string()];
        let progression = detector.detect(&values).unwrap();
        assert_eq!(progression.next(0), "mercredi");

        // Month names
        let values = vec!["janvier".to_string(), "février".to_string()];
        let progression = detector.detect(&values).unwrap();
        assert_eq!(progression.next(0), "mars");
    }

    #[test]
    fn test_date_progression_detector_es() {
        let locale = get_locale("es").unwrap();
        let detector = DateProgressionDetector { locale };

        // Day names
        let values = vec!["lunes".to_string(), "martes".to_string()];
        let progression = detector.detect(&values).unwrap();
        assert_eq!(progression.next(0), "miércoles");

        // Month names
        let values = vec!["enero".to_string(), "febrero".to_string()];
        let progression = detector.detect(&values).unwrap();
        assert_eq!(progression.next(0), "marzo");
    }

    #[test]
    fn test_detect_progression() {
        let locale = get_locale("en").unwrap();

        // Should detect numeric progression
        let values1 = vec!["1".to_string(), "3".to_string()];
        let p1 = detect_progression(&values1, locale).unwrap();
        assert_eq!(p1.next(0), "5");

        // Should detect suffixed number progression
        let values2 = vec!["X10".to_string(), "X20".to_string()];
        let p2 = detect_progression(&values2, locale).unwrap();
        assert_eq!(p2.next(0), "X30");

        // Should detect date progression
        let values3 = vec!["Mar".to_string(), "Apr".to_string()];
        let p3 = detect_progression(&values3, locale).unwrap();
        assert_eq!(p3.next(0), "May");

        // Should return None for no progression
        let values4 = vec!["1".to_string(), "A".to_string(), "foo".to_string()];
        assert!(detect_progression(&values4, locale).is_none());

        // Numeric should have priority over suffixed
        let values5 = vec!["1".to_string(), "2".to_string()];
        let p5 = detect_progression(&values5, locale).unwrap();
        match p5 {
            Progression::Numeric(_) => {} // Correct
            _ => panic!("Should be numeric progression"),
        }
        assert_eq!(p5.next(0), "3");
    }
}