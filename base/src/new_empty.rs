use chrono::DateTime;

use std::collections::HashMap;

use crate::{
    calc_result::Range,
    constants::{DEFAULT_WINDOW_HEIGHT, DEFAULT_WINDOW_WIDTH},
    expressions::{
        lexer::LexerMode,
        parser::{
            stringify::{rename_sheet_in_node, to_rc_format, to_string},
            Parser,
        },
        types::CellReferenceRC,
    },
    language::get_language,
    locale::get_locale,
    model::{get_milliseconds_since_epoch, Model, ParsedDefinedName},
    types::{
        DefinedName, Metadata, SheetState, Workbook, WorkbookSettings, WorkbookView, Worksheet,
        WorksheetView,
    },
    utils::ParsedReference,
};

use chrono_tz::Tz;

pub const APPLICATION: &str = "IronCalc Sheets";
pub const APP_VERSION: &str = "10.0000";
pub const IRONCALC_USER: &str = "IronCalc User";

/// Name cannot be blank, must be shorter than 31 characters.
/// You can use all alphanumeric characters but not the following special characters:
/// \ , / , * , ? , : , [ , ].
fn is_valid_sheet_name(name: &str) -> bool {
    let invalid = ['\\', '/', '*', '?', ':', '[', ']'];
    !name.is_empty() && name.chars().count() <= 31 && !name.contains(&invalid[..])
}

impl Model {
    /// Creates a new worksheet. Note that it does not check if the name or the sheet_id exists
    fn new_empty_worksheet(name: &str, sheet_id: u32, view_ids: &[&u32]) -> Worksheet {
        let mut views = HashMap::new();
        for id in view_ids {
            views.insert(
                **id,
                WorksheetView {
                    row: 1,
                    column: 1,
                    range: [1, 1, 1, 1],
                    top_row: 1,
                    left_column: 1,
                },
            );
        }
        Worksheet {
            cols: vec![],
            rows: vec![],
            comments: vec![],
            dimension: "A1".to_string(),
            merge_cells: vec![],
            name: name.to_string(),
            shared_formulas: vec![],
            sheet_data: Default::default(),
            sheet_id,
            state: SheetState::Visible,
            color: Default::default(),
            frozen_columns: 0,
            frozen_rows: 0,
            show_grid_lines: true,
            views,
        }
    }

    pub fn get_new_sheet_id(&self) -> u32 {
        let mut index = 1;
        let worksheets = &self.workbook.worksheets;
        for worksheet in worksheets {
            index = index.max(worksheet.sheet_id);
        }
        index + 1
    }

    pub(crate) fn parse_formulas(&mut self) {
        self.parser.set_lexer_mode(LexerMode::R1C1);
        let worksheets = &self.workbook.worksheets;
        for worksheet in worksheets {
            let shared_formulas = &worksheet.shared_formulas;
            let cell_reference = CellReferenceRC {
                sheet: worksheet.get_name(),
                row: 1,
                column: 1,
            };
            let mut parse_formula = Vec::new();
            for formula in shared_formulas {
                let t = self.parser.parse(formula, &cell_reference);
                parse_formula.push(t);
            }
            self.parsed_formulas.push(parse_formula);
        }
        self.parser.set_lexer_mode(LexerMode::A1);
    }

