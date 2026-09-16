//! Merged cells under stable addressing: a merge is one commit, a merged range is a register.

use crate::collab::fractional_index::{FractionalIndex, FractionalKey};
use crate::collab::log::Timestamp;
use crate::collab::model::{CollabModel, Stable, StableCellAddress, StableRange};
use crate::collab::patch::{Patch, Property};
use crate::expressions::types::Area;
use crate::merged_cells::{merge_across_ranges, merge_down_ranges};
use crate::types::{Alignment, Cell, HorizontalAlignment, MergedCell, Position};
use std::cmp::Ordering;

/// The keys of ordinals `first..=last` on `index`, and the tail keys that have to be inserted
/// first for the whole span to be addressable.
fn axis_keys(
    index: &FractionalIndex,
    first: i32,
    last: i32,
) -> (Vec<FractionalKey>, Vec<FractionalKey>) {
    let planned = index.plan_virtual(last as usize);
    let len = index.len() as i32;
    let keys = (first..=last)
        .map(|o| match index.key(o as usize - 1) {
            Some(key) => key.clone(),
            None => planned[(o - len - 1) as usize].clone(),
        })
        .collect();
    (keys, planned)
}

/// Whether two ordinal rectangles `(row1, column1, row2, column2)` overlap.
fn intersects(a: (i32, i32, i32, i32), b: (i32, i32, i32, i32)) -> bool {
    a.0 <= b.2 && b.0 <= a.2 && a.1 <= b.3 && b.1 <= a.3
}

/// A stored merge, the rectangle it currently covers and the stamp of the write that made it.
struct MergeEntry {
    timestamp: Timestamp,
    range: StableRange,
    area: (i32, i32, i32, i32),
}
impl MergeEntry {
    fn new(range: StableRange, area: (i32, i32, i32, i32), timestamp: Timestamp) -> Self {
        MergeEntry {
            range,
            area,
            timestamp,
        }
    }
}
impl Eq for MergeEntry {}
impl PartialEq for MergeEntry {
    fn eq(&self, other: &Self) -> bool {
        self.cmp(&other) == Ordering::Equal
    }
}
impl PartialOrd for MergeEntry {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
impl Ord for MergeEntry {
    fn cmp(&self, other: &Self) -> Ordering {
        match self.timestamp.cmp(&other.timestamp) {
            Ordering::Equal => match self.range.rows.cmp(&other.range.rows) {
                Ordering::Equal => self.range.cols.cmp(&other.range.cols),
                ord => ord,
            },
            ord => ord,
        }
    }
}

impl CollabModel<'_> {
    /// Merges `range` into a single merged cell anchored at its top-left corner, following the
    /// rules of [`Model::merge_cells`](crate::Model::merge_cells).
    pub fn merge_cells(&mut self, range: &Area) -> Result<(), String> {
        self.merge_with(range, false)
    }

    /// [`Self::merge_cells`], centering the merged cell horizontally ("merge & center").
    pub fn merge_cells_center(&mut self, range: &Area) -> Result<(), String> {
        self.merge_with(range, true)
    }

    /// Merges each row of `range` separately ("merge across"); every row is validated up front.
    pub fn merge_cells_across(&mut self, range: &Area) -> Result<(), String> {
        let ranges = merge_across_ranges(range)?;
        for range in &ranges {
            self.check_merge(range)?;
        }
        for range in &ranges {
            self.merge_cells(range)?;
        }
        Ok(())
    }

    /// Merges each column of `range` separately ("merge down"); every column is validated up front.
    pub fn merge_cells_down(&mut self, range: &Area) -> Result<(), String> {
        let ranges = merge_down_ranges(range)?;
        for range in &ranges {
            self.check_merge(range)?;
        }
        for range in &ranges {
            self.merge_cells(range)?;
        }
        Ok(())
    }

    /// Unmerges every stored range whose rectangle intersects `range`, content and styles kept.
    pub fn unmerge_cells(&mut self, range: &Area) -> Result<(), String> {
        let Area {
            sheet,
            row,
            column,
            width,
            height,
        } = *range;
        if row < 1 || column < 1 || width < 1 || height < 1 {
            return Err("Invalid range".to_string());
        }
        let id = self.sheet_of(sheet)?;
        let worksheet = self.workbook.worksheet(sheet)?;
        let rect = (row, column, row + height - 1, column + width - 1);
        let patches: Vec<Patch> = worksheet
            .merged_cells
            .iter()
            .filter(|range| {
                Stable::resolve_merged(range, &worksheet.index)
                    .is_some_and(|stored| intersects(stored, rect))
            })
            .map(|range| Patch::SetMergedRange {
                sheet: id,
                range: range.clone(),
                merged: false,
                prev: true,
            })
            .collect();
        if patches.is_empty() {
            return Ok(());
        }
        self.commit_local(patches);
        Ok(())
    }

