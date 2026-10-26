# dooai-engine

Document AI for Odoo, written in Rust and served over HTTP. It runs the models on your own hardware, so documents
never leave your servers. The first capability is reading documents (OCR and PDF); extraction, classification and
embeddings come next.

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
| `POST` | `/v1/ocr?language=ru` | an image (PNG, JPEG, WebP, TIFF, BMP) or a PDF | the document, below |

`language` is optional. It names the language of the page (`ru`, `ko`, `el`, `pt-BR`...) and selects the recognizer
trained on its alphabet. Without it, the general recognizer reads the page: Chinese, Japanese and every Latin-alphabet
language. A language whose recognizer is not installed answers `400` with the list of installed ones.

Every answer is a document with pages: an image is a document of one page, a PDF has one page per sheet.

```json
{
  "text": "INVOICE 2026-0042\nSubtotal\tUSD 100.00\nTotal\tUSD 118.00",
  "pages": [{
    "page": 1,
    "text": "INVOICE 2026-0042\nSubtotal\tUSD 100.00\nTotal\tUSD 118.00",
    "confidence": 0.998,
    "rotation_degrees": 0,
    "engine": "paddle-ocr/general",
    "lines": [{ "text": "INVOICE 2026-0042", "confidence": 0.999,
                "polygon": [{"x": 38.1, "y": 40.3}, {"x": 441.0, "y": 40.3}, {"x": 441.0, "y": 82.8}, {"x": 38.1, "y": 82.8}],
                "bounding_box": { "left": 38.1, "top": 40.3, "right": 441.0, "bottom": 82.8 } }]
  }]
}
```

`text` is in reading order: one line per visual row, and the pieces of a row (table columns) separated by a tab. The
top-level `text` joins the pages with a blank line.
`polygon` has the four corners of the line and keeps the tilt of the text. `rotation_degrees` is the clockwise turn
applied to make the page upright. Geometry always refers to the original image, so it can be drawn straight over the
photo; the corners follow the reading direction of the text.

`engine` tells how a page was read. A PDF page that carries text is read from it (`pdf-text-layer`: exact, no OCR, and
no `lines` or `confidence`); a page without text, like a scan, is drawn at 200 DPI and read by OCR (`paddle-ocr/...`).
Each page of a PDF is decided on its own.

An image or PDF that cannot be read answers `422 {"error": "..."}`, and so does a PDF with more than 30 pages.

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
| `engine-ocr` | PP-OCR on ONNX Runtime: orientation, text detection, line recognition and the pipeline that joins them |
| `engine-document` | Reads a file: images go to OCR; PDFs are read per page from their text layer or by OCR (`hayro` draws pages, `pdf_oxide` reads text) |
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

The pipeline follows the PP-OCR design, with a few additions of its own:

1. A classifier proposes how to turn the page upright (0/90/180/270). When it is unsure (the two best classes are
   close), both turns are read and the one that yields more confident text wins.
2. A detector finds each text line as a rotated rectangle; each line is cut out with a perspective transform and
   turned around if it is upside down. If most lines were upside down, the page is read again turned a half turn more.
3. The recognizer for the requested alphabet reads each line.
4. Rows are put in reading order in the page's own frame, so tilted photos read like straight pages, and the geometry is
   mapped back onto the original image.

## Known limits

- Password-protected PDFs are not read, and HEIC photos (iPhone's default format) are not decoded: convert them first.
- A dense A4 page takes about 4 seconds on CPU: lines are recognized one at a time.
- Without `language`, non-Latin alphabets other than Chinese and Japanese are misread: pass the language.
- Pages tilted around 45 degrees are misjudged by the orientation classifier.
- Lines are recognized one by one, and each model session serves one request at a time.
