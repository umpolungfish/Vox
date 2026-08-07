#!/bin/sh
# Build the verification corpus at every optimisation level gcc offers a
# meaningful difference at. verify.py takes the results as its arguments:
#
#   ./build_corpus.sh && python3 verify.py corpus_O*.so
set -e
for opt in O0 O1 O2 O3 Os; do
    gcc -shared -fPIC -"$opt" -o "corpus_$opt.so" corpus.c
done
ls -l corpus_O*.so
