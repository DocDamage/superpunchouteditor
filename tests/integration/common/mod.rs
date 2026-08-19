//! Common test utilities for integration tests

// Each integration test is compiled as a separate binary, so a given binary
// intentionally uses only a subset of the shared palette helpers.
#[allow(dead_code)]
pub mod palette_utils;
pub mod portrait_utils;
pub mod sprite_utils;
