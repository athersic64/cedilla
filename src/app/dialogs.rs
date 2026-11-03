// SPDX-License-Identifier: GPL-3.0

use std::{collections::VecDeque, path::PathBuf};

use cosmic::{Element, Task, iced::{Alignment, Length}, theme, widget};

use crate::{
    app::{DiscardChangesAction, Message},
    fl,
};

/// Represents a [`DialogPage`] of the application
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DialogPage {
    /// Dialog for creating a new file on the vault
    NewVaultFile(String),
    /// Dialog for creating a new folder on the vault
    NewVaultFolder(String),
    /// Delete the currently selected folder/file
    DeleteNode(PathBuf),
    /// Rename the currently selected folder/file
    RenameNode(PathBuf, String),
    /// Move the given path to a selected folder: first = what to move, second = selected target folder (None if none chosen yet)
    MoveNode(PathBuf, Option<PathBuf>),
    /// Dialog for when closing a file with pending changes
    ConfirmCloseFile(DiscardChangesAction),
    /// Open file was modified externally
    ExternalFileModified(PathBuf),
    /// Insert table with interactive matrix preview
    InsertTable { rows: usize, cols: usize },
}

impl DialogPage {
    /// View of the [`DialogPage`]
    pub fn display(&self, dialog_state: &DialogState) -> Option<Element<'_, Message>> {
        let spacing = theme::active().cosmic().spacing;

