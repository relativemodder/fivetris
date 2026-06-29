use std::fmt;

use base64::Engine;

use super::board::{Board, HighlightBoard};
use super::game_state::{
    ClearFlashState, GameMode, GameState, GravityState, HoldState, LockDelayState, QueueState,
    SpinState, Stats,
};
use super::piece::{
    Cell, DEFAULT_HIDDEN_ROWS, MAX_BOARD_WIDTH, MAX_VISIBLE_HEIGHT, PieceState, Rotation,
    Tetromino, cell_from_piece, piece_from_name,
};

#[derive(Debug)]
pub enum CodecError {
    Json(serde_json::Error),
    MissingLegacyQueueSection,
    MissingLegacyBoardSection,
    InvalidLegacyQueueEncoding(base64::DecodeError),
    InvalidLegacyQueueHex(char),
    InvalidLegacyBoardCharacter(char),
    InvalidLegacyBoardShape(String),
    UnsupportedLegacyBoardEncoding,
}

impl fmt::Display for CodecError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Json(error) => write!(f, "state codec json error: {error}"),
            Self::MissingLegacyQueueSection => {
                write!(f, "legacy import is missing a queue section")
            }
            Self::MissingLegacyBoardSection => {
                write!(f, "legacy import is missing a board section")
            }
            Self::InvalidLegacyQueueEncoding(error) => {
                write!(f, "legacy queue data is not valid base64: {error}")
            }
            Self::InvalidLegacyQueueHex(ch) => {
                write!(f, "legacy queue contains an invalid hex digit: {ch}")
            }
            Self::InvalidLegacyBoardCharacter(ch) => {
                write!(f, "legacy board contains an invalid cell character: {ch}")
            }
            Self::InvalidLegacyBoardShape(message) => {
                write!(f, "legacy board shape error: {message}")
            }
            Self::UnsupportedLegacyBoardEncoding => {
                write!(
                    f,
                    "legacy board payload uses compressed encoding that is not supported"
                )
            }
        }
    }
}

impl std::error::Error for CodecError {}

pub fn encode_json_state(game: &GameState) -> Result<String, CodecError> {
    serde_json::to_string(game).map_err(CodecError::Json)
}

pub fn decode_json_state(input: &str) -> Result<GameState, CodecError> {
    serde_json::from_str(input).map_err(CodecError::Json)
}

pub fn decode_legacy_clipboard(input: &str) -> Result<GameState, CodecError> {
    let sections: Vec<&str> = input.split('[').collect();
    let queue_section = sections
        .get(1)
        .map(|section| section.trim())
        .filter(|section| !section.is_empty())
        .ok_or(CodecError::MissingLegacyQueueSection)?;
    let board_section = sections
        .get(2)
        .map(|section| section.trim())
        .filter(|section| !section.is_empty())
        .ok_or(CodecError::MissingLegacyBoardSection)?;

    let (seed, hold_piece, queue) = decode_legacy_queue(queue_section)?;
    let board = decode_legacy_board_text(board_section)?;

    Ok(build_imported_state(board, seed, hold_piece, queue))
}

fn build_imported_state(
    board: Board,
    seed: u16,
    hold_piece: Option<Tetromino>,
    queue: Vec<Tetromino>,
) -> GameState {
    let total_height = board.total_height();

    GameState {
        mode: GameMode::Training,
        highlights: HighlightBoard::new(board.width, total_height),
        current: PieceState {
            kind: Tetromino::I,
            rotation: Rotation::Spawn,
            x: board.spawn_x(),
            y: board.spawn_y(),
        },
        hold: HoldState {
            piece: hold_piece,
            ..Default::default()
        },
        queue: QueueState {
            visible: queue,
            seed,
            ..Default::default()
        },
        stats: Stats::default(),
        spin: SpinState::default(),
        gravity: GravityState::default(),
        ghost_piece: true,
        auto_color: false,
        auto_lock_on_ground: false,
        mirror_queue_with_field: false,
        highlight_clear: false,
        pc_leftover: 0,
        garbage_hole_pos: 0,
        garbage_hole_size: 0,
        garbage_hole_next_change: 0,
        lock_delay: LockDelayState::default(),
        clear_flash: ClearFlashState::default(),
        board,
    }
}

