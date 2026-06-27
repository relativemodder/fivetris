use std::fmt;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::resources;

#[derive(Debug)]
pub enum ConfigError {
    Io(io::Error),
    InvalidDirectory(PathBuf),
    Parse {
        path: PathBuf,
        line: usize,
        message: String,
    },
}

impl fmt::Display for ConfigError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(error) => write!(formatter, "io error: {error}"),
            Self::InvalidDirectory(path) => {
                write!(
                    formatter,
                    "skin palette directory not found: {}",
                    path.display()
                )
            }
            Self::Parse {
                path,
                line,
                message,
            } => write!(formatter, "{}:{line}: {message}", path.display()),
        }
    }
}

impl std::error::Error for ConfigError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io(error) => Some(error),
            Self::InvalidDirectory(_) | Self::Parse { .. } => None,
        }
    }
}

impl From<io::Error> for ConfigError {
    fn from(error: io::Error) -> Self {
        Self::Io(error)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BlockStyle {
    Flat,
    Inset,
}

impl BlockStyle {
    fn from_ini_value(value: &str) -> Option<Self> {
        match value.trim() {
            "0" => Some(Self::Flat),
            "1" => Some(Self::Inset),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SkinPalette {
    pub name: String,
    pub empty: [u8; 4],
    pub i: [u8; 4],
    pub j: [u8; 4],
    pub s: [u8; 4],
    pub o: [u8; 4],
    pub z: [u8; 4],
    pub l: [u8; 4],
    pub t: [u8; 4],
    pub gray: [u8; 4],
    pub field: [u8; 4],
    pub background: [u8; 4],
    pub box_color: [u8; 4],
    pub text: [u8; 4],
    pub style: BlockStyle,
}

impl SkinPalette {
    pub fn matches_name(&self, name: &str) -> bool {
        self.name.eq_ignore_ascii_case(name)
    }
}

pub fn find_skin_palette<'a>(palettes: &'a [SkinPalette], name: &str) -> Option<&'a SkinPalette> {
    palettes.iter().find(|palette| palette.matches_name(name))
}

pub fn load_skin_palettes(dir: &Path) -> Result<Vec<SkinPalette>, ConfigError> {
    if !dir.is_dir() {
        return Err(ConfigError::InvalidDirectory(dir.to_path_buf()));
    }

    let mut ini_paths = Vec::new();
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.extension().and_then(|ext| ext.to_str()) == Some("ini") {
            ini_paths.push(path);
        }
    }
    ini_paths.sort();

    let mut palettes = Vec::new();
    for path in ini_paths {
        let raw = fs::read_to_string(&path)?;
        palettes.extend(load_skin_palettes_from_text(&path, &raw)?);
    }

    Ok(palettes)
}

pub fn load_builtin_skin_palettes() -> Result<Vec<SkinPalette>, ConfigError> {
    load_skin_palettes_from_text(
        Path::new("<embedded colors.ini>"),
        resources::BUILTIN_COLORS_INI,
    )
}

fn load_skin_palettes_from_text(
    path: &Path,
    raw: &str,
) -> Result<Vec<SkinPalette>, ConfigError> {
    let mut palettes = Vec::new();
    let mut current_name: Option<String> = None;
    let mut current_values: Vec<(String, String)> = Vec::new();

    for (index, raw_line) in raw.lines().enumerate() {
        let line_number = index + 1;
        let line = raw_line.trim();

        if line.is_empty() || line.starts_with(';') || line.starts_with('#') {
            continue;
        }

        if line.starts_with('[') {
            if !line.ends_with(']') {
                return Err(ConfigError::Parse {
                    path: path.to_path_buf(),
                    line: line_number,
                    message: "unterminated section header".to_string(),
                });
            }

            if let Some(name) = current_name.take() {
                palettes.push(build_palette(path, &name, &current_values)?);
                current_values.clear();
            }

            let name = line[1..line.len() - 1].trim();
            if name.is_empty() {
                return Err(ConfigError::Parse {
                    path: path.to_path_buf(),
                    line: line_number,
                    message: "empty section name".to_string(),
                });
            }

            current_name = Some(name.to_string());
            continue;
        }

        let Some((key, value)) = line.split_once('=') else {
            return Err(ConfigError::Parse {
                path: path.to_path_buf(),
                line: line_number,
                message: "expected key=value entry".to_string(),
            });
        };

        if current_name.is_none() {
            return Err(ConfigError::Parse {
                path: path.to_path_buf(),
                line: line_number,
                message: "entry appears before any section header".to_string(),
            });
        }

        current_values.push((key.trim().to_string(), value.trim().to_string()));
    }

    if let Some(name) = current_name {
        palettes.push(build_palette(path, &name, &current_values)?);
    }

    Ok(palettes)
}

fn build_palette(
    path: &Path,
    name: &str,
    entries: &[(String, String)],
) -> Result<SkinPalette, ConfigError> {
    let color = |key: &str, default: [u8; 4]| -> Result<[u8; 4], ConfigError> {
        match entries.iter().rev().find(|(entry_key, _)| entry_key == key) {
            Some((_, value)) => parse_ini_color(path, name, key, value),
            None => Ok(default),
        }
    };

    let style = match entries
        .iter()
        .rev()
        .find(|(entry_key, _)| entry_key == "STYLE")
    {
        Some((_, value)) => {
            BlockStyle::from_ini_value(value).ok_or_else(|| ConfigError::Parse {
                path: path.to_path_buf(),
                line: 0,
                message: format!("[{name}] invalid STYLE value: {value}"),
            })?
        }
        None => BlockStyle::Inset,
    };

    Ok(SkinPalette {
        name: name.to_string(),
        empty: color("E", [0x00, 0x00, 0x00, 0xFF])?,
        i: color("I", [0x00, 0xD0, 0xFF, 0xFF])?,
        j: color("J", [0x40, 0x80, 0xFF, 0xFF])?,
        s: color("S", [0x40, 0xD0, 0x40, 0xFF])?,
        o: color("O", [0xFF, 0xE0, 0x20, 0xFF])?,
        z: color("Z", [0xFF, 0x40, 0x20, 0xFF])?,
        l: color("L", [0xFF, 0x80, 0x20, 0xFF])?,
        t: color("T", [0xA0, 0x40, 0xF0, 0xFF])?,
        gray: color("G", [0xCC, 0xCC, 0xCC, 0xFF])?,
        field: color("F", [0x2F, 0x31, 0x36, 0xFF])?,
        background: color("BKG", [0x2F, 0x31, 0x36, 0xFF])?,
        box_color: color("BOX", [0x00, 0x00, 0x00, 0xFF])?,
        text: color("TXT", [0xFF, 0xFF, 0xFF, 0xFF])?,
        style,
    })
}

fn parse_ini_color(
    path: &Path,
    section: &str,
    key: &str,
    value: &str,
) -> Result<[u8; 4], ConfigError> {
    let hex = value.strip_prefix("0x").unwrap_or(value);
    if hex.len() != 6 {
        return Err(ConfigError::Parse {
            path: path.to_path_buf(),
            line: 0,
            message: format!("[{section}] invalid {key} color: {value}"),
        });
    }

    let rgb = u32::from_str_radix(hex, 16).map_err(|_| ConfigError::Parse {
        path: path.to_path_buf(),
        line: 0,
        message: format!("[{section}] invalid {key} color: {value}"),
    })?;

    Ok([
        ((rgb >> 16) & 0xFF) as u8,
        ((rgb >> 8) & 0xFF) as u8,
        (rgb & 0xFF) as u8,
        0xFF,
    ])
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::time::{SystemTime, UNIX_EPOCH};

    use super::{BlockStyle, find_skin_palette, load_skin_palettes};

    #[test]
    fn loads_reference_style_skin_palette_file() {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time before unix epoch")
            .as_nanos();
        let dir = std::env::temp_dir().join(format!("fivetris-skins-{unique}"));
        fs::create_dir_all(&dir).expect("create temp skin dir");
        let path = dir.join("colors.ini");
        fs::write(
            &path,
            "[DEFAULT]\nE=0x000000\nI=0x00D0FF\nJ=0x4080FF\nS=0x40D040\nO=0xFFE020\nZ=0xFF4020\nL=0xFF8020\nT=0xA040F0\nG=0xBBBBBB\nF=0x202020\nBKG=0x2F3136\nBOX=0x000000\nTXT=0xFFFFFF\nSTYLE=1\n\n[CHROMA]\nE=0x000000\nI=0x2DC1FF\nJ=0x2E55FF\nS=0x81FF06\nO=0xFFC12C\nZ=0xFF2D59\nL=0xFF802C\nT=0xFF22C3\nG=0xB2B2B2\nF=0x222222\nBKG=0x222222\nBOX=0x000000\nTXT=0xFFFFFF\nSTYLE=0\n",
        )
        .expect("write skin file");

        let palettes = load_skin_palettes(&dir).expect("load palettes");
        assert_eq!(palettes.len(), 2);

        let default_palette = find_skin_palette(&palettes, "default").expect("find default");
        assert_eq!(default_palette.i, [0x00, 0xD0, 0xFF, 0xFF]);
        assert_eq!(default_palette.background, [0x2F, 0x31, 0x36, 0xFF]);
        assert_eq!(default_palette.style, BlockStyle::Inset);

        let chroma = find_skin_palette(&palettes, "CHROMA").expect("find chroma");
        assert_eq!(chroma.text, [0xFF, 0xFF, 0xFF, 0xFF]);
        assert_eq!(chroma.style, BlockStyle::Flat);

        let _ = fs::remove_file(path);
        let _ = fs::remove_dir(dir);
    }
}
