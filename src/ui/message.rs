use iced::widget::text_editor;

use crate::core::{HttpMethod, HttpResponse};

#[derive(Clone, Debug)]
pub enum Message {
    BodyEdited(text_editor::Action),
    CancelRequest,
    DeleteRequested(String),
    HeaderAdded,
    HeaderNameEdited(usize, String),
    HeaderRemoved(usize),
    HeaderToggled(usize, bool),
    HeaderValueEdited(usize, String),
    ImportToggled,
    ImportEdited(text_editor::Action),
    ImportConfirmed,
    ImportSaveAccepted,
    ImportSaveDeclined,
    LayoutToggled,
    MethodSelected(HttpMethod),
    NewRequested,
    OpenRequested(String),
    PrettyPrintToggled,
    ResponseReady(String, Result<HttpResponse, String>),
    Send,
    SidebarToggled,
    RequestRenameStarted,
    RequestNameChanged(String),
    RequestRenameCommitted,
    SaveRequested,
    TabAddRequested,
    TabClosed(String),
    TabSelected(String),
    UrlChanged(String),
}
