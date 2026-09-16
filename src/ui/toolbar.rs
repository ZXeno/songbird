use iced::widget::{button, container, row, text};
use iced::{Alignment, Element, Fill, alignment};

use super::app::{App, Notice};
use super::message::Message;
use super::theme::{DANGER, MUTED, SUCCESS, UI_TEXT_SIZE, tinted_text};
use super::workspace::Layout;

pub(super) fn view(app: &App) -> Element<'_, Message> {
    let notice: Element<'_, Message> = match &app.notice {
        Some(notice) => {
            let color = match notice {
                Notice::Info(_) => MUTED,
                Notice::Success(_) => SUCCESS,
                Notice::Error(_) => DANGER,
            };
            tinted_text(notice.text(), color).into()
        }
        None => text("").into(),
    };

    row![
        button(text("Show Left Pane").size(UI_TEXT_SIZE))
            .on_press(Message::SidebarToggled)
            .style(button::secondary),
        button(text("New").size(UI_TEXT_SIZE))
            .on_press(Message::NewRequested)
            .style(button::secondary),
        button(text("Import cURL").size(UI_TEXT_SIZE))
            .on_press(Message::ImportToggled)
            .style(button::secondary),
        button(text("Save Current").size(UI_TEXT_SIZE))
            .on_press(Message::SaveRequested)
            .style(button::secondary),
        button(text(layout_label(app.layout)).size(UI_TEXT_SIZE))
            .on_press(Message::LayoutToggled)
            .style(button::secondary),
        container(notice)
            .width(Fill)
            .align_x(alignment::Horizontal::Right),
    ]
    .spacing(8)
    .align_y(Alignment::Center)
    .into()
}

fn layout_label(layout: Layout) -> &'static str {
    match layout {
        Layout::Horizontal => "Horizontal",
        Layout::Vertical => "Vertical",
    }
}