    /// The merged cells of the worksheet as ordinal rectangles.
    /// A range whose corners the sheet no longer resolves is skipped.
    pub fn get_merged_cells(&self, sheet: u32) -> Result<Vec<MergedCell>, String> {
        Ok(self
            .workbook
            .worksheet(sheet)?
            .merged_ranges()
            .map(|(row, column, last_row, last_column)| MergedCell {
                row,
                column,
                width: last_column - column + 1,
                height: last_row - row + 1,
            })
            .collect())
    }

    /// Merges `range` as one commit:
    /// 1. content and link of its single content cell move to the anchor
    /// 2. covered cells are cleared
    /// 3. merged style is stamped on every cell of the range and the range itself is registered
    ///    as merged.
    fn merge_with(&mut self, range: &Area, center: bool) -> Result<(), String> {
        let Area {
            sheet,
            row,
            column,
            width,
            height,
        } = *range;
        self.check_merge_range(range)?;
        let content_cell = self.merge_range_content_cell(range)?;
        let (source_row, source_column) = content_cell.unwrap_or((row, column));
        let mut merged_style = self.get_style_for_cell(sheet, source_row, source_column)?;
        if center {
            let alignment = merged_style
                .alignment
                .get_or_insert_with(Alignment::default);
            alignment.horizontal = HorizontalAlignment::Center;
        }
        let id = self.sheet_of(sheet)?;
        let i = sheet as usize;
        let index = &self.workbook.worksheets[i].index;
        let (row_keys, planned_rows) = axis_keys(&index.rows, row, row + height - 1);
        let (col_keys, planned_cols) = axis_keys(&index.cols, column, column + width - 1);

        let mut patches = Vec::new();
        if !planned_rows.is_empty() {
            patches.push(Patch::InsertRows {
                sheet: id,
                keys: planned_rows,
            });
        }
        if !planned_cols.is_empty() {
            patches.push(Patch::InsertColumns {
                sheet: id,
                keys: planned_cols,
            });
        }
        let at_of = |r: i32, c: i32| -> StableCellAddress {
            (
                row_keys[(r - row) as usize].clone(),
                col_keys[(c - column) as usize].clone(),
            )
        };
        let anchor = at_of(row, column);
        if (source_row, source_column) != (row, column) {
            let source = at_of(source_row, source_column);
            match self.workbook.worksheets[i].cell(source_row, source_column) {
                Some(Cell::CellFormula { .. }) | Some(Cell::ArrayFormula { .. }) => {
                    // Formulas are stored relative to their cell: re-enter the text at the anchor
                    // so the formula stays as written.
                    let text = self.get_localized_cell_content(sheet, source_row, source_column)?;
                    let input =
                        self.input_patches(sheet, row, column, &text, merged_style.clone())?;
                    patches.extend(input);
                }
                Some(_) => patches.push(Patch::SetCellValue {
                    sheet: id,
                    at: anchor.clone(),
                    value: self.cell_input(i, &source),
                    ts: None,
                    prev: Box::new(self.cell_input(i, &anchor)),
                }),
                None => {}
            }
            if let Some(link) = self.workbook.worksheets[i].links.get(&source).cloned() {
                let prev = self.workbook.worksheets[i].links.get(&anchor).cloned();
                patches.push(Patch::SetCellLink {
                    sheet: id,
                    at: anchor.clone(),
                    link: Some(link),
                    prev,
                });
            }
        }

        for r in row..row + height {
            for c in column..column + width {
                let at = at_of(r, c);
                if (r, c) == (row, column) {
                    continue;
                }
                if self.workbook.worksheets[i].cell(r, c).is_some() {
                    patches.push(Patch::SetCellValue {
                        sheet: id,
                        at: at.clone(),
                        value: None,
                        ts: None,
                        prev: Box::new(self.cell_input(i, &at)),
                    });
                }
                if let Some(link) = self.workbook.worksheets[i].links.get(&at).cloned() {
                    patches.push(Patch::SetCellLink {
                        sheet: id,
                        at,
                        link: None,
                        prev: Some(link),
                    });
                }
            }
        }

        for r in row..row + height {
            for c in column..column + width {
                let at = at_of(r, c);
                let mut style = merged_style.clone();
                if r != row {
                    style.border.top = None;
                }
                if r != row + height - 1 {
                    style.border.bottom = None;
                }
                if c != column {
                    style.border.left = None;
                }
                if c != column + width - 1 {
                    style.border.right = None;
                }
                // Every attribute is stamped: a cleared cell keeps no style of its own, and the
                // merged style has to outrank whatever the anchor's own write left behind.
                let stored = self.cell_style_at(i, &at).unwrap_or_default();
                patches.push(Patch::SetCellStyle {
                    sheet: id,
                    at,
                    props: Property::all(&style),
                    ts: None,
                    prev: Property::all(&stored),
                });
            }
        }

        let last = at_of(row + height - 1, column + width - 1);
        patches.push(Patch::SetMergedRange {
            sheet: id,
            range: StableRange {
                rows: Some((anchor.0.clone(), last.0)),
                cols: Some((anchor.1.clone(), last.1)),
            },
            merged: true,
            prev: false,
        });
        self.commit_local(patches);
        Ok(())
    }

