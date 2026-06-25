#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]
#![allow(clippy::panic)]

// Generated per-file tests
// one #[test] per .xlsx file, produced by build.rs.
include!(concat!(env!("OUT_DIR"), "/generated_tests.rs"));