    pub(crate) fn parse_defined_names(&mut self) {
        let mut parsed_defined_names = HashMap::new();
        for defined_name in &self.workbook.defined_names {
            let parsed_defined_name_formula = if let Ok(reference) =
                ParsedReference::parse_reference_formula(
                    None,
                    &defined_name.formula,
                    &self.locale,
                    |name| self.get_sheet_index_by_name(name),
                ) {
                match reference {
                    ParsedReference::CellReference(cell_reference) => {
                        ParsedDefinedName::CellReference(cell_reference)
                    }
                    ParsedReference::Range(left, right) => {
                        ParsedDefinedName::RangeReference(Range { left, right })
                    }
                }
            } else {
                ParsedDefinedName::InvalidDefinedNameFormula
            };

            let local_sheet_index = if let Some(sheet_id) = defined_name.sheet_id {
                if let Some(sheet_index) = self.get_sheet_index_by_sheet_id(sheet_id) {
                    Some(sheet_index)
                } else {
                    // TODO: Error: Sheet with given sheet_id not found.
                    continue;
                }
            } else {
                None
            };

            parsed_defined_names.insert(
                (local_sheet_index, defined_name.name.to_lowercase()),
                parsed_defined_name_formula,
            );
        }

        self.parsed_defined_names = parsed_defined_names;
    }

    /// Reparses all formulas and defined names
    pub(crate) fn reset_parsed_structures(&mut self) {
        let defined_names = self.workbook.get_defined_names_with_scope();
        self.parser
            .set_worksheets_and_names(self.workbook.get_worksheet_names(), defined_names);
        self.parsed_formulas = vec![];
        self.parse_formulas();
        self.parsed_defined_names = HashMap::new();
        self.parse_defined_names();
        self.evaluate();
    }

    /// Adds a sheet with a automatically generated name
    pub fn new_sheet(&mut self) -> (String, u32) {
        // First we find a name

        // TODO: The name should depend on the locale
        let base_name = "Sheet";
        let base_name_uppercase = base_name.to_uppercase();
        let mut index = 1;
        while self
            .workbook
            .get_worksheet_names()
            .iter()
            .map(|s| s.to_uppercase())
            .any(|x| x == format!("{}{}", base_name_uppercase, index))
        {
            index += 1;
        }
        let sheet_name = format!("{}{}", base_name, index);
        // Now we need a sheet_id
        let sheet_id = self.get_new_sheet_id();
        let view_ids: Vec<&u32> = self.workbook.views.keys().collect();
        let worksheet = Model::new_empty_worksheet(&sheet_name, sheet_id, &view_ids);
        self.workbook.worksheets.push(worksheet);
        self.reset_parsed_structures();
        (sheet_name, self.workbook.worksheets.len() as u32 - 1)
    }

    /// Inserts a sheet with a particular index
    /// Fails if a worksheet with that name already exists or the name is invalid
    /// Fails if the index is too large
    pub fn insert_sheet(
        &mut self,
        sheet_name: &str,
        sheet_index: u32,
        sheet_id: Option<u32>,
    ) -> Result<(), String> {
        if !is_valid_sheet_name(sheet_name) {
            return Err(format!("Invalid name for a sheet: '{}'", sheet_name));
        }
        if self
            .workbook
            .get_worksheet_names()
            .iter()
            .map(|s| s.to_uppercase())
            .any(|x| x == sheet_name.to_uppercase())
        {
            return Err("A worksheet already exists with that name".to_string());
        }
        let sheet_id = match sheet_id {
            Some(id) => id,
            None => self.get_new_sheet_id(),
        };
        let view_ids: Vec<&u32> = self.workbook.views.keys().collect();
        let worksheet = Model::new_empty_worksheet(sheet_name, sheet_id, &view_ids);
        if sheet_index as usize > self.workbook.worksheets.len() {
            return Err("Sheet index out of range".to_string());
        }
        self.workbook
            .worksheets
            .insert(sheet_index as usize, worksheet);
        self.reset_parsed_structures();
        Ok(())
    }

    /// Adds a sheet with a specific name
    /// Fails if a worksheet with that name already exists or the name is invalid
    pub fn add_sheet(&mut self, sheet_name: &str) -> Result<(), String> {
        self.insert_sheet(sheet_name, self.workbook.worksheets.len() as u32, None)
    }

