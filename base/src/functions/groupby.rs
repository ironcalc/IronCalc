//! GROUPBY and PIVOTBY (summaries of a table by one or more fields), and a
//! function name used on its own as a LAMBDA (`=GROUPBY(A2:A9, C2:C9, SUM)`,
//! `=BYROW(A1:C3, SUM)`).

use std::collections::HashMap;

use crate::{
    calc_result::CalcResult,
    constants::{LAST_COLUMN, LAST_ROW},
    expressions::{
        parser::{ArrayNode, NamedVariable, Node},
        token::Error,
        types::CellReferenceIndex,
    },
    functions::{util::compare_values, Function},
    language::get_default_language,
    model::Model,
    number_format::to_precision,
};

/// Rows that belong together are the ones whose keys match (text ignores
/// case, numbers are compared to 15 digits, blank and empty text match).
fn group_key(node: &ArrayNode) -> String {
    match node {
        ArrayNode::Empty => "e".to_string(),
        ArrayNode::String(s) if s.is_empty() => "e".to_string(),
        ArrayNode::String(s) => format!("s{}", s.to_lowercase()),
        ArrayNode::Number(f) => format!("n{}", to_precision(*f, 15)),
        ArrayNode::Boolean(b) => format!("b{b}"),
        ArrayNode::Error(e) => format!("x{e}"),
    }
}

fn node_result(node: &ArrayNode, cell: CellReferenceIndex) -> CalcResult {
    match node {
        ArrayNode::Number(f) => CalcResult::Number(*f),
        ArrayNode::Boolean(b) => CalcResult::Boolean(*b),
        ArrayNode::String(s) => CalcResult::String(s.clone()),
        ArrayNode::Error(e) => CalcResult::new_error(e.clone(), cell, String::new()),
        ArrayNode::Empty => CalcResult::EmptyCell,
    }
}

fn compare_nodes(a: &ArrayNode, b: &ArrayNode, cell: CellReferenceIndex) -> i32 {
    compare_values(&node_result(a, cell), &node_result(b, cell))
}

/// A spilled blank shows as 0, so blanks in a result are empty text.
fn shown(node: ArrayNode) -> ArrayNode {
    match node {
        ArrayNode::Empty => ArrayNode::String(String::new()),
        other => other,
    }
}

fn text(s: &str) -> ArrayNode {
    ArrayNode::String(s.to_string())
}

fn is_true(node: &ArrayNode) -> bool {
    match node {
        ArrayNode::Boolean(b) => *b,
        ArrayNode::Number(f) => *f != 0.0,
        _ => false,
    }
}

/// One output line of a summary: its keys (blank past a subtotal's level,
/// "Total" for the grand total) and the data rows it covers.
struct Line {
    keys: Vec<ArrayNode>,
    rows: Vec<usize>,
}

/// What each summary cell is worked out from: the function, whether it
/// also gets the whole (PERCENTOF and two-parameter LAMBDAs), and the
/// value columns.
struct Summary {
    function: CalcResult,
    two: bool,
    values: Vec<Vec<ArrayNode>>,
    all: Vec<usize>,
}

/// How the lines are laid out: how many key columns, the total depth
/// (0 none, 1 grand total, 2 and up also subtotals; negative puts them
/// on top), the sort order (1-based columns, keys first then values;
/// negative is descending) and whether the fields are a flat table.
struct Layout<'s> {
    width: usize,
    depth: i32,
    sort: &'s [i32],
    table: bool,
}

const TOTAL: &str = "Total";

impl<'a> Model<'a> {
    /// A function name used without brackets (`SUM`, or `_xleta.SUM` as
    /// Excel stores it) is a LAMBDA that calls that function. PERCENTOF takes
    /// two values, every other function one.
    pub(crate) fn eta_lambda(&mut self, name: &str) -> Option<CalcResult> {
        let bare = name
            .trim_start_matches("_xleta.")
            .trim_start_matches("_xlfn.");
        let kind = self
            .language
            .functions
            .lookup(bare)
            .or_else(|| get_default_language().functions.lookup(bare))?;
        if matches!(kind, Function::Lambda | Function::Let) {
            return None;
        }
        let count = if matches!(kind, Function::Percentof) {
            2
        } else {
            1
        };
        let parameters: Vec<NamedVariable> = (1..=count)
            .map(|i| NamedVariable {
                name: format!("_eta{i}"),
                id: None,
                is_optional: false,
            })
            .collect();
        let body = Node::FunctionKind {
            kind,
            args: parameters
                .iter()
                .map(|p| Node::NamedVariableKind {
                    name: p.name.clone(),
                    id: None,
                })
                .collect(),
        };
        let id = self.get_next_lambda_id();
        self.lambdas.insert(id, (parameters, body));
        Some(CalcResult::Lambda(id))
    }

