#!/bin/sh

cargo build --example gain --release

for path in $LD_LIBRARY_PATH; do
    patchelf --add-rpath $path target/release/examples/libgain.so
done

mkdir -p ~/.vst3/Gain.vst3/Contents/x86_64-linux
cp target/release/examples/libgain.so ~/.vst3/Gain.vst3/Contents/x86_64-linux/Gain.so
