use enigo::{Direction, Enigo, Key, Keyboard, Settings};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct KeySelection {
    pub key: String,
    #[allow(dead_code)]
    pub code: String,
    #[allow(dead_code)]
    pub label: String,
}

pub struct KeyController {
    enigo: Enigo,
    held: Option<Key>,
}

impl KeyController {
    pub fn new() -> Result<Self, String> {
        let enigo = Enigo::new(&Settings::default())
            .map_err(|error| format!("Impossible d’initialiser le clavier : {error}"))?;
        Ok(Self { enigo, held: None })
    }

    pub fn hold(&mut self, selection: KeySelection) -> Result<(), String> {
        self.release()?;
        let key = map_key(&selection.key)?;
        self.enigo
            .key(key, Direction::Press)
            .map_err(|error| format!("Impossible de maintenir cette touche : {error}"))?;
        self.held = Some(key);
        Ok(())
    }

    pub fn release(&mut self) -> Result<(), String> {
        let Some(key) = self.held.take() else {
            return Ok(());
        };

        self.enigo
            .key(key, Direction::Release)
            .map_err(|error| format!("Impossible de relâcher la touche : {error}"))
    }
}

impl Drop for KeyController {
    fn drop(&mut self) {
        let _ = self.release();
    }
}

fn function_key(value: &str) -> Option<Key> {
    Some(match value {
        "F1" => Key::F1,
        "F2" => Key::F2,
        "F3" => Key::F3,
        "F4" => Key::F4,
        "F5" => Key::F5,
        "F6" => Key::F6,
        "F7" => Key::F7,
        "F8" => Key::F8,
        "F9" => Key::F9,
        "F10" => Key::F10,
        "F11" => Key::F11,
        "F12" => Key::F12,
        "F13" => Key::F13,
        "F14" => Key::F14,
        "F15" => Key::F15,
        "F16" => Key::F16,
        "F17" => Key::F17,
        "F18" => Key::F18,
        "F19" => Key::F19,
        "F20" => Key::F20,
        "F21" => Key::F21,
        "F22" => Key::F22,
        "F23" => Key::F23,
        "F24" => Key::F24,
        _ => return None,
    })
}

pub fn map_key(value: &str) -> Result<Key, String> {
    let named = match value {
        " " => Some(Key::Space),
        "Alt" | "AltGraph" => Some(Key::Alt),
        "Backspace" => Some(Key::Backspace),
        "CapsLock" => Some(Key::CapsLock),
        "Control" => Some(Key::Control),
        "Delete" => Some(Key::Delete),
        "ArrowDown" => Some(Key::DownArrow),
        "End" => Some(Key::End),
        "Enter" => Some(Key::Return),
        "Escape" => Some(Key::Escape),
        "Home" => Some(Key::Home),
        "Insert" => Some(Key::Insert),
        "ArrowLeft" => Some(Key::LeftArrow),
        "Meta" => Some(Key::Meta),
        "PageDown" => Some(Key::PageDown),
        "PageUp" => Some(Key::PageUp),
        "Pause" => Some(Key::Pause),
        "PrintScreen" => Some(Key::PrintScr),
        "ArrowRight" => Some(Key::RightArrow),
        "ScrollLock" => Some(Key::ScrollLock),
        "Shift" => Some(Key::Shift),
        "Tab" => Some(Key::Tab),
        "ArrowUp" => Some(Key::UpArrow),
        _ => function_key(value),
    };

    if let Some(key) = named {
        return Ok(key);
    }

    let mut chars = value.chars();
    match (chars.next(), chars.next()) {
        (Some(character), None) => Ok(Key::Unicode(character.to_lowercase().next().unwrap_or(character))),
        _ => Err(format!("La touche « {value} » n’est pas encore prise en charge")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_character_keys() {
        assert_eq!(map_key("A").unwrap(), Key::Unicode('a'));
        assert_eq!(map_key("é").unwrap(), Key::Unicode('é'));
    }

    #[test]
    fn maps_named_keys() {
        assert_eq!(map_key(" ").unwrap(), Key::Space);
        assert_eq!(map_key("ArrowUp").unwrap(), Key::UpArrow);
        assert_eq!(map_key("F12").unwrap(), Key::F12);
    }

    #[test]
    fn rejects_unknown_named_keys() {
        assert!(map_key("Dead").is_err());
    }
}

