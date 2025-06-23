use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use pitch_detection::detector::PitchDetector;
use pitch_detection::detector::yin::YINDetector;
use std::io::Write;

fn main() -> Result<(), anyhow::Error> {
    let host = cpal::default_host();
    let device = host
        .default_input_device()
        .ok_or_else(|| anyhow::anyhow!("No input device available"))?;
    let config = device.default_input_config()?;

    println!("Default input device: {}", device.name()?);
    println!("Using sample rate: {}", config.sample_rate().0);
    println!("Listening... Please make a sound into the microphone.");

    let sample_rate = config.sample_rate().0;
    let stream = match config.sample_format() {
        cpal::SampleFormat::F32 => device.build_input_stream(
            &config.into(),
            move |data: &[f32], _: &cpal::InputCallbackInfo| {
                process_input(data, sample_rate);
            },
            |err| eprintln!("an error occurred on stream: {}", err),
            None,
        )?,
        _ => {
            return Err(anyhow::anyhow!("Unsupported sample format"));
        }
    };

    stream.play()?;

    std::thread::sleep(std::time::Duration::from_secs(600));

    Ok(())
}

/// Process the input audio data and detect pitch
fn process_input(data: &[f32], sample_rate: u32) {
    const POWER_THRESHOLD: f32 = 0.1;
    const CLARITY_THRESHOLD: f32 = 0.7;

    let size = data.len();
    let mut detector = YINDetector::new(size, size / 2);

    let pitch = detector.get_pitch(
        data,
        sample_rate as usize,
        POWER_THRESHOLD,
        CLARITY_THRESHOLD,
    );

    match pitch {
        Some(pitch) => {
            let (note, offset) = frequency_to_note(pitch.frequency);
            print!(
                "\rNote: {:<4} | Offset: {:>+5.1} cents | Freq: {:>7.2} Hz | Clarity: {:.2}   ",
                note, offset, pitch.frequency, pitch.clarity
            );
            std::io::stdout().flush().unwrap();
        }
        None => {}
    }
}

/// Convert frequency to note name and cents offset
fn frequency_to_note(freq: f32) -> (String, f32) {
    let a4_freq = 440.0;
    let a4_midi = 69.0;
    let note_names = [
        "C", "C#", "D", "D#", "E", "F", "F#", "G", "G#", "A", "A#", "B",
    ];

    // Calculate the MIDI note number for the given frequency
    let midi_note_float = 12.0 * (freq / a4_freq).log2() + a4_midi;

    // Round to the nearest MIDI note
    let midi_note = midi_note_float.round() as i32;

    // Ensure MIDI note is within the valid range
    let standard_freq = a4_freq * 2.0f32.powf((midi_note as f32 - a4_midi) / 12.0);

    // Calculate the cents offset from the standard frequency
    let cents_offset = 1200.0 * (freq / standard_freq).log2();

    let octave = (midi_note / 12) - 1;
    let note_index = (midi_note % 12) as usize;
    let note_name = note_names[note_index];

    let note_with_octave = format!("{}{}", note_name, octave);

    (note_with_octave, cents_offset)
}
