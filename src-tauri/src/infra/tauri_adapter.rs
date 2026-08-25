use crate::domain::hotkey::{Combo, Mods, PhysicalKey};
use std::str::FromStr;
use tauri_plugin_global_shortcut::{Code, Modifiers, Shortcut};

pub fn combo_to_shortcut(combo: &Combo) -> Option<Shortcut> {
    let mut modifiers = Modifiers::default();
    if combo.mods.ctrl {
        modifiers |= Modifiers::CONTROL;
    }
    if combo.mods.alt {
        modifiers |= Modifiers::ALT;
    }
    if combo.mods.shift {
        modifiers |= Modifiers::SHIFT;
    }
    if combo.mods.logo {
        modifiers |= Modifiers::SUPER;
    }

    let key = Code::from_str(&combo.key.0).ok()?;

    Some(Shortcut::new(Some(modifiers), key))
}

pub fn shortcut_to_combo(shortcut: &Shortcut) -> Option<Combo> {
    let combo = Combo {
        mods: Mods {
            shift: shortcut.mods.shift(),
            logo: shortcut.mods.meta(),
            alt: shortcut.mods.alt(),
            ctrl: shortcut.mods.ctrl(),
        },
        key: PhysicalKey(shortcut.key.to_string()),
    };

    Some(combo)
}
