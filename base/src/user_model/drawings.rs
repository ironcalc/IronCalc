use crate::charts::{Chart, ChartData, DataSource, Fill, Line};
use crate::drawing_types::Anchor;

use super::{common::UserModel, history::Diff};

impl UserModel<'_> {
    /// The ids of the charts of `sheet`, in drawing order.
    ///
    /// See also:
    /// * [Model::get_chart_ids](crate::Model::get_chart_ids)
    pub fn get_chart_ids(&self, sheet: u32) -> Result<Vec<u32>, String> {
        self.model.get_chart_ids(sheet)
    }

    /// The chart `id` of `sheet`.
    ///
    /// See also:
    /// * [Model::get_chart](crate::Model::get_chart)
    pub fn get_chart(&self, sheet: u32, id: u32) -> Result<&Chart, String> {
        self.model.get_chart(sheet, id)
    }

    /// The values a chart plots, resolved from the workbook.
    ///
    /// See also:
    /// * [Model::get_chart_data](crate::Model::get_chart_data)
    pub fn get_chart_data(&self, source: &DataSource) -> Result<ChartData, String> {
        self.model.get_chart_data(source)
    }

    /// The frame the drawing `id` of `sheet` spans.
    ///
    /// See also:
    /// * [Model::get_drawing_anchor](crate::Model::get_drawing_anchor)
    pub fn get_drawing_anchor(&self, sheet: u32, id: u32) -> Result<Anchor, String> {
        self.model.get_drawing_anchor(sheet, id)
    }

    /// The background and the border of the drawing `id` of `sheet`.
    ///
    /// See also:
    /// * [Model::get_drawing_style](crate::Model::get_drawing_style)
    pub fn get_drawing_style(&self, sheet: u32, id: u32) -> Result<(Fill, Line), String> {
        self.model.get_drawing_style(sheet, id)
    }

    /// Adds `chart` to `sheet` in the frame `anchor` spans and returns its id.
    ///
    /// See also:
    /// * [Model::new_chart](crate::Model::new_chart)
    pub fn new_chart(&mut self, sheet: u32, anchor: Anchor, chart: Chart) -> Result<u32, String> {
        let id = self.model.new_chart(sheet, anchor, chart)?;
        let drawing = self.model.get_drawing(sheet, id)?.clone();
        self.push_diff_list(vec![Diff::NewChart {
            sheet,
            drawing: Box::new(drawing),
        }]);
        Ok(id)
    }

    /// Removes the chart `id` from `sheet`.
    ///
    /// See also:
    /// * [Model::delete_chart](crate::Model::delete_chart)
    pub fn delete_chart(&mut self, sheet: u32, id: u32) -> Result<(), String> {
        self.model.get_chart(sheet, id)?;
        let (index, old) = self.model.remove_drawing(sheet, id)?;
        self.push_diff_list(vec![Diff::DeleteChart {
            sheet,
            index,
            old: Box::new(old),
        }]);
        Ok(())
    }

    /// Replaces the chart `id` of `sheet` by `chart`.
    ///
    /// See also:
    /// * [Model::update_chart](crate::Model::update_chart)
    pub fn update_chart(&mut self, sheet: u32, id: u32, chart: Chart) -> Result<(), String> {
        let old = self.model.get_chart(sheet, id)?.clone();
        self.model.update_chart(sheet, id, chart.clone())?;
        self.push_diff_list(vec![Diff::UpdateChart {
            sheet,
            id,
            old: Box::new(old),
            new: Box::new(chart),
        }]);
        Ok(())
    }

    /// Moves or resizes the drawing `id` of `sheet` to the frame `anchor` spans.
    ///
    /// See also:
    /// * [Model::move_drawing](crate::Model::move_drawing)
    pub fn move_drawing(&mut self, sheet: u32, id: u32, anchor: Anchor) -> Result<(), String> {
        let old = self.model.get_drawing(sheet, id)?.anchor.clone();
        self.model.move_drawing(sheet, id, anchor.clone())?;
        self.push_diff_list(vec![Diff::MoveDrawing {
            sheet,
            id,
            old,
            new: anchor,
        }]);
        Ok(())
    }

    /// Sets the background and the border of the drawing `id` of `sheet`.
    ///
    /// See also:
    /// * [Model::set_drawing_style](crate::Model::set_drawing_style)
    pub fn set_drawing_style(
        &mut self,
        sheet: u32,
        id: u32,
        fill: Fill,
        border: Line,
    ) -> Result<(), String> {
        let drawing = self.model.get_drawing(sheet, id)?;
        let (old_fill, old_border) = (drawing.fill.clone(), drawing.border.clone());
        self.model
            .set_drawing_style(sheet, id, fill.clone(), border.clone())?;
        self.push_diff_list(vec![Diff::SetDrawingStyle {
            sheet,
            id,
            old_fill,
            old_border,
            new_fill: fill,
            new_border: border,
        }]);
        Ok(())
    }
}
