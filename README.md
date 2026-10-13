# vieneu-tts-rs

Giọng đọc tiếng Việt **[VieNeu-TTS](https://github.com/pnnbao97/VieNeu-TTS)
v3** (Turbo và Nano) chạy bằng **Rust, không cần Python**.

*Vietnamese text-to-speech with VieNeu-TTS v3 in Rust, without Python.
English below.*

> ### Công sức thuộc về tác giả VieNeu-TTS
>
> Toàn bộ model, dữ liệu huấn luyện, kiến trúc và bản cài đặt gốc là công
> trình của **Phạm Nguyễn Ngọc Bảo** ([@pnnbao97](https://github.com/pnnbao97))
> và dự án **VieNeu-TTS**. Codec âm thanh là
> [MOSS Audio Tokenizer Nano](https://huggingface.co/OpenMOSS-Team/MOSS-Audio-Tokenizer-Nano-ONNX)
> của **OpenMOSS-Team**. Phần chuẩn hoá văn bản và phiên âm là
> [sea-g2p](https://github.com/pnnbao97/sea-g2p), cũng của Phạm Nguyễn Ngọc Bảo.
>
> Repo này **chỉ viết lại phần chạy suy luận (inference) bằng Rust**, theo
> đúng code Python của VieNeu-TTS, để dùng được trong ứng dụng Rust mà
> không cần cài Python. Mọi cải tiến về chất lượng giọng đọc là của nhóm
> VieNeu.
>
> - Mã nguồn gốc: <https://github.com/pnnbao97/VieNeu-TTS>
> - Model v3 Turbo: <https://huggingface.co/pnnbao-ump/VieNeu-TTS-v3-Turbo>
> - Model v3 Nano: <https://huggingface.co/pnnbao-ump/VieNeu-TTS-v3-Nano>
> - Discord của VieNeu: <https://discord.gg/yJt8kzjzWZ>
>
> Model **không** được đóng gói hay đăng lại ở đây: thư viện tải trực tiếp
> từ Hugging Face và GitHub của tác giả, ghim theo commit đã kiểm chứng.

## Có gì

| Model | Tải về | Chất lượng | Tốc độ (4 nhân CPU) |
| --- | --- | --- | --- |
| **v3 Turbo** | 583 MB | giọng tự nhiên nhất, 48 kHz, 25 giọng | ~1× thời gian thực |
| **v3 Nano** | 347 MB | 24 kHz, 11 giọng, cho máy yếu hơn | ~0,6× thời gian thực |

(Dung lượng đã gồm từ điển sea-g2p 63 MB.)

- Phiên âm bằng crate Rust của sea-g2p; tokenizer bằng `tokenizers`.
- Đồ thị ONNX chạy bằng ONNX Runtime qua [`ort`](https://github.com/pykeio/ort).
- Phần "keo" NumPy của VieNeu (embedding, lấy mẫu top-k/top-p, phạt lặp
  theo cửa sổ, giới hạn khung, chống "nói thêm" ở câu ngắn, bước Euler +
  classifier-free guidance của Nano) được port sang Rust.
- Chưa có: nhân bản giọng từ file mẫu (voice cloning), streaming theo
  khung, watermark âm thanh (tuỳ chọn trong bản Python, mặc định tắt).

## Dùng thử

```sh
cargo run --release --example say -- turbo "Xin chào, mình là VieNeu." out.wav
cargo run --release --example say -- turbo "Xin chào." out.wav "Ngọc Huyền"
cargo run --release --example say -- turbo --voices     # danh sách giọng
cargo run --release --example say -- nano "Xin chào." out.wav
```

Lần đầu chạy sẽ tải model vào `./models/`.

## Dùng trong code

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
let rate = tts.sample_rate(); // 48 000 (Turbo) / 24 000 (Nano)
```

Nên đưa vào từng câu hoặc đoạn ngắn; câu dài được tự chia theo dấu câu.

### Tính năng (features)

- `download-ort` (mặc định): `ort` tải sẵn ONNX Runtime khi build.
- `download` (mặc định): hàm `download::download` để tải model.
- `shared-ort`: dùng ONNX Runtime mà chương trình đã link sẵn (ví dụ qua
  sherpa-onnx), gọi `vieneu_tts::use_ort_api_base(OrtGetApiBase())` một
  lần trước khi nạp model. Dùng với `default-features = false`.

Ví dụ thực tế: trình đọc sách [Marginalia](https://github.com/zunnievo94/Marginalia)
dùng crate này với `shared-ort`, chung ONNX Runtime với sherpa-onnx.

## Đã kiểm chứng thế nào

Bản port được so trực tiếp với gói Python `vieneu` 3.8.3 (chạy trên ONNX
Runtime). Chi tiết, script và kết quả nằm trong [`verification/`](verification/):

| Kiểm tra | Kết quả |
| --- | --- |
| Phiên âm (10 câu có số, ngày, email, tiếng Anh, °C, %) | **giống hệt** sea-g2p Python |
| Token của tokenizer Turbo | **giống hệt** `tokenizers` Python |
| Turbo, tắt lấy mẫu ngẫu nhiên (greedy) | **cùng 17 × 16 mã âm thanh** với Python |
| Nano, cùng nhiễu đầu vào | cùng sóng âm: sai khác tối đa 7·10⁻¹⁰ (200 dB) với cùng bản ONNX Runtime |
| Nhận dạng giọng nói (8 câu × 3 lần, zipformer-vi) | Turbo 1,9% từ sai (Python 3,9%), Nano 4,7% (Python 2,3%) |

Chênh lệch ở dòng cuối là do lấy mẫu ngẫu nhiên: hai dòng kiểm tra tất định
ở trên cho thấy mạng chạy giống hệt.

## Giấy phép

[Apache-2.0](LICENSE), giống VieNeu-TTS và sea-g2p. Xem [NOTICE](NOTICE)
về nguồn gốc từng phần. Model tải về giữ giấy phép riêng (đều Apache-2.0
tại thời điểm viết). Nếu dùng cho nghiên cứu, hãy trích dẫn VieNeu-TTS:

```bibtex
@misc{vieneutts2026,
  title        = {VieNeu-TTS: Advanced Vietnamese Text-to-Speech with Instant Voice Cloning},
  author       = {Pham Nguyen Ngoc Bao},
  year         = {2026},
  publisher    = {Hugging Face},
  howpublished = {\url{https://huggingface.co/pnnbao-ump/VieNeu-TTS}}
}
```

---

## English

A Rust port of the inference code of
[VieNeu-TTS](https://github.com/pnnbao97/VieNeu-TTS) v3 Turbo and Nano, the
Vietnamese text-to-speech models by **Phạm Nguyễn Ngọc Bảo**. All models,
training and the reference implementation are the VieNeu-TTS authors'
work (the audio codec is OpenMOSS-Team's MOSS Audio Tokenizer Nano, the
phonemizer is [sea-g2p](https://github.com/pnnbao97/sea-g2p)); this crate
only re-implements inference in Rust so it can run without Python.
Models are downloaded from the authors' Hugging Face repositories, pinned
to the verified commits, and are not redistributed here.

The port reproduces the Python package exactly where it is deterministic:
the same phonemes and tokens, the same audio codes from Turbo with
sampling off, and the same Nano waveform for the same starting noise. See
[`verification/`](verification/).
