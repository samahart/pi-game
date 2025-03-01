use std::sync::mpsc;

use cpal::{
    ChannelCount, SampleFormat, Stream,
    traits::{DeviceTrait, HostTrait, StreamTrait},
};
use dasp::{Sample, sample::ToSample};
use mpsc::Sender;
use vosk::{DecodingState, Model, Recognizer};

const GRAMMAR: &str = "start point one two three four five six seven eight nine zero oh";

pub fn stream_words(model_path: String, words_tx: Sender<Vec<String>>) -> Stream {
    // Setup microphone
    let audio_input_device = cpal::default_host()
        .default_input_device()
        .expect("No input device connected");

    println!("{:?}", audio_input_device.name());

    let config = audio_input_device
        .default_input_config()
        .expect("Failed to load default input config");
    let channels = config.channels();

    // Setup speech-to-text model
    let model = Model::new(model_path).expect("Could not create the model");
    let mut recognizer =
        Recognizer::new_with_grammar(&model, config.sample_rate().0 as f32, &[GRAMMAR, "[unk]"])
            .expect("Could not create the Recognizer");
    recognizer.set_words(true);
    recognizer.set_partial_words(true);

    let err_fn = move |err| {
        eprintln!("an error occurred on stream: {}", err);
    };

    let mut prev_str: String = Default::default();

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
    return stream;
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

fn stereo_to_mono(input_data: &[i16]) -> Vec<i16> {
    let mut result = Vec::with_capacity(input_data.len() / 2);
    result.extend(
        input_data
            .chunks_exact(2)
            .map(|chunk| chunk[0] / 2 + chunk[1] / 2),
    );

    result
}
