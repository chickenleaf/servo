/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use egui_file_dialog::{DialogState, FileDialog as EguiFileDialog};
use log::warn;
use servo::ipc_channel::ipc::IpcSender;

#[derive(Debug)]
pub struct FileDialog {
    pub dialog: EguiFileDialog,
    pub multiple: bool,
    pub response_sender: Option<IpcSender<Option<Vec<String>>>>,
}

#[derive(Debug)]
pub enum Dialog {
    File(FileDialog),
}

impl Dialog {
    pub fn update(&mut self, ctx: &egui::Context) -> bool {
        match self {
            Dialog::File(dialog) => {
                if dialog.dialog.state() == DialogState::Closed {
                    if dialog.multiple {
                        dialog.dialog.pick_multiple();
                    } else {
                        dialog.dialog.pick_file();
                    }
                }

                let state = dialog.dialog.update(ctx).state();

                match state {
                    DialogState::Open => true,
                    DialogState::Selected(path) => {
                        if let Some(sender) = dialog.response_sender.take() {
                            let result = if path.exists() {
                                Some(vec![path.to_string_lossy().into()])
                            } else {
                                None
                            };
                            if let Err(e) = sender.send(result) {
                                warn!("Failed to send file selection response: {}", e);
                            }
                        }
                        false
                    },
                    DialogState::SelectedMultiple(paths) => {
                        if let Some(sender) = dialog.response_sender.take() {
                            let result = if paths.iter().all(|path| path.exists()) {
                                Some(
                                    paths
                                        .iter()
                                        .map(|path| path.to_string_lossy().into())
                                        .collect(),
                                )
                            } else {
                                None
                            };

                            if let Err(e) = sender.send(result) {
                                warn!("Failed to send file selection response: {}", e);
                            }
                        }
                        false
                    },
                    DialogState::Cancelled => {
                        if let Some(sender) = dialog.response_sender.as_ref() {
                            if let Err(e) = sender.send(None) {
                                warn!("Failed to send cancellation response: {}", e);
                            }
                        }
                        false
                    },
                    DialogState::Closed => false,
                }
            },
        }
    }
}
