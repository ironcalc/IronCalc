use crate::{
    calc_result::CalcResult,
    expressions::{parser::Node, token::Error, types::CellReferenceIndex},
    Model,
};

use super::util::{compare_values, from_wildcard_to_regex, result_matches_regex};

impl Model {
    // =DAVERAGE(database, field, criteria)
    pub(crate) fn fn_daverage(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        if args.len() != 3 {
            return CalcResult::new_args_number_error(cell);
        }

        let (db_left, db_right) = match self.get_reference(&args[0], cell) {
            Ok(r) => (r.left, r.right),
            Err(e) => return e,
        };

        let field_col = match self.resolve_db_field_column(db_left, db_right, &args[1], cell) {
            Ok(c) => c,
            Err(e) => return e,
        };

        let criteria = match self.get_reference(&args[2], cell) {
            Ok(r) => (r.left, r.right),
            Err(e) => return e,
        };

        let mut sum = 0.0f64;
        let mut count = 0usize;

        let mut row = db_left.row + 1; // skip header
        while row <= db_right.row {
            if self.db_row_matches_criteria(db_left, db_right, row, criteria) {
                let v = self.evaluate_cell(CellReferenceIndex {
                    sheet: db_left.sheet,
                    row,
                    column: field_col,
                });
                if let CalcResult::Number(n) = v {
                    if n.is_finite() {
                        sum += n;
                        count += 1;
                    }
                }
            }
            row += 1;
        }

        if count == 0 {
            return CalcResult::Error {
                error: Error::DIV,
                origin: cell,
                message: "No numeric values matched criteria".to_string(),
            };
        }

        CalcResult::Number(sum / count as f64)
    }

    // =DCOUNT(database, field, criteria)
    // Counts numeric entries in the field for rows that match criteria
    pub(crate) fn fn_dcount(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        if args.len() != 3 {
            return CalcResult::new_args_number_error(cell);
        }

        let (db_left, db_right) = match self.get_reference(&args[0], cell) {
            Ok(r) => (r.left, r.right),
            Err(e) => return e,
        };

        let field_col = match self.resolve_db_field_column(db_left, db_right, &args[1], cell) {
            Ok(c) => c,
            Err(e) => return e,
        };

        let criteria = match self.get_reference(&args[2], cell) {
            Ok(r) => (r.left, r.right),
            Err(e) => return e,
        };

        let mut count = 0usize;
        let mut row = db_left.row + 1; // skip header
        while row <= db_right.row {
            if self.db_row_matches_criteria(db_left, db_right, row, criteria) {
                let v = self.evaluate_cell(CellReferenceIndex {
                    sheet: db_left.sheet,
                    row,
                    column: field_col,
                });
                if matches!(v, CalcResult::Number(_)) {
                    count += 1;
                }
            }
            row += 1;
        }

        CalcResult::Number(count as f64)
    }

    // =DGET(database, field, criteria)
    // Returns the (single) field value for the unique matching row
    pub(crate) fn fn_dget(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        if args.len() != 3 {
            return CalcResult::new_args_number_error(cell);
        }

        let (db_left, db_right) = match self.get_reference(&args[0], cell) {
            Ok(r) => (r.left, r.right),
            Err(e) => return e,
        };

        let field_col = match self.resolve_db_field_column(db_left, db_right, &args[1], cell) {
            Ok(c) => c,
            Err(e) => return e,
        };

        let criteria = match self.get_reference(&args[2], cell) {
            Ok(r) => (r.left, r.right),
            Err(e) => return e,
        };

        let mut result: Option<CalcResult> = None;
        let mut matches = 0usize;

        let mut row = db_left.row + 1; // skip header
        while row <= db_right.row {
            if self.db_row_matches_criteria(db_left, db_right, row, criteria) {
                matches += 1;
                if matches > 1 {
                    return CalcResult::Error {
                        error: Error::NUM,
                        origin: cell,
                        message: "More than one matching record".to_string(),
                    };
                }
                result = Some(self.evaluate_cell(CellReferenceIndex {
                    sheet: db_left.sheet,
                    row,
                    column: field_col,
                }));
            }
            row += 1;
        }

        match (matches, result) {
            (0, _) | (_, None) => CalcResult::Error {
                error: Error::VALUE,
                origin: cell,
                message: "No matching record".to_string(),
            },
            (_, Some(v)) => v,
        }
    }

