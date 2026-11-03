// SPDX-License-Identifier: GPL-3.0

use crate::{app::Message, fl};
use cosmic::widget::menu::{self, ItemHeight, ItemWidth, KeyBind};
use cosmic::{Apply, Element};
use std::collections::HashMap;

/// Represents a Action that executes after clicking on the application Menu
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MenuAction {
    /// Open the About [`ContextPage`] of the application
    About,
    /// Open the Settings [`ContextPage`] of the application
    Settings,
    /// Open a FileDialog to pick a new file to open
    OpenFile,
    /// Open a FileDialog to pick a workspace folder to open
    OpenFolder,
    /// Create a new empty file
    NewFile,
    /// Create a new vault file
    NewVaultFile,
    /// Create a new vault folder
    NewVaultFolder,
    /// Save the current file
    SaveFile,
    /// Toggle the preview for the current file
    TogglePreview,
    /// Toggle the editor for the current file
    ToggleEditor,
    /// Toggle line numbers
    ToggleLineNumbers,
    /// Undo
    Undo,
    /// Redo
    Redo,
    /// Paste image from clipboard
    PasteImage,
    /// Search
    Search,
    /// Close the current open popup dialog
    CloseCurrentDialog,
    /// Open a recent file by index
    OpenRecentFile(usize),
    /// Open a recent workspace by index
    OpenRecentWorkspace(usize),
    /// Clear recent files list
    ClearRecentFiles,
    /// Clear recent workspaces list
    ClearRecentWorkspaces,
}

impl menu::action::MenuAction for MenuAction {
    type Message = crate::app::Message;

    fn message(&self) -> Self::Message {
        match self {
            MenuAction::About => Message::MenuAction(MenuAction::About),
            MenuAction::Settings => Message::MenuAction(MenuAction::Settings),
            MenuAction::OpenFile => Message::MenuAction(MenuAction::OpenFile),
            MenuAction::OpenFolder => Message::MenuAction(MenuAction::OpenFolder),
            MenuAction::NewFile => Message::MenuAction(MenuAction::NewFile),
            MenuAction::NewVaultFile => Message::MenuAction(MenuAction::NewVaultFile),
            MenuAction::NewVaultFolder => Message::MenuAction(MenuAction::NewVaultFolder),
            MenuAction::SaveFile => Message::MenuAction(MenuAction::SaveFile),
            MenuAction::TogglePreview => Message::MenuAction(MenuAction::TogglePreview),
            MenuAction::ToggleEditor => Message::MenuAction(MenuAction::ToggleEditor),
            MenuAction::ToggleLineNumbers => Message::MenuAction(MenuAction::ToggleLineNumbers),
            MenuAction::Undo => Message::MenuAction(MenuAction::Undo),
            MenuAction::Redo => Message::MenuAction(MenuAction::Redo),
            MenuAction::PasteImage => Message::MenuAction(MenuAction::PasteImage),
            MenuAction::Search => Message::MenuAction(MenuAction::Search),
            MenuAction::CloseCurrentDialog => Message::MenuAction(MenuAction::CloseCurrentDialog),
            MenuAction::OpenRecentFile(i) => Message::MenuAction(MenuAction::OpenRecentFile(*i)),
            MenuAction::OpenRecentWorkspace(i) => {
                Message::MenuAction(MenuAction::OpenRecentWorkspace(*i))
            }
            MenuAction::ClearRecentFiles => Message::MenuAction(MenuAction::ClearRecentFiles),
            MenuAction::ClearRecentWorkspaces => {
                Message::MenuAction(MenuAction::ClearRecentWorkspaces)
            }
        }
    }
}

