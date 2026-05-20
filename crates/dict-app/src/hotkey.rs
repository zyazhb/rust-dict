//! System-wide shortcut to open the compact float search panel.

use std::str::FromStr;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::Once;

use egui::{Context, Event, Key, Modifiers};
use global_hotkey::{
    hotkey::{Code, HotKey, Modifiers as GModifiers},
    GlobalHotKeyEvent, GlobalHotKeyManager, HotKeyState,
};

pub const DEFAULT_COMPACT_HOTKEY: &str = "super+control+KeyD";

static COMPACT_HOTKEY_ID: AtomicU32 = AtomicU32::new(0);
static COMPACT_HOTKEY_PENDING: AtomicBool = AtomicBool::new(false);
static WAKEUP_HANDLER: Once = Once::new();

/// Route OS hotkey events into egui's winit loop so they work while the window is unfocused.
pub fn install_wakeup_handler(ctx: Context) {
    WAKEUP_HANDLER.call_once(|| {
        GlobalHotKeyEvent::set_event_handler(Some(move |event: GlobalHotKeyEvent| {
            if event.state != HotKeyState::Pressed {
                return;
            }
            if event.id != COMPACT_HOTKEY_ID.load(Ordering::SeqCst) {
                return;
            }
            COMPACT_HOTKEY_PENDING.store(true, Ordering::SeqCst);
            ctx.request_repaint();
        }));
    });
}

pub fn set_compact_hotkey_id(id: u32) {
    COMPACT_HOTKEY_ID.store(id, Ordering::SeqCst);
}

pub fn effective_hotkey_str(stored: &str) -> &str {
    if stored.trim().is_empty() {
        DEFAULT_COMPACT_HOTKEY
    } else {
        stored.trim()
    }
}

pub fn parse_hotkey(hotkey_str: &str) -> Result<HotKey, global_hotkey::hotkey::HotKeyParseError> {
    HotKey::from_str(effective_hotkey_str(hotkey_str))
}

pub fn format_label(hotkey_str: &str) -> String {
    let Ok(hk) = parse_hotkey(hotkey_str) else {
        return hotkey_str.to_string();
    };
    let mut label = String::new();
    #[cfg(target_os = "macos")]
    {
        if hk.mods.contains(GModifiers::SUPER) {
            label.push('⌘');
        }
        if hk.mods.contains(GModifiers::CONTROL) {
            label.push('⌃');
        }
        if hk.mods.contains(GModifiers::ALT) {
            label.push('⌥');
        }
        if hk.mods.contains(GModifiers::SHIFT) {
            label.push('⇧');
        }
    }
    #[cfg(not(target_os = "macos"))]
    {
        if hk.mods.contains(GModifiers::SUPER) {
            label.push_str("Win+");
        }
        if hk.mods.contains(GModifiers::CONTROL) {
            label.push_str("Ctrl+");
        }
        if hk.mods.contains(GModifiers::ALT) {
            label.push_str("Alt+");
        }
        if hk.mods.contains(GModifiers::SHIFT) {
            label.push_str("Shift+");
        }
    }
    label.push_str(&code_display_name(hk.key));
    label
}

fn code_display_name(code: Code) -> String {
    let raw = code.to_string();
    raw.strip_prefix("Key")
        .map(str::to_string)
        .unwrap_or(raw)
}

pub struct GlobalHotkeys {
    manager: GlobalHotKeyManager,
    compact_float: HotKey,
}

impl GlobalHotkeys {
    pub fn register(hotkey_str: &str) -> Result<Self, global_hotkey::Error> {
        let manager = GlobalHotKeyManager::new()?;
        let compact_float = parse_hotkey(hotkey_str)
            .map_err(|e| global_hotkey::Error::HotKeyParseError(e.to_string()))?;
        manager.register(compact_float)?;
        set_compact_hotkey_id(compact_float.id());
        Ok(Self {
            manager,
            compact_float,
        })
    }

    pub fn set_hotkey(&mut self, hotkey_str: &str) -> Result<(), global_hotkey::Error> {
        let _ = self.manager.unregister(self.compact_float);
        let compact_float = parse_hotkey(hotkey_str)
            .map_err(|e| global_hotkey::Error::HotKeyParseError(e.to_string()))?;
        self.manager.register(compact_float)?;
        self.compact_float = compact_float;
        set_compact_hotkey_id(self.compact_float.id());
        Ok(())
    }

    pub fn compact_float_id(&self) -> u32 {
        self.compact_float.id()
    }

    pub fn sync_wakeup_id(&self) {
        set_compact_hotkey_id(self.compact_float_id());
    }

