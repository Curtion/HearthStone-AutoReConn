use inputbot::KeybdKey;
use log::{info, warn};

// Helper functions for hotkey parsing
pub fn key_string_to_keybdkey(key_str: &str) -> Option<KeybdKey> {
    match key_str.to_uppercase().as_str() {
        "F1" => Some(KeybdKey::F1Key),
        "F2" => Some(KeybdKey::F2Key),
        "F3" => Some(KeybdKey::F3Key),
        "F4" => Some(KeybdKey::F4Key),
        "F5" => Some(KeybdKey::F5Key),
        "F6" => Some(KeybdKey::F6Key),
        "F7" => Some(KeybdKey::F7Key),
        "F8" => Some(KeybdKey::F8Key),
        "F9" => Some(KeybdKey::F9Key),
        "F10" => Some(KeybdKey::F10Key),
        "F11" => Some(KeybdKey::F11Key),
        "F12" => Some(KeybdKey::F12Key),
        "CTRL" => Some(KeybdKey::LControlKey), // Default to Left Control
        "LCTRL" | "LCONTROL" => Some(KeybdKey::LControlKey),
        "RCTRL" | "RCONTROL" => Some(KeybdKey::RControlKey),
        "SHIFT" => Some(KeybdKey::LShiftKey), // Default to Left Shift
        "LSHIFT" => Some(KeybdKey::LShiftKey),
        "RSHIFT" => Some(KeybdKey::RShiftKey),
        "ALT" => Some(KeybdKey::LAltKey), // Default to Left Alt
        "LALT" => Some(KeybdKey::LAltKey),
        "RALT" => Some(KeybdKey::RAltKey),
        "A" => Some(KeybdKey::AKey),
        "B" => Some(KeybdKey::BKey),
        "C" => Some(KeybdKey::CKey),
        "D" => Some(KeybdKey::DKey),
        "E" => Some(KeybdKey::EKey),
        "F" => Some(KeybdKey::FKey),
        "G" => Some(KeybdKey::GKey),
        "H" => Some(KeybdKey::HKey),
        "I" => Some(KeybdKey::IKey),
        "J" => Some(KeybdKey::JKey),
        "K" => Some(KeybdKey::KKey),
        "L" => Some(KeybdKey::LKey),
        "M" => Some(KeybdKey::MKey),
        "N" => Some(KeybdKey::NKey),
        "O" => Some(KeybdKey::OKey),
        "P" => Some(KeybdKey::PKey),
        "Q" => Some(KeybdKey::QKey),
        "R" => Some(KeybdKey::RKey),
        "S" => Some(KeybdKey::SKey),
        "T" => Some(KeybdKey::TKey),
        "U" => Some(KeybdKey::UKey),
        "V" => Some(KeybdKey::VKey),
        "W" => Some(KeybdKey::WKey),
        "X" => Some(KeybdKey::XKey),
        "Y" => Some(KeybdKey::YKey),
        "Z" => Some(KeybdKey::ZKey),
        "0" | "NUM0" => Some(KeybdKey::Numrow0Key),
        "1" | "NUM1" => Some(KeybdKey::Numrow1Key),
        "2" | "NUM2" => Some(KeybdKey::Numrow2Key),
        "3" | "NUM3" => Some(KeybdKey::Numrow3Key),
        "4" | "NUM4" => Some(KeybdKey::Numrow4Key),
        "5" | "NUM5" => Some(KeybdKey::Numrow5Key),
        "6" | "NUM6" => Some(KeybdKey::Numrow6Key),
        "7" | "NUM7" => Some(KeybdKey::Numrow7Key),
        "8" | "NUM8" => Some(KeybdKey::Numrow8Key),
        "9" | "NUM9" => Some(KeybdKey::Numrow9Key),
        "NUMPAD0" => Some(KeybdKey::Numpad0Key),
        "NUMPAD1" => Some(KeybdKey::Numpad1Key),
        "NUMPAD2" => Some(KeybdKey::Numpad2Key),
        "NUMPAD3" => Some(KeybdKey::Numpad3Key),
        "NUMPAD4" => Some(KeybdKey::Numpad4Key),
        "NUMPAD5" => Some(KeybdKey::Numpad5Key),
        "NUMPAD6" => Some(KeybdKey::Numpad6Key),
        "NUMPAD7" => Some(KeybdKey::Numpad7Key),
        "NUMPAD8" => Some(KeybdKey::Numpad8Key),
        "NUMPAD9" => Some(KeybdKey::Numpad9Key),
        "ESC" | "ESCAPE" => Some(KeybdKey::EscapeKey),
        "SPACE" => Some(KeybdKey::SpaceKey),
        "ENTER" => Some(KeybdKey::EnterKey),
        "TAB" => Some(KeybdKey::TabKey),
        _ => {
            warn!("警告: 未映射的键字符串: {}", key_str);
            None
        }
    }
}

pub fn parse_hotkey_config(hotkey_str: &str) -> (Option<KeybdKey>, Vec<KeybdKey>) {
    let parts: Vec<String> = hotkey_str
        .split('+')
        .map(|s| s.trim().to_uppercase())
        .collect();
    if parts.is_empty() {
        warn!("警告: 热键字符串为空。");
        return (None, Vec::new());
    }

    // The last part is considered the main key, others are modifiers
    let main_key_str = parts.last().unwrap();
    let main_key = key_string_to_keybdkey(main_key_str);
    if main_key.is_none() {
        warn!(
            "警告: 无法解析主键: '{}' 从配置 '{}'",
            main_key_str, hotkey_str
        );
    }

    let mut modifier_keys = Vec::new();
    if parts.len() > 1 {
        for i in 0..parts.len() - 1 {
            match key_string_to_keybdkey(&parts[i]) {
                Some(key) => {
                    modifier_keys.push(key);
                }
                None => {}
            }
        }
    }
    (main_key, modifier_keys)
}

pub fn register_hotkey(main_key: KeybdKey, modifier_keys: Vec<KeybdKey>) {
    let modifier_keys_clone = modifier_keys.clone();
    main_key.bind(move || {
        let mut all_modifiers_pressed = true;
        if !modifier_keys_clone.is_empty() {
            for modifier in &modifier_keys_clone {
                if !modifier.is_pressed() {
                    all_modifiers_pressed = false;
                    break;
                }
            }
        }
        if all_modifiers_pressed {
            info!("执行重连操作...");
        }
    });
}
