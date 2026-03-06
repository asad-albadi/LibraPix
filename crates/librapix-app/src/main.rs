mod ui;

use iced::widget::{Space, button, column, container, image, row, scrollable, text, text_input};
use iced::{ContentFit, Element, Length, Task, Theme};
use librapix_config::{LocalePreference, ThemePreference, lexical_normalize_path, load_or_create};
use librapix_core::app::{
    AppMessage, AppState, IndexingSummary, LibraryRootView, RootLifecycle, Route,
};
use librapix_core::domain::non_destructive;
use librapix_i18n::{Locale, TextKey, Translator};
use librapix_indexer::{IgnoreEngine, ScanRoot, scan_roots};
use librapix_projections::ProjectionMedia;
use librapix_projections::gallery::{GalleryQuery, GallerySort, project_gallery};
use librapix_projections::timeline::{TimelineGranularity, project_timeline};
use librapix_search::{FuzzySearchStrategy, SearchDocument, SearchQuery, SearchStrategy};
use librapix_storage::{
    IndexedMediaWrite, IndexedMetadataStatus, SourceRootLifecycle, Storage, TagKind,
};
use librapix_thumbnails::ensure_image_thumbnail;
use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use ui::*;

fn main() -> iced::Result {
    iced::application(Librapix::default, update, view)
        .title(title)
        .theme(theme)
        .run()
}

#[derive(Debug, Clone)]
enum Message {
    OpenGallery,
    OpenTimeline,
    RootInputChanged(String),
    SelectRoot(i64),
    AddRoot,
    UpdateRoot,
    DeactivateRoot,
    ReactivateRoot,
    RemoveRoot,
    RefreshRoots,
    RunIndexing,
    SearchQueryChanged(String),
    RunSearchQuery,
    RunTimelineProjection,
    RunGalleryProjection,
    SelectMedia(i64),
    DetailsTagInputChanged(String),
    AttachAppTag,
    AttachGameTag,
    DetachTag,
    OpenSelectedFile,
    OpenSelectedFolder,
    CopySelectedPath,
    IgnoreRuleInputChanged(String),
    EnableIgnoreRule,
    DisableIgnoreRule,
}

struct Librapix {
    state: AppState,
    i18n: Translator,
    theme_preference: ThemePreference,
    runtime: RuntimeContext,
    thumbnail_status: String,
    details_tag_input: String,
    details_lines: Vec<String>,
    details_action_status: String,
    details_preview_path: Option<PathBuf>,
    details_title: String,
    ignore_rule_input: String,
    ignore_rules_preview: Vec<String>,
    gallery_items: Vec<BrowseItem>,
    timeline_items: Vec<BrowseItem>,
    search_items: Vec<BrowseItem>,
    indexing_status: String,
    browse_status: String,
    root_status: String,
}

#[derive(Debug, Clone)]
struct BrowseItem {
    media_id: i64,
    title: String,
    subtitle: String,
    thumbnail_path: Option<PathBuf>,
    is_group_header: bool,
    line: String,
}

#[derive(Debug, Clone)]
struct RuntimeContext {
    database_file: PathBuf,
    thumbnails_dir: PathBuf,
}

impl Default for Librapix {
    fn default() -> Self {
        let bootstrap = bootstrap_runtime();
        let mut app = Self {
            state: AppState {
                library_roots: bootstrap.roots,
                ..AppState::default()
            },
            i18n: Translator::new(bootstrap.locale),
            theme_preference: bootstrap.theme_preference,
            runtime: RuntimeContext {
                database_file: bootstrap.database_file,
                thumbnails_dir: bootstrap.thumbnails_dir,
            },
            thumbnail_status: String::new(),
            details_tag_input: String::new(),
            details_lines: Vec::new(),
            details_action_status: String::new(),
            details_preview_path: None,
            details_title: String::new(),
            ignore_rule_input: String::new(),
            ignore_rules_preview: Vec::new(),
            gallery_items: Vec::new(),
            timeline_items: Vec::new(),
            search_items: Vec::new(),
            indexing_status: String::new(),
            browse_status: String::new(),
            root_status: String::new(),
        };
        refresh_ignore_rules_preview(&mut app);
        app
    }
}

fn title(app: &Librapix) -> String {
    let _ = app.i18n.locale();
    app.i18n.text(TextKey::AppTitle).to_owned()
}

fn theme(app: &Librapix) -> Theme {
    match app.theme_preference {
        ThemePreference::System => Theme::TokyoNight,
        ThemePreference::Dark => Theme::Dark,
        ThemePreference::Light => Theme::Light,
    }
}

