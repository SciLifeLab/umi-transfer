#!/bin/bash
# Regenerate tests/results reference files from current seqdata (100-base fixtures).
# Run from project root. Requires: cargo build (binary at target/debug/umi-transfer).

set -e
BIN="target/debug/umi-transfer"
SEQ="tests/seqdata"
RES="tests/results"

if [[ ! -x "$BIN" ]]; then
  echo "Build first: cargo build"
  exit 1
fi

TMP=$(mktemp -d)
trap "rm -rf $TMP" EXIT
cp "$SEQ"/*.fq "$SEQ"/*.gz "$TMP/" 2>/dev/null || true

run() {
  "$BIN" external --force "$@"
}

# 1) Header (default)
run --in "$TMP/read1.fq" --in2 "$TMP/read2.fq" --umi "$TMP/umi.fq"
cp "$TMP/read1_with_UMIs.fq" "$RES/header_correct_read1.fq"
cp "$TMP/read2_with_UMIs.fq" "$RES/header_correct_read2.fq"

# 2) Inline
run --in "$TMP/read1.fq" --in2 "$TMP/read2.fq" --umi "$TMP/umi.fq" --position inline
cp "$TMP/read1_with_UMIs.fq" "$RES/inline_correct_read1.fq"
cp "$TMP/read2_with_UMIs.fq" "$RES/inline_correct_read2.fq"

# 3) correct_numbers (header)
run --in "$TMP/read1.fq" --in2 "$TMP/read2.fq" --umi "$TMP/umi.fq" --correct_numbers
cp "$TMP/read1_with_UMIs.fq" "$RES/header_corrected_read1.fq"
cp "$TMP/read2_with_UMIs.fq" "$RES/header_corrected_read2.fq"

# 4) correct_numbers + inline
run --in "$TMP/read1.fq" --in2 "$TMP/read2.fq" --umi "$TMP/umi.fq" --correct_numbers --position inline
cp "$TMP/read1_with_UMIs.fq" "$RES/inline_corrected_read1.fq"
cp "$TMP/read2_with_UMIs.fq" "$RES/inline_corrected_read2.fq"

# 5) gzip (header)
run --in "$TMP/read1.fq" --in2 "$TMP/read2.fq" --umi "$TMP/umi.fq" --gzip
cp "$TMP/read1_with_UMIs.fq.gz" "$RES/header_correct_read1.fq.gz"
cp "$TMP/read2_with_UMIs.fq.gz" "$RES/header_correct_read2.fq.gz"

# 6) gzip + inline
run --in "$TMP/read1.fq" --in2 "$TMP/read2.fq" --umi "$TMP/umi.fq" --gzip --position inline
cp "$TMP/read1_with_UMIs.fq.gz" "$RES/inline_correct_read1.fq.gz"
cp "$TMP/read2_with_UMIs.fq.gz" "$RES/inline_correct_read2.fq.gz"

# 7) compression level 9 + gzip (header)
run --in "$TMP/read1.fq" --in2 "$TMP/read2.fq" --umi "$TMP/umi.fq" --compression_level 9 --gzip
cp "$TMP/read1_with_UMIs.fq.gz" "$RES/header_correct_read1_lvl9.fq.gz"
cp "$TMP/read2_with_UMIs.fq.gz" "$RES/header_correct_read2_lvl9.fq.gz"

# 8) compression level 9 + gzip + inline
run --in "$TMP/read1.fq" --in2 "$TMP/read2.fq" --umi "$TMP/umi.fq" --compression_level 9 --gzip --position inline
cp "$TMP/read1_with_UMIs.fq.gz" "$RES/inline_correct_read1_lvl9.fq.gz"
cp "$TMP/read2_with_UMIs.fq.gz" "$RES/inline_correct_read2_lvl9.fq.gz"

# 9) underscore delimiter
run --in "$TMP/read1.fq" --in2 "$TMP/read2.fq" --umi "$TMP/umi.fq" --delim "_"
cp "$TMP/read1_with_UMIs.fq" "$RES/delim_underscore_read1.fq"
cp "$TMP/read2_with_UMIs.fq" "$RES/delim_underscore_read2.fq"

# 10) switch umi and read2: --in read1 --in2 umi --umi read2 -> read1_with_UMIs.fq, umi_with_UMIs.fq
run --in "$TMP/read1.fq" --in2 "$TMP/umi.fq" --umi "$TMP/read2.fq"
cp "$TMP/read1_with_UMIs.fq" "$RES/umi_read2_switch_read1.fq"
cp "$TMP/umi_with_UMIs.fq" "$RES/umi_read2_switch_read2.fq"

echo "Regenerated all reference files in $RES"
ls -la "$RES"
