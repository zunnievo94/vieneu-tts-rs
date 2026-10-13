import sys, time, soundfile as sf
from vieneu import Vieneu
mode, out, n = sys.argv[1], sys.argv[2], int(sys.argv[3])
tts = Vieneu(mode="v3nano") if mode == "nano" else Vieneu()
voice = "Minh Quân" if mode == "nano" else "Hải Đăng"
sents = open(sys.argv[4]).read().splitlines()
t0 = time.time(); total = 0
for i, s in enumerate(sents):
    for k in range(n):
        w = tts.infer(s, voice=voice, apply_watermark=False)
        total += len(w) / tts.sample_rate
        sf.write(f"{out}/py_{mode}_{i}_{k}.wav", w, tts.sample_rate)
print(f"{mode}: {time.time()-t0:.1f}s for {total:.1f}s audio", file=sys.stderr)
