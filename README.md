# vieneu-tts-rs

Vietnamese text-to-speech with **[VieNeu-TTS](https://github.com/pnnbao97/VieNeu-TTS)
v3** (Turbo and Nano) in **Rust, without Python**.

It came out of work on [Marginalia](https://github.com/zunnievo94/Marginalia),
an open-source ebook reader that reads books aloud, while looking for
natural Vietnamese voices that run without Python. It lives in its own
repository in case other Rust programs can use it too.

> ### Credit belongs to the VieNeu-TTS authors
>
> The models, their training data, architecture and the reference
> implementation are the work of **Phạm Nguyễn Ngọc Bảo**
> ([@pnnbao97](https://github.com/pnnbao97)) and the **VieNeu-TTS** project.
> The audio codec is
> [MOSS Audio Tokenizer Nano](https://huggingface.co/OpenMOSS-Team/MOSS-Audio-Tokenizer-Nano-ONNX)
> by **OpenMOSS-Team**. Text normalization and phonemes come from
> [sea-g2p](https://github.com/pnnbao97/sea-g2p), also by Phạm Nguyễn Ngọc Bảo.
>
> This repository **only re-implements the inference code in Rust**,
> following VieNeu-TTS's Python code, so it can run in a Rust application
> with no Python installed. Everything that makes the voices good is the
> VieNeu team's.
>
> - Source: <https://github.com/pnnbao97/VieNeu-TTS>
> - v3 Turbo model: <https://huggingface.co/pnnbao-ump/VieNeu-TTS-v3-Turbo>
> - v3 Nano model: <https://huggingface.co/pnnbao-ump/VieNeu-TTS-v3-Nano>
> - VieNeu Discord: <https://discord.gg/yJt8kzjzWZ>
>
> No model is bundled or re-hosted here: the crate downloads every file
> from the authors' own Hugging Face and GitHub repositories, pinned to
> the commits it was verified against.

## What it does

| Model | Download | Voice | Speed on 4 CPU cores |
| --- | --- | --- | --- |
| **v3 Turbo** | 583 MB | the most natural; 48 kHz, 25 preset voices | about real time |
| **v3 Nano** | 347 MB | 24 kHz, 11 preset voices, for slower machines | about 0.6 × real time |

(Sizes include the 63 MB sea-g2p dictionary.)

- Phonemes come from sea-g2p's Rust crate; the Turbo tokenizer runs on
  `tokenizers`.
- The ONNX graphs run in ONNX Runtime through [`ort`](https://github.com/pykeio/ort).
- The NumPy glue of VieNeu-TTS is ported: embeddings and the speaker
  projection, top-k/top-p sampling with a windowed repetition penalty, the
  frame cap and the guard against short chunks that babble, and Nano's
  Euler steps with classifier-free guidance.
- Not ported: voice cloning from a reference clip, frame-level streaming,
  and the audio watermark (optional in the Python package, off by default).

## Try it

```sh
cargo run --release --example say -- turbo "Xin chào, mình là VieNeu." out.wav
cargo run --release --example say -- turbo "Xin chào." out.wav "Ngọc Huyền"
cargo run --release --example say -- turbo --voices
cargo run --release --example say -- nano "Xin chào." out.wav
```

The first run downloads the model into `./models/`.

## Use it

```toml
[dependencies]
vieneu-tts = { git = "https://github.com/zunnievo94/vieneu-tts-rs" }
```

```rust
use vieneu_tts::{Engine, Model};

let dir = std::path::Path::new("models/vieneu-tts-v3-turbo");
vieneu_tts::download::download(Model::Turbo, dir, &mut |done, total| {})?;
let mut tts = Engine::load(Model::Turbo, dir, "Hải Đăng")?;
let samples: Vec<f32> = tts.synthesize("Trăm năm trong cõi người ta.")?; // mono, [-1, 1]
let rate = tts.sample_rate(); // 48 000 (Turbo) or 24 000 (Nano)
```

Pass one sentence or a short paragraph at a time; long sentences are split
at punctuation.

### Features

- `download-ort` (default): `ort` downloads a prebuilt ONNX Runtime at
  build time.
- `download` (default): `download::download` fetches model files.
- `shared-ort`: use an ONNX Runtime the program already links (for example
  through sherpa-onnx). Call `vieneu_tts::use_ort_api_base(OrtGetApiBase())`
  once before loading a model, and turn off the default features. This is
  how Marginalia uses the crate, sharing sherpa-onnx's runtime.

## How it was verified

The port was compared with the official Python package `vieneu` 3.8.3 (its
ONNX Runtime path). Scripts, inputs and outputs are in
[`verification/`](verification/):

| Check | Result |
| --- | --- |
| Phonemes (10 sentences with numbers, dates, e-mail, English words, °C, %) | **identical** to Python sea-g2p |
| Turbo tokenizer ids | **identical** to Python `tokenizers` |
| Turbo with sampling off (greedy) | **the same 17 × 16 audio codes** as Python |
| Nano with the same starting noise | the same waveform: max difference 7·10⁻¹⁰ (200 dB) on the same ONNX Runtime |
| Speech recognition (8 sentences × 3 takes, zipformer-vi) | Turbo 1.9% word errors (Python 3.9%), Nano 4.7% (Python 2.3%) |

The differences in the last row are sampling noise: the deterministic
checks above show the networks run identically, and 24 more takes of the
sentence Nano gets wrong most often scored 4.2% for both.

## License

[Apache-2.0](LICENSE), like VieNeu-TTS and sea-g2p. [NOTICE](NOTICE) says
where each part comes from. The downloaded models keep their own licenses
(all Apache-2.0 at the time of writing). For research use, please cite
VieNeu-TTS:

```bibtex
@misc{vieneutts2026,
  title        = {VieNeu-TTS: Advanced Vietnamese Text-to-Speech with Instant Voice Cloning},
  author       = {Pham Nguyen Ngoc Bao},
  year         = {2026},
  publisher    = {Hugging Face},
  howpublished = {\url{https://huggingface.co/pnnbao-ump/VieNeu-TTS}}
}
```
