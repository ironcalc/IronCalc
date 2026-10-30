//! Defined names and named functions: the types the API uses for them and
//! the methods of the [`Model`] that list, create, check, update and delete
//! them.

use bitcode::{Decode, Encode};
use serde::{Deserialize, Serialize};

use crate::{
    calc_result::{CalcResult, Range},
    expressions::{
        lexer::LexerMode,
        parser::{
            stringify::{
                rename_defined_name_in_node, to_english_string, to_rc_format, visit_free_names,
                NameUse,
            },
            NamedVariable, Node,
        },
        types::{CellReferenceIndex, CellReferenceRC},
        utils::{is_valid_identifier, parse_reference_a1, parse_reference_r1c1},
    },
    language::get_default_language,
    locale::get_default_locale,
    model::Model,
    types::{parameter_names, DefinedName, NamedFunction, NamedFunctionParameter},
    utils as common,
    workbook::{strip_user_function_prefix, USER_FUNCTION_PREFIX},
};

/// Excel limits defined-name comments to 255 characters.
pub const DEFINED_NAME_COMMENT_MAX_LENGTH: usize = 255;

fn validate_defined_name_comment(comment: &str) -> Result<(), String> {
    if comment.chars().count() > DEFINED_NAME_COMMENT_MAX_LENGTH {
        return Err(format!(
            "Comment: Comment must be at most {DEFINED_NAME_COMMENT_MAX_LENGTH} characters"
        ));
    }
    Ok(())
}

/// True if `name` would be read as a cell reference (`AB12`, `R1C1`)
fn looks_like_reference(name: &str) -> bool {
    parse_reference_a1(name).is_some() || parse_reference_r1c1(name).is_some()
}

/// The name of a named function without the `_xludf.` prefix, and whether it
/// had it. The prefix is matched ignoring case.
fn split_user_function_prefix(name: &str) -> (&str, bool) {
    let stripped = strip_user_function_prefix(name);
    (stripped, stripped.len() != name.len())
}

/// Named function and parameter descriptions have the same limit as comments
fn validate_description(description: &str) -> Result<(), String> {
    if description.chars().count() > DEFINED_NAME_COMMENT_MAX_LENGTH {
        return Err(format!(
            "Description: Description must be at most {DEFINED_NAME_COMMENT_MAX_LENGTH} characters"
        ));
    }
    Ok(())
}

/// A parsed formula for a defined name
#[derive(Clone)]
pub(crate) enum ParsedDefinedName {
    /// CellReference (`=C4`)
    CellReference(CellReferenceIndex),
    /// A Range (`=C4:D6`)
    RangeReference(Range),
    /// Any other formula: a constant (`=3.14`, `="Hello"`), an expression
    /// (`=SUM(Data!$E$2:$E$21)*2`) or a dynamic range (`=OFFSET(...)`).
    /// It is evaluated in the context of the cell that uses the name.
    Formula(Node),
    /// A formula that does not parse
    InvalidDefinedNameFormula,
}

/// A defined name as seen by the API, both when listing names and when
/// creating or updating one. `scope` is the index of the sheet the name is
/// local to, or `None` for a global name. The formula is in the active
/// language/locale (see [`Model::get_defined_name_list`]).
///
/// Most names are global, visible and have no comment, so
/// [`DefinedNameEntry::new`] takes just the name and the formula and the
/// rest can be chained:
///
/// ```
/// use ironcalc_base::DefinedNameEntry;
/// let entry = DefinedNameEntry::new("Rate", "Sheet1!$A$1").scope(0).hidden(true);
/// assert_eq!(entry.scope, Some(0));
/// assert!(entry.comment.is_empty());
/// ```
#[derive(Serialize, Deserialize, Default, Debug, PartialEq, Eq, Clone)]
pub struct DefinedNameEntry {
    /// The name, as typed by the user
    pub name: String,
    /// Index of the sheet the name is local to, `None` for a global name
    #[serde(default)]
    pub scope: Option<u32>,
    /// Formula in the active language/locale, without the leading `=`
    pub formula: String,
    /// Hidden names are not shown in the UI but still resolve in formulas
    #[serde(default)]
    pub hidden: bool,
    /// Free text describing the name, at most 255 characters. Empty means no comment.
    #[serde(default)]
    pub comment: String,
}

impl DefinedNameEntry {
    /// A global, visible name without a comment
    pub fn new(name: &str, formula: &str) -> Self {
        Self {
            name: name.to_string(),
            formula: formula.to_string(),
            ..Default::default()
        }
    }

    /// Makes the name local to the sheet with index `sheet`
    pub fn scope(mut self, sheet: u32) -> Self {
        self.scope = Some(sheet);
        self
    }

    /// Sets the hidden flag
    pub fn hidden(mut self, hidden: bool) -> Self {
        self.hidden = hidden;
        self
    }

    /// Sets the comment
    pub fn comment(mut self, comment: &str) -> Self {
        self.comment = comment.to_string();
        self
    }
}

