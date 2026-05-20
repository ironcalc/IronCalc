use chrono::DateTime;

use std::collections::HashMap;

use crate::{
    calc_result::Range,
    constants::{DEFAULT_WINDOW_HEIGHT, DEFAULT_WINDOW_WIDTH},
    expressions::{
        lexer::LexerMode,
        parser::{
            static_analysis::run_static_analysis_on_node,
            stringify::{rename_sheet_in_node, to_localized_string, to_rc_format},
            Node, Parser,
        },
        types::CellReferenceRC,
    },
    language::{get_default_language, get_language},
    locale::{get_default_locale, get_locale},
    model::{get_milliseconds_since_epoch, Model, ParsedDefinedName},
    types::{
        Border, BorderItem, BorderStyle, DefinedName, Fill, Font, FontScheme, Metadata, SheetState,
        Style, Styles, Workbook, WorkbookSettings, WorkbookView, Worksheet, WorksheetView,
    },
    utils::ParsedReference,
};

use crate::tz::Tz;

pub const APPLICATION: &str = "IronCalc Sheets";
pub const APP_VERSION: &str = "10.0000";
pub const IRONCALC_USER: &str = "IronCalc User";

// Default IronCalc theme palette (matches xlsx/src/import/theme.rs DEFAULT_PALETTE)
const ACCENT1: &str = "#4472C4";
const ACCENT2: &str = "#ED7D31";
const ACCENT3: &str = "#A5A5A5";
const ACCENT4: &str = "#FFC000";
const ACCENT5: &str = "#5B9BD5";
const ACCENT6: &str = "#70AD47";
const DK2: &str = "#44546A";

fn tint_color(hex: &str, tint: f64) -> String {
    fn hue_to_rgb(p: f64, q: f64, t: f64) -> f64 {
        let mut c = t;
        if c < 0.0 {
            c += 1.0;
        }
        if c > 1.0 {
            c -= 1.0;
        }
        if c < 1.0 / 6.0 {
            return p + (q - p) * 6.0 * t;
        }
        if c < 0.5 {
            return q;
        }
        if c < 2.0 / 3.0 {
            return p + (q - p) * (2.0 / 3.0 - t) * 6.0;
        }
        p
    }
    if tint == 0.0 {
        return hex.to_string();
    }
    let r = u8::from_str_radix(&hex[1..3], 16).unwrap_or(0) as i32;
    let g = u8::from_str_radix(&hex[3..5], 16).unwrap_or(0) as i32;
    let b = u8::from_str_radix(&hex[5..7], 16).unwrap_or(0) as i32;
    let max_c = r.max(g).max(b);
    let min_c = r.min(g).min(b);
    let chroma = (max_c - min_c) as f64 / 255.0;
    if chroma == 0.0 {
        let l = (r as f64 / 255.0 * 100.0).round() as i32;
        let new_l = if tint < 0.0 {
            (l as f64 * (1.0 + tint)).round() as i32
        } else {
            (l as f64 + (100.0 - l as f64) * tint).round() as i32
        };
        let v = (new_l as f64 / 100.0 * 255.0).round() as u8;
        return format!("#{:02X}{:02X}{:02X}", v, v, v);
    }
    let rf = r as f64 / 255.0;
    let gf = g as f64 / 255.0;
    let bf = b as f64 / 255.0;
    let luminosity = (max_c + min_c) as f64 / (255.0 * 2.0);
    let saturation = if luminosity > 0.5 {
        0.5 * chroma / (1.0 - luminosity)
    } else {
        0.5 * chroma / luminosity
    };
    let hue = if max_c == r {
        if g >= b {
            60.0 * (gf - bf) / chroma
        } else {
            ((gf - bf) / chroma + 6.0) * 60.0
        }
    } else if max_c == g {
        ((bf - rf) / chroma + 2.0) * 60.0
    } else {
        ((rf - gf) / chroma + 4.0) * 60.0
    };
    let h = hue.round() as i32;
    let s = (saturation * 100.0).round() as i32;
    let l = (luminosity * 100.0).round() as i32;
    let new_l = if tint < 0.0 {
        (l as f64 * (1.0 + tint)).round() as i32
    } else {
        (l as f64 + (100.0 - l as f64) * tint).round() as i32
    };
    let hue_f = h as f64 / 360.0;
    let sat_f = s as f64 / 100.0;
    let lum_f = new_l as f64 / 100.0;
    let (nr, ng, nb) = if sat_f == 0.0 {
        let v = (lum_f * 255.0).round() as u8;
        (v, v, v)
    } else {
        let q = if lum_f < 0.5 {
            lum_f * (1.0 + sat_f)
        } else {
            lum_f + sat_f - lum_f * sat_f
        };
        let p = 2.0 * lum_f - q;
        let nr = (255.0 * hue_to_rgb(p, q, hue_f + 1.0 / 3.0)).round() as u8;
        let ng = (255.0 * hue_to_rgb(p, q, hue_f)).round() as u8;
        let nb = (255.0 * hue_to_rgb(p, q, hue_f - 1.0 / 3.0)).round() as u8;
        (nr, ng, nb)
    };
    format!("#{:02X}{:02X}{:02X}", nr, ng, nb)
}

