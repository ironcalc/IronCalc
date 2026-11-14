use chrono::DateTime;
use ironcalc_base::{
    new_empty::{APPLICATION, APP_VERSION, IRONCALC_USER},
    types::{DateSystem, Workbook},
};

use crate::error::XlsxError;

/// Name of the custom document property that records the date system
pub const DATE_SYSTEM_PROPERTY: &str = "IronCalcDateSystem";

/// The value of the `IronCalcDateSystem` property: the date of serial 0, as
/// ODF does with `table:null-date`.
pub fn date_system_property_value(date_system: DateSystem) -> &'static str {
    match date_system {
        DateSystem::IronCalc => "1899-12-30",
        DateSystem::Excel1900 => "1900-01-01",
        DateSystem::Excel1904 => "1904-01-01",
    }
}

/// The date system named by an `IronCalcDateSystem` property value
pub fn date_system_from_property_value(value: &str) -> Option<DateSystem> {
    match value.trim() {
        "1899-12-30" => Some(DateSystem::IronCalc),
        "1900-01-01" => Some(DateSystem::Excel1900),
        "1904-01-01" => Some(DateSystem::Excel1904),
        _ => None,
    }
}

// Custom File Properties part. Only engines that know the property read it;
// Excel and LibreOffice keep custom properties when they save the file.
pub(crate) fn get_custom_xml(workbook: &Workbook) -> String {
    let value = date_system_property_value(workbook.settings.date_system);
    format!(
        "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>
<Properties xmlns=\"http://schemas.openxmlformats.org/officeDocument/2006/custom-properties\" \
            xmlns:vt=\"http://schemas.openxmlformats.org/officeDocument/2006/docPropsVTypes\">\
  <property fmtid=\"{{D5CDD505-2E9C-101B-9397-08002B2CF9AE}}\" pid=\"2\" name=\"{DATE_SYSTEM_PROPERTY}\">\
    <vt:lpwstr>{value}</vt:lpwstr>\
  </property>\
</Properties>"
    )
}

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
  <Application>{APPLICATION}</Application>\
  <AppVersion>{APP_VERSION}</AppVersion>\
</Properties>"
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
        None => return Err(XlsxError::Xml(format!("Invalid timestamp: {milliseconds}"))),
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
<dc:creator>{creator}</dc:creator>\
<cp:keywords></cp:keywords>\
<dc:description></dc:description>\
<cp:lastModifiedBy>{last_modified_by}</cp:lastModifiedBy>\
<cp:revision></cp:revision>\
<dcterms:created xsi:type=\"dcterms:W3CDTF\">{created}</dcterms:created>\
<dcterms:modified xsi:type=\"dcterms:W3CDTF\">{last_modified}</dcterms:modified>\
<cp:category></cp:category>\
<cp:contentStatus></cp:contentStatus>\
</cp:coreProperties>"
    ))
}