        let dialog = match &self {
            DialogPage::NewVaultFile(file_name) => widget::dialog()
                .title(fl!("new-vault-file"))
                .primary_action(
                    widget::button::suggested(fl!("create"))
                        .on_press(Message::DialogAction(DialogAction::DialogComplete)),
                )
                .secondary_action(
                    widget::button::standard(fl!("cancel"))
                        .on_press(Message::DialogAction(DialogAction::DialogCancel)),
                )
                .control(
                    widget::column::with_children(vec![
                        widget::text::body(fl!("file-name")).into(),
                        widget::text_input("", file_name.as_str())
                            .id(dialog_state.dialog_text_input.clone())
                            .on_input(move |name| {
                                Message::DialogAction(DialogAction::DialogUpdate(
                                    DialogPage::NewVaultFile(name),
                                ))
                            })
                            .on_submit(|_x| Message::DialogAction(DialogAction::DialogComplete))
                            .into(),
                    ])
                    .spacing(spacing.space_xxs),
                ),
            DialogPage::NewVaultFolder(folder_name) => widget::dialog()
                .title(fl!("new-folder"))
                .primary_action(
                    widget::button::suggested(fl!("create"))
                        .on_press(Message::DialogAction(DialogAction::DialogComplete)),
                )
                .secondary_action(
                    widget::button::standard(fl!("cancel"))
                        .on_press(Message::DialogAction(DialogAction::DialogCancel)),
                )
                .control(
                    widget::column::with_children(vec![
                        widget::text::body(fl!("folder-name")).into(),
                        widget::text_input("", folder_name.as_str())
                            .id(dialog_state.dialog_text_input.clone())
                            .on_input(move |name| {
                                Message::DialogAction(DialogAction::DialogUpdate(
                                    DialogPage::NewVaultFolder(name),
                                ))
                            })
                            .on_submit(|_x| Message::DialogAction(DialogAction::DialogComplete))
                            .into(),
                    ])
                    .spacing(spacing.space_xxs),
                ),
            DialogPage::DeleteNode(_entity) => widget::dialog()
                .title(fl!("delete-node"))
                .primary_action(
                    widget::button::suggested(fl!("delete"))
                        .on_press(Message::DialogAction(DialogAction::DialogComplete)),
                )
                .secondary_action(
                    widget::button::standard(fl!("cancel"))
                        .on_press(Message::DialogAction(DialogAction::DialogCancel)),
                )
                .control(
                    widget::column::with_children(vec![
                        widget::text::body(fl!("delete-confirmation")).into(),
                    ])
                    .spacing(spacing.space_xxs),
                ),
            DialogPage::RenameNode(node_path, new_name) => widget::dialog()
                .title(fl!("rename"))
                .primary_action(
                    widget::button::suggested(fl!("rename"))
                        .on_press(Message::DialogAction(DialogAction::DialogComplete)),
                )
                .secondary_action(
                    widget::button::standard(fl!("cancel"))
                        .on_press(Message::DialogAction(DialogAction::DialogCancel)),
                )
                .control(
                    widget::column::with_children(vec![
                        widget::text::body(fl!("name")).into(),
                        widget::text_input("", new_name.as_str())
                            .id(dialog_state.dialog_text_input.clone())
                            .on_input(move |name| {
                                Message::DialogAction(DialogAction::DialogUpdate(
                                    DialogPage::RenameNode(node_path.clone(), name),
                                ))
                            })
                            .on_submit(|_x| Message::DialogAction(DialogAction::DialogComplete))
                            .into(),
                    ])
                    .spacing(spacing.space_xxs),
                ),
            DialogPage::MoveNode(source_path, selected_target) => {
                let folder_list = widget::column::with_children(
                    dialog_state
                        .available_folders
                        .iter()
                        .map(|(path, name, indent)| {
                            let is_selected = selected_target.as_ref() == Some(path);
                            let indent_padding = (*indent as f32) * 32.0;

                            widget::button::custom(
                                widget::row::with_children(vec![
                                    widget::space::horizontal().width(indent_padding).into(),
                                    widget::icon::from_name("folder-symbolic").size(16).into(),
                                    widget::text::body(name.clone()).into(),
                                ])
                                .align_y(Alignment::Center)
                                .spacing(spacing.space_xxs),
                            )
                            .on_press(Message::DialogAction(DialogAction::DialogUpdate(
                                DialogPage::MoveNode(source_path.clone(), Some(path.clone())),
                            )))
                            .class(if is_selected {
                                theme::Button::Suggested
                            } else {
                                theme::Button::MenuItem
                            })
                            .width(cosmic::iced::Length::Fill)
                            .into()
                        })
                        .collect::<Vec<Element<Message>>>(),
                )
                .spacing(spacing.space_xxxs);

                widget::dialog()
                    .title(fl!("move-to"))
                    .primary_action(
                        widget::button::suggested(fl!("move")).on_press_maybe(
                            selected_target
                                .as_ref()
                                .map(|_| Message::DialogAction(DialogAction::DialogComplete)),
                        ),
                    )
                    .secondary_action(
                        widget::button::standard(fl!("cancel"))
                            .on_press(Message::DialogAction(DialogAction::DialogCancel)),
                    )
                    .control(
                        widget::scrollable(folder_list).height(cosmic::iced::Length::Fixed(300.0)),
                    )
            }
            DialogPage::ConfirmCloseFile(action) => widget::dialog()
                .title(fl!("save-changes-closing"))
                .primary_action(
                    widget::button::suggested(fl!("save-file"))
                        .on_press(Message::DialogAction(DialogAction::DialogComplete)),
                )
                .secondary_action(
                    widget::button::destructive(fl!("discard-changes")).on_press(
                        Message::DialogAction(DialogAction::DiscardChanges(action.clone())),
                    ),
                )
                .tertiary_action(
                    widget::button::text(fl!("cancel"))
                        .on_press(Message::DialogAction(DialogAction::DialogCancel)),
                )
                .control(
                    widget::column::with_children(vec![
                        widget::text::body(fl!("save-warning")).into(),
                    ])
                    .spacing(spacing.space_xxs),
                ),
            DialogPage::ExternalFileModified(path) => {
                let file_name = path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("this file");

                widget::dialog()
                    .title(fl!("file-changed-externally"))
                    .primary_action(
                        widget::button::suggested(fl!("reload"))
                            .on_press(Message::DialogAction(DialogAction::DialogComplete)),
                    )
                    .secondary_action(
                        widget::button::standard(fl!("keep-my-version"))
                            .on_press(Message::DialogAction(DialogAction::KeepMyFile)),
                    )
                    .control(
                        widget::column::with_children(vec![
                            widget::text::body(format!("\"{file_name}\" {}", fl!("modified-text")))
                                .into(),
                        ])
                        .spacing(spacing.space_xxs),
                    )
            }
            DialogPage::InsertTable { rows, cols } => {
                let current_rows = *rows;
                let current_cols = *cols;

                let max_grid_rows = 6;
                let max_grid_cols = 6;

                let mut grid_rows: Vec<Element<Message>> = Vec::with_capacity(max_grid_rows);

                for r in 1..=max_grid_rows {
                    let mut row_items: Vec<Element<Message>> = Vec::with_capacity(max_grid_cols);
                    for c in 1..=max_grid_cols {
                        let is_selected = r <= current_rows && c <= current_cols;
                        let btn = if is_selected {
                            widget::button::suggested("")
                        } else {
                            widget::button::standard("")
                        };
                        row_items.push(
                            btn.width(Length::Fixed(32.0))
                                .height(Length::Fixed(32.0))
                                .on_press(Message::DialogAction(DialogAction::DialogUpdate(
                                    DialogPage::InsertTable { rows: r, cols: c },
                                )))
                                .into(),
                        );
                    }
                    grid_rows.push(
                        widget::row::with_children(row_items)
                            .spacing(4)
                            .into(),
                    );
                }

                let grid_column = widget::column::with_children(grid_rows).spacing(4);

                let row_controls = widget::row![
                    widget::text::body("Rows:"),
                    widget::button::standard("-").on_press_maybe((current_rows > 1).then_some(
                        Message::DialogAction(DialogAction::DialogUpdate(
                            DialogPage::InsertTable { rows: current_rows - 1, cols: current_cols },
                        ))
                    )),
                    widget::text::body(format!("{current_rows}")),
                    widget::button::standard("+").on_press_maybe((current_rows < 50).then_some(
                        Message::DialogAction(DialogAction::DialogUpdate(
                            DialogPage::InsertTable { rows: current_rows + 1, cols: current_cols },
                        ))
                    )),
                    widget::space::horizontal().width(Length::Fixed(16.0)),
                    widget::text::body("Columns:"),
                    widget::button::standard("-").on_press_maybe((current_cols > 1).then_some(
                        Message::DialogAction(DialogAction::DialogUpdate(
                            DialogPage::InsertTable { rows: current_rows, cols: current_cols - 1 },
                        ))
                    )),
                    widget::text::body(format!("{current_cols}")),
                    widget::button::standard("+").on_press_maybe((current_cols < 20).then_some(
                        Message::DialogAction(DialogAction::DialogUpdate(
                            DialogPage::InsertTable { rows: current_rows, cols: current_cols + 1 },
                        ))
                    )),
                ]
                .align_y(Alignment::Center)
                .spacing(spacing.space_xxs);

                widget::dialog()
                    .title(fl!("insert-table"))
                    .primary_action(
                        widget::button::suggested(fl!("insert"))
                            .on_press(Message::DialogAction(DialogAction::DialogComplete)),
                    )
                    .secondary_action(
                        widget::button::standard(fl!("cancel"))
                            .on_press(Message::DialogAction(DialogAction::DialogCancel)),
                    )
                    .control(
                        widget::column::with_children(vec![
                            widget::text::title3(format!("{current_rows} × {current_cols} Table")).into(),
                            grid_column.into(),
                            row_controls.into(),
                        ])
                        .spacing(spacing.space_xs)
                        .align_x(Alignment::Center),
                    )
            }
        };

