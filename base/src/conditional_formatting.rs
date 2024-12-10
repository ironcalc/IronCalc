use std::collections::HashMap;

use crate::expressions::utils::parse_reference_a1;
use crate::formatter::dates::{date_to_serial_number, from_excel_date};
use crate::{
    calc_result::CalcResult,
    cell::CellValue,
    cf_types::{
        CfCellResult, CfDataBar, CfIcon, CfRating, CfRule, CfRuleInput, Cfvo, ColorScaleThreshold,
        ConditionalFormatting, ExtendedStyle, Icon, IconThreshold, PeriodType, TextOperator,
        ValueOperator,
    },
    expressions::types::{CellReferenceIndex, CellReferenceRC},
    types::Dxf,
    Model,
};

use chrono::{Datelike, Duration, Months, NaiveDate};

// ---------------------------------------------------------------------------
// Free helper functions for CF evaluation
// ---------------------------------------------------------------------------

/// Parses a space-separated sqref like "A1:C3 E5" into a list of (row1,col1,row2,col2) tuples.
fn parse_sqref(sqref: &str) -> Vec<(i32, i32, i32, i32)> {
    sqref
        .split_whitespace()
        .filter_map(parse_range_part)
        .collect()
}

fn parse_range_part(s: &str) -> Option<(i32, i32, i32, i32)> {
    let upper = s.to_uppercase();
    let parts: Vec<&str> = upper.splitn(2, ':').collect();
    match parts.len() {
        1 => {
            let r = parse_reference_a1(parts[0])?;
            Some((r.row, r.column, r.row, r.column))
        }
        2 => {
            let r1 = parse_reference_a1(parts[0])?;
            let r2 = parse_reference_a1(parts[1])?;
            Some((r1.row, r1.column, r2.row, r2.column))
        }
        _ => None,
    }
}

/// Interpolates a color along the color scale for a given value.
fn interpolate_color(v: f64, thresholds: &[f64], colors: &[String]) -> String {
    let n = thresholds.len();
    if n == 0 || colors.len() != n {
        return "#000000".to_string();
    }
    if v <= thresholds[0] {
        return colors[0].clone();
    }
    if v >= thresholds[n - 1] {
        return colors[n - 1].clone();
    }
    for i in 0..n - 1 {
        if v >= thresholds[i] && v <= thresholds[i + 1] {
            let span = thresholds[i + 1] - thresholds[i];
            let t = if span.abs() < f64::EPSILON {
                0.0
            } else {
                (v - thresholds[i]) / span
            };
            return lerp_color(&colors[i], &colors[i + 1], t);
        }
    }
    colors[0].clone()
}

fn lerp_color(c1: &str, c2: &str, t: f64) -> String {
    let (r1, g1, b1) = parse_hex_color(c1);
    let (r2, g2, b2) = parse_hex_color(c2);
    let r = (r1 as f64 + (r2 as f64 - r1 as f64) * t).round() as u8;
    let g = (g1 as f64 + (g2 as f64 - g1 as f64) * t).round() as u8;
    let b = (b1 as f64 + (b2 as f64 - b1 as f64) * t).round() as u8;
    format!("#{r:02X}{g:02X}{b:02X}")
}

fn parse_hex_color(s: &str) -> (u8, u8, u8) {
    let s = s.trim_start_matches('#');
    if s.len() == 6 {
        let r = u8::from_str_radix(&s[0..2], 16).unwrap_or(0);
        let g = u8::from_str_radix(&s[2..4], 16).unwrap_or(0);
        let b = u8::from_str_radix(&s[4..6], 16).unwrap_or(0);
        (r, g, b)
    } else {
        (0, 0, 0)
    }
}

/// Returns the 0-indexed icon position for `v` given the ordered thresholds.
/// The icon with index `i` applies when `v >= thresholds[i]`.
fn compute_icon_index(v: f64, thresholds: &[(f64, bool)]) -> u32 {
    let mut idx = 0u32;
    for (i, &(value, is_strict)) in thresholds.iter().enumerate() {
        if v > value || (v == value && is_strict) {
            idx = i as u32;
        }
    }
    idx
}

/// Stable string key for a CellValue, used for duplicate detection.
fn cell_value_key(v: &crate::cell::CellValue) -> Option<String> {
    use crate::cell::CellValue;
    match v {
        CellValue::Number(n) => Some(format!("{n}")),
        CellValue::String(s) => Some(s.to_lowercase()),
        CellValue::Boolean(b) => Some(b.to_string()),
        CellValue::None => None,
    }
}

impl<'a> Model<'a> {
    /// Evaluates all conditional formatting rules for the workbook.
    ///
    /// Iterates every worksheet's CF rules in priority order (lowest priority number,
    /// processed last). The result for each cell is stored in
    /// `cf_cache` and consumed by `get_extended_style_for_cell`.
    pub fn evaluate_conditional_formatting(&mut self) {
        self.cf_cache.clear();
        let sheet_count = self.workbook.worksheets.len();
        for sheet_idx in 0..sheet_count {
            let mut cfs = self.workbook.worksheets[sheet_idx]
                .conditional_formatting
                .clone();
            // Lower priority number = higher priority; process high-priority first so that
            // the first writer into cf_cache wins.
            cfs.sort_by_key(|cf| cf.priority);
            for cf in cfs {
                let ranges = parse_sqref(&cf.range);
                if ranges.is_empty() {
                    continue;
                }
                self.apply_cf_rule(sheet_idx as u32, &cf.cf_rule, &ranges);
            }
        }
    }