fn solid_fill(color: &str) -> Fill {
    Fill {
        pattern_type: "solid".to_string(),
        fg_color: Some(color.to_string()),
        bg_color: None,
    }
}

fn thin_box_border(color: &str) -> Border {
    let item = Some(BorderItem {
        style: BorderStyle::Thin,
        color: Some(color.to_string()),
    });
    Border {
        left: item.clone(),
        right: item.clone(),
        top: item.clone(),
        bottom: item,
        ..Default::default()
    }
}

fn double_box_border(color: &str) -> Border {
    let item = Some(BorderItem {
        style: BorderStyle::Double,
        color: Some(color.to_string()),
    });
    Border {
        left: item.clone(),
        right: item.clone(),
        top: item.clone(),
        bottom: item,
        ..Default::default()
    }
}

fn thick_bottom_border(color: &str) -> Border {
    Border {
        bottom: Some(BorderItem {
            style: BorderStyle::Thick,
            color: Some(color.to_string()),
        }),
        ..Default::default()
    }
}

fn thin_top_double_bottom_border(color: &str) -> Border {
    Border {
        top: Some(BorderItem {
            style: BorderStyle::Thin,
            color: Some(color.to_string()),
        }),
        bottom: Some(BorderItem {
            style: BorderStyle::Double,
            color: Some(color.to_string()),
        }),
        ..Default::default()
    }
}