        Some(dialog.into())
    }
}

/// Represents an Action related to a Dialog
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DialogAction {
    /// File has been modified externally and user want's to keep it's version of the file
    KeepMyFile,
    /// Asks to open the [`DialogPage`] for creating a new vault file
    OpenNewVaultFileDialog,
    /// Asks to open the [`DialogPage`] for creating a new vault folder
    OpenNewVaultFolderDialog,
    /// Asks to open the [`DialogPage`] for deleting a vault folder/file
    OpenDeleteNodeDialog(PathBuf),
    /// Asks to open the [`DialogPage`] for renaming a vault folder/file
    OpenRenameNodeDialog(PathBuf),
    /// Asks to open the [`DialogPage`] for moving a node in the vault
    OpenMoveNodeDialog(PathBuf),
    /// Asks to open the [`DialogPage`] for asking confirmation before closing a file
    OpenConfirmCloseFileDialog(DiscardChangesAction),
    /// Asks to open the [`DialogPage`] for inserting a table
    OpenInsertTableDialog,
    /// Action after user confirms wants to discard changes on Save dialog
    DiscardChanges(DiscardChangesAction),
    /// Action after user confirms/ok's/accepts the action of a Dialog
    DialogComplete,
    /// Action after user cancels the action of a Dialog
    DialogCancel,
    /// Updates the value of the given [`DialogPage`]
    DialogUpdate(DialogPage),
}

