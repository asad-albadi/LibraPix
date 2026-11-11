//! Reusable presentation structures; callers own product state and services.

pub mod details_pane;
pub mod dialog;
// Shared patterns are staged for the remaining dialog migrations in Task 6.
#[allow(dead_code)]
pub mod field;
pub mod sidebar;
#[allow(dead_code)]
pub mod status;

#[cfg(test)]
mod tests;
