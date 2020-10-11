#![windows_subsystem = "windows"]
mod program;

use hotkey;
use std::thread;
use systray;
use webbrowser;

fn main() -> Result<(), systray::Error> {
    if program::is_fw_rule() {
        // 首先恢复网络
        program::disable();
    }
    thread::spawn(|| {
        let mut hk = hotkey::Listener::new();
        hk.register_hotkey(
            hotkey::modifiers::SHIFT | hotkey::modifiers::ALT,
            'R' as u32,
            || program::start(),
        )
        .unwrap();
        hk.listen();
    });
    let mut app = match systray::Application::new() {
        Ok(w) => w,
        Err(_) => panic!("程序运行错误！"),
    };
    app.set_icon_from_file("hsarec.ico")?;

    app.add_menu_item("开始拔线(Shift+Alt+R)", move |_| {
        program::start();
        Ok::<_, systray::Error>(())
    })?;
    app.add_menu_separator()?;
    app.add_menu_item("关于我", |_| {
        match webbrowser::open("https://blog.3gxk.net/about.html") {
            Err(e) => println!("{:?}", e),
            _ => (),
        }
        Ok::<_, systray::Error>(())
    })?;
    app.add_menu_item("退出程序", |window| {
        window.quit();
        Ok::<_, systray::Error>(())
    })?;
    app.wait_for_message()?;
    Ok(())
}
