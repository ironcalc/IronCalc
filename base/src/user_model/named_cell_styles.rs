use crate::{types::Style, user_model::history::Diff, UserModel};

impl<'a> UserModel<'a> {
    /// Returns the list of all named style names.
    pub fn get_named_style_list(&self) -> Vec<String> {
        self.model.get_named_style_list()
    }

    /// Returns the `Style` associated with the named style.
    pub fn get_named_style(&self, name: &str) -> Result<Style, String> {
        self.model.get_named_style(name)
    }

    /// Creates a new named style. Fails if a style with that name already exists.
    pub fn create_named_style(&mut self, name: &str, style: &Style) -> Result<(), String> {
        self.model.create_named_style(name, style)?;
        self.push_diff_list(vec![Diff::CreateNamedStyle {
            name: name.to_string(),
            style: Box::new(style.clone()),
        }]);
        Ok(())
    }

    /// Deletes a named style. Fails if the style does not exist or is built-in.
    /// Cells that used this style keep their formatting.
    pub fn delete_named_style(&mut self, name: &str) -> Result<(), String> {
        let old_xf_id = self.model.workbook.styles.get_xf_id_by_name(name)?;
        self.model.delete_named_style(name)?;
        self.push_diff_list(vec![Diff::DeleteNamedStyle {
            name: name.to_string(),
            old_xf_id,
        }]);
        Ok(())
    }

    /// Updates the formatting and optionally the name of a named style.
    /// All cells, rows, and columns using the style pick up the new formatting.
    /// Fails if the style does not exist, is built-in, or if `new_name` is already taken.
    pub fn update_named_style(
        &mut self,
        name: &str,
        new_name: &str,
        style: &Style,
    ) -> Result<(), String> {
        let old_style = self.model.get_named_style(name)?;
        self.model.update_named_style(name, new_name, style)?;
        self.push_diff_list(vec![Diff::UpdateNamedStyle {
            name: name.to_string(),
            new_name: new_name.to_string(),
            old_style: Box::new(old_style),
            new_style: Box::new(style.clone()),
        }]);
        Ok(())
    }

    /// Returns all Excel built-in named styles as `(name, Style)` pairs.
    /// These are static definitions that are not stored in the workbook until a user applies one.
    pub fn get_builtin_named_styles(&self) -> Vec<(String, Style)> {
        crate::builtin_styles::builtin_named_styles()
    }

    /// Applies a named style (custom or built-in) to the current selection.
    ///
    /// If the style is not in the model but is a known built-in, it is first added to the
    /// model's style table, then applied to every cell in the selection with undo support.
    pub fn on_apply_named_style(&mut self, name: &str) -> Result<(), String> {
        let mut diff_list = Vec::new();

        // Ensure the style exists in the model, adding it from builtins if needed.
        // Only fall back to the builtins when the name is genuinely absent;
        // errors on an existing style (e.g. an invalid xf id in a malformed
        // workbook) must surface as-is.
        let style_index = if self.model.workbook.styles.get_xf_id_by_name(name).is_ok() {
            self.model
                .workbook
                .styles
                .get_or_create_style_index_by_name(name)?
        } else {
            let style = crate::builtin_styles::get_builtin_style(name)
                .ok_or_else(|| format!("Named style '{name}' not found"))?;
            self.model
                .workbook
                .styles
                .create_named_style(name, &style)?;
            let style_index = self.model.workbook.styles.get_style_index_by_name(name)?;
            diff_list.push(Diff::CreateNamedStyle {
                name: name.to_string(),
                style: Box::new(style),
            });
            style_index
        };

        // Resolve the selection range.
        let sheet = if let Some(view) = self.model.workbook.views.get(&self.model.view_id) {
            view.sheet
        } else {
            return Ok(());
        };
        let range = if let Ok(worksheet) = self.model.workbook.worksheet(sheet) {
            if let Some(view) = worksheet.views.get(&self.model.view_id) {
                view.range
            } else {
                return Ok(());
            }
        } else {
            return Ok(());
        };

        let [row_start, column_start, row_end, column_end] = range;
        for row in row_start..=row_end {
            for column in column_start..=column_end {
                let old_value = self.model.get_cell_style_or_none(sheet, row, column)?;
                self.model.workbook.worksheet_mut(sheet)?.set_cell_style(
                    row,
                    column,
                    style_index,
                )?;
                diff_list.push(Diff::ApplyNamedStyle {
                    sheet,
                    row,
                    column,
                    old_value: Box::new(old_value),
                    name: name.to_string(),
                });
            }
        }
        self.push_diff_list(diff_list);
        Ok(())
    }
}
