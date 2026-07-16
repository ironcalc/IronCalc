use crate::collab::DynError;
use crate::types::Workbook;
use crate::user_model::history::Diff;

pub enum Patch {}

fn diff_to_patch(workbook: &Workbook, diff: Diff) {
    match diff {
        Diff::SetCellValue {
            sheet,
            row,
            column,
            new_value,
            old_value,
        } => {}
        Diff::SetArrayValue {
            sheet,
            row,
            column,
            width,
            height,
            new_value,
            old_values,
        } => {}
        Diff::RangeClearContents {
            sheet,
            row,
            column,
            width,
            height,
            old_value,
        } => {}
        Diff::RangeClearAll {
            sheet,
            row,
            column,
            width,
            height,
            old_value,
            old_style,
        } => {}
        Diff::CellClearFormatting {
            sheet,
            row,
            column,
            old_style,
        } => {}
        Diff::SetCellStyle {
            sheet,
            row,
            column,
            old_value,
            new_value,
        } => {}
        Diff::SetColumnWidth {
            sheet,
            column,
            new_value,
            old_value,
        } => {}
        Diff::SetColumnHidden {
            sheet,
            column,
            new_value,
            old_value,
        } => {}
        Diff::SetRowHeight {
            sheet,
            row,
            new_value,
            old_value,
        } => {}
        Diff::SetRowHidden {
            sheet,
            row,
            new_value,
            old_value,
        } => {}
        Diff::SetColumnStyle {
            sheet,
            column,
            old_value,
            new_value,
        } => {}
        Diff::SetRowStyle {
            sheet,
            row,
            old_value,
            new_value,
        } => {}
        Diff::DeleteColumnStyle {
            sheet,
            column,
            old_value,
        } => {}
        Diff::DeleteRowStyle {
            sheet,
            row,
            old_value,
        } => {}
        Diff::InsertRows { sheet, row, count } => {}
        Diff::DeleteRows {
            sheet,
            row,
            count,
            old_data,
        } => {}
        Diff::InsertColumns {
            sheet,
            column,
            count,
        } => {}
        Diff::DeleteColumns {
            sheet,
            column,
            count,
            old_data,
        } => {}
        Diff::DeleteSheet { sheet, old_data } => {}
        Diff::SetFrozenRowsCount {
            sheet,
            new_value,
            old_value,
        } => {}
        Diff::SetFrozenColumnsCount {
            sheet,
            new_value,
            old_value,
        } => {}
        Diff::NewSheet { index, name } => {}
        Diff::DuplicateSheet {
            source_index,
            new_index,
        } => {}
        Diff::RenameSheet {
            index,
            old_value,
            new_value,
        } => {}
        Diff::SetSheetColor {
            index,
            old_value,
            new_value,
        } => {}
        Diff::SetSheetState {
            index,
            old_value,
            new_value,
        } => {}
        Diff::SetShowGridLines {
            sheet,
            old_value,
            new_value,
        } => {}
        Diff::SetTheme {
            old_value,
            new_value,
        } => {}
        Diff::CreateDefinedName { name, scope, value } => {}
        Diff::DeleteDefinedName {
            name,
            scope,
            old_value,
        } => {}
        Diff::UpdateDefinedName {
            name,
            scope,
            old_formula,
            new_name,
            new_scope,
            new_formula,
        } => {}
        Diff::MoveColumns {
            sheet,
            column,
            column_count,
            delta,
        } => {}
        Diff::MoveRows {
            sheet,
            row,
            row_count,
            delta,
        } => {}
        Diff::SetLocale {
            old_value,
            new_value,
        } => {}
        Diff::SetTimezone {
            old_value,
            new_value,
        } => {}
        Diff::CreateNamedStyle { name, xf_id } => {}
        Diff::DeleteNamedStyle { name, old_xf_id } => {}
        Diff::UpdateNamedStyle {
            name,
            new_name,
            old_xf_id,
            new_xf_id,
        } => {}
        Diff::AddConditionalFormatting {
            sheet,
            range,
            rule,
            priority,
        } => {}
        Diff::DeleteConditionalFormatting {
            sheet,
            index,
            old_range,
            old_rule,
            old_priority,
        } => {}
        Diff::UpdateConditionalFormatting {
            sheet,
            index,
            old_range,
            old_rule,
            old_priority,
            new_range,
            new_rule,
        } => {}
        Diff::SwapConditionalFormattingPriority {
            sheet,
            index_a,
            index_b,
            priority_a,
            priority_b,
        } => {}
    }
}

fn apply_patch(&mut self, patch: Patch) -> Result<(), DynError> {
    todo!()
}