    /// Unmerges duplicate and overlapping merges after a delivered batch.
    pub(crate) fn reconcile_merges(&mut self) {
        let mut cancelled = Vec::new();
        for worksheet in &self.workbook.worksheets {
            if worksheet.merged_cells.len() < 2 {
                continue;
            }
            let index = &worksheet.index;
            let live: Vec<MergeEntry> = worksheet
                .merged_cells
                .iter()
                .filter_map(|range| {
                    let rect = Stable::resolve_merged(range, index)?;
                    let ts = index
                        .registers
                        .merges
                        .get(range)
                        .copied()
                        .unwrap_or_default();
                    Some(MergeEntry::new(range.clone(), rect, ts))
                })
                .collect();
            // Same rectangle written twice: the oldest write stands, the others are cancelled.
            let mut kept: Vec<&MergeEntry> = Vec::new();
            let mut out = Vec::new();
            for entry in &live {
                match kept.iter().position(|other| other.area == entry.area) {
                    Some(at) => {
                        if entry < kept[at] {
                            out.push(kept[at].range.clone());
                            kept[at] = entry;
                        } else {
                            out.push(entry.range.clone());
                        }
                    }
                    None => kept.push(entry),
                }
            }
            // Merges that overlap without being the same rectangle cancel each other.
            for (at, entry) in kept.iter().enumerate() {
                if kept
                    .iter()
                    .enumerate()
                    .any(|(other, o)| other != at && intersects(entry.area, o.area))
                {
                    out.push(entry.range.clone());
                }
            }

            cancelled.extend(out.into_iter().map(|range| Patch::SetMergedRange {
                sheet: worksheet.sheet_id,
                range,
                merged: false,
                prev: true,
            }));
        }
        if !cancelled.is_empty() {
            self.commit_local(cancelled);
        }
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]
    use super::*;
    use crate::collab::log::Commit;
    use crate::types::Worksheet;
    use crate::UserModel;

    fn area(row: i32, column: i32, width: i32, height: i32) -> Area {
        Area {
            sheet: 0,
            row,
            column,
            width,
            height,
        }
    }

    /// A delivered batch: what the framework hands to a replica, reconciliation included.
    fn deliver(model: &mut CollabModel<'_>, commits: &[Commit]) {
        model.apply_batch(commits).unwrap();
    }

    fn rects(sheet: &Worksheet<Stable>) -> Vec<(i32, i32, i32, i32)> {
        let mut out: Vec<_> = sheet.merged_ranges().collect();
        out.sort();
        out
    }

    fn converged(a: &CollabModel<'_>, b: &CollabModel<'_>) {
        let (sa, sb) = (&a.workbook.worksheets[0], &b.workbook.worksheets[0]);
        assert_eq!(rects(sb), rects(sa));
        assert_eq!(sb.index.registers.merges, sa.index.registers.merges);
        for row in 1..=6 {
            for col in 1..=6 {
                assert_eq!(
                    b.get_formatted_cell_value(0, row, col),
                    a.get_formatted_cell_value(0, row, col),
                    "value at row {row} column {col}"
                );
                assert_eq!(
                    b.get_style_for_cell(0, row, col),
                    a.get_style_for_cell(0, row, col),
                    "style at row {row} column {col}"
                );
            }
        }
    }