fn update(app: &mut Librapix, message: Message) -> Task<Message> {
    match message {
        Message::OpenGallery => {
            app.state.apply(AppMessage::OpenGallery);
        }
        Message::OpenTimeline => {
            app.state.apply(AppMessage::OpenTimeline);
        }
        Message::RootInputChanged(value) => {
            app.state.apply(AppMessage::SetRootInput);
            app.state.set_root_input(value);
        }
        Message::SelectRoot(id) => {
            app.state.apply(AppMessage::SetSelectedRoot);
            app.state.set_selected_root(Some(id));
        }
        Message::AddRoot => {
            if let Some(path) = normalized_input_path(&app.state.root_input)
                && with_storage(&app.runtime, |storage| storage.upsert_source_root(&path)).is_ok()
            {
                refresh_roots(app);
                app.state.clear_selection_and_input();
                app.root_status = app.i18n.text(TextKey::RootActionSuccess).to_owned();
            } else {
                app.root_status = app.i18n.text(TextKey::ErrorInvalidRootPathLabel).to_owned();
            }
        }
        Message::UpdateRoot => {
            if let (Some(id), Some(path)) = (
                app.state.selected_root_id,
                normalized_input_path(&app.state.root_input),
            ) && with_storage(&app.runtime, |storage| {
                storage.update_source_root_path(id, &path)
            })
            .is_ok()
            {
                refresh_roots(app);
                app.root_status = app.i18n.text(TextKey::RootActionSuccess).to_owned();
            } else {
                app.root_status = app.i18n.text(TextKey::ErrorInvalidRootPathLabel).to_owned();
            }
        }
        Message::DeactivateRoot => {
            if let Some(id) = app.state.selected_root_id
                && with_storage(&app.runtime, |storage| {
                    storage.set_source_root_lifecycle(id, SourceRootLifecycle::Deactivated)
                })
                .is_ok()
            {
                refresh_roots(app);
                app.root_status = app.i18n.text(TextKey::RootActionSuccess).to_owned();
            }
        }
        Message::ReactivateRoot => {
            if let Some(id) = app.state.selected_root_id
                && with_storage(&app.runtime, |storage| {
                    storage.set_source_root_lifecycle(id, SourceRootLifecycle::Active)
                })
                .is_ok()
            {
                refresh_roots(app);
                app.root_status = app.i18n.text(TextKey::RootActionSuccess).to_owned();
            }
        }
        Message::RemoveRoot => {
            if let Some(id) = app.state.selected_root_id
                && with_storage(&app.runtime, |storage| storage.remove_source_root(id)).is_ok()
            {
                refresh_roots(app);
                app.state.apply(AppMessage::ClearRootSelection);
                app.state.clear_selection_and_input();
                app.root_status = app.i18n.text(TextKey::RootActionSuccess).to_owned();
            }
        }
        Message::RefreshRoots => {
            refresh_roots(app);
        }
        Message::RunIndexing => {
            run_indexing(app);
        }
        Message::SearchQueryChanged(value) => {
            app.state.apply(AppMessage::SetSearchQuery);
            app.state.set_search_query(value);
        }
        Message::RunSearchQuery => {
            run_read_model_query(app);
        }
        Message::RunTimelineProjection => {
            run_timeline_projection(app);
        }
        Message::RunGalleryProjection => {
            run_gallery_projection(app);
        }
        Message::SelectMedia(media_id) => {
            app.state.apply(AppMessage::SetSelectedMedia);
            app.state.set_selected_media(Some(media_id));
            load_media_details(app);
        }
        Message::DetailsTagInputChanged(value) => {
            app.details_tag_input = value;
        }
        Message::AttachAppTag => {
            attach_tag_to_selected_media(app, TagKind::App);
        }
        Message::AttachGameTag => {
            attach_tag_to_selected_media(app, TagKind::Game);
        }
        Message::DetachTag => {
            detach_tag_from_selected_media(app);
        }
        Message::OpenSelectedFile => {
            open_selected_path(app, false);
        }
        Message::OpenSelectedFolder => {
            open_selected_path(app, true);
        }
        Message::CopySelectedPath => {
            copy_selected_path(app);
        }
        Message::IgnoreRuleInputChanged(value) => {
            app.ignore_rule_input = value;
        }
        Message::EnableIgnoreRule => {
            set_ignore_rule_enabled(app, true);
        }
        Message::DisableIgnoreRule => {
            set_ignore_rule_enabled(app, false);
        }
    }

    Task::none()
}

