//! The two conversions between the ordinal formula AST and the stable token stream.
//!
//! Both walks are iterative with an explicit stack: formula depth is author-controlled, so
//! recursing over a `Node` tree is a stack overflow waiting to happen.

use std::collections::BTreeMap;
use std::fmt;

use crate::collab::formula::{
    ArrayValue, FormulaError, LambdaParam, StableAxisRef, StableFormula, StableSheetRef,
    StableToken,
};
use crate::collab::model::{CollabModel, SheetIndexes, Stable};
use crate::collab::patch::{DefinedNameId, Patch, SheetId};
use crate::constants::{LAST_COLUMN, LAST_ROW};
use crate::expressions::parser::{ArrayNode, NamedVariable, Node};
use crate::expressions::token;
use crate::types::Position;

/// Why a `Node` has no stable form.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BindError {
    /// A reference names a sheet that does not exist, or a row/column off the materialized grid.
    UnboundReference,
    /// A name with no live defined-name entry to point at.
    UnresolvedDefinedName(String),
    /// Defensive: the emitted stream did not pass the validator.
    Invalid(FormulaError),
}

impl fmt::Display for BindError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BindError::UnboundReference => write!(f, "reference does not resolve to a live cell"),
            BindError::UnresolvedDefinedName(name) => {
                write!(f, "defined name \"{name}\" not found")
            }
            BindError::Invalid(err) => write!(f, "bound stream is not a formula: {err}"),
        }
    }
}

impl std::error::Error for BindError {}

/// Why a token stream has no ordinal form.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LowerError {
    Invalid(FormulaError),
    /// Defensive: a validated stream ran the operand stack dry.
    Corrupt,
    /// The host sheet the stream is lowered against does not exist.
    UnknownHostSheet(u32),
}

impl fmt::Display for LowerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LowerError::Invalid(err) => write!(f, "not a formula: {err}"),
            LowerError::Corrupt => write!(f, "validated stream underflowed the operand stack"),
            LowerError::UnknownHostSheet(sheet) => write!(f, "host sheet '{sheet}' does not exist"),
        }
    }
}

impl std::error::Error for LowerError {}

