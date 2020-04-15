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

use hound;
use std::rc::Rc;
use ttspico as pico;

fn main() {
    // 1. Create a Pico system
    // NOTE: There should at most one System per thread!
    let sys = pico::System::new(4 * 1024 * 1024).expect("Could not init system");

    // 2. Load Text Analysis (TA) and Speech Generation (SG) resources for the voice you want to use
    let ta_res =
        pico::System::load_resource(Rc::clone(&sys), "ttspico-sys/build/pico/lang/en-US_ta.bin")
            .expect("Failed to load TA");
    let sg_res = pico::System::load_resource(
        Rc::clone(&sys),
        "ttspico-sys/build/pico/lang/en-US_lh0_sg.bin",
    )
    .expect("Failed to load SG");
    println!(
        "TA: {}, SG: {}",
        ta_res.borrow().name().unwrap(),
        sg_res.borrow().name().unwrap()
    );

    // 3. Create a Pico voice definition and attach the loaded resources to it
    let voice = pico::System::create_voice(sys, "TestVoice").expect("Failed to create voice");
    voice
        .borrow_mut()
        .add_resource(ta_res)
        .expect("Failed to add TA to voice");
    voice
        .borrow_mut()
        .add_resource(sg_res)
        .expect("Failed to add SG to voice");

    // 4. Create an engine from the voice definition
    // UNSAFE: Creating an engine without attaching the resources will result in a crash!
    let mut engine = unsafe { pico::Voice::create_engine(voice).expect("Failed to create engine") };

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

    let spec = hound::WavSpec {
        channels: 1,
        sample_rate: 16000,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };

    let mut writer = hound::WavWriter::create("speech.wav", spec).unwrap();
    for sample in pcm_data {
        writer.write_sample(sample).unwrap();
    }
}
