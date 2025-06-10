use anyhow::Result;
use crossbeam_channel::Sender;
use gpui::{
    App, Application, Bounds, Context, KeyDownEvent, SharedString, TitlebarOptions, Window,
    WindowBounds, WindowOptions, div, prelude::*, px, rgb, size,
};
use log::error;

pub enum GuiMessage {
    SaveHotKeys(String),
}

struct Setting {
    hotkeys: SharedString,
    current_pressed_keys: SharedString,
    tx: Sender<GuiMessage>,
}

impl Render for Setting {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let tx = self.tx.clone();

        // 当前快捷键显示区域
        let current_hotkey_display = div()
            .flex()
            .flex_col()
            .items_center()
            .justify_center()
            .p_4()
            .mb_4()
            .bg(rgb(0xf5f5f5))
            .border(px(1.0))
            .border_color(rgb(0xcccccc))
            .rounded(px(8.0))
            .child(
                div()
                    .text_sm()
                    .text_color(rgb(0x666666))
                    .child("当前快捷键:")
            )
            .child(
                div()
                    .text_lg()
                    .font_weight(gpui::FontWeight::BOLD)
                    .text_color(rgb(0x333333))
                    .child(self.hotkeys.clone())
            );

        // 实时按键显示区域
        let pressed_keys_display = div()
            .flex()
            .flex_col()
            .items_center()
            .justify_center()
            .p_4()
            .mb_4()
            .bg(rgb(0xe3f2fd))
            .border(px(1.0))
            .border_color(rgb(0x2196f3))
            .rounded(px(8.0))
            .child(
                div()
                    .text_sm()
                    .text_color(rgb(0x1976d2))
                    .child("当前按下的键:")
            )
            .child(
                div()
                    .text_lg()
                    .font_weight(gpui::FontWeight::BOLD)
                    .text_color(rgb(0x0d47a1))
                    .child(if self.current_pressed_keys.is_empty() {
                        "无".to_string()
                    } else {
                        self.current_pressed_keys.to_string()
                    })
            );

        // 保存按钮
        let mut button = div()
            .id("save-button")
            .flex()
            .bg(rgb(0x2e7d32))
            .justify_center()
            .items_center()
            .shadow_lg()
            .border(px(2.0))
            .border_color(rgb(0x0000ff))
            .text_xl()
            .text_color(rgb(0xffffff))
            .cursor_pointer()
            .p_4()
            .rounded(px(8.0))
            .child("保存当前快捷键设置");

        button
            .interactivity()
            .on_click(cx.listener(move |this, _, _, _| {
                // 如果有按下的键，则保存为新的快捷键
                let new_hotkey = if !this.current_pressed_keys.is_empty() {
                    this.current_pressed_keys.to_string()
                } else {
                    "Alt+E".to_string()
                };
                this.hotkeys = SharedString::from(new_hotkey.clone());
                tx.send(GuiMessage::SaveHotKeys(new_hotkey))
                    .unwrap_or_else(|e| {
                        error!("无法发送消息: {}", e);
                    });
            }));

        // 主容器，添加键盘事件监听
        div()
            .flex()
            .flex_col()
            .items_center()
            .justify_center()
            .size_full()
            .bg(rgb(0xffffff))
            .p_8()
            .on_key_down(cx.listener(|this, event: &KeyDownEvent, _window, _cx| {
                let keystroke = &event.keystroke;

                // 构建按键字符串
                let mut key_parts = Vec::new();

                // 添加修饰键
                if keystroke.modifiers.control {
                    key_parts.push("Ctrl".to_string());
                }
                if keystroke.modifiers.alt {
                    key_parts.push("Alt".to_string());
                }
                if keystroke.modifiers.shift {
                    key_parts.push("Shift".to_string());
                }
                if keystroke.modifiers.platform {
                    key_parts.push("Win".to_string());
                }

                // 添加主键
                let main_key = match keystroke.key.as_str() {
                    "escape" => "Esc",
                    "enter" => "Enter",
                    "space" => "Space",
                    "tab" => "Tab",
                    key => key,
                };
                key_parts.push(main_key.to_uppercase());

                let pressed_keys = if key_parts.len() > 1 {
                    key_parts.join("+")
                } else {
                    key_parts.join("")
                };

                // 更新当前按下的键
                this.current_pressed_keys = SharedString::from(pressed_keys);
            }))
            .child(current_hotkey_display)
            .child(pressed_keys_display)
            .child(button)
    }
}

pub fn app(tx: Sender<GuiMessage>, reconnect_hotkey: &str) -> Result<()> {
    let reconnect_hotkey = reconnect_hotkey.to_string();

    Application::new().run(|cx: &mut App| {
        let bounds = Bounds::centered(None, size(px(600.), px(600.0)), cx);
        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                titlebar: Some(TitlebarOptions {
                    title: Some("快捷键设置".into()),
                    ..Default::default()
                }),
                ..Default::default()
            },
            |_, cx| {
                cx.new(|_| Setting {
                    hotkeys: reconnect_hotkey.into(),
                    current_pressed_keys: SharedString::from(""),
                    tx,
                })
            },
        )
        .unwrap();
        cx.activate(true);
    });
    Ok(())
}
