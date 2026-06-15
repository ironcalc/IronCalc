#![allow(clippy::unwrap_used)]
#![allow(clippy::panic)]

use ironcalc::import::load_from_xlsx;
use ironcalc_base::types::VerticalAlignment;

#[test]
// `xfId` is optional on a cellXfs <xf>: it references cellStyleXfs and many real-world
// Excel/LibreOffice/Numbers exports omit it. Before this was made optional the importer
// hard-errored with `Missing "xfId" XML attribute` and the whole file failed to load.
//
// optional_xf_id.xlsx is such a file: every <xf> in <cellXfs> lacks an `xfId` attribute.
// Loading it must succeed, and the per-cell styles referenced by those <xf>s must still
// be parsed (A2 uses the xf with `vertical="top" wrapText="1"`).
fn test_cell_xfs_without_xf_id_loads() {
    let model = load_from_xlsx("tests/optional_xf_id.xlsx", "en", "UTC", "en").unwrap();

    let style_a2 = model.get_style_for_cell(0, 2, 1).unwrap();
    let alignment = style_a2.alignment.unwrap();
    assert_eq!(alignment.vertical, VerticalAlignment::Top);
    assert!(alignment.wrap_text);
}
