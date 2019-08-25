use cpal::traits::{DeviceTrait, EventLoopTrait, HostTrait};
use std::io::Write;
use ttspico as pico;

fn main() {
    let sys = pico::System::new(4 * 1024 * 1024).expect("Could not init system");

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

    let mut voice = sys
        .create_voice("TestVoice")
        .expect("Failed to create voice");
    voice
        .add_resource(&ta_res)
        .expect("Failed to add TA to voice");
    voice
        .add_resource(&sg_res)
        .expect("Failed to add SG to voice");

    let mut engine = unsafe { voice.create_engine().expect("Failed to create engine") };

    let mut text_bytes: &[u8] = b"1, 2, 3, Hello Rust!\0"; //< The null terminator tells Pico to start synthesizing!
    while text_bytes.len() > 0 {
        let n_put = engine
            .put_text(text_bytes)
            .expect("pico_putTextUtf8 failed");
        text_bytes = &text_bytes[n_put..];
    }

    let mut pcm_data = vec![0i16; 0];
    let mut pcm_buf = [0i16; 1024];
    'tts: loop {
        let (n_written, status) = engine.get_data(&mut pcm_buf[..]).expect("TTS error");
        pcm_data.extend(&pcm_buf[..n_written]);
        if status == ttspico::EngineStatus::Idle {
            break 'tts;
        }
    }

    audio_out(&*pcm_data);
}

fn audio_out(pcm_data: &[i16]) -> ! {
    let host = cpal::default_host();
    let evt_loop = host.event_loop();
    let dev = host
        .default_output_device()
        .expect("No sound output device");

    // 16-bit PCM, signed, 16kHz
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
    evt_loop.run(move |id, result| {
        let data = result.expect("Error in audio stream");
        match data {
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
