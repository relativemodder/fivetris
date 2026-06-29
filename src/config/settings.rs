use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use super::load_skin_palettes;
use crate::config::KeyName;
use crate::config::skins::load_builtin_skin_palettes;
use crate::core::{BagMode, DEFAULT_BOARD_WIDTH, DEFAULT_HIDDEN_ROWS, DEFAULT_VISIBLE_HEIGHT};
use crate::render::load_texture_atlas;
use crate::resources;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PersistedBagMode {
    SevenBag,
    FourteenBag,
    Random,
}

impl Default for PersistedBagMode {
    fn default() -> Self {
        Self::SevenBag
    }
}

impl From<BagMode> for PersistedBagMode {
    fn from(value: BagMode) -> Self {
        match value {
            BagMode::SevenBag => Self::SevenBag,
            BagMode::FourteenBag => Self::FourteenBag,
            BagMode::Random => Self::Random,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ThemeMode {
    Auto,
    Light,
    Dark,
}

impl Default for ThemeMode {
    fn default() -> Self {
        Self::Auto
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct KeyBindings {
    #[serde(default = "default_pause_key")]
    pub pause: KeyName,
    #[serde(default = "default_reset_training_key")]
    pub reset_training: KeyName,
    #[serde(default = "default_reset_cheese_key")]
    pub reset_cheese: KeyName,
    #[serde(default = "default_reset_four_wide_key")]
    pub reset_four_wide: KeyName,
    #[serde(default = "default_reset_perfect_clear_key")]
    pub reset_perfect_clear: KeyName,
    #[serde(default = "default_reset_master_key")]
    pub reset_master: KeyName,
    #[serde(default = "default_screenshot_key")]
    pub screenshot: KeyName,
    #[serde(default = "default_move_left_key")]
    pub move_left: KeyName,
    #[serde(default = "default_move_right_key")]
    pub move_right: KeyName,
    #[serde(default = "default_soft_drop_key")]
    pub soft_drop: KeyName,
    #[serde(default = "default_hard_drop_keys")]
    pub hard_drop: Vec<KeyName>,
    #[serde(default = "default_rotate_ccw_key")]
    pub rotate_ccw: KeyName,
    #[serde(default = "default_rotate_cw_key")]
    pub rotate_cw: KeyName,
    #[serde(default = "default_rotate_180_key")]
    pub rotate_180: KeyName,
    #[serde(default = "default_hold_key")]
    pub hold: KeyName,
    #[serde(default)]
    pub hold_with_shift: bool,
    #[serde(default = "default_undo_key")]
    pub undo: KeyName,
    #[serde(default = "default_redo_key")]
    pub redo: KeyName,
}

impl Default for KeyBindings {
    fn default() -> Self {
        Self {
            pause: default_pause_key(),
            reset_training: default_reset_training_key(),
            reset_cheese: default_reset_cheese_key(),
            reset_four_wide: default_reset_four_wide_key(),
            reset_perfect_clear: default_reset_perfect_clear_key(),
            reset_master: default_reset_master_key(),
            screenshot: default_screenshot_key(),
            move_left: default_move_left_key(),
            move_right: default_move_right_key(),
            soft_drop: default_soft_drop_key(),
            hard_drop: default_hard_drop_keys(),
            rotate_ccw: default_rotate_ccw_key(),
            rotate_cw: default_rotate_cw_key(),
            rotate_180: default_rotate_180_key(),
            hold: default_hold_key(),
            hold_with_shift: false,
            undo: default_undo_key(),
            redo: default_redo_key(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct AppConfig {
    pub show_ghost: bool,
    pub volume_percent: u8,
    pub selected_skin: String,
    pub selected_texture: String,
    pub das_ms: u32,
    pub arr_ms: u32,
    pub sdd_ms: u32,
    pub sds_ms: u32,
    pub das_cancel: bool,
    pub board_width: usize,
    pub board_visible_height: usize,
    pub board_hidden_rows: usize,
    pub board_cell_size: u16,
    pub preview_cell_size: u16,
    pub infinite_hold: bool,
    pub auto_color: bool,
    pub auto_lock_on_ground: bool,
    #[serde(alias = "mirror_queue_with_field")]
    pub mirror_queue: bool,
    pub highlight_clear: bool,
    pub colored_ghost: bool,
    pub ghost_opacity_percent: u8,
    pub grid_brightness: u8,
    pub static_bag: bool,
    pub bag_mode: PersistedBagMode,
    pub theme: ThemeMode,
    pub bindings: KeyBindings,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            show_ghost: true,
            volume_percent: 70,
            selected_skin: "default".to_string(),
            selected_texture: "rounded-square.png".to_string(),
            das_ms: 133,
            arr_ms: 16,
            sdd_ms: 67,
            sds_ms: 16,
            das_cancel: true,
            board_width: DEFAULT_BOARD_WIDTH,
            board_visible_height: DEFAULT_VISIBLE_HEIGHT,
            board_hidden_rows: DEFAULT_HIDDEN_ROWS,
            board_cell_size: 30,
            preview_cell_size: 14,
            infinite_hold: true,
            auto_color: false,
            auto_lock_on_ground: false,
            mirror_queue: false,
            highlight_clear: false,
            colored_ghost: false,
            ghost_opacity_percent: 22,
            grid_brightness: 0,
            static_bag: false,
            bag_mode: PersistedBagMode::default(),
            theme: ThemeMode::default(),
            bindings: KeyBindings::default(),
        }
    }
}

impl AppConfig {
    pub fn skin_search_dirs() -> Vec<PathBuf> {
        vec![resources::data_skins_dir()]
    }

    pub fn textures_dir() -> PathBuf {
        resources::data_textures_dir()
    }

    pub fn default_path() -> PathBuf {
        resources::config_file("settings.json")
    }

    pub fn load() -> Result<Self, io::Error> {
        Self::load_from_path(&Self::default_path())
    }

    pub fn load_from_path(path: &Path) -> Result<Self, io::Error> {
        #[cfg(not(target_arch = "wasm32"))]
        {
            match fs::read_to_string(path) {
                Ok(raw) => {
                    let mut config: Self = serde_json::from_str(&raw)
                        .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))?;
                    config.normalize();
                    Ok(config)
                }
                Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(Self::default()),
                Err(error) => Err(error),
            }
        }
        #[cfg(target_arch = "wasm32")]
        {
            let key = path.to_string_lossy();
            let storage = web_sys::window()
                .and_then(|w| w.local_storage().ok())
                .flatten()
                .ok_or_else(|| {
                    io::Error::new(io::ErrorKind::Unsupported, "localStorage unavailable")
                })?;
            match storage
                .get_item(&key)
                .map_err(|e| io::Error::new(io::ErrorKind::Other, format!("{e:?}")))?
            {
                Some(raw) => {
                    let mut config: Self = serde_json::from_str(&raw)
                        .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))?;
                    config.normalize();
                    Ok(config)
                }
                None => Ok(Self::default()),
            }
        }
    }

    pub fn save(&self) -> Result<(), io::Error> {
        self.save_to_path(&Self::default_path())
    }

    pub fn save_to_path(&self, path: &Path) -> Result<(), io::Error> {
        let mut normalized = self.clone();
        normalized.normalize();
        let raw = serde_json::to_string_pretty(&normalized)
            .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))?;
        #[cfg(not(target_arch = "wasm32"))]
        {
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent)?;
            }
            fs::write(path, raw)
        }
        #[cfg(target_arch = "wasm32")]
        {
            let key = path.to_string_lossy();
            let storage = web_sys::window()
                .and_then(|w| w.local_storage().ok())
                .flatten()
                .ok_or_else(|| {
                    io::Error::new(io::ErrorKind::Unsupported, "localStorage unavailable")
                })?;
            storage
                .set_item(&key, &raw)
                .map_err(|e| io::Error::new(io::ErrorKind::Other, format!("{e:?}")))?;
            Ok(())
        }
    }

    fn normalize(&mut self) {
        self.volume_percent = self.volume_percent.min(100);
        if self.board_width == 0 {
            self.board_width = DEFAULT_BOARD_WIDTH;
        }
        if self.board_visible_height == 0 {
            self.board_visible_height = DEFAULT_VISIBLE_HEIGHT;
        }
        if self.board_hidden_rows > 8 {
            self.board_hidden_rows = DEFAULT_HIDDEN_ROWS;
        }
        self.ghost_opacity_percent = self.ghost_opacity_percent.min(100);
        self.grid_brightness = self.grid_brightness.min(100);
        if self.board_cell_size == 0 {
            self.board_cell_size = 28;
        }
        if self.preview_cell_size == 0 {
            self.preview_cell_size = 14;
        }
        if self.bindings.hard_drop.is_empty() {
            self.bindings.hard_drop = default_hard_drop_keys();
        }
    }

    pub fn available_skins() -> Vec<String> {
        let mut skins = Vec::new();

        if let Ok(loaded) = load_builtin_skin_palettes() {
            for palette in loaded {
                skins.push(palette.name);
            }
        }

        for dir in Self::skin_search_dirs() {
            if let Ok(loaded) = load_skin_palettes(&dir) {
                for palette in loaded {
                    skins.push(palette.name);
                }
            }
        }

        skins.sort_by_key(|name| name.to_ascii_lowercase());
        skins.dedup_by(|left, right| left.eq_ignore_ascii_case(right));

        if skins.is_empty() {
            skins.push(Self::default().selected_skin);
        }

        skins
    }

    pub fn available_textures() -> Vec<String> {
        let mut textures = resources::builtin_texture_names()
            .iter()
            .map(|name| (*name).to_string())
            .collect::<Vec<_>>();

        if let Ok(entries) = fs::read_dir(Self::textures_dir()) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().and_then(|ext| ext.to_str()) == Some("png") {
                    if load_texture_atlas(&path).is_ok() {
                        if let Some(name) = path.file_name().and_then(|name| name.to_str()) {
                            textures.push(name.to_string());
                        }
                    }
                }
            }
        }
        textures.sort();
        textures.dedup();
        textures
    }
}

