import sys, numpy as np
from vieneu import Vieneu
tts = Vieneu(mode="v3nano"); eng = tts.engine
v = tts.get_preset_voice("Minh Quân")
ph = open(sys.argv[1]).read().strip()
wav = eng.infer(ph, v["speaker_emb"], v["style"], seed=1234)
# Recreate the noise the engine drew for that seed.
ids = eng.encode_phones(ph); mask = ids != eng.pad
st = np.asarray(v["style"], np.float32).reshape(1, *np.asarray(v["style"]).shape[-2:])
ctx = eng.s_text.run(None, {"ids": ids, "style": st})[0]
import math
log_s = float(eng.s_dur.run(None, {"ctx": ctx, "ctx_mask": mask, "spk": np.asarray(v["speaker_emb"], np.float32).reshape(1,-1)})[0][0])
T = max(2, int(round(min(math.exp(log_s), 15.0) * eng.fps)))
x = np.random.default_rng(1234).standard_normal((1, 144, T)).astype(np.float32)
x.astype("<f4").tofile(sys.argv[2]); wav.astype("<f4").tofile(sys.argv[3])
print("T", T, "samples", wav.size)
