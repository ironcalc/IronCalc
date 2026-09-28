#[cfg(not(target_arch = "wasm32"))]
use regex::Regex;
#[cfg(target_arch = "wasm32")]
use regex_lite::Regex;

use crate::{
    calc_result::CalcResult,
    expressions::{
        parser::{ArrayNode, Node},
        token::Error,
        types::CellReferenceIndex,
    },
    model::Model,
};

impl<'a> Model<'a> {
    /// REGEXEXTRACT(text, regular_expression, [return_mode], [case_sensitivity])
    ///
    /// Returns the first substring of `text` that matches `regular_expression`.
    /// If the pattern contains capture groups, returns the content of the first group.
    /// `return_mode`: 0 (default) the first match; 1 a horizontal array of every
    /// match; 2 a horizontal array of the capture groups of the first match.
    /// `case_sensitivity`: 0 (default, or left out) matches case, 1 ignores it.
    /// Returns #N/A if there is no match, #VALUE! if the regex is invalid.
    pub(crate) fn fn_regexextract(
        &mut self,
        args: &[Node],
        cell: CellReferenceIndex,
    ) -> CalcResult {
        if args.len() < 2 || args.len() > 4 {
            return CalcResult::new_args_number_error(cell);
        }
        let text = match self.get_string(&args[0], cell) {
            Ok(s) => s,
            Err(e) => return e,
        };
        let pattern = match self.get_string(&args[1], cell) {
            Ok(s) => s,
            Err(e) => return e,
        };
        // 0: the first match, 1: every match, 2: the groups of the first match
        let mode = if args.len() >= 3 {
            match self.get_number(&args[2], cell) {
                Ok(n) => n.trunc() as i32,
                Err(e) => return e,
            }
        } else {
            0
        };
        if !(0..=2).contains(&mode) {
            return CalcResult::new_error(Error::VALUE, cell, "Invalid return mode".to_string());
        }
        let return_all = mode == 1;
        let pattern = match self.regex_case(args.get(3), cell) {
            Ok(true) => format!("(?i){pattern}"),
            Ok(false) => pattern,
            Err(e) => return e,
        };
        let re = match Regex::new(&pattern) {
            Ok(r) => r,
            Err(_) => {
                return CalcResult::new_error(
                    Error::VALUE,
                    cell,
                    "Invalid regular expression".to_string(),
                )
            }
        };
        if return_all {
            let row: Vec<ArrayNode> = re
                .find_iter(&text)
                .map(|m| ArrayNode::String(m.as_str().to_string()))
                .collect();
            if row.is_empty() {
                return CalcResult::new_error(Error::NA, cell, "No match found".to_string());
            }
            CalcResult::Array(vec![row])
        } else if mode == 2 {
            match re.captures(&text) {
                None => CalcResult::new_error(Error::NA, cell, "No match found".to_string()),
                Some(caps) => {
                    let row: Vec<ArrayNode> = if caps.len() > 1 {
                        caps.iter()
                            .skip(1)
                            .map(|m| ArrayNode::String(m.map_or("", |m| m.as_str()).to_string()))
                            .collect()
                    } else {
                        vec![ArrayNode::String(
                            caps.get(0).map_or("", |m| m.as_str()).to_string(),
                        )]
                    };
                    CalcResult::Array(vec![row])
                }
            }
        } else {
            match re.captures(&text) {
                None => CalcResult::new_error(Error::NA, cell, "No match found".to_string()),
                Some(caps) => {
                    // If there is at least one explicit capture group, return group 1.
                    // Otherwise return the full match (group 0).
                    let matched = if caps.len() > 1 {
                        caps.get(1).map_or("", |m| m.as_str())
                    } else {
                        caps.get(0).map_or("", |m| m.as_str())
                    };
                    CalcResult::String(matched.to_string())
                }
            }
        }
    }

