# megra.rs

(Versión español: [LEAME.md](https://github.com/the-drunk-coder/megra.rs/blob/main/LEAME.md))

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

To install this, go to this page: https://www.rust-lang.org/learn/get-started .
This will guide you to the process of installing the necessary tools for your operating system.

On Windows, that's a bit annoying because you need some VisualStudio components which
might take a lot of space. 

Any version of Rust above 1.57 (stable version as of early 2022) should work (last tested with 1.57).

My goal is to provide precompiled binaries later on.

### Install with Cargo

In a terminal, type:

```
cargo install megra_rs
```

On Windows, type:

```
cargo install megra_rs --features ringbuffer
```

The ringbuffer feature makes sure you can use various blocksizes. The blocksize is otherwise fixed to 512. If you can
control the blocksize in your system (with JACK and CoreAudio that's typically possible), you can use the version without
the ringbuffer. If you're not sure, use the ringbuffer. It has a small performance penalty but shouldn't matter on modern
systems.

Now you can call Mégra from a terminal calling `megra_rs`. On Linux, make sure JACK is started (or call with the `pw-jack` prefix 
if using *PipeWire*). On Windows, you can also find the exe file and double-click it.

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

Now you'll have a sound event for every sample. That means, if you have a folder called `bd` in the samples folder, you can call it like this:

```(lisp)
;; call this in the Mégra editor (place cursor between outer parenthesis and hit "Ctrl+Return")
(once (bd))
```
You can also search by keyword ... if you have a sample called `jazz.flac` in your `bd` folder, you can call it like:

```(lisp)
(once (bd 'jazz))
```

If you don't provide any keyword, a random sample from the folder is chosen. Samples outside of the abovementioned samples folder can't be called.

There's currently no way to specify custom sample folders, but you can also load individual samples to a set by 
hand using `(load-sample :set '<set> :path "<path-to-sample>")`. These don't need to be in the samples folder, the path can point anywhere. 

As mentioned above, make sure you cofigure your audio system to the samplerate of your samples, otherwise loading samples will be slow due to resampling !

## Sketchbook
The files generated and read by the editor can be found in:

* Linux: `~/.config/megra/sketchbook`.
* Windows: `C:\Users\<username>\AppData\Roaming\parkellipsen\megra\config\sketchbook`
* macOS: `/Users/<username>/Library/Application Support/de.parkellipsen.megra/sketchbook`

Files in these folders will be automatically displayed in the editor's selection menu.

## Running and Startup Options

To run Mégra after installation, call `megra_rs` in a terminal or (i.e. on Windows) double-click on the executable.

Whether you start with Cargo or from a terminal or program launcher, the options are:

```
-v, --version       Print version
-r, --repl          no editor, repl only (i.e. for integration with other editors)
-h, --help          Print this help
-n, --no-samples    don't load default samples
-o, --output-mode   output mode (stereo, 2ch, 8ch), default: stereo
-l, --list-devices  list available audio devices
-d, --device        choose device
--live-buffer-time  the capacity of the live input buffer in seconds, default: 3
--font              editor font (mononoki, ComicMono or custom path, default: mononoki)
--reverb-mode       convolution or freeverb (default/fallback is freeverb). for convolution, you need to specify an IR
--reverb-ir         path to an impulse response in FLAC format
```

If the `-r` option is used, Mégra is started in REPL (command-line) mode. If you want to integrate it in your favourite editor, that might be helpful.

## Learning Mégra

Now that you should have things up and running, it's time to learn how to use this language, right ? Here's where you can start!

The english documentation can be found in the main repository: https://github.com/the-drunk-coder/megra.rs/tree/main/documentation/en

### Tutorial

The folder contains a file called `tutorial/megra_basic_tutorial.megra3`. You can put that in your sketchbook folder (see above) and 
directly follow the tutorial. There might be more files in there that you can follow later.

### Examples

There's also a folder called `examples/`, a growing selection of interesting examples.

### Reference Documentation

The documentation folder contains two markdown files, called `Function_List.md` which is an ongoing project to document all the functions 
that Mégra currently provides. There should also be an example to play around with for each function.

Regarding the sound events, there's a file called `Sound_Events_and_Parameters.md` which should cover all the currently available sound events.
