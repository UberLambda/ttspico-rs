//! Uses Pico TTS to speak a phrase (via [`cpal`]).

// The MIT License
//
// Copyright (c) 2019 Paolo Jovon <paolo.jovon@gmail.com>
//
// Permission is hereby granted, free of charge, to any person obtaining a copy
// of this software and associated documentation files (the "Software"), to deal
// in the Software without restriction, including without limitation the rights
// to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
// copies of the Software, and to permit persons to whom the Software is
// furnished to do so, subject to the following conditions:
//
// The above copyright notice and this permission notice shall be included in
// all copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN
// THE SOFTWARE.

use cpal::traits::{EventLoopTrait, HostTrait};
use ttspico as pico;

fn main() {
    // 1. Create a Pico system
    // NOTE: There should at most one System per thread!
    let sys = pico::System::new(4 * 1024 * 1024).expect("Could not init system");

    // 2. Load Text Analysis (TA) and Speech Generation (SG) resources for the voice you want to use
    let ta_res = sys
        .load_resource("ttspico-sys/build/pico/lang/en-US_ta.bin")
        .expect("Failed to load TA");
    let sg_res = sys
        .load_resource("ttspico-sys/build/pico/lang/en-US_lh0_sg.bin")
        .expect("Failed to load SG");
    println!(
        "TA: {}, SG: {}",
        ta_res.name().unwrap(),
        sg_res.name().unwrap()
    );

    // 3. Create a Pico voice definition and attach the loaded resources to it
    let mut voice = sys
        .create_voice("TestVoice")
        .expect("Failed to create voice");
    voice
        .add_resource(&ta_res)
        .expect("Failed to add TA to voice");
    voice
        .add_resource(&sg_res)
        .expect("Failed to add SG to voice");

    // 4. Create an engine from the voice definition
    // UNSAFE: Creating an engine without attaching the resources will result in a crash!
    let mut engine = unsafe { voice.create_engine().expect("Failed to create engine") };

    // 5. Put (UTF-8) text to be spoken into the engine
    // See `Engine::put_text()` for more details.
    let mut text_bytes: &[u8] = b"1, 2, 3, Hello Rust!\0"; //< The null terminator tells Pico to start synthesizing!
    while text_bytes.len() > 0 {
        let n_put = engine
            .put_text(text_bytes)
            .expect("pico_putTextUtf8 failed");
        text_bytes = &text_bytes[n_put..];
    }

    // 6. Do the actual text-to-speech, getting audio data (16-bit signed PCM @ 16kHz) from the input text
    // Speech audio is computed in small chunks, one "step" at a time; see `Engine::get_data()` for more details.
    let mut pcm_data = vec![0i16; 0];
    let mut pcm_buf = [0i16; 1024];
    'tts: loop {
        let (n_written, status) = engine
            .get_data(&mut pcm_buf[..])
            .expect("pico_getData error");
        pcm_data.extend(&pcm_buf[..n_written]);
        if status == ttspico::EngineStatus::Idle {
            break 'tts;
        }
    }

    audio_out(&*pcm_data);
}

/// Plays an audio buffer (16-bit signed PCM @ 16kHz) to the system default output device.
/// Exits the current process when done.
fn audio_out(pcm_data: &[i16]) -> ! {
    let host = cpal::default_host();
    let evt_loop = host.event_loop();
    let dev = host
        .default_output_device()
        .expect("No sound output device");

    let format = cpal::Format {
        channels: 1,
        sample_rate: cpal::SampleRate(16_000),
        data_type: cpal::SampleFormat::I16,
    };

    let stream_id = evt_loop
        .build_output_stream(&dev, &format)
        .expect("Failed to open audio stream");
    evt_loop
        .play_stream(stream_id.clone())
        .expect("Failed to play audio stream");

    let mut rem = pcm_data;
    evt_loop.run(move |_id, result| {
        let out_data = result.expect("Error in audio stream");
        match out_data {
            cpal::StreamData::Output {
                buffer: cpal::UnknownTypeOutputBuffer::I16(mut u16_buf),
            } => {
                let n_to_copy = std::cmp::min(u16_buf.len(), rem.len());
                u16_buf[..n_to_copy].copy_from_slice(&rem[..n_to_copy]);
                rem = &rem[n_to_copy..];
                if rem.is_empty() {
                    std::process::exit(0);
                }
            }
            _ => panic!("Invalid audio stream format"),
        }
    });
}
