#![allow(clippy::unwrap_used)]

mod _rels;
mod charts;
mod conditional_formatting;
mod doc_props;
mod drawings;
mod dxfs_styles;
mod escape;
mod shared_strings;
mod styles;
mod styles_util;
mod theme;
mod workbook;
mod workbook_xml_rels;
mod worksheets;
mod xml_constants;

use std::io::BufWriter;
use std::{
    fs,
    io::{Seek, Write},
};

use ironcalc_base::drawing_types::{Drawing, DrawingContent};
use ironcalc_base::expressions::utils::number_to_column;
use ironcalc_base::types::{SheetState, Workbook};
use ironcalc_base::{get_milliseconds_since_epoch, Model};

use self::xml_constants::XML_DECLARATION;

use crate::error::XlsxError;

#[cfg(test)]
mod test;

/// The drawing part of a worksheet: its number (`xl/drawings/drawing{number}.xml`),
/// the drawings it holds and the number of the chart part of each of them.
struct DrawingPart<'a> {
    number: usize,
    drawings: Vec<&'a Drawing>,
    chart_numbers: Vec<usize>,
}

/// Plans the drawing and chart parts: one drawing part per worksheet that has
/// at least one chart, parts numbered in worksheet order. Drawings we cannot
/// write are left out.
fn plan_drawing_parts(workbook: &Workbook) -> Vec<Option<DrawingPart<'_>>> {
    let mut drawing_number = 0;
    let mut chart_number = 0;
    workbook
        .worksheets
        .iter()
        .map(|worksheet| {
            let drawings: Vec<&Drawing> = worksheet
                .drawings
                .iter()
                .filter(|drawing| matches!(drawing.content, DrawingContent::Chart(_)))
                .collect();
            if drawings.is_empty() {
                return None;
            }
            drawing_number += 1;
            let chart_numbers = drawings
                .iter()
                .map(|_| {
                    chart_number += 1;
                    chart_number
                })
                .collect();
            Some(DrawingPart {
                number: drawing_number,
                drawings,
                chart_numbers,
            })
        })
        .collect()
}

fn get_content_types_xml(workbook: &Workbook, drawing_parts: &[Option<DrawingPart>]) -> String {
    // A list of all files in the zip
    let mut content = vec![
        r#"<Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types">"#.to_string(),
        r#"<Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/>"#.to_string(),
        r#"<Default Extension="xml" ContentType="application/xml"/>"#.to_string(),
        r#"<Override PartName="/xl/workbook.xml" ContentType="application/vnd.openxmlformats-officedocument.spreadsheetml.sheet.main+xml"/>"#.to_string(),
    ];
    for worksheet in 0..workbook.worksheets.len() {
        let sheet = format!(
            r#"<Override PartName="/xl/worksheets/sheet{}.xml" ContentType="application/vnd.openxmlformats-officedocument.spreadsheetml.worksheet+xml"/>"#,
            worksheet + 1
        );
        content.push(sheet);
    }
    for part in drawing_parts.iter().flatten() {
        content.push(format!(
            r#"<Override PartName="/xl/drawings/drawing{}.xml" ContentType="application/vnd.openxmlformats-officedocument.drawing+xml"/>"#,
            part.number
        ));
        for chart_number in &part.chart_numbers {
            content.push(format!(
                r#"<Override PartName="/xl/charts/chart{chart_number}.xml" ContentType="application/vnd.openxmlformats-officedocument.drawingml.chart+xml"/>"#
            ));
        }
    }
    content.push(
        r#"<Override PartName="/xl/theme/theme1.xml" ContentType="application/vnd.openxmlformats-officedocument.theme+xml"/>"#.to_string(),
    );
    content.extend([
        r#"<Override PartName="/xl/styles.xml" ContentType="application/vnd.openxmlformats-officedocument.spreadsheetml.styles+xml"/>"#.to_string(),
        r#"<Override PartName="/xl/sharedStrings.xml" ContentType="application/vnd.openxmlformats-officedocument.spreadsheetml.sharedStrings+xml"/>"#.to_string(),
        r#"<Override PartName="/xl/metadata.xml" ContentType="application/vnd.openxmlformats-officedocument.spreadsheetml.sheetMetadata+xml"/>"#.to_string(),
        r#"<Override PartName="/docProps/core.xml" ContentType="application/vnd.openxmlformats-package.core-properties+xml"/>"#.to_string(),
        r#"<Override PartName="/docProps/app.xml" ContentType="application/vnd.openxmlformats-officedocument.extended-properties+xml"/>"#.to_string(),
        r#"</Types>"#.to_string(),
    ]);
    format!("{XML_DECLARATION}\n{}", content.join(""))
}

/// Exports a model to an xlsx file
pub fn save_to_xlsx(model: &Model, file_name: &str) -> Result<(), XlsxError> {
    let file_path = std::path::Path::new(&file_name);
    if file_path.exists() {
        return Err(XlsxError::IO(format!("file {file_name} already exists")));
    }
    let file = fs::File::create(file_path).unwrap();
    let writer = BufWriter::new(file);
    save_xlsx_to_writer(model, writer)?;

    Ok(())
}

