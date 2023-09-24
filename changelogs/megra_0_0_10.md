# Changes for Mégra Version 0.0.10:

* introduce `progn`
* introduce `match`
* introduce `fun` (function definition)
* introduce `callback` (callback definition, same as `fun` but as a mnemonic)
* introduce `let` (variable definition), `defpart` now maps to `let` (no change from user perspective)
* introduce `print`
* introduce midi helpers `mtof`, `mtosym`, `veltodyn`
* introduce `concat` to concatenate symbols and strings
* introduce `map` struct, `pair` constructor and `insert` method
* add osc sender
* add some extra types (f64, i32, i64) for osc sender
* add osc receiver 
* osc callbacks (toplevel functions with args)
* much more flexible midi callback (toplevel functions with args)
* start midi port from language instead of from command line
* negative playback rates for samples (can't believe I didn't think about that before ...)
* (kinda) lazy evaluation for the arithmetic functions
* 16-channel mode
* arbitrary labels for `learn`
* allow defining event mappings for `learn` as map
* sync on `ctrl` events
