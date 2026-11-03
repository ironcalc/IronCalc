//! Charts on a worksheet.
//!
//! A chart lives inside a [`Drawing`], the frame that gives it a place on the
//! sheet and a background. The public API talks about charts and creates or
//! removes the drawing underneath; moving the frame and styling it are the
//! only operations that name the drawing.

use crate::charts::{Chart, Fill, Line, LineStyle};
use crate::drawing_types::{Anchor, Drawing, DrawingContent};
use crate::types::Color;
use crate::Model;

impl Model<'_> {
    /// The ids of the charts of `sheet`, in drawing order.
    pub fn get_chart_ids(&self, sheet: u32) -> Result<Vec<u32>, String> {
        Ok(self
            .workbook
            .worksheet(sheet)?
            .drawings
            .iter()
            .filter(|d| matches!(d.content, DrawingContent::Chart(_)))
            .map(|d| d.id)
            .collect())
    }

    /// The chart with drawing id `id` in `sheet`.
    pub fn get_chart(&self, sheet: u32, id: u32) -> Result<&Chart, String> {
        match &self.get_drawing(sheet, id)?.content {
            DrawingContent::Chart(chart) => Ok(chart),
            DrawingContent::Unsupported => Err(format!("Drawing {id} is not a chart")),
        }
    }

    /// Adds `chart` to `sheet` in the frame `anchor` spans, with an automatic
    /// background and border. Returns the id of the new chart.
    pub fn new_chart(&mut self, sheet: u32, anchor: Anchor, chart: Chart) -> Result<u32, String> {
        let worksheet = self.workbook.worksheet(sheet)?;
        let id = worksheet.drawings.iter().map(|d| d.id).max().unwrap_or(0) + 1;
        let count = self.get_chart_ids(sheet)?.len();
        let drawing = Drawing {
            id,
            name: format!("Chart {}", count + 1),
            anchor,
            fill: Fill::Automatic,
            border: Line {
                style: LineStyle::Automatic,
                color: Color::None,
            },
            content: DrawingContent::Chart(Box::new(chart)),
        };
        self.workbook.worksheet_mut(sheet)?.drawings.push(drawing);
        Ok(id)
    }

    /// Removes the chart `id` from `sheet`.
    pub fn delete_chart(&mut self, sheet: u32, id: u32) -> Result<(), String> {
        self.get_chart(sheet, id)?;
        self.remove_drawing(sheet, id)?;
        Ok(())
    }

    /// Replaces the chart `id` of `sheet` by `chart`. The frame is untouched.
    pub fn update_chart(&mut self, sheet: u32, id: u32, chart: Chart) -> Result<(), String> {
        self.get_chart(sheet, id)?;
        self.get_drawing_mut(sheet, id)?.content = DrawingContent::Chart(Box::new(chart));
        Ok(())
    }

    /// Moves or resizes the drawing `id` of `sheet` to the frame `anchor` spans.
    pub fn move_drawing(&mut self, sheet: u32, id: u32, anchor: Anchor) -> Result<(), String> {
        self.get_drawing_mut(sheet, id)?.anchor = anchor;
        Ok(())
    }

    /// Sets the background and the border of the drawing `id` of `sheet`.
    pub fn set_drawing_style(
        &mut self,
        sheet: u32,
        id: u32,
        fill: Fill,
        border: Line,
    ) -> Result<(), String> {
        let drawing = self.get_drawing_mut(sheet, id)?;
        drawing.fill = fill;
        drawing.border = border;
        Ok(())
    }

    /// The frame the drawing `id` of `sheet` spans.
    pub fn get_drawing_anchor(&self, sheet: u32, id: u32) -> Result<Anchor, String> {
        Ok(self.get_drawing(sheet, id)?.anchor.clone())
    }

    /// The background and the border of the drawing `id` of `sheet`.
    pub fn get_drawing_style(&self, sheet: u32, id: u32) -> Result<(Fill, Line), String> {
        let drawing = self.get_drawing(sheet, id)?;
        Ok((drawing.fill.clone(), drawing.border.clone()))
    }

    pub(crate) fn get_drawing(&self, sheet: u32, id: u32) -> Result<&Drawing, String> {
        self.workbook
            .worksheet(sheet)?
            .drawings
            .iter()
            .find(|d| d.id == id)
            .ok_or_else(|| format!("There is no drawing {id} in sheet {sheet}"))
    }

    fn get_drawing_mut(&mut self, sheet: u32, id: u32) -> Result<&mut Drawing, String> {
        self.workbook
            .worksheet_mut(sheet)?
            .drawings
            .iter_mut()
            .find(|d| d.id == id)
            .ok_or_else(|| format!("There is no drawing {id} in sheet {sheet}"))
    }

    /// Removes the drawing `id` and returns it with the position it had, so
    /// it can be put back.
    pub(crate) fn remove_drawing(
        &mut self,
        sheet: u32,
        id: u32,
    ) -> Result<(usize, Drawing), String> {
        let drawings = &mut self.workbook.worksheet_mut(sheet)?.drawings;
        let index = drawings
            .iter()
            .position(|d| d.id == id)
            .ok_or_else(|| format!("There is no drawing {id} in sheet {sheet}"))?;
        Ok((index, drawings.remove(index)))
    }

    /// Puts a drawing (back) at `index`, refusing an id already in use.
    pub(crate) fn insert_drawing_at(
        &mut self,
        sheet: u32,
        index: usize,
        drawing: Drawing,
    ) -> Result<(), String> {
        let drawings = &mut self.workbook.worksheet_mut(sheet)?.drawings;
        if drawings.iter().any(|d| d.id == drawing.id) {
            return Err(format!(
                "There is already a drawing {} in sheet {sheet}",
                drawing.id
            ));
        }
        drawings.insert(index.min(drawings.len()), drawing);
        Ok(())
    }
}