fn default_builtin_styles() -> Styles {
    let mut styles = Styles::default();

    // Good, Bad, Neutral
    let _ = styles.create_named_style(
        "Good",
        &Style {
            font: Font {
                color: Some("#006100".to_string()),
                ..Default::default()
            },
            fill: solid_fill("#C6EFCE"),
            ..Default::default()
        },
    );
    let _ = styles.create_named_style(
        "Bad",
        &Style {
            font: Font {
                color: Some("#9C0006".to_string()),
                ..Default::default()
            },
            fill: solid_fill("#FFC7CE"),
            ..Default::default()
        },
    );
    let _ = styles.create_named_style(
        "Neutral",
        &Style {
            font: Font {
                color: Some("#9C5700".to_string()),
                ..Default::default()
            },
            fill: solid_fill("#FFEB9C"),
            ..Default::default()
        },
    );

    // Data and Model
    let _ = styles.create_named_style(
        "Calculation",
        &Style {
            font: Font {
                b: true,
                color: Some("#FA7D00".to_string()),
                ..Default::default()
            },
            fill: solid_fill("#F2F2F2"),
            border: thin_box_border("#7F7F7F"),
            ..Default::default()
        },
    );
    let _ = styles.create_named_style(
        "Check Cell",
        &Style {
            font: Font {
                b: true,
                color: Some("#FFFFFF".to_string()),
                ..Default::default()
            },
            fill: solid_fill("#A5A5A5"),
            border: double_box_border("#3F3F3F"),
            ..Default::default()
        },
    );
    let _ = styles.create_named_style(
        "Explanatory Text",
        &Style {
            font: Font {
                i: true,
                color: Some("#7F7F7F".to_string()),
                ..Default::default()
            },
            ..Default::default()
        },
    );
    let _ = styles.create_named_style(
        "Input",
        &Style {
            font: Font {
                color: Some("#3F3F76".to_string()),
                ..Default::default()
            },
            fill: solid_fill("#FFCC99"),
            border: thin_box_border("#7F7F7F"),
            ..Default::default()
        },
    );
    let _ = styles.create_named_style(
        "Linked Cell",
        &Style {
            font: Font {
                color: Some("#FA7D00".to_string()),
                ..Default::default()
            },
            border: Border {
                bottom: Some(BorderItem {
                    style: BorderStyle::Double,
                    color: Some("#FF8001".to_string()),
                }),
                ..Default::default()
            },
            ..Default::default()
        },
    );
    let _ = styles.create_named_style(
        "Note",
        &Style {
            fill: solid_fill("#FFFFE1"),
            border: thin_box_border("#B2B2B2"),
            ..Default::default()
        },
    );
    let _ = styles.create_named_style(
        "Output",
        &Style {
            font: Font {
                b: true,
                color: Some("#3F3F3F".to_string()),
                ..Default::default()
            },
            fill: solid_fill("#F2F2F2"),
            border: thin_box_border("#3F3F3F"),
            ..Default::default()
        },
    );
    let _ = styles.create_named_style(
        "Warning Text",
        &Style {
            font: Font {
                color: Some("#FF0000".to_string()),
                ..Default::default()
            },
            ..Default::default()
        },
    );

    // Titles and Headings
    let _ = styles.create_named_style(
        "Title",
        &Style {
            font: Font {
                sz: 18,
                color: Some(DK2.to_string()),
                scheme: FontScheme::Major,
                ..Default::default()
            },
            ..Default::default()
        },
    );
    let _ = styles.create_named_style(
        "Heading 1",
        &Style {
            font: Font {
                b: true,
                sz: 15,
                color: Some(DK2.to_string()),
                ..Default::default()
            },
            border: thick_bottom_border(ACCENT1),
            ..Default::default()
        },
    );
    let h2_border_color = tint_color(ACCENT1, 0.5);
    let _ = styles.create_named_style(
        "Heading 2",
        &Style {
            font: Font {
                b: true,
                sz: 13,
                color: Some(DK2.to_string()),
                ..Default::default()
            },
            border: thick_bottom_border(&h2_border_color),
            ..Default::default()
        },
    );
    let _ = styles.create_named_style(
        "Heading 3",
        &Style {
            font: Font {
                b: true,
                color: Some(DK2.to_string()),
                ..Default::default()
            },
            border: Border {
                bottom: Some(BorderItem {
                    style: BorderStyle::Thin,
                    color: Some(ACCENT1.to_string()),
                }),
                ..Default::default()
            },
            ..Default::default()
        },
    );
    let _ = styles.create_named_style(
        "Heading 4",
        &Style {
            font: Font {
                b: true,
                i: true,
                color: Some(DK2.to_string()),
                ..Default::default()
            },
            ..Default::default()
        },
    );
    let _ = styles.create_named_style(
        "Total",
        &Style {
            font: Font {
                b: true,
                ..Default::default()
            },
            border: thin_top_double_bottom_border(ACCENT1),
            ..Default::default()
        },
    );

    // Themed Cell Styles: 20% / 40% / 60% tints and solid for each accent
    for (accent_name, accent_hex) in [
        ("Accent1", ACCENT1),
        ("Accent2", ACCENT2),
        ("Accent3", ACCENT3),
        ("Accent4", ACCENT4),
        ("Accent5", ACCENT5),
        ("Accent6", ACCENT6),
    ] {
        let c20 = tint_color(accent_hex, 0.8);
        let _ = styles.create_named_style(
            &format!("20% - {accent_name}"),
            &Style {
                fill: solid_fill(&c20),
                ..Default::default()
            },
        );
        let c40 = tint_color(accent_hex, 0.6);
        let _ = styles.create_named_style(
            &format!("40% - {accent_name}"),
            &Style {
                fill: solid_fill(&c40),
                ..Default::default()
            },
        );
        let c60 = tint_color(accent_hex, 0.4);
        let _ = styles.create_named_style(
            &format!("60% - {accent_name}"),
            &Style {
                fill: solid_fill(&c60),
                ..Default::default()
            },
        );
        let _ = styles.create_named_style(
            accent_name,
            &Style {
                fill: solid_fill(accent_hex),
                ..Default::default()
            },
        );
    }

    // Number Format styles
    let _ = styles.create_named_style(
        "Comma",
        &Style {
            num_fmt: "#,##0.00".to_string(),
            ..Default::default()
        },
    );
    let _ = styles.create_named_style(
        "Comma [0]",
        &Style {
            num_fmt: "#,##0".to_string(),
            ..Default::default()
        },
    );
    let _ = styles.create_named_style(
        "Currency",
        &Style {
            num_fmt: r#"_("$"* #,##0.00_);_("$"* \(#,##0.00\);_("$"* "-"??_);_(@_)"#.to_string(),
            ..Default::default()
        },
    );
    let _ = styles.create_named_style(
        "Currency [0]",
        &Style {
            num_fmt: r#"_("$"* #,##0_);_("$"* \(#,##0\);_("$"* "-"_);_(@_)"#.to_string(),
            ..Default::default()
        },
    );
    let _ = styles.create_named_style(
        "Percent",
        &Style {
            num_fmt: "0%".to_string(),
            ..Default::default()
        },
    );

    styles
}

