pub mod files;
pub mod security;

// Errors: we use `anyhow::Error` across the codebase as the canonical error type.
// The old `errors` module was removed to avoid duplication and keep a single
// error handling strategy.