    // =DMAX(database, field, criteria)
    pub(crate) fn fn_dmax(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.db_extreme(args, cell, true)
    }

    // =DMIN(database, field, criteria)
    pub(crate) fn fn_dmin(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.db_extreme(args, cell, false)
    }

    // =DSUM(database, field, criteria)
    pub(crate) fn fn_dsum(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        if args.len() != 3 {
            return CalcResult::new_args_number_error(cell);
        }

        let (db_left, db_right) = match self.get_reference(&args[0], cell) {
            Ok(r) => (r.left, r.right),
            Err(e) => return e,
        };

        let field_col = match self.resolve_db_field_column(db_left, db_right, &args[1], cell) {
            Ok(c) => c,
            Err(e) => return e,
        };

        let criteria = match self.get_reference(&args[2], cell) {
            Ok(r) => (r.left, r.right),
            Err(e) => return e,
        };

        let mut sum = 0.0f64;

        // skip header
        let mut row = db_left.row + 1;
        while row <= db_right.row {
            if self.db_row_matches_criteria(db_left, db_right, row, criteria) {
                let v = self.evaluate_cell(CellReferenceIndex {
                    sheet: db_left.sheet,
                    row,
                    column: field_col,
                });
                if let CalcResult::Number(n) = v {
                    if n.is_finite() {
                        sum += n;
                    }
                }
            }
            row += 1;
        }

        CalcResult::Number(sum)
    }

    /// Resolve the "field" (2nd arg) to an absolute column index (i32) within the sheet.
    /// Field can be a number (1-based index) or a header name (case-insensitive).
    /// Returns the absolute column index, not a 1-based offset within the database range.
    fn resolve_db_field_column(
        &mut self,
        db_left: CellReferenceIndex,
        db_right: CellReferenceIndex,
        field_arg: &Node,
        cell: CellReferenceIndex,
    ) -> Result<i32, CalcResult> {
        // If numeric -> index
        if let Ok(n) = self.get_number(field_arg, cell) {
            let idx = if n < 1.0 {
                n.ceil() as i32
            } else {
                n.floor() as i32
            };
            if idx < 1 || db_left.column + idx - 1 > db_right.column {
                return Err(CalcResult::Error {
                    error: Error::VALUE,
                    origin: cell,
                    message: "Field index out of range".to_string(),
                });
            }
            return Ok(db_left.column + idx - 1);
        }

        // Otherwise treat as header name
        let wanted = match self.get_string(field_arg, cell) {
            Ok(s) => s.to_lowercase(),
            Err(e) => return Err(e),
        };

        let mut col = db_left.column;
        while col <= db_right.column {
            let v = self.evaluate_cell(CellReferenceIndex {
                sheet: db_left.sheet,
                row: db_left.row,
                column: col,
            });
            if let CalcResult::String(s) = v {
                if s.to_lowercase() == wanted {
                    return Ok(col);
                }
            }
            col += 1;
        }

        Err(CalcResult::Error {
            error: Error::VALUE,
            origin: cell,
            message: "Field header not found".to_string(),
        })
    }

