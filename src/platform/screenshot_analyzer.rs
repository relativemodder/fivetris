use image::RgbaImage;

use crate::core::game_state::GameState;
use crate::core::highlight::auto_color_board;
use crate::core::piece::{Cell, Rotation};

#[derive(Debug, Clone, PartialEq)]
pub struct ScreenshotAnalysisConfig {
    pub board_width: usize,
    pub board_visible_height: usize,
    pub neighborhood_radius: u32,
    pub empty_lightness_threshold: f32,
    pub empty_lightness_high_threshold: f32,
    pub gray_saturation_threshold: f32,
}

impl Default for ScreenshotAnalysisConfig {
    fn default() -> Self {
        Self {
            board_width: crate::core::DEFAULT_BOARD_WIDTH,
            board_visible_height: crate::core::DEFAULT_VISIBLE_HEIGHT,
            neighborhood_radius: 1,
            empty_lightness_threshold: 20.833,
            empty_lightness_high_threshold: 80.0,
            gray_saturation_threshold: 8.333,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BoardAnalysisResult {
    pub width: usize,
    pub visible_rows: usize,
    pub cells: Vec<Cell>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AnalysisError {
    InvalidConfig(&'static str),
    ImageTooSmall,
    InvalidImageBuffer,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ImportError {
    WidthMismatch { expected: usize, actual: usize },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum HueBand {
    Red,
    Orange,
    Yellow,
    Green,
    Cyan,
    Blue,
    Magenta,
    Unknown,
}

pub fn analyze_board_image(
    image: &RgbaImage,
    config: &ScreenshotAnalysisConfig,
) -> Result<BoardAnalysisResult, AnalysisError> {
    if config.board_width == 0 {
        return Err(AnalysisError::InvalidConfig(
            "board_width must be greater than zero",
        ));
    }
    if config.board_visible_height == 0 {
        return Err(AnalysisError::InvalidConfig(
            "board_visible_height must be greater than zero",
        ));
    }
    if image.width() == 0 || image.height() == 0 {
        return Err(AnalysisError::ImageTooSmall);
    }

    let cell_size = image.width() as f32 / config.board_width as f32;
    if cell_size < 1.0 {
        return Err(AnalysisError::ImageTooSmall);
    }

    let rows_in_image = (image.height() as f32 / cell_size).floor() as usize;
    if rows_in_image == 0 {
        return Err(AnalysisError::ImageTooSmall);
    }

    let visible_rows = rows_in_image.min(config.board_visible_height);
    let vertical_offset = image.height() as f32 - rows_in_image as f32 * cell_size;
    let center = cell_size / 2.0;
    let mut cells = Vec::with_capacity(config.board_width * visible_rows);

    for row in 0..visible_rows {
        for col in 0..config.board_width {
            let sample_x = (col as f32 * cell_size + center).floor() as i32;
            let sample_y = (vertical_offset + row as f32 * cell_size + center).floor() as i32;
            let avg = average_neighborhood(image, sample_x, sample_y, config.neighborhood_radius);
            cells.push(classify_cell(avg, config));
        }
    }

    Ok(BoardAnalysisResult {
        width: config.board_width,
        visible_rows,
        cells,
    })
}

pub fn import_board_from_analysis(
    game: &mut GameState,
    analysis: BoardAnalysisResult,
) -> Result<(), ImportError> {
    if game.board.width != analysis.width {
        return Err(ImportError::WidthMismatch {
            expected: game.board.width,
            actual: analysis.width,
        });
    }

    game.board
        .replace_visible_rows_bottom_aligned(&analysis.cells, analysis.visible_rows);
    game.highlights.sync_to_board(&game.board);
    if game.auto_color {
        auto_color_board(&mut game.board);
    }

    game.current.x = game.board.spawn_x();
    game.current.y = game.board.spawn_y();
    game.current.rotation = Rotation::Spawn;
    game.hold.swapped_this_turn = false;
    game.stats.lost = false;
    game.stats.perfect_clear = false;
    game.stats.attack_text = None;
    game.stats.b2b_text = None;
    game.spin.is_t_spin = false;
    game.spin.is_mini = false;
    game.spin.last_kick_index = None;
    game.lock_delay.reset();
    game.clear_flash.active = false;
    game.clear_flash.lines.clear();
    game.clear_flash.timer_ms = 0;

    Ok(())
}

fn average_neighborhood(image: &RgbaImage, center_x: i32, center_y: i32, radius: u32) -> [u8; 4] {
    let radius = radius as i32;
    let width = image.width() as i32;
    let height = image.height() as i32;
    let mut totals = [0u32; 4];
    let mut samples = 0u32;

    for dy in -radius..=radius {
        for dx in -radius..=radius {
            let x = (center_x + dx).clamp(0, width - 1) as u32;
            let y = (center_y + dy).clamp(0, height - 1) as u32;
            let pixel = image.get_pixel(x, y).0;
            for channel in 0..4 {
                totals[channel] += u32::from(pixel[channel]);
            }
            samples += 1;
        }
    }

    [
        (totals[0] / samples) as u8,
        (totals[1] / samples) as u8,
        (totals[2] / samples) as u8,
        (totals[3] / samples) as u8,
    ]
}

fn classify_cell(rgba: [u8; 4], config: &ScreenshotAnalysisConfig) -> Cell {
    if rgba[3] == 0 {
        return Cell::Empty;
    }

    let (hue, saturation, lightness) = rgb_to_hsl(rgba[0], rgba[1], rgba[2]);
    if lightness < config.empty_lightness_threshold {
        return Cell::Empty;
    }
    if lightness > config.empty_lightness_high_threshold {
        return Cell::Empty;
    }
    if saturation < config.gray_saturation_threshold {
        return Cell::Gray;
    }

    match classify_hue_band(scale_hue_to_reference(hue)) {
        HueBand::Red => Cell::Z,
        HueBand::Orange => Cell::L,
        HueBand::Yellow => Cell::O,
        HueBand::Green => Cell::S,
        HueBand::Cyan => Cell::I,
        HueBand::Blue => Cell::J,
        HueBand::Magenta => Cell::T,
        HueBand::Unknown => Cell::Gray,
    }
}

fn classify_hue_band(hue: f32) -> HueBand {
    if (220.0..=240.0).contains(&hue) || (0.0..=15.0).contains(&hue) {
        HueBand::Red
    } else if (15.0..=27.0).contains(&hue) {
        HueBand::Orange
    } else if (27.0..=45.0).contains(&hue) {
        HueBand::Yellow
    } else if (45.0..=100.0).contains(&hue) {
        HueBand::Green
    } else if (100.0..=135.0).contains(&hue) {
        HueBand::Cyan
    } else if (135.0..=175.0).contains(&hue) {
        HueBand::Blue
    } else if (175.0..=220.0).contains(&hue) {
        HueBand::Magenta
    } else {
        HueBand::Unknown
    }
}

fn scale_hue_to_reference(hue_degrees: f32) -> f32 {
    hue_degrees * (240.0 / 360.0)
}

fn rgb_to_hsl(red: u8, green: u8, blue: u8) -> (f32, f32, f32) {
    let red = red as f32 / 255.0;
    let green = green as f32 / 255.0;
    let blue = blue as f32 / 255.0;

    let max = red.max(green).max(blue);
    let min = red.min(green).min(blue);
    let lightness = (max + min) / 2.0;

    if (max - min).abs() < f32::EPSILON {
        return (0.0, 0.0, lightness * 100.0);
    }

    let delta = max - min;
    let saturation = delta / (1.0 - (2.0 * lightness - 1.0).abs());
    let hue = if (max - red).abs() < f32::EPSILON {
        60.0 * (((green - blue) / delta).rem_euclid(6.0))
    } else if (max - green).abs() < f32::EPSILON {
        60.0 * (((blue - red) / delta) + 2.0)
    } else {
        60.0 * (((red - green) / delta) + 4.0)
    };

    (hue, saturation * 100.0, lightness * 100.0)
}

pub fn screenshot_image_to_rgba(
    image: &crate::platform::ScreenshotImage,
) -> Result<RgbaImage, AnalysisError> {
    RgbaImage::from_raw(image.width, image.height, image.rgba.clone())
        .ok_or(AnalysisError::InvalidImageBuffer)
}

#[cfg(test)]
mod tests {
    use super::{BoardAnalysisResult, HueBand, classify_hue_band, import_board_from_analysis};
    use crate::core::board::{Board, HighlightBoard};
    use crate::core::game_state::GameState;
    use crate::core::piece::{Cell, Tetromino};
    use crate::core::test_support;

    fn make_game_state() -> GameState {
        let mut game = test_support::game_state();
        game.board = Board::new(10, 20, 4);
        game.highlights = HighlightBoard::new(10, 24);
        game.current.kind = Tetromino::T;
        game.current.y = 22;
        game.hold.swapped_this_turn = true;
        game.hold.infinite_hold = false;
        game.queue.visible = vec![Tetromino::I];
        game.stats.damage = 1;
        game.stats.lines = 2;
        game.stats.moves = 3;
        game.stats.b2b = true;
        game.stats.perfect_clear = true;
        game.stats.combo = 4;
        game.stats.lost = true;
        game.stats.attack_text = Some("attack".to_string());
        game.stats.b2b_text = Some("b2b".to_string());
        game.spin.is_t_spin = true;
        game.spin.is_mini = true;
        game.spin.last_kick_index = Some(1);
        game.lock_delay.active = true;
        game.lock_delay.timer_ms = 100;
        game.lock_delay.moves = 2;
        game.clear_flash.active = true;
        game.clear_flash.lines = vec![20];
        game.clear_flash.timer_ms = 150;
        game
    }

    #[test]
    fn hue_bands_match_reference_boundaries() {
        assert_eq!(classify_hue_band(0.0), HueBand::Red);
        assert_eq!(classify_hue_band(20.0), HueBand::Orange);
        assert_eq!(classify_hue_band(35.0), HueBand::Yellow);
        assert_eq!(classify_hue_band(80.0), HueBand::Green);
        assert_eq!(classify_hue_band(120.0), HueBand::Cyan);
        assert_eq!(classify_hue_band(160.0), HueBand::Blue);
        assert_eq!(classify_hue_band(200.0), HueBand::Magenta);
    }

    #[test]
    fn import_board_from_analysis_clears_highlights_and_transient_state() {
        let mut game = make_game_state();
        game.highlights.set(2, 10, 9);

        import_board_from_analysis(
            &mut game,
            BoardAnalysisResult {
                width: 10,
                visible_rows: 2,
                cells: vec![
                    Cell::J,
                    Cell::Empty,
                    Cell::L,
                    Cell::Empty,
                    Cell::Empty,
                    Cell::Empty,
                    Cell::Empty,
                    Cell::Empty,
                    Cell::Empty,
                    Cell::Empty,
                    Cell::Gray,
                    Cell::S,
                    Cell::Z,
                    Cell::I,
                    Cell::O,
                    Cell::T,
                    Cell::J,
                    Cell::L,
                    Cell::S,
                    Cell::Z,
                ],
            },
        )
        .unwrap();

        assert!(game.highlights.cells.iter().all(|&cell| cell == 0));
        assert!(!game.hold.swapped_this_turn);
        assert!(!game.stats.lost);
        assert!(!game.stats.perfect_clear);
        assert_eq!(game.stats.attack_text, None);
        assert_eq!(game.stats.b2b_text, None);
        assert!(!game.spin.is_t_spin);
        assert!(!game.spin.is_mini);
        assert_eq!(game.spin.last_kick_index, None);
        assert!(!game.lock_delay.active);
        assert_eq!(game.lock_delay.timer_ms, 0);
        assert_eq!(game.lock_delay.moves, 0);
        assert!(!game.clear_flash.active);
        assert!(game.clear_flash.lines.is_empty());
        assert_eq!(game.clear_flash.timer_ms, 0);
    }

    #[test]
    fn import_board_from_analysis_auto_colors_gray_tetromino_regions() {
        let mut game = make_game_state();
        game.auto_color = true;

        import_board_from_analysis(
            &mut game,
            BoardAnalysisResult {
                width: 10,
                visible_rows: 2,
                cells: vec![
                    Cell::Empty,
                    Cell::Empty,
                    Cell::Empty,
                    Cell::Empty,
                    Cell::Gray,
                    Cell::Empty,
                    Cell::Empty,
                    Cell::Empty,
                    Cell::Empty,
                    Cell::Empty,
                    Cell::Empty,
                    Cell::Empty,
                    Cell::Empty,
                    Cell::Gray,
                    Cell::Gray,
                    Cell::Gray,
                    Cell::Empty,
                    Cell::Empty,
                    Cell::Empty,
                    Cell::Empty,
                ],
            },
        )
        .unwrap();

        let bottom = game.board.total_height() - 1;
        assert_eq!(game.board.get(4, bottom), Cell::T);
        assert_eq!(game.board.get(3, bottom), Cell::T);
        assert_eq!(game.board.get(4, bottom - 1), Cell::T);
        assert_eq!(game.board.get(5, bottom), Cell::T);
    }
}
