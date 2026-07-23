//! Server-side materialization: rebuild workbook bytes from a collaboration
//! document, with no browser involved.
//!
//! The recipe mirrors a joiner: attach a blank workbook to a fresh document
//! (deterministic bootstrap), apply the room state as one remote update, and
//! serialize the reconciled model. Used by hosting servers to refresh their
//! stored workbook from a room document (e.g. on room close) and to
//! reconstruct historical versions from a persisted update log.

use crate::user_model::UserModel;

use super::session::CollabSession;

/// The bootstrap of the blank scaffold workbook must lose every LWW tie
/// against real clients; Yjs map conflicts resolve toward the higher client
/// id, so the materializer claims the lowest one.
const MATERIALIZE_CLIENT_ID: u64 = 0;

/// Rebuilds serialized workbook bytes (`UserModel::from_bytes` format) from
/// the full state of a collaboration document, encoded as a single yrs v1
/// update (`encode_state_as_update_v1` against an empty state vector).
pub fn materialize_doc_update(update: &[u8]) -> Result<Vec<u8>, String> {
    // Empty name/default settings: bootstrap registers with no content never
    // compete with the document's values (same rule as a blank joiner).
    let mut um = UserModel::new_empty("", "en", "UTC", "en")?;
    let mut session = CollabSession::attach(&mut um, MATERIALIZE_CLIENT_ID)?;
    session.apply_remote(&mut um, update)?;
    Ok(um.to_bytes())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn materializes_a_document_back_into_workbook_bytes() {
        // A "client" builds a workbook and shares it into a document.
        let mut um = UserModel::new_empty("Budget", "en", "UTC", "en").unwrap();
        um.set_user_input(0, 1, 1, "=6*7").unwrap();
        um.set_user_input(0, 2, 1, "hello").unwrap();
        let mut session = CollabSession::attach(&mut um, 7).unwrap();
        let full_state = session.flush_local(&mut um).unwrap();

        // The server rebuilds the workbook from the document alone.
        let bytes = materialize_doc_update(&full_state).unwrap();
        let rebuilt = UserModel::from_bytes(&bytes, "en").unwrap();
        assert_eq!(rebuilt.get_name(), "Budget");
        assert_eq!(rebuilt.get_formatted_cell_value(0, 1, 1).unwrap(), "42");
        assert_eq!(rebuilt.get_formatted_cell_value(0, 2, 1).unwrap(), "hello");
    }
}
