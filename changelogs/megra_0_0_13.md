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
* New language construct: `mapper`, a rudimentary event mapper that takes functions
* New langauge constructs: event getters to extract a parameter from an event `ev-param`, `ev-tag` and `ev-name`
* New language construct: `note` event (can be used to drive scoreviz in conjunction with `mapper`)
* New (non-lazy) arithmetic functions: `round`, `floor` and `ceil`
* `midi-mul`, `midi-add`, `midi-sub`, `midi-div`, to work with midi notes
* `to-string` to turn numbers and some other stuff into strings
* Language: line between function and macro is now very blurred, i.e. you can do `(fun (name) (fun (concat "to-" name) () (send-to name)))`, 
  where in the child function definition `name` will be replaced ... this may explode at any given moment !!
* Bugfix (Visualizer): a failed visualizer message doesn't crash scheduling thread anymore
* Improvement (Visualizer): sending visualizer messages in multiple batches for bigger generators
* Bugfix: `rep` not chrashing when generator contains childless states
* Synth: `freeze-add` for additive live looping
