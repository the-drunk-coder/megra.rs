# megra.rs

(Versión español: [LEAME.md](https://github.com/the-drunk-coder/megra.rs/blob/main/LEAME.md))

Mégra is a **domain-specific** programming **language** (DSL for short) designed for **live-coding music** with stochastic elements.
It's implemented in pure Rust. It's development is heavily influenced by the author's own artistic practice, yet it should be fairly
easy to learn for anybody else.

The [Documentation](#documentation) contains all infos to get Mégra up and running.

## Table Of Contents

* [Features](#features)
* [Limitations](#limitations)
* [Documentation](#documentation)
* [Questions & Feedback](#questions--feedback)

## Features

* It lets you make music with Markov chains!
* It follows a sequencing paradigm rather than a modular synth paradigm.
* It iss a standalone program that comes with its own (simple) editor and synthesizer/sampler.
* It works with Linux (JACK or PipeWire), Windows 10/11 (WASAPI), and macOS.

## Limitations

* While it recently feels more like a "real" programming language, it's still far from being "turing-complete".
* It loads all your samples to memory, so if you have a lot of samples, make sure you have enough RAM.
* While some work has been done on more complex synthesis, it's still primarily focused on using samples.
* Except for some basic note names, Mégra doesn't reflect music theory, western or otherwise. There's no scales, scale completions, chords, chord completions or tunings, nor any helpers to work with functional harmony. Maybe there never will be. 
* The editor is fairly primitive (you can use it in REPL mode and integrate in other editors if you want).

These issues are being addressed in no particular order ...

## Documentation

The (english) documentation can be found here:

https://megra-doc.readthedocs.io/en/latest/

It contains:

* Installation & Configuration Info
* Tutorial
* Reference

## Questions & Feedback 

If you have questions, suggestions, or think the documentation could be improved, please open a ticket 
in the documentation repository: 

[https://github.com/the-drunk-coder/megra-doc/issues](https://github.com/the-drunk-coder/megra-doc/issues)

If you found a bug, or have comments or suggestions regarding Mégra itself, please open a ticket in the 
main repository: 

[https://github.com/the-drunk-coder/megra.rs/issues](https://github.com/the-drunk-coder/megra.rs/issues)

Don't hesitate to ask any question or post any comment, there's no threshold! 

You can also ping me on Mastodon: https://social.toplap.org/@megra

**If you made a track with Mégra**, you can also post it in a ticket on Github, and I'll collect them 
in a meta-ticket :). Or you can mention/tag Mégra on Mastodon (see above).

If you want to ask something non-publicly, write me an email! You can find the address in
the `Cargo.toml` file!