/// Name cannot be blank, must be shorter than 31 characters.
/// You can use all alphanumeric characters but not the following special characters:
/// \ , / , * , ? , : , [ , ].
fn is_valid_sheet_name(name: &str) -> bool {
    let invalid = ['\\', '/', '*', '?', ':', '[', ']'];
    !name.is_empty() && name.chars().count() <= 31 && !name.contains(&invalid[..])
}

impl<'a> Model<'a> {
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
            conditional_formatting: vec![],
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

    // This function parses all the internal formulas in all the worksheets
    // (in the default language ("en") and locale ("en") and the RC format)
    pub(crate) fn parse_formulas(&mut self) {
        let locale = self.locale;
        let language = self.language;

        self.parser.set_locale(get_default_locale());
        self.parser.set_language(get_default_language());
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
                let static_result = run_static_analysis_on_node(&t);
                parse_formula.push((t, static_result));
            }
            self.parsed_formulas.push(parse_formula);
        }
        self.parser.set_lexer_mode(LexerMode::A1);
        self.parser.set_locale(locale);
        self.parser.set_language(language);
    }

    pub(crate) fn parse_defined_names(&mut self) {
        // Collect first to avoid borrow conflicts when calling self.parser below.
        let entries: Vec<(String, String, Option<u32>)> = self
            .workbook
            .defined_names
            .iter()
            .map(|dn| (dn.name.clone(), dn.formula.clone(), dn.sheet_id))
            .collect();

        let mut parsed_defined_names = HashMap::new();

        for (name, formula, sheet_id) in entries {
            let parsed_defined_name_formula = if let Ok(reference) =
                ParsedReference::parse_reference_formula(None, &formula, self.locale, |n| {
                    self.get_sheet_index_by_name(n)
                }) {
                match reference {
                    ParsedReference::CellReference(cell_reference) => {
                        ParsedDefinedName::CellReference(cell_reference)
                    }
                    ParsedReference::Range(left, right) => {
                        ParsedDefinedName::RangeReference(Range { left, right })
                    }
                }
            } else {
                // Try the full parser — the formula might be a LAMBDA definition.
                // Defined-name formulas may carry a leading '='; strip it before parsing.
                let formula_body = formula.strip_prefix('=').unwrap_or(&formula);
                let dummy_ref = CellReferenceRC {
                    sheet: self
                        .workbook
                        .worksheets
                        .first()
                        .map(|ws| ws.get_name())
                        .unwrap_or_else(|| "Sheet1".to_string()),
                    row: 1,
                    column: 1,
                };
                match self.parser.parse(formula_body, &dummy_ref) {
                    Node::LambdaDefKind { parameters, body } => {
                        ParsedDefinedName::LambdaDefinition(parameters, *body)
                    }
                    _ => ParsedDefinedName::InvalidDefinedNameFormula,
                }
            };

            let local_sheet_index = if let Some(sid) = sheet_id {
                if let Some(idx) = self.get_sheet_index_by_sheet_id(sid) {
                    Some(idx)
                } else {
                    // Sheet with given sheet_id not found.
                    continue;
                }
            } else {
                None
            };

            parsed_defined_names.insert(
                (local_sheet_index, name.to_lowercase()),
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

    /// Gets the base name for new sheets
    fn get_sheet_name(&self) -> String {
        let language = self.language;
        match language.code.as_str() {
            "en" => "Sheet".to_string(),
            "es" => "Hoja".to_string(),
            "fr" => "Feuil".to_string(),
            "de" => "Tabelle".to_string(),
            "it" => "Foglio".to_string(),
            _ => "Sheet".to_string(),
        }
    }

    /// Adds a sheet with a automatically generated name
    pub fn new_sheet(&mut self) -> (String, u32) {
        // First we find a name
        let base_name = self.get_sheet_name();
        let base_name_uppercase = base_name.to_uppercase();
        let mut index = 1;
        while self
            .workbook
            .get_worksheet_names()
            .iter()
            .map(|s| s.to_uppercase())
            .any(|x| x == format!("{base_name_uppercase}{index}"))
        {
            index += 1;
        }
        let sheet_name = format!("{base_name}{index}");
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
            return Err(format!("Invalid name for a sheet: '{sheet_name}'"));
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
        Err(format!("Could not find sheet {old_name}"))
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
            return Err(format!("Invalid name for a sheet: '{new_name}'."));
        }
        if let Some(new_index) = self.get_sheet_index_by_name(new_name) {
            if new_index != sheet_index {
                return Err(format!("Sheet already exists: '{new_name}'."));
            }
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
            let formula = to_localized_string(&t, cell_reference, self.locale, self.language);
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
    pub fn new_empty(
        name: &'a str,
        locale_id: &'a str,
        timezone: &'a str,
        language_id: &'a str,
    ) -> Result<Model<'a>, String> {
        let tz = Tz::parse(timezone)?;
        let locale = match get_locale(locale_id) {
            Ok(l) => l,
            Err(_) => return Err(format!("Invalid locale: {locale_id}")),
        };
        let language = match get_language(language_id) {
            Ok(l) => l,
            Err(_) => return Err(format!("Invalid language: {language_id}")),
        };

        let milliseconds = get_milliseconds_since_epoch();
        let seconds = milliseconds / 1000;
        let dt = match DateTime::from_timestamp(seconds, 0) {
            Some(s) => s,
            None => return Err(format!("Invalid timestamp: {milliseconds}")),
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

        let sheet_name = match language.code.as_str() {
            "en" => "Sheet1".to_string(),
            "es" => "Hoja1".to_string(),
            "fr" => "Feuil1".to_string(),
            "de" => "Tabelle1".to_string(),
            "it" => "Foglio1".to_string(),
            _ => "Sheet1".to_string(),
        };

        // String versions of the locale are added here to simplify the serialize/deserialize logic
        let workbook = Workbook {
            shared_strings: vec![],
            defined_names: vec![],
            worksheets: vec![Model::new_empty_worksheet(&sheet_name, 1, &[&0])],
            styles: default_builtin_styles(),
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
        };
        let parsed_formulas = Vec::new();
        let worksheets = &workbook.worksheets;
        let worksheet_names = worksheets.iter().map(|s| s.get_name()).collect();
        let parser = Parser::new(worksheet_names, vec![], HashMap::new(), locale, language);
        let cells = HashMap::new();

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
            variable_stack: HashMap::new(),
            last_variable_id: 0,
            lambdas: HashMap::new(),
            last_lambda_id: 0,
            spill_cells: Vec::new(),
            support: HashMap::new(),
            cf_cache: HashMap::new(),
        };
        model.parse_formulas();
        model.evaluate_conditional_formatting();
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
