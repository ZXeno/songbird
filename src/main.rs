use std::sync::Arc;

use iced::{Size, Theme, window};

use songbird::http_client::ReqwestExecutor;
use songbird::storage::JsonFileRequestRepository;
use songbird::ui::App;

fn main() -> iced::Result {
    let storage = Arc::new(JsonFileRequestRepository::with_default_path());
    let executor = Arc::new(ReqwestExecutor::new().expect("HTTP client should initialize"));

    iced::application(
        move || App::new(storage.clone(), executor.clone()),
        App::update,
        App::view,
    )
    .title(window_title)
    .theme(window_theme)
    .window(window::Settings {
        size: Size::new(1280.0, 840.0),
        min_size: Some(Size::new(900.0, 600.0)),
        ..window::Settings::default()
    })
    .run()
}

fn window_title(_app: &App) -> String {
    String::from("Songbird - Let your APIs sing!")
}

fn window_theme(_app: &App) -> Theme {
    Theme::TokyoNight
}