    /// Renames a sheet and updates all existing references to that sheet.
    /// It can fail if:
    ///   * The original sheet does not exists
    ///   * The target sheet already exists
    ///   * The target sheet name is invalid
    pub fn rename_sheet(&mut self, old_name: &str, new_name: &str) -> Result<(), String> {
        if let Some(sheet_index) = self.get_sheet_index_by_name(old_name) {
            return self.rename_sheet_by_index(sheet_index, new_name);
        }
        Err(format!("Could not find sheet {}", old_name))
    }

    /// Renames a sheet and updates all existing references to that sheet.
    /// It can fail if:
    ///   * The original index is out of bounds
    ///   * The target sheet name already exists
    ///   * The target sheet name is invalid
    pub fn rename_sheet_by_index(
        &mut self,
        sheet_index: u32,
        new_name: &str,
    ) -> Result<(), String> {
        if !is_valid_sheet_name(new_name) {
            return Err(format!("Invalid name for a sheet: '{}'.", new_name));
        }
        if self.get_sheet_index_by_name(new_name).is_some() {
            return Err(format!("Sheet already exists: '{}'.", new_name));
        }
        // Gets the new name and checks that a sheet with that index exists
        let old_name = self.workbook.worksheet(sheet_index)?.get_name();

        // Parse all formulas with the old name
        // All internal formulas are R1C1
        self.parser.set_lexer_mode(LexerMode::R1C1);

        for worksheet in &mut self.workbook.worksheets {
            // R1C1 formulas are not tied to a cell (but are tied to a cell)
            let cell_reference = &CellReferenceRC {
                sheet: worksheet.get_name(),
                row: 1,
                column: 1,
            };
            let mut formulas = Vec::new();
            for formula in &worksheet.shared_formulas {
                let mut t = self.parser.parse(formula, cell_reference);
                rename_sheet_in_node(&mut t, sheet_index, new_name);
                formulas.push(to_rc_format(&t));
            }
            worksheet.shared_formulas = formulas;
        }

        // Set the mode back to A1
        self.parser.set_lexer_mode(LexerMode::A1);

        // We reparse all the defined names formulas
        let mut defined_names = Vec::new();
        // Defined names do not have a context, we can use anything
        let cell_reference = &CellReferenceRC {
            sheet: old_name.clone(),
            row: 1,
            column: 1,
        };
        for defined_name in &mut self.workbook.defined_names {
            let mut t = self.parser.parse(&defined_name.formula, cell_reference);
            rename_sheet_in_node(&mut t, sheet_index, new_name);
            let formula = to_string(&t, cell_reference);
            defined_names.push(DefinedName {
                name: defined_name.name.clone(),
                formula,
                sheet_id: defined_name.sheet_id,
            });
        }
        self.workbook.defined_names = defined_names;

        // Update the name of the worksheet
        self.workbook.worksheet_mut(sheet_index)?.set_name(new_name);
        self.reset_parsed_structures();
        Ok(())
    }

    /// Deletes a sheet by index. Fails if:
    ///   * The sheet does not exists
    ///   * It is the last sheet
    pub fn delete_sheet(&mut self, sheet_index: u32) -> Result<(), String> {
        let worksheets = &self.workbook.worksheets;
        let sheet_count = worksheets.len() as u32;
        if sheet_count == 1 {
            return Err("Cannot delete only sheet".to_string());
        };
        if sheet_index >= sheet_count {
            return Err("Sheet index too large".to_string());
        };
        self.workbook.worksheets.remove(sheet_index as usize);
        self.reset_parsed_structures();
        Ok(())
    }

    /// Deletes a sheet by name. Fails if:
    ///   * The sheet does not exists
    ///   * It is the last sheet
    pub fn delete_sheet_by_name(&mut self, name: &str) -> Result<(), String> {
        if let Some(sheet_index) = self.get_sheet_index_by_name(name) {
            self.delete_sheet(sheet_index)
        } else {
            Err("Sheet not found".to_string())
        }
    }

