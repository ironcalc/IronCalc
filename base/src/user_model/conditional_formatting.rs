use crate::cf_types::{CfRule, ConditionalFormatting};

use super::{common::UserModel, history::Diff};

impl<'a> UserModel<'a> {
    /// Returns all CF rules for the given sheet in list order.
    pub fn get_conditional_formatting_list(
        &self,
        sheet: u32,
    ) -> Result<Vec<ConditionalFormatting>, String> {
        self.model.get_conditional_formatting_list(sheet)
    }

    /// Adds a new CF rule to `sheet`.
    pub fn add_conditional_formatting(
        &mut self,
        sheet: u32,
        range: &str,
        rule: CfRule,
    ) -> Result<(), String> {
        let priority = self
            .model
            .add_conditional_formatting(sheet, range, rule.clone())?;
        self.push_diff_list(vec![Diff::AddConditionalFormatting {
            sheet,
            range: range.to_string(),
            rule: Box::new(rule),
            priority,
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
        new_rule: CfRule,
    ) -> Result<(), String> {
        let old = self.model.update_conditional_formatting(
            sheet,
            index as usize,
            new_range,
            new_rule.clone(),
        )?;
        self.push_diff_list(vec![Diff::UpdateConditionalFormatting {
            sheet,
            index,
            old_range: old.range,
            old_rule: Box::new(old.cf_rule),
            old_priority: old.priority,
            new_range: new_range.to_string(),
            new_rule: Box::new(new_rule),
        }]);
        self.evaluate_if_not_paused();
        Ok(())
    }
}