    /// Check whether a database row matches the criteria range.
    /// Criteria logic: OR across criteria rows; AND across columns within a row.
    fn db_row_matches_criteria(
        &mut self,
        db_left: CellReferenceIndex,
        db_right: CellReferenceIndex,
        row: i32,
        criteria: (CellReferenceIndex, CellReferenceIndex),
    ) -> bool {
        let (c_left, c_right) = criteria;

        // Read criteria headers (first row of criteria range)
        // Map header name (lowercased) -> db column (if exists)
        let mut crit_cols: Vec<Option<i32>> = Vec::new();
        let mut col = c_left.column;
        while col <= c_right.column {
            let h = self.evaluate_cell(CellReferenceIndex {
                sheet: c_left.sheet,
                row: c_left.row,
                column: col,
            });
            let db_col = if let CalcResult::String(s) = h {
                let wanted = s.to_lowercase();
                // Find corresponding DB column
                let mut db_c = db_left.column;
                let mut found: Option<i32> = None;
                while db_c <= db_right.column {
                    let hdr = self.evaluate_cell(CellReferenceIndex {
                        sheet: db_left.sheet,
                        row: db_left.row,
                        column: db_c,
                    });
                    if let CalcResult::String(hs) = hdr {
                        if hs.to_lowercase() == wanted {
                            found = Some(db_c);
                            break;
                        }
                    }
                    db_c += 1;
                }
                found
            } else {
                None
            };
            crit_cols.push(db_col);
            col += 1;
        }

        // If no criteria rows (only headers), everything matches
        if c_right.row <= c_left.row {
            return true;
        }

        // Evaluate each criteria row (OR)
        let mut r = c_left.row + 1;
        while r <= c_right.row {
            // AND across columns for this criteria row
            let mut and_ok = true;

            for (offset, maybe_db_col) in crit_cols.iter().enumerate() {
                // Criteria cell
                let ccell = self.evaluate_cell(CellReferenceIndex {
                    sheet: c_left.sheet,
                    row: r,
                    column: c_left.column + offset as i32,
                });

                // Empty criteria cell -> ignored
                if matches!(ccell, CalcResult::EmptyCell | CalcResult::EmptyArg) {
                    continue;
                }

                // Header without mapping -> ignore this criteria column (Excel ignores unknown headers)
                let db_col = match maybe_db_col {
                    Some(c) => *c,
                    None => continue,
                };

                // Database value for this row/column
                let db_val = self.evaluate_cell(CellReferenceIndex {
                    sheet: db_left.sheet,
                    row,
                    column: db_col,
                });

                if !self.criteria_cell_matches(&db_val, &ccell) {
                    and_ok = false;
                    break;
                }
            }

            if and_ok {
                // This criteria row satisfied (OR)
                return true;
            }

            r += 1;
        }

        // none matched
        false
    }

