use anyhow::Result;
use crossbeam_channel::Sender;
use gpui::{
    App, Application, Bounds, Context, SharedString, TitlebarOptions, Window, WindowBounds,
    WindowOptions, div, prelude::*, px, rgb, size,
};
use log::{error, info};

pub enum GuiMessage {
    SaveHotKeys(String),
}

struct Setting {
    hotkeys: SharedString,
    tx: Sender<GuiMessage>,
}

impl Render for Setting {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let tx = self.tx.clone();
        let mut button = div()
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
            .child(format!("{} 保存配置", self.hotkeys));
        button
            .interactivity()
            .on_click(cx.listener(move |_, _, _, _| {
                info!("保存快捷键");
                tx.send(GuiMessage::SaveHotKeys("Alt+R".to_string()))
                    .unwrap_or_else(|e| {
                        error!("无法发送消息: {}", e);
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

pub fn app(tx: Sender<GuiMessage>, reconnect_hotkey: &str) -> Result<()> {
    let reconnect_hotkey = reconnect_hotkey.to_string();
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
    Ok(())
}