    /// Deletes a sheet by sheet_id. Fails if:
    ///   * The sheet by sheet_id does not exists
    ///   * It is the last sheet
    pub fn delete_sheet_by_sheet_id(&mut self, sheet_id: u32) -> Result<(), String> {
        if let Some(sheet_index) = self.get_sheet_index_by_sheet_id(sheet_id) {
            self.delete_sheet(sheet_index)
        } else {
            Err("Sheet not found".to_string())
        }
    }

    pub(crate) fn get_sheet_index_by_sheet_id(&self, sheet_id: u32) -> Option<u32> {
        let worksheets = &self.workbook.worksheets;
        for (index, worksheet) in worksheets.iter().enumerate() {
            if worksheet.sheet_id == sheet_id {
                return Some(index as u32);
            }
        }
        None
    }

    /// Creates a new workbook with one empty sheet
    pub fn new_empty(name: &str, locale_id: &str, timezone: &str) -> Result<Model, String> {
        let tz: Tz = match &timezone.parse() {
            Ok(tz) => *tz,
            Err(_) => return Err(format!("Invalid timezone: {}", &timezone)),
        };
        let locale = match get_locale(locale_id) {
            Ok(l) => l.clone(),
            Err(_) => return Err(format!("Invalid locale: {}", locale_id)),
        };

        let milliseconds = get_milliseconds_since_epoch();
        let seconds = milliseconds / 1000;
        let dt = match DateTime::from_timestamp(seconds, 0) {
            Some(s) => s,
            None => return Err(format!("Invalid timestamp: {}", milliseconds)),
        };
        // "2020-08-06T21:20:53Z
        let now = dt.format("%Y-%m-%dT%H:%M:%SZ").to_string();

        let mut views = HashMap::new();
        views.insert(
            0,
            WorkbookView {
                sheet: 0,
                window_width: DEFAULT_WINDOW_WIDTH,
                window_height: DEFAULT_WINDOW_HEIGHT,
            },
        );

        // String versions of the locale are added here to simplify the serialize/deserialize logic
        let workbook = Workbook {
            shared_strings: vec![],
            defined_names: vec![],
            worksheets: vec![Model::new_empty_worksheet("Sheet1", 1, &[&0])],
            styles: Default::default(),
            name: name.to_string(),
            settings: WorkbookSettings {
                tz: timezone.to_string(),
                locale: locale_id.to_string(),
            },
            metadata: Metadata {
                application: APPLICATION.to_string(),
                app_version: APP_VERSION.to_string(),
                creator: IRONCALC_USER.to_string(),
                last_modified_by: IRONCALC_USER.to_string(),
                created: now.clone(),
                last_modified: now,
            },
            tables: HashMap::new(),
            views,
            users: Vec::new(),
        };
        let parsed_formulas = Vec::new();
        let worksheets = &workbook.worksheets;
        let worksheet_names = worksheets.iter().map(|s| s.get_name()).collect();
        let parser = Parser::new(worksheet_names, vec![], HashMap::new());
        let cells = HashMap::new();

        // FIXME: Add support for display languages
        #[allow(clippy::expect_used)]
        let language = get_language("en").expect("").clone();

        let mut model = Model {
            workbook,
            shared_strings: HashMap::new(),
            parsed_formulas,
            parsed_defined_names: HashMap::new(),
            parser,
            cells,
            locale,
            language,
            tz,
            view_id: 0,
        };
        model.parse_formulas();
        Ok(model)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_valid_sheet_name() {
        assert!(is_valid_sheet_name("Sheet1"));
        assert!(is_valid_sheet_name("Zażółć gęślą jaźń"));

        assert!(is_valid_sheet_name(" "));
        assert!(!is_valid_sheet_name(""));

        assert!(is_valid_sheet_name("🙈"));

        assert!(is_valid_sheet_name("AAAAAAAAAABBBBBBBBBBCCCCCCCCCCD")); // 31
        assert!(!is_valid_sheet_name("AAAAAAAAAABBBBBBBBBBCCCCCCCCCCDE")); // 32
    }
}
