use std::fmt;
use std::path::{Path, PathBuf};

use egui::{Pos2, Rect};
use image::{self, RgbaImage};

const EXPECTED_TILE_COUNT: u32 = 10;
const TILE_COLUMN_ORDER: [u32; EXPECTED_TILE_COUNT as usize] = [9, 4, 5, 3, 2, 0, 1, 6, 7, 8];

#[derive(Debug, Clone)]
pub struct TextureAtlas {
    pub path: PathBuf,
    pub image: RgbaImage,
    pub tile_width: u32,
    pub tile_height: u32,
    pub tile_count: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TextureError {
    Io(String),
    InvalidDimensions {
        width: u32,
        height: u32,
    },
    InvalidAspectRatio {
        width: u32,
        height: u32,
        expected_tiles: u32,
    },
}

impl fmt::Display for TextureError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(message) => write!(f, "texture atlas load error: {message}"),
            Self::InvalidDimensions { width, height } => {
                write!(
                    f,
                    "texture atlas dimensions must be non-zero, got {width}x{height}"
                )
            }
            Self::InvalidAspectRatio {
                width,
                height,
                expected_tiles,
            } => {
                write!(
                    f,
                    "texture atlas must be a single-row strip with {expected_tiles} equal tiles, got {width}x{height}"
                )
            }
        }
    }
}

impl std::error::Error for TextureError {}

impl TextureAtlas {
    pub fn tile_uv_rect(&self, tile_index: usize) -> Rect {
        let clamped = tile_index.min(self.tile_count.saturating_sub(1) as usize);
        let column = TILE_COLUMN_ORDER[clamped] as f32;
        let tile_count = self.tile_count as f32;
        let min_x = column / tile_count;
        let max_x = (column + 1.0) / tile_count;
        Rect::from_min_max(Pos2::new(min_x, 0.0), Pos2::new(max_x, 1.0))
    }
}

pub fn load_texture_atlas(path: &Path) -> Result<TextureAtlas, TextureError> {
    let image = image::open(path)
        .map_err(|error| TextureError::Io(error.to_string()))?
        .into_rgba8();
    let (width, height) = image.dimensions();
    let (tile_width, tile_height, tile_count) = validate_texture_atlas_dimensions(width, height)?;

    Ok(TextureAtlas {
        path: path.to_path_buf(),
        image,
        tile_width,
        tile_height,
        tile_count,
    })
}

pub fn load_texture_atlas_bytes(name: &str, bytes: &[u8]) -> Result<TextureAtlas, TextureError> {
    let image = image::load_from_memory(bytes)
        .map_err(|error| TextureError::Io(error.to_string()))?
        .into_rgba8();
    let (width, height) = image.dimensions();
    let (tile_width, tile_height, tile_count) = validate_texture_atlas_dimensions(width, height)?;

    Ok(TextureAtlas {
        path: PathBuf::from(format!("embedded:{name}")),
        image,
        tile_width,
        tile_height,
        tile_count,
    })
}

fn validate_texture_atlas_dimensions(
    width: u32,
    height: u32,
) -> Result<(u32, u32, u32), TextureError> {
    if width == 0 || height == 0 {
        return Err(TextureError::InvalidDimensions { width, height });
    }

    if width != height.saturating_mul(EXPECTED_TILE_COUNT) {
        return Err(TextureError::InvalidAspectRatio {
            width,
            height,
            expected_tiles: EXPECTED_TILE_COUNT,
        });
    }

    Ok((height, height, EXPECTED_TILE_COUNT))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validation_accepts_current_single_row_strip() {
        let result = validate_texture_atlas_dimensions(300, 30).expect("atlas should validate");

        assert_eq!(result, (30, 30, 10));
    }

    #[test]
    fn tile_uv_rect_uses_reference_column_order() {
        let bytes = crate::resources::builtin_texture_bytes("template.png")
            .expect("template.png should be embedded");
        let atlas =
            load_texture_atlas_bytes("template.png", bytes).expect("template atlas should load");

        assert_eq!(
            atlas.tile_uv_rect(0),
            Rect::from_min_max(Pos2::new(0.9, 0.0), Pos2::new(1.0, 1.0))
        );
        assert_eq!(
            atlas.tile_uv_rect(1),
            Rect::from_min_max(Pos2::new(0.4, 0.0), Pos2::new(0.5, 1.0))
        );
        assert_eq!(
            atlas.tile_uv_rect(9),
            Rect::from_min_max(Pos2::new(0.8, 0.0), Pos2::new(0.9, 1.0))
        );
    }

    #[test]
    fn validation_rejects_zero_dimensions() {
        let error = validate_texture_atlas_dimensions(0, 30).expect_err("zero width should fail");

        assert_eq!(
            error,
            TextureError::InvalidDimensions {
                width: 0,
                height: 30,
            }
        );
    }

    #[test]
    fn validation_rejects_non_square_tiles() {
        let error = validate_texture_atlas_dimensions(320, 30)
            .expect_err("unexpected strip shape should fail");

        assert_eq!(
            error,
            TextureError::InvalidAspectRatio {
                width: 320,
                height: 30,
                expected_tiles: 10,
            }
        );
    }

    #[test]
    fn load_texture_atlas_reads_current_asset() {
        let bytes = crate::resources::builtin_texture_bytes("template.png")
            .expect("template.png should be embedded");
        let atlas =
            load_texture_atlas_bytes("template.png", bytes).expect("template atlas should load");

        assert_eq!(atlas.tile_width, 30);
        assert_eq!(atlas.tile_height, 30);
        assert_eq!(atlas.tile_count, 10);
        assert_eq!(atlas.image.dimensions(), (300, 30));
    }
}
