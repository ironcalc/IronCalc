// Furigana (phonetic runs) of the text in a cell.
//
// The runs belong to a shared string (see `Workbook::phonetic_runs`), as in xlsx files.
// The same text can have different readings in different cells (山田 can be read ヤマダ or
// サンダ), so like Excel a text with other furigana gets its own shared string entry.

use crate::{
    model::Model,
    types::{Cell, PhoneticRun},
};

/// Checks that the runs are inside the text, in order and do not overlap
fn validate_runs(text: &str, runs: &[PhoneticRun]) -> Result<(), String> {
    let len = text.encode_utf16().count() as u32;
    let mut previous_end = 0;
    for run in runs {
        if run.start >= run.end || run.end > len {
            return Err(format!(
                "Invalid phonetic run {}..{} for a text of length {len}",
                run.start, run.end
            ));
        }
        if run.start < previous_end {
            return Err("Phonetic runs must be in order and not overlap".to_string());
        }
        if run.text.is_empty() {
            return Err("A phonetic run needs a reading".to_string());
        }
        previous_end = run.end;
    }
    Ok(())
}

impl<'a> Model<'a> {
    /// Returns the furigana of the text in a cell (empty if there are none)
    pub fn get_cell_phonetic_runs(
        &self,
        sheet: u32,
        row: i32,
        column: i32,
    ) -> Result<Vec<PhoneticRun>, String> {
        let worksheet = self.workbook.worksheet(sheet)?;
        if let Some(cell) = worksheet.cell(row, column) {
            if let Cell::SharedString { si, .. } = *cell {
                return Ok(self
                    .workbook
                    .phonetic_runs
                    .get(&si)
                    .cloned()
                    .unwrap_or_default());
            }
        }
        Ok(vec![])
    }

    /// Sets the furigana of the text in a cell. The cell must hold a text.
    /// The positions of the runs are UTF-16 positions in the text. No runs removes them.
    pub fn set_cell_phonetic_runs(
        &mut self,
        sheet: u32,
        row: i32,
        column: i32,
        runs: Vec<PhoneticRun>,
    ) -> Result<(), String> {
        let (si, style) = match self.workbook.worksheet(sheet)?.cell(row, column) {
            Some(cell) => match *cell {
                Cell::SharedString { si, s } => (si, s),
                _ => return Err("The cell does not hold a text".to_string()),
            },
            None => return Err("The cell is empty".to_string()),
        };
        let text = self
            .workbook
            .shared_strings
            .get(si as usize)
            .ok_or("Invalid shared string")?
            .clone();
        validate_runs(&text, &runs)?;
        let wanted = if runs.is_empty() { None } else { Some(&runs) };
        let same_runs = |index: i32| self.workbook.phonetic_runs.get(&index) == wanted;
        if same_runs(si) {
            return Ok(());
        }
        // Another entry with the same text and furigana, or a new one. Other cells with this
        // text keep their own furigana. The index from text to shared string is not updated,
        // so a text typed later without furigana keeps using the entry it used before.
        let existing = self
            .workbook
            .shared_strings
            .iter()
            .enumerate()
            .position(|(index, s)| *s == text && same_runs(index as i32));
        let new_si = match existing {
            Some(index) => index as i32,
            None => {
                let index = self.workbook.shared_strings.len() as i32;
                self.workbook.shared_strings.push(text);
                if !runs.is_empty() {
                    self.workbook.phonetic_runs.insert(index, runs);
                }
                index
            }
        };
        self.workbook
            .worksheet_mut(sheet)?
            .set_cell_with_string(row, column, new_si, style)
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]
    use super::*;
    use crate::test::util::new_empty_model;

    fn run(start: u32, end: u32, text: &str) -> PhoneticRun {
        PhoneticRun {
            start,
            end,
            text: text.to_string(),
        }
    }