    fn summary_function(
        &mut self,
        node: &Node,
        cell: CellReferenceIndex,
    ) -> Result<(CalcResult, bool), CalcResult> {
        match self.evaluate_node_in_context(node, cell) {
            CalcResult::Lambda(id) => {
                let two = self.lambdas.get(&id).is_some_and(|l| l.0.len() >= 2);
                Ok((CalcResult::Lambda(id), two))
            }
            error @ CalcResult::Error { .. } => Err(error),
            _ => Err(CalcResult::new_error(
                Error::VALUE,
                cell,
                "Expected a function such as SUM, or a LAMBDA".to_string(),
            )),
        }
    }

    /// The function's answer for the given rows of value column `column`.
    fn summarize(
        &mut self,
        summary: &Summary,
        column: usize,
        rows: &[usize],
        whole: &[usize],
        cell: CellReferenceIndex,
    ) -> ArrayNode {
        let pick = |rows: &[usize]| -> Vec<Vec<ArrayNode>> {
            rows.iter()
                .map(|r| {
                    vec![summary
                        .values
                        .get(*r)
                        .and_then(|v| v.get(column))
                        .cloned()
                        .unwrap_or(ArrayNode::Empty)]
                })
                .collect()
        };
        let mut args = vec![CalcResult::Array(pick(rows))];
        if summary.two {
            args.push(CalcResult::Array(pick(whole)));
        }
        match self.call_lambda_with_values(summary.function.clone(), args, cell) {
            CalcResult::Number(f) => ArrayNode::Number(f),
            CalcResult::Boolean(b) => ArrayNode::Boolean(b),
            CalcResult::String(s) => ArrayNode::String(s),
            CalcResult::Error { error, .. } => ArrayNode::Error(error),
            CalcResult::EmptyCell | CalcResult::EmptyArg => ArrayNode::String(String::new()),
            CalcResult::Array(a) if a.len() == 1 && a[0].len() == 1 => shown(a[0][0].clone()),
            CalcResult::Range { left, right } if left == right => match self.evaluate_cell(left) {
                CalcResult::Number(f) => ArrayNode::Number(f),
                CalcResult::String(s) => ArrayNode::String(s),
                CalcResult::Boolean(b) => ArrayNode::Boolean(b),
                _ => ArrayNode::String(String::new()),
            },
            _ => ArrayNode::Error(Error::CALC),
        }
    }

    fn summary_sort(
        &mut self,
        node: Option<&Node>,
        columns: usize,
        cell: CellReferenceIndex,
    ) -> Result<Vec<i32>, CalcResult> {
        let node = match node {
            None | Some(Node::EmptyArgKind) => return Ok(Vec::new()),
            Some(n) => n,
        };
        let grid = self.summary_grid(node, cell)?;
        let mut out = Vec::new();
        for value in grid.iter().flatten() {
            match value {
                ArrayNode::Number(f)
                    if f.trunc() != 0.0 && (f.abs().trunc() as usize) <= columns =>
                {
                    out.push(f.trunc() as i32)
                }
                ArrayNode::Empty => {}
                _ => {
                    return Err(CalcResult::new_error(
                        Error::VALUE,
                        cell,
                        "Invalid sort order".to_string(),
                    ))
                }
            }
        }
        Ok(out)
    }

    fn summary_option(
        &mut self,
        node: Option<&Node>,
        cell: CellReferenceIndex,
    ) -> Result<Option<i32>, CalcResult> {
        match node {
            None | Some(Node::EmptyArgKind) => Ok(None),
            Some(n) => Ok(Some(self.get_number(n, cell)?.trunc() as i32)),
        }
    }

