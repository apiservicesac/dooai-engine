# dooai-engine

Document AI for Odoo, written in Rust and served over HTTP. It runs on your own hardware, so documents never leave
your servers. Nothing in it is specific to a country.

| Capability | What it does |
|---|---|
| **Read** (`/v1/ocr`) | Text and geometry of an image or a PDF, in 50+ languages, upright even when photographed sideways |
| **Classify** (`/v1/classify`) | What kind of document a text is: invoice, credit note, delivery note, receipt, contract, purchase order, quotation, bank statement. It says `unknown` instead of guessing |
| **Extract** (`/v1/extract`) | Issuer, customer, rows, taxes and totals as JSON, with every sum checked. Needs a language model you provide |
| **Match** (`/v1/embed`, `/v1/similar`) | Text embeddings in 100 languages, to find the contact, product or invoice a text refers to |

How well each one works, and what the numbers do not prove, is in [`docs/evaluation.md`](docs/evaluation.md).

## Quick start

Everything runs in Docker through `./dev.sh` and `make`; Rust does not have to be installed.

```sh
make models      # downloads the models into models/ (~220 MB)
make image       # builds the server image
docker run --rm -p 8080:8080 -v "$PWD/models":/models:ro dooai-engine

curl -X POST --data-binary @crates/engine-ocr/tests/fixtures/invoice.png http://localhost:8080/v1/ocr
curl -X POST -H 'content-type: application/json' -d '{"text": "FACTURA F001-123 Total a pagar 118.00"}' \
  http://localhost:8080/v1/classify
```

## API

`GET /health` answers `{"status": "ok", "capabilities": [...]}`: what this installation can do.

### `POST /v1/ocr?language=ru`

The body is an image (PNG, JPEG, WebP, TIFF, BMP) or a PDF. `language` is optional: it names the language of the page
(`ru`, `ko`, `el`, `pt-BR`...) and selects the recognizer for its alphabet. Without it the general recognizer reads
the page: Chinese, Japanese and every Latin-alphabet language. A language whose recognizer is not installed answers
`400` with the list of installed ones.

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
top-level `text` joins the pages with a blank line. `polygon` has the four corners of the line and keeps the tilt of
the text. `rotation_degrees` is the clockwise turn applied to make the page upright. Geometry always refers to the
original image, so it can be drawn straight over the photo.

`engine` tells how a page was read. A PDF page that carries text is read from it (`pdf-text-layer`: exact, no OCR, and
no `lines` or `confidence`); a page without text, like a scan, is drawn at 200 DPI and read by OCR (`paddle-ocr/...`).
Each page of a PDF is decided on its own. An image or PDF that cannot be read answers `422`, and so does a PDF with more
than 30 pages.

### `POST /v1/classify`

`{"text": "..."}` answers what kind of document the text is. Only the start of the text is read.

```json
{ "label": "invoice", "confidence": 0.83, "matched": ["invoice"], "scores": [{ "label": "invoice", "score": 5 }] }
```

`label` is `unknown` (with confidence 0) when the evidence is thin or ambiguous: ask something stronger then.
`matched` lists the words that point to the kind, so a person can check the reason. The kinds and their words are a
JSON lexicon you can replace with `ENGINE_LEXICON_FILE` (see `crates/engine-classify/data/document-lexicon.json`).

### `POST /v1/extract?language=ru`

The body is an image or a PDF. The document is read, its kind is told and its data extracted:

```json
{
  "classification": { "label": "invoice", "confidence": 0.83, "matched": ["invoice"], "scores": [] },
  "document": {
    "number": "2026-0042", "issue_date": "2026-10-25", "due_date": null, "currency": "USD",
    "supplier": { "name": "Northwind Traders Ltd", "tax_ids": [{ "scheme": "VAT", "value": "GB123456789" }], "address": null },
    "customer": { "name": null, "tax_ids": [], "address": null },
    "lines": [{ "description": "Chairs", "quantity": 2, "unit_price": 50, "amount": 100 }],
    "taxes": [{ "name": "Tax", "rate": 18, "base": 100, "amount": 18 }],
    "subtotal": 100, "tax_total": 18, "adjustment_total": null, "total": 118, "payment_terms": null
  },
  "validation": { "checks_performed": 6, "issues": [] },
  "attempts": 1, "model": "your-model", "text": "..."
}
```

