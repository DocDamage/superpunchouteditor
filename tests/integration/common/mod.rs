//! Common test utilities for integration tests

// Each integration test is compiled as a separate binary, so a given binary
// intentionally uses only a subset of these shared helper modules.
#[allow(dead_code)]
pub mod palette_utils;
#[allow(dead_code)]
pub mod portrait_utils;
#[allow(dead_code)]
pub mod sprite_utils;
