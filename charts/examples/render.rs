//! Renders every column chart of an xlsx file as SVG.
//!
//!     cargo run -p ironcharts --example render -- Book.xlsx out_dir
//!
//! Each chart becomes `out_dir/<sheet name>_<chart name>.svg`, at the size
//! its anchor gives it in the sheet.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::path::Path;

use ironcalc::import::load_from_xlsx;
use ironcalc_base::charts::Chart;
use ironcalc_base::drawing_types::{Anchor, DrawingContent};
use ironcalc_base::locale::get_locale;
use ironcalc_base::Model;
use ironcharts::layout::Context;
use ironcharts::text::Inter;
use ironcharts::{layout, to_svg};

/// English Metric Units per pixel at 96 dpi.
const EMU_PER_PIXEL: f64 = 9525.0;

/// The size in pixels of the frame an anchor spans.
fn anchor_size(model: &Model, sheet: u32, anchor: &Anchor) -> (f64, f64) {
    let (from, to) = (&anchor.from, &anchor.to);
    let mut width = 0.0;
    for column in from.column..to.column {
        width += model.get_column_width(sheet, column).unwrap();
    }
    width += (to.column_offset - from.column_offset) as f64 / EMU_PER_PIXEL;
    let mut height = 0.0;
    for row in from.row..to.row {
        height += model.get_row_height(sheet, row).unwrap();
    }
    height += (to.row_offset - from.row_offset) as f64 / EMU_PER_PIXEL;
    (width.round(), height.round())
}

fn main() {
    let mut args = std::env::args().skip(1);
    let (Some(file), Some(out_dir)) = (args.next(), args.next()) else {
        eprintln!("usage: render <file.xlsx> <out_dir>");
        std::process::exit(1);
    };
    std::fs::create_dir_all(&out_dir).unwrap();
    let model = load_from_xlsx(&file, "en", "UTC", "en").expect("cannot read the file");
    let theme = model.get_theme();
    let locale = get_locale("en").unwrap();
    for (sheet, worksheet) in model.workbook.worksheets.iter().enumerate() {
        for drawing in &worksheet.drawings {
            let DrawingContent::Chart(chart) = &drawing.content else {
                continue;
            };
            let Chart::ColumnChart(column_chart) = chart.as_ref();
            let data = match model.get_chart_data(&column_chart.data) {
                Ok(data) => data,
                Err(error) => {
                    eprintln!("{}: {error}", drawing.name);
                    continue;
                }
            };
            let (width, height) = anchor_size(&model, sheet as u32, &drawing.anchor);
            let context = Context {
                width,
                height,
                theme: &theme,
                locale,
                measurer: &Inter,
            };
            let scene = layout(
                column_chart,
                &data,
                &drawing.fill,
                &drawing.border,
                &context,
            );
            let name = format!(
                "{}_{}.svg",
                worksheet.get_name().replace(' ', "_"),
                drawing.name.replace(' ', "_")
            );
            let path = Path::new(&out_dir).join(name);
            std::fs::write(&path, to_svg(&scene)).unwrap();
            println!("{} ({width}x{height})", path.display());
        }
    }
}
