use std::fmt;

use egui::Key;
use serde::de::{self, Visitor};
use serde::{Deserialize, Deserializer, Serialize, Serializer};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum KeyName {
    Escape,
    F1,
    F2,
    F3,
    F4,
    F5,
    F6,
    F7,
    R,
    Left,
    Right,
    Down,
    Up,
    Comma,
    Period,
    Slash,
    Space,
    Z,
    X,
    A,
    C,
    Y,
}

impl KeyName {
    pub const ALL: [Self; 22] = [
        Self::Escape,
        Self::F1,
        Self::F2,
        Self::F3,
        Self::F4,
        Self::F5,
        Self::F6,
        Self::F7,
        Self::R,
        Self::Left,
        Self::Right,
        Self::Down,
        Self::Up,
        Self::Comma,
        Self::Period,
        Self::Slash,
        Self::Space,
        Self::Z,
        Self::X,
        Self::A,
        Self::C,
        Self::Y,
    ];

    pub const fn egui_key(self) -> Key {
        match self {
            Self::Escape => Key::Escape,
            Self::F1 => Key::F1,
            Self::F2 => Key::F2,
            Self::F3 => Key::F3,
            Self::F4 => Key::F4,
            Self::F5 => Key::F5,
            Self::F6 => Key::F6,
            Self::F7 => Key::F7,
            Self::R => Key::R,
            Self::Left => Key::ArrowLeft,
            Self::Right => Key::ArrowRight,
            Self::Down => Key::ArrowDown,
            Self::Up => Key::ArrowUp,
            Self::Comma => Key::Comma,
            Self::Period => Key::Period,
            Self::Slash => Key::Slash,
            Self::Space => Key::Space,
            Self::Z => Key::Z,
            Self::X => Key::X,
            Self::A => Key::A,
            Self::C => Key::C,
            Self::Y => Key::Y,
        }
    }

    pub const fn serialized_name(self) -> &'static str {
        match self {
            Self::Escape => "escape",
            Self::F1 => "f1",
            Self::F2 => "f2",
            Self::F3 => "f3",
            Self::F4 => "f4",
            Self::F5 => "f5",
            Self::F6 => "f6",
            Self::F7 => "f7",
            Self::R => "r",
            Self::Left => "left",
            Self::Right => "right",
            Self::Down => "down",
            Self::Up => "up",
            Self::Comma => "comma",
            Self::Period => "period",
            Self::Slash => "slash",
            Self::Space => "space",
            Self::Z => "z",
            Self::X => "x",
            Self::A => "a",
            Self::C => "c",
            Self::Y => "y",
        }
    }

    pub const fn display_name(self) -> &'static str {
        match self {
            Self::Escape => "Esc",
            Self::F1 => "F1",
            Self::F2 => "F2",
            Self::F3 => "F3",
            Self::F4 => "F4",
            Self::F5 => "F5",
            Self::F6 => "F6",
            Self::F7 => "F7",
            Self::R => "R",
            Self::Left => "Left",
            Self::Right => "Right",
            Self::Down => "Down",
            Self::Up => "Up",
            Self::Comma => ",",
            Self::Period => ".",
            Self::Slash => "/",
            Self::Space => "Space",
            Self::Z => "Z",
            Self::X => "X",
            Self::A => "A",
            Self::C => "C",
            Self::Y => "Y",
        }
    }

    pub const fn from_egui_key(key: Key) -> Option<Self> {
        match key {
            Key::Escape => Some(Self::Escape),
            Key::F1 => Some(Self::F1),
            Key::F2 => Some(Self::F2),
            Key::F3 => Some(Self::F3),
            Key::F4 => Some(Self::F4),
            Key::F5 => Some(Self::F5),
            Key::F6 => Some(Self::F6),
            Key::F7 => Some(Self::F7),
            Key::R => Some(Self::R),
            Key::ArrowLeft => Some(Self::Left),
            Key::ArrowRight => Some(Self::Right),
            Key::ArrowDown => Some(Self::Down),
            Key::ArrowUp => Some(Self::Up),
            Key::Comma => Some(Self::Comma),
            Key::Period => Some(Self::Period),
            Key::Slash => Some(Self::Slash),
            Key::Space => Some(Self::Space),
            Key::Z => Some(Self::Z),
            Key::X => Some(Self::X),
            Key::A => Some(Self::A),
            Key::C => Some(Self::C),
            Key::Y => Some(Self::Y),
            _ => None,
        }
    }

    pub fn lookup(name: &str) -> Option<Self> {
        let normalized = name.trim().to_ascii_lowercase();
        match normalized.as_str() {
            "esc" | "escape" => Some(Self::Escape),
            "f1" => Some(Self::F1),
            "f2" => Some(Self::F2),
            "f3" => Some(Self::F3),
            "f4" => Some(Self::F4),
            "f5" => Some(Self::F5),
            "f6" => Some(Self::F6),
            "f7" => Some(Self::F7),
            "r" => Some(Self::R),
            "left" | "arrow_left" | "arrowleft" => Some(Self::Left),
            "right" | "arrow_right" | "arrowright" => Some(Self::Right),
            "down" | "arrow_down" | "arrowdown" => Some(Self::Down),
            "up" | "arrow_up" | "arrowup" => Some(Self::Up),
            "," | "comma" => Some(Self::Comma),
            "." | "period" => Some(Self::Period),
            "/" | "slash" => Some(Self::Slash),
            "space" => Some(Self::Space),
            "z" => Some(Self::Z),
            "x" => Some(Self::X),
            "a" => Some(Self::A),
            "c" => Some(Self::C),
            "y" => Some(Self::Y),
            _ => None,
        }
    }
}

