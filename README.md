# megra.rs

Mégra implemented in Rust ! 

## WARNING

This is still in an early development stage and has some severe limitations! 

* It only works at a blocksize of 512 (Linux and macOS, as Windows WASAPI doesn't provide fixed blocksizes)
* So far only on MacOS and Linux (with Jack)
* There's no proper developer documentation so far

These issues are being addressed in no particular order ...

## Installation

Currently you still need cargo (https://doc.rust-lang.org/cargo/) installed !
Any version above 1.45 should work (last tested with 1.55).

* Download the repo ...
* In repo folder, type ...

```
cargo run --release -- -e -o 2ch
```

## Finding and Using Samples
If you don't already have a sample set at hand, some samples (enough to follow the documentation) can be found here: https://github.com/the-drunk-coder/megra-public-samples

Mégra currently only supports samples in **FLAC** format.

Place the samples (on Linux at least) in the folder `.config/megra/samples`. Now you'll have a sound event for every sample.

You can also load individual samples to a set by hand using `(load-sample :set '<set> :path "<path-to-sample>")`.

Make sure you cofigure your audio system to the samplerate of your samples, otherwise loading samples will be slow due to resampling !

## Sketchbook
The files generated and read by the editor can be found in `.config/megra/sketchbook`.

## Startup optinos

```
-v, --version       Print version
-r, --repl          no editor, repl only (i.e. for integration with other editors)
-h, --help          Print this help
-n, --no-samples    don't load default samples
-o, --output-mode   output mode (stereo, 2ch, 8ch), default: stereo
-l, --list-devices  list available audio devices
-d, --device        choose device
--live-buffer-time  the capacity of the live input buffer in seconds, default: 3
--reverb-mode       convolution or freeverb (default/fallback is freeverb). for convolution, you need to specify an IR
--reverb-ir         path to an impulse response in FLAC format
```

If the `-r` option is used, Mégra is started in REPL (command-line) mode. If you want to integrate it in your favourite editor, that might be helpful.



