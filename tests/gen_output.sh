#!/bin/sh
set -e

for file in `ls *.ez`; do
  cargo run  -q -- $file -stdlib-path ../lib/zig-cache/lib/libstd.a
  ./a.out > "intended/$file.output"
  echo "generated intended/$file.output"
done
