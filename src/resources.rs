use std::path::PathBuf;

pub const BUILTIN_ICON_PNG: &[u8] = include_bytes!("../assets/icon.png");

pub const BUILTIN_COLORS_INI: &str = include_str!("../assets/colors.ini");
pub const BUILTIN_PIECE_LIST: &str = include_str!("../assets/piece_list.txt");

const BUILTIN_TEXTURES: [(&str, &[u8]); 2] = [
    (
        "template.png",
        include_bytes!("../assets/textures/template.png"),
    ),
    (
        "rounded-square.png",
        include_bytes!("../assets/textures/rounded-square.png"),
    ),
];

const BUILTIN_FONTS: [(&str, &[u8]); 4] = [
    ("Inter.ttf", include_bytes!("../assets/fonts/Inter.ttf")),
    (
        "Inter-Italic.ttf",
        include_bytes!("../assets/fonts/Inter-Italic.ttf"),
    ),
    (
        "FontAwesome-Solid.ttf",
        include_bytes!("../assets/fonts/FontAwesome-Solid.ttf"),
    ),
    (
        "FontAwesome-Regular.ttf",
        include_bytes!("../assets/fonts/FontAwesome-Regular.ttf"),
    ),
];

const BUILTIN_AUDIO: [(&str, &[u8]); 11] = [
    ("se_move.wav", include_bytes!("../assets/audio/se_move.wav")),
    (
        "se_rotate.wav",
        include_bytes!("../assets/audio/se_rotate.wav"),
    ),
    (
        "se_hdrop.wav",
        include_bytes!("../assets/audio/se_hdrop.wav"),
    ),
    ("se_hold.wav", include_bytes!("../assets/audio/se_hold.wav")),
    ("se_spin.wav", include_bytes!("../assets/audio/se_spin.wav")),
    (
        "se_clear_line.wav",
        include_bytes!("../assets/audio/se_clear_line.wav"),
    ),
    (
        "se_clear_tetris.wav",
        include_bytes!("../assets/audio/se_clear_tetris.wav"),
    ),
    (
        "se_clear_spin.wav",
        include_bytes!("../assets/audio/se_clear_spin.wav"),
    ),
    (
        "se_clear_btb.wav",
        include_bytes!("../assets/audio/se_clear_btb.wav"),
    ),
    ("se_down.wav", include_bytes!("../assets/audio/se_down.wav")),
    ("se_lose.wav", include_bytes!("../assets/audio/se_lose.wav")),
];

pub fn config_dir() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from(".config"))
        .join("fivetris")
}

pub fn data_dir() -> PathBuf {
    dirs::data_dir()
        .unwrap_or_else(|| PathBuf::from(".local/share"))
        .join("fivetris")
}

pub fn config_file(name: &str) -> PathBuf {
    config_dir().join(name)
}

pub fn data_subdir(name: &str) -> PathBuf {
    data_dir().join(name)
}

pub fn data_skins_dir() -> PathBuf {
    data_subdir("skins")
}

pub fn data_textures_dir() -> PathBuf {
    data_subdir("textures")
}

pub fn data_audio_dir() -> PathBuf {
    data_subdir("audio")
}

pub fn builtin_texture_names() -> &'static [&'static str] {
    &["template.png", "rounded-square.png"]
}

pub fn builtin_texture_bytes(name: &str) -> Option<&'static [u8]> {
    BUILTIN_TEXTURES
        .iter()
        .find(|(filename, _)| *filename == name)
        .map(|(_, bytes)| *bytes)
}

pub fn builtin_audio_bytes(name: &str) -> Option<&'static [u8]> {
    BUILTIN_AUDIO
        .iter()
        .find(|(filename, _)| *filename == name)
        .map(|(_, bytes)| *bytes)
}

pub fn builtin_font_bytes(name: &str) -> Option<&'static [u8]> {
    BUILTIN_FONTS
        .iter()
        .find(|(filename, _)| *filename == name)
        .map(|(_, bytes)| *bytes)
}
