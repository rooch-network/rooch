#!/bin/bash
rm -rf rooch-artifacts/*
mkdir -p rooch-artifacts/
cp -v target/release/rooch rooch-artifacts/
cp -v README.md rooch-artifacts/
if [ "$1" == "windows-latest" ]; then
  7z a -r rooch-$1.zip rooch-artifacts
else
  zip -r rooch-$1.zip rooch-artifacts
fi
