# dooai-engine

Document AI for Odoo, written in Rust and served over HTTP. It runs the models on your own hardware, so documents
never leave your servers. The first capability is OCR; extraction, classification and embeddings come next.

Nothing in it is specific to a country: it reads text, it does not know any tax authority.

## Quick start

Everything runs in Docker through `./dev.sh` and `make`; Rust does not have to be installed.

```sh
make models      # downloads the PP-OCRv6 small models into models/small (~31 MB)
make image       # builds the server image
docker run --rm -p 8080:8080 -v "$PWD/models/small":/models:ro dooai-engine

curl -X POST --data-binary @crates/engine-ocr/tests/fixtures/invoice.png http://localhost:8080/v1/ocr
```

## API

| Method | Path | Body | Answer |
|---|---|---|---|
| `GET` | `/health` | | `{"status": "ok"}` |
| `POST` | `/v1/ocr` | the encoded image (PNG, JPEG, WebP, TIFF, BMP) | the document, below |

```json
{
  "text": "INVOICE 2026-0042\nSubtotal\tUSD 100.00\nTotal\tUSD 118.00",
  "confidence": 0.998,
  "lines": [{ "text": "INVOICE 2026-0042", "confidence": 0.999,
              "bounding_box": { "left": 38.1, "top": 40.3, "right": 440.9, "bottom": 82.8 } }]
}
```

`text` is in reading order: one line per visual row, and the pieces of a row (table columns) separated by a tab.
An image that cannot be decoded answers `422 {"error": "..."}`.

Configuration, all optional:

| Variable | Default | Meaning |
|---|---|---|
| `ENGINE_ADDRESS` | `0.0.0.0:8080` | Where the server listens |
| `ENGINE_MODELS_DIR` | `models/small` | Directory with the OCR models |
| `ENGINE_MAX_UPLOAD_BYTES` | `20971520` | Largest accepted image |

The server has no authentication: put it behind a gateway that owns API keys, quotas and metering.

## Layout

| Crate | Responsibility |
|---|---|
| `engine-core` | Shared types (`OcrDocument`, `BoundingBox`...), the `OcrEngine` trait and `EngineError` |
| `engine-ocr` | PP-OCR on ONNX Runtime: text detection, line recognition and the pipeline that joins them |
| `engine-server` | The HTTP API (axum) |

## Development

```sh
make test    # unit tests; add `make models` first to also run the real-model OCR test
make lint    # clippy, any warning fails
make fmt     # format
make ci      # format check, clippy and tests
```

Code style: types own behaviour (no free functions), one responsibility per file, descriptive names (no single-letter
variables, no leading underscores), and every fixed value lives in its crate's `settings.rs`.

## Models

The code does not ship weights. `make models` downloads [PP-OCRv6](https://huggingface.co/PaddlePaddle) from Hugging Face
(PaddlePaddle, Apache-2.0). Pick the tier with `MODEL_SIZE=tiny|small|medium`.

## Known limits

- Text boxes are axis-aligned: rotated or curved text is read worse than horizontal text.
- Lines are recognized one by one, and the model session serves one request at a time.
