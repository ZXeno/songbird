use iced::widget::{button, column, container, row, scrollable, text};
use iced::{Alignment, Color, Element, Fill};

use crate::core::HttpMethod;

use super::app::App;
use super::message::Message;
use super::theme::{HEADER_TEXT_SIZE, MONO, MUTED, UI_TEXT_SIZE, tinted_text, truncate_label};

const SIDEBAR_WIDTH: f32 = 260.0;

pub(super) fn view(app: &App) -> Element<'_, Message> {
    let mut list = column![].spacing(4);
    for entry in &app.saved {
        let id = entry.id.clone();
        let label = truncate_label(&entry.name, 30);
        let method = entry.request.method;
        list = list.push(
            row![
              button(
                  row![
                      text(method.to_string())
                          .size(UI_TEXT_SIZE)
                          .font(MONO)
                          .style(move |_| text::Style {
                              color: Some(method_color(method))
                          }),
                      text(label).size(UI_TEXT_SIZE),
                  ]
                  .spacing(6)
                  .align_y(Alignment::Center)
              )
              .width(Fill)
              .on_press(Message::OpenRequested(id.clone()))
              .style(button::text),
              button(text("X").size(UI_TEXT_SIZE))
                  .on_press(Message::DeleteRequested(id))
                  .style(button::danger)
            ]
            .align_y(Alignment::Center),
        );
    }
    if app.saved.is_empty() {
        list = list.push(tinted_text("Nothing saved yet", MUTED).size(UI_TEXT_SIZE));
    }

    container(
        column![
            text(format!("Saved ({})", app.saved.len())).size(HEADER_TEXT_SIZE),
            scrollable(list).height(Fill),
        ]
        .spacing(8),
    )
    .width(SIDEBAR_WIDTH)
    .height(Fill)
    .padding(8)
    .style(container::rounded_box)
    .into()
}

fn method_color(method: HttpMethod) -> Color {
    match method {
        HttpMethod::Get => Color::from_rgb(0.30, 0.69, 0.42),
        HttpMethod::Post => Color::from_rgb(0.95, 0.63, 0.22),
        HttpMethod::Put => Color::from_rgb(0.36, 0.60, 0.90),
        HttpMethod::Patch => Color::from_rgb(0.68, 0.45, 0.90),
        HttpMethod::Delete => Color::from_rgb(0.90, 0.36, 0.36),
        HttpMethod::Head => Color::from_rgb(0.60, 0.60, 0.60),
        HttpMethod::Options => Color::from_rgb(0.30, 0.75, 0.75),
    }
}
