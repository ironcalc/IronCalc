#![allow(clippy::unwrap_used)]

mod _rels;
mod doc_props;
mod escape;
mod shared_strings;
mod styles;
mod workbook;
mod workbook_xml_rels;
mod worksheets;
mod xml_constants;

use std::io::BufWriter;
use std::{
    fs,
    io::{Seek, Write},
};

use ironcalc_base::expressions::utils::number_to_column;
use ironcalc_base::types::Workbook;
use ironcalc_base::{get_milliseconds_since_epoch, Model};

use self::xml_constants::XML_DECLARATION;

use crate::error::XlsxError;

#[cfg(test)]
mod test;

fn get_content_types_xml(workbook: &Workbook) -> String {
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
    // we skip the theme and calcChain
    // r#"<Override PartName="/xl/theme/theme1.xml" ContentType="application/vnd.openxmlformats-officedocument.theme+xml"/>"#,
    // r#"<Override PartName="/xl/calcChain.xml" ContentType="application/vnd.openxmlformats-officedocument.spreadsheetml.calcChain+xml"/>"#,
    content.extend([
        r#"<Override PartName="/xl/styles.xml" ContentType="application/vnd.openxmlformats-officedocument.spreadsheetml.styles+xml"/>"#.to_string(),
        r#"<Override PartName="/xl/sharedStrings.xml" ContentType="application/vnd.openxmlformats-officedocument.spreadsheetml.sharedStrings+xml"/>"#.to_string(),
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
        return Err(XlsxError::IO(format!("file {} already exists", file_name)));
    }
    let file = fs::File::create(file_path).unwrap();
    let writer = BufWriter::new(file);
    save_xlsx_to_writer(model, writer)?;

    Ok(())
}

pub fn save_xlsx_to_writer<W: Write + Seek>(model: &Model, writer: W) -> Result<W, XlsxError> {
    let workbook = &model.workbook;
    let selected_sheet = match workbook.views.get(&0) {
        Some(view) => view.sheet,
        _ => 0,
    };
    let mut zip = zip::ZipWriter::new(writer);

    let options = zip::write::FileOptions::default();

    // root folder
    zip.start_file("[Content_Types].xml", options)?;
    zip.write_all(get_content_types_xml(workbook).as_bytes())?;

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

    zip.add_directory("xl/_rels", options)?;
    zip.start_file("xl/_rels/workbook.xml.rels", options)?;
    zip.write_all(workbook_xml_rels::get_workbook_xml_rels(workbook).as_bytes())?;

    zip.add_directory("xl/worksheets", options)?;
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
        zip.write_all(
            worksheets::get_worksheet_xml(
                worksheet,
                &model.parsed_formulas[sheet_index],
                sheet_dimension_str,
                is_sheet_selected,
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
        return Err(XlsxError::IO(format!("file {} already exists", file_name)));
    }
    let s = bitcode::encode(&model.workbook);
    let mut file = fs::File::create(file_path)?;
    file.write_all(&s)?;

    Ok(())
}
