mod test_condition;
mod test_dates;
mod test_de_examples;
mod test_en_examples;
mod test_fr_examples;
mod test_general;
mod test_parse_formatted_number;
mod test_rounding;
mod test_scientific;
mod test_special_dollar;
mod test_time;
mod test_time_rounding;

use crate::{formatter::format::Formatted, locale::Locale, types::DateSystem};

/// `format::format_number` in the IronCalc date system, for the tests
pub(crate) fn format_number(value: f64, format: &str, locale: &Locale) -> Formatted {
    crate::formatter::format::format_number(value, format, locale, DateSystem::IronCalc)
}

/// `format::parse_formatted_number` in the IronCalc date system, for the tests
pub(crate) fn parse_formatted_number(
    original: &str,
    currencies: &[&str],
    locale: &Locale,
) -> Result<(f64, Option<String>), String> {
    crate::formatter::format::parse_formatted_number(
        original,
        currencies,
        locale,
        DateSystem::IronCalc,
    )
}
