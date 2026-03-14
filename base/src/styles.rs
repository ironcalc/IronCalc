use crate::{
    model::Model,
    types::{Border, CellStyles, CellXfs, Fill, Font, NumFmt, Style, Styles},
};

impl Styles {
    fn get_font_index(&self, font: &Font) -> Option<i32> {
        for (font_index, item) in self.fonts.iter().enumerate() {
            if item == font {
                return Some(font_index as i32);
            }
        }
        None
    }
    fn get_fill_index(&self, fill: &Fill) -> Option<i32> {
        for (fill_index, item) in self.fills.iter().enumerate() {
            if item == fill {
                return Some(fill_index as i32);
            }
        }
        None
    }
    fn get_border_index(&self, border: &Border) -> Option<i32> {
        for (border_index, item) in self.borders.iter().enumerate() {
            if item == border {
                return Some(border_index as i32);
            }
        }
        None
    }

    pub fn create_new_style(&mut self, style: &Style) -> i32 {
        let font = &style.font;
        let font_id = if let Some(index) = self.get_font_index(font) {
            index
        } else {
            self.fonts.push(font.clone());
            self.fonts.len() as i32 - 1
        };
        let fill = &style.fill;
        let fill_id = if let Some(index) = self.get_fill_index(fill) {
            index
        } else {
            self.fills.push(fill.clone());
            self.fills.len() as i32 - 1
        };
        let border = &style.border;
        let border_id = if let Some(index) = self.get_border_index(border) {
            index
        } else {
            self.borders.push(border.clone());
            self.borders.len() as i32 - 1
        };
        let num_fmt_id =
            NumFmt::get_or_register(&style.num_fmt.format_code, &mut self.num_fmts).num_fmt_id;
        debug_assert!(
            num_fmt_id >= 0,
            "num_fmt_id sentinel -1 must not reach CellXfs"
        );
        self.cell_xfs.push(CellXfs {
            xf_id: 0,
            num_fmt_id,
            font_id,
            fill_id,
            border_id,
            apply_number_format: false,
            apply_border: false,
            apply_alignment: false,
            apply_protection: false,
            apply_font: false,
            apply_fill: false,
            quote_prefix: style.quote_prefix,
            alignment: style.alignment.clone(),
        });
        self.cell_xfs.len() as i32 - 1
    }

    pub fn get_style_index(&self, style: &Style) -> Option<i32> {
        let font_id = self.get_font_index(&style.font)?;
        let fill_id = self.get_fill_index(&style.fill)?;
        let border_id = self.get_border_index(&style.border)?;
        let incoming_id = style.num_fmt.num_fmt_id;
        let fmt_code = style.num_fmt.format_code.as_str();

        self.cell_xfs
            .iter()
            .position(|xf| {
                xf.alignment == style.alignment
                // Compare by ID when available; string comparison collapses locale IDs 14/22
                // with custom formats sharing the same code. Fall back for -1 sentinel only.
                && if incoming_id >= 0 {
                    xf.num_fmt_id == incoming_id
                } else {
                    NumFmt::format_code_for_id(xf.num_fmt_id, &self.num_fmts) == fmt_code
                }
                && xf.fill_id == fill_id
                && xf.border_id == border_id
                && xf.font_id == font_id
                && xf.quote_prefix == style.quote_prefix
            })
            .map(|i| i as i32)
    }

    pub(crate) fn get_style_index_or_create(&mut self, style: &Style) -> i32 {
        // Check if style exist. If so sets style cell number to that otherwise create a new style.
        if let Some(index) = self.get_style_index(style) {
            index
        } else {
            self.create_new_style(style)
        }
    }

    /// Adds a named cell style from an existing index
    /// Fails if the named style already exists or if there is not a style with that index
    pub fn add_named_cell_style(
        &mut self,
        style_name: &str,
        style_index: i32,
    ) -> Result<(), String> {
        if self.get_style_index_by_name(style_name).is_ok() {
            return Err("A style with that name already exists".to_string());
        }
        if self.cell_xfs.len() < style_index as usize {
            return Err("There is no style with that index".to_string());
        }
        let cell_style = CellStyles {
            name: style_name.to_string(),
            xf_id: style_index,
            builtin_id: 0,
        };
        self.cell_styles.push(cell_style);
        Ok(())
    }

    // Returns the index of the style or fails.
    // NB: this method is case sensitive
    pub fn get_style_index_by_name(&self, style_name: &str) -> Result<i32, String> {
        for cell_style in &self.cell_styles {
            if cell_style.name == style_name {
                return Ok(cell_style.xf_id);
            }
        }
        Err(format!("Style '{style_name}' not found"))
    }

    pub fn create_named_style(&mut self, style_name: &str, style: &Style) -> Result<(), String> {
        let style_index = self.create_new_style(style);
        self.add_named_cell_style(style_name, style_index)
    }

    pub(crate) fn get_style_with_quote_prefix(&mut self, index: i32) -> Result<i32, String> {
        let mut style = self.get_style(index)?;
        style.quote_prefix = true;
        Ok(self.get_style_index_or_create(&style))
    }

    pub(crate) fn get_style_with_format(
        &mut self,
        index: i32,
        format_code: &str,
    ) -> Result<i32, String> {
        let mut style = self.get_style(index)?;
        style.num_fmt = NumFmt::get_or_register(format_code, &mut self.num_fmts);
        Ok(self.get_style_index_or_create(&style))
    }