const fn default_pause_key() -> KeyName {
    KeyName::Escape
}

const fn default_reset_training_key() -> KeyName {
    KeyName::F1
}

const fn default_reset_cheese_key() -> KeyName {
    KeyName::F2
}

const fn default_reset_four_wide_key() -> KeyName {
    KeyName::F3
}

const fn default_reset_perfect_clear_key() -> KeyName {
    KeyName::F4
}

const fn default_reset_master_key() -> KeyName {
    KeyName::F5
}

const fn default_screenshot_key() -> KeyName {
    KeyName::F6
}

const fn default_move_left_key() -> KeyName {
    KeyName::Left
}

const fn default_move_right_key() -> KeyName {
    KeyName::Right
}

const fn default_soft_drop_key() -> KeyName {
    KeyName::Down
}

fn default_hard_drop_keys() -> Vec<KeyName> {
    vec![KeyName::Up, KeyName::Space]
}

const fn default_rotate_ccw_key() -> KeyName {
    KeyName::Z
}

const fn default_rotate_cw_key() -> KeyName {
    KeyName::X
}

const fn default_rotate_180_key() -> KeyName {
    KeyName::A
}

const fn default_hold_key() -> KeyName {
    KeyName::C
}

const fn default_undo_key() -> KeyName {
    KeyName::Z
}

