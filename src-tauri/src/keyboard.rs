use enigo::{Direction, Enigo, Key, Keyboard, Settings};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct KeySelection {
    pub key: String,
    pub code: String,
    #[allow(dead_code)]
    pub label: String,
}

pub struct KeyController {
    enigo: Enigo,
    held: Vec<Key>,
}

fn map_selections(selections: &[KeySelection]) -> Result<Vec<Key>, String> {
    if selections.is_empty() {
        return Err("Choisis au moins une touche".to_string());
    }
    if selections.len() > 8 {
        return Err("Tu peux maintenir au maximum 8 touches".to_string());
    }

    selections
        .iter()
        .map(map_selection)
        .collect()
}

fn map_selection(selection: &KeySelection) -> Result<Key, String> {
    let physical_key = match selection.code.as_str() {
        "ControlLeft" => Some(Key::LControl),
        "ControlRight" => Some(Key::RControl),
        "ShiftLeft" => Some(Key::LShift),
        "ShiftRight" => Some(Key::RShift),
        "Numpad0" => Some(Key::Numpad0),
        "Numpad1" => Some(Key::Numpad1),
        "Numpad2" => Some(Key::Numpad2),
        "Numpad3" => Some(Key::Numpad3),
        "Numpad4" => Some(Key::Numpad4),
        "Numpad5" => Some(Key::Numpad5),
        "Numpad6" => Some(Key::Numpad6),
        "Numpad7" => Some(Key::Numpad7),
        "Numpad8" => Some(Key::Numpad8),
        "Numpad9" => Some(Key::Numpad9),
        "NumpadAdd" => Some(Key::Add),
        "NumpadSubtract" => Some(Key::Subtract),
        "NumpadMultiply" => Some(Key::Multiply),
        "NumpadDivide" => Some(Key::Divide),
        "NumpadDecimal" | "NumpadComma" => Some(Key::Decimal),
        "NumpadEnter" => Some(Key::Return),
        _ => None,
    };

    physical_key.map_or_else(|| map_key(&selection.key), Ok)
}

impl KeyController {
    pub fn new() -> Result<Self, String> {
        let enigo = Enigo::new(&Settings::default())
            .map_err(|error| format!("Impossible d’initialiser le clavier : {error}"))?;
        Ok(Self {
            enigo,
            held: Vec::new(),
        })
    }

    pub fn hold(&mut self, selections: Vec<KeySelection>) -> Result<(), String> {
        let keys = map_selections(&selections)?;
        self.release()?;

        for key in keys {
            if let Err(error) = self.enigo.key(key, Direction::Press) {
                let _ = self.release();
                return Err(format!("Impossible de maintenir cette touche : {error}"));
            }
            self.held.push(key);
        }

        Ok(())
    }

    pub fn release(&mut self) -> Result<(), String> {
        let mut first_error = None;

        while let Some(key) = self.held.pop() {
            if let Err(error) = self.enigo.key(key, Direction::Release) {
                first_error.get_or_insert_with(|| {
                    format!("Impossible de relâcher une touche : {error}")
                });
            }
        }

        first_error.map_or(Ok(()), Err)
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
        (Some(character), None) => Ok(Key::Unicode(
            character.to_lowercase().next().unwrap_or(character),
        )),
        _ => Err(format!(
            "La touche « {value} » n’est pas encore prise en charge"
        )),
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

    #[test]
    fn maps_multiple_keys_in_order() {
        let selections = vec![
            KeySelection {
                key: "Control".into(),
                code: "ControlLeft".into(),
                label: "Ctrl".into(),
            },
            KeySelection {
                key: "A".into(),
                code: "KeyA".into(),
                label: "A".into(),
            },
        ];

        assert_eq!(
            map_selections(&selections).unwrap(),
            vec![Key::LControl, Key::Unicode('a')]
        );
    }

    #[test]
    fn preserves_sided_modifiers_and_numpad_keys() {
        let selections = vec![
            KeySelection {
                key: "Control".into(),
                code: "ControlRight".into(),
                label: "Ctrl".into(),
            },
            KeySelection {
                key: "1".into(),
                code: "Numpad1".into(),
                label: "1".into(),
            },
        ];

        assert_eq!(
            map_selections(&selections).unwrap(),
            vec![Key::RControl, Key::Numpad1]
        );
    }

    #[test]
    fn rejects_empty_or_oversized_combinations() {
        assert!(map_selections(&[]).is_err());

        let selections = (0..9)
            .map(|_| KeySelection {
                key: "A".into(),
                code: "KeyA".into(),
                label: "A".into(),
            })
            .collect::<Vec<_>>();
        assert!(map_selections(&selections).is_err());
    }
}
