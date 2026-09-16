use iced::widget::{
    button, checkbox, column, container, pick_list, row, rule, scrollable, text, text_editor, text_input,
};
use iced::{Alignment, Background, Color, Element, Fill, Length, Theme, alignment, border, task};

use crate::core::{HttpMethod, HttpResponse, new_id};
use crate::format::{self, BodyFormat};

use super::app::App;
use super::editor::{JsonBodyStatus, RequestEditor};
use super::message::Message;
use super::theme::{
    DANGER, HEADER_TEXT_SIZE, MONO, MUTED, SUCCESS, UI_TEXT_SIZE, UI_TEXT_SIZE_SM, WARNING, tinted_text, truncate_label,
    RADIUS, TEXT_SCALE_ALPHA, HOVERED_SCALE_ALPHA,
};

pub(super) struct WorkspaceTab {
    pub(super) id: String,
    pub(super) editor: RequestEditor,
    pub(super) open_id: Option<String>,
    pub(super) response: Option<HttpResponse>,
    pub(super) in_flight: Option<task::Handle>,
}

impl WorkspaceTab {
    pub(super) fn new() -> Self {
        Self {
            id: new_id(),
            editor: RequestEditor::empty(),
            open_id: None,
            response: None,
            in_flight: None,
        }
    }

    pub(super) fn is_pristine(&self) -> bool {
        self.open_id.is_none()
            && self.response.is_none()
            && self.in_flight.is_none()
            && self.editor.url.trim().is_empty()
            && self.editor.body.text().trim().is_empty()
            && self.editor.headers.iter().all(|row| row.header.name.trim().is_empty())
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(super) enum Layout {
    #[default]
    Vertical,
    Horizontal,
}

pub(super) fn view(app: &App) -> Element<'_, Message> {
    let editor_pane = container(
        scrollable(
            column![
                view_request_bar(app),
                view_headers_section(app),
                view_body_section(app),
            ]
            .width(Fill)
            .spacing(12),
        )
        .width(Fill)
        .height(Fill),
    )
    .width(Fill)
    .height(Fill)
    .padding(4);

    let output_content = view_response_section(app);
    let output_pane: Element<'_, Message> = match app.layout {
        Layout::Horizontal => container(scrollable(output_content).width(Fill).height(Fill))
            .width(Fill)
            .height(Fill)
            .padding(4)
            .into(),
        Layout::Vertical => container(output_content)
            .width(Fill)
            .height(Fill)
            .padding(4)
            .into(),
    };

    let panes: Element<'_, Message> = match app.layout {
        Layout::Horizontal => column![editor_pane, rule::horizontal(1), output_pane]
            .width(Fill)
            .height(Fill)
            .spacing(8)
            .into(),
        Layout::Vertical => row![editor_pane, rule::vertical(1), output_pane]
            .width(Fill)
            .height(Fill)
            .spacing(8)
            .into(),
    };

    column![view_tab_bar(app), panes]
        .width(Fill)
        .height(Fill)
        .spacing(4)
        .into()
}

fn view_tab_bar(app: &App) -> Element<'_, Message> {
    let mut bar = row![].spacing(4).align_y(Alignment::Center);
    for (index, tab) in app.tabs.iter().enumerate() {
        let tab_id = tab.id.clone();
        let active = index == app.active_tab;
        let label = tab_label(app, tab);
        bar = bar.push(
            container(
                row![
                    button(text(label).size(UI_TEXT_SIZE))
                        .on_press(Message::TabSelected(tab_id.clone()))
                        .style(tab_label_style(active))
                        .padding(iced::Padding {
                            top: 3.0,
                            right: 0.0,
                            bottom: 3.0,
                            left: 8.0,
                        }),
                    button(text("X").size(UI_TEXT_SIZE))
                        .on_press(Message::TabClosed(tab_id))
                        .style(tab_close_style(active))
                        .padding([3,5]),
                ]
                .spacing(6)
                .align_y(Alignment::Center),
            )
            .padding(2)
            .style(move |theme| tab_chip_style(active, theme)),
        );
    }
    bar = bar.push(
        button(text("+").size(UI_TEXT_SIZE))
        .on_press(Message::TabAddRequested)
        .style(button::secondary),
    );
    bar.into()
}

fn tab_label(app: &App, tab: &WorkspaceTab) -> String {
    let label = match &tab.open_id {
        Some(id) => app
            .saved
            .iter()
            .find(|entry| &entry.id == id)
            .map(|entry| entry.name.clone())
            .unwrap_or_else(|| "New Request".to_owned()),
        None => {
            let url = tab.editor.url.trim();
            if url.is_empty() {
                "New request".to_owned()
            } else {
                format!("{} {}", tab.editor.method, url)
            }
        }
    };
    let label = if tab.in_flight.is_some() {
        format!("* {label}")
    } else {
        label
    };
    truncate_label(&label, 28)
}

