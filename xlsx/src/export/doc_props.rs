use chrono::DateTime;
use ironcalc_base::{
    new_empty::{APPLICATION, APP_VERSION, IRONCALC_USER},
    types::Workbook,
};

use crate::error::XlsxError;

// Application-Defined File Properties part
pub(crate) fn get_app_xml(_: &Workbook) -> String {
    // contains application name and version

    // The next few are not needed:
    // security. It is password protected (not implemented)
    // Scale
    // Titles of parts

    format!(
        "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>
<Properties xmlns=\"http://schemas.openxmlformats.org/officeDocument/2006/extended-properties\" \
            xmlns:vt=\"http://schemas.openxmlformats.org/officeDocument/2006/docPropsVTypes\">\
  <Application>{}</Application>\
  <AppVersion>{}</AppVersion>\
</Properties>",
        APPLICATION, APP_VERSION
    )
}

// Core File Properties part
pub(crate) fn get_core_xml(workbook: &Workbook, milliseconds: i64) -> Result<String, XlsxError> {
    // contains the name of the creator, last modified and date
    let metadata = &workbook.metadata;
    let creator = metadata.creator.to_string();
    let last_modified_by = IRONCALC_USER.to_string();
    let created = metadata.created.to_string();
    // FIXME add now

    let seconds = milliseconds / 1000;
    let dt = match DateTime::from_timestamp(seconds, 0) {
        Some(s) => s,
        None => {
            return Err(XlsxError::Xml(format!(
                "Invalid timestamp: {}",
                milliseconds
            )))
        }
    };
    let last_modified = dt.format("%Y-%m-%dT%H:%M:%SZ").to_string();
    Ok(format!(
        "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>
<cp:coreProperties \
 xmlns:cp=\"http://schemas.openxmlformats.org/package/2006/metadata/core-properties\" \
 xmlns:dc=\"http://purl.org/dc/elements/1.1/\" xmlns:dcterms=\"http://purl.org/dc/terms/\" \
 xmlns:dcmitype=\"http://purl.org/dc/dcmitype/\" \
 xmlns:xsi=\"http://www.w3.org/2001/XMLSchema-instance\"> \
<dc:title></dc:title><dc:subject></dc:subject>\
<dc:creator>{}</dc:creator>\
<cp:keywords></cp:keywords>\
<dc:description></dc:description>\
<cp:lastModifiedBy>{}</cp:lastModifiedBy>\
<cp:revision></cp:revision>\
<dcterms:created xsi:type=\"dcterms:W3CDTF\">{}</dcterms:created>\
<dcterms:modified xsi:type=\"dcterms:W3CDTF\">{}</dcterms:modified>\
<cp:category></cp:category>\
<cp:contentStatus></cp:contentStatus>\
</cp:coreProperties>",
        creator, last_modified_by, created, last_modified
    ))
}
