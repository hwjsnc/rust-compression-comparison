#!/bin/sh
set -e

for scheme in $(find . -maxdepth 1 -type d -not -path "."); do
  cd $scheme
  cargo clean
  cd ..
done
