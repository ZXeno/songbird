use iced::widget::{button, column, container, row, text, text_editor};
use iced::{Alignment, Element, Fill, alignment};

use super::app::App;
use super::message::Message;
use super::theme::{HEADER_TEXT_SIZE, MONO, SUCCESS, UI_TEXT_SIZE, tinted_text};

#[derive(Clone, Debug)]
pub(super) enum ImportState {
    Input { content: text_editor::Content },
    ConfirmSave { tab_id: String },
}

pub(super) fn view(app: &App) -> Element<'_, Message> {
    let Some(state) = &app.import else {
        return column![].into();
    };

    let panel = match state {
        ImportState::Input { content } => column! [
            row![
                text("Paste a cURL command").size(HEADER_TEXT_SIZE).width(Fill),
                button(text("Cancel").size(UI_TEXT_SIZE))
                    .on_press(Message::ImportToggled)
                    .style(button::secondary),
            ]
                .align_y(Alignment::Center),
                text_editor(content)
                    .height(120)
                    .font(MONO)
                    .placeholder("curl -X POST https://api.example.com \\\n -H 'Content-Type: application/json' \\\n -d '{...}'")
                    .on_action(Message::ImportEdited),
                container(
                    button(text("Parse & load").size(UI_TEXT_SIZE))
                    .on_press(Message::ImportConfirmed)
                    .style(button::primary)
                )
                    .width(Fill)
                    .align_x(alignment::Horizontal::Right),
        ]
        .spacing(8),
        ImportState::ConfirmSave { .. } => column![
            row![
                tinted_text("Request imported into a new tab", SUCCESS)
                    .size(HEADER_TEXT_SIZE)
                    .width(Fill),
                button(text("Close").size(UI_TEXT_SIZE))
                    .on_press(Message::ImportSaveDeclined)
                    .style(button::secondary),
            ]
            .align_y(Alignment::Center),
            text("Save this request to your saved requests?").size(UI_TEXT_SIZE),
            row![
                button(text("Save and import").size(UI_TEXT_SIZE))
                    .on_press(Message::ImportSaveAccepted)
                    .style(button::primary),
                button(text("Import without save").size(UI_TEXT_SIZE))
                    .on_press(Message::ImportSaveDeclined)
                    .style(button::secondary),
            ]
            .spacing(8),
        ]
        .spacing(8)
    };

    container(panel)
        .width(Fill)
        .padding(10)
        .style(container::rounded_box)
        .into()
}
