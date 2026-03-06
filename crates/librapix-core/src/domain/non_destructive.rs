/// Hard guarantees that all application features must uphold.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NonDestructiveRule {
    NeverModifySourceMedia,
    StoreAppMetadataInManagedStorage,
    KeepExtractionReadOnly,
}

pub fn required_rules() -> &'static [NonDestructiveRule] {
    &[
        NonDestructiveRule::NeverModifySourceMedia,
        NonDestructiveRule::StoreAppMetadataInManagedStorage,
        NonDestructiveRule::KeepExtractionReadOnly,
    ]
}
