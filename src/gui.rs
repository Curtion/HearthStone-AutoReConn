use anyhow::Result;
use crossbeam_channel::Sender;
use gpui::{
    App, Application, Bounds, Context, SharedString, TitlebarOptions, Window, WindowBounds,
    WindowOptions, div, prelude::*, px, rgb, size,
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

        // 主容器
        let main_container = div()
            .flex()
            .flex_col()
            .items_center()
            .justify_center()
            .size_full()
            .bg(rgb(0xffffff)) // shadcn/ui 通常使用白色或非常浅的灰色背景
            .p_8();

        // 卡片式容器的通用样式
        let card_style = |children| {
            div()
                .w(px(350.)) // 固定宽度，或根据内容调整
                .p_6()
                .mb_6()
                .bg(rgb(0xf9fafb)) // 非常浅的灰色背景，类似于 shadcn/ui 卡片
                .border(px(1.))
                .border_color(rgb(0xe5e7eb)) // 浅灰色边框
                .rounded(px(12.)) // 更大的圆角
                .shadow_md() // 细微的阴影
                .children(children)
        };

        // 当前快捷键显示区域
        let current_hotkey_display = card_style(vec![
            div()
                .text_sm()
                .font_weight(gpui::FontWeight::MEDIUM)
                .text_color(rgb(0x6b7280)) // 稍暗的灰色文本
                .mb_2()
                .child("当前快捷键")
                .into_any_element(),
            div()
                .text_2xl() // 更大的字体
                .font_weight(gpui::FontWeight::SEMIBOLD) // 半粗体
                .text_color(rgb(0x1f2937)) // 深灰色文本
                .child(self.hotkeys.clone())
                .into_any_element(),
        ]);

        // 实时按键显示区域
        let pressed_keys_display = card_style(vec![
            div()
                .text_sm()
                .font_weight(gpui::FontWeight::MEDIUM)
                .text_color(rgb(0x6b7280))
                .mb_2()
                .child("当前按下的键")
                .into_any_element(),
            div()
                .text_2xl()
                .font_weight(gpui::FontWeight::SEMIBOLD)
                .text_color(rgb(0x1f2937))
                .child(if self.current_pressed_keys.is_empty() {
                    "无".to_string()
                } else {
                    self.current_pressed_keys.to_string()
                })
                .into_any_element(),
        ]);

        // 保存按钮
        let mut button = div()
            .id("save-button")
            .w(px(350.)) // 与卡片同宽
            .h(px(48.)) // 合适的按钮高度
            .flex()
            .justify_center()
            .items_center()
            .bg(rgb(0x2563eb)) // shadcn/ui 风格的蓝色主色调
            .text_color(rgb(0xffffff)) // 白色文本
            .font_weight(gpui::FontWeight::MEDIUM)
            .rounded(px(8.)) // 圆角
            .cursor_pointer()
            .child("保存快捷键");
        // shadcn/ui 风格的按钮通常有 hover 和 active 状态，这里用 gpui 模拟
        // .hover(|s| s.bg(rgb(0x1d4ed8))) // hover 时颜色变深
        // .active(|s| s.bg(rgb(0x1e40af))); // active 时颜色更深

        button
            .interactivity()
            .on_click(cx.listener(move |this, _, _, _| {
                let new_hotkey = if !this.current_pressed_keys.is_empty() {
                    this.current_pressed_keys.to_string()
                } else {
                    // 如果没有按键，可以考虑不清空或使用默认值，或者提示用户
                    this.hotkeys.to_string() // 保持不变或设置为一个默认值
                    // "Alt+E".to_string() // 或者恢复默认
                };
                // 只有在 current_pressed_keys 不为空时才更新和发送消息
                if !this.current_pressed_keys.is_empty() {
                    this.hotkeys = SharedString::from(new_hotkey.clone());
                    tx.send(GuiMessage::SaveHotKeys(new_hotkey))
                        .unwrap_or_else(|e| {
                            error!("无法发送消息: {}", e);
                        });
                } else {
                    // 可选：提示用户需要先按下按键组合
                    log::info!("没有新的按键组合被按下，未保存。");
                }
            }));

        main_container
            .child(current_hotkey_display)
            .child(pressed_keys_display)
            .child(button)
    }
}

pub fn app(tx: Sender<GuiMessage>, reconnect_hotkey: &str) -> Result<()> {
    let reconnect_hotkey = reconnect_hotkey.to_string();

    Application::new().run(|cx: &mut App| {
        let bounds = Bounds::centered(None, size(px(600.), px(600.0)), cx);
        let window = cx
            .open_window(
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
        let view = window.update(cx, |_, _, cx| cx.entity()).unwrap();
        cx.observe_keystrokes(move |event, _window, cx| {
            let keystroke: &gpui::Keystroke = &event.keystroke;
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
            view.update(cx, |view, cx| {
                view.current_pressed_keys = SharedString::from(pressed_keys.clone());
                cx.notify();
            })
        })
        .detach();

        cx.on_window_closed(move |app| {
            if app.windows().is_empty() {
                log::info!("所有窗口已关闭，退出应用程序。");
                app.quit();
            }
        })
        .detach();

        cx.activate(true);
    });
    Ok(())
}