Whatever the document does not state is `null`: nothing is guessed. Tax identifiers are a list of `{scheme, value}`,
whatever the country calls them. `validation` checks that every row multiplies, the rows add up to the subtotal, the
taxes to the tax total, subtotal plus tax plus adjustments to the total, that the currency is ISO 4217 and the dates
exist. An issue has a `severity`: `error` (the document contradicts itself) or `warning` (something is missing). With
`checks_performed` 0 nothing was verified. When the numbers do not add up, the model is shown exactly what is wrong and
asked once more before the result is returned with its issues.

It needs a language model that speaks the OpenAI chat completions API (OpenAI, vLLM, llama.cpp, Ollama...): set
`ENGINE_LLM_URL` and `ENGINE_LLM_MODEL`. Without them it answers `501`; if the model cannot be reached, `502`.

### `POST /v1/embed` and `POST /v1/similar`

```json
{ "texts": ["Distribuciones Peñalver S.A.", "Invoice 001"] }
{ "query": "Peñalver Distribution Ltd", "candidates": [{ "id": "7", "text": "Distribuciones Peñalver S.A." }], "top_k": 5 }
```

`/v1/embed` answers one unit-length vector per text; `/v1/similar` ranks the candidates by cosine similarity. They
need the embedding model (`501` otherwise). Similar is not the same: `Latitude 5420` and `5430` score 0.98, so use
this to propose candidates and decide with exact data.

## Configuration

All optional.

| Variable | Default | Meaning |
|---|---|---|
| `ENGINE_ADDRESS` | `0.0.0.0:8080` | Where the server listens |
| `ENGINE_MODELS_DIR` | `models` | Directory with the models |
| `ENGINE_MAX_UPLOAD_BYTES` | `20971520` | Largest accepted image or PDF |
| `ENGINE_LEXICON_FILE` | built in | Kinds of document and the words that name them |
| `ENGINE_LLM_URL`, `ENGINE_LLM_MODEL` | none | Language model of `/v1/extract`; set both |
| `ENGINE_LLM_API_KEY` | none | Sent as a bearer token to the language model |

The server has no authentication: put it behind a gateway that owns API keys, quotas and metering.

## Layout

| Crate | Responsibility |
|---|---|
| `engine-core` | Shared types (`OcrDocument`, `TextQuad`...), the `OcrEngine` and `TextEmbedder` traits and `EngineError` |
| `engine-onnx` | Runs ONNX models on one shared ONNX Runtime environment |
| `engine-ocr` | PP-OCR: orientation, text detection, line recognition and the pipeline that joins them |
| `engine-document` | Reads a file: images go to OCR; PDFs are read per page from their text layer or by OCR (`hayro` draws pages, `pdf_oxide` reads text) |
| `engine-classify` | The lexicon classifier of kinds of document |
| `engine-extract` | The document schema, the arithmetic validator and the language-model extractor |
| `engine-embed` | multilingual-e5-small embeddings and the similarity ranker |
| `engine-server` | The HTTP API (axum) |

## Development

```sh
make test    # unit and integration tests; run `make models` first to also run the ones that read real documents
make lint    # clippy, any warning fails
make fmt     # format
make ci      # format check, clippy and tests
```

See [`CONTRIBUTING.md`](CONTRIBUTING.md) for the code style.

## Models

The code does not ship weights. `make models` downloads them from Hugging Face, one folder per model under `models/`:

- `detection`, `orientation/document`, `orientation/textline` and `recognition/general`: PaddlePaddle (Apache-2.0).
- `recognition/<alphabet>`: the language packs (cyrillic, korean, greek, arabic, thai, devanagari), also PaddlePaddle,
  installed only if you need them (`make models PACKS="cyrillic korean"`).
- `embedding`: multilingual-e5-small by Microsoft Research (MIT), int8.

`recognition/general` and the detection and orientation models are required; without `embedding` the text endpoints
answer `501`.

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
- A dense A4 page takes about 4 seconds on CPU: lines are recognized one at a time, and each model session serves one
  request at a time.
- Without `language`, alphabets other than Latin, Chinese and Japanese are misread: pass the language.
- Pages tilted around 45 degrees are misjudged by the orientation classifier.
- The classifier knows the kinds of document by the words that name them: a ticket that never says what it is, or a
  title in a language not in the lexicon, is `unknown`. The kinds are the eight of the lexicon.
- Text embeddings do not tell apart products that differ in one detail (a model number, a size).
- `/v1/extract` has not been measured against a real language model; its arithmetic validation has.
- Text boxes are straight rectangles: strongly curved text is read worse.

## License

[Apache License 2.0](LICENSE). The models keep their own licenses, listed in [`NOTICE`](NOTICE).