impl Serialize for KeyName {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.serialized_name())
    }
}

impl<'de> Deserialize<'de> for KeyName {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_str(KeyNameVisitor)
    }
}

struct KeyNameVisitor;

impl Visitor<'_> for KeyNameVisitor {
    type Value = KeyName;

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("a supported key name")
    }

    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        KeyName::lookup(value).ok_or_else(|| E::unknown_variant(value, &SUPPORTED_KEY_NAMES))
    }
}

const SUPPORTED_KEY_NAMES: [&str; 21] = [
    "escape", "f1", "f2", "f3", "f4", "f5", "f6", "f7", "left", "right", "down", "up", "comma",
    "period", "slash", "space", "z", "x", "a", "c", "y",
];

#[cfg(test)]
mod tests {
    use super::KeyName;

    #[test]
    fn maps_supported_egui_keys() {
        for key_name in KeyName::ALL {
            assert_eq!(KeyName::from_egui_key(key_name.egui_key()), Some(key_name));
        }
    }

    #[test]
    fn looks_up_serialized_and_alias_names() {
        assert_eq!(KeyName::lookup("escape"), Some(KeyName::Escape));
        assert_eq!(KeyName::lookup("Esc"), Some(KeyName::Escape));
        assert_eq!(KeyName::lookup("left"), Some(KeyName::Left));
        assert_eq!(KeyName::lookup("arrow_left"), Some(KeyName::Left));
        assert_eq!(KeyName::lookup("ArrowDown"), Some(KeyName::Down));
        assert_eq!(KeyName::lookup("space"), Some(KeyName::Space));
        assert_eq!(KeyName::lookup("unknown"), None);
    }

    #[test]
    fn serializes_to_stable_names() {
        assert_eq!(
            serde_json::to_string(&KeyName::Left).expect("serialize key"),
            "\"left\""
        );
        assert_eq!(
            serde_json::to_string(&KeyName::Escape).expect("serialize key"),
            "\"escape\""
        );
    }

    #[test]
    fn deserializes_aliases() {
        assert_eq!(
            serde_json::from_str::<KeyName>("\"arrow_right\"").expect("deserialize key"),
            KeyName::Right
        );
        assert_eq!(
            serde_json::from_str::<KeyName>("\"F6\"").expect("deserialize key"),
            KeyName::F6
        );
    }
}
