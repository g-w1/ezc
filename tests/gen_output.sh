#!/bin/sh
set -e

for file in `ls *.ez`; do
  cargo run  -q -- $file -stdlib-path ../lib/zig-cache/lib/libstd.a
  ./a.out > "intended/$file.output"
  echo "generated intended/$file.output"
done

cd module
make
./out > ../intended/module.output
echo "generated intended/module.output"
cd ..

cd cinterface
make
./out > ../intended/cinterface.output
echo "generated intended/cinterface.output"
cd ..
