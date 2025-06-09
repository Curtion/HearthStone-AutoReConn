use crossbeam_channel::Sender;
use gpui::{
    App, Application, Bounds, Context, SharedString, TitlebarOptions, Window, WindowBounds,
    WindowOptions, div, prelude::*, px, rgb, size,
};
use log::error;

use crate::config::Config;

pub enum GuiMessage {
    SaveConfig(Config),
}

struct Setting {
    hotkeys: SharedString,
    tx: Sender<GuiMessage>,
}

impl Render for Setting {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let mut button = div()
            .id("counter")
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
            .child(format!("Count"));
        let tx = self.tx.clone();
        button
            .interactivity()
            .on_click(cx.listener(move |_, _, _, _| {
                tx.send(GuiMessage::SaveConfig(Config {
                    reconnect_hotkey: "Alt+R".to_string(),
                }))
                .unwrap_or_else(|e| {
                    error!("Failed to send message: {}", e);
                });
            }));
        div()
            .flex()
            .flex_col()
            .items_center()
            .justify_center()
            .size_full()
            .bg(rgb(0xffffff))
            .child(button)
    }
}

pub fn app(tx: Sender<GuiMessage>, config: Config) {
    let reconnect_hotkey = config.reconnect_hotkey.clone();
    Application::new().run(|cx: &mut App| {
        let bounds = Bounds::centered(None, size(px(500.), px(500.0)), cx);
        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                titlebar: Some(TitlebarOptions {
                    title: Some("设置".into()),
                    ..Default::default()
                }),
                ..Default::default()
            },
            |_, cx| {
                cx.new(|_| Setting {
                    hotkeys: reconnect_hotkey.into(),
                    tx,
                })
            },
        )
        .unwrap();
        cx.activate(true);
    });
}