    #[test]
    fn same_merge_both_orders() {
        let mut a = CollabModel::new(1);
        a.new_sheet();
        a.set_user_input(0, 3, 3, "x".to_string()).unwrap();
        let mut b = CollabModel::new(2);
        deliver(&mut b, &a.flush());

        a.merge_cells(&area(2, 2, 2, 2)).unwrap();
        deliver(&mut b, &a.flush());
        a.evaluate();
        b.evaluate();
        for m in [&a, &b] {
            assert_eq!(m.get_formatted_cell_value(0, 2, 2), Ok("x".to_string()));
            assert_eq!(m.get_merged_cells(0).unwrap().len(), 1);
        }
        converged(&a, &b);

        b.unmerge_cells(&area(2, 2, 2, 2)).unwrap();
        deliver(&mut a, &b.flush());
        a.evaluate();
        b.evaluate();
        for m in [&a, &b] {
            assert_eq!(m.get_formatted_cell_value(0, 2, 2), Ok("x".to_string()));
            assert!(m.get_merged_cells(0).unwrap().is_empty());
        }
        converged(&a, &b);
    }

    #[test]
    fn undo_of_merge_converges() {
        let mut a = UserModel::new_empty_with_session("model", "en", "UTC", "en", 1).unwrap();
        let mut b = UserModel::new_empty_with_session("model", "en", "UTC", "en", 2).unwrap();
        a.set_user_input(0, 3, 3, "x").unwrap();
        b.apply_external_diffs(&a.flush_send_queue()).unwrap();

        a.set_selected_cell(3, 3).unwrap();
        a.merge_cells(&area(2, 2, 2, 2)).unwrap();
        // A covered cell cannot stay selected: the selection snaps to the anchor.
        assert_eq!(a.get_selected_cell(), (0, 2, 2));
        b.apply_external_diffs(&a.flush_send_queue()).unwrap();
        assert_eq!(b.get_formatted_cell_value(0, 2, 2).unwrap(), "x");
        assert_eq!(b.get_merged_cells(0).unwrap().len(), 1);

        a.undo().unwrap();
        b.apply_external_diffs(&a.flush_send_queue()).unwrap();
        for m in [&a, &b] {
            assert!(m.get_merged_cells(0).unwrap().is_empty());
            assert_eq!(m.get_formatted_cell_value(0, 3, 3).unwrap(), "x");
            assert_eq!(m.get_formatted_cell_value(0, 2, 2).unwrap(), "");
        }
    }

    #[test]
    fn merge_past_tail_is_one_commit() {
        let mut a = CollabModel::new(1);
        a.new_sheet();
        let mut b = CollabModel::new(2);
        deliver(&mut b, &a.flush());

        a.merge_cells(&area(20, 1, 2, 2)).unwrap();
        let commits = a.flush();
        assert_eq!(commits.len(), 1);
        let patches = &commits[0].patches;
        assert!(matches!(patches[0], Patch::InsertRows { .. }));
        assert!(matches!(patches[1], Patch::InsertColumns { .. }));
        assert!(matches!(
            patches[patches.len() - 1],
            Patch::SetMergedRange { merged: true, .. }
        ));
        deliver(&mut b, &commits);
        for m in [&a, &b] {
            assert_eq!(
                m.get_merged_cells(0).unwrap(),
                vec![MergedCell {
                    row: 20,
                    column: 1,
                    width: 2,
                    height: 2
                }]
            );
        }
        converged(&a, &b);

        let mut a = UserModel::new_empty_with_session("model", "en", "UTC", "en", 1).unwrap();
        let mut b = UserModel::new_empty_with_session("model", "en", "UTC", "en", 2).unwrap();
        b.apply_external_diffs(&a.flush_send_queue()).unwrap();
        let rows_before = a.model.workbook.worksheets[0].index.rows.len();

        a.merge_cells(&area(20, 1, 2, 2)).unwrap();
        b.apply_external_diffs(&a.flush_send_queue()).unwrap();
        a.undo().unwrap();
        b.apply_external_diffs(&a.flush_send_queue()).unwrap();
        for m in [&a, &b] {
            assert!(m.get_merged_cells(0).unwrap().is_empty());
            assert_eq!(m.model.workbook.worksheets[0].index.rows.len(), rows_before);
        }
    }