fn view_request_bar(app: &App) -> Element<'_, Message> {
    let tab = app.active_tab();
    let send_button = if tab.in_flight.is_some() {
        button(text("Cancel").size(UI_TEXT_SIZE))
            .on_press(Message::CancelRequest)
            .style(button::danger)
    } else {
        button(text("Send").size(UI_TEXT_SIZE))
            .on_press(Message::Send)
            .style(button::primary)
    };

    let open_label = tab
        .open_id
        .as_ref()
        .and_then(|id| app.saved.iter().find(|entry| entry.id == *id))
        .map(|entry| format!("Editing: {}", entry.name));

    let mut bar = row![
        pick_list(
            HttpMethod::ALL,
            Some(tab.editor.method),
            Message::MethodSelected,
        )
        .width(110),
        text_input("https://api.example.com/path", &tab.editor.url)
            .on_input(Message::UrlChanged)
            .on_submit(Message::Send)
            .size(UI_TEXT_SIZE),
        send_button
    ]
    .spacing(8)
    .align_y(Alignment::Center)
    .width(Fill);

    if let Some(label) = open_label {
        bar = bar.push(
            container(tinted_text(label, MUTED).size(UI_TEXT_SIZE))
                .width(Fill)
                .align_x(alignment::Horizontal::Right),
        );
    }
    bar.into()
}

fn view_headers_section(app: &App) -> Element<'_, Message> {
    let tab = app.active_tab();
    let mut body = column![
        row![
            text("Headers").size(HEADER_TEXT_SIZE).width(Fill),
            tinted_text(format!("{} enabled", enabled_header_count(app)), MUTED)
                .size(UI_TEXT_SIZE_SM),
            button(text("+ Add").size(UI_TEXT_SIZE))
                .on_press(Message::HeaderAdded)
                .style(button::secondary),
        ]
        .spacing(8)
        .align_y(Alignment::Center)
    ]
    .spacing(8);

    if tab.editor.headers.is_empty() {
        body = body.push(tinted_text("No headers defined", MUTED).size(UI_TEXT_SIZE));
    }
    for (index, header_row) in tab.editor.headers.iter().enumerate() {
        body = body.push(
            row![
                checkbox(header_row.enabled)
                    .on_toggle(move |enabled| Message::HeaderToggled(index, enabled)),
                text_input("Name", &header_row.header.name)
                    .on_input(move |name| Message::HeaderNameEdited(index, name))
                    .size(UI_TEXT_SIZE)
                    .width(Fill),
                text_input("Value", &header_row.header.value)
                    .on_input(move |value| Message::HeaderValueEdited(index, value))
                    .size(UI_TEXT_SIZE)
                    .width(Fill),
                button(text("X").size(UI_TEXT_SIZE))
                    .on_press(Message::HeaderRemoved(index))
                    .style(button::danger),
            ]
            .spacing(6)
            .align_y(Alignment::Center),
        );
    }

    container(body)
        .width(Fill)
        .padding(10)
        .style(container::rounded_box)
        .into()
}

fn enabled_header_count(app: &App) -> usize {
    app.active_tab()
        .editor
        .headers
        .iter()
        .filter(|row| row.enabled && !row.header.name.trim().is_empty())
        .count()
}

fn view_body_section(app: &App) -> Element<'_, Message> {
    let tab = app.active_tab();
    let badge = match tab.editor.json_status() {
        JsonBodyStatus::Empty => tinted_text("JSON: empty", MUTED),
        JsonBodyStatus::Valid => tinted_text("JSON: valid", SUCCESS),
        JsonBodyStatus::Invalid(error)=> tinted_text(format!("JSON: invalid ({error})"), DANGER),
    };

    let mut body = column![
        row![
            text("Body").size(HEADER_TEXT_SIZE).width(Fill),
            badge.size(UI_TEXT_SIZE_SM)
        ]
        .align_y(Alignment::Center),
        text_editor(&tab.editor.body)
            .height(200)
            .font(MONO)
            .placeholder("Request body (JSON, form data, raw text...)")
            .size(UI_TEXT_SIZE)
            .on_action(Message::BodyEdited),
    ]
    .spacing(8);

    if !tab.editor.method.allows_body() {
        body = body.push(
            tinted_text(
                format!("{} requests do not send a body", tab.editor.method),
                MUTED
            )
            .size(UI_TEXT_SIZE_SM),
        );
    }

    container(body)
        .width(Fill)
        .padding(10)
        .style(container::rounded_box)
        .into()
}

