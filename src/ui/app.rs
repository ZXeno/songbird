use std::sync::Arc;

use iced::widget::{column, row, rule, text_editor};
use iced::{Element, Fill, Task};

use crate::core::{
    curl,
    SavedRequest,
};
use crate::storage::RequestRepository;
use crate::http_client::HttpExecutor;

use super::editor::RequestEditor;
use super::import::{self, ImportState};
use super::message::Message;
use super::sidebar;
use super::toolbar;
use super::workspace::{self, Layout, WorkspaceTab};

#[derive(Clone, Debug)]
pub(super) enum Notice {
    Info(String),
    Success(String),
    Error(String),
}

impl Notice {
    pub(super) fn text(&self) -> &str {
        match self {
            Notice::Info(message) | Notice::Success(message) | Notice::Error(message) => message, 
        }
    }
}

pub struct App {
    pub(super) tabs: Vec<WorkspaceTab>,
    pub(super) active_tab: usize,
    pub(super) saved: Vec<SavedRequest>,
    pub(super) sidebar_open: bool,
    pub(super) layout: Layout,
    pub(super) pretty_print: bool,
    pub(super) import: Option<ImportState>,
    pub(super) notice: Option<Notice>,
    storage: Arc<dyn RequestRepository>,
    executor: Arc<dyn HttpExecutor>,
}

