# What was measured

Numbers from the development of dooai-engine, October 2026, on CPU. They guided decisions; they are not a benchmark.
Every sample is small. The test documents are not in this repository: they were generated, or downloaded for local use
only.

## Reading documents (OCR)

**Real receipts**: 30 receipts of CORD (Indonesian, photographed; CC-BY-4.0) with the text transcribed in the dataset.
*Token F1* compares the set of words; *order-aware* compares the text in reading order.

| Condition | Token F1 | Order-aware |
|---|---|---|
| Clean | 0.878 | 0.959 |
| Rotated 8° | 0.840 | 0.921 |
| Rotated 15° | 0.797 | 0.856 |
| Rotated 90° / upside down | 0.811 / 0.878 | 0.892 / 0.959 |
| Shadow, medium blur | 0.837, 0.708 | 0.888, 0.827 |
| Low resolution, poor JPEG | 0.703, 0.617 | 0.847, 0.794 |
| Noise | 0.488 | 0.631 |
| Heavy blur | 0.285 | 0.370 |

About 0.3 s per receipt. A dense A4 page (46 rows, around 320 text fragments) takes about 4.5 s, because lines are
recognized one at a time.

**Synthetic invoices** (generated with known text, character similarity): Spanish, French, German, Portuguese and
Polish 1.00; Russian 0.99, Greek 0.99, Korean 0.97, Chinese 1.00, Japanese 0.99, each with its own recognizer. Pages
rotated 4° to 30° and by 90°, 180° and 270° read at 1.00; a page rotated 45° is not read correctly. Without
`language=ru|ko|el|...` the general recognizer misreads those alphabets (Russian 0.48).

The detector scales pages down to 960 px on the long side; a dense A4 page at 200 DPI was read perfectly at 960,
1600 and 2400 px, so the default was kept.

## Telling the kind of document

Two approaches were measured on the same documents.

| Approach | Held-out synthetic | 26 real invoices | 30 real receipts |
|---|---|---|---|
| Embedding of the whole text against examples (dropped) | 40% | 69% | 57% |
| Linear head on embeddings, trained on augmented examples (dropped) | 71% on terms never seen | 46% | 87% |
| **Lexicon of titles and phrases (kept)** | 100% precision when it answers | **26/26** | 13/30, **17 `unknown`** |

The invoices are the Katanaml set (synthetic invoices rendered as images; its licence is not stated, so they were used
locally only). The lexicon gave **no wrong answer on the 56 real documents** and answered `unknown` to all 12 letters,
menus, CVs and similar documents. Its limit is vocabulary: tickets that never say what they are (a till receipt with
only `TOTAL` and `CASH`) are mostly `unknown`, and a title written in a language or a way the lexicon does not know is
`unknown` too. When tested with titles and phrases it had never seen, it answered 6% to 17% of the documents and was
right 83% to 94% of the time it answered. The "100%" synthetic figure uses the same vocabulary as the lexicon: it is a
ceiling, not an accuracy.

## Text embeddings (`/v1/embed`, `/v1/similar`)

- Duplicate contacts among 40 unrelated companies (uppercase, no accents, typos, reordered words, abbreviations):
  280/280, **the same as a plain string-distance baseline**. The test was too easy to tell them apart.
- Among 64 confusable companies (same generic word or same distinctive word) with the generic word translated
  (`Servicios Peñalver S.A.` vs `Peñalver Services Ltd`): **62/64 against 56/64** for the baseline. With a typo in the
  distinctive word the baseline was slightly better (64/64 against 63/64).
- The same product described in another language, among 10: 30/30 against 29/30.
- Products that differ in one detail (`Latitude 5420` vs `5430`, `2 metros` vs `5 metros`, size `L` vs `S`) score
  **0.96 to 0.98**. A similarity score alone must not decide that two products are the same.

Use embeddings to propose candidates across languages; decide with exact data (tax id, document number, amounts).

## Extraction (`/v1/extract`)

The validator is exact logic and has its own tests: rows, sums, taxes, total, currency and dates. The language-model
path was tested only against a stub server that checks what is sent and how a wrong answer is repaired. **It has not
been measured against a real model**, so no accuracy is claimed.