    #[test]
    fn set_and_get() {
        let mut model = new_empty_model();
        model._set("A1", "山田太郎");
        let runs = vec![run(0, 2, "ヤマダ"), run(2, 4, "タロウ")];
        model.set_cell_phonetic_runs(0, 1, 1, runs.clone()).unwrap();
        assert_eq!(model.get_cell_phonetic_runs(0, 1, 1).unwrap(), runs);
        model._set("B1", "=PHONETIC(A1)");
        model.evaluate();
        assert_eq!(model._get_text("A1"), "山田太郎");
        assert_eq!(model._get_text("B1"), "ヤマダタロウ");
    }

    #[test]
    fn same_text_with_different_readings() {
        let mut model = new_empty_model();
        model._set("A1", "山田");
        model._set("A2", "山田");
        model._set("A3", "山田");
        model
            .set_cell_phonetic_runs(0, 1, 1, vec![run(0, 2, "ヤマダ")])
            .unwrap();
        model
            .set_cell_phonetic_runs(0, 2, 1, vec![run(0, 2, "サンダ")])
            .unwrap();
        // The same reading reuses the entry
        model
            .set_cell_phonetic_runs(0, 3, 1, vec![run(0, 2, "ヤマダ")])
            .unwrap();
        model._set("B1", "=PHONETIC(A1)");
        model._set("B2", "=PHONETIC(A2)");
        model._set("B3", "=PHONETIC(A3)");
        // Typed later, without furigana
        model._set("A4", "山田");
        model._set("B4", "=PHONETIC(A4)");
        model.evaluate();
        assert_eq!(model._get_text("B1"), "ヤマダ");
        assert_eq!(model._get_text("B2"), "サンダ");
        assert_eq!(model._get_text("B3"), "ヤマダ");
        assert_eq!(model._get_text("B4"), "山田");
        assert_eq!(model.workbook.shared_strings, ["山田", "山田", "山田"]);
    }

    #[test]
    fn remove_furigana() {
        let mut model = new_empty_model();
        model._set("A1", "山田");
        model
            .set_cell_phonetic_runs(0, 1, 1, vec![run(0, 2, "ヤマダ")])
            .unwrap();
        model.set_cell_phonetic_runs(0, 1, 1, vec![]).unwrap();
        assert_eq!(model.get_cell_phonetic_runs(0, 1, 1).unwrap(), vec![]);
        // It goes back to the entry without furigana
        assert_eq!(model.workbook.shared_strings, ["山田", "山田"]);
    }

    #[test]
    fn formulas_see_new_furigana() {
        // The editor sets the input first and the furigana right after
        let model = new_empty_model();
        let mut model = crate::UserModel::from_model(model);
        model.set_user_input(0, 1, 2, "=PHONETIC(A1)").unwrap();
        model.set_user_input(0, 1, 1, "山田").unwrap();
        assert_eq!(model.get_formatted_cell_value(0, 1, 2).unwrap(), "山田");
        model
            .set_cell_phonetic_runs(0, 1, 1, vec![run(0, 2, "ヤマダ")])
            .unwrap();
        assert_eq!(model.get_formatted_cell_value(0, 1, 2).unwrap(), "ヤマダ");
    }

    #[test]
    fn invalid_runs() {
        let mut model = new_empty_model();
        model._set("A1", "山田");
        model._set("A2", "123");
        assert!(model
            .set_cell_phonetic_runs(0, 1, 1, vec![run(0, 3, "ヤマダ")])
            .is_err());
        assert!(model
            .set_cell_phonetic_runs(0, 1, 1, vec![run(1, 2, "ダ"), run(0, 1, "ヤマ")])
            .is_err());
        assert!(model
            .set_cell_phonetic_runs(0, 2, 1, vec![run(0, 1, "イチ")])
            .is_err());
        assert!(model
            .set_cell_phonetic_runs(0, 3, 1, vec![run(0, 1, "イチ")])
            .is_err());
    }
}
