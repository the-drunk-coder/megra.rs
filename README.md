# megra.rs

Mégra implemented in Rust ! 

## WARNING

This is still in an early development stage and has some severe limitations! 

* It only works at a blocksize of 512
* So far only on MacOS and Linux (with Jack)
* There's no user- or developer documentation so far

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

Place the samples (on Linux at least) in the folder `.config/megra/samples`. Now you'll have a sound event for every sample.

## Sketchbook
The files generated and read by the editor can be found in `.config/megra/sketchbook`.

## Startup optinos

```
-v, --version       Print version
-e, --editor        Use integrated editor (experimental)
-h, --help          Print this help
-n, --no-samples    don't load default samples
-o, --output-mode   output mode (stereo, 2ch, 8ch), default: stereo
-l, --list-devices  list available audio devices
-d, --device        choose device
--live-buffer-time  the capacity of the live input buffer in seconds, default: 3
```

If the `-e` option is omitted, Mégra is started in REPL (command-line) mode. If you want to integrate it in your favourite editor, that might be helpful.



