#!/bin/bash

docker run --rm -it \
  -v $(pwd):/kslm \
  -w /kslm \
  kslm-builder:260309 \
  cargo build --release