    /// REGEXREPLACE(text, regular_expression, replacement, [occurrence], [case_sensitivity])
    ///
    /// Replaces every substring of `text` that matches `regular_expression` with
    /// `replacement`. `occurrence`: 0 or left out replaces every match; a positive
    /// n replaces only the n-th match, a negative n counts from the last match.
    /// Returns #VALUE! if the regex is invalid.
    pub(crate) fn fn_regexreplace(
        &mut self,
        args: &[Node],
        cell: CellReferenceIndex,
    ) -> CalcResult {
        if args.len() < 3 || args.len() > 5 {
            return CalcResult::new_args_number_error(cell);
        }
        let text = match self.get_string(&args[0], cell) {
            Ok(s) => s,
            Err(e) => return e,
        };
        let pattern = match self.get_string(&args[1], cell) {
            Ok(s) => s,
            Err(e) => return e,
        };
        let replacement = match self.get_string(&args[2], cell) {
            Ok(s) => s,
            Err(e) => return e,
        };
        // 0 (or left out) replaces every match; n the n-th, -n the n-th from the end
        let occurrence = match args.get(3) {
            Some(node) if !matches!(node, Node::EmptyArgKind) => {
                match self.get_number(node, cell) {
                    Ok(n) => n.trunc() as i64,
                    Err(e) => return e,
                }
            }
            _ => 0,
        };
        let pattern = match self.regex_case(args.get(4), cell) {
            Ok(true) => format!("(?i){pattern}"),
            Ok(false) => pattern,
            Err(e) => return e,
        };
        let re = match Regex::new(&pattern) {
            Ok(r) => r,
            Err(_) => {
                return CalcResult::new_error(
                    Error::VALUE,
                    cell,
                    "Invalid regular expression".to_string(),
                )
            }
        };
        if occurrence == 0 {
            return CalcResult::String(re.replace_all(&text, replacement.as_str()).into_owned());
        }
        let matches: Vec<_> = re.captures_iter(&text).collect();
        let count = matches.len() as i64;
        let index = if occurrence > 0 {
            occurrence - 1
        } else {
            count + occurrence
        };
        match usize::try_from(index).ok().and_then(|i| matches.get(i)) {
            Some(caps) => {
                let Some(whole) = caps.get(0) else {
                    return CalcResult::String(text);
                };
                let mut replaced = String::new();
                caps.expand(&replacement, &mut replaced);
                CalcResult::String(format!(
                    "{}{}{}",
                    &text[..whole.start()],
                    replaced,
                    &text[whole.end()..]
                ))
            }
            None => CalcResult::String(text),
        }
    }

    /// REGEXTEST(text, regular_expression, [case_sensitivity])
    ///
    /// Returns `TRUE` if `regular_expression` matches anywhere in `text`, `FALSE`
    /// otherwise. Returns #VALUE! if the regex is invalid.
    pub(crate) fn fn_regextest(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        if args.len() < 2 || args.len() > 3 {
            return CalcResult::new_args_number_error(cell);
        }
        let text = match self.get_string(&args[0], cell) {
            Ok(s) => s,
            Err(e) => return e,
        };
        let pattern = match self.get_string(&args[1], cell) {
            Ok(s) => s,
            Err(e) => return e,
        };
        let pattern = match self.regex_case(args.get(2), cell) {
            Ok(true) => format!("(?i){pattern}"),
            Ok(false) => pattern,
            Err(e) => return e,
        };
        let re = match Regex::new(&pattern) {
            Ok(r) => r,
            Err(_) => {
                return CalcResult::new_error(
                    Error::VALUE,
                    cell,
                    "Invalid regular expression".to_string(),
                )
            }
        };
        CalcResult::Boolean(re.is_match(&text))
    }

    /// The optional case_sensitivity argument of Excel's regex functions:
    /// 0 (or left out) matches case, 1 ignores it. Ok(true) means ignore.
    fn regex_case(
        &mut self,
        node: Option<&Node>,
        cell: CellReferenceIndex,
    ) -> Result<bool, CalcResult> {
        match node {
            None | Some(Node::EmptyArgKind) => Ok(false),
            Some(node) => match self.get_number(node, cell)?.trunc() as i32 {
                0 => Ok(false),
                1 => Ok(true),
                _ => Err(CalcResult::new_error(
                    Error::VALUE,
                    cell,
                    "case_sensitivity must be 0 or 1".to_string(),
                )),
            },
        }
    }
}
