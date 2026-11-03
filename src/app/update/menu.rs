// SPDX-License-Identifier: GPL-3.0

use crate::app::app_menu::MenuAction;
use crate::app::context_page::ContextPage;
use crate::app::core::utils;
use crate::app::{AppModel, Message, PreviewState};
use crate::app::{State, dialogs};
use cosmic::prelude::*;

impl AppModel {
    pub fn handle_menu_action(&mut self, action: MenuAction) -> Task<cosmic::Action<Message>> {
        let State::Ready { preview_state, .. } = &mut self.state else {
            return Task::none();
        };

        match action {
            MenuAction::About => self.handle_toggle_context_page(ContextPage::About),
            MenuAction::Settings => self.handle_toggle_context_page(ContextPage::Settings),
            MenuAction::OpenFile => Task::perform(
                async move {
                    match utils::files::open_markdown_file_picker().await {
                        Some(path) => Some(utils::files::load_file(path.into()).await),
                        None => None,
                    }
                },
                |res| match res {
                    Some(result) => cosmic::action::app(Message::OpenFile(result)),
                    None => cosmic::action::none(),
                },
            ),
            MenuAction::OpenFolder => {
                let current = self.active_workspace_path().to_string_lossy().to_string();
                Task::perform(
                    async move {
                        utils::files::open_folder_picker(current).await
                    },
                    |res| match res {
                        Some(path) => {
                            cosmic::action::app(Message::OpenWorkspaceFolder(std::path::PathBuf::from(path)))
                        }
                        None => cosmic::action::none(),
                    },
                )
            }
            MenuAction::NewFile => self.handle_new_file(),
            MenuAction::NewVaultFile => {
                self.handle_dialog_action(dialogs::DialogAction::OpenNewVaultFileDialog)
            }
            MenuAction::NewVaultFolder => {
                self.handle_dialog_action(dialogs::DialogAction::OpenNewVaultFolderDialog)
            }
            MenuAction::SaveFile => self.handle_save_file(),
            MenuAction::TogglePreview => {
                match preview_state {
                    PreviewState::Hidden => *preview_state = PreviewState::Shown,
                    PreviewState::Shown => *preview_state = PreviewState::Hidden,
                    PreviewState::PreviewOnly => *preview_state = PreviewState::Shown,
                }
                Task::none()
            }
            MenuAction::ToggleEditor => {
                match preview_state {
                    PreviewState::PreviewOnly => *preview_state = PreviewState::Shown,
                    PreviewState::Shown => *preview_state = PreviewState::PreviewOnly,
                    PreviewState::Hidden => *preview_state = PreviewState::Shown,
                }
                Task::none()
            }
            MenuAction::ToggleLineNumbers => {
                let next = match self.config.show_line_numbers {
                    crate::config::ShowState::Show => crate::config::ShowState::Hide,
                    crate::config::ShowState::Hide => crate::config::ShowState::Show,
                };
                self.handle_config_input(crate::config::ConfigInput::LineNumbersShowState(next))
            }
            MenuAction::ToggleSidebar => {
                self.core.nav_bar_toggle();
                Task::none()
            }
            MenuAction::ToggleAutohideSidebar => {
                let next = match self.config.autohide_sidebar {
                    crate::config::BoolState::Yes => crate::config::BoolState::No,
                    crate::config::BoolState::No => crate::config::BoolState::Yes,
                };
                self.handle_config_input(crate::config::ConfigInput::AutohideSidebar(next))
            }
            MenuAction::Undo => self.handle_undo(),
            MenuAction::Redo => self.handle_redo(),
            MenuAction::PasteImage => self.handle_paste_image(),
            MenuAction::Search => self.handle_search(utils::search::SearchAction::ToggleSearch),
            MenuAction::CloseCurrentDialog => self.handle_dialog_action(dialogs::DialogAction::DialogCancel),
            MenuAction::OpenRecentFile(index) => {
                if let Some(path) = self.config.recent_files.get(index).cloned() {
                    if path.exists() {
                        return Task::perform(
                            utils::files::load_file(path),
                            |res| cosmic::action::app(Message::OpenFile(res)),
                        );
                    } else {
                        self.config.recent_files.retain(|p| p != &path);
                        if let Some(handler) = crate::config::CedillaConfig::config_handler() {
                            let _ = self.config.set_recent_files(&handler, self.config.recent_files.clone());
                        }
                    }
                }
                Task::none()
            }
            MenuAction::OpenRecentWorkspace(index) => {
                if let Some(path) = self.config.recent_workspaces.get(index).cloned() {
                    if path.exists() {
                        return self.handle_open_workspace_folder(path);
                    } else {
                        self.config.recent_workspaces.retain(|p| p != &path);
                        if let Some(handler) = crate::config::CedillaConfig::config_handler() {
                            let _ = self.config.set_recent_workspaces(&handler, self.config.recent_workspaces.clone());
                        }
                    }
                }
                Task::none()
            }
            MenuAction::ClearRecentFiles => {
                self.config.recent_files.clear();
                if let Some(handler) = crate::config::CedillaConfig::config_handler() {
                    let _ = self.config.set_recent_files(&handler, Vec::new());
                }
                Task::none()
            }
            MenuAction::ClearRecentWorkspaces => {
                self.config.recent_workspaces.clear();
                if let Some(handler) = crate::config::CedillaConfig::config_handler() {
                    let _ = self.config.set_recent_workspaces(&handler, Vec::new());
                }
                Task::none()
            }
        }
    }
}
