# Changes for Mégra Version 0.0.14:

* Language: `:shift` and `(shift)` to time-shift individual generators
* Bugfix: `freezr` resolves buffer numbers (again)
* Language: change `match` behaviour
* Language: something like error messages ...
* DSP: glitch-free (or at least less glitchy) live-looping
* Scoreviz Interface: articulations for notes
* Scoreviz Interface: syllables for notes
* Scoreviz Interface: repetition marks
* Language: `keep-state` and `defpart` now define a variable that can contain stateful generators
* Language: evaluate global variables in calls to user-defined functions
* Internals: better duration handling for `infer` (duration suffix trees)
* Internals: slightly different event/duration mapping (a symbol now refers to event+duration)
* Visualizer: correct off-by-one-state error
* Language: several ways to use different durations in `learn`
* Bugfix: local variables are again resolved for midi and osc
* Language: `add` and `sub` can now process various types and global vars
* Language: new numeric type u128
* Language: `now` statement to get time since unix epoch in milliseconds, as u128
* Language: `get` for maps
* Language: make `inh`/`inhibit` and `exh`/`exhibit` the minimalist modifiers they once were
* Language/Sound: a synchronized loop recorder - `looprec` 
* Language: commands to clear live buffers - `clearbufs`, `clearlive`, `clearfreeze`, `clearalllive`, `clearallfreeze`
* Language: allow keywords as parameter descriptors for `facts` and `vals`
