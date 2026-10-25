#!/usr/bin/env bash
# Downloads the PP-OCRv6 ONNX models (Apache-2.0, PaddlePaddle) into models/<size>.
# Usage: scripts/download-models.sh [tiny|small|medium]
set -euo pipefail

MODEL_SIZE="${1:-small}"
PROJECT_DIRECTORY="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
TARGET_DIRECTORY="${PROJECT_DIRECTORY}/models/${MODEL_SIZE}"
HUB_URL="https://huggingface.co/PaddlePaddle"

mkdir -p "${TARGET_DIRECTORY}"
for part in det rec; do
  repository="${HUB_URL}/PP-OCRv6_${MODEL_SIZE}_${part}_onnx/resolve/main"
  curl --fail --location --silent --show-error -o "${TARGET_DIRECTORY}/${part}.onnx" "${repository}/inference.onnx"
  curl --fail --location --silent --show-error -o "${TARGET_DIRECTORY}/${part}.yml" "${repository}/inference.yml"
done
echo "Models ready in ${TARGET_DIRECTORY}"