fn view(app: &Librapix) -> Element<'_, Message> {
    let _required_rules = non_destructive::required_rules();
    let is_gallery = matches!(app.state.active_route, Route::Gallery);
    let is_timeline = matches!(app.state.active_route, Route::Timeline);

    // ── Header ──
    let header = container(
        row![
            text(app.i18n.text(TextKey::AppTitle))
                .size(FONT_DISPLAY)
                .color(TEXT_PRIMARY),
            Space::new().width(Length::Fill),
            text_input(
                app.i18n.text(TextKey::SearchInputLabel),
                &app.state.search_query
            )
            .on_input(Message::SearchQueryChanged)
            .on_submit(Message::RunSearchQuery)
            .width(Length::Fixed(400.0))
            .style(search_input_style),
            Space::new().width(Length::Fill),
        ]
        .align_y(iced::Alignment::Center),
    )
    .padding([0, SPACE_XL as u16])
    .center_y(HEADER_HEIGHT)
    .width(Length::Fill)
    .style(header_style);

    // ── Sidebar: Browse navigation ──
    let nav_section = column![
        section_heading(app.i18n.text(TextKey::BrowseSectionLabel)),
        button(text(app.i18n.text(TextKey::GalleryTab)).size(FONT_BODY))
            .width(Length::Fill)
            .on_press(Message::OpenGallery)
            .style(nav_button_style(is_gallery))
            .padding([SPACE_SM as u16, SPACE_MD as u16]),
        button(text(app.i18n.text(TextKey::TimelineTab)).size(FONT_BODY))
            .width(Length::Fill)
            .on_press(Message::OpenTimeline)
            .style(nav_button_style(is_timeline))
            .padding([SPACE_SM as u16, SPACE_MD as u16]),
    ]
    .spacing(SPACE_XS);

    // ── Sidebar: Library roots ──
    let roots_list: Element<'_, Message> = if app.state.library_roots.is_empty() {
        text(app.i18n.text(TextKey::EmptyRootsLabel))
            .size(FONT_BODY)
            .color(TEXT_TERTIARY)
            .into()
    } else {
        app.state
            .library_roots
            .iter()
            .fold(column![].spacing(SPACE_2XS), |col, root| {
                let is_selected = app.state.selected_root_id == Some(root.id);
                let path_name = root
                    .normalized_path
                    .file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_else(|| root.normalized_path.display().to_string());
                let status_color = match root.lifecycle {
                    RootLifecycle::Active => SUCCESS_COLOR,
                    RootLifecycle::Unavailable => WARNING_COLOR,
                    RootLifecycle::Deactivated => TEXT_DISABLED,
                };
                col.push(
                    button(
                        row![
                            text("\u{25CF}").size(FONT_CAPTION).color(status_color),
                            text(path_name).size(FONT_BODY).color(if is_selected {
                                TEXT_PRIMARY
                            } else {
                                TEXT_SECONDARY
                            }),
                        ]
                        .spacing(SPACE_SM)
                        .align_y(iced::Alignment::Center),
                    )
                    .width(Length::Fill)
                    .on_press(Message::SelectRoot(root.id))
                    .style(nav_button_style(is_selected))
                    .padding([SPACE_XS as u16, SPACE_SM as u16]),
                )
            })
            .into()
    };

    let selected_root_actions: Element<'_, Message> = if app.state.selected_root_id.is_some() {
        column![
            row![
                button(text(app.i18n.text(TextKey::RootUpdateButton)).size(FONT_CAPTION))
                    .on_press(Message::UpdateRoot)
                    .style(subtle_button_style)
                    .padding([SPACE_2XS as u16, SPACE_SM as u16]),
                button(text(app.i18n.text(TextKey::RootDeactivateButton)).size(FONT_CAPTION))
                    .on_press(Message::DeactivateRoot)
                    .style(subtle_button_style)
                    .padding([SPACE_2XS as u16, SPACE_SM as u16]),
            ]
            .spacing(SPACE_XS),
            row![
                button(text(app.i18n.text(TextKey::RootReactivateButton)).size(FONT_CAPTION))
                    .on_press(Message::ReactivateRoot)
                    .style(subtle_button_style)
                    .padding([SPACE_2XS as u16, SPACE_SM as u16]),
                button(text(app.i18n.text(TextKey::RootRemoveButton)).size(FONT_CAPTION))
                    .on_press(Message::RemoveRoot)
                    .style(subtle_button_style)
                    .padding([SPACE_2XS as u16, SPACE_SM as u16]),
            ]
            .spacing(SPACE_XS),
        ]
        .spacing(SPACE_XS)
        .into()
    } else {
        column![].into()
    };

    let library_section = column![
        section_heading(app.i18n.text(TextKey::LibrarySectionLabel)),
        roots_list,
        text_input(
            app.i18n.text(TextKey::FolderPathPlaceholder),
            &app.state.root_input
        )
        .on_input(Message::RootInputChanged)
        .style(field_input_style),
        row![
            button(text(app.i18n.text(TextKey::RootAddButton)).size(FONT_BODY))
                .on_press(Message::AddRoot)
                .style(primary_button_style)
                .padding([SPACE_XS as u16, SPACE_MD as u16]),
            button(text(app.i18n.text(TextKey::RootRefreshButton)).size(FONT_BODY))
                .on_press(Message::RefreshRoots)
                .style(subtle_button_style)
                .padding([SPACE_XS as u16, SPACE_MD as u16]),
        ]
        .spacing(SPACE_XS),
        selected_root_actions,
        text(app.root_status.clone())
            .size(FONT_CAPTION)
            .color(TEXT_TERTIARY),
    ]
    .spacing(SPACE_SM);

    // ── Sidebar: Indexing ──
    let indexing_section = column![
        section_heading(app.i18n.text(TextKey::IndexingSectionLabel)),
        button(text(app.i18n.text(TextKey::IndexRunButton)).size(FONT_BODY))
            .on_press(Message::RunIndexing)
            .style(primary_button_style)
            .width(Length::Fill)
            .padding([SPACE_SM as u16, SPACE_MD as u16]),
        text(app.indexing_status.clone())
            .size(FONT_CAPTION)
            .color(TEXT_TERTIARY),
        text(app.thumbnail_status.clone())
            .size(FONT_CAPTION)
            .color(TEXT_TERTIARY),
    ]
    .spacing(SPACE_SM);

    // ── Sidebar: Ignore rules ──
    let ignore_section = column![
        section_heading(app.i18n.text(TextKey::IgnoreRuleInputLabel)),
        text_input("*.tmp, **/cache/**", &app.ignore_rule_input)
            .on_input(Message::IgnoreRuleInputChanged)
            .style(field_input_style),
        row![
            button(text(app.i18n.text(TextKey::IgnoreRuleAddButton)).size(FONT_CAPTION))
                .on_press(Message::EnableIgnoreRule)
                .style(subtle_button_style)
                .padding([SPACE_2XS as u16, SPACE_SM as u16]),
            button(text(app.i18n.text(TextKey::IgnoreRuleDisableButton)).size(FONT_CAPTION))
                .on_press(Message::DisableIgnoreRule)
                .style(subtle_button_style)
                .padding([SPACE_2XS as u16, SPACE_SM as u16]),
        ]
        .spacing(SPACE_XS),
        app.ignore_rules_preview
            .iter()
            .take(6)
            .fold(column![].spacing(SPACE_2XS), |col, rule| {
                col.push(text(rule.clone()).size(FONT_CAPTION).color(TEXT_TERTIARY))
            }),
    ]
    .spacing(SPACE_SM);

    let sidebar = container(
        scrollable(
            column![
                nav_section,
                h_divider(),
                library_section,
                h_divider(),
                indexing_section,
                h_divider(),
                ignore_section,
            ]
            .spacing(SPACE_LG)
            .padding(SPACE_LG as u16),
        )
        .height(Length::Fill),
    )
    .width(Length::Fixed(SIDEBAR_WIDTH))
    .style(sidebar_style);

    // ── Media pane ──
    let media_content = render_media_panel(app);

    // ── Details pane ──
    let details_content = render_details_panel(app);

    // ── Body ──
    let body = row![
        sidebar,
        container(scrollable(media_content).height(Length::Fill))
            .padding(SPACE_LG as u16)
            .width(Length::Fill),
        container(scrollable(details_content).height(Length::Fill))
            .width(Length::Fixed(DETAILS_WIDTH))
            .padding(SPACE_LG as u16)
            .style(details_pane_style),
    ]
    .height(Length::Fill);

    container(column![header, body])
        .width(Length::Fill)
        .height(Length::Fill)
        .style(app_bg_style)
        .into()
}

fn render_media_panel(app: &Librapix) -> Element<'_, Message> {
    let route_title = match app.state.active_route {
        Route::Gallery => app.i18n.text(TextKey::GalleryTab),
        Route::Timeline => app.i18n.text(TextKey::TimelineTab),
    };
    let run_msg = match app.state.active_route {
        Route::Gallery => Message::RunGalleryProjection,
        Route::Timeline => Message::RunTimelineProjection,
    };
    let browse_items = match app.state.active_route {
        Route::Gallery => &app.gallery_items,
        Route::Timeline => &app.timeline_items,
    };
    let item_count = browse_items.iter().filter(|i| !i.is_group_header).count();

    let content_header = row![
        text(route_title).size(FONT_TITLE).color(TEXT_PRIMARY),
        Space::new().width(Length::Fill),
        button(text(app.i18n.text(TextKey::RefreshButton)).size(FONT_BODY))
            .on_press(run_msg)
            .style(subtle_button_style)
            .padding([SPACE_XS as u16, SPACE_MD as u16]),
        text(format!(
            "{item_count} {}",
            app.i18n.text(TextKey::ItemsLabel)
        ))
        .size(FONT_BODY)
        .color(TEXT_SECONDARY),
    ]
    .spacing(SPACE_SM)
    .align_y(iced::Alignment::Center);

    let search_section: Element<'_, Message> = if !app.state.search_query.trim().is_empty() {
        if app.search_items.is_empty() {
            container(
                text(app.i18n.text(TextKey::EmptySearchResultsLabel))
                    .size(FONT_BODY)
                    .color(TEXT_SECONDARY),
            )
            .padding(SPACE_XL as u16)
            .width(Length::Fill)
            .style(empty_state_style)
            .into()
        } else {
            let search_header = row![
                text(app.i18n.text(TextKey::SearchResultLabel))
                    .size(FONT_SUBTITLE)
                    .color(TEXT_PRIMARY),
                text(format!("({})", app.search_items.len()))
                    .size(FONT_BODY)
                    .color(TEXT_SECONDARY),
            ]
            .spacing(SPACE_SM);

            let mut grid = column![search_header].spacing(SPACE_SM);
            let mut current_row = row![].spacing(GALLERY_GAP);
            let mut col_idx = 0;
            for item in app.search_items.iter().take(12) {
                let selected = app.state.selected_media_id == Some(item.media_id);
                let card = render_gallery_card(item, selected);
                current_row = current_row.push(card);
                col_idx += 1;
                if col_idx >= GALLERY_COLUMNS {
                    grid = grid.push(current_row);
                    current_row = row![].spacing(GALLERY_GAP);
                    col_idx = 0;
                }
            }
            if col_idx > 0 {
                for _ in col_idx..GALLERY_COLUMNS {
                    current_row =
                        current_row.push(container(text("")).width(Length::FillPortion(1)));
                }
                grid = grid.push(current_row);
            }
            grid.into()
        }
    } else {
        column![].into()
    };

    let empty_label = match app.state.active_route {
        Route::Gallery => app.i18n.text(TextKey::EmptyGalleryLabel),
        Route::Timeline => app.i18n.text(TextKey::EmptyTimelineLabel),
    };

    let browse_content: Element<'_, Message> = if browse_items.is_empty() {
        container(text(empty_label).size(FONT_SUBTITLE).color(TEXT_SECONDARY))
            .padding(SPACE_2XL as u16)
            .width(Length::Fill)
            .style(empty_state_style)
            .into()
    } else {
        match app.state.active_route {
            Route::Gallery => render_gallery_grid(browse_items, app.state.selected_media_id),
            Route::Timeline => render_timeline_view(browse_items, app.state.selected_media_id),
        }
    };

    column![content_header, search_section, browse_content]
        .spacing(SPACE_LG)
        .into()
}

