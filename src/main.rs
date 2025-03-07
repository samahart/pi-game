use ctrlc;
use pi_game::game::Game;
use pi_game::stt::start_stt;
use std::sync::mpsc::{TryRecvError, channel};
use std::{env, time::Duration};

fn main() {
    let mut args = env::args();
    args.next();
    let model_path = args.next().expect("A model path was not provided");

    // How long to wait for new words before checking for end of game
    let record_duration = Duration::from_millis(50);

    // Setup channel for sending words between the audio processing thread and game thread
    let (words_tx, words_rx) = channel();

    // Setup shutdown handler
    let (main_shutdown_tx, main_shutdown_rx) = channel();
    let (thread_shutdown_tx, thread_shutdown_rx) = channel();
    ctrlc::set_handler(move || {
        main_shutdown_tx
            .send(())
            .expect("Could not send shutdown signal");
        thread_shutdown_tx
            .send(())
            .expect("Could not send shutdown signal");
    })
    .expect("Error setting up shutdown handler");

    // Note: need to keep ownership of the stream here or else it will get dropped on function return
    let (handle, stream) = start_stt(model_path, words_tx, thread_shutdown_rx);

    // Play game until shutdown signal received
    let mut game = Game::new();
    while let Err(TryRecvError::Empty) = main_shutdown_rx.try_recv() {
        if let Ok(words) = words_rx.recv_timeout(record_duration) {
            game.play(words);
        }
    }
    handle.join().expect("Error joining STT thread");
    drop(stream);
}
