use crate::{
    constants::{LAST_COLUMN, LAST_ROW},
    expressions::types::Area,
    types::Style,
};

use super::{
    border_utils::is_max_border, common::BorderType, history::Diff, BorderArea, UserModel,
};

impl UserModel {
    fn get_new_border_value(
        &self,
        border_area: &BorderArea,
        old_value: &Option<Style>,
        row: i32,
        column: i32,
        range: (i32, i32, i32, i32),
    ) -> Result<Style, String> {
        let mut new_value = match old_value {
            Some(value) => value.clone(),
            None => Default::default(),
        };
        let (first_row, first_column, last_row, last_column) = range;

        match border_area.r#type {
            BorderType::All => {
                new_value.border.top = Some(border_area.item.clone());
                new_value.border.right = Some(border_area.item.clone());
                new_value.border.bottom = Some(border_area.item.clone());
                new_value.border.left = Some(border_area.item.clone());
            }
            BorderType::Inner => {
                if row != first_row {
                    new_value.border.top = Some(border_area.item.clone());
                }
                if row != last_row {
                    new_value.border.bottom = Some(border_area.item.clone());
                }
                if column != first_column {
                    new_value.border.left = Some(border_area.item.clone());
                }
                if column != last_column {
                    new_value.border.right = Some(border_area.item.clone());
                }
            }
            BorderType::Outer => {
                if row == first_row {
                    new_value.border.top = Some(border_area.item.clone());
                }
                if row == last_row {
                    new_value.border.bottom = Some(border_area.item.clone());
                }
                if column == first_column {
                    new_value.border.left = Some(border_area.item.clone());
                }
                if column == last_column {
                    new_value.border.right = Some(border_area.item.clone());
                }
            }
            BorderType::Top => {
                if row == first_row {
                    new_value.border.top = Some(border_area.item.clone());
                }
            }
            BorderType::Right => {
                if column == last_column {
                    new_value.border.right = Some(border_area.item.clone());
                }
            }
            BorderType::Bottom => {
                if row == last_row {
                    new_value.border.bottom = Some(border_area.item.clone());
                }
            }
            BorderType::Left => {
                if column == first_column {
                    new_value.border.left = Some(border_area.item.clone());
                }
            }
            BorderType::CenterH => {
                if row != first_row {
                    new_value.border.top = Some(border_area.item.clone());
                }
                if row != last_row {
                    new_value.border.bottom = Some(border_area.item.clone());
                }
            }
            BorderType::CenterV => {
                if column != first_column {
                    new_value.border.left = Some(border_area.item.clone());
                }
                if column != last_column {
                    new_value.border.right = Some(border_area.item.clone());
                }
            }
            BorderType::None => {
                new_value.border.top = None;
                new_value.border.right = None;
                new_value.border.bottom = None;
                new_value.border.left = None;
            }
        }
        Ok(new_value)
    }

    fn set_rows_with_border(
        &mut self,
        sheet: u32,
        first_row: i32,
        last_row: i32,
        border_area: &BorderArea,
    ) -> Result<(), String> {
        let mut diff_list = Vec::new();
        for row in first_row..=last_row {
            let old_value = self.model.get_row_style(sheet, row)?;
            let mut new_value = match &old_value {
                Some(value) => value.clone(),
                None => Default::default(),
            };

            match border_area.r#type {
                BorderType::All => {
                    new_value.border.top = Some(border_area.item.clone());
                    new_value.border.right = Some(border_area.item.clone());
                    new_value.border.bottom = Some(border_area.item.clone());
                    new_value.border.left = Some(border_area.item.clone());
                }
                BorderType::Inner => {
                    if row != first_row {
                        new_value.border.top = Some(border_area.item.clone());
                    }
                    if row != last_row {
                        new_value.border.bottom = Some(border_area.item.clone());
                    }
                }
                BorderType::Outer => {
                    if row == first_row {
                        new_value.border.top = Some(border_area.item.clone());
                    }
                    if row == last_row {
                        new_value.border.bottom = Some(border_area.item.clone());
                    }
                }
                BorderType::Top => {
                    if row == first_row {
                        new_value.border.top = Some(border_area.item.clone());
                    }
                }
                BorderType::Right => {
                    // noop
                }
                BorderType::Bottom => {
                    if row == last_row {
                        new_value.border.bottom = Some(border_area.item.clone());
                    }
                }
                BorderType::Left => {
                    // noop
                }
                BorderType::CenterH => {
                    if row != first_row {
                        new_value.border.top = Some(border_area.item.clone());
                    }
                    if row != last_row {
                        new_value.border.bottom = Some(border_area.item.clone());
                    }
                }
                BorderType::CenterV => {
                    new_value.border.left = Some(border_area.item.clone());
                    new_value.border.right = Some(border_area.item.clone());
                }
                BorderType::None => {
                    new_value.border.top = None;
                    new_value.border.right = None;
                    new_value.border.bottom = None;
                    new_value.border.left = None;
                }
            }

            // We need to go throw each non-empty cell in the row
            let columns: Vec<i32> = self
                .model
                .workbook
                .worksheet(sheet)?
                .sheet_data
                .get(&row)
                .map(|row_data| row_data.keys().copied().collect())
                .unwrap_or_default();
            for column in columns {
                let old_value = self.model.get_cell_style_or_none(sheet, row, column)?;
                let new_value = self.get_new_border_value(
                    border_area,
                    &old_value,
                    row,
                    column,
                    (first_row, 1, last_row, LAST_COLUMN),
                )?;

                self.model.set_cell_style(sheet, row, column, &new_value)?;
                diff_list.push(Diff::SetCellStyle {
                    sheet,
                    row,
                    column,
                    old_value: Box::new(old_value),
                    new_value: Box::new(new_value),
                });
            }

            self.model.set_row_style(sheet, row, &new_value)?;
            diff_list.push(Diff::SetRowStyle {
                sheet,
                row,
                old_value: Box::new(old_value),
                new_value: Box::new(new_value),
            });
        }
        self.push_diff_list(diff_list);
        Ok(())
    }

    fn set_columns_with_border(
        &mut self,
        sheet: u32,
        first_column: i32,
        last_column: i32,
        border_area: &BorderArea,
    ) -> Result<(), String> {
        let mut diff_list = Vec::new();
        // We need all the rows in the column to update the style
        // NB: This is too much, this is all the rows that have values
        let data_rows: Vec<i32> = self
            .model
            .workbook
            .worksheet(sheet)?
            .sheet_data
            .keys()
            .copied()
            .collect();
        let styled_rows = &self.model.workbook.worksheet(sheet)?.rows.clone();
        for column in first_column..=last_column {
            let old_value = self.model.get_column_style(sheet, column)?;
            let mut new_value = match &old_value {
                Some(value) => value.clone(),
                None => Default::default(),
            };

            match border_area.r#type {
                BorderType::All => {
                    new_value.border.top = Some(border_area.item.clone());
                    new_value.border.right = Some(border_area.item.clone());
                    new_value.border.bottom = Some(border_area.item.clone());
                    new_value.border.left = Some(border_area.item.clone());
                }
                BorderType::Inner => {
                    if column != first_column {
                        new_value.border.left = Some(border_area.item.clone());
                    }
                    if column != last_column {
                        new_value.border.right = Some(border_area.item.clone());
                    }
                }
                BorderType::Outer => {
                    if column == first_column {
                        new_value.border.left = Some(border_area.item.clone());
                    }
                    if column == last_column {
                        new_value.border.right = Some(border_area.item.clone());
                    }
                }
                BorderType::Top => {
                    // noop
                }
                BorderType::Right => {
                    if column == last_column {
                        new_value.border.right = Some(border_area.item.clone());
                    }
                }
                BorderType::Bottom => {
                    // noop
                }
                BorderType::Left => {
                    if column == first_column {
                        new_value.border.left = Some(border_area.item.clone());
                    }
                }
                BorderType::CenterH => {
                    new_value.border.top = Some(border_area.item.clone());
                    new_value.border.bottom = Some(border_area.item.clone());
                }
                BorderType::CenterV => {
                    if column != first_column {
                        new_value.border.left = Some(border_area.item.clone());
                    }
                    if column != last_column {
                        new_value.border.right = Some(border_area.item.clone());
                    }
                }
                BorderType::None => {
                    new_value.border.top = None;
                    new_value.border.right = None;
                    new_value.border.bottom = None;
                    new_value.border.left = None;
                }
            }
            // We need to go through each non empty cell in the column
            for &row in &data_rows {
                if let Some(data_row) = self.model.workbook.worksheet(sheet)?.sheet_data.get(&row) {
                    if data_row.get(&column).is_some() {
                        let old_value = self.model.get_cell_style_or_none(sheet, row, column)?;
                        let new_value = self.get_new_border_value(
                            border_area,
                            &old_value,
                            row,
                            column,
                            (1, first_column, LAST_ROW, last_column),
                        )?;

                        self.model.set_cell_style(sheet, row, column, &new_value)?;
                        diff_list.push(Diff::SetCellStyle {
                            sheet,
                            row,
                            column,
                            old_value: Box::new(old_value),
                            new_value: Box::new(new_value),
                        });
                    }
                }
            }

            // We also need to overwrite those that have a row style
            for row_s in styled_rows.iter() {
                let row = row_s.r;
                let old_value = self.model.get_cell_style_or_none(sheet, row, column)?;
                let new_value = self.get_new_border_value(
                    border_area,
                    &old_value,
                    row,
                    column,
                    (1, first_column, LAST_ROW, last_column),
                )?;

                self.model.set_cell_style(sheet, row, column, &new_value)?;
                diff_list.push(Diff::SetCellStyle {
                    sheet,
                    row,
                    column,
                    old_value: Box::new(old_value),
                    new_value: Box::new(new_value),
                });
            }

            self.model.set_column_style(sheet, column, &new_value)?;
            diff_list.push(Diff::SetColumnStyle {
                sheet,
                column,
                old_value: Box::new(old_value),
                new_value: Box::new(new_value),
            });
        }
        self.push_diff_list(diff_list);
        Ok(())
    }

    /// Sets the border in an area of cells.
    /// When setting the border we need to check if the adjacent cells have a "heavier" border
    /// If that is the case we need to change it
    pub fn set_area_with_border(
        &mut self,
        range: &Area,
        border_area: &BorderArea,
    ) -> Result<(), String> {
        let sheet = range.sheet;
        let first_row = range.row;
        let first_column = range.column;
        let last_row = first_row + range.height - 1;
        let last_column = first_column + range.width - 1;
        if first_row == 1 && last_row == LAST_ROW {
            // full columns
            self.set_columns_with_border(sheet, first_column, last_column, border_area)?;
            return Ok(());
        }
        if first_column == 1 && last_column == LAST_COLUMN {
            // full rows
            self.set_rows_with_border(sheet, first_row, last_row, border_area)?;
            return Ok(());
        }
        let mut diff_list = Vec::new();
        for row in first_row..=last_row {
            for column in first_column..=last_column {
                let old_value = self.model.get_cell_style_or_none(sheet, row, column)?;
                let new_value = self.get_new_border_value(
                    border_area,
                    &old_value,
                    row,
                    column,
                    (first_row, first_column, last_row, last_column),
                )?;

                self.model.set_cell_style(sheet, row, column, &new_value)?;
                diff_list.push(Diff::SetCellStyle {
                    sheet,
                    row,
                    column,
                    old_value: Box::new(old_value),
                    new_value: Box::new(new_value),
                });
            }
        }

        // bottom of the cells above the first
        if first_row > 1
            && [
                BorderType::All,
                BorderType::None,
                BorderType::Outer,
                BorderType::Top,
            ]
            .contains(&border_area.r#type)
        {
            let row = first_row - 1;
            for column in first_column..=last_column {
                let old_value = self.model.get_style_for_cell(sheet, row, column)?;
                if is_max_border(Some(&border_area.item), old_value.border.bottom.as_ref()) {
                    let mut style = old_value.clone();
                    if border_area.r#type == BorderType::None {
                        style.border.bottom = None;
                    } else {
                        style.border.bottom = Some(border_area.item.clone());
                    }
                    self.model.set_cell_style(sheet, row, column, &style)?;
                    diff_list.push(Diff::SetCellStyle {
                        sheet,
                        row,
                        column,
                        old_value: Box::new(Some(old_value)),
                        new_value: Box::new(style),
                    });
                }
            }
        }
        // Cells to the right
        if last_column < LAST_COLUMN
            && [
                BorderType::All,
                BorderType::None,
                BorderType::Outer,
                BorderType::Right,
            ]
            .contains(&border_area.r#type)
        {
            let column = last_column + 1;
            for row in first_row..=last_row {
                let old_value = self.model.get_style_for_cell(sheet, row, column)?;
                // If the border in the adjacent cell is "heavier" we change it
                if is_max_border(Some(&border_area.item), old_value.border.left.as_ref()) {
                    let mut style = old_value.clone();
                    if border_area.r#type == BorderType::None {
                        style.border.left = None;
                    } else {
                        style.border.left = Some(border_area.item.clone());
                    }
                    self.model.set_cell_style(sheet, row, column, &style)?;
                    diff_list.push(Diff::SetCellStyle {
                        sheet,
                        row,
                        column,
                        old_value: Box::new(Some(old_value)),
                        new_value: Box::new(style),
                    });
                }
            }
        }
        // Cells bellow
        if last_row < LAST_ROW
            && [
                BorderType::All,
                BorderType::None,
                BorderType::Outer,
                BorderType::Bottom,
            ]
            .contains(&border_area.r#type)
        {
            let row = last_row + 1;
            for column in first_column..=last_column {
                let old_value = self.model.get_style_for_cell(sheet, row, column)?;
                if is_max_border(Some(&border_area.item), old_value.border.top.as_ref()) {
                    let mut style = old_value.clone();
                    if border_area.r#type == BorderType::None {
                        style.border.top = None;
                    } else {
                        style.border.top = Some(border_area.item.clone());
                    }
                    self.model.set_cell_style(sheet, row, column, &style)?;
                    diff_list.push(Diff::SetCellStyle {
                        sheet,
                        row,
                        column,
                        old_value: Box::new(Some(old_value)),
                        new_value: Box::new(style),
                    });
                }
            }
        }
        // Cells to the left
        if first_column > 1
            && [
                BorderType::All,
                BorderType::None,
                BorderType::Outer,
                BorderType::Left,
            ]
            .contains(&border_area.r#type)
        {
            let column = first_column - 1;
            for row in first_row..=last_row {
                let old_value = self.model.get_style_for_cell(sheet, row, column)?;
                if is_max_border(Some(&border_area.item), old_value.border.right.as_ref()) {
                    let mut style = old_value.clone();
                    if border_area.r#type == BorderType::None {
                        style.border.right = None;
                    } else {
                        style.border.right = Some(border_area.item.clone());
                    }
                    self.model.set_cell_style(sheet, row, column, &style)?;
                    diff_list.push(Diff::SetCellStyle {
                        sheet,
                        row,
                        column,
                        old_value: Box::new(Some(old_value)),
                        new_value: Box::new(style),
                    });
                }
            }
        }

        self.push_diff_list(diff_list);
        Ok(())
    }
}
