#!/usr/bin/env bash

if [[ $OSTYPE == 'darwin'* ]]; then
  cargoMakeUrl="https://github.com/sagiegurari/cargo-make/releases/download/0.35.0/cargo-make-v0.35.0-x86_64-apple-darwin.zip"
  cargoMakeZipSubFolder="cargo-make-v0.35.0-x86_64-apple-darwin"
else
  cargoMakeUrl="https://github.com/sagiegurari/cargo-make/releases/download/0.35.0/cargo-make-v0.35.0-x86_64-unknown-linux-musl.zip"
  cargoMakeZipSubFolder="cargo-make-v0.35.0-x86_64-unknown-linux-musl"
fi

cargoMakeLocalPackage="/tmp/cargo-make.zip"
cargoMakeTempDir="/tmp/cargo-make"

echo "Clean up old binary packages."
rm $cargoMakeLocalPackage
rm -rf $cargoMakeTempDir
echo ""

echo "Downloading cargo-make binary package from $cargoMakeUrl to $cargoMakeLocalPackage"
wget $cargoMakeUrl -O $cargoMakeLocalPackage
echo ""

echo "Unzip cargo-make package $cargoMakeLocalPackage to $cargoMakeTempDir"
unzip $cargoMakeLocalPackage -d $cargoMakeTempDir

cargoBinaryDir="${HOME}/.cargo/bin"
echo "Copy ${cargoMakeTempDir}/${cargoMakeZipSubFolder}/cargo-make to cargo binary dir: $cargoBinaryDir"
cp -v "${cargoMakeTempDir}/${cargoMakeZipSubFolder}/cargo-make" $cargoBinaryDir