pub fn save_xlsx_to_writer<W: Write + Seek>(model: &Model, writer: W) -> Result<W, XlsxError> {
    let workbook = &model.workbook;
    // IronCalc does not keep track of the selected sheet: the active tab is
    // the first visible sheet.
    let selected_sheet = workbook
        .worksheets
        .iter()
        .position(|ws| ws.state == SheetState::Visible)
        .unwrap_or(0) as u32;
    let mut zip = zip::ZipWriter::new(writer);

    let options = zip::write::SimpleFileOptions::default();

    let drawing_parts = plan_drawing_parts(workbook);

    // root folder
    zip.start_file("[Content_Types].xml", options)?;
    zip.write_all(get_content_types_xml(workbook, &drawing_parts).as_bytes())?;

    zip.add_directory("docProps", options)?;
    zip.start_file("docProps/app.xml", options)?;
    zip.write_all(doc_props::get_app_xml(workbook).as_bytes())?;
    zip.start_file("docProps/core.xml", options)?;
    let milliseconds = get_milliseconds_since_epoch();
    zip.write_all(doc_props::get_core_xml(workbook, milliseconds)?.as_bytes())?;

    // Package-relationship item
    zip.add_directory("_rels", options)?;
    zip.start_file("_rels/.rels", options)?;
    zip.write_all(_rels::get_dot_rels(workbook).as_bytes())?;

    zip.add_directory("xl", options)?;
    zip.start_file("xl/sharedStrings.xml", options)?;
    zip.write_all(shared_strings::get_shared_strings_xml(workbook).as_bytes())?;
    zip.start_file("xl/styles.xml", options)?;
    zip.write_all(styles::get_styles_xml(workbook).as_bytes())?;
    zip.start_file("xl/workbook.xml", options)?;
    zip.write_all(workbook::get_workbook_xml(workbook, selected_sheet).as_bytes())?;
    zip.start_file("xl/metadata.xml", options)?;
    let metadata_xml = include_str!("metadata.xml");
    zip.write_all(metadata_xml.as_bytes())?;

    zip.add_directory("xl/theme", options)?;
    zip.start_file("xl/theme/theme1.xml", options)?;
    zip.write_all(theme::get_theme_xml(&workbook.theme).as_bytes())?;

    zip.add_directory("xl/_rels", options)?;
    zip.start_file("xl/_rels/workbook.xml.rels", options)?;
    zip.write_all(workbook_xml_rels::get_workbook_xml_rels(workbook).as_bytes())?;

    // drawings and charts
    if drawing_parts.iter().any(Option::is_some) {
        zip.add_directory("xl/drawings", options)?;
        zip.add_directory("xl/drawings/_rels", options)?;
        zip.add_directory("xl/charts", options)?;
    }
    for (sheet, part) in drawing_parts.iter().enumerate() {
        let Some(part) = part else {
            continue;
        };
        zip.start_file(format!("xl/drawings/drawing{}.xml", part.number), options)?;
        zip.write_all(drawings::get_drawing_xml(&part.drawings).as_bytes())?;
        zip.start_file(
            format!("xl/drawings/_rels/drawing{}.xml.rels", part.number),
            options,
        )?;
        zip.write_all(drawings::get_drawing_xml_rels(&part.chart_numbers).as_bytes())?;
        for (drawing, chart_number) in part.drawings.iter().zip(&part.chart_numbers) {
            if let DrawingContent::Chart(chart) = &drawing.content {
                zip.start_file(format!("xl/charts/chart{chart_number}.xml"), options)?;
                zip.write_all(
                    charts::get_chart_xml(chart, drawing, sheet as u32, model).as_bytes(),
                )?;
            }
        }
    }

    zip.add_directory("xl/worksheets", options)?;
    // sheet rels parts (one per sheet with external hyperlinks or drawings)
    let mut has_worksheet_rels = false;
    for (sheet_index, worksheet) in workbook.worksheets.iter().enumerate() {
        let drawing_number = drawing_parts[sheet_index].as_ref().map(|part| part.number);
        if let Some(rels_xml) = worksheets::get_worksheet_xml_rels(worksheet, drawing_number) {
            if !has_worksheet_rels {
                zip.add_directory("xl/worksheets/_rels", options)?;
                has_worksheet_rels = true;
            }
            zip.start_file(
                format!("xl/worksheets/_rels/sheet{}.xml.rels", sheet_index + 1),
                options,
            )?;
            zip.write_all(rels_xml.as_bytes())?;
        }
    }
    for (sheet_index, worksheet) in workbook.worksheets.iter().enumerate() {
        let id = sheet_index + 1;
        zip.start_file(format!("xl/worksheets/sheet{id}.xml"), options)?;
        let dimension = model
            .workbook
            .worksheet(sheet_index as u32)
            .unwrap()
            .dimension();
        let column_min_str = number_to_column(dimension.min_column).unwrap();
        let column_max_str = number_to_column(dimension.max_column).unwrap();
        let min_row = dimension.min_row;
        let max_row = dimension.max_row;
        let sheet_dimension_str = &format!("{column_min_str}{min_row}:{column_max_str}{max_row}");
        let is_sheet_selected = selected_sheet as usize == sheet_index;
        let drawing_rel_id =
            worksheets::get_drawing_rel_id(worksheet, drawing_parts[sheet_index].is_some());
        zip.write_all(
            worksheets::get_worksheet_xml(
                worksheet,
                &model.parsed_formulas[sheet_index],
                sheet_dimension_str,
                is_sheet_selected,
                drawing_rel_id,
            )
            .as_bytes(),
        )?;
    }

    let writer = zip.finish()?;
    Ok(writer)
}

/// Exports a model to an icalc file
pub fn save_to_icalc(model: &Model, file_name: &str) -> Result<(), XlsxError> {
    let file_path = std::path::Path::new(&file_name);
    if file_path.exists() {
        return Err(XlsxError::IO(format!("file {file_name} already exists")));
    }
    let s = bitcode::encode(&model.workbook);
    let mut file = fs::File::create(file_path)?;
    file.write_all(&s)?;

    Ok(())
}