impl App {
    pub fn new(storage: Arc<dyn RequestRepository>, executor: Arc<dyn HttpExecutor>) -> Self {
        let mut app = Self {
            tabs: vec![WorkspaceTab::new()],
            active_tab: 0,
            saved: Vec::new(),
            sidebar_open: true,
            layout: Layout::default(),
            pretty_print: true,
            import: None,
            notice: None,
            storage,
            executor,
        };
        match app.storage.load_all() {
            Ok(entries) => app.saved = entries,
            Err(error) => {
                app.notice = Some(Notice::Error(format!("Failed to load saved requests: {error}")))
            }
        }
        app
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::MethodSelected(method) => self.active_tab_mut().editor.method = method,
            Message::UrlChanged(url) => self.active_tab_mut().editor.url = url,
            Message::BodyEdited(action) => self.active_tab_mut().editor.body.perform(action),
            Message::HeaderNameEdited(index, name) => {
                if let Some(row) = self.active_tab_mut().editor.headers.get_mut(index) {
                    row.header.name = name;
                }
            }
            Message::HeaderValueEdited(index, value) => {
                if let Some(row) = self.active_tab_mut().editor.headers.get_mut(index) {
                    row.header.value = value;
                }
            }
            Message::HeaderToggled(index, enabled) => {
                if let Some(row) = self.active_tab_mut().editor.headers.get_mut(index) {
                    row.enabled = enabled;
                }
            }
            Message::HeaderRemoved(index) => self.active_tab_mut().editor.remove_header(index),
            Message::HeaderAdded => self.active_tab_mut().editor.add_header(),
            Message::Send => return self.send(),
            Message::CancelRequest => self.cancel_request(),
            Message::ResponseReady(tab_id, result) => {
                if let Some(tab) = self.tabs.iter_mut().find(|tab| tab.id == tab_id) {
                    tab.in_flight = None;
                    match result {
                        Ok(response) => tab.response = Some(response),
                        Err(error) => self.notice = Some(Notice::Error(error)),
                    }
                }
            }
            Message::SidebarToggled => self.sidebar_open = !self.sidebar_open,
            Message::LayoutToggled => {
                self.layout = match self.layout {
                    Layout::Horizontal => Layout::Vertical,
                    Layout::Vertical => Layout::Horizontal,
                }
            }
            Message::PrettyPrintToggled => self.pretty_print = !self.pretty_print,
            Message::NewRequested => self.new_tab_request(),
            Message::SaveRequested => self.save_active(),
            Message::OpenRequested(id) => self.open_saved(&id),
            Message::DeleteRequested(id) => self.delete_saved(&id),
            Message::TabSelected(id) => {
                if let Some(index) = self.tabs.iter().position(|tab| tab.id == id) {
                    self.active_tab = index;
                }
            }
            Message::TabClosed(id) => self.close_tab(&id),
            Message::TabAddRequested => {
                self.tabs.push(WorkspaceTab::new());
                self.active_tab = self.tabs.len() - 1;
                self.notice = None;
            }
            Message::ImportToggled => {
                self.import = match self.import.take() {
                    Some(_) => None,
                    None => Some(ImportState::Input {
                        content: text_editor::Content::new(),
                    }),
                };
            }
            Message::ImportEdited(action) => {
                if let Some(ImportState::Input { content }) = &mut self.import {
                    content.perform(action);
                }
            }
            Message::ImportConfirmed => self.run_import(),
            Message::ImportSaveAccepted => {
                if let Some(ImportState::ConfirmSave { tab_id }) = self.import.take() {
                    self.save_tab(&tab_id);
                }
            }
            Message::ImportSaveDeclined => self.import = None,
        }
        Task::none()
    }

    pub fn view(&self) -> Element<'_, Message> {
        let main = if self.sidebar_open {
            row![
                sidebar::view(self),
                rule::vertical(1),
                workspace::view(self),
            ]
        } else {
            row![workspace::view(self)]
        };

        column![
            toolbar::view(self),
            import::view(self),
            main.width(Fill).height(Fill),
        ]
        .width(Fill)
        .height(Fill)
        .spacing(10)
        .padding(10)
        .into()
    }

    pub(super) fn active_tab(&self) -> &WorkspaceTab {
        &self.tabs[self.active_tab.min(self.tabs.len() - 1)]
    }

    fn active_tab_mut(&mut self) -> &mut WorkspaceTab {
        let index = self.active_tab.min(self.tabs.len() - 1);
        &mut self.tabs[index]
    }

    fn send(&mut self) -> Task<Message> {
        let tab = self.active_tab_mut();
        if tab.in_flight.is_some() {
            return Task::none();
        }
        let request = tab.editor.to_request();
        if request.url.is_empty() {
            self.notice = Some(Notice::Error("Enter a URL before sending".to_owned()));
            return Task::none();
        }
        let tab_id = tab.id.clone();
        tab.response = None;
        self.notice = None;
        let executor = self.executor.clone();
        let (task, handle) = Task::perform(executor.execute(request), move |result| {
            Message::ResponseReady(tab_id.clone(), result.map_err(|error| error.to_string()))
        })
        .abortable();
        self.active_tab_mut().in_flight = Some(handle);
        task
    }

    fn cancel_request(&mut self) {
        if let Some(handle) = self.active_tab_mut().in_flight.take() {
            handle.abort();
        }
        self.notice = Some(Notice::Info("Request cancelled".to_owned()));
    }

    fn new_tab_request(&mut self) {
        if !self.active_tab().is_pristine() {
            self.tabs.push(WorkspaceTab::new());
            self.active_tab = self.tabs.len() - 1;
        }
        self.notice = None;
    }

    fn close_tab(&mut self, id: &str) {
        let Some(index) = self.tabs.iter().position(|tab| tab.id == id) else {
            return;
        };
        let mut tab = self.tabs.remove(index);
        if let Some(handle) = tab.in_flight.take() {
            handle.abort();
        }
        if self.tabs.is_empty() {
            self.tabs.push(WorkspaceTab::new());
            self.active_tab = 0;
            return;
        }
        if index < self.active_tab {
            self.active_tab -= 1;
        }
        self.active_tab = self.active_tab.min(self.tabs.len() - 1);
    }

    fn save_active(&mut self) {
        let tab_id = self.active_tab().id.clone();
        self.save_tab(&tab_id);
    }

    fn save_tab(&mut self, tab_id: &str) {
        let Some(index) = self.tabs.iter().position(|tab| tab.id == tab_id) else {
            return;
        };
        let request = self.tabs[index].editor.to_request();
        if request.url.is_empty() {
            self.notice = Some(Notice::Error("Enter a URL before saving".to_owned()));
            return;
        }
        if let Some(id) = self.tabs[index].open_id.clone() {
            if let Some(existing) = self.saved.iter_mut().find(|entry| entry.id == id) {
                existing.request = request;
                existing.refresh_name();
                let snapshot = existing.clone();
                match self.storage.save(&snapshot) {
                    Ok(()) => {
                        self.notice = Some(Notice::Success(format!("Updated '{}'", snapshot.name)));
                    }
                    Err(error) => self.notice = Some(Notice::Error(error.to_string())),
                }
                return;
            }
        }
        let entry = SavedRequest::new(request);
        match self.storage.save(&entry) {
            Ok(()) => {
                self.tabs[index].open_id = Some(entry.id.clone());
                self.saved.push(entry);
                self.notice = Some(Notice::Success("Request saved".to_owned()));
            }
            Err(error) => self.notice = Some(Notice::Error(error.to_string())),
        }
    }

    fn open_saved(&mut self, id: &str) {
        let Some(entry) = self.saved.iter().find(|entry| entry.id == id) else {
            return;
        };
        if let Some(index) = self
            .tabs
                .iter()
                .position(|tab| tab.open_id.as_deref() == Some(id)) {
                    self.active_tab = index;
                    return;
        }
        let mut tab = WorkspaceTab::new();
        tab.editor = RequestEditor::load(&entry.request);
        tab.open_id = Some(entry.id.clone());
        self.tabs.push(tab);
        self.active_tab = self.tabs.len() - 1;
        self.notice = None;
    }
    
    fn delete_saved(&mut self, id: &str) {
        match self.storage.delete(id) {
            Ok(()) => {
                self.saved.retain(|entry| entry.id != id);
                for tab in &mut self.tabs {
                    if tab.open_id.as_deref() == Some(id) {
                        tab.open_id = None;
                    }
                }
                self.notice = Some(Notice::Success("Request deleted".to_owned()));
            }
            Err(error) => self.notice = Some(Notice::Error(error.to_string())),
        }
    }

    fn run_import(&mut self) {
        let Some(ImportState::Input { content }) = self.import.take() else {
            return;
        };
        match curl::parse(&content.text()) {
            Ok(request) => {
                let mut tab = WorkspaceTab::new();
                tab.editor = RequestEditor::load(&request);
                let tab_id = tab.id.clone();
                self.tabs.push(tab);
                self.active_tab = self.tabs.len() - 1;
                self.notice = Some(Notice::Success("cURL command imported".to_owned()));
                self.import = Some(ImportState::ConfirmSave { tab_id });
            }
            Err(error) => {
                self.import = Some(ImportState::Input { content });
                self.notice = Some(Notice::Error(error.to_string()));
            }
        }
    }
}