    /// Returns true when the user pressed the compact-float hotkey (including while unfocused).
    pub fn poll_compact_float(&self) -> bool {
        COMPACT_HOTKEY_PENDING.swap(false, Ordering::SeqCst)
    }
}

/// While capturing: `None` = no key yet; `Some(None)` = cancelled (Escape); `Some(Some(s))` = new shortcut.
pub fn capture_from_input(ctx: &Context) -> Option<Option<String>> {
    ctx.input(|input| {
        for event in &input.events {
            let Event::Key {
                key,
                pressed: true,
                modifiers,
                ..
            } = event
            else {
                continue;
            };
            if *key == Key::Escape {
                return Some(None);
            }
            if let Some(s) = key_event_to_hotkey_string(*key, *modifiers) {
                return Some(Some(s));
            }
        }
        None
    })
}

fn key_event_to_hotkey_string(key: Key, mods: Modifiers) -> Option<String> {
    let key_token = egui_key_to_code_name(key)?;
    let mod_tokens = modifiers_to_tokens(mods);
    if mod_tokens.is_empty() {
        return None;
    }
    let mut parts = mod_tokens;
    parts.push(key_token);
    Some(parts.join("+"))
}

fn modifiers_to_tokens(mods: Modifiers) -> Vec<&'static str> {
    let mut v = Vec::new();
    if mods.shift {
        v.push("shift");
    }
    if mods.alt {
        v.push("alt");
    }
    #[cfg(target_os = "macos")]
    {
        if mods.mac_cmd {
            v.push("super");
        }
        if mods.ctrl {
            v.push("control");
        }
    }
    #[cfg(not(target_os = "macos"))]
    {
        if mods.ctrl {
            v.push("control");
        }
    }
    v
}

fn egui_key_to_code_name(key: Key) -> Option<&'static str> {
    Some(match key {
        Key::A => "KeyA",
        Key::B => "KeyB",
        Key::C => "KeyC",
        Key::D => "KeyD",
        Key::E => "KeyE",
        Key::F => "KeyF",
        Key::G => "KeyG",
        Key::H => "KeyH",
        Key::I => "KeyI",
        Key::J => "KeyJ",
        Key::K => "KeyK",
        Key::L => "KeyL",
        Key::M => "KeyM",
        Key::N => "KeyN",
        Key::O => "KeyO",
        Key::P => "KeyP",
        Key::Q => "KeyQ",
        Key::R => "KeyR",
        Key::S => "KeyS",
        Key::T => "KeyT",
        Key::U => "KeyU",
        Key::V => "KeyV",
        Key::W => "KeyW",
        Key::X => "KeyX",
        Key::Y => "KeyY",
        Key::Z => "KeyZ",
        Key::Num0 => "Digit0",
        Key::Num1 => "Digit1",
        Key::Num2 => "Digit2",
        Key::Num3 => "Digit3",
        Key::Num4 => "Digit4",
        Key::Num5 => "Digit5",
        Key::Num6 => "Digit6",
        Key::Num7 => "Digit7",
        Key::Num8 => "Digit8",
        Key::Num9 => "Digit9",
        Key::F1 => "F1",
        Key::F2 => "F2",
        Key::F3 => "F3",
        Key::F4 => "F4",
        Key::F5 => "F5",
        Key::F6 => "F6",
        Key::F7 => "F7",
        Key::F8 => "F8",
        Key::F9 => "F9",
        Key::F10 => "F10",
        Key::F11 => "F11",
        Key::F12 => "F12",
        Key::Space => "Space",
        Key::Enter => "Enter",
        Key::Tab => "Tab",
        Key::Backspace => "Backspace",
        Key::Delete => "Delete",
        Key::Insert => "Insert",
        Key::Home => "Home",
        Key::End => "End",
        Key::PageUp => "PageUp",
        Key::PageDown => "PageDown",
        Key::ArrowUp => "ArrowUp",
        Key::ArrowDown => "ArrowDown",
        Key::ArrowLeft => "ArrowLeft",
        Key::ArrowRight => "ArrowRight",
        Key::Backtick => "Backquote",
        Key::Minus => "Minus",
        Key::Equals => "Equal",
        Key::OpenBracket => "BracketLeft",
        Key::CloseBracket => "BracketRight",
        Key::Backslash => "Backslash",
        Key::Semicolon => "Semicolon",
        Key::Quote => "Quote",
        Key::Comma => "Comma",
        Key::Period => "Period",
        Key::Slash => "Slash",
        _ => return None,
    })
}
