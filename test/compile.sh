#!/bin/bash

set -eu

echo "Creating binaries for tests..."

mkdir -p bin/
gcc test/exitcodes.c -o bin/exitcodes
gcc test/signal.c -o bin/signal

echo "Binaries are created!"