    /// The groups at one level, in first-seen order, then sorted.
    fn sorted_groups(
        &mut self,
        keys: &[Vec<ArrayNode>],
        rows: &[usize],
        columns: &[usize],
        layout: &Layout,
        summary: &Summary,
        cell: CellReferenceIndex,
    ) -> Vec<(Vec<ArrayNode>, Vec<usize>)> {
        let mut order: Vec<String> = Vec::new();
        let mut groups: HashMap<String, (Vec<ArrayNode>, Vec<usize>)> = HashMap::new();
        for r in rows {
            let values: Vec<ArrayNode> = columns
                .iter()
                .map(|c| {
                    keys.get(*r)
                        .and_then(|k| k.get(*c))
                        .cloned()
                        .unwrap_or(ArrayNode::Empty)
                })
                .collect();
            let key = values
                .iter()
                .map(group_key)
                .collect::<Vec<_>>()
                .join("\u{1f}");
            match groups.get_mut(&key) {
                Some(g) => g.1.push(*r),
                None => {
                    order.push(key.clone());
                    groups.insert(key, (values, vec![*r]));
                }
            }
        }
        let list: Vec<(Vec<ArrayNode>, Vec<usize>)> =
            order.iter().filter_map(|k| groups.remove(k)).collect();
        // What each group is sorted by: the chosen key columns and value
        // summaries, then its own keys ascending.
        let mut sort_values: Vec<Vec<(ArrayNode, bool)>> = Vec::with_capacity(list.len());
        for (values, group_rows) in &list {
            let mut by = Vec::new();
            for s in layout.sort {
                let column = s.unsigned_abs() as usize - 1;
                let ascending = *s > 0;
                if column < layout.width {
                    if let Some(i) = columns.iter().position(|c| *c == column) {
                        by.push((values[i].clone(), ascending));
                    }
                } else {
                    let v = self.summarize(
                        summary,
                        column - layout.width,
                        group_rows,
                        &summary.all,
                        cell,
                    );
                    by.push((v, ascending));
                }
            }
            for v in values {
                by.push((v.clone(), true));
            }
            sort_values.push(by);
        }
        let mut index: Vec<usize> = (0..list.len()).collect();
        index.sort_by(|a, b| {
            for (x, y) in sort_values[*a].iter().zip(sort_values[*b].iter()) {
                let order = compare_nodes(&x.0, &y.0, cell);
                if order != 0 {
                    let order = if x.1 { order } else { -order };
                    return order.cmp(&0);
                }
            }
            std::cmp::Ordering::Equal
        });
        let mut list: Vec<Option<(Vec<ArrayNode>, Vec<usize>)>> =
            list.into_iter().map(Some).collect();
        index.into_iter().filter_map(|i| list[i].take()).collect()
    }

    #[allow(clippy::too_many_arguments)]
    fn nest_lines(
        &mut self,
        keys: &[Vec<ArrayNode>],
        rows: &[usize],
        level: usize,
        prefix: &mut Vec<ArrayNode>,
        layout: &Layout,
        summary: &Summary,
        cell: CellReferenceIndex,
        out: &mut Vec<Line>,
    ) {
        let groups = self.sorted_groups(keys, rows, &[level], layout, summary, cell);
        for (values, group_rows) in groups {
            prefix.push(values[0].clone());
            if level + 1 >= layout.width {
                out.push(Line {
                    keys: prefix.clone(),
                    rows: group_rows,
                });
            } else {
                let subtotal = layout.depth.unsigned_abs() as usize >= level + 2;
                let mut sub_keys = prefix.clone();
                sub_keys.resize(layout.width, ArrayNode::Empty);
                if subtotal && layout.depth < 0 {
                    out.push(Line {
                        keys: sub_keys.clone(),
                        rows: group_rows.clone(),
                    });
                }
                self.nest_lines(
                    keys,
                    &group_rows,
                    level + 1,
                    prefix,
                    layout,
                    summary,
                    cell,
                    out,
                );
                if subtotal && layout.depth > 0 {
                    out.push(Line {
                        keys: sub_keys,
                        rows: group_rows,
                    });
                }
            }
            prefix.pop();
        }
    }

