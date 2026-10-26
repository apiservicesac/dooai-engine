# Contributing

## Building

Everything runs in Docker through `./dev.sh` and `make`; nothing else is installed on the host.

```sh
make models   # downloads the models into models/ (needed by the tests that read real documents)
make test     # unit and integration tests
make lint     # clippy, any warning fails
make fmt      # format
make ci       # format check, clippy and tests, as CI runs them
make image    # the server's Docker image
```

## Code style

- Types own behaviour: no free functions; one responsibility per type and one type per file.
- Descriptive names: no single-letter variables (a coordinate `x`/`y` is the only exception), no leading underscores.
- Every fixed value lives in its crate's `settings.rs`, with a comment saying what it controls.
- Comments explain why, not what; keep them few.
- Use established libraries rather than hand-written equivalents.
- Add dependencies with `cargo add`, so they are the current versions.

## Measure before you decide

Changes to models, thresholds or scoring come with the measurement that justifies them, on real documents when
possible. [`docs/evaluation.md`](docs/evaluation.md) shows how the current numbers were obtained and what they do not
prove.
