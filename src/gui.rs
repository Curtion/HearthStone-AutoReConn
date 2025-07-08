use eframe::{App, Frame, egui};
use flume::{Receiver, Sender};
use log;

#[derive(Debug, Clone)]
pub enum GuiOutMessage {
    SaveHotKeys(String),
}

#[derive(Debug, Clone)]
pub enum GuiInMessage {
    Exit,
    Show,
    Hide,
}

pub struct EguiApp {
    hotkeys: String,
    current_pressed_keys: String,
    gui_out_tx: Sender<GuiOutMessage>,
    gui_in_rx: Receiver<GuiInMessage>,
    visible: bool,
}

impl EguiApp {
    pub fn new(
        gui_out_tx: Sender<GuiOutMessage>,
        gui_in_rx: Receiver<GuiInMessage>,
        hotkeys: String,
    ) -> Self {
        Self {
            hotkeys,
            current_pressed_keys: String::new(),
            gui_out_tx,
            gui_in_rx,
            visible: true,
        }
    }
}

impl App for EguiApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut Frame) {
        // In reactive mode, we need to check for messages and request a repaint if we get one
        // if !self.gui_in_rx.is_empty() {
        //     ctx.request_repaint();
        // }

        // if let Ok(msg) = self.gui_in_rx.try_recv() {
        //     match msg {
        //         GuiInMessage::Show => {
        //             self.visible = true;
        //             // When we show the window, we want to repaint it immediately
        //             ctx.request_repaint();
        //         }
        //         GuiInMessage::Hide => self.visible = false,
        //         GuiInMessage::Exit => {
        //             ctx.send_viewport_cmd(egui::ViewportCommand::Close);
        //             return;
        //         }
        //     }
        // }

        // // If the user presses the close button, hide the window instead of closing
        // if ctx.input(|i| i.viewport().close_requested()) {
        //     self.visible = false;
        //     // Prevent the app from closing
        //     ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
        // }

        // // Set window visibility based on the state
        // ctx.send_viewport_cmd(egui::ViewportCommand::Visible(self.visible));

        // if !self.visible {
        //     // If the window is hidden, we check for messages periodically.
        //     // This is cheaper than `ctx.request_repaint()` which would be a busy-loop.
        //     ctx.request_repaint_after(std::time::Duration::from_millis(100));
        //     return;
        // }

        // // Handle keyboard input for hotkey setting
        // let old_keys = self.current_pressed_keys.clone();
        // ctx.input(|i| {
        //     for event in &i.events {
        //         if let egui::Event::Key {
        //             key,
        //             pressed: true, // Only on key down
        //             modifiers,
        //             ..
        //         } = event
        //         {
        //             let mut key_parts = Vec::new();
        //             if modifiers.ctrl {
        //                 key_parts.push("Ctrl".to_string());
        //             }
        //             if modifiers.alt {
        //                 key_parts.push("Alt".to_string());
        //             }
        //             if modifiers.shift {
        //                 key_parts.push("Shift".to_string());
        //             }
        //             if modifiers.command {
        //                 key_parts.push("Win".to_string());
        //             }

        //             let main_key = format!("{:?}", key);
        //             // Avoid registering single modifier keys as hotkeys
        //             if !is_modifier(key) {
        //                 key_parts.push(main_key.to_uppercase());
        //             }

        //             self.current_pressed_keys = if key_parts.len() > 1 {
        //                 key_parts.join("+")
        //             } else if key_parts.len() == 1 && !is_modifier(key) {
        //                 key_parts.join("")
        //             } else {
        //                 // If only modifiers are pressed, don't update the display
        //                 self.current_pressed_keys.clone()
        //             };
        //         }
        //     }
        // });

        // // If the pressed keys have changed, request a repaint to show the new keys
        // if self.current_pressed_keys != old_keys {
        //     ctx.request_repaint();
        // }

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.heading("快捷键设置");
            });

            ui.add_space(10.0);

            egui::Grid::new("hotkey_grid")
                .num_columns(2)
                .spacing([40.0, 4.0])
                .striped(true)
                .show(ui, |ui| {
                    ui.label("当前快捷键:");
                    ui.label(&self.hotkeys);
                    ui.end_row();

                    ui.label("当前按下的键:");
                    ui.label(if self.current_pressed_keys.is_empty() {
                        "无"
                    } else {
                        &self.current_pressed_keys
                    });
                    ui.end_row();
                });

            ui.add_space(10.0);

            ui.vertical_centered(|ui| {
                if ui.button("保存快捷键").clicked() {
                    if !self.current_pressed_keys.is_empty() {
                        self.hotkeys = self.current_pressed_keys.clone();
                        self.gui_out_tx
                            .send(GuiOutMessage::SaveHotKeys(self.hotkeys.clone()))
                            .unwrap_or_else(|e| log::error!("无法发送消息: {}", e));
                        // Clear pressed keys after saving
                        self.current_pressed_keys.clear();
                        // Request a repaint to show that the keys have been cleared
                        ctx.request_repaint();
                    } else {
                        log::info!("没有新的按键组合被按下，未保存。");
                    }
                }
            });
        });
    }
}