const fn default_redo_key() -> KeyName {
    KeyName::Y
}

#[cfg(test)]
mod tests {
    use std::time::{SystemTime, UNIX_EPOCH};

    use super::{AppConfig, PersistedBagMode, ThemeMode};
    use crate::config::KeyName;

    #[test]
    fn default_config_matches_expected_values() {
        let config = AppConfig::default();

        assert!(config.show_ghost);
        assert_eq!(config.volume_percent, 70);
        assert_eq!(config.selected_skin, "default");
        assert_eq!(config.selected_texture, "rounded-square.png");
        assert_eq!(config.das_ms, 133);
        assert_eq!(config.arr_ms, 16);
        assert_eq!(config.sdd_ms, 67);
        assert_eq!(config.sds_ms, 16);
        assert!(config.das_cancel);
        assert_eq!(config.board_width, 10);
        assert_eq!(config.board_visible_height, 20);
        assert_eq!(config.board_hidden_rows, 4);
        assert_eq!(config.board_cell_size, 30);
        assert_eq!(config.preview_cell_size, 14);
        assert!(config.infinite_hold);
        assert!(!config.auto_color);
        assert!(!config.auto_lock_on_ground);
        assert!(!config.mirror_queue);
        assert!(!config.highlight_clear);
        assert!(!config.static_bag);
        assert_eq!(config.bag_mode, PersistedBagMode::SevenBag);
        assert_eq!(config.bindings.pause, KeyName::Escape);
        assert_eq!(config.bindings.move_left, KeyName::Left);
        assert_eq!(config.bindings.hard_drop, vec![KeyName::Up, KeyName::Space]);
        assert!(!config.bindings.hold_with_shift);
    }