impl DialogAction {
    /// Executes the [`DialogAction`]
    pub fn execute(
        self,
        dialog_pages: &mut VecDeque<DialogPage>,
        dialog_state: &DialogState,
    ) -> Task<cosmic::Action<Message>> {
        match self {
            DialogAction::KeepMyFile => {
                dialog_pages.pop_front();
                Task::done(cosmic::action::app(Message::SaveFile))
            }
            DialogAction::OpenNewVaultFileDialog => {
                dialog_pages.push_back(DialogPage::NewVaultFile(String::new()));
                widget::text_input::focus(dialog_state.dialog_text_input.clone())
            }
            DialogAction::OpenNewVaultFolderDialog => {
                dialog_pages.push_back(DialogPage::NewVaultFolder(String::new()));
                widget::text_input::focus(dialog_state.dialog_text_input.clone())
            }
            DialogAction::DialogComplete => {
                if let Some(dialog_page) = dialog_pages.pop_front() {
                    match dialog_page {
                        DialogPage::NewVaultFile(file_name) => {
                            if !file_name.is_empty() {
                                return Task::done(cosmic::action::app(Message::NewVaultFile(
                                    file_name,
                                )));
                            }
                        }
                        DialogPage::NewVaultFolder(folder_name) => {
                            if !folder_name.is_empty() {
                                return Task::done(cosmic::action::app(Message::NewVaultFolder(
                                    folder_name,
                                )));
                            }
                        }
                        DialogPage::DeleteNode(entity) => {
                            return Task::done(cosmic::action::app(Message::DeleteNode(entity)));
                        }
                        DialogPage::RenameNode(entity, new_name) => {
                            if !new_name.is_empty() {
                                return Task::done(cosmic::action::app(Message::RenameNode(
                                    entity, new_name,
                                )));
                            }
                        }
                        DialogPage::MoveNode(source_entity, Some(target_path)) => {
                            return Task::done(cosmic::action::app(Message::MoveNode(
                                source_entity,
                                target_path,
                            )));
                        }
                        DialogPage::MoveNode(_, None) => {}
                        DialogPage::ConfirmCloseFile(action) => match action {
                            DiscardChangesAction::CloseApp => {
                                return Task::done(cosmic::action::app(Message::SaveFile));
                            }
                            DiscardChangesAction::OpenFile(_file_path) => {
                                return Task::done(cosmic::action::app(Message::SaveFile));
                            }
                        },

                        DialogPage::ExternalFileModified(path) => {
                            return Task::done(cosmic::action::app(Message::DiscardChanges(
                                DiscardChangesAction::OpenFile(path.clone()),
                            )));
                        }
                        DialogPage::InsertTable { rows, cols } => {
                            return Task::done(cosmic::action::app(Message::ApplyFormatting(
                                crate::app::core::utils::markdown::SelectionAction::InsertTable {
                                    rows,
                                    cols,
                                },
                            )));
                        }
                    }
                }
                Task::none()
            }
            DialogAction::DiscardChanges(action) => {
                dialog_pages.pop_front();
                Task::done(cosmic::action::app(Message::DiscardChanges(action)))
            }
            DialogAction::DialogCancel => {
                dialog_pages.pop_front();
                Task::none()
            }
            DialogAction::DialogUpdate(dialog_page) => {
                dialog_pages[0] = dialog_page;
                Task::none()
            }
            DialogAction::OpenDeleteNodeDialog(entity) => {
                dialog_pages.push_back(DialogPage::DeleteNode(entity));
                Task::none()
            }
            DialogAction::OpenRenameNodeDialog(entity) => {
                dialog_pages.push_back(DialogPage::RenameNode(entity, String::new()));
                widget::text_input::focus(dialog_state.dialog_text_input.clone())
            }
            DialogAction::OpenMoveNodeDialog(entity) => {
                dialog_pages.push_back(DialogPage::MoveNode(entity, None));
                Task::none()
            }
            DialogAction::OpenConfirmCloseFileDialog(action) => {
                dialog_pages.push_back(DialogPage::ConfirmCloseFile(action));
                Task::none()
            }
            DialogAction::OpenInsertTableDialog => {
                dialog_pages.push_back(DialogPage::InsertTable { rows: 3, cols: 3 });
                Task::none()
            }
        }
    }
}

/// State of all the dialog widgets of the app
pub struct DialogState {
    /// Input inside of the Dialog Pages of the Application
    pub dialog_text_input: widget::Id,
    /// Available folders to move a node in
    pub available_folders: Vec<(PathBuf, String, u16)>,
}

impl Default for DialogState {
    fn default() -> Self {
        Self {
            dialog_text_input: widget::Id::unique(),
            available_folders: Vec::new(),
        }
    }
}