    #[test]
    fn overlapping_merges_cancel() {
        let mut a = CollabModel::new(1);
        a.new_sheet();
        a.set_user_input(0, 2, 2, "x".to_string()).unwrap();
        let setup = a.flush();
        let mut b = CollabModel::new(2);
        let mut c = CollabModel::new(3);
        deliver(&mut b, &setup);
        deliver(&mut c, &setup);

        a.merge_cells(&area(1, 1, 2, 2)).unwrap();
        b.merge_cells(&area(2, 2, 2, 2)).unwrap();
        let from_a = a.flush();
        let from_b = b.flush();
        deliver(&mut b, &from_a);
        deliver(&mut a, &from_b);

        // The cancel each peer emitted, delivered to the other and then again: it is idempotent.
        let cancel_a = a.flush();
        let cancel_b = b.flush();
        assert!(!cancel_a.is_empty() && !cancel_b.is_empty());
        deliver(&mut b, &cancel_a);
        deliver(&mut a, &cancel_b);
        deliver(&mut b, &cancel_a);
        deliver(&mut a, &cancel_b);

        // C sees the same commits in another order, and cancels the same pair.
        deliver(&mut c, &from_b);
        deliver(&mut c, &from_a);
        deliver(&mut c, &cancel_b);
        deliver(&mut c, &cancel_a);
        let cancel_c = c.flush();
        deliver(&mut a, &cancel_c);
        deliver(&mut b, &cancel_c);
        assert!(a.flush().is_empty() && b.flush().is_empty() && c.flush().is_empty());
        for m in [&mut a, &mut b, &mut c] {
            m.evaluate();
        }
        for m in [&a, &b, &c] {
            assert!(m.get_merged_cells(0).unwrap().is_empty());
            // A's merge moved the value to its anchor before either merge was cancelled.
            assert_eq!(m.get_formatted_cell_value(0, 1, 1), Ok("x".to_string()));
        }
        converged(&a, &b);
        converged(&a, &c);
    }

    #[test]
    fn identical_merges_keep_one() {
        for (row, height) in [(1, 2), (20, 2)] {
            let mut a = CollabModel::new(1);
            a.new_sheet();
            let mut b = CollabModel::new(2);
            deliver(&mut b, &a.flush());

            let range = area(row, 1, 2, height);
            a.merge_cells(&range).unwrap();
            b.merge_cells(&range).unwrap();
            let from_a = a.flush();
            let from_b = b.flush();
            deliver(&mut b, &from_a);
            deliver(&mut a, &from_b);

            // The duplicate is dropped by the rule, not by a commit.
            assert!(a.local.pending.is_empty() && b.local.pending.is_empty());
            for m in [&a, &b] {
                assert_eq!(m.get_merged_cells(0).unwrap().len(), 1);
            }
            a.evaluate();
            b.evaluate();
            converged(&a, &b);
        }
    }

    #[test]
    fn covered_cell_guard_and_remote_write() {
        let mut a = CollabModel::new(1);
        a.new_sheet();
        a.set_user_input(0, 1, 1, "anchor".to_string()).unwrap();
        let mut b = CollabModel::new(2);
        deliver(&mut b, &a.flush());

        a.merge_cells(&area(1, 1, 2, 2)).unwrap();
        assert_eq!(
            a.set_user_input(0, 2, 2, "no".to_string()),
            Err("Cannot edit a cell that is part of a merged cell".to_string())
        );
        // B has not seen the merge yet, so its write into the covered cell stands.
        b.set_user_input(0, 2, 2, "late".to_string()).unwrap();
        let merge = a.flush();
        let write = b.flush();
        deliver(&mut b, &merge);
        deliver(&mut a, &write);
        a.evaluate();
        b.evaluate();
        for m in [&a, &b] {
            assert_eq!(m.get_merged_cells(0).unwrap().len(), 1);
            assert_eq!(
                m.get_formatted_cell_value(0, 1, 1),
                Ok("anchor".to_string())
            );
        }
        converged(&a, &b);

        a.unmerge_cells(&area(1, 1, 2, 2)).unwrap();
        deliver(&mut b, &a.flush());
        a.evaluate();
        b.evaluate();
        for m in [&a, &b] {
            assert_eq!(m.get_formatted_cell_value(0, 2, 2), Ok("late".to_string()));
        }
        converged(&a, &b);
    }

