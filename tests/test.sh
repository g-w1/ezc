#!/bin/sh


for file in `ls *.ez`; do
  cargo run -q -- $file -stdlib-path ../lib/zig-cache/lib/libstd.a 2>&1 >/dev/null
  ./a.out > tmp.out
  case $? in
    0) echo "\"$file\" PASSED RUNNING";;
    *)
      echo "\"$file\" FAILED RUNNING"
      exit 1
      ;;
    esac
  diff tmp.out "intended/$file.output"
  case $? in
    0) echo "\"$file\" PASSED COMPARING";;
    *)
      echo "\"$file\" FAILED COMPARING"
      exit 1
      ;;
    esac
    rm tmp.out
done

echo "ALL TESTS PASSED"
