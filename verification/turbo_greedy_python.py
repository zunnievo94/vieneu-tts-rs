import sys, numpy as np
from vieneu import Vieneu
tts = Vieneu()
eng = tts.engine
cap = {}
orig = eng._decode_codes
def grab(c):
    cap['c'] = c; return orig(c)
eng._decode_codes = grab
spk, codes = tts._resolve_ref("Hải Đăng", None, True, True)
eng.babble_retries = 0
eng.infer(phonemes=open(sys.argv[1]).read().strip(), speaker_emb=spk, ref_codes=codes, temperature=0.0, max_new_frames=300, frame_cap=False)
for f in cap['c']: print("FRAME\t" + " ".join(map(str, f)))