    // -----------------------------------------------------------------------
    // CF evaluation helpers
    // -----------------------------------------------------------------------

    fn apply_cf_rule(&mut self, sheet: u32, rule: &CfRule, ranges: &[(i32, i32, i32, i32)]) {
        match rule {
            CfRule::ColorScale { thresholds } => {
                self.apply_cf_color_scale(sheet, thresholds, ranges);
            }
            CfRule::CellIs {
                operator,
                formula,
                formula2,
                dxf_id,
            } => {
                self.apply_cf_cell_is(
                    sheet,
                    operator,
                    formula,
                    formula2.as_deref(),
                    *dxf_id,
                    ranges,
                );
            }
            CfRule::Formula { formula, dxf_id } => {
                self.apply_cf_formula(sheet, formula, *dxf_id, ranges);
            }
            CfRule::DuplicateValues { dxf_id } => {
                self.apply_cf_duplicate_values(sheet, *dxf_id, ranges);
            }
            CfRule::AboveAverage { dxf_id } => {
                self.apply_cf_average(sheet, *dxf_id, true, ranges);
            }
            CfRule::BelowAverage { dxf_id } => {
                self.apply_cf_average(sheet, *dxf_id, false, ranges);
            }
            CfRule::Top10 {
                rank,
                percent,
                dxf_id,
            } => {
                self.apply_cf_top_n(sheet, *rank, *percent, false, *dxf_id, ranges);
            }
            CfRule::Bottom10 {
                rank,
                percent,
                dxf_id,
            } => {
                self.apply_cf_top_n(sheet, *rank, *percent, true, *dxf_id, ranges);
            }
            CfRule::DataBar {
                min,
                max,
                positive_color,
                negative_color,
                is_gradient,
                show_value,
            } => {
                self.apply_cf_data_bar(
                    sheet,
                    min.as_ref(),
                    max.as_ref(),
                    positive_color,
                    negative_color,
                    *is_gradient,
                    *show_value,
                    ranges,
                );
            }
            CfRule::IconSet {
                thresholds,
                show_value,
            } => {
                self.apply_cf_icon_set(sheet, thresholds, *show_value, ranges);
            }
            CfRule::Text {
                operator,
                value,
                dxf_id,
            } => {
                self.apply_cf_text(sheet, operator, value, *dxf_id, ranges);
            }
            CfRule::UniqueValues { dxf_id } => {
                self.apply_cf_unique_values(sheet, *dxf_id, ranges);
            }
            CfRule::TimePeriod {
                time_period,
                dxf_id,
                ..
            } => {
                self.apply_cf_time_period(sheet, time_period, *dxf_id, ranges);
            }
            CfRule::IconRating {
                icon,
                color,
                show_value,
                thresholds,
            } => {
                self.apply_cf_icon_rating(sheet, icon, color, thresholds, *show_value, ranges);
            }
            CfRule::Blanks { dxf_id } => {
                self.apply_cf_blanks(sheet, *dxf_id, false, ranges);
            }
            CfRule::NotBlanks { dxf_id } => {
                self.apply_cf_blanks(sheet, *dxf_id, true, ranges);
            }
            CfRule::Errors { dxf_id } => {
                self.apply_cf_errors(sheet, *dxf_id, false, ranges);
            }
            CfRule::NoErrors { dxf_id } => {
                self.apply_cf_errors(sheet, *dxf_id, true, ranges);
            }
        }
    }

    /// Only inserts into cf_cache if the cell has no entry yet (first-wins, since rules are
    /// processed in ascending priority order).
    fn update_cf_cache(&mut self, sheet: u32, row: i32, col: i32, result: CfCellResult) {
        self.cf_cache
            .entry((sheet, row, col))
            .or_default()
            .push(result);
    }

    /// Collects all numeric values from the given set of ranges on `sheet`.
    fn collect_numeric_values(&self, sheet: u32, ranges: &[(i32, i32, i32, i32)]) -> Vec<f64> {
        let mut values = Vec::new();
        for &(r1, c1, r2, c2) in ranges {
            for row in r1..=r2 {
                for col in c1..=c2 {
                    if let Ok(CellValue::Number(n)) = self.get_cell_value_by_index(sheet, row, col)
                    {
                        values.push(n);
                    }
                }
            }
        }
        values
    }

    /// Resolves a Cfvo threshold to a concrete f64 value.
    fn resolve_cfvo(&mut self, cfvo: &Cfvo, values: &[f64], sheet: u32) -> f64 {
        match cfvo {
            Cfvo::Min => values.iter().cloned().fold(f64::INFINITY, f64::min),
            Cfvo::Max => values.iter().cloned().fold(f64::NEG_INFINITY, f64::max),
            Cfvo::Number(n) => *n,
            Cfvo::Percent(p) => {
                if values.is_empty() {
                    return 0.0;
                }
                let min = values.iter().cloned().fold(f64::INFINITY, f64::min);
                let max = values.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
                let p = *p;
                min + (max - min) * p / 100.0
            }
            Cfvo::Percentile(p) => {
                if values.is_empty() {
                    return 0.0;
                }
                let mut sorted = values.to_vec();
                sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
                let idx = ((p / 100.0) * (sorted.len() - 1) as f64).floor() as usize;
                sorted[idx.min(sorted.len() - 1)]
            }
            Cfvo::Formula(f) => self.evaluate_formula(f, sheet).unwrap_or(0.0),
        }
    }