fn render_gallery_grid(items: &[BrowseItem], selected_id: Option<i64>) -> Element<'_, Message> {
    let mut grid = column![].spacing(GALLERY_GAP);
    let mut current_row = row![].spacing(GALLERY_GAP);
    let mut col_idx = 0;
    for item in items.iter().filter(|i| !i.is_group_header).take(100) {
        let card = render_gallery_card(item, selected_id == Some(item.media_id));
        current_row = current_row.push(card);
        col_idx += 1;
        if col_idx >= GALLERY_COLUMNS {
            grid = grid.push(current_row);
            current_row = row![].spacing(GALLERY_GAP);
            col_idx = 0;
        }
    }
    if col_idx > 0 {
        for _ in col_idx..GALLERY_COLUMNS {
            current_row = current_row.push(container(text("")).width(Length::FillPortion(1)));
        }
        grid = grid.push(current_row);
    }
    grid.into()
}

fn render_gallery_card(item: &BrowseItem, selected: bool) -> Element<'_, Message> {
    let thumb: Element<'_, Message> = if let Some(path) = &item.thumbnail_path {
        image(image::Handle::from_path(path))
            .width(Length::Fill)
            .height(Length::Fixed(THUMB_HEIGHT))
            .content_fit(ContentFit::Cover)
            .into()
    } else {
        container(text(""))
            .width(Length::Fill)
            .height(Length::Fixed(THUMB_HEIGHT))
            .style(thumb_placeholder_style)
            .into()
    };
    let caption = container(
        column![
            text(item.title.clone()).size(FONT_BODY).color(TEXT_PRIMARY),
            text(item.subtitle.clone())
                .size(FONT_CAPTION)
                .color(TEXT_TERTIARY),
        ]
        .spacing(SPACE_2XS),
    )
    .padding([SPACE_XS as u16, SPACE_SM as u16]);

    button(column![thumb, caption])
        .width(Length::FillPortion(1))
        .on_press(Message::SelectMedia(item.media_id))
        .style(card_button_style(selected))
        .padding(0)
        .into()
}

fn render_timeline_view(items: &[BrowseItem], selected_id: Option<i64>) -> Element<'_, Message> {
    items
        .iter()
        .take(120)
        .fold(column![].spacing(SPACE_SM), |col, item| {
            if item.is_group_header {
                col.push(
                    container(
                        text(item.title.clone())
                            .size(FONT_SUBTITLE)
                            .color(TEXT_PRIMARY),
                    )
                    .padding([SPACE_MD as u16, 0]),
                )
            } else {
                let thumb: Element<'_, Message> = if let Some(path) = &item.thumbnail_path {
                    image(image::Handle::from_path(path))
                        .width(Length::Fixed(100.0))
                        .height(Length::Fixed(72.0))
                        .content_fit(ContentFit::Cover)
                        .into()
                } else {
                    container(text(""))
                        .width(Length::Fixed(100.0))
                        .height(Length::Fixed(72.0))
                        .style(thumb_placeholder_style)
                        .into()
                };
                let is_selected = selected_id == Some(item.media_id);
                col.push(
                    button(
                        row![
                            thumb,
                            column![
                                text(item.title.clone()).size(FONT_BODY).color(TEXT_PRIMARY),
                                text(item.subtitle.clone())
                                    .size(FONT_CAPTION)
                                    .color(TEXT_TERTIARY),
                            ]
                            .spacing(SPACE_2XS)
                            .width(Length::Fill),
                        ]
                        .spacing(SPACE_SM)
                        .align_y(iced::Alignment::Center),
                    )
                    .width(Length::Fill)
                    .on_press(Message::SelectMedia(item.media_id))
                    .style(card_button_style(is_selected))
                    .padding(SPACE_SM as u16),
                )
            }
        })
        .into()
}

