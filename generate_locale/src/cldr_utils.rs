use std::{collections::HashMap, sync::OnceLock};

/// Static map from CLDR date patterns to Excel custom date formats.
///
/// CLDR tokens:
/// - EEEE → dddd (weekday name)
/// - MMMM → mmmm (full month)
/// - MMM  → mmm  (short month)
/// - d / dd → d / dd
/// - M / MM → m / mm
/// - y → yyyy (4-digit year)
/// - yy → yy (2-digit year)
///
/// Literal text like `'de'` must be quoted in Excel using double quotes.
static CLDR_TO_EXCEL_DATE_FORMATS: OnceLock<HashMap<&'static str, &'static str>> = OnceLock::new();

/// Look up the Excel date format corresponding to a CLDR pattern.
pub fn cldr_to_excel_date_format(cldr_pattern: &str) -> String {
    println!("Converting CLDR pattern: {}", cldr_pattern);
    CLDR_TO_EXCEL_DATE_FORMATS
        .get_or_init(|| {
            let mut m = HashMap::new();

            // German-style with dots
            m.insert("EEEE, d. MMMM y", "dddd, d. mmmm yyyy");
            m.insert("d. MMMM y", "d. mmmm yyyy");
            m.insert("dd.MM.y", "dd.mm.yyyy");
            m.insert("dd.MM.yy", "dd.mm.yy");

            // "EEEE, d MMMM y" / slashed short date
            m.insert("EEEE, d MMMM y", "dddd, d mmmm yyyy");
            m.insert("d MMMM y", "d mmmm yyyy");
            m.insert("d MMM y", "d mmm yyyy");
            m.insert("dd/MM/y", "dd/mm/yyyy");

            // US-style
            m.insert("EEEE, MMMM d, y", "dddd, mmmm d, yyyy");
            m.insert("MMMM d, y", "mmmm d, yyyy");
            m.insert("MMM d, y", "mmm d, yyyy");
            m.insert("M/d/yy", "m/d/yy");

            // Spanish-style with literal "de"
            m.insert(
                "EEEE, d 'de' MMMM 'de' y",
                "dddd, d \"de\" mmmm \"de\" yyyy",
            );
            m.insert("d 'de' MMMM 'de' y", "d \"de\" mmmm \"de\" yyyy");
            // medium here reuses "d MMM y" → already in map above
            m.insert("d/M/yy", "d/m/yy");

            // Variant without comma before month
            m.insert("EEEE d MMMM y", "dddd d mmmm yyyy");

            // Italian-style with slashes
            m.insert("dd/MM/yy", "dd/mm/yy");

            m
        })
        .get(cldr_pattern)
        .unwrap()
        .to_string()
}
