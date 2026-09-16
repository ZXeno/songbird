use iced::widget::text_editor;

use crate::core::{HttpMethod, HttpResponse};

#[derive(Clone, Debug)]
pub enum Message {
    MethodSelected(HttpMethod),
    UrlChanged(String),
    BodyEdited(text_editor::Action),
    HeaderNameEdited(usize, String),
    HeaderValueEdited(usize, String),
    HeaderToggled(usize, bool),
    HeaderRemoved(usize),
    HeaderAdded,
    Send,
    CancelRequest,
    ResponseReady(String, Result<HttpResponse, String>),
    SidebarToggled,
    LayoutToggled,
    PrettyPrintToggled,
    NewRequested,
    SaveRequested,
    OpenRequested(String),
    DeleteRequested(String),
    TabSelected(String),
    TabClosed(String),
    TabAddRequested,
    ImportToggled,
    ImportEdited(text_editor::Action),
    ImportConfirmed,
    ImportSaveAccepted,
    ImportSaveDeclined,
}