    /// Implements Excel-like criteria matching for a single value.
    /// Supports prefixes: <>, >=, <=, >, <, = ; wildcards * and ? for string equals.
    fn criteria_cell_matches(&self, db_val: &CalcResult, crit_cell: &CalcResult) -> bool {
        // Convert the criteria cell to a string for operator parsing if possible,
        // otherwise fall back to equality via compare_values.
        let crit_str_opt = match crit_cell {
            CalcResult::String(s) => Some(s.clone()),
            CalcResult::Number(n) => Some(n.to_string()),
            CalcResult::Boolean(b) => Some(if *b {
                "TRUE".to_string()
            } else {
                "FALSE".to_string()
            }),
            CalcResult::EmptyCell | CalcResult::EmptyArg => return true,
            CalcResult::Error { .. } => return false,
            CalcResult::Range { .. } | CalcResult::Array(_) => return false,
        };

        if crit_str_opt.is_none() {
            return compare_values(db_val, crit_cell) == 0;
        }
        let mut crit = match crit_str_opt {
            Some(s) => s.trim().to_string(),
            None => return false,
        };

        // Detect operator prefix
        let mut op = "="; // default equality (with wildcard semantics for strings)
        let prefixes = ["<>", ">=", "<=", ">", "<", "="];
        for p in prefixes.iter() {
            if crit.starts_with(p) {
                op = p;
                crit = crit[p.len()..].trim().to_string();
                break;
            }
        }

        // Try to parse numeric RHS
        let rhs_num = crit.parse::<f64>().ok();

        match op {
            ">" | ">=" | "<" | "<=" => {
                if let Some(t) = rhs_num {
                    // numeric comparison
                    if let CalcResult::Number(n) = db_val {
                        match op {
                            ">" => *n > t,
                            ">=" => *n >= t,
                            "<" => *n < t,
                            "<=" => *n <= t,
                            _ => false,
                        }
                    } else {
                        // For non-numbers, use compare_values with a number token to emulate Excel ordering
                        let rhs = CalcResult::Number(t);
                        let c = compare_values(db_val, &rhs);
                        match op {
                            ">" => c > 0,
                            ">=" => c >= 0,
                            "<" => c < 0,
                            "<=" => c <= 0,
                            _ => false,
                        }
                    }
                } else {
                    // string comparison (case-insensitive) using compare_values semantics
                    let rhs = CalcResult::String(crit.to_lowercase());
                    let lhs = match db_val {
                        CalcResult::String(s) => CalcResult::String(s.to_lowercase()),
                        x => x.clone(),
                    };
                    let c = compare_values(&lhs, &rhs);
                    match op {
                        ">" => c > 0,
                        ">=" => c >= 0,
                        "<" => c < 0,
                        "<=" => c <= 0,
                        _ => false,
                    }
                }
            }
            "<>" => {
                // not equal (with wildcard semantics for strings)
                // If rhs has wildcards and db_val is string, do regex; else use compare_values != 0
                if let CalcResult::String(s) = db_val {
                    if crit.contains('*') || crit.contains('?') {
                        if let Ok(re) = from_wildcard_to_regex(&crit.to_lowercase(), true) {
                            return !result_matches_regex(
                                &CalcResult::String(s.to_lowercase()),
                                &re,
                            );
                        }
                    }
                }
                let rhs = if let Some(n) = rhs_num {
                    CalcResult::Number(n)
                } else {
                    CalcResult::String(crit.to_lowercase())
                };
                let lhs = match db_val {
                    CalcResult::String(s) => CalcResult::String(s.to_lowercase()),
                    x => x.clone(),
                };
                compare_values(&lhs, &rhs) != 0
            }
            _ => {
                // equality. For strings, support wildcards (*, ?)
                if let Some(n) = rhs_num {
                    // numeric equals
                    if let CalcResult::Number(m) = db_val {
                        (*m - n).abs() <= f64::EPSILON
                    } else {
                        compare_values(db_val, &CalcResult::Number(n)) == 0
                    }
                } else {
                    // textual/boolean equals (case-insensitive), wildcard-enabled for strings
                    if let CalcResult::String(s) = db_val {
                        if crit.contains('*') || crit.contains('?') {
                            if let Ok(re) = from_wildcard_to_regex(&crit.to_lowercase(), true) {
                                return result_matches_regex(
                                    &CalcResult::String(s.to_lowercase()),
                                    &re,
                                );
                            }
                        }
                        return s.to_lowercase() == crit.to_lowercase();
                    }
                    // Fallback: compare_values equality
                    compare_values(db_val, &CalcResult::String(crit.to_lowercase())) == 0
                }
            }
        }
    }

    /// Shared implementation for DMAX/DMIN
    fn db_extreme(
        &mut self,
        args: &[Node],
        cell: CellReferenceIndex,
        want_max: bool,
    ) -> CalcResult {
        if args.len() != 3 {
            return CalcResult::new_args_number_error(cell);
        }

        let (db_left, db_right) = match self.get_reference(&args[0], cell) {
            Ok(r) => (r.left, r.right),
            Err(e) => return e,
        };

        let field_col = match self.resolve_db_field_column(db_left, db_right, &args[1], cell) {
            Ok(c) => c,
            Err(e) => return e,
        };

        let criteria = match self.get_reference(&args[2], cell) {
            Ok(r) => (r.left, r.right),
            Err(e) => return e,
        };

        let mut best: Option<f64> = None;

        let mut row = db_left.row + 1; // skip header
        while row <= db_right.row {
            if self.db_row_matches_criteria(db_left, db_right, row, criteria) {
                let v = self.evaluate_cell(CellReferenceIndex {
                    sheet: db_left.sheet,
                    row,
                    column: field_col,
                });
                if let CalcResult::Number(n) = v {
                    if n.is_finite() {
                        best = Some(match best {
                            None => n,
                            Some(cur) => {
                                if want_max {
                                    n.max(cur)
                                } else {
                                    n.min(cur)
                                }
                            }
                        });
                    }
                }
            }
            row += 1;
        }

        match best {
            Some(v) => CalcResult::Number(v),
            None => CalcResult::Error {
                error: Error::VALUE,
                origin: cell,
                message: "No numeric values matched criteria".to_string(),
            },
        }
    }
}
