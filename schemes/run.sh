#!/bin/sh
set -e

printf "%s\n" "scheme,settings,corpus,compression speed (MB/s),compression speed standard deviation (MB/s),decompression speed (MB/s),decompression speed standard deviation (MB/s),compression ratio"
for scheme in $(find . -maxdepth 1 -type d -not -path "."); do
  cd "$scheme"
  cargo run --release
  cd ..
done
