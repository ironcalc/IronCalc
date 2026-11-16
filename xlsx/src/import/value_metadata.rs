//! Errors stored as rich values.
//!
//! Newer Excel errors (`#SPILL!`, `#CALC!`) and errors that carry extra detail
//! are written for compatibility as `#VALUE!` in the cell, with a `vm` (value
//! metadata) attribute pointing to the actual error:
//!
//! ```text
//! <c r="A1" t="e" vm="1"><v>#VALUE!</v></c>
//!   vm="1"                -> 1-based index into <valueMetadata> in xl/metadata.xml
//!   <bk><rc t="1" v="0"/> -> t: 1-based index into <metadataTypes> (XLRICHVALUE)
//!                            v: 0-based index into <futureMetadata name="XLRICHVALUE">
//!   <xlrd:rvb i="0"/>     -> 0-based index into <rv> in xl/richData/rdrichvalue.xml
//!   <rv s="0"><v>13</v>.. -> values of structure s in rdrichvaluestructure.xml
//!   <s t="_error"><k n="errorType"/>..
//! ```
//!
//! The `vm` index says nothing by itself about which error it is.

use std::collections::HashMap;
use std::io::{BufReader, Read};

use super::worksheets::Relationship;
use super::xml::XmlNode;

/// The errors referenced by each `vm` index of the workbook.
#[derive(Default)]
pub(crate) struct ValueMetadata {
    /// Entry `n` is for `vm="n+1"`. None if it is not an error we know.
    errors: Vec<Option<&'static str>>,
}

impl ValueMetadata {
    /// The error referenced by the `vm` attribute of a cell, if any.
    pub(crate) fn error_name(&self, vm: &str) -> Option<&'static str> {
        let index = vm.parse::<usize>().ok()?.checked_sub(1)?;
        self.errors.get(index).copied().flatten()
    }
}

/// The error for an `errorType` of an `_error` rich value.
/// Only the codes seen in files written by Excel are listed; for any other the
/// value cached in the cell is kept.
fn error_name_of(error_type: i32) -> Option<&'static str> {
    match error_type {
        2 => Some("#VALUE!"),
        8 => Some("#SPILL!"),
        13 => Some("#CALC!"),
        _ => None,
    }
}

fn part_path(rels: &HashMap<String, Relationship>, rel_type_suffix: &str) -> Option<String> {
    let target = &rels
        .values()
        .find(|r| r.rel_type.ends_with(rel_type_suffix))?
        .target;
    Some(if let Some(absolute) = target.strip_prefix('/') {
        absolute.to_string()
    } else {
        format!("xl/{target}")
    })
}

fn load_part<R: Read + std::io::Seek>(
    archive: &mut zip::read::ZipArchive<R>,
    rels: &HashMap<String, Relationship>,
    rel_type_suffix: &str,
) -> Option<XmlNode> {
    let path = part_path(rels, rel_type_suffix)?;
    let file = archive.by_name(&path).ok()?;
    XmlNode::parse(BufReader::new(file)).ok()
}

/// For each `vm` index, the 0-based index of its rich value, if it has one.
fn rich_value_indices(metadata: &XmlNode) -> Vec<Option<usize>> {
    let types: Vec<&str> = metadata
        .descendants()
        .filter(|n| n.has_tag_name("metadataType"))
        .map(|n| n.attribute("name").unwrap_or(""))
        .collect();
    // The rich value of each block of <futureMetadata name="XLRICHVALUE">
    let future_rich_values: Vec<Option<usize>> = metadata
        .children()
        .filter(|n| n.has_tag_name("futureMetadata") && n.attribute("name") == Some("XLRICHVALUE"))
        .flat_map(|n| n.children().filter(|bk| bk.has_tag_name("bk")))
        .map(|bk| {
            bk.descendants()
                .find(|n| n.has_tag_name("rvb"))
                .and_then(|rvb| rvb.attribute("i"))
                .and_then(|i| i.parse().ok())
        })
        .collect();
    let Some(value_metadata) = metadata
        .children()
        .find(|n| n.has_tag_name("valueMetadata"))
    else {
        return Vec::new();
    };
    value_metadata
        .children()
        .filter(|n| n.has_tag_name("bk"))
        .map(|bk| {
            let rc = bk.children().find(|n| n.has_tag_name("rc"))?;
            let t: usize = rc.attribute("t")?.parse().ok()?;
            let v: usize = rc.attribute("v")?.parse().ok()?;
            if *types.get(t.checked_sub(1)?)? != "XLRICHVALUE" {
                return None;
            }
            *future_rich_values.get(v)?
        })
        .collect()
}

