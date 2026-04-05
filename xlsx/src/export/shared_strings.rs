use std::collections::{HashMap, HashSet};

use ironcalc_base::types::{Cell, Workbook};

use super::{escape::escape_xml, xml_constants::XML_DECLARATION};

/// Returns the shared strings XML and a mapping from content hash to sequential index
/// for use by the worksheet exporter.
///
/// Only strings actually referenced by worksheet cells are included;
/// strings whose only live references come from the undo/redo history are excluded.
pub(crate) fn get_shared_strings_xml(workbook: &Workbook) -> (String, HashMap<u64, usize>) {
    let cell_hashes: HashSet<u64> = workbook
        .worksheets
        .iter()
        .flat_map(|ws| {
            ws.sheet_data.values().flat_map(|row| {
                row.values().filter_map(|cell| match cell {
                    Cell::SharedString { si, .. } => Some(*si),
                    _ => None,
                })
            })
        })
        .collect();

    let mut sorted: Vec<(u64, &String)> = workbook
        .string_pool
        .iter()
        .filter(|(k, _)| cell_hashes.contains(k))
        .collect();
    sorted.sort_by_key(|(k, _)| *k);

    let mut hash_to_index: HashMap<u64, usize> = HashMap::new();
    let mut si_elements: Vec<String> = Vec::new();

    for (i, (hash, value)) in sorted.iter().enumerate() {
        hash_to_index.insert(*hash, i);
        si_elements.push(format!("<si><t>{}</t></si>", escape_xml(value)));
    }

    let count = si_elements.len();
    let xml = format!(
        "{}\n\
         <sst xmlns=\"http://schemas.openxmlformats.org/spreadsheetml/2006/main\" \
         count=\"{count}\" uniqueCount=\"{count}\">\
         {}\
         </sst>",
        XML_DECLARATION,
        si_elements.join("")
    );
    (xml, hash_to_index)
}
