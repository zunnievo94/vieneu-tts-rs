# Kiểm chứng bản port / Verification

Bản Rust được so với gói Python chính thức **`vieneu` 3.8.3** (đường
ONNX Runtime trên CPU, `sea-g2p` 0.10.0), trên máy 4 nhân, tháng 10/2026.
Model ở đúng các commit mà thư viện ghim (xem `src/turbo.rs`, `src/nano.rs`).

*The Rust port was compared with the official Python package `vieneu`
3.8.3 (CPU / ONNX Runtime path, `sea-g2p` 0.10.0). Results below; every
script and output is in this folder.*

## 1. Phiên âm (sea-g2p) — giống hệt

- Đầu vào: [`g2p_sentences.txt`](g2p_sentences.txt) (số, ngày tháng, email,
  tiếng Anh xen kẽ, °C, %, ngoặc kép).
- Python: `SEAPipeline(lang="vi").run(line, punc_norm=True)` →
  [`results/g2p_python.txt`](results/g2p_python.txt)
- Rust: test `phonemize_lines` →
  [`results/g2p_rust.txt`](results/g2p_rust.txt)
- `diff`: không khác byte nào.

```sh
VIENEU_TEST_DICT=models/vieneu-tts-v3-turbo/sea_g2p.bin VIENEU_TEST_TEXTS=verification/g2p_sentences.txt \
  cargo test --release phonemize_lines -- --ignored --nocapture
```

## 2. Tokenizer của Turbo — giống hệt

Các chuỗi phoneme ở trên qua `tokenizer.json` của Turbo: Python
`tokenizers` → [`results/tokens_python.txt`](results/tokens_python.txt),
Rust (test `tokenize_lines`) → [`results/tokens_rust.txt`](results/tokens_rust.txt).
Giống hệt.

## 3. Turbo, tắt lấy mẫu (greedy) — cùng mã âm thanh

Với `temperature = 0` (vẫn bật phạt lặp), cùng phoneme
([`results/turbo_greedy_phonemes.txt`](results/turbo_greedy_phonemes.txt)),
giọng "Hải Đăng": Python ([`turbo_greedy_python.py`](turbo_greedy_python.py))
và Rust (test `greedy_frames`) sinh ra **cùng 17 khung × 16 mã codec**,
gồm cả khung dừng:
[`results/turbo_greedy_frames_python.txt`](results/turbo_greedy_frames_python.txt) =
[`results/turbo_greedy_frames_rust.txt`](results/turbo_greedy_frames_rust.txt).

Điều này kiểm chứng toàn bộ đường suy luận: dựng prompt, embedding, chiếu
x-vector của giọng, backbone prefill/decode và KV-cache, vòng acoustic 16
codebook, đầu ra và token dừng.

## 4. Nano, cùng nhiễu đầu vào — cùng sóng âm

Python ([`nano_fixed_noise_python.py`](nano_fixed_noise_python.py)) chạy
với `seed=1234` và xuất nhiễu ban đầu; Rust (test `fixed_noise_output`)
dùng đúng nhiễu đó: [`results/nano_fixed_noise.txt`](results/nano_fixed_noise.txt).
Cùng phiên bản ONNX Runtime với Python (1.31): 34 560 mẫu, sai khác lớn
nhất 7·10⁻¹⁰ (200 dB), gần như giống từng bit. Với ONNX Runtime 1.28 (bản
sherpa-onnx link sẵn): 6,5·10⁻⁶ (111,6 dB), chỉ là sai số làm tròn.

## 5. Nhận dạng giọng nói (WER)

8 câu văn ([`sentences.txt`](sentences.txt)) × 3 lần mỗi engine, chấm bằng
model nhận dạng tiếng Việt sherpa-onnx `zipformer-vi-2025-04-20`
([`score_asr.py`](score_asr.py)); mẫu Python tạo bằng
[`python_reference.py`](python_reference.py). Kết quả đầy đủ (kể cả từng
câu sai): [`results/asr_wer.txt`](results/asr_wer.txt).

| Engine | Tỉ lệ từ sai |
| --- | --- |
| Turbo — Python | 3,9% |
| Turbo — Rust | 1,9% |
| Nano — Python | 2,3% |
| Nano — Rust | 4,7% |
| Piper `vi_VN-vais1000-medium` (đối chứng) | 3,9% |

- Lỗi "đi xa dần" → "đi xa" có ở mọi engine, kể cả Piper: lỗi của máy
  nhận dạng.
- Chênh lệch Python/Rust là do lấy mẫu ngẫu nhiên; mục 3 và 4 cho thấy
  mạng chạy giống hệt. Để chắc, câu Nano hay sai nhất ("Chúc bạn đọc sách
  vui vẻ.") được chạy thêm 24 lần mỗi bên: Python 4,2%, Rust 4,2%
  ([`results/asr_nano_24_takes.txt`](results/asr_nano_24_takes.txt)). Bộ
  sinh nhiễu của Rust cũng đã được kiểm tra trên 10⁶ mẫu: trung bình
  −0,001, phương sai 1,000, độ nhọn 2,99, tương quan bậc 1 −0,001 (test
  `dump_normals`).
- Mẫu Rust lúc đó được tạo qua CLI của Marginalia, vốn thêm 1,5 s im lặng
  cuối chương; đoạn im lặng đó được cắt trước khi chấm vì máy nhận dạng
  hay "nghe" ra chữ trong im lặng dài.

## 6. Tốc độ (4 nhân, cùng 8 câu)

| | Rust | Python |
| --- | --- | --- |
| Turbo | 0,87–0,94 × thời gian thực | 1,0 × |
| Nano | 0,58–0,60 × | 0,52 × |

(tính cả thời gian nạp model; số nhỏ hơn là nhanh hơn.)

## Nghe thử

[`samples/`](samples/) — câu 3 ("Buổi sáng hôm ấy, trời mưa lâm thâm…"):
`py_*` từ Python, `rs_*` từ Rust; `rs_turbo_ngoc_huyen.wav` là giọng
"Ngọc Huyền". Âm thanh do model VieNeu-TTS sinh ra (Apache-2.0).
