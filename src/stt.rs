use cpal::{
    ChannelCount, SampleFormat, Stream,
    traits::{DeviceTrait, HostTrait, StreamTrait},
};
use dasp::{Sample, sample::ToSample};
use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender, TryRecvError, channel};
use std::thread;
use std::time::Duration;
use vosk::{DecodingState, LogLevel, Model, Recognizer, set_log_level};

const GRAMMAR: &str = "start point one two three four five six seven eight nine zero oh";
const STREAM_RECEIVE_TIMEOUT: Duration = Duration::from_millis(50);

pub fn start_stt(
    model_path: String,
    words_tx: Sender<Vec<String>>,
    shutdown_rx: Receiver<()>,
) -> (thread::JoinHandle<()>, Stream) {
    // Setup microphone
    let audio_input_device = cpal::default_host()
        .default_input_device()
        .expect("No input device connected");

    let config = audio_input_device
        .default_input_config()
        .expect("Failed to load default input config");
    let channels = config.channels();

    #[cfg(debug_assertions)]
    println!(
        "Default input device {:?}",
        audio_input_device
            .name()
            .expect("No input device name found")
    );
    #[cfg(debug_assertions)]
    println!("Default input config: {:?}", config);

    // Setup speech-to-text model
    set_log_level(LogLevel::Error); // make vosk output less verbose
    let model = Model::new(model_path).expect("Could not create the model");
    let mut recognizer =
        Recognizer::new_with_grammar(&model, config.sample_rate().0 as f32, &[GRAMMAR, "[unk]"])
            .expect("Could not create the Recognizer");
    recognizer.set_words(true);
    recognizer.set_partial_words(true);

    let (stream_tx, stream_rx) = channel();
    let handle = thread::spawn(|| {
        recognize_thead(recognizer, stream_rx, words_tx, shutdown_rx); // Call your function in the new thread
    });

    let err_fn = move |err| {
        eprintln!("an error occurred on stream: {}", err);
    };

    let stream = match config.sample_format() {
        SampleFormat::I8 => audio_input_device.build_input_stream(
            &config.into(),
            move |data: &[i8], _| handle_mic_data(data, channels, &stream_tx),
            err_fn,
            None,
        ),
        SampleFormat::I16 => audio_input_device.build_input_stream(
            &config.into(),
            move |data: &[i16], _| handle_mic_data(data, channels, &stream_tx),
            err_fn,
            None,
        ),
        SampleFormat::I32 => audio_input_device.build_input_stream(
            &config.into(),
            move |data: &[i32], _| handle_mic_data(data, channels, &stream_tx),
            err_fn,
            None,
        ),
        SampleFormat::F32 => audio_input_device.build_input_stream(
            &config.into(),
            move |data: &[f32], _| handle_mic_data(data, channels, &stream_tx),
            err_fn,
            None,
        ),
        sample_format => panic!("Unsupported sample format '{sample_format}'"),
    }
    .expect("Could not build stream");

    stream.play().expect("Could not play stream");
    return (handle, stream);
}

/// Handles incoming audio and sends it to STT thread to process into words
fn handle_mic_data<T: Sample + ToSample<i16>>(
    data: &[T],
    channels: ChannelCount,
    steam_tx: &mpsc::Sender<Vec<i16>>,
) {
    let data: Vec<i16> = data.iter().map(|v| v.to_sample()).collect();
    let data = if channels != 1 {
        stereo_to_mono(&data)
    } else {
        data
    };
    let _ = steam_tx.send(data);
}

// STT thread that processes microphone sampled data into words
fn recognize_thead(
    mut recognizer: Recognizer,
    stream_rx: mpsc::Receiver<Vec<i16>>,
    words_tx: mpsc::Sender<Vec<String>>,
    shutdown_rx: Receiver<()>,
) {
    let mut prev_input: String = Default::default();
    while let Err(TryRecvError::Empty) = shutdown_rx.try_recv() {
        if let Ok(data) = stream_rx.recv_timeout(STREAM_RECEIVE_TIMEOUT) {
            let state = recognizer.accept_waveform(&data).unwrap();
            let possible_new_str: Option<&str> = match state {
                DecodingState::Running => {
                    let partial_result = recognizer.partial_result().partial;
                    if prev_input != partial_result {
                        Some(partial_result)
                    } else {
                        None
                    }
                }
                DecodingState::Finalized => {
                    let result = recognizer
                        .result()
                        .single()
                        .expect("Max alternatives set to 0")
                        .text;
                    if prev_input != result {
                        Some(result)
                    } else {
                        None
                    }
                }
                DecodingState::Failed => None,
            };
            if let Some(new_str) = possible_new_str {
                if !new_str.is_empty() {
                    let new_words: Vec<String> = new_str
                        .replace(prev_input.as_str(), "")
                        .trim()
                        .split(" ")
                        .map(|s| s.trim().to_string())
                        .collect();
                    if !new_words.is_empty() {
                        words_tx
                            .send(new_words)
                            .expect("error sending words between threads");
                    }
                }
                prev_input = new_str.to_string();
            }
        }
    }
}

fn stereo_to_mono(input_data: &[i16]) -> Vec<i16> {
    let mut result = Vec::with_capacity(input_data.len() / 2);
    result.extend(
        input_data
            .chunks_exact(2)
            .map(|chunk| chunk[0] / 2 + chunk[1] / 2),
    );
    result
}
