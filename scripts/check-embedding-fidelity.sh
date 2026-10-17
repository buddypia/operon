#!/usr/bin/env bash
# Whether the shipped EmbeddingGemma 2 runtime (llama.cpp on Metal, in-process)
# still produces the vectors the tests rank on: every text in
# tests/fixtures/embedding-golden.json is embedded through
# `Operon knowledge embed-golden` and compared with its sentence-transformers
# reference vector. Fails when any cosine is below EMBEDDING_FIDELITY_MIN_COSINE
# (0.995, src/config.rs). Change 157, spec Req 15.
#
# not-wired: it needs the 310MB model downloaded through Settings › データ, which
# a CI runner does not have; there it could only report "no model". Run it by
# hand after bumping llama-cpp-2, the llama.cpp submodule, or the model pin.
set -uo pipefail

cd "${CLAUDE_PROJECT_DIR:-$(dirname "$0")/..}" || exit 1

fixture=tests/fixtures/embedding-golden.json
cargo build --locked --release --quiet || exit 1
report=$(target/release/operon knowledge embed-golden "$fixture")
status=$?
printf '%s\n' "$report" | head -5
case $status in
  0) echo "fidelity: $(printf '%s\n' "$report" | grep -c .) texts, lowest $(printf '%s\n' "$report" | head -1 | cut -f1) >= 0.995" ;;
  3) echo "fidelity: the model is not downloaded (Settings › データ)" >&2 ;;
  *) echo "fidelity: below 0.995 or failed (exit $status)" >&2 ;;
esac
exit "$status"
