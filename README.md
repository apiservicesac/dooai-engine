# dooai-engine

Document AI for Odoo, written in Rust and served over HTTP. It runs the models on your own hardware, so documents
never leave your servers. The first capability is OCR; extraction, classification and embeddings come next.

Nothing in it is specific to a country: it reads text, it does not know any tax authority.

## Quick start

Everything runs in Docker through `./dev.sh` and `make`; Rust does not have to be installed.

```sh
make models      # downloads the OCR models and every language pack into models/ (~94 MB)
make image       # builds the server image
docker run --rm -p 8080:8080 -v "$PWD/models":/models:ro dooai-engine

curl -X POST --data-binary @crates/engine-ocr/tests/fixtures/invoice.png http://localhost:8080/v1/ocr
```

## API

| Method | Path | Body | Answer |
|---|---|---|---|
| `GET` | `/health` | | `{"status": "ok"}` |
| `POST` | `/v1/ocr?language=ru` | the encoded image (PNG, JPEG, WebP, TIFF, BMP) | the document, below |

`language` is optional. It names the language of the page (`ru`, `ko`, `el`, `pt-BR`...) and selects the recognizer
trained on its alphabet. Without it, the general recognizer reads the page: Chinese, Japanese and every Latin-alphabet
language. A language whose recognizer is not installed answers `400` with the list of installed ones.

```json
{
  "text": "INVOICE 2026-0042\nSubtotal\tUSD 100.00\nTotal\tUSD 118.00",
  "confidence": 0.998,
  "rotation_degrees": 0,
  "engine": "paddle-ocr/general",
  "lines": [{ "text": "INVOICE 2026-0042", "confidence": 0.999,
              "polygon": [{"x": 38.1, "y": 40.3}, {"x": 441.0, "y": 40.3}, {"x": 441.0, "y": 82.8}, {"x": 38.1, "y": 82.8}],
              "bounding_box": { "left": 38.1, "top": 40.3, "right": 441.0, "bottom": 82.8 } }]
}
```

`text` is in reading order: one line per visual row, and the pieces of a row (table columns) separated by a tab.
`polygon` has the four corners of the line, clockwise from the top-left, and keeps the tilt of the text.
`rotation_degrees` is the clockwise turn applied to make the page upright; line geometry refers to the turned page.
An image that cannot be decoded answers `422 {"error": "..."}`.

Configuration, all optional:

| Variable | Default | Meaning |
|---|---|---|
| `ENGINE_ADDRESS` | `0.0.0.0:8080` | Where the server listens |
| `ENGINE_MODELS_DIR` | `models` | Directory with the OCR models |
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

The code does not ship weights. `make models` downloads them from Hugging Face (PaddlePaddle, Apache-2.0), one folder per
model under `models/`: `detection`, `orientation/document`, `orientation/textline`, and `recognition/<alphabet>`.
`recognition/general` is required; the other alphabets (cyrillic, korean, greek, arabic, thai, devanagari) are language
packs, installed only if you need them (`make models PACKS="cyrillic korean"`).

## How a page is read

The pipeline follows the PP-OCR design: a classifier turns the page upright (0/90/180/270), a detector finds each text
line as a rotated rectangle, each line is cut out with a perspective transform and turned around if upside down, and the
recognizer for the requested alphabet reads it. Rows are put in reading order in the page's own frame, so tilted photos
read like straight pages.

## Known limits

- Images only: PDFs are not read yet.
- The page orientation classifier can turn a sparse receipt upside down when it is unsure.
- Without `language`, non-Latin alphabets other than Chinese and Japanese are misread: pass the language.
- Pages tilted around 45 degrees are misjudged by the orientation classifier.
- Lines are recognized one by one, and each model session serves one request at a time.
