use crate::{
    model::Model,
    number_format::{get_default_num_fmt_id, get_new_num_fmt_index, get_num_fmt},
    types::{Border, CellStyleXfs, CellStyles, CellXfs, Dxf, Fill, Font, NumFmt, Style, Styles},
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

    // Returns `(num_fmt_id, font_id, fill_id, border_id)` for the style,
    // adding any missing components to the pools.
    fn get_or_create_component_ids(&mut self, style: &Style) -> (i32, i32, i32, i32) {
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
        (num_fmt_id, font_id, fill_id, border_id)
    }

    // Creates the base record of a named style: a new entry in `cell_style_xfs`
    // together with its "plain representative" in `cell_xfs` (the xf cells get
    // when the style is applied to them). Returns the new `xf_id`.
    fn create_base_style(&mut self, style: &Style) -> i32 {
        let (num_fmt_id, font_id, fill_id, border_id) = self.get_or_create_component_ids(style);
        // The apply* flags on a cellStyleXfs record mark which formatting
        // categories the style includes; IronCalc styles include all of them
        // (the default), otherwise Excel would treat the style as empty.
        self.cell_style_xfs.push(CellStyleXfs {
            num_fmt_id,
            font_id,
            fill_id,
            border_id,
            ..Default::default()
        });
        let xf_id = self.cell_style_xfs.len() as i32 - 1;
        self.cell_xfs.push(CellXfs {
            xf_id,
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
        xf_id
    }

    pub fn create_new_style(&mut self, style: &Style) -> i32 {
        let (num_fmt_id, font_id, fill_id, border_id) = self.get_or_create_component_ids(style);
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
            // Only anonymous formats qualify: an xf parented to a named style
            // (xf_id != 0) changes when the style is updated, so visually equal
            // formatting must not be deduplicated into it.
            if cell_xf.xf_id != 0 {
                continue;
            }
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

    /// Adds a named cell style pointing to an existing `cell_style_xfs` record.
    /// Fails if the named style already exists or if `xf_id` is not a valid index.
    pub(crate) fn add_named_cell_style(
        &mut self,
        style_name: &str,
        xf_id: i32,
    ) -> Result<(), String> {
        if self.get_xf_id_by_name(style_name).is_ok() {
            return Err("A style with that name already exists".to_string());
        }
        if xf_id < 0 || xf_id as usize >= self.cell_style_xfs.len() {
            return Err("There is no cell style xf with that index".to_string());
        }
        let cell_style = CellStyles {
            name: style_name.to_string(),
            xf_id,
            builtin_id: 0,
        };
        self.cell_styles.push(cell_style);
        Ok(())
    }

    // Returns the `xf_id` (index into `cell_style_xfs`) of the named style or fails.
    // NB: this method is case sensitive
    pub(crate) fn get_xf_id_by_name(&self, style_name: &str) -> Result<i32, String> {
        for cell_style in &self.cell_styles {
            if cell_style.name == style_name {
                return Ok(cell_style.xf_id);
            }
        }
        Err(format!("Style '{style_name}' not found"))
    }

    // Returns the index in `cell_xfs` of the "plain representative" of the named
    // style: an xf parented to the style's `xf_id` carrying no local overrides.
    // This is the index cells get when the style is applied to them.
    // NB: this method is case sensitive
    pub fn get_style_index_by_name(&self, style_name: &str) -> Result<i32, String> {
        let xf_id = self.get_xf_id_by_name(style_name)?;
        for (index, cell_xf) in self.cell_xfs.iter().enumerate() {
            if cell_xf.xf_id == xf_id && !Self::cell_xf_has_overrides(cell_xf) {
                return Ok(index as i32);
            }
        }
        Err(format!("Style '{style_name}' has no plain cell xf"))
    }

    fn cell_xf_has_overrides(cell_xf: &CellXfs) -> bool {
        cell_xf.apply_number_format
            || cell_xf.apply_font
            || cell_xf.apply_fill
            || cell_xf.apply_border
            || cell_xf.apply_alignment
            || cell_xf.apply_protection
    }

    // Same as `get_style_index_by_name` but creates the plain representative if
    // the workbook lacks one (e.g. an imported named style applied to no cell).
    pub(crate) fn get_or_create_style_index_by_name(
        &mut self,
        style_name: &str,
    ) -> Result<i32, String> {
        if let Ok(index) = self.get_style_index_by_name(style_name) {
            return Ok(index);
        }
        let xf_id = self.get_xf_id_by_name(style_name)?;
        let style_xf = self
            .cell_style_xfs
            .get(xf_id as usize)
            .ok_or_else(|| format!("Style '{style_name}' points to an invalid xf id"))?;
        self.cell_xfs.push(CellXfs {
            xf_id,
            num_fmt_id: style_xf.num_fmt_id,
            font_id: style_xf.font_id,
            fill_id: style_xf.fill_id,
            border_id: style_xf.border_id,
            apply_number_format: false,
            apply_border: false,
            apply_alignment: false,
            apply_protection: false,
            apply_font: false,
            apply_fill: false,
            quote_prefix: false,
            alignment: None,
        });
        Ok(self.cell_xfs.len() as i32 - 1)
    }

    // Returns the `Style` of a named style. Reads the plain representative in
    // `cell_xfs` when there is one (it also carries alignment); otherwise the
    // style is reconstructed from its `cell_style_xfs` record.
    pub(crate) fn get_style_by_name(&self, style_name: &str) -> Result<Style, String> {
        if let Ok(index) = self.get_style_index_by_name(style_name) {
            return self.get_style(index);
        }
        let xf_id = self.get_xf_id_by_name(style_name)?;
        let style_xf = self
            .cell_style_xfs
            .get(xf_id as usize)
            .ok_or_else(|| format!("Style '{style_name}' points to an invalid xf id"))?;
        Ok(Style {
            alignment: None,
            num_fmt: get_num_fmt(style_xf.num_fmt_id, &self.num_fmts),
            fill: self.fills[style_xf.fill_id as usize].clone(),
            font: self.fonts[style_xf.font_id as usize].clone(),
            border: self.borders[style_xf.border_id as usize].clone(),
            quote_prefix: false,
        })
    }

    pub fn create_named_style(&mut self, style_name: &str, style: &Style) -> Result<(), String> {
        if self.get_xf_id_by_name(style_name).is_ok() {
            return Err("A style with that name already exists".to_string());
        }
        let xf_id = self.create_base_style(style);
        self.add_named_cell_style(style_name, xf_id)
    }

    /// Returns the names of all named styles
    pub fn get_named_style_list(&self) -> Vec<String> {
        self.cell_styles.iter().map(|cs| cs.name.clone()).collect()
    }

    /// Returns true if the named style is built-in and cannot be deleted or modified.
    /// In OOXML, only "Normal" has builtinId=0; all other built-ins have builtinId > 0.
    /// Custom styles also have builtinId=0 (absent attribute), so we distinguish by name for the zero case.
    pub fn is_builtin_style(&self, style_name: &str) -> bool {
        self.cell_styles.iter().any(|cs| {
            cs.name == style_name && (cs.builtin_id > 0 || cs.name.eq_ignore_ascii_case("Normal"))
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

    /// Renames an existing named style entry. Its `xf_id` never changes.
    /// Fails if the style does not exist.
    pub(crate) fn rename_named_style_entry(
        &mut self,
        style_name: &str,
        new_name: &str,
    ) -> Result<(), String> {
        let cs = self
            .cell_styles
            .iter_mut()
            .find(|cs| cs.name == style_name)
            .ok_or_else(|| format!("Style '{style_name}' not found"))?;
        cs.name = new_name.to_string();
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
        let style_index = self
            .workbook
            .styles
            .get_or_create_style_index_by_name(style_name)?;
        self.workbook
            .worksheet_mut(sheet)?
            .set_cell_style(row, column, style_index)
    }

    pub fn set_sheet_style(&mut self, sheet: u32, style_name: &str) -> Result<(), String> {
        let style_index = self
            .workbook
            .styles
            .get_or_create_style_index_by_name(style_name)?;
        self.workbook.worksheet_mut(sheet)?.set_style(style_index)?;
        Ok(())
    }

    pub fn set_sheet_row_style(
        &mut self,
        sheet: u32,
        row: i32,
        style_name: &str,
    ) -> Result<(), String> {
        let style_index = self
            .workbook
            .styles
            .get_or_create_style_index_by_name(style_name)?;
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
        let style_index = self
            .workbook
            .styles
            .get_or_create_style_index_by_name(style_name)?;
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
        self.workbook.styles.get_style_by_name(name)
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
    /// The style's `xf_id` never changes:
    /// * The style's `cell_style_xfs` record is rewritten in place.
    /// * The new components are propagated to every `cell_xfs` entry parented to
    ///   the style, except those a cell overrides locally (its `apply_*` flags).
    ///
    /// Cells keep their style index, so they pick up the new formatting without
    /// being touched.
    pub fn update_named_style(
        &mut self,
        name: &str,
        new_name: &str,
        style: &Style,
    ) -> Result<(), String> {
        let styles = &mut self.workbook.styles;
        if styles.is_builtin_style(name) {
            return Err(format!("Cannot modify built-in style '{name}'"));
        }
        let xf_id = styles.get_xf_id_by_name(name)?;
        if name != new_name && styles.get_xf_id_by_name(new_name).is_ok() {
            return Err(format!("A style named '{new_name}' already exists"));
        }
        if xf_id < 0 || xf_id as usize >= styles.cell_style_xfs.len() {
            return Err(format!("Style '{name}' points to an invalid xf id"));
        }

        let (num_fmt_id, font_id, fill_id, border_id) = styles.get_or_create_component_ids(style);

        styles.cell_style_xfs[xf_id as usize] = CellStyleXfs {
            num_fmt_id,
            font_id,
            fill_id,
            border_id,
            ..Default::default()
        };

        for cell_xf in styles.cell_xfs.iter_mut().filter(|xf| xf.xf_id == xf_id) {
            if !cell_xf.apply_number_format {
                cell_xf.num_fmt_id = num_fmt_id;
            }
            if !cell_xf.apply_font {
                cell_xf.font_id = font_id;
            }
            if !cell_xf.apply_fill {
                cell_xf.fill_id = fill_id;
            }
            if !cell_xf.apply_border {
                cell_xf.border_id = border_id;
            }
            if !cell_xf.apply_alignment {
                cell_xf.alignment = style.alignment.clone();
            }
        }

        if name != new_name {
            styles.rename_named_style_entry(name, new_name)?;
        }
        Ok(())
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
