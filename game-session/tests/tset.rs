use game_session_io::*;
use gtest::{Log, ProgramBuilder, System};

const GAME_SESSION_PROGRAM_ID: u64 = 100;
const WORDLE_PROGRAM_ID: u64 = 200;

const USER: u64 = 64;

#[test]
fn test_start_game_without_initialization() {
    let system = System::new();
    system.init_logger();

    let game_session_program =
        ProgramBuilder::from_file("../target/wasm32-unknown-unknown/debug/game_session.opt.wasm")
            .with_id(GAME_SESSION_PROGRAM_ID)
            .build(&system);

    // Attempt to start the game without initialization
    let result = game_session_program.send(USER, GameSessionAction::StartGame);

    // Assert that the operation fails due to lack of initialization
    assert!(result.main_failed());
}

#[test]
fn test_win() {
let system = System::new();
    system.init_logger();

    let game_session_program =
        ProgramBuilder::from_file("../target/wasm32-unknown-unknown/debug/game_session.opt.wasm")
            .with_id(GAME_SESSION_PROGRAM_ID)
            .build(&system);
    let wordle_program =
        ProgramBuilder::from_file("../target/wasm32-unknown-unknown/debug/wordle.opt.wasm")
            .with_id(WORDLE_PROGRAM_ID)
            .build(&system);

    let result = wordle_program.send_bytes(USER, []);
    assert!(!result.main_failed());
    game_session_program.send(USER,GameSessionInit {wordle_program_id: WORDLE_PROGRAM_ID.into(),},);

    game_session_program.send(USER, GameSessionAction::StartGame);

    game_session_program.send(USER,GameSessionAction::CheckWord {word: "horsa".to_string(),},);

    game_session_program.send(USER,GameSessionAction::CheckWord {word: "house".to_string(),},);

    let state: GameSessionState = game_session_program.read_state(()).unwrap();
    println!("{:?}", state);

    assert_eq!(
        state.game_sessions[0].1.session_status,
        SessionStatus::GameOver(GameStatus::Win)
    );
    assert_eq!(state.game_sessions[0].1.tries, 2);
}

#[test]
fn test_invalid_word_input() {
    let system = System::new();
    system.init_logger();

    let game_session_program =
        ProgramBuilder::from_file("../target/wasm32-unknown-unknown/debug/game_session.opt.wasm")
            .with_id(GAME_SESSION_PROGRAM_ID)
            .build(&system);
    let wordle_program =
        ProgramBuilder::from_file("../target/wasm32-unknown-unknown/debug/wordle.opt.wasm")
            .with_id(WORDLE_PROGRAM_ID)
            .build(&system);

    let result = wordle_program.send_bytes(USER, []);
    assert!(!result.main_failed());
    game_session_program.send(
        USER,
        GameSessionInit {
            wordle_program_id: WORDLE_PROGRAM_ID.into(),
        },
    );

    // StartGame success
    game_session_program.send(USER, GameSessionAction::StartGame);

    // Try to send an invalid word (not 5 letters)
    let result = game_session_program.send(
        USER,
        GameSessionAction::CheckWord {
            word: "invalid".to_string(), // Invalid word
        },
    );
    assert!(result.main_failed()); // Assert that the word is rejected
}

#[test]
fn test_lose_exceeded_tries_limit() {
    let system = System::new();
    system.init_logger();

    let game_session_program =
        ProgramBuilder::from_file("../target/wasm32-unknown-unknown/debug/game_session.opt.wasm")
            .with_id(GAME_SESSION_PROGRAM_ID)
            .build(&system);
    let wordle_program =
        ProgramBuilder::from_file("../target/wasm32-unknown-unknown/debug/wordle.opt.wasm")
            .with_id(WORDLE_PROGRAM_ID)
            .build(&system);

    let result = wordle_program.send_bytes(USER, []);
    assert!(!result.main_failed());

    let result = game_session_program.send(
        USER,
        GameSessionInit {
            wordle_program_id: WORDLE_PROGRAM_ID.into(),
        },
    );
    assert!(!result.main_failed());

    // StartGame success
    game_session_program.send(USER, GameSessionAction::StartGame);

    game_session_program.send(USER,GameSessionAction::CheckWord {word: "qqqqq".to_string(),},);
    game_session_program.send(USER,GameSessionAction::CheckWord {word: "qqqqq".to_string(),},);
    game_session_program.send(USER,GameSessionAction::CheckWord {word: "qqqqq".to_string(),},);
    game_session_program.send(USER,GameSessionAction::CheckWord {word: "qqqqq".to_string(),},);
    game_session_program.send(USER,GameSessionAction::CheckWord {word: "qqqqq".to_string(),},);

    let state: GameSessionState = game_session_program.read_state(b"").unwrap();
    println!("{:?}", state);
     assert_eq!(
            state.game_sessions[0].1.session_status,
            SessionStatus::GameOver(GameStatus::Lose)
        );
}

#[test]
#[ignore]
fn test_lose_timeout() {
    let system = System::new();
    system.init_logger();

    let game_session_program =
        ProgramBuilder::from_file("../target/wasm32-unknown-unknown/debug/game_session.opt.wasm")
            .with_id(GAME_SESSION_PROGRAM_ID)
            .build(&system);
    let wordle_program =
        ProgramBuilder::from_file("../target/wasm32-unknown-unknown/debug/wordle.opt.wasm")
            .with_id(WORDLE_PROGRAM_ID)
            .build(&system);

    let result = wordle_program.send_bytes(USER, []);
    assert!(!result.main_failed());
    let result = game_session_program.send(
        USER,
        GameSessionInit {
            wordle_program_id: WORDLE_PROGRAM_ID.into(),
        },
    );
    assert!(!result.main_failed());

    // StartGame success
    let result = game_session_program.send(USER, GameSessionAction::StartGame);
    let log = Log::builder()
        .dest(USER)
        .source(GAME_SESSION_PROGRAM_ID)
        .payload(GameSessionEvent::StartSuccess);
    assert!(!result.main_failed() && result.contains(&log));

    let result = system.spend_blocks(200);
    println!("{:?}", result);
    let _log = Log::builder()
        .dest(USER)
        .source(GAME_SESSION_PROGRAM_ID)
        .payload(GameSessionEvent::GameOver(GameStatus::Lose));
    let state: GameSessionState = game_session_program.read_state(b"").unwrap();
    println!("{:?}", state);
}
