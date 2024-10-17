#!/bin/sh

cargo build --example gain

for path in $LD_LIBRARY_PATH; do
    patchelf --add-rpath $path target/debug/examples/libgain.so
done

cp target/debug/examples/libgain.so ~/.vst3/Gain.vst3/Contents/x86_64-linux/Gain.so
