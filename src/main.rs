use ctrlc;
use pi_game::stt::stream_words;
use std::sync::mpsc::{TryRecvError, channel};
use std::{env, time::Duration};

fn main() {
    let mut args = env::args();
    args.next();

    let model_path = args.next().expect("A model path was not provided");
    let record_duration = Duration::from_millis(50);

    // Setup channel for sending words between the audio processing thread and game thread
    let (words_tx, words_rx) = channel();

    // Setup shutdown handler
    let (shutdown_tx, shutdown_rx) = channel();

    ctrlc::set_handler(move || {
        shutdown_tx
            .send(())
            .expect("Could not send shutdown signal")
    })
    .expect("Error setting up shutdown handler");

    println!("Recording...");

    // Start streaming speech-to-text
    // Need to keep an ownership of the stream or else it will get dropped on function return
    let stream = stream_words(model_path, words_tx);

    // Play game until shutdown signal received
    while let Err(TryRecvError::Empty) = shutdown_rx.try_recv() {
        if let Ok(words) = words_rx.recv_timeout(record_duration) {
            println!("{:?}", words);
        }
    }

    drop(stream);
}
