#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Locale {
    EnUs,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextKey {
    AppTitle,
    AppSubtitle,
    GalleryTab,
    TimelineTab,
    ActiveViewLabel,
    RegisteredRootsLabel,
    RootInputLabel,
    RootSelectedLabel,
    RootLifecycleLabel,
    RootAddButton,
    RootUpdateButton,
    RootDeactivateButton,
    RootReactivateButton,
    RootRemoveButton,
    RootRefreshButton,
    RootSelectButton,
    IndexRunButton,
    ScanSummaryLabel,
    RootLifecycleActive,
    RootLifecycleUnavailable,
    RootLifecycleDeactivated,
    NonDestructiveNotice,
}

#[derive(Debug, Clone, Copy)]
pub struct Translator {
    locale: Locale,
}

impl Translator {
    pub fn new(locale: Locale) -> Self {
        Self { locale }
    }

    pub fn locale(&self) -> Locale {
        self.locale
    }

    pub fn with_locale(self, locale: Locale) -> Self {
        Self { locale }
    }

    pub fn text(self, key: TextKey) -> &'static str {
        match self.locale {
            Locale::EnUs => en_us(key),
        }
    }
}

fn en_us(key: TextKey) -> &'static str {
    match key {
        TextKey::AppTitle => "Librapix",
        TextKey::AppSubtitle => "Non-destructive local media manager",
        TextKey::GalleryTab => "Gallery",
        TextKey::TimelineTab => "Timeline",
        TextKey::ActiveViewLabel => "Active view",
        TextKey::RegisteredRootsLabel => "Registered library roots",
        TextKey::RootInputLabel => "Library root path",
        TextKey::RootSelectedLabel => "Selected root",
        TextKey::RootLifecycleLabel => "Lifecycle",
        TextKey::RootAddButton => "Add root",
        TextKey::RootUpdateButton => "Update selected",
        TextKey::RootDeactivateButton => "Deactivate selected",
        TextKey::RootReactivateButton => "Reactivate selected",
        TextKey::RootRemoveButton => "Remove selected",
        TextKey::RootRefreshButton => "Refresh roots",
        TextKey::RootSelectButton => "Select",
        TextKey::IndexRunButton => "Run indexing baseline",
        TextKey::ScanSummaryLabel => "Last indexing summary",
        TextKey::RootLifecycleActive => "active",
        TextKey::RootLifecycleUnavailable => "unavailable",
        TextKey::RootLifecycleDeactivated => "deactivated",
        TextKey::NonDestructiveNotice => {
            "Source files are always read-only. Librapix metadata stays in app storage."
        }
    }
}
