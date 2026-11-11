use super::modal_overlay;
use crate::ui::components::separator::horizontal;
use crate::ui::patterns::dialog::{DialogSpec, dialog};
use crate::ui::*;
use crate::{Librapix, Message, display_name_for_root, format};
use iced::{
    Element, Length,
    widget::{Space, column, row, text},
};
use librapix_i18n::TextKey;

pub(crate) fn view(app: &Librapix) -> Element<'_, Message> {
    let root = app.library_stats_root_id.and_then(|root_id| {
        app.state
            .library_roots
            .iter()
            .find(|root| root.id == root_id)
    });

    let title_suffix = root
        .map(display_name_for_root)
        .unwrap_or_else(|| app.i18n.text(TextKey::LibrarySectionLabel).to_owned());
    let dialog_title = format!(
        "{}: {title_suffix}",
        app.i18n.text(TextKey::LibraryStatsDialogTitle)
    );

    let path_line: Element<'_, Message> = if let Some(root) = root {
        text(root.normalized_path.display().to_string())
            .size(FONT_CAPTION)
            .style(text_tertiary)
            .into()
    } else {
        column![].into()
    };

    let stats_body: Element<'_, Message> = if let Some(stats) = &app.library_stats_record {
        column![
            section_heading(app.i18n.text(TextKey::LibraryStatsSummarySectionLabel)),
            render_stats_row(
                app.i18n.text(TextKey::LibraryStatsTotalSizeLabel),
                format::format_file_size(stats.total_size_bytes),
            ),
            render_stats_row(
                app.i18n.text(TextKey::LibraryStatsTotalMediaLabel),
                stats.total_media_count.to_string(),
            ),
            render_stats_row(
                app.i18n.text(TextKey::LibraryStatsTotalImagesLabel),
                stats.total_images_count.to_string(),
            ),
            render_stats_row(
                app.i18n.text(TextKey::LibraryStatsTotalVideosLabel),
                stats.total_videos_count.to_string(),
            ),
            render_stats_row(
                app.i18n.text(TextKey::LibraryStatsImageSizeLabel),
                format::format_file_size(stats.total_image_size_bytes),
            ),
            render_stats_row(
                app.i18n.text(TextKey::LibraryStatsVideoSizeLabel),
                format::format_file_size(stats.total_video_size_bytes),
            ),
            horizontal(),
            section_heading(app.i18n.text(TextKey::LibraryStatsIndexingSectionLabel)),
            render_stats_row(
                app.i18n.text(TextKey::LibraryStatsMissingLabel),
                stats.missing_count.to_string(),
            ),
            render_stats_row(
                app.i18n.text(TextKey::LibraryStatsLastIndexedLabel),
                format::format_timestamp(stats.last_indexed_unix_seconds),
            ),
            render_stats_row(
                app.i18n.text(TextKey::LibraryStatsOldestFileLabel),
                format::format_timestamp(stats.oldest_modified_unix_seconds),
            ),
            render_stats_row(
                app.i18n.text(TextKey::LibraryStatsNewestFileLabel),
                format::format_timestamp(stats.newest_modified_unix_seconds),
            ),
        ]
        .spacing(SPACE_SM)
        .into()
    } else {
        text(app.i18n.text(TextKey::LibraryStatsNotAvailableLabel))
            .size(FONT_BODY)
            .style(text_secondary)
            .into()
    };

    let dialog_content = column![path_line, horizontal(), stats_body,].spacing(SPACE_LG);

    modal_overlay(dialog(DialogSpec {
        title: dialog_title.into(),
        close_message: Some((
            app.i18n.text(TextKey::DismissButton),
            Message::CloseLibraryStatisticsDialog,
        )),
        body: dialog_content.into(),
        footer: None,
        max_width: 560.0,
        max_height: 640.0,
    }))
}

fn render_stats_row<'a>(label: &'a str, value: String) -> Element<'a, Message> {
    row![
        text(label).size(FONT_BODY).style(text_secondary),
        Space::new().width(Length::Fill),
        text(value).size(FONT_BODY).style(text_primary),
    ]
    .align_y(iced::Alignment::Center)
    .into()
}