    /// The lines of a summary: groups (nested, or flat for a table), their
    /// subtotals and the grand total.
    fn summary_lines(
        &mut self,
        keys: &[Vec<ArrayNode>],
        rows: &[usize],
        layout: &Layout,
        summary: &Summary,
        cell: CellReferenceIndex,
    ) -> Vec<Line> {
        let mut lines = Vec::new();
        if layout.table || layout.width <= 1 {
            let columns: Vec<usize> = (0..layout.width).collect();
            for (values, group_rows) in
                self.sorted_groups(keys, rows, &columns, layout, summary, cell)
            {
                lines.push(Line {
                    keys: values,
                    rows: group_rows,
                });
            }
        } else {
            self.nest_lines(
                keys,
                rows,
                0,
                &mut Vec::new(),
                layout,
                summary,
                cell,
                &mut lines,
            );
        }
        if layout.depth != 0 {
            let mut total = vec![text(TOTAL)];
            total.resize(layout.width.max(1), ArrayNode::Empty);
            let line = Line {
                keys: total,
                rows: rows.to_vec(),
            };
            if layout.depth < 0 {
                lines.insert(0, line);
            } else {
                lines.push(line);
            }
        }
        lines
    }

    /// Reads the fields and values, drops the header row when there is one,
    /// and applies the filter. Returns the header names, whether to show
    /// them, and the data rows kept.
    #[allow(clippy::type_complexity)]
    fn summary_data(
        &mut self,
        grids: &mut [Vec<Vec<ArrayNode>>],
        headers: Option<i32>,
        filter: Option<&Node>,
        cell: CellReferenceIndex,
    ) -> Result<(Vec<Vec<ArrayNode>>, bool, Vec<usize>), CalcResult> {
        let height = grids.first().map_or(0, |g| g.len());
        if grids.iter().any(|g| g.len() != height) || height == 0 {
            return Err(CalcResult::new_error(
                Error::VALUE,
                cell,
                "The fields and values must have the same number of rows".to_string(),
            ));
        }
        let values = grids.last().cloned().unwrap_or_default();
        let mode = match headers {
            Some(h) if (0..=3).contains(&h) => h,
            Some(_) => {
                return Err(CalcResult::new_error(
                    Error::VALUE,
                    cell,
                    "field_headers must be 0 to 3".to_string(),
                ))
            }
            None => {
                let first_text = values.first().is_some_and(|r| {
                    r.iter()
                        .all(|v| matches!(v, ArrayNode::String(s) if !s.is_empty()))
                });
                let second_number = values
                    .get(1)
                    .is_some_and(|r| r.iter().any(|v| matches!(v, ArrayNode::Number(_))));
                if first_text && second_number {
                    3
                } else {
                    0
                }
            }
        };
        let has = mode == 1 || mode == 3;
        let show = mode >= 2;
        let mut names = Vec::new();
        let count = grids.len();
        for (g, grid) in grids.iter_mut().enumerate() {
            let width = grid.first().map_or(0, |r| r.len());
            if has {
                names.push(grid.remove(0));
            } else {
                let label = if g + 1 == count { "Value" } else { "Field" };
                names.push((1..=width).map(|i| text(&format!("{label} {i}"))).collect());
            }
        }
        let height = if has { height - 1 } else { height };
        let mut rows: Vec<usize> = (0..height).collect();
        if let Some(node) = filter {
            if !matches!(node, Node::EmptyArgKind) {
                let mut grid = self.summary_grid(node, cell)?;
                if grid.len() == height + 1 && has {
                    grid.remove(0);
                }
                if grid.len() != height {
                    return Err(CalcResult::new_error(
                        Error::VALUE,
                        cell,
                        "The filter must have one value per row".to_string(),
                    ));
                }
                rows.retain(|r| grid[*r].first().is_some_and(is_true));
            }
        }
        if rows.is_empty() {
            return Err(CalcResult::new_error(
                Error::CALC,
                cell,
                "No rows to summarize".to_string(),
            ));
        }
        Ok((names, show, rows))
    }