fn render_details_panel(app: &Librapix) -> Element<'_, Message> {
    if app.state.selected_media_id.is_none() {
        return container(
            column![
                text(app.i18n.text(TextKey::SelectPhotoTitle))
                    .size(FONT_SUBTITLE)
                    .color(TEXT_SECONDARY),
                text(app.i18n.text(TextKey::SelectPhotoSubtitle))
                    .size(FONT_BODY)
                    .color(TEXT_TERTIARY),
            ]
            .spacing(SPACE_SM)
            .padding(SPACE_2XL as u16),
        )
        .width(Length::Fill)
        .style(empty_state_style)
        .into();
    }

    let preview: Element<'_, Message> = if let Some(path) = &app.details_preview_path {
        container(
            image(image::Handle::from_path(path))
                .width(Length::Fill)
                .content_fit(ContentFit::Contain),
        )
        .width(Length::Fill)
        .style(card_style)
        .into()
    } else {
        container(text(""))
            .width(Length::Fill)
            .height(Length::Fixed(160.0))
            .style(thumb_placeholder_style)
            .into()
    };

    let metadata = if app.details_lines.is_empty() {
        column![
            text(app.i18n.text(TextKey::DetailsNoSelectionLabel))
                .size(FONT_BODY)
                .color(TEXT_TERTIARY)
        ]
    } else {
        app.details_lines
            .iter()
            .fold(column![].spacing(SPACE_XS), |col, line| {
                col.push(text(line.clone()).size(FONT_BODY).color(TEXT_SECONDARY))
            })
    };

    column![
        preview,
        text(app.details_title.clone())
            .size(FONT_SUBTITLE)
            .color(TEXT_PRIMARY),
        h_divider(),
        column![
            section_heading(app.i18n.text(TextKey::FileInfoLabel)),
            metadata,
        ]
        .spacing(SPACE_SM),
        h_divider(),
        column![
            section_heading(app.i18n.text(TextKey::DetailsTagsSectionLabel)),
            text_input(
                app.i18n.text(TextKey::DetailsTagInputLabel),
                &app.details_tag_input
            )
            .on_input(Message::DetailsTagInputChanged)
            .style(field_input_style),
            row![
                button(text(app.i18n.text(TextKey::DetailsAttachTagButton)).size(FONT_BODY))
                    .on_press(Message::AttachAppTag)
                    .style(action_button_style)
                    .padding([SPACE_XS as u16, SPACE_MD as u16]),
                button(text(app.i18n.text(TextKey::DetailsAttachGameTagButton)).size(FONT_BODY))
                    .on_press(Message::AttachGameTag)
                    .style(action_button_style)
                    .padding([SPACE_XS as u16, SPACE_MD as u16]),
                button(text(app.i18n.text(TextKey::DetailsDetachTagButton)).size(FONT_BODY))
                    .on_press(Message::DetachTag)
                    .style(subtle_button_style)
                    .padding([SPACE_XS as u16, SPACE_MD as u16]),
            ]
            .spacing(SPACE_XS),
        ]
        .spacing(SPACE_SM),
        h_divider(),
        column![
            section_heading(app.i18n.text(TextKey::DetailsActionsSectionLabel)),
            row![
                button(text(app.i18n.text(TextKey::DetailsOpenFileButton)).size(FONT_BODY))
                    .on_press(Message::OpenSelectedFile)
                    .style(action_button_style)
                    .padding([SPACE_XS as u16, SPACE_MD as u16]),
                button(text(app.i18n.text(TextKey::DetailsOpenFolderButton)).size(FONT_BODY))
                    .on_press(Message::OpenSelectedFolder)
                    .style(action_button_style)
                    .padding([SPACE_XS as u16, SPACE_MD as u16]),
                button(text(app.i18n.text(TextKey::DetailsCopyPathButton)).size(FONT_BODY))
                    .on_press(Message::CopySelectedPath)
                    .style(subtle_button_style)
                    .padding([SPACE_XS as u16, SPACE_MD as u16]),
            ]
            .spacing(SPACE_XS),
        ]
        .spacing(SPACE_SM),
        text(app.details_action_status.clone())
            .size(FONT_CAPTION)
            .color(TEXT_TERTIARY),
        text(app.i18n.text(TextKey::NonDestructiveNotice))
            .size(FONT_CAPTION)
            .color(TEXT_TERTIARY),
    ]
    .spacing(SPACE_LG)
    .into()
}

struct BootstrapRuntime {
    locale: Locale,
    theme_preference: ThemePreference,
    database_file: PathBuf,
    thumbnails_dir: PathBuf,
    roots: Vec<LibraryRootView>,
}

fn bootstrap_runtime() -> BootstrapRuntime {
    let mut runtime = BootstrapRuntime {
        locale: Locale::EnUs,
        theme_preference: ThemePreference::System,
        database_file: PathBuf::from("librapix.db"),
        thumbnails_dir: PathBuf::from("thumbnails"),
        roots: Vec::new(),
    };

    let loaded = match load_or_create() {
        Ok(config) => config,
        Err(_) => return runtime,
    };

    runtime.locale = match loaded.config.locale {
        LocalePreference::EnUs => Locale::EnUs,
    };
    runtime.theme_preference = loaded.config.theme.clone();

    let database_file = loaded
        .config
        .path_overrides
        .database_file
        .clone()
        .unwrap_or(loaded.paths.database_file);
    runtime.database_file = database_file.clone();
    runtime.thumbnails_dir = loaded
        .config
        .path_overrides
        .thumbnails_dir
        .clone()
        .unwrap_or(loaded.paths.thumbnails_dir);

    let storage = match Storage::open(&database_file) {
        Ok(storage) => storage,
        Err(_) => return runtime,
    };

    for source in &loaded.config.library_source_roots {
        let _ = storage.upsert_source_root(&source.path);
    }
    let _ = storage.ensure_default_ignore_rules();
    let _ = storage.reconcile_source_root_availability();

    runtime.roots = storage
        .list_source_roots()
        .map_or_else(|_| Vec::new(), map_roots_from_storage);
    runtime
}

fn normalized_input_path(value: &str) -> Option<PathBuf> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return None;
    }
    let cwd = std::env::current_dir().ok()?;
    Some(lexical_normalize_path(&PathBuf::from(trimmed), &cwd))
}

fn refresh_roots(app: &mut Librapix) {
    let roots = with_storage(&app.runtime, |storage| {
        storage.reconcile_source_root_availability()?;
        storage.list_source_roots()
    })
    .map(map_roots_from_storage)
    .unwrap_or_default();
    app.state.apply(AppMessage::ReplaceLibraryRoots);
    app.state.replace_library_roots(roots);
    refresh_ignore_rules_preview(app);
}