fn view_response_section(app: &App) -> Element<'_, Message> {
    let tab = app.active_tab();
    if tab.in_flight.is_some() {
        return centered_hint("Sending request...");
    }
    let Some(response) = &tab.response else {
        return centered_hint("No response yet");
    };

    let body_format = format::detect_format(&response.body);

    let mut summary = row![
        tinted_text(
            format!("{} {}",
                response.status,
                response.reason.as_deref().unwrap_or("")),
                status_color(response.status),
            )
            .size(UI_TEXT_SIZE)
            .font(MONO),
        tinted_text(
            format!("{} · {}{}",
                response.duration_ms,
                format_bytes(response.body.len()),
                if response.truncated {
                    " · truncated"
                } else {
                    ""
                }
            ),
            MUTED,
        )
        .size(UI_TEXT_SIZE)
    ]
    .spacing(12)
    .align_y(Alignment::Center);

    if body_format != BodyFormat::Plain {
        summary = summary.push(
                button(text(if app.pretty_print { "Pretty" } else { "Raw" }).size(UI_TEXT_SIZE))
                .on_press(Message::PrettyPrintToggled)
                .style(button::secondary),
            );
    }

    let mut headers = column![].spacing(2);
    for header in &response.headers {
        headers = headers.push(
                text(format!("{}: {}", header.name, header.value))
                    .size(UI_TEXT_SIZE)
                    .font(MONO),
            );
    }

    let body_label = if app.pretty_print {
        format::format_body(&response.body, body_format)
    } else {
        response.body.clone()
    };

    let mut card = column![
        summary,
        tinted_text("Response headers", MUTED).size(HEADER_TEXT_SIZE),
        scrollable(headers).height(110),
        rule::horizontal(1),
        scrollable(text(body_label).font(MONO).size(UI_TEXT_SIZE))
            .height(body_panel_height(app.layout))
            .width(Fill),
    ]
    .spacing(8);

    if app.layout == Layout::Vertical {
        card = card.height(Fill);
    }

    let mut card_container = container(card).width(Fill).padding(10);
    if app.layout == Layout::Vertical {
        card_container = card_container.height(Fill);
    }
    card_container.style(container::rounded_box).into()
}

fn body_panel_height(layout: Layout) -> Length {
    match layout {
        Layout::Vertical => Length::Fill,
        Layout::Horizontal => Length::from(280.0),
    }
}

fn tab_chip_style(active: bool, theme: &Theme) -> container::Style {
    let palette = theme.extended_palette();
    let background = if active {
        palette.secondary.base.color
    } else {
        palette.background.weak.color
    };
    container::Style {
        background: Some(Background::Color(background)),
        border: border::rounded(RADIUS),
        ..container::Style::default()
    }
}

fn tab_label_style (active: bool) -> impl Fn(&Theme, button::Status) -> button::Style {
    move |theme, status | {
        let palette = theme.extended_palette();
        let mut text_color = if active {
            palette.secondary.base.text
        } else {
            palette.background.base.text
        };
        if let button::Status::Hovered = status {
            text_color = text_color.scale_alpha(TEXT_SCALE_ALPHA);
        }
        button::Style {
            text_color,
            ..button::Style::default()
        }
    }
}

fn tab_close_style(active: bool) -> impl Fn(&Theme, button::Status) -> button::Style {
    move |theme, status| {
        let palette = theme.extended_palette();
        let base_text = if active {
            palette.secondary.base.text
        } else {
            palette.background.base.text
        };
        let mut style = button::Style {
            text_color: base_text.scale_alpha(TEXT_SCALE_ALPHA),
            ..button::Style::default()
        };
        if let button::Status::Hovered = status {
            style.background = Some(Background::Color(
                        base_text.scale_alpha(HOVERED_SCALE_ALPHA),
                    ));
            style.border = border::rounded(RADIUS);
            style.text_color = base_text;
        }
        style
    }
}

fn centered_hint(message: &str) -> Element<'_, Message> {
    container(tinted_text(message, MUTED).size(UI_TEXT_SIZE_SM))
        .width(Fill)
        .height(Fill)
        .center_x(Fill)
        .center_y(Fill)
        .into()
}

fn format_bytes(len: usize) -> String {
    if len < 1024 {
        format!("{len} B")
    } else if len< 1024 * 1024 {
        format!("{:.1} KB", len as f64 / 1024.0)
    } else {
        format!("{:.1} MB", len as f64 / (1024.0 * 1024.0))
    }
}

fn status_color(status: u16) -> Color {
    match status {
        200..=299 => SUCCESS,
        300..=399 => WARNING,
        _ => DANGER,
    }
}
