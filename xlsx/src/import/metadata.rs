use std::io::{BufReader, Read};

use ironcalc_base::types::Metadata;

use crate::error::XlsxError;

use super::util::get_value_or_default;
use super::xml::XmlNode;

struct AppData {
    application: String,
    app_version: String,
}

struct CoreData {
    creator: String,
    last_modified_by: String,
    created: String,
    last_modified: String,
}

fn load_core<R: Read + std::io::Seek>(
    archive: &mut zip::read::ZipArchive<R>,
) -> Result<CoreData, XlsxError> {
    let file = archive.by_name("docProps/core.xml")?;
    let doc = XmlNode::parse(BufReader::new(file))?;
    let core_data = &doc;
    // Note the namespace should be "http://purl.org/dc/elements/1.1/"
    let creator = get_value_or_default(core_data, "creator", "Anonymous User");
    // Note namespace is "http://schemas.openxmlformats.org/package/2006/metadata/core-properties"
    let last_modified_by = get_value_or_default(core_data, "lastModifiedBy", "Anonymous User");
    // In these two cases the namespace is "http://purl.org/dc/terms/"
    let created = get_value_or_default(core_data, "created", "");
    let last_modified = get_value_or_default(core_data, "modified", "");

    Ok(CoreData {
        creator,
        last_modified_by,
        created,
        last_modified,
    })
}

fn load_app<R: Read + std::io::Seek>(
    archive: &mut zip::read::ZipArchive<R>,
) -> Result<AppData, XlsxError> {
    let file = archive.by_name("docProps/app.xml")?;
    let doc = XmlNode::parse(BufReader::new(file))?;
    let app_data = &doc;

    let application = get_value_or_default(app_data, "Application", "Unknown application");
    let app_version = get_value_or_default(app_data, "AppVersion", "");
    Ok(AppData {
        application,
        app_version,
    })
}

pub(super) fn load_metadata<R: Read + std::io::Seek>(
    archive: &mut zip::read::ZipArchive<R>,
) -> Result<Metadata, XlsxError> {
    let app_data = load_app(archive)?;
    let core_data = load_core(archive)?;
    Ok(Metadata {
        application: app_data.application,
        app_version: app_data.app_version,
        creator: core_data.creator,
        last_modified_by: core_data.last_modified_by,
        created: core_data.created,
        last_modified: core_data.last_modified,
    })
}
