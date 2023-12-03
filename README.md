# megra.rs

(Versión español: [LEAME.md](https://github.com/the-drunk-coder/megra.rs/blob/main/LEAME.md))

Mégra is a **domain-specific** programming **language** (DSL for short) designed for **live-coding music** with stochastic elements.
Its predecessor was implemented in Common Lisp, this one is implemented in pure Rust !

This readme should contain all the necessary information to get Mégra up and running.

## Table Of Contents

* [WARNING](#warning)
* [Features](#features)
* [Limitations](#limitations)
* [Documentation](#documentation)
* [Questions & Feedback](#questions--feedback)

## WARNING

This is still in a relatively early development stage and has some limitations! It hasn't been
excessively tested on all platforms.

It's also heavily guided by the author's artistic practice, so some design decsisions might seem
odd if you're familiar with other live coding systems.

## Features

* It lets you make music with Markov chains!
* It follows a sequencing paradigm rather than a modular synth paradigm.
* It comes with its own (simple) editor.
* It works with Linux (JACK or PipeWire), Windows 10/11 (WASAPI), and macOS.

## Limitations

* It isn't a turing-complete programming language. In fact it's mostly a bunch of hard-coded methods at this point.
* It loads all your samples to memory, so if you have a lot of samples, make sure you have enough RAM.
* It's focused on samples. Synthesis is pretty primitive and limited to some basic waveforms at this point.
* It currently doesn't allow you to create fancy synths unless you want to code them in Rust.
* Except for some basic note names, Mégra doesn't reflect (traditional) music theory. There's no scales, scale completions, chords, chord completions or tunings, nor any helpers to work with functional harmony. Maybe there never will be. 
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

**If you made a track with Mégra**, you can also post it there and I'll collect them 
in a meta-ticket :).

If you want to ask something non-publicly, write me an email! You can find the address in
the `Cargo.toml` file!
