#!/usr/bin/env bash
# Downloads the OCR models (Apache-2.0, PaddlePaddle) into models/, one directory per model:
#   detection/  orientation/document/  orientation/textline/  recognition/general/  recognition/<pack>/
# The core models are always installed; language packs add recognizers for more alphabets.
# Usage: scripts/download-models.sh [pack...]   (default: every pack)
set -euo pipefail

PROJECT_DIRECTORY="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
MODELS_DIRECTORY="${PROJECT_DIRECTORY}/models"
HUB_URL="https://huggingface.co/PaddlePaddle"

# pack -> Hugging Face repository
declare -A PACKS=(
  [cyrillic]="cyrillic_PP-OCRv5_mobile_rec_onnx"
  [korean]="korean_PP-OCRv5_mobile_rec_onnx"
  [greek]="el_PP-OCRv5_mobile_rec_onnx"
  [arabic]="arabic_PP-OCRv5_mobile_rec_onnx"
  [thai]="th_PP-OCRv5_mobile_rec_onnx"
  [devanagari]="devanagari_PP-OCRv5_mobile_rec_onnx"
)

download() {
  local repository="$1" target="$2"
  mkdir -p "${MODELS_DIRECTORY}/${target}"
  curl --fail --location --silent --show-error -o "${MODELS_DIRECTORY}/${target}/model.onnx" "${HUB_URL}/${repository}/resolve/main/inference.onnx"
  curl --fail --location --silent --show-error -o "${MODELS_DIRECTORY}/${target}/model.yml" "${HUB_URL}/${repository}/resolve/main/inference.yml"
  echo "  ${target}"
}

echo "Core models:"
download PP-OCRv6_small_det_onnx detection
download PP-LCNet_x1_0_doc_ori_onnx orientation/document
download PP-LCNet_x1_0_textline_ori_onnx orientation/textline
download PP-OCRv6_small_rec_onnx recognition/general

selected=("$@")
[ ${#selected[@]} -gt 0 ] || selected=("${!PACKS[@]}")
echo "Language packs:"
for pack in "${selected[@]}"; do
  [ -n "${PACKS[$pack]:-}" ] || { echo "unknown pack '${pack}'; available: ${!PACKS[*]}" >&2; exit 2; }
  download "${PACKS[$pack]}" "recognition/${pack}"
done
echo "Models ready in ${MODELS_DIRECTORY}"
