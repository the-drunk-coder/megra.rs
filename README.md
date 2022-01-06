# megra.rs

Mégra is a **domain-specific** programming **language** (DSL for short) designed for **live-coding music** with stochastic elements.
Its predecessor was implemented in Common Lisp, this one is implemented in pure Rust !

This readme should contain all the necessary information to get Mégra up and running.

## Table Of Contents
* [WARNING](#warning)
* [Features](#features)
* [Limitations](#limitations)
* [Installation](#installation)
* [Audio Configuration](#audio-configuration)
* [Finding and Using Samples](#finding-and-using-samples)
* [Sketchbook](#sketchbook)
* [Running and Startup Options](#running-and-startup-options)
* [Learning Mégra](#learning-mégra) 

## WARNING

This is still in a relatively early development stage and has some limitations! It hasn't been
excessively tested on all platforms.

## Features

* It lets you make music with Markov chains!
* It follows a sequencing paradigm rather than a modular synth paradigm.
* It comes with its own (simple) editor.
* It works with Linux (JACK or PipeWire), Windows 10/11 (WASAPI), and macOS.

## Limitations

* It isn't a turing-complete programming language.
* It loads all your samples to memory, so if you have a lot of samples, make sure you have enough RAM.
* It's focused on samples. Synthesis is pretty primitive and limited to some basic waveforms at this point.
* It currently doesn't allow you to create fancy synths unless you want to code them in Rust.
* The editor is fairly primitive (you can use it in REPL mode and integrate in other editors if you want).

These issues are being addressed in no particular order ...

## Installation

Currently you still need `rustc` and `cargo` installed !

To install this, go to this page: https://www.rust-lang.org/learn/get-started.
This will guide you to the process of installing the necessary tools for your operating system.

On Windows, that's a bit annoying because you need some VisualStudio components which
might take a lot of space. 

Any version of Rust above 1.57 (stable version as of early 2022) should work (last tested with 1.57).

My goal is to provide precompiled binaries later on.

### Install with Cargo

In a terminal, type:

```
cargo install megra.rs
```

On Windows, type:

```
cargo install megra.rs --features ringbuffer
```

The ringbuffer features makes sure you can use various blocksizes. The blocksize is otherwise fixed to 512. If you can
control the blocksize in your system (with JACK and CoreAudio that's typically possible), you can use the version without
the ringbuffer. If you're not sure, use the ringbuffer. It has a small performance penalty but shouldn't matter on modern
systems.

### Compile from Source

* Download the repo ...
* In repo folder, type ...

```
cargo run --release -- -e -o 2ch
```
Before starting, make sure you read the chapter about the audio configuration!

## Audio Configuration

As mentioned, the default version of Mégra only works at a fixed blocksize of 512, so if that is the version you installed, make sure
your system blocksize is at 512. Any samplerate should work, but be aware that all samples you use will be resampled to the current samplerate
if they don't match, which might increase loading time. I recommend using the samplerate your samples are stored in. 

## Finding and Using Samples

If you don't already have a sample set at hand, some samples (enough to follow the documentation) can be found here:
https://github.com/the-drunk-coder/megra-public-samples

Mégra currently only supports samples in **FLAC** format.

Place the samples in the folder (folder will be created at first start):

* Linux: `~/.config/megra/samples`.
* Windows: `C:\Users\<username>\AppData\Roaming\parkellipsen\megra\config\samples`
* macOS: `/Users/<username>/Library/Application Support/de.parkellipsen.megra/samples`

Now you'll have a sound event for every sample.

You can also load individual samples to a set by hand using `(load-sample :set '<set> :path "<path-to-sample>")`.

As mentioned above, make sure you cofigure your audio system to the samplerate of your samples, otherwise loading samples will be slow due to resampling !

## Sketchbook
The files generated and read by the editor can be found in:

* Linux: `~/.config/megra/sketchbook`.
* Windows: `C:\Users\<username>\AppData\Roaming\parkellipsen\megra\config\sketchbook`
* macOS: `/Users/<username>/Library/Application Support/de.parkellipsen.megra/sketchbook`

## Running and Startup Options

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

## Learning Mégra

Now that you should have things up and running, it's time to learn how to use this langunage, right ? Here's where you can start!


