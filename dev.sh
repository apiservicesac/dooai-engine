#!/bin/sh
# Runs development tooling inside Docker so nothing is installed on the host.
#   ./dev.sh cargo <args>    Rust toolchain (rust:1-trixie), cargo cache kept in .cache/
set -e
cd "$(dirname "$0")"
PROJECT="$PWD"
TERMINAL=""; [ -t 0 ] && TERMINAL="-it"
mkdir -p .cache/cargo
BASE="docker run --rm $TERMINAL -u $(id -u):$(id -g) -e HOME=/tmp -e USER=developer -v $PROJECT:/project"
TOOL="$1"; shift
case "$TOOL" in
  cargo)
    # The server's port is published only when asked: ENGINE_PUBLISH_PORT=1 (make run).
    PORT=""; [ -n "$ENGINE_PUBLISH_PORT" ] && PORT="-p ${ENGINE_PORT:-8080}:8080"
    exec $BASE $PORT -e CARGO_HOME=/project/.cache/cargo -e CARGO_TARGET_DIR=/project/target -e ENGINE_MODELS_DIR \
      -w /project rust:1-trixie cargo "$@" ;;
  *)
    echo "usage: ./dev.sh cargo ..." >&2; exit 2 ;;
esac