/// What one step of the bind walk owes: visit a node's children, or emit its own token.
enum Step<'a> {
    Visit(&'a Node),
    Emit(&'a Node),
}

/// What a `Wrong*` node's dead axis carries: absolute ordinal 0, which upstream both renders and
/// evaluates as `#REF!`.
const DEAD_AXIS: (bool, i32) = (true, 0);

/// Rows and columns that sheets have not materialized yet (but they were referenced by formulas).
/// Each `rows`/`cols` is a pair of (sheet-id, highest-referenced-row (or column)).
#[derive(Debug, Default)]
pub struct MintPlan {
    rows: BTreeMap<u32, i32>,
    cols: BTreeMap<u32, i32>,
}

impl MintPlan {
    fn note(axis: &mut BTreeMap<u32, i32>, at: u32, ordinal: i32) {
        let slot = axis.entry(at).or_default();
        *slot = (*slot).max(ordinal);
    }
}

/// Where a stream is lowered against: the cell whose offsets it resolves relative to, or — when
/// `absolute` — no cell at all, every reference coming back as an ordinal.
pub struct Host {
    sheet: u32,
    row: i32,
    column: i32,
    absolute: bool,
}

impl Host {
    /// Relative cell address point of reference (using `$` for cell address).
    pub fn relative(sheet: u32, row: i32, column: i32) -> Self {
        Host {
            sheet,
            row,
            column,
            absolute: false,
        }
    }

    /// Absolute cell address point of reference (e.g. `=A1`).
    pub fn absolute(sheet: u32) -> Self {
        Host {
            sheet,
            row: 1,
            column: 1,
            absolute: true,
        }
    }
}

impl CollabModel<'_> {
    /// The ordering context of worksheet `at`, if it is live.
    fn indexes(&self, at: u32) -> Option<&SheetIndexes> {
        self.workbook
            .worksheets
            .get(at as usize)
            .map(|ws| &ws.index)
    }

    fn sheet_id_at(&self, at: u32) -> Option<SheetId> {
        self.workbook
            .worksheets
            .get(at as usize)
            .map(|ws| ws.sheet_id)
    }

    fn position_of_sheet(&self, id: SheetId) -> Option<u32> {
        self.workbook
            .worksheets
            .iter()
            .position(|ws| ws.sheet_id == id)
            .map(|at| at as u32)
    }

    /// Binds a parsed formula authored in cell (`host_sheet`, `host_row`, `host_column`).
    ///
    /// If formula refers to row/column that doesn't exist yet, it will be recorded in `plan` and
    /// materialized later via [Patch::InsertRows]/[Patch::InsertColumns].
    pub fn bind_formula(
        &self,
        node: &Node,
        host_sheet: u32,
        host_row: i32,
        host_column: i32,
        plan: &mut MintPlan,
    ) -> Result<StableFormula, BindError> {
        let mut work = vec![Step::Visit(node)];
        let mut tokens = Vec::new();
        while let Some(step) = work.pop() {
            let node = match step {
                // Children are pushed last-first so the stack pops them left to right.
                Step::Visit(node) => {
                    work.push(Step::Emit(node));
                    match node {
                        Node::OpRangeKind { left, right }
                        | Node::OpConcatenateKind { left, right }
                        | Node::OpSumKind { left, right, .. }
                        | Node::OpProductKind { left, right, .. }
                        | Node::OpPowerKind { left, right }
                        | Node::CompareKind { left, right, .. } => {
                            work.push(Step::Visit(right));
                            work.push(Step::Visit(left));
                        }
                        Node::UnaryKind { right, .. } => work.push(Step::Visit(right)),
                        Node::ImplicitIntersection { child, .. }
                        | Node::SpillRangeOperator { child } => work.push(Step::Visit(child)),
                        Node::LambdaDefKind { body, .. } => work.push(Step::Visit(body)),
                        Node::FunctionKind { args, .. } | Node::NamedFunctionKind { args, .. } => {
                            work.extend(args.iter().rev().map(Step::Visit));
                        }
                        Node::LambdaCallKind { lambda, args } => {
                            work.extend(args.iter().rev().map(Step::Visit));
                            work.push(Step::Visit(lambda));
                        }
                        // Unparseable text is stored verbatim, alone.
                        Node::ParseErrorKind { formula, .. } => {
                            return StableFormula::new(vec![StableToken::RawText(formula.clone())])
                                .map_err(BindError::Invalid);
                        }
                        _ => {}
                    }
                    continue;
                }
                Step::Emit(node) => node,
            };
            tokens.push(self.bind_token(node, host_sheet, host_row, host_column, plan)?);
        }
        StableFormula::new(tokens).map_err(BindError::Invalid)
    }

    /// The `InsertRows`/`InsertColumns` to create rows/columns required by formula bind.
    pub(crate) fn mint_patches(&self, plan: &MintPlan) -> Vec<Patch> {
        let mut patches = Vec::new();
        for (&sheet_idx, &row_idx) in plan.rows.iter() {
            let (Some(index), Some(sheet)) = (self.indexes(sheet_idx), self.sheet_id_at(sheet_idx))
            else {
                continue;
            };
            let keys = index.rows.plan_virtual(row_idx as usize);
            if keys.is_empty() {
                continue;
            }
            patches.push(Patch::InsertRows { sheet, keys });
        }
        for (&sheet_idx, &col_idx) in plan.cols.iter() {
            let (Some(index), Some(sheet)) = (self.indexes(sheet_idx), self.sheet_id_at(sheet_idx))
            else {
                continue;
            };
            let keys = index.cols.plan_virtual(col_idx as usize);
            if keys.is_empty() {
                continue;
            }
            patches.push(Patch::InsertColumns { sheet, keys });
        }
        patches
    }

    /// The key naming the row/column at `ordinal` on sheet `at`, per axis. One on the grid that the
    /// index has not reached yet is minted, exactly as a write to that cell would mint it.
    fn bind_axis(
        &self,
        at: u32,
        is_row: bool,
        absolute: bool,
        offset: i32,
        host: i32,
        plan: &mut MintPlan,
    ) -> Result<StableAxisRef, BindError> {
        let index = self.indexes(at).ok_or(BindError::UnboundReference)?;
        let ordinal = if absolute { offset } else { host + offset };
        let stored = if is_row {
            Stable::row_at(index, ordinal)
        } else {
            Stable::col_at(index, ordinal)
        };
        let key = match stored {
            Some(key) => key,
            None => {
                let (axis, limit, planned) = match is_row {
                    true => (&mut plan.rows, LAST_ROW, &index.rows),
                    false => (&mut plan.cols, LAST_COLUMN, &index.cols),
                };
                if ordinal > limit || ordinal < 1 {
                    return Err(BindError::UnboundReference);
                }
                // Planning is deterministic, so the key an ordinal gets is the same however many
                // times, and from however far, the axis is planned out to it.
                let key = planned
                    .plan_virtual(ordinal as usize)
                    .pop()
                    .ok_or(BindError::UnboundReference)?;
                MintPlan::note(axis, at, ordinal);
                key
            }
        };
        Ok(StableAxisRef { key, absolute })
    }

    /// The sheet a reference targets: its position, and whether it keeps an explicit prefix.
    fn bind_sheet(
        &self,
        sheet_name: &Option<String>,
        sheet_index: u32,
        host_sheet: u32,
    ) -> Result<(StableSheetRef, u32), BindError> {
        match sheet_name {
            None => Ok((StableSheetRef::Current, host_sheet)),
            // The parser already resolved the name; an explicit prefix is kept even for the host
            // sheet, for display fidelity.
            Some(_) => {
                let id = self
                    .sheet_id_at(sheet_index)
                    .ok_or(BindError::UnboundReference)?;
                Ok((StableSheetRef::Sheet(id), sheet_index))
            }
        }
    }

    fn bind_token(
        &self,
        node: &Node,
        host_sheet: u32,
        host_row: i32,
        host_column: i32,
        plan: &mut MintPlan,
    ) -> Result<StableToken, BindError> {
        Ok(match node {
            Node::BooleanKind(value) => StableToken::Boolean(*value),
            Node::NumberKind(value) => StableToken::Number(*value),
            Node::StringKind(value) => StableToken::String(value.clone()),
            Node::ReferenceKind {
                sheet_name,
                sheet_index,
                absolute_row,
                absolute_column,
                row,
                column,
            } => {
                let (sheet, at) = self.bind_sheet(sheet_name, *sheet_index, host_sheet)?;
                StableToken::CellRef {
                    sheet,
                    row: self.bind_axis(at, true, *absolute_row, *row, host_row, plan)?,
                    column: self.bind_axis(
                        at,
                        false,
                        *absolute_column,
                        *column,
                        host_column,
                        plan,
                    )?,
                }
            }
            Node::RangeKind {
                sheet_name,
                sheet_index,
                absolute_row1,
                absolute_column1,
                row1,
                column1,
                absolute_row2,
                absolute_column2,
                row2,
                column2,
            } => {
                let (sheet, at) = self.bind_sheet(sheet_name, *sheet_index, host_sheet)?;
                StableToken::RangeRef {
                    sheet,
                    row1: self.bind_axis(at, true, *absolute_row1, *row1, host_row, plan)?,
                    column1: self.bind_axis(
                        at,
                        false,
                        *absolute_column1,
                        *column1,
                        host_column,
                        plan,
                    )?,
                    row2: self.bind_axis(at, true, *absolute_row2, *row2, host_row, plan)?,
                    column2: self.bind_axis(
                        at,
                        false,
                        *absolute_column2,
                        *column2,
                        host_column,
                        plan,
                    )?,
                }
            }
            // Upstream keeps these and evaluates them to #REF!; a stable stream has no way to say
            // "unknown sheet", so authoring one is rejected instead.
            Node::WrongReferenceKind { .. } | Node::WrongRangeKind { .. } => {
                return Err(BindError::UnboundReference)
            }
            Node::OpRangeKind { .. } => StableToken::OpRange,
            Node::OpConcatenateKind { .. } => StableToken::OpConcatenate,
            Node::OpPowerKind { .. } => StableToken::OpPower,
            Node::OpSumKind { kind, .. } => StableToken::OpSum(kind.clone()),
            Node::OpProductKind { kind, .. } => StableToken::OpProduct(kind.clone()),
            Node::CompareKind { kind, .. } => StableToken::Compare(kind.clone()),
            Node::UnaryKind { kind, .. } => StableToken::Unary(kind.clone()),
            Node::ImplicitIntersection { automatic, .. } => StableToken::ImplicitIntersection {
                automatic: *automatic,
            },
            Node::SpillRangeOperator { .. } => StableToken::SpillRange,
            Node::FunctionKind { kind, args } => StableToken::Function {
                kind: kind.clone(),
                argc: args.len() as u16,
            },
            // The parser's `id` is a resolution artifact of the last evaluation, never authored.
            Node::NamedFunctionKind { name, args, .. } => StableToken::NamedFunction {
                name: name.clone(),
                argc: args.len() as u16,
            },
            Node::NamedVariableKind { name, .. } => StableToken::NamedVariable(name.clone()),
            Node::LambdaDefKind { parameters, .. } => StableToken::LambdaDef {
                parameters: parameters
                    .iter()
                    .map(|p| LambdaParam {
                        name: p.name.clone(),
                        optional: p.is_optional,
                    })
                    .collect(),
            },
            Node::LambdaCallKind { args, .. } => StableToken::LambdaCall {
                argc: args.len() as u16,
            },
            Node::ArrayKind(rows) => StableToken::Array(
                rows.iter()
                    .map(|row| row.iter().map(ArrayValue::from).collect())
                    .collect(),
            ),
            Node::DefinedNameKind((name, scope, _)) => {
                let scope = match scope {
                    Some(at) => Some(
                        self.sheet_id_at(*at)
                            .ok_or_else(|| BindError::UnresolvedDefinedName(name.clone()))?,
                    ),
                    None => None,
                };
                let id = self
                    .defined_name_id_of(scope, name)
                    .ok_or_else(|| BindError::UnresolvedDefinedName(name.clone()))?;
                StableToken::DefinedName(id)
            }
            Node::TableNameKind(name) => StableToken::TableName(name.clone()),
            Node::ErrorKind(kind) => StableToken::Error(kind.clone()),
            Node::EmptyArgKind => StableToken::EmptyArg,
            // Handled by the walk, which stops at the first one.
            Node::ParseErrorKind { formula, .. } => StableToken::RawText(formula.clone()),
        })
    }

    /// Rebuilds the ordinal AST for a formula, anchored as `host` says.
    ///
    /// References whose sheet or key no longer resolves come back as the `Wrong*` nodes upstream
    /// evaluates to `#REF!`: displacement happens here, not by rewriting stored formulas.
    pub fn lower(&self, formula: &StableFormula, host: &Host) -> Result<Node, LowerError> {
        StableFormula::validate(formula.tokens()).map_err(LowerError::Invalid)?;
        if self.indexes(host.sheet).is_none() {
            return Err(LowerError::UnknownHostSheet(host.sheet));
        }
        let mut stack: Vec<Node> = Vec::new();
        for token in formula.tokens() {
            let node = self.lower_token(token, &mut stack, host)?;
            stack.push(node);
        }
        // The validator guarantees exactly one value is left.
        stack.pop().ok_or(LowerError::Corrupt)
    }

    /// Pops `n` operands, oldest first.
    fn pop_args(stack: &mut Vec<Node>, n: u16) -> Result<Vec<Node>, LowerError> {
        let at = stack
            .len()
            .checked_sub(n as usize)
            .ok_or(LowerError::Corrupt)?;
        Ok(stack.split_off(at))
    }

    fn pop_one(stack: &mut Vec<Node>) -> Result<Box<Node>, LowerError> {
        stack.pop().map(Box::new).ok_or(LowerError::Corrupt)
    }

    /// The sheet a reference names: how it renders, and where it sits — `None` once it is deleted,
    /// which leaves its keys with no index to resolve against.
    fn lower_sheet(
        &self,
        sheet: &StableSheetRef,
        host_sheet: u32,
    ) -> (Option<String>, Option<u32>) {
        let StableSheetRef::Sheet(id) = sheet else {
            return (None, Some(host_sheet));
        };
        match self.position_of_sheet(*id) {
            Some(at) => (
                Some(self.workbook.worksheets[at as usize].name.clone()),
                Some(at),
            ),
            // The name register outlives the sheet, so a revived id renders as it did before.
            None => (
                self.workbook
                    .meta
                    .sheet_names
                    .get(id)
                    .map(|name| name.value.clone()),
                None,
            ),
        }
    }

    /// An axis' `(absolute, value)` pair — an ordinal when absolute, else an offset from the host.
    /// `None` once the key is gone from the index.
    fn lower_axis(
        &self,
        at: u32,
        is_row: bool,
        axis: &StableAxisRef,
        host: &Host,
    ) -> Option<(bool, i32)> {
        let index = self.indexes(at)?;
        let ordinal = if is_row {
            Stable::row_ordinal(index, &axis.key)?
        } else {
            Stable::col_ordinal(index, &axis.key)?
        };
        Some(match axis.absolute || host.absolute {
            true => (true, ordinal),
            false => (false, ordinal - if is_row { host.row } else { host.column }),
        })
    }

    fn lower_token(
        &self,
        token: &StableToken,
        stack: &mut Vec<Node>,
        host: &Host,
    ) -> Result<Node, LowerError> {
        Ok(match token {
            StableToken::Boolean(value) => Node::BooleanKind(*value),
            StableToken::Number(value) => Node::NumberKind(*value),
            StableToken::String(value) => Node::StringKind(value.clone()),
            StableToken::CellRef { sheet, row, column } => {
                let (name, at) = self.lower_sheet(sheet, host.sheet);
                let (r, c) = match at {
                    Some(at) => (
                        self.lower_axis(at, true, row, host),
                        self.lower_axis(at, false, column, host),
                    ),
                    None => (None, None),
                };
                match (at, r, c) {
                    (Some(at), Some((absolute_row, row)), Some((absolute_column, column))) => {
                        Node::ReferenceKind {
                            sheet_name: name,
                            sheet_index: at,
                            absolute_row,
                            absolute_column,
                            row,
                            column,
                        }
                    }
                    (_, r, c) => {
                        let (absolute_row, row) = r.unwrap_or(DEAD_AXIS);
                        let (absolute_column, column) = c.unwrap_or(DEAD_AXIS);
                        Node::WrongReferenceKind {
                            sheet_name: name,
                            absolute_row,
                            absolute_column,
                            row,
                            column,
                        }
                    }
                }
            }
            StableToken::RangeRef {
                sheet,
                row1,
                column1,
                row2,
                column2,
            } => {
                let (name, at) = self.lower_sheet(sheet, host.sheet);
                let (r1, c1, r2, c2) = match at {
                    Some(at) => (
                        self.lower_axis(at, true, row1, host),
                        self.lower_axis(at, false, column1, host),
                        self.lower_axis(at, true, row2, host),
                        self.lower_axis(at, false, column2, host),
                    ),
                    None => (None, None, None, None),
                };
                let (absolute_row1, row1) = r1.unwrap_or(DEAD_AXIS);
                let (absolute_column1, column1) = c1.unwrap_or(DEAD_AXIS);
                let (absolute_row2, row2) = r2.unwrap_or(DEAD_AXIS);
                let (absolute_column2, column2) = c2.unwrap_or(DEAD_AXIS);
                // One dead corner poisons the whole rectangle.
                if let (Some(at), true) = (at, [r1, c1, r2, c2].iter().all(Option::is_some)) {
                    Node::RangeKind {
                        sheet_name: name,
                        sheet_index: at,
                        absolute_row1,
                        absolute_column1,
                        row1,
                        column1,
                        absolute_row2,
                        absolute_column2,
                        row2,
                        column2,
                    }
                } else {
                    Node::WrongRangeKind {
                        sheet_name: name,
                        absolute_row1,
                        absolute_column1,
                        row1,
                        column1,
                        absolute_row2,
                        absolute_column2,
                        row2,
                        column2,
                    }
                }
            }
            StableToken::DefinedName(id) => self.lower_defined_name(*id),
            StableToken::TableName(name) => Node::TableNameKind(name.clone()),
            StableToken::NamedVariable(name) => Node::NamedVariableKind {
                name: name.clone(),
                id: None,
            },
            StableToken::Error(kind) => Node::ErrorKind(kind.clone()),
            StableToken::EmptyArg => Node::EmptyArgKind,
            StableToken::Array(rows) => Node::ArrayKind(
                rows.iter()
                    .map(|row| row.iter().map(ArrayNode::from).collect())
                    .collect(),
            ),
            // Verbatim text is what upstream holds for a formula it cannot parse.
            StableToken::RawText(text) => Node::ParseErrorKind {
                formula: text.clone(),
                message: "Unparseable formula".to_string(),
                position: 0,
                expecting: Vec::new(),
            },
            StableToken::Unary(kind) => Node::UnaryKind {
                kind: kind.clone(),
                right: Self::pop_one(stack)?,
            },
            StableToken::ImplicitIntersection { automatic } => Node::ImplicitIntersection {
                automatic: *automatic,
                child: Self::pop_one(stack)?,
            },
            StableToken::SpillRange => Node::SpillRangeOperator {
                child: Self::pop_one(stack)?,
            },
            StableToken::OpRange => {
                let right = Self::pop_one(stack)?;
                Node::OpRangeKind {
                    left: Self::pop_one(stack)?,
                    right,
                }
            }
            StableToken::OpConcatenate => {
                let right = Self::pop_one(stack)?;
                Node::OpConcatenateKind {
                    left: Self::pop_one(stack)?,
                    right,
                }
            }
            StableToken::OpPower => {
                let right = Self::pop_one(stack)?;
                Node::OpPowerKind {
                    left: Self::pop_one(stack)?,
                    right,
                }
            }
            StableToken::OpSum(kind) => {
                let right = Self::pop_one(stack)?;
                Node::OpSumKind {
                    kind: kind.clone(),
                    left: Self::pop_one(stack)?,
                    right,
                }
            }
            StableToken::OpProduct(kind) => {
                let right = Self::pop_one(stack)?;
                Node::OpProductKind {
                    kind: kind.clone(),
                    left: Self::pop_one(stack)?,
                    right,
                }
            }
            StableToken::Compare(kind) => {
                let right = Self::pop_one(stack)?;
                Node::CompareKind {
                    kind: kind.clone(),
                    left: Self::pop_one(stack)?,
                    right,
                }
            }
            StableToken::Function { kind, argc } => Node::FunctionKind {
                kind: kind.clone(),
                args: Self::pop_args(stack, *argc)?,
            },
            StableToken::NamedFunction { name, argc } => Node::NamedFunctionKind {
                id: None,
                name: name.clone(),
                args: Self::pop_args(stack, *argc)?,
            },
            StableToken::LambdaDef { parameters } => Node::LambdaDefKind {
                parameters: parameters
                    .iter()
                    .map(|p| NamedVariable {
                        name: p.name.clone(),
                        id: None,
                        is_optional: p.optional,
                    })
                    .collect(),
                body: Self::pop_one(stack)?,
            },
            StableToken::LambdaCall { argc } => {
                let args = Self::pop_args(stack, *argc)?;
                Node::LambdaCallKind {
                    lambda: Self::pop_one(stack)?,
                    args,
                }
            }
        })
    }

    /// A live name lowers to its current display name and formula; a deleted one to the node the
    /// parser builds for a name it does not know, which evaluates to `#NAME?`.
    fn lower_defined_name(&self, id: DefinedNameId) -> Node {
        let unknown = || match self.workbook.meta.defined_names.get(&id) {
            Some(state) => Node::NamedVariableKind {
                name: state.name.value.1.clone(),
                id: None,
            },
            // No register ever held this id: there is not even a name left to render.
            None => Node::ErrorKind(token::Error::NAME),
        };
        // Only live names are listed, and a live name always has a formula.
        let Some((_, sheet_id, name)) = self
            .defined_name_display()
            .into_iter()
            .find(|(entry, ..)| *entry == id)
        else {
            return unknown();
        };
        let scope = match sheet_id {
            Some(sheet) => match self.position_of_sheet(sheet) {
                Some(at) => Some(at),
                None => return unknown(),
            },
            None => None,
        };
        // The body text is the projection `normalize_defined_names` already lowered, so lowering a
        // name never re-lowers another name's stream.
        match self
            .workbook
            .defined_names
            .iter()
            .find(|dn| dn.sheet_id == sheet_id && dn.name == name)
        {
            Some(dn) => Node::DefinedNameKind((name, scope, dn.formula.clone())),
            None => unknown(),
        }
    }
}