/// What rewrites the body of a named function: it gets the model, the parsed
/// body, the sheet it is resolved in and the names of its parameters. See
/// [`Model::update_named_function_bodies`].
pub(crate) type BodyUpdate<'f, 'a> = dyn Fn(&Model<'a>, &mut Node, u32, &[String]) + 'f;

/// A named function as seen by the API, both when listing functions and when
/// creating or updating one. `scope` is the index of the sheet the function is
/// local to, or `None` for a workbook function. The formula is the body of the
/// function (what goes after the parameters in a `LAMBDA`), in the active
/// language/locale, without a leading `=`.
///
/// ```
/// use ironcalc_base::{types::NamedFunctionParameter, NamedFunctionEntry};
/// let entry = NamedFunctionEntry::new("Profit", "(revenue-cost)/revenue")
///     .parameters(vec![
///         NamedFunctionParameter::new("revenue"),
///         NamedFunctionParameter::new("cost").description("Total cost"),
///     ])
///     .scope(0);
/// assert_eq!(entry.scope, Some(0));
/// assert_eq!(entry.parameters.len(), 2);
/// ```
#[derive(Serialize, Deserialize, Encode, Decode, Default, Debug, PartialEq, Eq, Clone)]
pub struct NamedFunctionEntry {
    /// The name, as typed by the user. If a built-in function of the active
    /// language is called the same it has the `_xludf.` prefix, as it has
    /// when it is called in a formula: `_xludf.VERDADERO` in Spanish,
    /// `_xludf.NOW` in English. The prefix is accepted wherever the name of
    /// a named function is given.
    pub name: String,
    /// Index of the sheet the function is local to, `None` for a workbook function
    #[serde(default)]
    pub scope: Option<u32>,
    /// Hidden functions are not shown in the UI but can still be called in formulas
    #[serde(default)]
    pub hidden: bool,
    /// Free text describing the function, at most 255 characters. Empty means
    /// no description.
    #[serde(default)]
    pub description: String,
    /// The parameters, in order
    #[serde(default)]
    pub parameters: Vec<NamedFunctionParameter>,
    /// The body of the function in the active language/locale, without a
    /// leading `=`
    pub formula: String,
}

impl NamedFunctionEntry {
    /// A workbook, visible function without parameters or description
    pub fn new(name: &str, formula: &str) -> Self {
        Self {
            name: name.to_string(),
            formula: formula.to_string(),
            ..Default::default()
        }
    }

    /// Sets the name
    pub fn name(mut self, name: &str) -> Self {
        self.name = name.to_string();
        self
    }

    /// Makes the function local to the sheet with index `sheet`
    pub fn scope(mut self, sheet: u32) -> Self {
        self.scope = Some(sheet);
        self
    }

    /// Sets the hidden flag
    pub fn hidden(mut self, hidden: bool) -> Self {
        self.hidden = hidden;
        self
    }

    /// Sets the description
    pub fn description(mut self, description: &str) -> Self {
        self.description = description.to_string();
        self
    }

    /// Sets the parameters
    pub fn parameters(mut self, parameters: Vec<NamedFunctionParameter>) -> Self {
        self.parameters = parameters;
        self
    }
}

/// The problems found in a named function being edited, to show next to
/// each field. See [`Model::check_named_function`].
#[derive(Serialize, Deserialize, Default, Debug, PartialEq, Eq, Clone)]
pub struct NamedFunctionCheck {
    /// The error in the name, if any
    pub name: Option<String>,
    /// The error in each parameter, in the order they were given
    pub parameters: Vec<Option<String>>,
    /// The errors in the formula
    pub formula: Vec<String>,
    /// Warnings about the formula: the function can be saved regardless
    pub warnings: Vec<String>,
}

impl NamedFunctionCheck {
    /// True if the function can be saved: no errors, warnings aside
    pub fn is_valid(&self) -> bool {
        self.first_error().is_none()
    }

    /// The first error: in the name, then the parameters, then the formula
    pub fn first_error(&self) -> Option<String> {
        self.name
            .iter()
            .chain(self.parameters.iter().flatten())
            .chain(self.formula.iter())
            .next()
            .cloned()
    }
}

/// True if the formula has a reference that is not qualified with a sheet
/// name (`A1` rather than `Sheet1!A1`), or one to a sheet that does not exist.
/// A defined name has no sheet of its own, so such references are ambiguous.
fn has_unqualified_reference(node: &Node) -> bool {
    match node {
        Node::ReferenceKind { sheet_name, .. } | Node::RangeKind { sheet_name, .. } => {
            sheet_name.is_none()
        }
        Node::WrongReferenceKind { .. } | Node::WrongRangeKind { .. } => true,
        Node::OpRangeKind { left, right }
        | Node::OpConcatenateKind { left, right }
        | Node::OpSumKind { left, right, .. }
        | Node::OpProductKind { left, right, .. }
        | Node::OpPowerKind { left, right }
        | Node::CompareKind { left, right, .. } => {
            has_unqualified_reference(left) || has_unqualified_reference(right)
        }
        Node::UnaryKind { right, .. } => has_unqualified_reference(right),
        Node::ImplicitIntersection { child, .. } | Node::SpillRangeOperator { child } => {
            has_unqualified_reference(child)
        }
        Node::LambdaDefKind { body, .. } => has_unqualified_reference(body),
        Node::LambdaCallKind { lambda, args } => {
            has_unqualified_reference(lambda) || args.iter().any(has_unqualified_reference)
        }
        Node::FunctionKind { args, .. } | Node::NamedFunctionKind { args, .. } => {
            args.iter().any(has_unqualified_reference)
        }
        Node::BooleanKind(_)
        | Node::NumberKind(_)
        | Node::StringKind(_)
        | Node::ArrayKind(_)
        | Node::DefinedNameKind { .. }
        | Node::TableNameKind(_)
        | Node::NamedVariableKind { .. }
        | Node::ErrorKind(_)
        | Node::ParseErrorKind { .. }
        | Node::EmptyArgKind => false,
    }
}

impl<'a> Model<'a> {
    /// The sheet id of a scope, `None` for the workbook
    pub(crate) fn scope_sheet_id(&self, scope: Option<u32>) -> Result<Option<u32>, String> {
        match scope {
            Some(index) => match self.workbook.worksheet(index) {
                Ok(ws) => Ok(Some(ws.sheet_id)),
                Err(_) => Err("Scope: Invalid sheet index".to_string()),
            },
            None => Ok(None),
        }
    }

    /// The name of a named function as it is shown and typed in the active
    /// language. If a built-in function of the language is called the same
    /// it takes the `_xludf.` prefix, like its calls in a formula do:
    /// `VERDADERO` is `_xludf.VERDADERO` in Spanish, where it is `TRUE`.
    fn named_function_display_name(&self, name: &str) -> String {
        if self.language.functions.lookup(name).is_some() {
            format!("{USER_FUNCTION_PREFIX}{name}")
        } else {
            name.to_string()
        }
    }

    /// True if there is a defined name or a named function called `name` with
    /// scope `sheet_id`. Defined names and named functions share a namespace.
    pub(crate) fn name_exists(&self, name: &str, sheet_id: Option<u32>) -> bool {
        let name_upper = name.to_uppercase();
        self.workbook
            .defined_names
            .iter()
            .any(|df| df.name.to_uppercase() == name_upper && df.sheet_id == sheet_id)
            || self
                .workbook
                .named_functions
                .iter()
                .any(|nf| nf.name.to_uppercase() == name_upper && nf.sheet_id == sheet_id)
    }

    /// The named function `name` as seen from `scope`: the one local to that
    /// sheet if there is one, the workbook one otherwise.
    pub(crate) fn get_parsed_named_function(
        &self,
        name: &str,
        scope: Option<u32>,
    ) -> Option<&(Vec<NamedVariable>, Node)> {
        let name_lower = name.to_lowercase();
        self.parsed_named_functions
            .get(&(scope, name_lower.clone()))
            .or_else(|| self.parsed_named_functions.get(&(None, name_lower)))
    }

    /// A lambda value for the named function `name` as seen from `scope`
    pub(crate) fn named_function_lambda(
        &mut self,
        name: &str,
        scope: Option<u32>,
    ) -> Option<CalcResult> {
        let (parameters, body) = self.get_parsed_named_function(name, scope)?.clone();
        let lambda_id = self.get_next_lambda_id();
        self.lambdas.insert(lambda_id, (parameters, body));
        Some(CalcResult::Lambda(lambda_id))
    }

    /// Returns the defined name with `name` and `scope` (`None` is global,
    /// `Some(index)` is local to the sheet with that index).
    pub(crate) fn get_defined_name(
        &self,
        name: &str,
        scope: Option<u32>,
    ) -> Result<&DefinedName, String> {
        let name_upper = name.to_uppercase();
        let sheet_id = match scope {
            Some(index) => Some(self.workbook.worksheet(index)?.sheet_id),
            None => None,
        };
        self.workbook
            .defined_names
            .iter()
            .find(|df| df.name.to_uppercase() == name_upper && df.sheet_id == sheet_id)
            .ok_or_else(|| "Defined name not found".to_string())
    }

    // Returns the formula for a defined name
    pub(crate) fn get_defined_name_formula(
        &self,
        name: &str,
        scope: Option<u32>,
    ) -> Result<String, String> {
        Ok(self.get_defined_name(name, scope)?.formula.clone())
    }

    /// Returns the list of defined names.
    ///
    /// Formulas are stored internally in English; they are translated into the
    /// active language/locale for display.
    pub fn get_defined_name_list(&self) -> Vec<DefinedNameEntry> {
        let context = self.defined_name_context();
        let sheet_ids: Vec<u32> = self
            .workbook
            .worksheets
            .iter()
            .map(|s| s.sheet_id)
            .collect();
        self.workbook
            .defined_names
            .iter()
            .map(|dn| {
                let scope = dn
                    .sheet_id
                    .and_then(|sheet_id| sheet_ids.iter().position(|&id| id == sheet_id))
                    .map(|pos| pos as u32);
                let formula = self.internal_formula_to_display(&dn.formula, &context);
                DefinedNameEntry {
                    name: dn.name.clone(),
                    scope,
                    formula,
                    hidden: dn.hidden,
                    comment: dn.comment.clone(),
                }
            })
            .collect()
    }

    /// Returns the list of named functions.
    ///
    /// Bodies are stored internally in English; they are translated into the
    /// active language/locale for display.
    pub fn get_named_function_list(&self) -> Vec<NamedFunctionEntry> {
        self.workbook
            .named_functions
            .iter()
            .map(|function| self.named_function_entry(function, true))
            .collect()
    }

    /// Adds a new defined name. See [`DefinedNameEntry`] for the meaning of
    /// the fields.
    pub fn new_defined_name(&mut self, entry: DefinedNameEntry) -> Result<(), String> {
        let DefinedNameEntry {
            name,
            scope,
            formula,
            hidden,
            comment,
        } = entry;
        let sheet_id = self.is_valid_defined_name(&name, scope, &formula)?;
        validate_defined_name_comment(&comment)?;
        // Defined-name formulas are stored internally in English so they keep
        // working when the user switches language/locale.
        let context = self.defined_name_context();
        let internal_formula = self.user_formula_to_internal(&formula, &context)?;
        if let Some((parameters, body)) = self.split_lambda_formula(&internal_formula, &context) {
            // A name defined as a LAMBDA is a named function
            let function = NamedFunction {
                name: split_user_function_prefix(&name).0.to_string(),
                sheet_id,
                hidden,
                description: comment,
                parameters,
                formula: body,
            };
            self.validate_lambda_named_function(&function, &name, None)?;
            self.workbook.named_functions.push(function);
        } else {
            self.workbook.defined_names.push(DefinedName {
                name,
                formula: internal_formula,
                sheet_id,
                hidden,
                comment,
            });
        }
        self.reset_parsed_structures();

        Ok(())
    }

    /// A named function created from a defined name whose formula is a LAMBDA
    /// follows the rules of the named functions (see
    /// [`Model::check_named_function`]). `current` is the (name, scope) of
    /// the defined name it replaces, if any. `typed_name` is the name as the
    /// user gave it, that may have the `_xludf.` prefix. The errors are those
    /// of a defined name: prefixed with the field they are about.
    fn validate_lambda_named_function(
        &mut self,
        function: &NamedFunction,
        typed_name: &str,
        current: Option<(&str, Option<u32>)>,
    ) -> Result<(), String> {
        // The body is already in the internal (English) form
        let mut entry = self.named_function_entry(function, false);
        entry.name = typed_name.to_string();
        let check = self.check_named_function_entry(&entry, current, true);
        if let Some(error) = check.name {
            return Err(format!("Name: {error}"));
        }
        match check.first_error() {
            Some(error) => Err(format!("Formula: {error}")),
            None => Ok(()),
        }
    }

    /// If `formula`, in internal English form, is a LAMBDA definition returns
    /// its parameters and its body, the body in internal English form.
    fn split_lambda_formula(
        &mut self,
        formula: &str,
        context: &CellReferenceRC,
    ) -> Option<(Vec<NamedFunctionParameter>, String)> {
        let body = formula.trim().strip_prefix('=').unwrap_or(formula);
        match self.parse_internal_formula(body, context) {
            Node::LambdaDefKind { parameters, body } => Some((
                parameters
                    .iter()
                    .map(|p| NamedFunctionParameter::new(&p.name).optional(p.is_optional))
                    .collect(),
                to_english_string(&body, context),
            )),
            _ => None,
        }
    }

    /// The context used to parse/stringify a name local to `scope`: A1 of that
    /// sheet, or of the first sheet for a workbook name.
    pub(crate) fn name_context(&self, scope: Option<u32>) -> CellReferenceRC {
        let sheet = match scope {
            Some(index) => self.workbook.worksheets.get(index as usize),
            None => self.workbook.worksheets.first(),
        }
        .map(|ws| ws.get_name())
        .unwrap_or_else(|| "Sheet1".to_string());
        CellReferenceRC {
            sheet,
            row: 1,
            column: 1,
        }
    }

    /// The scope, as a sheet index, of a name local to the sheet `sheet_id`
    pub(crate) fn scope_of(&self, sheet_id: Option<u32>) -> Option<u32> {
        sheet_id.and_then(|id| self.get_sheet_index_by_sheet_id(id))
    }

    /// A named function as seen by the API, with its name and its body in the
    /// active language/locale if `localized`, in the internal (English) form
    /// otherwise. See [`Model::named_function_display_name`].
    pub(crate) fn named_function_entry(
        &self,
        function: &NamedFunction,
        localized: bool,
    ) -> NamedFunctionEntry {
        let scope = self.scope_of(function.sheet_id);
        let formula = if localized {
            self.internal_body_to_display(
                &function.formula,
                &self.name_context(scope),
                &parameter_names(&function.parameters),
            )
        } else {
            function.formula.clone()
        };
        let name = if localized {
            self.named_function_display_name(&function.name)
        } else {
            function.name.clone()
        };
        NamedFunctionEntry {
            name,
            scope,
            hidden: function.hidden,
            description: function.description.clone(),
            parameters: function.parameters.clone(),
            formula,
        }
    }

    /// The function to store for an entry: the body goes in the internal
    /// (English) form, without the leading '='. If `internal` the body of the
    /// entry is already in that form (see [`Model::named_function_entry`])
    /// and is stored as it is: reading it in the language and locale of the
    /// user could change its meaning (`SUM(1,2)` is `SUM(1.2)` if the
    /// decimal separator is the comma).
    fn named_function_from_entry(
        &mut self,
        entry: NamedFunctionEntry,
        internal: bool,
    ) -> Result<NamedFunction, String> {
        let sheet_id = self.scope_sheet_id(entry.scope)?;
        let formula = if internal {
            entry.formula
        } else {
            let context = self.name_context(entry.scope);
            self.user_body_to_internal(
                &entry.formula,
                &context,
                &parameter_names(&entry.parameters),
            )?
        };
        Ok(NamedFunction {
            name: split_user_function_prefix(&entry.name).0.to_string(),
            sheet_id,
            hidden: entry.hidden,
            description: entry.description,
            parameters: entry.parameters,
            formula: formula.strip_prefix('=').unwrap_or(&formula).to_string(),
        })
    }

    /// The rules for a named function to create, or to replace `current`
    /// with: those of [`Model::check_named_function`] and the length of the
    /// descriptions.
    fn validate_named_function(
        &mut self,
        entry: &NamedFunctionEntry,
        current: Option<(&str, Option<u32>)>,
    ) -> Result<(), String> {
        if let Some(error) = self.check_named_function(entry, current).first_error() {
            return Err(error);
        }
        validate_description(&entry.description)?;
        for parameter in &entry.parameters {
            validate_description(&parameter.description)?;
        }
        Ok(())
    }

    /// Adds a new named function. See [`NamedFunctionEntry`] for the meaning
    /// of the fields.
    pub fn new_named_function(&mut self, entry: NamedFunctionEntry) -> Result<(), String> {
        self.validate_named_function(&entry, None)?;
        let function = self.named_function_from_entry(entry, false)?;
        self.workbook.named_functions.push(function);
        self.reset_parsed_structures();
        Ok(())
    }

    /// Adds a named function, with its body in the internal (English) form,
    /// without validating it: undo and redo restore functions that were
    /// valid when they were created.
    pub(crate) fn insert_named_function(
        &mut self,
        entry: NamedFunctionEntry,
    ) -> Result<(), String> {
        let function = self.named_function_from_entry(entry, true)?;
        self.workbook.named_functions.push(function);
        self.reset_parsed_structures();
        Ok(())
    }

    /// Deletes the named function `name` with scope `scope`
    pub fn delete_named_function(&mut self, name: &str, scope: Option<u32>) -> Result<(), String> {
        let index = self.named_function_index(name, scope)?;
        self.workbook.named_functions.remove(index);
        self.reset_parsed_structures();
        Ok(())
    }

    /// Returns the named function `name` with scope `scope` (`None` is
    /// global, `Some(index)` is local to the sheet with that index).
    pub(crate) fn get_named_function(
        &self,
        name: &str,
        scope: Option<u32>,
    ) -> Result<&NamedFunction, String> {
        let index = self.named_function_index(name, scope)?;
        self.workbook
            .named_functions
            .get(index)
            .ok_or_else(|| "Named function not found".to_string())
    }

    fn named_function_index(&self, name: &str, scope: Option<u32>) -> Result<usize, String> {
        let name = split_user_function_prefix(name).0;
        let sheet_id = match scope {
            Some(index) => Some(self.workbook.worksheet(index)?.sheet_id),
            None => None,
        };
        let name_upper = name.to_uppercase();
        self.workbook
            .named_functions
            .iter()
            .position(|nf| nf.name.to_uppercase() == name_upper && nf.sheet_id == sheet_id)
            .ok_or_else(|| "Named function not found".to_string())
    }

    /// Checks a named function being created, or being edited when `current`
    /// is the (name, scope) of the function it replaces. It reports every
    /// problem, field by field, with the messages to show. The function is
    /// valid when [`NamedFunctionCheck::is_valid`]: these are the rules
    /// [`Model::new_named_function`] and [`Model::update_named_function`]
    /// apply.
    pub fn check_named_function(
        &mut self,
        entry: &NamedFunctionEntry,
        current: Option<(&str, Option<u32>)>,
    ) -> NamedFunctionCheck {
        self.check_named_function_entry(entry, current, false)
    }

    /// [`Model::check_named_function`] for an entry whose body is in the
    /// active language/locale or, if `internal`, in the internal (English)
    /// form.
    fn check_named_function_entry(
        &mut self,
        entry: &NamedFunctionEntry,
        current: Option<(&str, Option<u32>)>,
        internal: bool,
    ) -> NamedFunctionCheck {
        let mut check = NamedFunctionCheck::default();
        let (name, has_prefix) = split_user_function_prefix(&entry.name);
        let name_upper = name.to_uppercase();
        let is_current_name = |candidate: &str, scope: Option<u32>| {
            current.is_some_and(|(current_name, current_scope)| {
                let current_name = split_user_function_prefix(current_name).0;
                current_name.to_uppercase() == candidate.to_uppercase() && current_scope == scope
            })
        };

        // Name
        if name.trim().is_empty() {
            check.name = Some("Name is required.".to_string());
        } else if looks_like_reference(name) {
            check.name = Some("This name looks like a cell reference (like AB12).".to_string());
        } else if !is_valid_identifier(name) {
            check.name =
                Some("Use letters, numbers, and underscores, starting with a letter.".to_string());
        } else if !has_prefix && self.language.functions.lookup(name).is_some() {
            // A built-in function of the active language: the name is fine
            // with the prefix, as it is called in a formula. A built-in
            // function of another language needs nothing here, it takes the
            // prefix where that language is used.
            check.name = Some(format!("{name_upper} is already a built-in function."));
        } else {
            match self.scope_sheet_id(entry.scope) {
                Ok(sheet_id) => {
                    if self.name_exists(name, sheet_id) && !is_current_name(name, entry.scope) {
                        check.name = Some(format!(
                            "A function or defined name called {name_upper} already exists."
                        ));
                    }
                }
                Err(_) => {
                    check.name = Some("The sheet of the function does not exist.".to_string())
                }
            }
        }

        // Parameters
        let mut seen: Vec<String> = Vec::new();
        // Arguments are matched by position, so the optional ones go last
        let mut seen_optional = false;
        for parameter in &entry.parameters {
            let parameter_name = parameter.name.as_str();
            let lower = parameter_name.to_lowercase();
            let error = if parameter_name.trim().is_empty() {
                Some("Each argument needs a name.".to_string())
            } else if looks_like_reference(parameter_name) {
                Some("This name looks like a cell reference. Try another.".to_string())
            } else if !is_valid_identifier(parameter_name) {
                Some(
                    "Argument names must start with a letter and contain only letters, numbers, and underscores."
                        .to_string(),
                )
            } else if seen.contains(&lower) {
                Some("Each argument needs a different name.".to_string())
            } else if seen_optional && !parameter.optional {
                Some("Arguments after an optional one must be optional too.".to_string())
            } else {
                None
            };
            seen_optional |= parameter.optional;
            seen.push(lower);
            check.parameters.push(error);
        }

        // Formula
        let context = self.name_context(entry.scope);
        let body = entry.formula.trim();
        let body = body.strip_prefix('=').unwrap_or(body).trim();
        if body.is_empty() {
            check.formula.push("Formula is required.".to_string());
            return check;
        }
        let bound = parameter_names(&entry.parameters);
        let mut node = if internal {
            self.parse_internal_body(body, &context, &bound)
        } else {
            let node = self.parser.parse_with_bound_names(body, &context, &bound);
            if let Node::ParseErrorKind { .. } = node {
                self.parse_internal_body(body, &context, &bound)
            } else {
                node
            }
        };
        if let Node::ParseErrorKind { .. } = node {
            check
                .formula
                .push("There's an error in this formula.".to_string());
            return check;
        }
        if has_unqualified_reference(&node) {
            check
                .formula
                .push("References in the formula need a sheet (like Sheet1!A1).".to_string());
        }
        let parameter_names: Vec<&str> = entry.parameters.iter().map(|p| p.name.as_str()).collect();
        let mut used: Vec<String> = Vec::new();
        let mut unknown_variables: Vec<String> = Vec::new();
        let mut unknown_functions: Vec<String> = Vec::new();
        visit_free_names(&mut node, &mut Vec::new(), &mut |free_name, name_use| {
            let free_name = free_name.as_str();
            // Matching parameters is case-sensitive, like the evaluation
            if parameter_names.contains(&free_name) {
                if !used.iter().any(|n| n == free_name) {
                    used.push(free_name.to_string());
                }
                return;
            }
            // A name qualified with a sheet is resolved in that sheet. It may
            // be a defined name: the one local to the sheet or the workbook one.
            // Any other name is resolved in the sheet of the cell that calls
            // the function: the sheet a local function belongs to, any sheet
            // for a workbook function, which can then only rely on the
            // workbook functions.
            let sheet = match name_use {
                NameUse::Qualified(sheet) => Some(sheet),
                NameUse::Variable | NameUse::Call => entry.scope,
            };
            let name_lower = free_name.to_lowercase();
            let is_defined_name = match name_use {
                NameUse::Qualified(sheet) => {
                    self.parsed_defined_names
                        .contains_key(&(Some(sheet), name_lower.clone()))
                        || self
                            .parsed_defined_names
                            .contains_key(&(None, name_lower.clone()))
                }
                NameUse::Variable | NameUse::Call => false,
            };
            let is_function = self.get_parsed_named_function(free_name, sheet).is_some()
                || free_name.to_uppercase() == name_upper;
            if is_defined_name || is_function {
                return;
            }
            let list = match name_use {
                NameUse::Variable => &mut unknown_variables,
                NameUse::Call | NameUse::Qualified(_) => &mut unknown_functions,
            };
            if !list.iter().any(|n| n == free_name) {
                list.push(free_name.to_string());
            }
        });
        for variable in &unknown_variables {
            check.formula.push(format!(
                "\"{variable}\" isn't an argument. Add it above or check the spelling."
            ));
        }
        for function in &unknown_functions {
            check
                .formula
                .push(format!("\"{function}\" isn't a known function."));
        }
        for parameter in parameter_names {
            if !parameter.trim().is_empty() && !used.iter().any(|n| n == parameter) {
                check
                    .warnings
                    .push(format!("\"{parameter}\" isn't used in the formula."));
            }
        }
        check
    }

    /// The number of cells whose formula uses the named function `name` with
    /// scope `scope`: calls it, passes it as a value or qualifies it with a
    /// sheet. A cell also uses it if it uses a named function that does.
    pub fn named_function_usage_count(
        &self,
        name: &str,
        scope: Option<u32>,
    ) -> Result<usize, String> {
        // Make sure it exists
        self.named_function_index(name, scope)?;
        let name = split_user_function_prefix(name).0;
        // The function and the functions that use it, directly or through
        // other functions, as (scope, lowercase name)
        let mut targets: Vec<(Option<u32>, String)> = vec![(scope, name.to_lowercase())];
        loop {
            let mut users = Vec::new();
            for (key, (parameters, body)) in &self.parsed_named_functions {
                if targets.contains(key) {
                    continue;
                }
                let mut found = false;
                let mut body = body.clone();
                let mut bound: Vec<String> = parameters.iter().map(|p| p.name.clone()).collect();
                visit_free_names(&mut body, &mut bound, &mut |free_name, name_use| {
                    found |= targets.iter().any(|(target_scope, target_name)| {
                        self.reference_is_named_function(
                            free_name,
                            name_use,
                            key.0.unwrap_or(0),
                            (*target_scope, target_name),
                        )
                    });
                });
                if found {
                    users.push(key.clone());
                }
            }
            if users.is_empty() {
                break;
            }
            targets.extend(users);
        }
        let mut count = 0;
        for (sheet_index, worksheet) in self.workbook.worksheets.iter().enumerate() {
            let Some(parsed_formulas) = self.parsed_formulas.get(sheet_index) else {
                continue;
            };
            // Whether each formula of the sheet uses the function
            let uses: Vec<bool> = parsed_formulas
                .iter()
                .map(|(node, _)| {
                    let mut found = false;
                    let mut node = Node::clone(node);
                    visit_free_names(&mut node, &mut Vec::new(), &mut |free_name, name_use| {
                        found |= targets.iter().any(|(target_scope, target_name)| {
                            self.reference_is_named_function(
                                free_name,
                                name_use,
                                sheet_index as u32,
                                (*target_scope, target_name),
                            )
                        });
                    });
                    found
                })
                .collect();
            count += worksheet
                .sheet_data
                .cells()
                .filter(|(_, _, cell)| {
                    cell.get_formula()
                        .is_some_and(|index| uses.get(index as usize) == Some(&true))
                })
                .count();
        }
        Ok(count)
    }

    /// Replaces the named function `name` with scope `scope` by `new`: every
    /// field of the function takes the value in `new`. A renamed function is
    /// renamed in every cell formula and in the bodies of the named functions
    /// that call it, its own body included.
    pub fn update_named_function(
        &mut self,
        name: &str,
        scope: Option<u32>,
        new: NamedFunctionEntry,
    ) -> Result<(), String> {
        self.named_function_index(name, scope)?;
        self.validate_named_function(&new, Some((name, scope)))?;
        self.replace_named_function(name, scope, new, false)
    }

    /// [`Model::update_named_function`] without validating `new`. Undo and
    /// redo use it with the body of `new` in the internal (English) form:
    /// `internal`.
    pub(crate) fn replace_named_function(
        &mut self,
        name: &str,
        scope: Option<u32>,
        new: NamedFunctionEntry,
        internal: bool,
    ) -> Result<(), String> {
        let index = self.named_function_index(name, scope)?;
        let old_name = self.workbook.named_functions[index].name.clone();
        let context = self.name_context(new.scope);
        let context_sheet = new.scope.unwrap_or(0);
        let mut function = self.named_function_from_entry(new, internal)?;
        if function.name != old_name {
            self.rename_named_function_in_formulas(&old_name, scope, &function.name);
            // The new body may still call the function by its old name
            let bound = parameter_names(&function.parameters);
            let mut body = self.parse_internal_body(&function.formula, &context, &bound);
            self.rename_named_function_in_node(
                &mut body,
                context_sheet,
                &bound,
                (scope, &old_name),
                &function.name,
            );
            function.formula = to_english_string(&body, &context);
        }
        self.workbook.named_functions[index] = function;
        self.reset_parsed_structures();
        Ok(())
    }

    /// True if a reference to `name` in a formula of `context_sheet` resolves
    /// to the named function `target` (scope, name).
    fn reference_is_named_function(
        &self,
        name: &str,
        name_use: NameUse,
        context_sheet: u32,
        target: (Option<u32>, &str),
    ) -> bool {
        let name_lower = name.to_lowercase();
        if target.1.to_lowercase() != name_lower {
            return false;
        }
        let qualified = match name_use {
            NameUse::Qualified(sheet) => Some(sheet),
            NameUse::Variable | NameUse::Call => None,
        };
        let sheet = qualified.unwrap_or(context_sheet);
        // A qualified name is the defined name local to that sheet, if any
        if qualified.is_some()
            && self
                .parsed_defined_names
                .contains_key(&(Some(sheet), name_lower.clone()))
        {
            return false;
        }
        // The function local to the sheet takes precedence over the workbook one
        let resolved = if self
            .parsed_named_functions
            .contains_key(&(Some(sheet), name_lower.clone()))
        {
            Some(sheet)
        } else if self
            .parsed_named_functions
            .contains_key(&(None, name_lower))
        {
            None
        } else {
            return false;
        };
        resolved == target.0
    }

    /// Renames the references in `node`, a formula of `context_sheet`, to the
    /// named function `target` (scope, name). The names in `bound` (the
    /// parameters, if `node` is the body of a named function) are not
    /// references to it.
    fn rename_named_function_in_node(
        &self,
        node: &mut Node,
        context_sheet: u32,
        bound: &[String],
        target: (Option<u32>, &str),
        new_name: &str,
    ) {
        visit_free_names(node, &mut bound.to_vec(), &mut |name, name_use| {
            if self.reference_is_named_function(name, name_use, context_sheet, target) {
                *name = new_name.to_string();
            }
        });
    }

    /// Renames the references to the named function (`scope`, `name`) in
    /// every cell formula, in the defined names and in the bodies of the
    /// named functions.
    fn rename_named_function_in_formulas(
        &mut self,
        name: &str,
        scope: Option<u32>,
        new_name: &str,
    ) {
        // All internal formulas are R1C1 and in English
        let locale = self.locale;
        let language = self.language;
        self.parser.set_locale(get_default_locale());
        self.parser.set_language(get_default_language());
        self.parser.set_lexer_mode(LexerMode::R1C1);
        for sheet_index in 0..self.workbook.worksheets.len() {
            let cell_reference = CellReferenceRC {
                sheet: self.workbook.worksheets[sheet_index].get_name(),
                row: 1,
                column: 1,
            };
            let shared_formulas = self.workbook.worksheets[sheet_index]
                .shared_formulas
                .clone();
            let mut formulas = Vec::with_capacity(shared_formulas.len());
            for formula in &shared_formulas {
                let mut t = self.parser.parse(formula, &cell_reference);
                self.rename_named_function_in_node(
                    &mut t,
                    sheet_index as u32,
                    &[],
                    (scope, name),
                    new_name,
                );
                formulas.push(to_rc_format(&t));
            }
            self.workbook.worksheets[sheet_index].shared_formulas = formulas;
        }
        self.parser.set_lexer_mode(LexerMode::A1);
        self.parser.set_locale(locale);
        self.parser.set_language(language);

        // The defined names, stored in English. A name local to a sheet is
        // resolved in that sheet, a workbook one in the first sheet.
        for index in 0..self.workbook.defined_names.len() {
            let name_scope = self.scope_of(self.workbook.defined_names[index].sheet_id);
            let context = self.name_context(name_scope);
            let formula = self.workbook.defined_names[index].formula.clone();
            let had_equals = formula.starts_with('=');
            let mut node = self
                .parse_internal_formula(formula.strip_prefix('=').unwrap_or(&formula), &context);
            if let Node::ParseErrorKind { .. } = node {
                continue;
            }
            self.rename_named_function_in_node(
                &mut node,
                name_scope.unwrap_or(0),
                &[],
                (scope, name),
                new_name,
            );
            let renamed = to_english_string(&node, &context);
            self.workbook.defined_names[index].formula = if had_equals {
                format!("={renamed}")
            } else {
                renamed
            };
        }

        self.update_named_function_bodies(&|model, body, context_sheet, bound| {
            model.rename_named_function_in_node(body, context_sheet, bound, (scope, name), new_name)
        });
    }

    /// Rewrites the body of every named function with `update`, which gets
    /// the parsed body, the sheet it is resolved in (the one the function
    /// is local to, or the first one) and the names of its parameters.
    pub(crate) fn update_named_function_bodies(&mut self, update: &BodyUpdate<'_, 'a>) {
        for index in 0..self.workbook.named_functions.len() {
            let scope = self.scope_of(self.workbook.named_functions[index].sheet_id);
            let context = self.name_context(scope);
            let formula = self.workbook.named_functions[index].formula.clone();
            let bound = parameter_names(&self.workbook.named_functions[index].parameters);
            let mut body = self.parse_internal_body(&formula, &context, &bound);
            update(self, &mut body, scope.unwrap_or(0), &bound);
            self.workbook.named_functions[index].formula = to_english_string(&body, &context);
        }
    }

    /// The context used to parse/stringify defined-name formulas. Defined names
    /// have no natural anchor cell, so we use the first worksheet's A1.
    pub(crate) fn defined_name_context(&self) -> CellReferenceRC {
        self.name_context(None)
    }

    /// Validates if a defined name can be created
    pub fn is_valid_defined_name(
        &mut self,
        name: &str,
        scope: Option<u32>,
        formula: &str,
    ) -> Result<Option<u32>, String> {
        if !is_valid_identifier(name) {
            return Err("Name: Invalid defined name".to_string());
        }
        let sheet_id = match scope {
            Some(index) => match self.workbook.worksheet(index) {
                Ok(ws) => Some(ws.sheet_id),
                Err(_) => return Err("Scope: Invalid sheet index".to_string()),
            },
            None => None,
        };
        // if the defined name already exist return error
        if self.name_exists(name, sheet_id) {
            return Err("Name: Defined name already exists".to_string());
        }

        // Make sure the formula is valid: a cell/range reference or any
        // formula that parses (a constant, an expression, a LAMBDA definition).
        let is_reference =
            common::ParsedReference::parse_reference_formula(None, formula, self.locale, |name| {
                self.get_sheet_index_by_name(name)
            })
            .is_ok();

        if !is_reference {
            // Try the full parser.
            // Defined-name formulas may carry a leading '='; strip it before parsing.
            let formula_body = formula.strip_prefix('=').unwrap_or(formula);
            // A name local to a sheet is parsed in that sheet, so that the
            // names it uses resolve to the local ones (see `parse_defined_names`)
            let context_sheet = match scope {
                Some(index) => self.workbook.worksheets.get(index as usize),
                None => self.workbook.worksheets.first(),
            }
            .map(|ws| ws.get_name())
            .unwrap_or_else(|| "Sheet1".to_string());
            let dummy_ref = CellReferenceRC {
                sheet: context_sheet,
                row: 1,
                column: 1,
            };
            // Accept the formula whether it is written in the active language
            // or already in the internal English form (e.g. generated by
            // undo/redo or cut & paste).
            let mut node = self.parser.parse(formula_body, &dummy_ref);
            if let Node::ParseErrorKind { .. } = node {
                node = self.parse_internal_formula(formula_body, &dummy_ref);
            }
            if matches!(node, Node::ParseErrorKind { .. }) || has_unqualified_reference(&node) {
                return Err("Formula: Invalid defined name formula".to_string());
            }
        }

        Ok(sheet_id)
    }

    /// Delete defined name of name and scope
    pub fn delete_defined_name(&mut self, name: &str, scope: Option<u32>) -> Result<(), String> {
        let name_upper = name.to_uppercase();
        let defined_names = &self.workbook.defined_names;
        let sheet_id = match scope {
            Some(index) => Some(self.workbook.worksheet(index)?.sheet_id),
            None => None,
        };
        let mut index = None;
        for (i, df) in defined_names.iter().enumerate() {
            if df.name.to_uppercase() == name_upper && df.sheet_id == sheet_id {
                index = Some(i);
            }
        }
        if let Some(i) = index {
            self.workbook.defined_names.remove(i);
            self.reset_parsed_structures();
            Ok(())
        } else {
            Err("Defined name not found".to_string())
        }
    }

    /// Replaces the defined name `name` with scope `scope` by `new`: every
    /// field of the name takes the value in `new`.
    pub fn update_defined_name(
        &mut self,
        name: &str,
        scope: Option<u32>,
        new: DefinedNameEntry,
    ) -> Result<(), String> {
        let new_name = &new.name;
        let new_scope = new.scope;
        let new_formula = &new.formula;
        let new_hidden = new.hidden;
        let new_comment = &new.comment;
        if !is_valid_identifier(new_name) {
            return Err("Name: Invalid defined name".to_string());
        };
        validate_defined_name_comment(new_comment)?;
        let name_upper = name.to_uppercase();
        let new_name_upper = new_name.to_uppercase();

        let new_sheet_id = self.scope_sheet_id(new_scope)?;
        if (name_upper != new_name_upper || scope != new_scope)
            && self.name_exists(new_name, new_sheet_id)
        {
            return Err("Name: Defined name already exists".to_string());
        }
        let defined_names = &self.workbook.defined_names;
        let sheet_id = match scope {
            Some(index) => Some(
                self.workbook
                    .worksheet(index)
                    .map_err(|_| "Scope: Invalid sheet index")?
                    .sheet_id,
            ),
            None => None,
        };

        let mut index = None;
        for (i, df) in defined_names.iter().enumerate() {
            if df.name.to_uppercase() == name_upper && df.sheet_id == sheet_id {
                index = Some(i);
            }
        }
        // Defined-name formulas are stored internally in English.
        let context = self.defined_name_context();
        let internal_formula = self.user_formula_to_internal(new_formula, &context)?;
        if let Some(i) = index {
            if let Some((parameters, body)) = self.split_lambda_formula(&internal_formula, &context)
            {
                // A name defined as a LAMBDA is a named function. The cells
                // using the old name keep it: it is a different thing now.
                let function = NamedFunction {
                    name: split_user_function_prefix(new_name).0.to_string(),
                    sheet_id: new_sheet_id,
                    hidden: new_hidden,
                    description: new_comment.to_string(),
                    parameters,
                    formula: body,
                };
                self.validate_lambda_named_function(&function, new_name, Some((name, scope)))?;
                self.workbook.defined_names.remove(i);
                self.workbook.named_functions.push(function);
                self.reset_parsed_structures();
                return Ok(());
            }
            if let Some(df) = self.workbook.defined_names.get_mut(i) {
                if *new_name != df.name {
                    // We need to rename the name in every formula:

                    // Parse all formulas with the old name
                    // All internal formulas are R1C1
                    self.parser.set_lexer_mode(LexerMode::R1C1);
                    let worksheets = &mut self.workbook.worksheets;
                    for worksheet in worksheets {
                        let cell_reference = CellReferenceRC {
                            sheet: worksheet.get_name(),
                            row: 1,
                            column: 1,
                        };
                        let mut formulas = Vec::new();
                        for formula in &worksheet.shared_formulas {
                            let mut t = self.parser.parse(formula, &cell_reference);
                            rename_defined_name_in_node(&mut t, name, scope, new_name);
                            formulas.push(to_rc_format(&t));
                        }
                        worksheet.shared_formulas = formulas;
                    }
                    // Se the mode back to A1
                    self.parser.set_lexer_mode(LexerMode::A1);
                    self.update_named_function_bodies(&|_, body, _, _| {
                        rename_defined_name_in_node(body, name, scope, new_name)
                    });
                }
                let df = &mut self.workbook.defined_names[i];
                df.name = new_name.to_string();
                df.sheet_id = new_sheet_id;
                df.formula = internal_formula;
                df.hidden = new_hidden;
                df.comment = new_comment.to_string();
                self.reset_parsed_structures();
            }
            Ok(())
        } else {
            Err("Defined name not found".to_string())
        }
    }
}