    /// The values of a range (a whole column stops at the last used row) or
    /// an array, or a single value as a 1x1 table.
    fn summary_grid(
        &mut self,
        node: &Node,
        cell: CellReferenceIndex,
    ) -> Result<Vec<Vec<ArrayNode>>, CalcResult> {
        match self.evaluate_node_in_context(node, cell) {
            CalcResult::Range { left, right } => {
                if left.sheet != right.sheet {
                    return Err(CalcResult::new_error(
                        Error::VALUE,
                        cell,
                        "Ranges are in different sheets".to_string(),
                    ));
                }
                let mut right = right;
                if let Ok(ws) = self.workbook.worksheet(left.sheet) {
                    let dimension = ws.dimension();
                    if right.row == LAST_ROW {
                        right.row = dimension.max_row.max(left.row);
                    }
                    if right.column == LAST_COLUMN {
                        right.column = dimension.max_column.max(left.column);
                    }
                }
                Ok(self.evaluate_range(left, right))
            }
            CalcResult::Array(a) => Ok(a),
            CalcResult::Number(n) => Ok(vec![vec![ArrayNode::Number(n)]]),
            CalcResult::String(s) => Ok(vec![vec![ArrayNode::String(s)]]),
            CalcResult::Boolean(b) => Ok(vec![vec![ArrayNode::Boolean(b)]]),
            error @ CalcResult::Error { .. } => Err(error),
            CalcResult::EmptyCell | CalcResult::EmptyArg => Ok(vec![vec![ArrayNode::Empty]]),
            CalcResult::Lambda(_) => Err(CalcResult::new_error(
                Error::VALUE,
                cell,
                "Unexpected lambda".to_string(),
            )),
        }
    }

    /// `=GROUPBY(row_fields, values, function, [field_headers], [total_depth],
    /// [sort_order], [filter_array], [field_relationship])`
    pub(crate) fn fn_groupby(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        if !(3..=8).contains(&args.len()) {
            return CalcResult::new_args_number_error(cell);
        }
        match self.groupby(args, cell) {
            Ok(r) | Err(r) => r,
        }
    }

    fn groupby(
        &mut self,
        args: &[Node],
        cell: CellReferenceIndex,
    ) -> Result<CalcResult, CalcResult> {
        let keys = self.summary_grid(&args[0], cell)?;
        let values = self.summary_grid(&args[1], cell)?;
        let (function, two) = self.summary_function(&args[2], cell)?;
        let headers = self.summary_option(args.get(3), cell)?;
        let width = keys.first().map_or(0, |r| r.len());
        let value_width = values.first().map_or(0, |r| r.len());
        let mut grids = vec![keys, values];
        let (names, show, rows) = self.summary_data(&mut grids, headers, args.get(6), cell)?;
        let table = self.summary_option(args.get(7), cell)?.unwrap_or(0) == 1;
        let depth =
            self.summary_option(args.get(4), cell)?
                .unwrap_or(if table { 1 } else { width as i32 });
        let sort = self.summary_sort(args.get(5), width + value_width, cell)?;
        let [keys, values] = [grids[0].clone(), grids[1].clone()];
        let summary = Summary {
            function,
            two,
            values,
            all: rows.clone(),
        };
        let layout = Layout {
            width,
            depth,
            sort: &sort,
            table,
        };
        let lines = self.summary_lines(&keys, &rows, &layout, &summary, cell);
        let mut out = Vec::new();
        if show {
            let mut header = names[0].clone();
            header.extend(names[1].iter().cloned());
            out.push(header.into_iter().map(shown).collect());
        }
        for line in lines {
            let mut row: Vec<ArrayNode> = line.keys.into_iter().map(shown).collect();
            for column in 0..value_width {
                row.push(self.summarize(&summary, column, &line.rows, &summary.all, cell));
            }
            out.push(row);
        }
        Ok(CalcResult::Array(out))
    }

    /// `=PIVOTBY(row_fields, col_fields, values, function, [field_headers],
    /// [row_total_depth], [row_sort_order], [col_total_depth],
    /// [col_sort_order], [filter_array], [relative_to])`
    pub(crate) fn fn_pivotby(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        if !(4..=11).contains(&args.len()) {
            return CalcResult::new_args_number_error(cell);
        }
        match self.pivotby(args, cell) {
            Ok(r) | Err(r) => r,
        }
    }