/// The error of each rich value, if it is an error we know.
fn rich_value_errors(rich_values: &XmlNode, structures: &XmlNode) -> Vec<Option<&'static str>> {
    // For each structure, the position of `errorType` if it is an `_error`
    let error_type_positions: Vec<Option<usize>> = structures
        .children()
        .filter(|n| n.has_tag_name("s"))
        .map(|s| {
            if s.attribute("t") != Some("_error") {
                return None;
            }
            s.children()
                .filter(|k| k.has_tag_name("k"))
                .position(|k| k.attribute("n") == Some("errorType"))
        })
        .collect();
    rich_values
        .children()
        .filter(|n| n.has_tag_name("rv"))
        .map(|rv| {
            let s: usize = rv.attribute("s")?.parse().ok()?;
            let position = (*error_type_positions.get(s)?)?;
            let value = rv
                .children()
                .filter(|v| v.has_tag_name("v"))
                .nth(position)?;
            error_name_of(value.text()?.trim().parse().ok()?)
        })
        .collect()
}

fn value_metadata_from(
    metadata: &XmlNode,
    rich_values: &XmlNode,
    structures: &XmlNode,
) -> ValueMetadata {
    let errors_by_rich_value = rich_value_errors(rich_values, structures);
    let errors = rich_value_indices(metadata)
        .into_iter()
        .map(|index| *errors_by_rich_value.get(index?)?)
        .collect();
    ValueMetadata { errors }
}

/// Reads the errors referenced by `vm` attributes. Nothing is fatal here: a
/// missing or broken part only means the cached cell values are used.
pub(super) fn load_value_metadata<R: Read + std::io::Seek>(
    archive: &mut zip::read::ZipArchive<R>,
    rels: &HashMap<String, Relationship>,
) -> ValueMetadata {
    let Some(metadata) = load_part(archive, rels, "/sheetMetadata") else {
        return ValueMetadata::default();
    };
    let Some(rich_values) = load_part(archive, rels, "/rdRichValue") else {
        return ValueMetadata::default();
    };
    let Some(structures) = load_part(archive, rels, "/rdRichValueStructure") else {
        return ValueMetadata::default();
    };
    value_metadata_from(&metadata, &rich_values, &structures)
}

#[cfg(test)]
mod tests {
    use super::*;

    const METADATA: &str = r#"<metadata xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main" xmlns:xlrd="http://schemas.microsoft.com/office/spreadsheetml/2017/richdata" xmlns:xda="http://schemas.microsoft.com/office/spreadsheetml/2017/dynamicarray">
<metadataTypes count="2"><metadataType name="XLDAPR"/><metadataType name="XLRICHVALUE"/></metadataTypes>
<futureMetadata name="XLDAPR" count="1"><bk><extLst><ext><xda:dynamicArrayProperties fDynamic="1"/></ext></extLst></bk></futureMetadata>
<futureMetadata name="XLRICHVALUE" count="3">
<bk><extLst><ext><xlrd:rvb i="2"/></ext></extLst></bk>
<bk><extLst><ext><xlrd:rvb i="0"/></ext></extLst></bk>
<bk><extLst><ext><xlrd:rvb i="1"/></ext></extLst></bk>
</futureMetadata>
<cellMetadata count="1"><bk><rc t="1" v="0"/></bk></cellMetadata>
<valueMetadata count="4"><bk><rc t="2" v="0"/></bk><bk><rc t="2" v="1"/></bk><bk><rc t="2" v="2"/></bk><bk><rc t="1" v="0"/></bk></valueMetadata>
</metadata>"#;

    // The second structure lists `errorType` in second place
    const STRUCTURES: &str = r#"<rvStructures count="2">
<s t="_error"><k n="errorType" t="i"/><k n="subType" t="i"/></s>
<s t="_error"><k n="colOffset" t="i"/><k n="errorType" t="i"/><k n="rwOffset" t="i"/><k n="subType" t="i"/></s>
</rvStructures>"#;

    const RICH_VALUES: &str = r#"<rvData count="3">
<rv s="0"><v>2</v><v>3</v></rv>
<rv s="1"><v>2</v><v>8</v><v>0</v><v>1</v></rv>
<rv s="0"><v>13</v><v>0</v></rv>
</rvData>"#;

    fn parse(metadata: &str) -> ValueMetadata {
        value_metadata_from(
            &XmlNode::parse_str(metadata).unwrap(),
            &XmlNode::parse_str(RICH_VALUES).unwrap(),
            &XmlNode::parse_str(STRUCTURES).unwrap(),
        )
    }

    #[test]
    fn vm_is_resolved_through_the_rich_values() {
        let value_metadata = parse(METADATA);
        assert_eq!(value_metadata.error_name("1"), Some("#CALC!"));
        assert_eq!(value_metadata.error_name("2"), Some("#VALUE!"));
        assert_eq!(value_metadata.error_name("3"), Some("#SPILL!"));
    }

    #[test]
    fn unknown_or_invalid_vm_gives_nothing() {
        let value_metadata = parse(METADATA);
        // Not a rich value
        assert_eq!(value_metadata.error_name("4"), None);
        assert_eq!(value_metadata.error_name("5"), None);
        assert_eq!(value_metadata.error_name("0"), None);
        assert_eq!(value_metadata.error_name("x"), None);
        assert_eq!(ValueMetadata::default().error_name("1"), None);
    }
}