fn decode_legacy_queue(
    input: &str,
) -> Result<(u16, Option<Tetromino>, Vec<Tetromino>), CodecError> {
    let decoded = base64::engine::general_purpose::STANDARD
        .decode(input)
        .map_err(CodecError::InvalidLegacyQueueEncoding)?;
    let mut hex = String::with_capacity(decoded.len() * 2);
    for byte in decoded {
        use std::fmt::Write as _;
        let _ = write!(&mut hex, "{byte:02X}");
    }

    if hex.len() < 5 {
        return Err(CodecError::InvalidLegacyBoardShape(
            "legacy queue payload is too short".to_string(),
        ));
    }

    let seed = u16::from_str_radix(&hex[..4], 16).map_err(|_| {
        CodecError::InvalidLegacyBoardShape("legacy queue seed is not valid hex".to_string())
    })?;
    let hold_nibble = parse_hex_nibble(hex.as_bytes()[4] as char)?;
    let mut queue_nibbles = hex[5..].chars().collect::<Vec<_>>();
    if queue_nibbles.last() == Some(&'E') {
        queue_nibbles.pop();
    }

    let hold_piece = normalize_legacy_piece(hold_nibble);
    let mut queue = Vec::with_capacity(queue_nibbles.len());
    for ch in queue_nibbles {
        if let Some(piece) = normalize_legacy_piece(parse_hex_nibble(ch)?) {
            queue.push(piece);
        }
    }

    Ok((seed, hold_piece, queue))
}

fn decode_legacy_board_text(input: &str) -> Result<Board, CodecError> {
    let normalized = input.replace('\r', "");
    let row_split = if normalized.contains('\n') {
        normalized
            .split('\n')
            .map(str::trim)
            .filter(|row| !row.is_empty())
            .collect::<Vec<_>>()
    } else {
        normalized
            .split('/')
            .map(str::trim)
            .filter(|row| !row.is_empty())
            .collect::<Vec<_>>()
    };

    if row_split.is_empty() {
        return Err(if looks_like_compressed_board(input) {
            CodecError::UnsupportedLegacyBoardEncoding
        } else {
            CodecError::InvalidLegacyBoardShape("legacy board is empty".to_string())
        });
    }

    let width = row_split[0].chars().count();
    if width == 0 {
        return Err(CodecError::InvalidLegacyBoardShape(
            "legacy board rows may not be empty".to_string(),
        ));
    }
    if width > MAX_BOARD_WIDTH {
        return Err(CodecError::InvalidLegacyBoardShape(format!(
            "legacy board width {width} exceeds maximum {MAX_BOARD_WIDTH}"
        )));
    }
    if row_split.len() > MAX_VISIBLE_HEIGHT {
        return Err(CodecError::InvalidLegacyBoardShape(format!(
            "legacy board height {} exceeds maximum {}",
            row_split.len(),
            MAX_VISIBLE_HEIGHT
        )));
    }

    let mut visible_cells = Vec::with_capacity(width * row_split.len());
    for row in &row_split {
        if row.chars().count() != width {
            return Err(CodecError::InvalidLegacyBoardShape(
                "legacy board rows must all have the same width".to_string(),
            ));
        }
        for ch in row.chars() {
            visible_cells.push(parse_legacy_board_cell(ch)?);
        }
    }

    let mut board = Board::new(width, row_split.len(), DEFAULT_HIDDEN_ROWS);
    board.replace_visible_rows_bottom_aligned(&visible_cells, row_split.len());
    Ok(board)
}

fn looks_like_compressed_board(input: &str) -> bool {
    !input.contains('\n')
        && !input.contains('/')
        && input
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '+' | '/' | '='))
}

fn parse_hex_nibble(ch: char) -> Result<u8, CodecError> {
    ch.to_digit(16)
        .map(|value| value as u8)
        .ok_or(CodecError::InvalidLegacyQueueHex(ch))
}

fn normalize_legacy_piece(value: u8) -> Option<Tetromino> {
    match value {
        15 => None,
        0..=7 => tetromino_from_index(value),
        _ => Some(Tetromino::Mono),
    }
}

fn tetromino_from_index(value: u8) -> Option<Tetromino> {
    match value {
        0 => Some(Tetromino::I),
        1 => Some(Tetromino::J),
        2 => Some(Tetromino::S),
        3 => Some(Tetromino::O),
        4 => Some(Tetromino::Z),
        5 => Some(Tetromino::L),
        6 => Some(Tetromino::T),
        7 => Some(Tetromino::Mono),
        _ => None,
    }
}

fn parse_legacy_board_cell(ch: char) -> Result<Cell, CodecError> {
    let upper = ch.to_ascii_uppercase();
    let cell = match upper {
        '.' | '_' | '-' | '0' => Cell::Empty,
        '8' | 'G' | 'M' | 'X' | '#' => Cell::Gray,
        '1' => Cell::I,
        '2' => Cell::J,
        '3' => Cell::S,
        '4' => Cell::O,
        '5' => Cell::Z,
        '6' => Cell::L,
        '7' => Cell::T,
        _ => match piece_from_name(upper) {
            Some(piece) => cell_from_piece(piece),
            None => return Err(CodecError::InvalidLegacyBoardCharacter(ch)),
        },
    };

    Ok(cell)
}