    #[test]
    fn shrunk_merge_vanishes_and_returns() {
        let mut a = UserModel::new_empty_with_session("model", "en", "UTC", "en", 1).unwrap();
        let mut b = UserModel::new_empty_with_session("model", "en", "UTC", "en", 2).unwrap();
        a.set_user_input(0, 2, 2, "x").unwrap(); // B2='x'
        a.merge_cells(&area(2, 2, 1, 2)).unwrap(); // B2:B3
        a.delete_rows(0, 3, 1).unwrap(); // remove row which was part of merged cells
        b.apply_external_diffs(&a.flush_send_queue()).unwrap();
        for m in [&mut a, &mut b] {
            assert!(m.get_merged_cells(0).unwrap().is_empty());
            assert!(m.model.workbook.worksheets[0]
                .merged_range_containing(2, 2)
                .is_none());
        }

        // A undoes delete rows
        a.undo().unwrap();
        // B changes the value
        b.set_user_input(0, 2, 2, "y").unwrap(); // B2='y'
        b.evaluate();
        assert_eq!(b.get_formatted_cell_value(0, 2, 2), Ok("y".to_string()));
        b.apply_external_diffs(&a.flush_send_queue()).unwrap();
        a.apply_external_diffs(&b.flush_send_queue()).unwrap();

        for m in [&a, &b] {
            assert_eq!(
                m.get_merged_cells(0).unwrap(),
                vec![MergedCell {
                    row: 2,
                    column: 2,
                    width: 1,
                    height: 2
                }]
            );
            assert_eq!(m.get_formatted_cell_value(0, 2, 2).unwrap(), "y");
        }
        converged(&a.model, &b.model);
    }

    #[test]
    fn revived_merge_over_a_newer_one_cancels_both() {
        let mut a = UserModel::new_empty_with_session("model", "en", "UTC", "en", 1).unwrap();
        let mut b = UserModel::new_empty_with_session("model", "en", "UTC", "en", 2).unwrap();
        a.set_user_input(0, 2, 2, "x").unwrap(); // B2='x'
        a.merge_cells(&area(2, 2, 1, 2)).unwrap(); // B2:B3
        a.delete_rows(0, 3, 1).unwrap(); // remove row with covered cell
        b.apply_external_diffs(&a.flush_send_queue()).unwrap();

        // B2:B3 is collapsed to a single cell, so B2:C2 is a valid merge.
        b.merge_cells(&area(2, 2, 2, 1)).unwrap(); // B2:C2
        a.apply_external_diffs(&b.flush_send_queue()).unwrap();

        // A undoes delete row
        a.undo().unwrap();
        assert!(a.get_merged_cells(0).unwrap().is_empty());
        assert!(a.model.local.pending.iter().any(|commit| commit
            .patches
            .iter()
            .any(|patch| matches!(patch, Patch::SetMergedRange { merged: false, .. }))));

        let cancel = a.flush_send_queue();
        b.apply_external_diffs(&cancel).unwrap();
        for m in [&a, &b] {
            assert!(m.get_merged_cells(0).unwrap().is_empty());
            assert_eq!(m.get_formatted_cell_value(0, 2, 2).unwrap(), "x");
        }
        converged(&a.model, &b.model);
    }

    #[test]
    fn move_splitting_merge_refused() {
        let mut a = CollabModel::new(1);
        a.new_sheet();
        a.set_user_input(0, 2, 1, "x".to_string()).unwrap();
        a.merge_cells(&area(2, 1, 1, 2)).unwrap();

        assert_eq!(
            a.move_rows_action(0, 3, 1, 5),
            Err("Cannot move rows because that would split a merged cell".to_string())
        );
        a.move_rows_action(0, 2, 2, 5).unwrap();
        a.evaluate();
        assert_eq!(
            a.get_merged_cells(0).unwrap(),
            vec![MergedCell {
                row: 7,
                column: 1,
                width: 1,
                height: 2
            }]
        );
        assert_eq!(a.get_formatted_cell_value(0, 7, 1), Ok("x".to_string()));
    }
}