    fn pivotby(
        &mut self,
        args: &[Node],
        cell: CellReferenceIndex,
    ) -> Result<CalcResult, CalcResult> {
        let row_keys = self.summary_grid(&args[0], cell)?;
        let col_keys = self.summary_grid(&args[1], cell)?;
        let values = self.summary_grid(&args[2], cell)?;
        let (function, two) = self.summary_function(&args[3], cell)?;
        let headers = self.summary_option(args.get(4), cell)?;
        let row_width = row_keys.first().map_or(0, |r| r.len());
        let col_width = col_keys.first().map_or(0, |r| r.len());
        let value_width = values.first().map_or(0, |r| r.len());
        let mut grids = vec![row_keys, col_keys, values];
        let (names, show, rows) = self.summary_data(&mut grids, headers, args.get(9), cell)?;
        let row_depth = self
            .summary_option(args.get(5), cell)?
            .unwrap_or(row_width as i32);
        let row_sort = self.summary_sort(args.get(6), row_width + value_width, cell)?;
        let col_depth = self
            .summary_option(args.get(7), cell)?
            .unwrap_or(col_width as i32);
        let col_sort = self.summary_sort(args.get(8), col_width + value_width, cell)?;
        let relative = self.summary_option(args.get(10), cell)?.unwrap_or(0);
        if !(0..=4).contains(&relative) {
            return Err(CalcResult::new_error(
                Error::VALUE,
                cell,
                "relative_to must be 0 to 4".to_string(),
            ));
        }
        let summary = Summary {
            function,
            two,
            values: grids[2].clone(),
            all: rows.clone(),
        };
        let row_lines = self.summary_lines(
            &grids[0],
            &rows,
            &Layout {
                width: row_width,
                depth: row_depth,
                sort: &row_sort,
                table: false,
            },
            &summary,
            cell,
        );
        let col_lines = self.summary_lines(
            &grids[1],
            &rows,
            &Layout {
                width: col_width,
                depth: col_depth,
                sort: &col_sort,
                table: false,
            },
            &summary,
            cell,
        );
        let height = grids[0].len();
        let membership: Vec<Vec<bool>> = col_lines
            .iter()
            .map(|line| {
                let mut m = vec![false; height];
                for r in &line.rows {
                    m[*r] = true;
                }
                m
            })
            .collect();
        let value_header = value_width > 1;
        let header_rows = col_width + usize::from(value_header);
        let mut out: Vec<Vec<ArrayNode>> = Vec::new();
        for h in 0..header_rows {
            let mut row: Vec<ArrayNode> = if show && h + 1 == header_rows {
                names[0].clone()
            } else {
                vec![ArrayNode::Empty; row_width]
            };
            for line in &col_lines {
                for v in 0..value_width {
                    let cell_value = if h < col_width {
                        if v == 0 {
                            line.keys.get(h).cloned().unwrap_or(ArrayNode::Empty)
                        } else {
                            ArrayNode::Empty
                        }
                    } else {
                        names[2].get(v).cloned().unwrap_or(ArrayNode::Empty)
                    };
                    row.push(cell_value);
                }
            }
            out.push(row.into_iter().map(shown).collect());
        }
        for line in &row_lines {
            let mut row: Vec<ArrayNode> = line.keys.iter().cloned().map(shown).collect();
            for (c, col_line) in col_lines.iter().enumerate() {
                let both: Vec<usize> = line
                    .rows
                    .iter()
                    .copied()
                    .filter(|r| membership[c][*r])
                    .collect();
                let whole: &[usize] = match relative {
                    1 | 4 => &line.rows,
                    2 => &summary.all,
                    _ => &col_line.rows,
                };
                for v in 0..value_width {
                    if both.is_empty() {
                        row.push(text(""));
                    } else {
                        let value = self.summarize(&summary, v, &both, whole, cell);
                        row.push(value);
                    }
                }
            }
            out.push(row);
        }
        Ok(CalcResult::Array(out))
    }
}
