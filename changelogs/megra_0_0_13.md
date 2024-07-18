# Changes for Mégra Version 0.0.13:

* Visualizer: `:exclude` keyword to selectively exclude parts from being visualized
* Visualizer: using `ls` with `compose` now leads to individual visualizations for the composed markov chains
* Visualizer: correctly clearing composed, wrapped generators (no "leftovers")
* Samples: adding "exclude.txt" to sample folder allows excluding samples
* Language: `vals` can now define a row of lookup keys when provided the `'keys` symbol as second parameter
* Effects: add configurable bitcrusher with parameters `bcbits` `bcmode` `bcdown` `bcmix`
* Bugfix: sample parameters after sample number are now evaluated
* Synth: add a simple `BLIT` oscillator
* Bugfix: `pear` now clears probability (sets it back to 100) after `:for`, as it should be ...
* Improvement: allow sending OSC from scheduled control events
* Improvement: allow print in ctrl event
* Improvement: finally implement ctrl events in `step-part`
* Improvement: `step-part` with identifier AND symbol ...
* Improvement: `bpm` for ctrl events
* Improvement: `step-part` for ctrl events
* Improvement: `vec` for `nuc`

