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
    ScanSummaryNew,
    ScanSummaryChanged,
    ScanSummaryUnchanged,
    ScanSummaryMissing,
    ScanSummaryUnreadable,
    SearchInputLabel,
    SearchRunButton,
    SearchResultLabel,
    TimelineRunButton,
    TimelineResultLabel,
    GalleryRunButton,
    GalleryResultLabel,
    ThumbnailStatusLabel,
    ThumbnailGeneratedLabel,
    ThumbnailReusedLabel,
    ThumbnailFailedLabel,
    ThumbnailUnavailable,
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
        TextKey::ScanSummaryNew => "new",
        TextKey::ScanSummaryChanged => "changed",
        TextKey::ScanSummaryUnchanged => "unchanged",
        TextKey::ScanSummaryMissing => "missing",
        TextKey::ScanSummaryUnreadable => "unreadable",
        TextKey::SearchInputLabel => "Search indexed media and tags",
        TextKey::SearchRunButton => "Run read-model query",
        TextKey::SearchResultLabel => "Read-model rows",
        TextKey::TimelineRunButton => "Run timeline projection",
        TextKey::TimelineResultLabel => "Timeline buckets",
        TextKey::GalleryRunButton => "Run gallery projection",
        TextKey::GalleryResultLabel => "Gallery rows",
        TextKey::ThumbnailStatusLabel => "Thumbnail cache status",
        TextKey::ThumbnailGeneratedLabel => "generated",
        TextKey::ThumbnailReusedLabel => "reused",
        TextKey::ThumbnailFailedLabel => "failed",
        TextKey::ThumbnailUnavailable => "unavailable",
        TextKey::RootLifecycleActive => "active",
        TextKey::RootLifecycleUnavailable => "unavailable",
        TextKey::RootLifecycleDeactivated => "deactivated",
        TextKey::NonDestructiveNotice => {
            "Source files are always read-only. Librapix metadata stays in app storage."
        }
    }
}