    #[test]
    fn available_skins_includes_reference_defaults() {
        let skins = AppConfig::available_skins();

        assert!(
            skins
                .iter()
                .any(|skin| skin.eq_ignore_ascii_case("default"))
        );
        assert!(skins.iter().any(|skin| skin.eq_ignore_ascii_case("chroma")));
    }

    #[test]
    fn config_round_trips_through_json_file() {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time before unix epoch")
            .as_nanos();
        let path = std::env::temp_dir().join(format!("fivetris-settings-{unique}.json"));
        let config = AppConfig {
            show_ghost: false,
            volume_percent: 22,
            selected_skin: "default".to_string(),
            selected_texture: "template.png".to_string(),
            das_ms: 85,
            arr_ms: 1,
            sdd_ms: 2,
            sds_ms: 3,
            das_cancel: true,
            board_width: 12,
            board_visible_height: 22,
            board_hidden_rows: 3,
            board_cell_size: 32,
            preview_cell_size: 16,
            infinite_hold: false,
            auto_color: true,
            auto_lock_on_ground: true,
            mirror_queue: true,
            highlight_clear: true,
            colored_ghost: true,
            ghost_opacity_percent: 50,
            grid_brightness: 50,
            static_bag: true,
            bag_mode: PersistedBagMode::Random,
            theme: ThemeMode::Auto,
            bindings: super::KeyBindings {
                hard_drop: vec![KeyName::Space],
                hold: KeyName::X,
                hold_with_shift: true,
                ..Default::default()
            },
        };

        config.save_to_path(&path).expect("save config");
        let loaded = AppConfig::load_from_path(&path).expect("load config");
        assert_eq!(loaded, config);

        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn old_json_missing_new_fields_uses_defaults() {
        let loaded: AppConfig = serde_json::from_str(
            r#"{
                "show_ghost": false,
                "volume_percent": 88,
                "selected_skin": "chroma",
                "selected_texture": "template.png"
            }"#,
        )
        .expect("deserialize legacy config");

        assert!(!loaded.show_ghost);
        assert_eq!(loaded.volume_percent, 88);
        assert_eq!(loaded.selected_skin, "chroma");
        assert_eq!(loaded.selected_texture, "template.png");
        assert_eq!(loaded.das_ms, 133);
        assert_eq!(loaded.arr_ms, 16);
        assert!(loaded.infinite_hold);
        assert_eq!(loaded.bag_mode, PersistedBagMode::SevenBag);
        assert_eq!(loaded.bindings.hard_drop, vec![KeyName::Up, KeyName::Space]);
    }

    #[test]
    fn deserializes_partial_new_fields_and_aliases() {
        let loaded: AppConfig = serde_json::from_str(
            r#"{
                "mirror_queue_with_field": true,
                "bag_mode": "fourteen_bag",
                "bindings": {
                    "move_left": "a",
                    "hard_drop": ["space"]
                }
            }"#,
        )
        .expect("deserialize partial config");

        assert!(loaded.mirror_queue);
        assert_eq!(loaded.bag_mode, PersistedBagMode::FourteenBag);
        assert_eq!(loaded.bindings.move_left, KeyName::A);
        assert_eq!(loaded.bindings.move_right, KeyName::Right);
        assert_eq!(loaded.bindings.hard_drop, vec![KeyName::Space]);
    }
}