    /// Raw `num_fmt_id` from `CellXfs` at `index`.
    pub(crate) fn get_num_fmt_id(&self, index: i32) -> Result<i32, String> {
        self.cell_xfs
            .get(index as usize)
            .map(|xf| xf.num_fmt_id)
            .ok_or_else(|| format!("Invalid style index: {index}"))
    }

    /// Returns (or creates) a style like `index` but with `num_fmt_id` set to `new_id`.
    /// Operates at the `CellXfs` level so IDs 14/22 are preserved without string round-trip.
    pub(crate) fn get_style_with_num_fmt_id(
        &mut self,
        index: i32,
        new_id: i32,
    ) -> Result<i32, String> {
        if !NumFmt::is_known_id(new_id, &self.num_fmts) {
            return Err(format!(
                "num_fmt_id {new_id} is neither a built-in ECMA-376 ID nor registered in num_fmts"
            ));
        }
        let base = self
            .cell_xfs
            .get(index as usize)
            .ok_or_else(|| format!("Invalid style index: {index}"))?
            .clone();
        let target = CellXfs {
            num_fmt_id: new_id,
            ..base
        };
        // Reuse an existing entry if possible.
        for (i, existing) in self.cell_xfs.iter().enumerate() {
            if *existing == target {
                return Ok(i as i32);
            }
        }
        self.cell_xfs.push(target);
        Ok(self.cell_xfs.len() as i32 - 1)
    }

    pub(crate) fn get_style_without_quote_prefix(&mut self, index: i32) -> Result<i32, String> {
        let mut style = self.get_style(index)?;
        style.quote_prefix = false;
        Ok(self.get_style_index_or_create(&style))
    }

    pub(crate) fn style_is_quote_prefix(&self, index: i32) -> bool {
        let cell_xf = &self.cell_xfs[index as usize];
        cell_xf.quote_prefix
    }

    pub(crate) fn get_style(&self, index: i32) -> Result<Style, String> {
        let cell_xf = &self
            .cell_xfs
            .get(index as usize)
            .ok_or_else(|| format!("Invalid style index: {index}"))?;
        let border_id = cell_xf.border_id as usize;
        let fill_id = cell_xf.fill_id as usize;
        let font_id = cell_xf.font_id as usize;
        let num_fmt_id = cell_xf.num_fmt_id;
        let quote_prefix = cell_xf.quote_prefix;
        let alignment = cell_xf.alignment.clone();

        Ok(Style {
            alignment,
            num_fmt: NumFmt::from_id(num_fmt_id, &self.num_fmts),
            fill: self.fills.get(fill_id).cloned().unwrap_or_default(),
            font: self.fonts.get(font_id).cloned().unwrap_or_default(),
            border: self.borders.get(border_id).cloned().unwrap_or_default(),
            quote_prefix,
        })
    }
}

// TODO: Try to find a better spot for styles setters
impl<'a> Model<'a> {
    pub fn set_cell_style(
        &mut self,
        sheet: u32,
        row: i32,
        column: i32,
        style: &Style,
    ) -> Result<(), String> {
        let style_index = self.workbook.styles.get_style_index_or_create(style);
        self.workbook
            .worksheet_mut(sheet)?
            .set_cell_style(row, column, style_index)
    }

    pub fn copy_cell_style(
        &mut self,
        source_cell: (u32, i32, i32),
        destination_cell: (u32, i32, i32),
    ) -> Result<(), String> {
        let source_style_index = self
            .workbook
            .worksheet(source_cell.0)?
            .get_style(source_cell.1, source_cell.2);

        self.workbook
            .worksheet_mut(destination_cell.0)?
            .set_cell_style(destination_cell.1, destination_cell.2, source_style_index)
    }

    /// Sets the style "style_name" in cell
    pub fn set_cell_style_by_name(
        &mut self,
        sheet: u32,
        row: i32,
        column: i32,
        style_name: &str,
    ) -> Result<(), String> {
        let style_index = self.workbook.styles.get_style_index_by_name(style_name)?;
        self.workbook
            .worksheet_mut(sheet)?
            .set_cell_style(row, column, style_index)
    }

    pub fn set_sheet_style(&mut self, sheet: u32, style_name: &str) -> Result<(), String> {
        let style_index = self.workbook.styles.get_style_index_by_name(style_name)?;
        self.workbook.worksheet_mut(sheet)?.set_style(style_index)?;
        Ok(())
    }

    pub fn set_sheet_row_style(
        &mut self,
        sheet: u32,
        row: i32,
        style_name: &str,
    ) -> Result<(), String> {
        let style_index = self.workbook.styles.get_style_index_by_name(style_name)?;
        self.workbook
            .worksheet_mut(sheet)?
            .set_row_style(row, style_index)?;
        Ok(())
    }

    pub fn set_sheet_column_style(
        &mut self,
        sheet: u32,
        column: i32,
        style_name: &str,
    ) -> Result<(), String> {
        let style_index = self.workbook.styles.get_style_index_by_name(style_name)?;
        self.workbook
            .worksheet_mut(sheet)?
            .set_column_style(column, style_index)?;
        Ok(())
    }
}