fn refresh_ignore_rules_preview(app: &mut Librapix) {
    let rows = with_storage(&app.runtime, |storage| storage.list_ignore_rules("global"))
        .map(|rows| {
            rows.into_iter()
                .map(|row| {
                    let status = if row.is_enabled {
                        app.i18n.text(TextKey::IgnoreRuleEnabled)
                    } else {
                        app.i18n.text(TextKey::IgnoreRuleDisabled)
                    };
                    format!("{} ({status})", row.pattern)
                })
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    app.ignore_rules_preview = rows;
}

fn set_ignore_rule_enabled(app: &mut Librapix, is_enabled: bool) {
    let pattern = app.ignore_rule_input.trim();
    if pattern.is_empty() {
        return;
    }
    let _ = with_storage(&app.runtime, |storage| {
        storage.upsert_ignore_rule("global", pattern, is_enabled)
    });
    refresh_ignore_rules_preview(app);
}

fn run_indexing(app: &mut Librapix) {
    app.indexing_status = app.i18n.text(TextKey::LoadingIndexingLabel).to_owned();
    let prep = with_storage(&app.runtime, |storage| {
        storage.reconcile_source_root_availability()?;
        storage.ensure_default_ignore_rules()?;

        let roots = storage.list_eligible_source_roots()?;
        let roots_for_scan = roots
            .iter()
            .map(|root| ScanRoot {
                source_root_id: root.id,
                normalized_path: root.normalized_path.clone(),
            })
            .collect::<Vec<_>>();

        let patterns = storage.list_enabled_ignore_patterns("global")?;
        Ok((roots_for_scan, patterns))
    });

    let summary = prep.ok().and_then(|(roots_for_scan, patterns)| {
        let ignore = IgnoreEngine::new(&patterns).ok()?;
        let root_ids = roots_for_scan
            .iter()
            .map(|root| root.source_root_id)
            .collect::<Vec<_>>();
        let existing = with_storage(&app.runtime, |storage| {
            storage.list_existing_indexed_media_snapshots(&root_ids)
        })
        .ok()?;

        let existing_for_indexer = existing
            .into_iter()
            .map(|entry| librapix_indexer::ExistingIndexedEntry {
                source_root_id: entry.source_root_id,
                absolute_path: entry.absolute_path,
                file_size_bytes: entry.file_size_bytes,
                modified_unix_seconds: entry.modified_unix_seconds,
            })
            .collect::<Vec<_>>();

        let result = scan_roots(&roots_for_scan, &ignore, &existing_for_indexer);
        let writes = result
            .candidates
            .iter()
            .map(|candidate| IndexedMediaWrite {
                source_root_id: candidate.source_root_id,
                absolute_path: candidate.absolute_path.clone(),
                media_kind: candidate.media_kind.as_str().to_owned(),
                file_size_bytes: candidate.file_size_bytes,
                modified_unix_seconds: candidate.modified_unix_seconds,
                width_px: candidate.width_px,
                height_px: candidate.height_px,
                metadata_status: match candidate.metadata_status {
                    librapix_indexer::MetadataStatus::Ok => IndexedMetadataStatus::Ok,
                    librapix_indexer::MetadataStatus::Partial => IndexedMetadataStatus::Partial,
                    librapix_indexer::MetadataStatus::Unreadable => {
                        IndexedMetadataStatus::Unreadable
                    }
                },
            })
            .collect::<Vec<_>>();

        let apply_summary = with_storage(&app.runtime, |storage| {
            storage.apply_incremental_index(&writes, &result.scanned_root_ids)
        })
        .ok()?;

        let _ = with_storage(&app.runtime, |storage| {
            storage.ensure_media_kind_tags_attached()
        });

        let read_models = with_storage(&app.runtime, |storage| {
            storage.list_media_read_models(200, 0)
        })
        .ok()?;

        let mut generated = 0usize;
        let mut reused = 0usize;
        let mut failed = 0usize;
        for row in &read_models {
            if row.media_kind != "image" {
                continue;
            }
            match ensure_image_thumbnail(
                &app.runtime.thumbnails_dir,
                &row.absolute_path,
                row.file_size_bytes,
                row.modified_unix_seconds,
                256,
            ) {
                Ok(outcome) => {
                    if outcome.generated {
                        generated += 1;
                    } else {
                        reused += 1;
                    }
                }
                Err(_) => failed += 1,
            }
        }
        app.thumbnail_status = format!(
            "{}: {}={generated}, {}={reused}, {}={failed}",
            app.i18n.text(TextKey::ThumbnailStatusLabel),
            app.i18n.text(TextKey::ThumbnailGeneratedLabel),
            app.i18n.text(TextKey::ThumbnailReusedLabel),
            app.i18n.text(TextKey::ThumbnailFailedLabel),
        );

        Some(IndexingSummary {
            scanned_roots: result.summary.scanned_roots,
            candidate_files: result.summary.candidate_files,
            ignored_entries: result.summary.ignored_entries,
            unreadable_entries: result.summary.unreadable_entries,
            new_files: result.summary.new_files,
            changed_files: result.summary.changed_files,
            unchanged_files: result.summary.unchanged_files,
            missing_marked: apply_summary.missing_marked_count,
            read_model_count: read_models.len(),
        })
    });

    if let Some(summary) = summary {
        app.state.apply(AppMessage::RecordIndexingSummary);
        app.state.record_indexing_summary(summary);
        app.indexing_status = app.i18n.text(TextKey::IndexingCompletedLabel).to_owned();
    } else {
        app.indexing_status = app.i18n.text(TextKey::ErrorIndexingFailedLabel).to_owned();
    }
    refresh_roots(app);
}

fn run_read_model_query(app: &mut Librapix) {
    app.browse_status = app.i18n.text(TextKey::LoadingSearchLabel).to_owned();
    let query = app.state.search_query.clone();
    let rows = with_storage(&app.runtime, |storage| {
        storage.list_media_read_models(200, 0)
    })
    .map(|rows| {
        let docs = rows
            .iter()
            .map(|row| SearchDocument {
                media_id: row.media_id,
                absolute_path: row.absolute_path.display().to_string(),
                file_name: row
                    .absolute_path
                    .file_name()
                    .map(|name| name.to_string_lossy().to_string())
                    .unwrap_or_default(),
                media_kind: row.media_kind.clone(),
                tags: row.tags.clone(),
            })
            .collect::<Vec<_>>();

        let strategy = FuzzySearchStrategy::default();
        let hits = strategy.search(
            &docs,
            &SearchQuery {
                text: query.clone(),
                limit: 20,
            },
        );

        hits.into_iter()
            .filter_map(|hit| {
                rows.iter()
                    .find(|row| row.media_id == hit.media_id)
                    .map(|row| (hit, row))
            })
            .map(|(_hit, row)| BrowseItem {
                media_id: row.media_id,
                title: row
                    .absolute_path
                    .file_name()
                    .map(|s| s.to_string_lossy().to_string())
                    .unwrap_or_else(|| row.absolute_path.display().to_string()),
                subtitle: if row.tags.is_empty() {
                    row.media_kind.clone()
                } else {
                    format!("{} \u{00B7} {}", row.media_kind, row.tags.join(", "))
                },
                thumbnail_path: if row.media_kind == "image" {
                    ensure_image_thumbnail(
                        &app.runtime.thumbnails_dir,
                        &row.absolute_path,
                        row.file_size_bytes,
                        row.modified_unix_seconds,
                        256,
                    )
                    .ok()
                    .map(|outcome| outcome.thumbnail_path)
                } else {
                    None
                },
                is_group_header: false,
                line: format!("{} | {}", row.absolute_path.display(), row.media_kind),
            })
            .collect::<Vec<_>>()
    })
    .unwrap_or_default();

    app.search_items = rows;
    app.browse_status = app.i18n.text(TextKey::SearchCompletedLabel).to_owned();
}

fn run_timeline_projection(app: &mut Librapix) {
    app.browse_status = app.i18n.text(TextKey::LoadingTimelineLabel).to_owned();
    let rows = with_storage(&app.runtime, |storage| {
        storage.list_media_read_models(500, 0)
    })
    .map(|rows| {
        let media = rows_to_projection_media(&rows);
        project_timeline(&media, TimelineGranularity::Day)
    })
    .map(|buckets| {
        let mut lines = Vec::new();
        let mut items = Vec::new();
        for bucket in buckets {
            lines.push(format!("{} ({})", bucket.label, bucket.item_count));
            items.push(BrowseItem {
                media_id: 0,
                title: bucket.label.clone(),
                subtitle: String::new(),
                thumbnail_path: None,
                is_group_header: true,
                line: bucket.label.clone(),
            });
            for item in bucket.items {
                items.push(BrowseItem {
                    media_id: item.media_id,
                    title: PathBuf::from(&item.absolute_path)
                        .file_name()
                        .map(|s| s.to_string_lossy().to_string())
                        .unwrap_or(item.absolute_path.clone()),
                    subtitle: item.media_kind.clone(),
                    thumbnail_path: if item.media_kind == "image" {
                        ensure_image_thumbnail(
                            &app.runtime.thumbnails_dir,
                            &PathBuf::from(&item.absolute_path),
                            0,
                            item.modified_unix_seconds,
                            256,
                        )
                        .ok()
                        .map(|outcome| outcome.thumbnail_path)
                    } else {
                        None
                    },
                    is_group_header: false,
                    line: format!("{} [{}]", item.absolute_path, item.media_kind),
                });
            }
        }
        (lines, items)
    })
    .unwrap_or_default();
    app.state.apply(AppMessage::ReplaceTimelinePreview);
    app.state.replace_timeline_preview(rows.0);
    app.timeline_items = rows.1;
    app.browse_status = app.i18n.text(TextKey::TimelineCompletedLabel).to_owned();
}

fn run_gallery_projection(app: &mut Librapix) {
    app.browse_status = app.i18n.text(TextKey::LoadingGalleryLabel).to_owned();
    let rows = with_storage(&app.runtime, |storage| {
        storage.list_media_read_models(500, 0)
    })
    .map(|rows| {
        let media = rows_to_projection_media(&rows);
        let query = GalleryQuery {
            media_kind: None,
            tag: None,
            sort: GallerySort::ModifiedDesc,
            limit: 60,
            offset: 0,
        };
        project_gallery(&media, &query)
            .into_iter()
            .map(|item| {
                let original = PathBuf::from(&item.absolute_path);
                let thumbnail_text = rows
                    .iter()
                    .find(|row| row.media_id == item.media_id)
                    .and_then(|row| {
                        if row.media_kind != "image" {
                            return None;
                        }
                        ensure_image_thumbnail(
                            &app.runtime.thumbnails_dir,
                            &row.absolute_path,
                            row.file_size_bytes,
                            row.modified_unix_seconds,
                            256,
                        )
                        .ok()
                        .map(|outcome| outcome.thumbnail_path.display().to_string())
                    })
                    .unwrap_or_else(|| app.i18n.text(TextKey::ThumbnailUnavailable).to_owned());
                BrowseItem {
                    media_id: item.media_id,
                    title: original
                        .file_name()
                        .map(|s| s.to_string_lossy().to_string())
                        .unwrap_or_else(|| original.display().to_string()),
                    subtitle: item.media_kind.clone(),
                    thumbnail_path: rows
                        .iter()
                        .find(|row| row.media_id == item.media_id)
                        .and_then(|row| {
                            if row.media_kind != "image" {
                                return None;
                            }
                            ensure_image_thumbnail(
                                &app.runtime.thumbnails_dir,
                                &row.absolute_path,
                                row.file_size_bytes,
                                row.modified_unix_seconds,
                                256,
                            )
                            .ok()
                            .map(|outcome| outcome.thumbnail_path)
                        }),
                    is_group_header: false,
                    line: format!(
                        "{} [{}] {}={thumbnail_text}",
                        original.display(),
                        item.media_kind,
                        app.i18n.text(TextKey::ThumbnailStatusLabel),
                    ),
                }
            })
            .collect::<Vec<_>>()
    })
    .unwrap_or_default();
    app.state.apply(AppMessage::ReplaceGalleryPreview);
    app.state
        .replace_gallery_preview(rows.iter().map(|item| item.line.clone()).collect());
    app.gallery_items = rows;
    app.browse_status = app.i18n.text(TextKey::GalleryCompletedLabel).to_owned();
}

fn load_media_details(app: &mut Librapix) {
    let Some(media_id) = app.state.selected_media_id else {
        app.details_action_status = app.i18n.text(TextKey::DetailsNoSelectionLabel).to_owned();
        app.details_preview_path = None;
        app.details_title.clear();
        return;
    };
    let details = with_storage(&app.runtime, |storage| {
        storage.get_media_read_model_by_id(media_id)
    })
    .ok()
    .flatten();
    if let Some(details) = details {
        app.details_title = details
            .absolute_path
            .file_name()
            .map(|name| name.to_string_lossy().to_string())
            .unwrap_or_else(|| details.absolute_path.display().to_string());
        app.details_preview_path = if details.media_kind == "image" {
            ensure_image_thumbnail(
                &app.runtime.thumbnails_dir,
                &details.absolute_path,
                details.file_size_bytes,
                details.modified_unix_seconds,
                512,
            )
            .ok()
            .map(|outcome| outcome.thumbnail_path)
        } else {
            None
        };
        app.details_lines = vec![
            format!("id={}", details.media_id),
            format!("path={}", details.absolute_path.display()),
            format!("kind={}", details.media_kind),
            format!("size={}", details.file_size_bytes),
            format!(
                "modified={}",
                details.modified_unix_seconds.unwrap_or_default()
            ),
            format!(
                "dimensions={}x{}",
                details.width_px.unwrap_or(0),
                details.height_px.unwrap_or(0)
            ),
            format!(
                "tags={}",
                if details.tags.is_empty() {
                    app.i18n.text(TextKey::EmptyTagsLabel).to_owned()
                } else {
                    details.tags.join("|")
                }
            ),
        ];
        app.details_action_status = app.i18n.text(TextKey::DetailsActionSuccess).to_owned();
    } else {
        app.details_lines.clear();
        app.details_preview_path = None;
        app.details_title.clear();
        app.details_action_status = app.i18n.text(TextKey::DetailsActionFailed).to_owned();
    }
}

fn attach_tag_to_selected_media(app: &mut Librapix, kind: TagKind) {
    let Some(media_id) = app.state.selected_media_id else {
        app.details_action_status = app.i18n.text(TextKey::DetailsNoSelectionLabel).to_owned();
        return;
    };
    let tag = app.details_tag_input.trim();
    if tag.is_empty() {
        app.details_action_status = app.i18n.text(TextKey::DetailsActionFailed).to_owned();
        return;
    }
    match with_storage(&app.runtime, |storage| {
        storage.attach_tag_name_to_media(media_id, tag, kind)
    }) {
        Ok(_) => {
            app.details_action_status = app.i18n.text(TextKey::DetailsActionSuccess).to_owned();
            load_media_details(app);
        }
        Err(_) => {
            app.details_action_status = app.i18n.text(TextKey::DetailsActionFailed).to_owned();
        }
    }
}

fn detach_tag_from_selected_media(app: &mut Librapix) {
    let Some(media_id) = app.state.selected_media_id else {
        app.details_action_status = app.i18n.text(TextKey::DetailsNoSelectionLabel).to_owned();
        return;
    };
    let tag = app.details_tag_input.trim();
    if tag.is_empty() {
        app.details_action_status = app.i18n.text(TextKey::DetailsActionFailed).to_owned();
        return;
    }
    match with_storage(&app.runtime, |storage| {
        storage.detach_tag_name_from_media(media_id, tag)
    }) {
        Ok(_) => {
            app.details_action_status = app.i18n.text(TextKey::DetailsActionSuccess).to_owned();
            load_media_details(app);
        }
        Err(_) => {
            app.details_action_status = app.i18n.text(TextKey::DetailsActionFailed).to_owned();
        }
    }
}

fn open_selected_path(app: &mut Librapix, containing_folder: bool) {
    let Some(media_id) = app.state.selected_media_id else {
        app.details_action_status = app.i18n.text(TextKey::DetailsNoSelectionLabel).to_owned();
        return;
    };
    let row = with_storage(&app.runtime, |storage| {
        storage.get_media_read_model_by_id(media_id)
    })
    .ok()
    .flatten();
    let Some(row) = row else {
        app.details_action_status = app.i18n.text(TextKey::DetailsActionFailed).to_owned();
        return;
    };
    let target = if containing_folder {
        row.absolute_path
            .parent()
            .map(PathBuf::from)
            .unwrap_or_else(|| row.absolute_path.clone())
    } else {
        row.absolute_path
    };
    if !target.exists() {
        app.details_action_status = app.i18n.text(TextKey::ErrorUnavailableFileLabel).to_owned();
        return;
    }
    match open_with_system_default(&target) {
        Ok(_) => {
            app.details_action_status = app.i18n.text(TextKey::DetailsActionSuccess).to_owned()
        }
        Err(_) => {
            app.details_action_status = app.i18n.text(TextKey::ErrorActionFailedLabel).to_owned()
        }
    }
}

fn copy_selected_path(app: &mut Librapix) {
    let Some(media_id) = app.state.selected_media_id else {
        app.details_action_status = app.i18n.text(TextKey::DetailsNoSelectionLabel).to_owned();
        return;
    };
    let row = with_storage(&app.runtime, |storage| {
        storage.get_media_read_model_by_id(media_id)
    })
    .ok()
    .flatten();
    let Some(row) = row else {
        app.details_action_status = app.i18n.text(TextKey::DetailsActionFailed).to_owned();
        return;
    };
    if !row.absolute_path.exists() {
        app.details_action_status = app.i18n.text(TextKey::ErrorUnavailableFileLabel).to_owned();
        return;
    }
    match copy_to_clipboard(&row.absolute_path.display().to_string()) {
        Ok(_) => {
            app.details_action_status = app.i18n.text(TextKey::DetailsActionSuccess).to_owned()
        }
        Err(_) => {
            app.details_action_status = app.i18n.text(TextKey::ErrorActionFailedLabel).to_owned()
        }
    }
}

fn open_with_system_default(path: &PathBuf) -> Result<(), std::io::Error> {
    #[cfg(target_os = "macos")]
    {
        Command::new("open").arg(path).status()?;
        Ok(())
    }
    #[cfg(target_os = "windows")]
    {
        Command::new("cmd")
            .args(["/C", "start", "", &path.display().to_string()])
            .status()?;
        Ok(())
    }
    #[cfg(all(unix, not(target_os = "macos")))]
    {
        Command::new("xdg-open").arg(path).status()?;
        Ok(())
    }
}

fn copy_to_clipboard(value: &str) -> Result<(), std::io::Error> {
    #[cfg(target_os = "macos")]
    {
        let mut child = Command::new("pbcopy").stdin(Stdio::piped()).spawn()?;
        if let Some(stdin) = &mut child.stdin {
            stdin.write_all(value.as_bytes())?;
        }
        child.wait()?;
        Ok(())
    }
    #[cfg(target_os = "windows")]
    {
        let mut child = Command::new("clip").stdin(Stdio::piped()).spawn()?;
        if let Some(stdin) = &mut child.stdin {
            stdin.write_all(value.as_bytes())?;
        }
        child.wait()?;
        Ok(())
    }
    #[cfg(all(unix, not(target_os = "macos")))]
    {
        let mut child = Command::new("xclip")
            .args(["-selection", "clipboard"])
            .stdin(Stdio::piped())
            .spawn()?;
        if let Some(stdin) = &mut child.stdin {
            stdin.write_all(value.as_bytes())?;
        }
        child.wait()?;
        Ok(())
    }
}

fn rows_to_projection_media(rows: &[librapix_storage::MediaReadModel]) -> Vec<ProjectionMedia> {
    rows.iter()
        .map(|row| ProjectionMedia {
            media_id: row.media_id,
            absolute_path: row.absolute_path.display().to_string(),
            media_kind: row.media_kind.clone(),
            modified_unix_seconds: row.modified_unix_seconds,
            tags: row.tags.clone(),
        })
        .collect()
}

fn with_storage<T>(
    runtime: &RuntimeContext,
    action: impl FnOnce(&mut Storage) -> Result<T, librapix_storage::StorageError>,
) -> Result<T, librapix_storage::StorageError> {
    let mut storage = Storage::open(&runtime.database_file)?;
    action(&mut storage)
}

fn map_roots_from_storage(roots: Vec<librapix_storage::SourceRootRecord>) -> Vec<LibraryRootView> {
    roots
        .into_iter()
        .map(|root| LibraryRootView {
            id: root.id,
            normalized_path: root.normalized_path,
            lifecycle: match root.lifecycle {
                SourceRootLifecycle::Active => RootLifecycle::Active,
                SourceRootLifecycle::Unavailable => RootLifecycle::Unavailable,
                SourceRootLifecycle::Deactivated => RootLifecycle::Deactivated,
            },
        })
        .collect()
}
