import sys, re, glob, numpy as np, soundfile as sf, sherpa_onnx, collections
d, cmpdir, sents = sys.argv[1], sys.argv[2], open(sys.argv[3]).read().splitlines()
prefixes = sys.argv[4:]
r = sherpa_onnx.OfflineRecognizer.from_transducer(encoder=f"{d}/encoder-epoch-12-avg-8.onnx", decoder=f"{d}/decoder-epoch-12-avg-8.onnx",
    joiner=f"{d}/joiner-epoch-12-avg-8.onnx", tokens=f"{d}/tokens.txt", num_threads=4)
norm = lambda t: re.sub(r"[^\w\s]", " ", t.lower()).split()
def ed(a, b):
    dp = list(range(len(b)+1))
    for i, x in enumerate(a, 1):
        prev, dp[0] = dp[0], i
        for j, y in enumerate(b, 1):
            prev, dp[j] = dp[j], min(dp[j]+1, dp[j-1]+1, prev+(x!=y))
    return dp[-1]
for pre in prefixes:
    errs = words = 0; bad = []
    for f in sorted(glob.glob(f"{cmpdir}/{pre}_*_*.wav")):
        i = int(f.rsplit("/",1)[1].split("_")[-2])
        w, sr = sf.read(f, dtype="float32")
        s = r.create_stream(); s.accept_waveform(sr, w); r.decode_stream(s)
        ref, hyp = norm(sents[i]), norm(s.result.text)
        e = ed(ref, hyp); errs += e; words += len(ref)
        if e: bad.append(f"  {f.rsplit('/',1)[1]}: {' '.join(hyp)}")
    print(f"{pre}: WER {errs/words:.1%} ({errs}/{words})"); print("\n".join(bad))
