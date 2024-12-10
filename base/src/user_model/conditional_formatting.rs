use crate::{
    cf_types::{CfRule, CfRuleInput, ConditionalFormatting},
    types::Dxf,
};

use super::{common::UserModel, history::Diff};

impl<'a> UserModel<'a> {
    /// Returns all CF rules for the given sheet in list order.
    pub fn get_conditional_formatting_list(
        &self,
        sheet: u32,
    ) -> Result<Vec<ConditionalFormatting>, String> {
        self.model.get_conditional_formatting_list(sheet)
    }

    /// Returns the differential format (Dxf) for the CF rule at `index` on `sheet`,
    /// or `None` if the rule type has no associated dxf (e.g. ColorScale, DataBar).
    pub fn get_dxf_for_conditional_formatting(
        &self,
        sheet: u32,
        index: u32,
    ) -> Result<Option<Dxf>, String> {
        self.model
            .get_dxf_for_conditional_formatting(sheet, index as usize)
    }

    /// Adds a new CF rule to `sheet`.
    pub fn add_conditional_formatting(
        &mut self,
        sheet: u32,
        range: &str,
        rule: CfRuleInput,
    ) -> Result<(), String> {
        let priority = self.model.add_conditional_formatting(sheet, range, rule)?;
        // Read back the stored entry so the Diff contains the dxf_id that was assigned.
        let (stored_rule, stop_if_true) = self
            .model
            .workbook
            .worksheet(sheet)
            .ok()
            .and_then(|ws| {
                ws.conditional_formatting
                    .iter()
                    .find(|cf| cf.priority == priority)
                    .map(|cf| (cf.cf_rule.clone(), cf.stop_if_true))
            })
            .unwrap_or((CfRule::DuplicateValues { dxf_id: 0 }, false));
        self.push_diff_list(vec![Diff::AddConditionalFormatting {
            sheet,
            range: range.to_string(),
            rule: Box::new(stored_rule),
            priority,
            stop_if_true,
        }]);
        self.evaluate_if_not_paused();
        Ok(())
    }

    /// Removes the CF rule at `index` from `sheet`.
    pub fn delete_conditional_formatting(&mut self, sheet: u32, index: u32) -> Result<(), String> {
        let old = self
            .model
            .delete_conditional_formatting(sheet, index as usize)?;
        self.push_diff_list(vec![Diff::DeleteConditionalFormatting {
            sheet,
            index,
            old_range: old.range,
            old_rule: Box::new(old.cf_rule),
            old_priority: old.priority,
            old_stop_if_true: old.stop_if_true,
        }]);
        self.evaluate_if_not_paused();
        Ok(())
    }

    /// Replaces the range and rule of the CF entry at `index` on `sheet`.
    pub fn update_conditional_formatting(
        &mut self,
        sheet: u32,
        index: u32,
        new_range: &str,
        new_rule: CfRuleInput,
    ) -> Result<(), String> {
        let old =
            self.model
                .update_conditional_formatting(sheet, index as usize, new_range, new_rule)?;
        // Read back the stored entry so the Diff contains the dxf_id that was assigned.
        let (stored_rule, new_stop_if_true) = self
            .model
            .workbook
            .worksheet(sheet)
            .ok()
            .and_then(|ws| {
                ws.conditional_formatting
                    .get(index as usize)
                    .map(|cf| (cf.cf_rule.clone(), cf.stop_if_true))
            })
            .unwrap_or((CfRule::DuplicateValues { dxf_id: 0 }, false));
        self.push_diff_list(vec![Diff::UpdateConditionalFormatting {
            sheet,
            index,
            old_range: old.range,
            old_rule: Box::new(old.cf_rule),
            old_priority: old.priority,
            old_stop_if_true: old.stop_if_true,
            new_range: new_range.to_string(),
            new_rule: Box::new(stored_rule),
            new_stop_if_true,
        }]);
        self.evaluate_if_not_paused();
        Ok(())
    }
}