pub fn menu_bar<'a>(
    key_binds: &HashMap<KeyBind, MenuAction>,
    recent_files: &[std::path::PathBuf],
    recent_workspaces: &[std::path::PathBuf],
) -> Element<'a, Message> {
    let mut recent_file_items = Vec::new();
    for (i, path) in recent_files.iter().enumerate() {
        let label = path
            .file_name()
            .map(|f| f.to_string_lossy().to_string())
            .unwrap_or_else(|| path.to_string_lossy().to_string());
        recent_file_items.push(menu::Item::Button(label, None, MenuAction::OpenRecentFile(i)));
    }
    if recent_file_items.is_empty() {
        recent_file_items.push(menu::Item::ButtonDisabled(
            fl!("no-recent-files"),
            None,
            MenuAction::OpenFile,
        ));
    } else {
        recent_file_items.push(menu::Item::Divider);
        recent_file_items.push(menu::Item::Button(
            fl!("clear-recent"),
            None,
            MenuAction::ClearRecentFiles,
        ));
    }

    let mut recent_ws_items = Vec::new();
    for (i, path) in recent_workspaces.iter().enumerate() {
        let label = path
            .file_name()
            .map(|f| f.to_string_lossy().to_string())
            .unwrap_or_else(|| path.to_string_lossy().to_string());
        recent_ws_items.push(menu::Item::Button(label, None, MenuAction::OpenRecentWorkspace(i)));
    }
    if recent_ws_items.is_empty() {
        recent_ws_items.push(menu::Item::ButtonDisabled(
            fl!("no-recent-workspaces"),
            None,
            MenuAction::OpenFolder,
        ));
    } else {
        recent_ws_items.push(menu::Item::Divider);
        recent_ws_items.push(menu::Item::Button(
            fl!("clear-recent"),
            None,
            MenuAction::ClearRecentWorkspaces,
        ));
    }

    menu::bar(vec![
        menu::Tree::with_children(
            menu::root(fl!("file")).apply(Element::from),
            menu::items(
                key_binds,
                vec![
                    menu::Item::Button(fl!("new-vault-file"), None, MenuAction::NewVaultFile),
                    menu::Item::Button(fl!("new-folder"), None, MenuAction::NewVaultFolder),
                    menu::Item::Button(fl!("open-file"), None, MenuAction::OpenFile),
                    menu::Item::Button(fl!("open-folder"), None, MenuAction::OpenFolder),
                    menu::Item::Folder(fl!("open-recent-files"), recent_file_items),
                    menu::Item::Folder(fl!("open-recent-workspaces"), recent_ws_items),
                    menu::Item::Button(fl!("save-file"), None, MenuAction::SaveFile),
                    menu::Item::Divider,
                    menu::Item::Button(fl!("new-file"), None, MenuAction::NewFile),
                ],
            ),
        ),
        menu::Tree::with_children(
            menu::root(fl!("edit")).apply(Element::from),
            menu::items(
                key_binds,
                vec![
                    menu::Item::Button(fl!("undo"), None, MenuAction::Undo),
                    menu::Item::Button(fl!("redo"), None, MenuAction::Redo),
                    menu::Item::Button(fl!("paste-image"), None, MenuAction::PasteImage),
                    menu::Item::Button(fl!("search"), None, MenuAction::Search),
                ],
            ),
        ),
        menu::Tree::with_children(
            menu::root(fl!("view")).apply(Element::from),
            menu::items(
                key_binds,
                vec![
                    menu::Item::Button(fl!("toggle-preview"), None, MenuAction::TogglePreview),
                    menu::Item::Button(fl!("toggle-editor"), None, MenuAction::ToggleEditor),
                    menu::Item::Button(fl!("toggle-line-numbers"), None, MenuAction::ToggleLineNumbers),
                    menu::Item::Divider,
                    menu::Item::Button(fl!("about"), None, MenuAction::About),
                    menu::Item::Button(fl!("settings"), None, MenuAction::Settings),
                ],
            ),
        ),
    ])
    .item_height(ItemHeight::Dynamic(40))
    .item_width(ItemWidth::Uniform(270))
    .spacing(4.0)
    .into()
}
