use fivetris::core::game_loop::GameLoop;
use fivetris::core::piece::Cell;
use fivetris::core::{BagMode, GameMode};
use fivetris::platform::{
    ScreenshotAnalysisConfig, analyze_board_image, import_board_from_analysis,
};
use image::{Rgba, RgbaImage};

fn board_image(rows: &[&[Rgba<u8>]], cell_size: u32) -> RgbaImage {
    let width = rows.first().map(|row| row.len()).unwrap_or(0) as u32;
    let height = rows.len() as u32;
    let mut image = RgbaImage::new(width * cell_size, height * cell_size);

    for (row_index, row) in rows.iter().enumerate() {
        for (col_index, color) in row.iter().enumerate() {
            for y in 0..cell_size {
                for x in 0..cell_size {
                    image.put_pixel(
                        col_index as u32 * cell_size + x,
                        row_index as u32 * cell_size + y,
                        *color,
                    );
                }
            }
        }
    }

    image
}

fn analyze_cells(rows: &[&[Rgba<u8>]]) -> Vec<Cell> {
    let image = board_image(rows, 8);
    analyze_board_image(
        &image,
        &ScreenshotAnalysisConfig {
            board_width: rows[0].len(),
            board_visible_height: rows.len(),
            ..ScreenshotAnalysisConfig::default()
        },
    )
    .unwrap()
    .cells
}

#[test]
fn detects_empty_cells() {
    let cells = analyze_cells(&[&[Rgba([0, 0, 0, 255]), Rgba([10, 10, 10, 255])]]);
    assert_eq!(cells, vec![Cell::Empty, Cell::Empty]);
}

#[test]
fn detects_gray_garbage_cells() {
    let cells = analyze_cells(&[&[Rgba([160, 160, 160, 255]), Rgba([136, 136, 136, 255])]]);
    assert_eq!(cells, vec![Cell::Gray, Cell::Gray]);
}

#[test]
fn detects_white_cells_as_empty_for_light_theme() {
    let cells = analyze_cells(&[&[
        Rgba([255, 255, 255, 255]),
        Rgba([245, 245, 245, 255]),
        Rgba([240, 240, 240, 255]),
    ]]);
    assert_eq!(cells, vec![Cell::Empty, Cell::Empty, Cell::Empty]);
}

#[test]
fn detects_warm_white_as_empty_not_o_piece() {
    let cells = analyze_cells(&[&[
        Rgba([255, 255, 230, 255]),
        Rgba([250, 250, 220, 255]),
    ]]);
    assert_eq!(cells, vec![Cell::Empty, Cell::Empty]);
}

#[test]
fn detects_dark_gray_garbage_on_light_theme() {
    let cells = analyze_cells(&[&[Rgba([85, 85, 85, 255]), Rgba([68, 68, 68, 255])]]);
    assert_eq!(cells, vec![Cell::Gray, Cell::Gray]);
}

#[test]
fn detects_colored_pieces_on_light_theme() {
    let cells = analyze_cells(&[&[
        Rgba([0, 255, 255, 255]),
        Rgba([255, 0, 0, 255]),
        Rgba([255, 255, 0, 255]),
    ]]);
    assert_eq!(cells, vec![Cell::I, Cell::Z, Cell::O]);
}

#[test]
fn detects_each_tetromino_color_bucket() {
    let cells = analyze_cells(&[&[
        Rgba([0, 255, 255, 255]),
        Rgba([0, 0, 255, 255]),
        Rgba([0, 255, 0, 255]),
        Rgba([255, 255, 0, 255]),
        Rgba([255, 0, 0, 255]),
        Rgba([255, 128, 0, 255]),
        Rgba([255, 0, 255, 255]),
    ]]);

    assert_eq!(
        cells,
        vec![
            Cell::I,
            Cell::J,
            Cell::S,
            Cell::O,
            Cell::Z,
            Cell::L,
            Cell::T
        ]
    );
}

#[test]
fn imports_fewer_visible_rows_bottom_aligned() {
    let image = board_image(
        &[
            &[Rgba([255, 255, 0, 255]), Rgba([0, 0, 0, 255])],
            &[Rgba([255, 0, 0, 255]), Rgba([160, 160, 160, 255])],
        ],
        10,
    );
    let analysis = analyze_board_image(
        &image,
        &ScreenshotAnalysisConfig {
            board_width: 2,
            board_visible_height: 4,
            ..ScreenshotAnalysisConfig::default()
        },
    )
    .unwrap();

    let mut game = GameLoop::new(GameMode::Training, BagMode::SevenBag, 0, true).game;
    game.board = fivetris::core::Board::new(2, 4, 1);
    import_board_from_analysis(&mut game, analysis).unwrap();

    assert_eq!(game.board.get(0, 0), Cell::Empty);
    assert_eq!(game.board.get(1, 0), Cell::Empty);
    assert_eq!(game.board.get(0, 1), Cell::Empty);
    assert_eq!(game.board.get(1, 1), Cell::Empty);
    assert_eq!(game.board.get(0, 2), Cell::Empty);
    assert_eq!(game.board.get(1, 2), Cell::Empty);
    assert_eq!(game.board.get(0, 3), Cell::O);
    assert_eq!(game.board.get(1, 3), Cell::Empty);
    assert_eq!(game.board.get(0, 4), Cell::Z);
    assert_eq!(game.board.get(1, 4), Cell::Gray);
}
