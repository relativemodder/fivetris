use base64::Engine;
use fivetris::core::piece::PieceState;
use fivetris::core::{
    BagMode, Board, Cell, ClearFlashState, GameMode, GameState, GravityState, HighlightBoard,
    HoldState, LockDelayState, QueueState, Rotation, SpinState, Stats, Tetromino,
    decode_json_state, decode_legacy_clipboard, encode_json_state,
};

fn sample_game_state() -> GameState {
    let mut board = Board::new(4, 3, 1);
    board.set(0, 1, Cell::I);
    board.set(3, 3, Cell::Gray);

    GameState {
        mode: GameMode::Training,
        highlights: HighlightBoard::new(4, 4),
        current: PieceState {
            kind: Tetromino::T,
            rotation: Rotation::Right,
            x: 1,
            y: 0,
        },
        hold: HoldState {
            piece: Some(Tetromino::O),
            swapped_this_turn: true,
            infinite_hold: false,
        },
        queue: QueueState {
            visible: vec![Tetromino::I, Tetromino::J, Tetromino::Mono],
            seed: 99,
            mode: BagMode::Random,
        },
        stats: Stats {
            damage: 7,
            lines: 4,
            moves: 12,
            b2b: true,
            perfect_clear: false,
            combo: 3,
            lost: false,
            attack_text: Some("Tetris".to_string()),
            b2b_text: None,
        },
        spin: SpinState {
            is_t_spin: true,
            is_mini: false,
            last_kick_index: Some(2),
        },
        gravity: GravityState {
            gravity_hz: Some(1),
            arr_ms: 10,
            das_ms: 120,
            sdd_ms: 15,
            sds_ms: 20,
            das_cancel: true,
        },
        ghost_piece: false,
        auto_color: true,
        auto_lock_on_ground: true,
        mirror_queue_with_field: true,
        highlight_clear: true,
        pc_leftover: 2,
        garbage_hole_pos: 3,
        garbage_hole_size: 1,
        garbage_hole_next_change: 5,
        lock_delay: LockDelayState {
            active: true,
            timer_ms: 80,
            moves: 4,
            max_moves: 15,
            delay_ms: 500,
        },
        clear_flash: ClearFlashState {
            active: true,
            lines: vec![2],
            timer_ms: 40,
            duration_ms: 300,
        },
        board,
    }
}

#[test]
fn json_state_round_trip() {
    let game = sample_game_state();
    let encoded = encode_json_state(&game).expect("json encode should succeed");
    let decoded = decode_json_state(&encoded).expect("json decode should succeed");

    assert_eq!(decoded, game);
}

#[test]
fn legacy_queue_decode_normalizes_sentinels_and_imports_board_text() {
    let queue_hex = "1234F0FE";
    let queue_bytes = queue_hex
        .as_bytes()
        .chunks_exact(2)
        .map(|pair| {
            u8::from_str_radix(std::str::from_utf8(pair).expect("hex is utf8"), 16)
                .expect("hex should parse")
        })
        .collect::<Vec<_>>();
    let queue_base64 = base64::engine::general_purpose::STANDARD.encode(queue_bytes);

    let imported = decode_legacy_clipboard(&format!("comment[{queue_base64}[I./.8"))
        .expect("legacy decode should succeed");

    assert_eq!(imported.queue.seed, 0x1234);
    assert_eq!(imported.hold.piece, None);
    assert_eq!(imported.queue.visible, vec![Tetromino::I]);
    assert_eq!(imported.board.width, 2);
    assert_eq!(imported.board.visible_height, 2);
    assert_eq!(imported.board.hidden_rows, 4);
    assert_eq!(imported.board.get(0, 4), Cell::I);
    assert_eq!(imported.board.get(1, 4), Cell::Empty);
    assert_eq!(imported.board.get(0, 5), Cell::Empty);
    assert_eq!(imported.board.get(1, 5), Cell::Gray);
}

#[test]
fn legacy_decode_rejects_invalid_input() {
    let error = decode_legacy_clipboard("[not-base64[??")
        .expect_err("invalid legacy clipboard should fail");

    let message = error.to_string();
    assert!(
        message.contains("base64") || message.contains("board"),
        "unexpected error message: {message}"
    );
}
