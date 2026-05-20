use crate::{
    model::Model,
    number_format::{get_default_num_fmt_id, get_new_num_fmt_index, get_num_fmt},
    types::{Border, CellStyles, CellXfs, Dxf, Fill, Font, NumFmt, Style, Styles},
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
    fn get_num_fmt_index(&self, format_code: &str) -> Option<i32> {
        if let Some(index) = get_default_num_fmt_id(format_code) {
            return Some(index);
        }
        for item in self.num_fmts.iter() {
            if item.format_code == format_code {
                return Some(item.num_fmt_id);
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
        let num_fmt = &style.num_fmt;
        let num_fmt_id;
        if let Some(index) = self.get_num_fmt_index(num_fmt) {
            num_fmt_id = index;
        } else {
            num_fmt_id = get_new_num_fmt_index(&self.num_fmts);
            self.num_fmts.push(NumFmt {
                format_code: num_fmt.to_string(),
                num_fmt_id,
            });
        }
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
        for (index, cell_xf) in self.cell_xfs.iter().enumerate() {
            let border_id = cell_xf.border_id as usize;
            let fill_id = cell_xf.fill_id as usize;
            let font_id = cell_xf.font_id as usize;
            let num_fmt_id = cell_xf.num_fmt_id;
            let quote_prefix = cell_xf.quote_prefix;
            if style
                == &(Style {
                    alignment: cell_xf.alignment.clone(),
                    num_fmt: get_num_fmt(num_fmt_id, &self.num_fmts),
                    fill: self.fills[fill_id].clone(),
                    font: self.fonts[font_id].clone(),
                    border: self.borders[border_id].clone(),
                    quote_prefix,
                })
            {
                return Some(index as i32);
            }
        }
        None
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

    /// Returns the names of all named styles
    pub fn get_named_style_list(&self) -> Vec<String> {
        self.cell_styles
            .iter()
            .map(|cs| cs.name.clone())
            .collect()
    }

    /// Returns true if the named style is built-in and cannot be deleted or modified.
    /// In OOXML, only "Normal" has builtinId=0; all other built-ins have builtinId > 0.
    /// Custom styles also have builtinId=0 (absent attribute), so we distinguish by name for the zero case.
    pub fn is_builtin_style(&self, style_name: &str) -> bool {
        self.cell_styles.iter().any(|cs| {
            cs.name == style_name
                && (cs.builtin_id > 0 || cs.name.eq_ignore_ascii_case("Normal"))
        })
    }

    /// Removes a named style entry. Does not remove the underlying cell_xfs entry.
    /// Fails if the style does not exist or is a built-in.
    pub(crate) fn delete_named_style_entry(&mut self, style_name: &str) -> Result<(), String> {
        let pos = self
            .cell_styles
            .iter()
            .position(|cs| cs.name == style_name)
            .ok_or_else(|| format!("Style '{style_name}' not found"))?;
        self.cell_styles.remove(pos);
        Ok(())
    }

    /// Updates the xf_id and name of an existing named style entry.
    /// Fails if the style does not exist.
    pub(crate) fn update_named_style_entry(
        &mut self,
        style_name: &str,
        new_name: &str,
        new_xf_id: i32,
    ) -> Result<(), String> {
        let cs = self
            .cell_styles
            .iter_mut()
            .find(|cs| cs.name == style_name)
            .ok_or_else(|| format!("Style '{style_name}' not found"))?;
        cs.name = new_name.to_string();
        cs.xf_id = new_xf_id;
        Ok(())
    }

    // Returns the style index of the style with `quote_prefix=true`
    // If there is no such style it creates it
    // TODO: It needs to be mutable, the name could reflect that
    pub(crate) fn get_style_with_quote_prefix(&mut self, index: i32) -> Result<i32, String> {
        let mut style = self.get_style(index)?;
        style.quote_prefix = true;
        Ok(self.get_style_index_or_create(&style))
    }

    // Returns the index of the style with the provided format.
    // If there is no style with that format, it creates a new one based on the style with the provided index.
    // TODO: It needs to be mutable, the name could reflect that
    pub(crate) fn get_style_with_format(
        &mut self,
        index: i32,
        num_fmt: &str,
    ) -> Result<i32, String> {
        let mut style = self.get_style(index)?;
        style.num_fmt = num_fmt.to_string();
        Ok(self.get_style_index_or_create(&style))
    }

    // Returns the style index of the style with `quote_prefix=false`
    // If there is no such style it creates it
    // TODO: It needs to be mutable, the name could reflect that
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
            .ok_or("Invalid index provided".to_string())?;
        let border_id = cell_xf.border_id as usize;
        let fill_id = cell_xf.fill_id as usize;
        let font_id = cell_xf.font_id as usize;
        let num_fmt_id = cell_xf.num_fmt_id;
        let quote_prefix = cell_xf.quote_prefix;
        let alignment = cell_xf.alignment.clone();

        Ok(Style {
            alignment,
            num_fmt: get_num_fmt(num_fmt_id, &self.num_fmts),
            fill: self.fills[fill_id].clone(),
            font: self.fonts[font_id].clone(),
            border: self.borders[border_id].clone(),
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

    /// Returns the list of all named style names.
    pub fn get_named_style_list(&self) -> Vec<String> {
        self.workbook.styles.get_named_style_list()
    }

    /// Returns the `Style` associated with the named style.
    pub fn get_named_style(&self, name: &str) -> Result<Style, String> {
        let xf_id = self.workbook.styles.get_style_index_by_name(name)?;
        self.workbook.styles.get_style(xf_id)
    }

    /// Creates a new named style. Fails if a style with that name already exists.
    pub fn create_named_style(&mut self, name: &str, style: &Style) -> Result<(), String> {
        self.workbook.styles.create_named_style(name, style)
    }

    /// Deletes a named style. Fails if the style does not exist or is built-in.
    /// Cells that used this style keep their formatting; only the name association is removed.
    pub fn delete_named_style(&mut self, name: &str) -> Result<(), String> {
        if self.workbook.styles.is_builtin_style(name) {
            return Err(format!("Cannot delete built-in style '{name}'"));
        }
        self.workbook.styles.delete_named_style_entry(name)
    }

    /// Updates the formatting and optionally the name of a named style.
    /// All cells, rows, and columns that use the old style are updated to the new formatting.
    /// Fails if the style does not exist, is built-in, or if `new_name` is already taken (when renaming).
    /// Returns `(old_xf_id, new_xf_id)` for diff tracking.
    pub fn update_named_style(
        &mut self,
        name: &str,
        new_name: &str,
        style: &Style,
    ) -> Result<(i32, i32), String> {
        if self.workbook.styles.is_builtin_style(name) {
            return Err(format!("Cannot modify built-in style '{name}'"));
        }
        let old_xf_id = self.workbook.styles.get_style_index_by_name(name)?;
        if name != new_name && self.workbook.styles.get_style_index_by_name(new_name).is_ok() {
            return Err(format!("A style named '{new_name}' already exists"));
        }
        let new_xf_id = self.workbook.styles.get_style_index_or_create(style);
        if old_xf_id != new_xf_id {
            for worksheet in &mut self.workbook.worksheets {
                for row_data in worksheet.sheet_data.values_mut() {
                    for cell in row_data.values_mut() {
                        if cell.get_style() == old_xf_id {
                            cell.set_style(new_xf_id);
                        }
                    }
                }
                for row in &mut worksheet.rows {
                    if row.s == old_xf_id {
                        row.s = new_xf_id;
                    }
                }
                for col in &mut worksheet.cols {
                    if col.style == Some(old_xf_id) {
                        col.style = Some(new_xf_id);
                    }
                }
            }
        }
        self.workbook
            .styles
            .update_named_style_entry(name, new_name, new_xf_id)?;
        Ok((old_xf_id, new_xf_id))
    }
}

impl Dxf {
    /// Applies this differential format on top of `base`, returning the merged style.
    /// Only fields present in the Dxf (i.e. `Some`) override the base.
    pub fn apply_to(&self, base: &Style) -> Style {
        let mut style = base.clone();

        if let Some(ref dxf_fill) = self.fill {
            style.fill = dxf_fill.clone();
        }

        if let Some(ref dxf_font) = self.font {
            // Override color only when it differs from the default (#000000)
            if dxf_font.color != Font::default().color {
                style.font.color = dxf_font.color.clone();
            }
            if let Some(b) = dxf_font.b {
                style.font.b = b;
            }
            if let Some(i) = dxf_font.i {
                style.font.i = i;
            }
            if let Some(u) = dxf_font.u {
                style.font.u = u;
            }
            if let Some(strike) = dxf_font.strike {
                style.font.strike = strike;
            }
        }

        if let Some(ref dxf_border) = self.border {
            style.border = dxf_border.clone();
        }

        if let Some(ref dxf_num_fmt) = self.num_fmt {
            style.num_fmt = dxf_num_fmt.format_code.clone();
        }

        if let Some(ref dxf_alignment) = self.alignment {
            style.alignment = Some(dxf_alignment.clone());
        }

        style
    }
}
