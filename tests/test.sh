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

for dir in cinterface module; do
  cd $dir
  make
  ./out > ../tmp.out
  case $? in
    0) echo "\"$dir\" PASSED RUNNING";;
    *)
      echo "\"$dir\" FAILED RUNNING"
      exit 1
      ;;
  esac
  cd ..
  diff tmp.out "intended/$dir.output"
  case $? in
    0) echo "\"$dir\" PASSED COMPARING";;
    *)
      echo "\"$dir\" FAILED COMPARING"
      exit 1
      ;;
    esac
    rm a.out
    rm tmp.out
done

echo "ALL TESTS PASSED"
