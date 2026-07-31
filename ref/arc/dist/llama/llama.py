from transformers import pipeline
from flask import Flask, request, jsonify

app = Flask(__name__)
pipe = pipeline('text-generation', model='meta-llama/Llama-3.2-3B-Instruct')


@app.route('/generate', methods=['POST'])
def generate():
    data = request.json
    text_input = data.get('input', '')
    results = pipe(
        text_input,
        temperature=0.0000000001,
        max_new_tokens=2048
    )
    return jsonify(results)


app.run(host='0.0.0.0', port=5000)
