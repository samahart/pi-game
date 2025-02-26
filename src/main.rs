use cpal::{
    ChannelCount, SampleFormat,
    traits::{DeviceTrait, HostTrait, StreamTrait},
};
use ctrlc;
use dasp::{Sample, sample::ToSample};
use std::sync::mpsc::{self, TryRecvError, channel};
use std::{env, time::Duration};
use vosk::{DecodingState, Model, Recognizer};

const GRAMMER: &str = "start point one two three four five six seven eight nine zero"; // TODO: add "oh" as alternative to zero

fn main() {
    let mut args = env::args();
    args.next();

    let model_path = args.next().expect("A model path was not provided");
    let record_duration = Duration::from_millis(50);

    let audio_input_device = cpal::default_host()
        .default_input_device()
        .expect("No input device connected");

    println!("{:?}", audio_input_device.name());

    let config = audio_input_device
        .default_input_config()
        .expect("Failed to load default input config");
    let channels = config.channels();

    let model = Model::new(model_path).expect("Could not create the model");
    let mut recognizer =
        Recognizer::new_with_grammar(&model, config.sample_rate().0 as f32, &[GRAMMER, "[unk]"])
            .expect("Could not create the Recognizer");

    recognizer.set_words(true);
    recognizer.set_partial_words(true);

    let mut prev_str: String = Default::default();

    let err_fn = move |err| {
        eprintln!("an error occurred on stream: {}", err);
    };

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
    let stream = match config.sample_format() {
        SampleFormat::I8 => audio_input_device.build_input_stream(
            &config.into(),
            move |data: &[i8], _| {
                recognize(&mut recognizer, data, channels, &words_tx, &mut prev_str)
            },
            err_fn,
            None,
        ),
        SampleFormat::I16 => audio_input_device.build_input_stream(
            &config.into(),
            move |data: &[i16], _| {
                recognize(&mut recognizer, data, channels, &words_tx, &mut prev_str)
            },
            err_fn,
            None,
        ),
        SampleFormat::I32 => audio_input_device.build_input_stream(
            &config.into(),
            move |data: &[i32], _| {
                recognize(&mut recognizer, data, channels, &words_tx, &mut prev_str)
            },
            err_fn,
            None,
        ),
        SampleFormat::F32 => audio_input_device.build_input_stream(
            &config.into(),
            move |data: &[f32], _| {
                recognize(&mut recognizer, data, channels, &words_tx, &mut prev_str)
            },
            err_fn,
            None,
        ),
        sample_format => panic!("Unsupported sample format '{sample_format}'"),
    }
    .expect("Could not build stream");

    stream.play().expect("Could not play stream");
    println!("Recording...");
    // Play game until shutdown signal received
    while let Err(TryRecvError::Empty) = shutdown_rx.try_recv() {
        if let Ok(words) = words_rx.recv_timeout(record_duration) {
            println!("{:?}", words);
        }
    }
    drop(stream);
}

/// Handles incoming audio and processes it into words
fn recognize<T: Sample + ToSample<i16>>(
    recognizer: &mut Recognizer,
    data: &[T],
    channels: ChannelCount,
    words_tx: &mpsc::Sender<Vec<String>>,
    prev_input: &mut String,
) {
    let data: Vec<i16> = data.iter().map(|v| v.to_sample()).collect();
    let data = if channels != 1 {
        stereo_to_mono(&data)
    } else {
        data
    };

    let state = recognizer.accept_waveform(&data).unwrap();
    let possible_new_str: Option<&str> = match state {
        DecodingState::Running => {
            // println!("partial: {:#?}", recognizer.partial_result());
            let partial_result = recognizer.partial_result().partial;
            if prev_input != partial_result {
                Some(partial_result)
            } else {
                None
            }
        }
        DecodingState::Finalized => {
            // println!("result: {:#?}", recognizer.result());
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
        *prev_input = new_str.to_string();
    }
}

pub fn stereo_to_mono(input_data: &[i16]) -> Vec<i16> {
    let mut result = Vec::with_capacity(input_data.len() / 2);
    result.extend(
        input_data
            .chunks_exact(2)
            .map(|chunk| chunk[0] / 2 + chunk[1] / 2),
    );

    result
}
