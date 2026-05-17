import sys
import time
import torch
torch.set_num_threads(4)  # 4 threads por instância
torch.set_num_interop_threads(1)

from flask import Flask, request, jsonify
from transformers import MarianMTModel, MarianTokenizer

PORT = int(sys.argv[1]) if len(sys.argv) > 1 else 8084
MODEL = "Helsinki-NLP/opus-mt-tc-big-en-pt"
device = "cuda" if torch.cuda.is_available() else "cpu"
print(f"[porta {PORT}] Carregando modelo no {device} com {torch.get_num_threads()} threads...")

tokenizer = MarianTokenizer.from_pretrained(MODEL)
model = MarianMTModel.from_pretrained(MODEL).to(device)
model.eval()
print(f"[porta {PORT}] Pronto!")

app = Flask(__name__)

@app.route("/translate", methods=["POST"])
def translate():
    texts = request.json.get("texts", [])
    if not texts:
        return jsonify([])
    results = []
    for i in range(0, len(texts), 32):
        chunk = texts[i:i+32]
        inputs = tokenizer(chunk, return_tensors="pt", padding=True, truncation=True, max_length=512).to(device)
        with torch.no_grad():
            translated = model.generate(**inputs, num_beams=1)
        results.extend(tokenizer.batch_decode(translated, skip_special_tokens=True))
    return jsonify(results)

@app.route("/bench", methods=["GET"])
def bench():
    samples = ["Chicken Breast", "Rolled Oats", "Whole Milk", "Brown Rice", "Peanut Butter",
               "Strawberries", "Blueberry Yogurt", "Olive Oil", "Sweet Potatoes", "Ground Turkey"] * 10
    start = time.time()
    inputs = tokenizer(samples, return_tensors="pt", padding=True, truncation=True, max_length=64).to(device)
    with torch.no_grad():
        translated = model.generate(**inputs, num_beams=1)
    results = tokenizer.batch_decode(translated, skip_special_tokens=True)
    elapsed = time.time() - start
    total_tokens = sum(len(tokenizer.encode(t)) for t in results)
    return jsonify({"port": PORT, "texts": len(samples), "tokens_out": total_tokens,
                    "elapsed_s": round(elapsed, 3), "tokens_per_sec": round(total_tokens / elapsed, 1)})

if __name__ == "__main__":
    app.run(host="127.0.0.1", port=PORT, threaded=False)
