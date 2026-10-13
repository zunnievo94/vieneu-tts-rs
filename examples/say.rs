//! Speak Vietnamese text into a WAV file.
//!
//! ```sh
//! cargo run --release --example say -- turbo "Xin chào, mình là VieNeu." out.wav
//! cargo run --release --example say -- nano "Xin chào." out.wav "Trúc Ly"
//! cargo run --release --example say -- turbo --voices
//! ```
//!
//! Models are downloaded into `./models/<id>` the first time.

use std::io::Write;
use std::path::PathBuf;
use std::time::Instant;

use anyhow::{bail, Context, Result};
use vieneu_tts::{Engine, Model};

fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let model = match args.first().map(String::as_str) {
        Some("turbo") => Model::Turbo,
        Some("nano") => Model::Nano,
        _ => {
            bail!("usage: say <turbo|nano> <text> <out.wav> [voice]  |  say <turbo|nano> --voices")
        }
    };
    let dir = PathBuf::from("models").join(model.id());
    if !model.is_complete(&dir) {
        eprintln!(
            "Downloading {} ({} MB) from {} …",
            model.id(),
            model.size() / 1_000_000,
            model.homepage()
        );
        vieneu_tts::download::download(model, &dir, &mut |done, total| {
            eprint!("\r{:>3}%", done * 100 / total.max(1));
            let _ = std::io::stderr().flush();
        })?;
        eprintln!();
    }

    if args.get(1).map(String::as_str) == Some("--voices") {
        for v in vieneu_tts::voices(model, &dir)? {
            println!("{:<16} {}", v.name, v.description);
        }
        return Ok(());
    }
    let text = args.get(1).context("missing text")?;
    let out = args.get(2).context("missing output file")?;
    let voice = args
        .get(3)
        .map(String::as_str)
        .unwrap_or(model.default_voice());

    let t = Instant::now();
    let mut tts = Engine::load(model, &dir, voice)?;
    eprintln!(
        "Loaded {} · {voice} in {:.1} s",
        model.id(),
        t.elapsed().as_secs_f32()
    );

    let t = Instant::now();
    let samples = tts.synthesize(text)?;
    let secs = samples.len() as f32 / tts.sample_rate() as f32;
    eprintln!("{secs:.1} s of audio in {:.1} s", t.elapsed().as_secs_f32());

    let spec = hound::WavSpec {
        channels: 1,
        sample_rate: tts.sample_rate(),
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };
    let mut wav = hound::WavWriter::create(out, spec)?;
    for s in samples {
        wav.write_sample((s.clamp(-1.0, 1.0) * i16::MAX as f32) as i16)?;
    }
    wav.finalize()?;
    eprintln!("Saved {out}");
    Ok(())
}