#[cfg(test)]
mod test {
    #![allow(clippy::unwrap_used)]
    use super::*;
    use crate::collab::patch::invert_patches;
    use crate::expressions::parser::stringify::to_rc_format;
    use crate::expressions::token::OpUnary;
    use crate::types::Cell;
    use std::collections::HashSet;

    /// A workbook with two sheets, each materialized out to `rows` x `columns`.
    fn model(rows: i32, columns: i32) -> CollabModel<'static> {
        let mut model = CollabModel::new(1);
        for sheet in 0..2 {
            model.new_sheet();
            model
                .set_user_input(sheet, rows, columns, "1".to_string())
                .unwrap();
        }
        model
    }

    /// The parsed AST the model holds for the formula in a cell.
    fn node_at(model: &CollabModel<'_>, sheet: u32, row: i32, column: i32) -> Node {
        let ws = &model.workbook.worksheets[sheet as usize];
        let r = Stable::row_at(&ws.index, row).unwrap();
        let c = Stable::col_at(&ws.index, column).unwrap();
        match ws.sheet_data.get(&r).and_then(|row| row.get(&c)).unwrap() {
            Cell::CellFormula { f, .. } => {
                model.parsed_formulas[sheet as usize][*f as usize].0.clone()
            }
            other => panic!("not a formula cell: {other:?}"),
        }
    }

    /// A token's variant name, for coverage bookkeeping.
    fn kind_of(token: &StableToken) -> String {
        let name = format!("{token:?}");
        let end = name.find(['(', ' ']).unwrap_or(name.len());
        name[..end].to_string()
    }

    /// Binds against a model whose grid is already materialized, so nothing needs minting.
    fn bind(
        model: &CollabModel<'_>,
        node: &Node,
        sheet: u32,
        row: i32,
        column: i32,
    ) -> Result<StableFormula, BindError> {
        model.bind_formula(node, sheet, row, column, &mut MintPlan::default())
    }

    /// Bind, ship through bitcode, lower again.
    fn round_trip(model: &CollabModel<'_>, node: &Node, sheet: u32, row: i32, column: i32) -> Node {
        let bound = bind(model, node, sheet, row, column).unwrap();
        let decoded: StableFormula = bitcode::decode(&bitcode::encode(&bound)).unwrap();
        assert_eq!(decoded, bound);
        model
            .lower(&decoded, &Host::relative(sheet, row, column))
            .unwrap()
    }

    /// Every token kind that a formula text can produce survives bind → bytes → lower unchanged,
    /// from any host cell.
    #[test]
    fn bind_lower_round_trip() {
        let mut model = model(30, 15);
        model
            .new_defined_name("MyName", None, "Sheet1!$A$1")
            .unwrap();
        let mut seen: HashSet<String> = HashSet::new();
        let formulas = [
            "=1+2*3-4/5",
            "=\"a\"&\"b\"",
            "=2^3%",
            "=-A1",
            "=TRUE",
            "=A1+$B$2",
            "=$A1+A$2",
            "=Sheet2!C3",
            "=Sheet1!C3",
            "=SUM(Sheet2!A1:B3)",
            "=SUM(A$1:$B2)",
            "=IF(A1>0,\"y\",-A1%)",
            "=IF(A1<>1,A1,)",
            "=#VALUE!",
            "={TRUE,1;\"s\",#REF!}",
            "=@A1",
            "=A1#",
            "=LAMBDA(a,b,a*b)(2,3)",
            "=LET(x,1,x+A1)",
            "=myLambda(1,A1)",
            "=MyName*2",
            "=notAName",
            "=SUM(A1:A5)*MyName",
        ];
        for (host_row, host_column) in [(1, 1), (5, 3), (12, 7)] {
            for text in formulas {
                model
                    .set_user_input(0, host_row, host_column, text.to_string())
                    .unwrap();
                let node = node_at(&model, 0, host_row, host_column);
                let bound = bind(&model, &node, 0, host_row, host_column).unwrap();
                seen.extend(bound.tokens().iter().map(kind_of));
                let lowered = round_trip(&model, &node, 0, host_row, host_column);
                assert_eq!(
                    to_rc_format(&lowered),
                    to_rc_format(&node),
                    "{text} at ({host_row}, {host_column})"
                );
            }
        }

        // The battery is only worth what it covers. `OpRange` and `TableName` have no formula text
        // this model can produce (no tables, and the range operator needs a dynamic left operand).
        for kind in [
            "Boolean",
            "Number",
            "String",
            "CellRef",
            "RangeRef",
            "DefinedName",
            "NamedVariable",
            "Error",
            "EmptyArg",
            "Array",
            "Unary",
            "ImplicitIntersection",
            "SpillRange",
            "OpConcatenate",
            "OpPower",
            "OpSum",
            "OpProduct",
            "Compare",
            "Function",
            "NamedFunction",
            "LambdaDef",
            "LambdaCall",
        ] {
            assert!(seen.contains(kind), "no formula produced a {kind} token");
        }
    }

    /// A bound reference names rows and columns, not positions: structural edits move what it
    /// renders as with no rewrite, and deleting its target turns it into a `#REF!` node.
    #[test]
    fn references_follow_structural_ops() {
        let mut model = model(30, 15);
        let node = Node::OpSumKind {
            kind: crate::expressions::token::OpSum::Add,
            left: Box::new(Node::ReferenceKind {
                sheet_name: None,
                sheet_index: 0,
                absolute_row: false,
                absolute_column: false,
                row: 5 - 10,
                column: 1 - 2,
            }),
            right: Box::new(Node::ReferenceKind {
                sheet_name: None,
                sheet_index: 0,
                absolute_row: true,
                absolute_column: true,
                row: 6,
                column: 1,
            }),
        };
        let bound = bind(&model, &node, 0, 10, 2).unwrap();

        let refs = |model: &CollabModel<'_>, host_row: i32| match model
            .lower(&bound, &Host::relative(0, host_row, 2))
            .unwrap()
        {
            Node::OpSumKind { left, right, .. } => (*left, *right),
            other => panic!("not a sum: {other:?}"),
        };

        // Two rows above everything: both targets and the host slide down, so the relative offset
        // is unchanged and the absolute ordinal is not.
        model.insert_rows(0, 1, 2).unwrap();
        let (left, right) = refs(&model, 12);
        assert!(matches!(left, Node::ReferenceKind { row: -5, .. }));
        assert!(matches!(
            right,
            Node::ReferenceKind {
                row: 8,
                absolute_row: true,
                ..
            }
        ));

        // A row between the target and the host: only the host moves, so the offset grows.
        model.insert_rows(0, 8, 1).unwrap();
        let (left, right) = refs(&model, 13);
        assert!(matches!(left, Node::ReferenceKind { row: -6, .. }));
        assert!(matches!(right, Node::ReferenceKind { row: 9, .. }));

        // Deleting the row the left reference names leaves it with nothing to point at.
        model.delete_rows(0, 7, 1).unwrap();
        let (left, right) = refs(&model, 12);
        assert!(matches!(
            left,
            Node::WrongReferenceKind {
                row: 0,
                absolute_row: true,
                ..
            }
        ));
        assert!(matches!(right, Node::ReferenceKind { row: 8, .. }));

        // Columns work the same way.
        model.insert_columns(0, 1, 3).unwrap();
        let (_, right) = refs(&model, 12);
        assert!(matches!(right, Node::ReferenceKind { column: 4, .. }));

        // A cross-sheet reference outlives its sheet: it renders under the name the register kept,
        // and comes back for real once the sheet does.
        // A restore brings back only the rows and columns the sheet's content named, so the target
        // is a cell that holds something.
        model.set_user_input(1, 1, 1, "7".to_string()).unwrap();
        let cross = Node::ReferenceKind {
            sheet_name: Some("Sheet2".to_string()),
            sheet_index: 1,
            absolute_row: true,
            absolute_column: true,
            row: 1,
            column: 1,
        };
        let bound = bind(&model, &cross, 0, 1, 1).unwrap();
        assert_eq!(
            model.lower(&bound, &Host::relative(0, 1, 1)).unwrap(),
            cross
        );

        model.flush();
        model.delete_sheet(1).unwrap();
        assert_eq!(
            model.lower(&bound, &Host::relative(0, 1, 1)).unwrap(),
            Node::WrongReferenceKind {
                sheet_name: Some("Sheet2".to_string()),
                absolute_row: true,
                absolute_column: true,
                row: 0,
                column: 0,
            }
        );

        let undo: Vec<_> = model
            .flush()
            .iter()
            .rev()
            .flat_map(|commit| invert_patches(&commit.patches))
            .collect();
        model.commit_local(undo);
        assert_eq!(
            model.lower(&bound, &Host::relative(0, 1, 1)).unwrap(),
            cross
        );
    }

    /// What has no stable form at all, and what is stored verbatim instead.
    #[test]
    fn bind_rejects_and_rawtext() {
        let mut model = model(10, 5);
        model
            .new_defined_name("MyName", None, "Sheet1!$A$1")
            .unwrap();

        // A sheet name the parser could not resolve, which is what typing `=Nope!A1` parses to.
        // Authoring one is now refused outright — upstream stores it and evaluates it to `#REF!`.
        assert!(model
            .set_user_input(0, 1, 1, "=Nope!A1".to_string())
            .is_err());
        let unknown_sheet = Node::WrongReferenceKind {
            sheet_name: Some("Nope".to_string()),
            absolute_row: true,
            absolute_column: true,
            row: 1,
            column: 1,
        };

        let relative = |row: i32, column: i32| Node::ReferenceKind {
            sheet_name: None,
            sheet_index: 0,
            absolute_row: false,
            absolute_column: false,
            row,
            column,
        };
        let cases = [
            (unknown_sheet, BindError::UnboundReference),
            // Offsets landing off the grid, above it and past its end.
            (relative(-1, 0), BindError::UnboundReference),
            (relative(0, -1), BindError::UnboundReference),
            // Past the grid itself, not merely past what the index has reached.
            (relative(LAST_ROW, 0), BindError::UnboundReference),
            (relative(0, LAST_COLUMN), BindError::UnboundReference),
            (
                Node::DefinedNameKind(("Nope".to_string(), None, "=1".to_string())),
                BindError::UnresolvedDefinedName("Nope".to_string()),
            ),
        ];
        for (node, expected) in cases {
            assert_eq!(bind(&model, &node, 0, 1, 1).unwrap_err(), expected);
        }

        // Unparseable text is the whole stream, and comes back exactly as it went in.
        let raw = Node::ParseErrorKind {
            formula: "=this is not a formula".to_string(),
            message: "boom".to_string(),
            position: 5,
            expecting: Vec::new(),
        };
        let bound = bind(&model, &raw, 0, 1, 1).unwrap();
        assert_eq!(
            bound.tokens(),
            [StableToken::RawText("=this is not a formula".to_string())]
        );
        assert_eq!(
            to_rc_format(&round_trip(&model, &raw, 0, 1, 1)),
            to_rc_format(&raw)
        );

        // A deleted defined name lowers to the node the parser builds for a name it never heard
        // of, which evaluates to #NAME?.
        let name = Node::DefinedNameKind(("MyName".to_string(), None, "=Sheet1!$A$1".to_string()));
        let bound = bind(&model, &name, 0, 1, 1).unwrap();
        model.delete_defined_name("MyName", None).unwrap();
        assert_eq!(
            model.lower(&bound, &Host::relative(0, 1, 1)).unwrap(),
            Node::NamedVariableKind {
                name: "MyName".to_string(),
                id: None,
            }
        );
        // An id no register ever held has not even a name left.
        let unknown = StableFormula::new(vec![StableToken::DefinedName(7)]).unwrap();
        assert_eq!(
            model.lower(&unknown, &Host::relative(0, 1, 1)).unwrap(),
            Node::ErrorKind(token::Error::NAME)
        );
    }

    /// Depth the recursive-descent parser could never produce from text: both walks are iterative,
    /// so it is only bounded by `MAX_TOKENS`.
    #[test]
    fn deep_formula_no_overflow() {
        let model = model(5, 5);
        let depth = 8_000;
        let mut node = Node::NumberKind(1.0);
        for _ in 0..depth {
            node = Node::UnaryKind {
                kind: OpUnary::Minus,
                right: Box::new(node),
            };
        }
        let bound = bind(&model, &node, 0, 1, 1).unwrap();
        assert_eq!(bound.tokens().len(), depth + 1);
        let lowered = model.lower(&bound, &Host::relative(0, 1, 1)).unwrap();
        // Comparing or dropping the tree recurses, so it is walked and dismantled iteratively.
        let mut cursor = &lowered;
        let mut seen = 0;
        while let Node::UnaryKind { right, .. } = cursor {
            cursor = right;
            seen += 1;
        }
        assert_eq!((seen, cursor), (depth, &Node::NumberKind(1.0)));
        for tree in [node, lowered] {
            let mut tree = tree;
            while let Node::UnaryKind { right, .. } = tree {
                tree = *right;
            }
        }
    }
}
