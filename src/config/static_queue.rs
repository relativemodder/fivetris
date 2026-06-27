use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use crate::core::piece::Tetromino;
use crate::resources;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StaticQueueError {
    InvalidPiece(char),
}

pub fn default_path() -> PathBuf {
    resources::data_dir().join("piece_list.txt")
}

pub fn load() -> Result<Option<Vec<Tetromino>>, io::Error> {
    let path = default_path();
    match fs::read_to_string(&path) {
        Ok(raw) => parse_static_queue(&raw)
            .map(Some)
            .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, format!("{error:?}"))),
        Err(error) if error.kind() == io::ErrorKind::NotFound => {
            parse_static_queue(resources::BUILTIN_PIECE_LIST)
                .map(Some)
                .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, format!("{error:?}")))
        }
        Err(error) => Err(error),
    }
}

pub fn load_from_path(path: &Path) -> Result<Option<Vec<Tetromino>>, io::Error> {
    match fs::read_to_string(path) {
        Ok(raw) => parse_static_queue(&raw)
            .map(Some)
            .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, format!("{error:?}"))),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(None),
        Err(error) => Err(error),
    }
}

pub fn parse_static_queue(raw: &str) -> Result<Vec<Tetromino>, StaticQueueError> {
    let mut pieces = Vec::new();

    for ch in raw.chars().filter(|ch| !ch.is_whitespace()) {
        pieces.push(parse_piece(ch)?);
    }

    Ok(pieces)
}

fn parse_piece(ch: char) -> Result<Tetromino, StaticQueueError> {
    match ch.to_ascii_uppercase() {
        'I' => Ok(Tetromino::I),
        'J' => Ok(Tetromino::J),
        'S' => Ok(Tetromino::S),
        'O' => Ok(Tetromino::O),
        'Z' => Ok(Tetromino::Z),
        'L' => Ok(Tetromino::L),
        'T' => Ok(Tetromino::T),
        other => Err(StaticQueueError::InvalidPiece(other)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_static_queue_collects_all_non_whitespace_pieces() {
        let parsed = parse_static_queue("ILSOTZJ\nIIIIIII\nOTOTOTO\n").expect("parse queue");

        assert_eq!(parsed.len(), 21);
        assert_eq!(parsed[0], Tetromino::I);
        assert_eq!(parsed[6], Tetromino::J);
        assert_eq!(parsed[7], Tetromino::I);
        assert_eq!(parsed[14], Tetromino::O);
        assert_eq!(parsed[20], Tetromino::O);
    }

    #[test]
    fn parse_static_queue_rejects_invalid_piece() {
        let error = parse_static_queue("ITX").expect_err("invalid piece should fail");

        assert_eq!(error, StaticQueueError::InvalidPiece('X'));
    }

    #[test]
    fn parse_static_queue_allows_empty_content() {
        let parsed = parse_static_queue("  \n\t").expect("empty content should parse");

        assert!(parsed.is_empty());
    }
}