    fn apply_cf_color_scale(
        &mut self,
        sheet: u32,
        thresholds: &[ColorScaleThreshold],
        ranges: &[(i32, i32, i32, i32)],
    ) {
        if thresholds.len() < 2 {
            return;
        }
        let values = self.collect_numeric_values(sheet, ranges);
        if values.is_empty() {
            return;
        }
        let colors: Vec<String> = thresholds.iter().map(|t| t.color.clone()).collect();
        let mut stops: Vec<f64> = Vec::with_capacity(thresholds.len());
        for t in thresholds {
            stops.push(self.resolve_cfvo(&t.cfvo, &values, sheet));
        }
        // Excel sorts only the threshold values, keeping colors at their original
        // positional indices. A formula cfvo that resolves out-of-order (e.g. $G$13=0
        // appearing after cfvo num=20) must be sorted into place while color[i] still
        // maps to the i-th stop in sorted order.
        stops.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

        for &(r1, c1, r2, c2) in ranges {
            for row in r1..=r2 {
                for col in c1..=c2 {
                    if let Ok(CellValue::Number(v)) = self.get_cell_value_by_index(sheet, row, col)
                    {
                        let color = interpolate_color(v, &stops, &colors);
                        self.update_cf_cache(sheet, row, col, CfCellResult::ColorScale(color));
                    }
                }
            }
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn apply_cf_data_bar(
        &mut self,
        sheet: u32,
        min: Option<&Cfvo>,
        max: Option<&Cfvo>,
        positive_color: &str,
        negative_color: &str,
        is_gradient: bool,
        show_value: bool,
        ranges: &[(i32, i32, i32, i32)],
    ) {
        let values = self.collect_numeric_values(sheet, ranges);
        if values.is_empty() {
            return;
        }
        let min_val = match min {
            Some(cfvo) => self.resolve_cfvo(cfvo, &values, sheet),
            None => values.iter().cloned().fold(0.0_f64, f64::min),
        };
        let max_val = match max {
            Some(cfvo) => self.resolve_cfvo(cfvo, &values, sheet),
            None => values.iter().cloned().fold(0.0_f64, f64::max),
        };
        let span = max_val - min_val;
        if span.abs() < f64::EPSILON {
            return;
        }
        let axis_position = (-min_val / span).clamp(0.0, 1.0);

        for &(r1, c1, r2, c2) in ranges {
            for row in r1..=r2 {
                for col in c1..=c2 {
                    if let Ok(CellValue::Number(v)) = self.get_cell_value_by_index(sheet, row, col)
                    {
                        let proportion = ((v - min_val) / span).clamp(0.0, 1.0);
                        self.update_cf_cache(
                            sheet,
                            row,
                            col,
                            CfCellResult::DataBar {
                                positive_color: positive_color.to_string(),
                                negative_color: negative_color.to_string(),
                                is_gradient,
                                value: proportion,
                                axis_position,
                                show_value,
                            },
                        );
                    }
                }
            }
        }
    }

    fn apply_cf_icon_set(
        &mut self,
        sheet: u32,
        thresholds: &[IconThreshold],
        show_value: bool,
        ranges: &[(i32, i32, i32, i32)],
    ) {
        if thresholds.is_empty() {
            return;
        }
        let values = self.collect_numeric_values(sheet, ranges);
        let stops: Vec<(f64, bool)> = thresholds
            .iter()
            .map(|t| (self.resolve_cfvo(&t.cfvo, &values, sheet), t.is_strict))
            .collect();
        let n = thresholds.len();

        for &(r1, c1, r2, c2) in ranges {
            for row in r1..=r2 {
                for col in c1..=c2 {
                    if let Ok(CellValue::Number(v)) = self.get_cell_value_by_index(sheet, row, col)
                    {
                        let idx = (compute_icon_index(v, &stops) as usize).min(n - 1);
                        self.update_cf_cache(
                            sheet,
                            row,
                            col,
                            CfCellResult::Icon {
                                icon: thresholds[idx].icon.clone(),
                                color: thresholds[idx].color.clone(),
                                show_value,
                            },
                        );
                    }
                }
            }
        }
    }

    fn apply_cf_icon_rating(
        &mut self,
        sheet: u32,
        icon: &Icon,
        color: &str,
        thresholds: &[(Cfvo, bool)],
        show_value: bool,
        ranges: &[(i32, i32, i32, i32)],
    ) {
        if thresholds.is_empty() {
            return;
        }
        // Thresholds are stored highest-value-first. For each threshold a cell value
        // satisfies (iterating lowest-to-highest), the filled-icon count increments by 1.
        // count ∈ [1, max] where max = thresholds.len() + 1.
        let max = (thresholds.len() + 1) as u32;

        let values = self.collect_numeric_values(sheet, ranges);
        // Pre-resolve all Cfvo entries before iterating cells.
        let resolved: Vec<(f64, bool)> = thresholds
            .iter()
            .map(|(cfvo, is_strict)| (self.resolve_cfvo(cfvo, &values, sheet), *is_strict))
            .collect();

        for &(r1, c1, r2, c2) in ranges {
            for row in r1..=r2 {
                for col in c1..=c2 {
                    if let Ok(CellValue::Number(v)) = self.get_cell_value_by_index(sheet, row, col)
                    {
                        // Iterate from the lowest-value threshold to the highest.
                        // is_strict=true → threshold applies when v >= value (gte).
                        // is_strict=false → threshold applies when v > value (gt).
                        let mut count = 1u32;
                        for &(threshold_val, is_strict) in resolved.iter().rev() {
                            let exceeds = if is_strict {
                                v >= threshold_val
                            } else {
                                v > threshold_val
                            };
                            if exceeds {
                                count += 1;
                            }
                        }
                        self.update_cf_cache(
                            sheet,
                            row,
                            col,
                            CfCellResult::Rating {
                                icon: icon.clone(),
                                count: count.min(max),
                                max,
                                color: color.to_string(),
                                show_value,
                            },
                        );
                    }
                }
            }
        }
    }

    fn apply_cf_cell_is(
        &mut self,
        sheet: u32,
        operator: &ValueOperator,
        formula: &str,
        formula2: Option<&str>,
        dxf_id: u32,
        ranges: &[(i32, i32, i32, i32)],
    ) {
        let threshold = match self.evaluate_formula(formula, sheet) {
            Some(v) => v,
            None => return,
        };
        let threshold2 = formula2.and_then(|f| self.evaluate_formula(f, sheet));

        for &(r1, c1, r2, c2) in ranges {
            for row in r1..=r2 {
                for col in c1..=c2 {
                    if let Ok(CellValue::Number(v)) = self.get_cell_value_by_index(sheet, row, col)
                    {
                        let matches = match operator {
                            ValueOperator::Equal => (v - threshold).abs() < f64::EPSILON,
                            ValueOperator::GreaterThan => v > threshold,
                            ValueOperator::GreaterThanOrEqual => v >= threshold,
                            ValueOperator::LessThan => v < threshold,
                            ValueOperator::LessThanOrEqual => v <= threshold,
                            ValueOperator::NotEqual => (v - threshold).abs() >= f64::EPSILON,
                            ValueOperator::Between => {
                                let t2 = threshold2.unwrap_or(threshold);
                                v >= threshold.min(t2) && v <= threshold.max(t2)
                            }
                            ValueOperator::NotBetween => {
                                let t2 = threshold2.unwrap_or(threshold);
                                v < threshold.min(t2) || v > threshold.max(t2)
                            }
                        };
                        if matches {
                            self.update_cf_cache(sheet, row, col, CfCellResult::Dxf(dxf_id));
                        }
                    }
                }
            }
        }
    }

    fn apply_cf_average(
        &mut self,
        sheet: u32,
        dxf_id: u32,
        above: bool,
        ranges: &[(i32, i32, i32, i32)],
    ) {
        let values = self.collect_numeric_values(sheet, ranges);
        if values.is_empty() {
            return;
        }
        let avg = values.iter().sum::<f64>() / values.len() as f64;

        for &(r1, c1, r2, c2) in ranges {
            for row in r1..=r2 {
                for col in c1..=c2 {
                    if let Ok(CellValue::Number(v)) = self.get_cell_value_by_index(sheet, row, col)
                    {
                        if (above && v > avg) || (!above && v < avg) {
                            self.update_cf_cache(sheet, row, col, CfCellResult::Dxf(dxf_id));
                        }
                    }
                }
            }
        }
    }

    fn apply_cf_top_n(
        &mut self,
        sheet: u32,
        rank: u32,
        percent: bool,
        bottom: bool,
        dxf_id: u32,
        ranges: &[(i32, i32, i32, i32)],
    ) {
        let values = self.collect_numeric_values(sheet, ranges);
        if values.is_empty() {
            return;
        }
        let n = if percent {
            ((rank as f64 / 100.0) * values.len() as f64).ceil() as usize
        } else {
            rank as usize
        }
        .max(1);

        let mut sorted = values.clone();
        if bottom {
            sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        } else {
            sorted.sort_by(|a, b| b.partial_cmp(a).unwrap_or(std::cmp::Ordering::Equal));
        }
        let threshold = sorted[n.min(sorted.len()) - 1];

        for &(r1, c1, r2, c2) in ranges {
            for row in r1..=r2 {
                for col in c1..=c2 {
                    if let Ok(CellValue::Number(v)) = self.get_cell_value_by_index(sheet, row, col)
                    {
                        let matches = if bottom {
                            v <= threshold
                        } else {
                            v >= threshold
                        };
                        if matches {
                            self.update_cf_cache(sheet, row, col, CfCellResult::Dxf(dxf_id));
                        }
                    }
                }
            }
        }
    }

    fn apply_cf_text(
        &mut self,
        sheet: u32,
        operator: &TextOperator,
        value: &str,
        dxf_id: u32,
        ranges: &[(i32, i32, i32, i32)],
    ) {
        let search = value.to_lowercase();
        for &(r1, c1, r2, c2) in ranges {
            for row in r1..=r2 {
                for col in c1..=c2 {
                    if let Ok(CellValue::String(s)) = self.get_cell_value_by_index(sheet, row, col)
                    {
                        let cell_lower = s.to_lowercase();
                        let matches = match operator {
                            TextOperator::Contains => cell_lower.contains(search.as_str()),
                            TextOperator::DoesNotContain => !cell_lower.contains(search.as_str()),
                            TextOperator::BeginsWith => cell_lower.starts_with(search.as_str()),
                            TextOperator::EndsWith => cell_lower.ends_with(search.as_str()),
                            TextOperator::Equals => cell_lower == search.as_str(),
                        };
                        if matches {
                            self.update_cf_cache(sheet, row, col, CfCellResult::Dxf(dxf_id));
                        }
                    }
                }
            }
        }
    }

    /// Evaluates a boolean CF formula rule across the given ranges.
    ///
    /// The formula is parsed once using the top-left cell of the first range as the parse
    /// anchor.
    fn apply_cf_formula(
        &mut self,
        sheet: u32,
        formula: &str,
        dxf_id: u32,
        ranges: &[(i32, i32, i32, i32)],
    ) {
        let Some(&(anchor_row, anchor_col, _, _)) = ranges.first() else {
            return;
        };
        let body = formula.trim().strip_prefix('=').unwrap_or(formula.trim());
        if body.is_empty() {
            return;
        }
        let sheet_name = match self.workbook.worksheets.get(sheet as usize) {
            Some(ws) => ws.get_name(),
            None => return,
        };
        let context_rc = CellReferenceRC {
            sheet: sheet_name,
            row: anchor_row,
            column: anchor_col,
        };
        let node = self.parser.parse(body, &context_rc);

        for &(r1, c1, r2, c2) in ranges {
            for row in r1..=r2 {
                for column in c1..=c2 {
                    let ctx = CellReferenceIndex { sheet, row, column };
                    let matches = match self.evaluate_node_in_context(&node, ctx) {
                        CalcResult::Number(n) => n != 0.0,
                        CalcResult::Boolean(b) => b,
                        _ => false,
                    };
                    if matches {
                        self.update_cf_cache(sheet, row, column, CfCellResult::Dxf(dxf_id));
                    }
                }
            }
        }
    }

    fn apply_cf_unique_values(&mut self, sheet: u32, dxf_id: u32, ranges: &[(i32, i32, i32, i32)]) {
        let mut counts: HashMap<String, u32> = HashMap::new();
        for &(r1, c1, r2, c2) in ranges {
            for row in r1..=r2 {
                for col in c1..=c2 {
                    if let Ok(v) = self.get_cell_value_by_index(sheet, row, col) {
                        if let Some(k) = cell_value_key(&v) {
                            *counts.entry(k).or_insert(0) += 1;
                        }
                    }
                }
            }
        }
        for &(r1, c1, r2, c2) in ranges {
            for row in r1..=r2 {
                for col in c1..=c2 {
                    if let Ok(v) = self.get_cell_value_by_index(sheet, row, col) {
                        if let Some(k) = cell_value_key(&v) {
                            if counts.get(&k).copied().unwrap_or(0) == 1 {
                                self.update_cf_cache(sheet, row, col, CfCellResult::Dxf(dxf_id));
                            }
                        }
                    }
                }
            }
        }
    }

    fn apply_cf_time_period(
        &mut self,
        sheet: u32,
        period: &PeriodType,
        dxf_id: u32,
        ranges: &[(i32, i32, i32, i32)],
    ) {
        let today_serial = match crate::tz::excel_serial_for_now(&self.tz) {
            Some(s) => s.floor() as i64,
            None => return,
        };
        let today = match from_excel_date(today_serial) {
            Ok(d) => d,
            Err(_) => return,
        };

        let serial_of = |d: NaiveDate| -> f64 {
            date_to_serial_number(d.day(), d.month(), d.year()).unwrap_or(0) as f64
        };

        let range: (f64, f64) = match period {
            PeriodType::Yesterday => {
                let s = today_serial as f64 - 1.0;
                (s, s)
            }
            PeriodType::Today => (today_serial as f64, today_serial as f64),
            PeriodType::Tomorrow => {
                let s = today_serial as f64 + 1.0;
                (s, s)
            }
            PeriodType::Last7Days => (today_serial as f64 - 6.0, today_serial as f64),
            PeriodType::Next7Days => (today_serial as f64, today_serial as f64 + 6.0),
            PeriodType::LastWeek => {
                let dow = today.weekday().num_days_from_monday() as i64;
                let this_mon = today - Duration::days(dow);
                let last_mon = this_mon - Duration::days(7);
                let last_sun = this_mon - Duration::days(1);
                (serial_of(last_mon), serial_of(last_sun))
            }
            PeriodType::ThisWeek => {
                let dow = today.weekday().num_days_from_monday() as i64;
                let this_mon = today - Duration::days(dow);
                let this_sun = this_mon + Duration::days(6);
                (serial_of(this_mon), serial_of(this_sun))
            }
            PeriodType::NextWeek => {
                let dow = today.weekday().num_days_from_monday() as i64;
                let next_mon = today - Duration::days(dow) + Duration::days(7);
                let next_sun = next_mon + Duration::days(6);
                (serial_of(next_mon), serial_of(next_sun))
            }
            PeriodType::LastMonth => {
                let Some(this_month_start) =
                    NaiveDate::from_ymd_opt(today.year(), today.month(), 1)
                else {
                    return;
                };
                let last_month_end = this_month_start - Duration::days(1);
                let Some(last_month_start) =
                    NaiveDate::from_ymd_opt(last_month_end.year(), last_month_end.month(), 1)
                else {
                    return;
                };
                (serial_of(last_month_start), serial_of(last_month_end))
            }
            PeriodType::ThisMonth => {
                let Some(start) = NaiveDate::from_ymd_opt(today.year(), today.month(), 1) else {
                    return;
                };
                let end = start + Months::new(1) - Duration::days(1);
                (serial_of(start), serial_of(end))
            }
            PeriodType::NextMonth => {
                let Some(this_start) = NaiveDate::from_ymd_opt(today.year(), today.month(), 1)
                else {
                    return;
                };
                let next_start = this_start + Months::new(1);
                let next_end = next_start + Months::new(1) - Duration::days(1);
                (serial_of(next_start), serial_of(next_end))
            }
            PeriodType::LastYear => {
                let y = today.year() - 1;
                let Some(start) = NaiveDate::from_ymd_opt(y, 1, 1) else {
                    return;
                };
                let Some(end) = NaiveDate::from_ymd_opt(y, 12, 31) else {
                    return;
                };
                (serial_of(start), serial_of(end))
            }
            PeriodType::ThisYear => {
                let y = today.year();
                let Some(start) = NaiveDate::from_ymd_opt(y, 1, 1) else {
                    return;
                };
                let Some(end) = NaiveDate::from_ymd_opt(y, 12, 31) else {
                    return;
                };
                (serial_of(start), serial_of(end))
            }
            PeriodType::NextYear => {
                let y = today.year() + 1;
                let Some(start) = NaiveDate::from_ymd_opt(y, 1, 1) else {
                    return;
                };
                let Some(end) = NaiveDate::from_ymd_opt(y, 12, 31) else {
                    return;
                };
                (serial_of(start), serial_of(end))
            }
            PeriodType::Between | PeriodType::NotBetween => return,
        };

        for &(r1, c1, r2, c2) in ranges {
            for row in r1..=r2 {
                for col in c1..=c2 {
                    if let Ok(CellValue::Number(v)) = self.get_cell_value_by_index(sheet, row, col)
                    {
                        let day = v.floor();
                        if day >= range.0 && day <= range.1 {
                            self.update_cf_cache(sheet, row, col, CfCellResult::Dxf(dxf_id));
                        }
                    }
                }
            }
        }
    }

    fn apply_cf_duplicate_values(
        &mut self,
        sheet: u32,
        dxf_id: u32,
        ranges: &[(i32, i32, i32, i32)],
    ) {
        let mut counts: HashMap<String, u32> = HashMap::new();
        for &(r1, c1, r2, c2) in ranges {
            for row in r1..=r2 {
                for col in c1..=c2 {
                    if let Ok(v) = self.get_cell_value_by_index(sheet, row, col) {
                        let key = cell_value_key(&v);
                        if let Some(k) = key {
                            *counts.entry(k).or_insert(0) += 1;
                        }
                    }
                }
            }
        }
        for &(r1, c1, r2, c2) in ranges {
            for row in r1..=r2 {
                for col in c1..=c2 {
                    if let Ok(v) = self.get_cell_value_by_index(sheet, row, col) {
                        if let Some(k) = cell_value_key(&v) {
                            if counts.get(&k).copied().unwrap_or(0) > 1 {
                                self.update_cf_cache(sheet, row, col, CfCellResult::Dxf(dxf_id));
                            }
                        }
                    }
                }
            }
        }
    }

    fn apply_cf_blanks(
        &mut self,
        sheet: u32,
        dxf_id: u32,
        invert: bool,
        ranges: &[(i32, i32, i32, i32)],
    ) {
        for &(r1, c1, r2, c2) in ranges {
            for row in r1..=r2 {
                for col in c1..=c2 {
                    let is_blank = matches!(
                        self.get_cell_value_by_index(sheet, row, col),
                        Ok(CellValue::None)
                    );
                    if is_blank != invert {
                        self.update_cf_cache(sheet, row, col, CfCellResult::Dxf(dxf_id));
                    }
                }
            }
        }
    }

    fn apply_cf_errors(
        &mut self,
        sheet: u32,
        dxf_id: u32,
        invert: bool,
        ranges: &[(i32, i32, i32, i32)],
    ) {
        use crate::types::CellType;
        for &(r1, c1, r2, c2) in ranges {
            for row in r1..=r2 {
                for col in c1..=c2 {
                    let is_error = self
                        .workbook
                        .worksheet(sheet)
                        .ok()
                        .and_then(|ws| ws.cell(row, col))
                        .is_some_and(|c| c.get_type() == CellType::ErrorValue);
                    if is_error != invert {
                        self.update_cf_cache(sheet, row, col, CfCellResult::Dxf(dxf_id));
                    }
                }
            }
        }
    }

    // -----------------------------------------------------------------------

    /// Returns the extended cell style for (`sheet`, `row`, `column`), including
    /// any conditional-formatting overlay computed by the last evaluate() call.
    pub fn get_extended_style_for_cell(
        &self,
        sheet: u32,
        row: i32,
        column: i32,
    ) -> Result<ExtendedStyle, String> {
        let base = self.get_style_for_cell(sheet, row, column)?;
        let extended = match self.cf_cache.get(&(sheet, row, column)) {
            Some(c) => c,
            None => {
                return Ok(ExtendedStyle {
                    style: base,
                    icon: None,
                    data_bar: None,
                    rating: None,
                })
            }
        };
        let mut style = base;
        let mut icon = None;
        let mut data_bar = None;
        let mut rating = None;
        let mut color_scale_applied = false;
        for cf_result in extended {
            match cf_result {
                CfCellResult::Dxf(dxf_id) => {
                    if let Some(dxf) = self.workbook.styles.dxfs.get(*dxf_id as usize) {
                        style = dxf.apply_to(&style);
                    }
                }
                // For exclusive types (ColorScale, DataBar, Icon, Rating), the first result in
                // the vec has the highest priority; later entries are skipped.
                CfCellResult::ColorScale(color) => {
                    if !color_scale_applied {
                        style.fill.fg_color = None;
                        style.fill.bg_color = Some(color.clone());
                        style.fill.pattern_type = "solid".to_string();
                        color_scale_applied = true;
                    }
                }
                CfCellResult::DataBar {
                    positive_color,
                    negative_color,
                    is_gradient,
                    value,
                    axis_position,
                    show_value,
                } => {
                    if data_bar.is_none() {
                        data_bar = Some(CfDataBar {
                            positive_color: positive_color.clone(),
                            negative_color: negative_color.clone(),
                            is_gradient: *is_gradient,
                            value: *value,
                            axis_position: *axis_position,
                            show_value: *show_value,
                        });
                    }
                }
                CfCellResult::Icon {
                    icon: cf_icon,
                    color,
                    show_value,
                } => {
                    if icon.is_none() {
                        icon = Some(CfIcon {
                            icon: cf_icon.clone(),
                            color: color.clone(),
                            show_value: *show_value,
                        });
                    }
                }
                CfCellResult::Rating {
                    icon: cf_icon,
                    count,
                    max,
                    color,
                    show_value,
                } => {
                    if rating.is_none() {
                        rating = Some(CfRating {
                            icon: cf_icon.clone(),
                            count: *count,
                            max: *max,
                            color: color.clone(),
                            show_value: *show_value,
                        });
                    }
                }
            }
        }
        Ok(ExtendedStyle {
            style,
            icon,
            data_bar,
            rating,
        })
    }

    // -----------------------------------------------------------------------
    // CRUD API for conditional formatting rules
    // -----------------------------------------------------------------------

    /// Appends `dxf` to the workbook's dxf table and returns its new index.
    fn create_dxf(&mut self, dxf: Dxf) -> u32 {
        let id = self.workbook.styles.dxfs.len() as u32;
        self.workbook.styles.dxfs.push(dxf);
        id
    }

    /// Converts a `CfRuleInput` into a stored `CfRule`, creating a dxf entry when a format is provided.
    fn cf_rule_from_input(&mut self, rule: CfRuleInput) -> CfRule {
        match rule {
            CfRuleInput::ColorScale { thresholds } => CfRule::ColorScale { thresholds },
            CfRuleInput::CellIs {
                operator,
                formula,
                formula2,
                format,
            } => CfRule::CellIs {
                operator,
                formula,
                formula2,
                dxf_id: self.create_dxf(format),
            },
            CfRuleInput::Text {
                operator,
                value,
                format,
            } => CfRule::Text {
                operator,
                value,
                dxf_id: self.create_dxf(format),
            },
            CfRuleInput::Formula { formula, format } => CfRule::Formula {
                formula,
                dxf_id: self.create_dxf(format),
            },
            CfRuleInput::TimePeriod {
                time_period,
                date1,
                date2,
                format,
            } => CfRule::TimePeriod {
                time_period,
                date1,
                date2,
                dxf_id: self.create_dxf(format),
            },
            CfRuleInput::DuplicateValues { format } => CfRule::DuplicateValues {
                dxf_id: self.create_dxf(format),
            },
            CfRuleInput::UniqueValues { format } => CfRule::UniqueValues {
                dxf_id: self.create_dxf(format),
            },
            CfRuleInput::Blanks { format } => CfRule::Blanks {
                dxf_id: self.create_dxf(format),
            },
            CfRuleInput::NotBlanks { format } => CfRule::NotBlanks {
                dxf_id: self.create_dxf(format),
            },
            CfRuleInput::Errors { format } => CfRule::Errors {
                dxf_id: self.create_dxf(format),
            },
            CfRuleInput::NoErrors { format } => CfRule::NoErrors {
                dxf_id: self.create_dxf(format),
            },
            CfRuleInput::AboveAverage { format } => CfRule::AboveAverage {
                dxf_id: self.create_dxf(format),
            },
            CfRuleInput::BelowAverage { format } => CfRule::BelowAverage {
                dxf_id: self.create_dxf(format),
            },
            CfRuleInput::Top10 {
                rank,
                percent,
                format,
            } => CfRule::Top10 {
                rank,
                percent,
                dxf_id: self.create_dxf(format),
            },
            CfRuleInput::Bottom10 {
                rank,
                percent,
                format,
            } => CfRule::Bottom10 {
                rank,
                percent,
                dxf_id: self.create_dxf(format),
            },
            CfRuleInput::DataBar {
                min,
                max,
                positive_color,
                negative_color,
                is_gradient,
                show_value,
            } => CfRule::DataBar {
                min,
                max,
                positive_color,
                negative_color,
                is_gradient,
                show_value,
            },
            CfRuleInput::IconSet {
                thresholds,
                show_value,
            } => CfRule::IconSet {
                thresholds,
                show_value,
            },
            CfRuleInput::IconRating {
                icon,
                color,
                thresholds,
                show_value,
            } => CfRule::IconRating {
                icon,
                color,
                thresholds,
                show_value,
            },
        }
    }

    /// Returns all CF rules for the given sheet in list order.
    pub fn get_conditional_formatting_list(
        &self,
        sheet: u32,
    ) -> Result<Vec<ConditionalFormatting>, String> {
        Ok(self
            .workbook
            .worksheet(sheet)?
            .conditional_formatting
            .clone())
    }

    /// Returns the differential format (Dxf) for the CF rule at `index` on `sheet`,
    /// or `None` if the rule type has no dxf (e.g. ColorScale, DataBar, IconSet).
    pub fn get_dxf_for_conditional_formatting(
        &self,
        sheet: u32,
        index: usize,
    ) -> Result<Option<Dxf>, String> {
        let ws = self.workbook.worksheet(sheet)?;
        let cf = ws
            .conditional_formatting
            .get(index)
            .ok_or_else(|| format!("Conditional formatting index {index} out of bounds"))?;
        let dxf_id = match &cf.cf_rule {
            CfRule::CellIs { dxf_id, .. }
            | CfRule::Text { dxf_id, .. }
            | CfRule::TimePeriod { dxf_id, .. }
            | CfRule::DuplicateValues { dxf_id }
            | CfRule::UniqueValues { dxf_id }
            | CfRule::Blanks { dxf_id }
            | CfRule::NotBlanks { dxf_id }
            | CfRule::Errors { dxf_id }
            | CfRule::NoErrors { dxf_id }
            | CfRule::AboveAverage { dxf_id }
            | CfRule::BelowAverage { dxf_id }
            | CfRule::Top10 { dxf_id, .. }
            | CfRule::Bottom10 { dxf_id, .. } => *dxf_id,
            _ => return Ok(None),
        };
        Ok(self.workbook.styles.dxfs.get(dxf_id as usize).cloned())
    }

    /// Adds a new CF rule to `sheet`, appended with priority = 1 + current max.
    /// Returns the assigned priority.
    pub fn add_conditional_formatting(
        &mut self,
        sheet: u32,
        range: &str,
        rule: CfRuleInput,
    ) -> Result<u32, String> {
        if parse_sqref(range).is_empty() {
            return Err(format!("Invalid conditional formatting range: '{range}'"));
        }
        let final_rule = self.cf_rule_from_input(rule);
        let ws = self.workbook.worksheet_mut(sheet)?;
        let priority = ws
            .conditional_formatting
            .iter()
            .map(|cf| cf.priority)
            .max()
            .map(|m| m + 1)
            .unwrap_or(1);
        ws.conditional_formatting.push(ConditionalFormatting {
            range: range.to_string(),
            cf_rule: final_rule,
            priority,
            stop_if_true: false,
        });
        Ok(priority)
    }

    /// Removes the CF rule at `index` from `sheet`.  Returns the removed rule.
    pub fn delete_conditional_formatting(
        &mut self,
        sheet: u32,
        index: usize,
    ) -> Result<crate::cf_types::ConditionalFormatting, String> {
        let ws = self.workbook.worksheet_mut(sheet)?;
        if index >= ws.conditional_formatting.len() {
            return Err(format!(
                "Conditional formatting index {index} out of bounds"
            ));
        }
        Ok(ws.conditional_formatting.remove(index))
    }

    /// Replaces the range and rule of the CF entry at `index` on `sheet`.
    /// The priority is preserved.  Returns the previous entry.
    pub fn update_conditional_formatting(
        &mut self,
        sheet: u32,
        index: usize,
        new_range: &str,
        new_rule: CfRuleInput,
    ) -> Result<ConditionalFormatting, String> {
        if parse_sqref(new_range).is_empty() {
            return Err(format!(
                "Invalid conditional formatting range: '{new_range}'"
            ));
        }
        let final_rule = self.cf_rule_from_input(new_rule);
        let ws = self.workbook.worksheet_mut(sheet)?;
        if index >= ws.conditional_formatting.len() {
            return Err(format!(
                "Conditional formatting index {index} out of bounds"
            ));
        }
        let old = ws.conditional_formatting[index].clone();
        ws.conditional_formatting[index].range = new_range.to_string();
        ws.conditional_formatting[index].cf_rule = final_rule;
        Ok(old)
    }

    /// Inserts a CF entry at `index` without modifying priority (used for undo/redo).
    pub(crate) fn insert_conditional_formatting_at(
        &mut self,
        sheet: u32,
        index: usize,
        cf: crate::cf_types::ConditionalFormatting,
    ) -> Result<(), String> {
        let ws = self.workbook.worksheet_mut(sheet)?;
        if index > ws.conditional_formatting.len() {
            return Err(format!(
                "Conditional formatting index {index} out of bounds"
            ));
        }
        ws.conditional_formatting.insert(index, cf);
        Ok(())
    }
}
