# Changes for Mégra Version 0.0.9:

* `cyc` repetition time handling has been fixed
* dynamic parameters for `facts` and `vals`
* new and more flexible ways to work with samples:
  * `keys`, `keys-add`, `keys-sub` function to replace keys along the pipeline
  * `random-sample` with shorthand `rands` to choose a different, random sample every time
  * `sample-number` (with `add`,`mul`,`sub` and `div`), with shorthand `sno` to modify sample numbers 
* corrections in markov chain learning
* short samples for `learn` don't cause a crash anymore
* fixed crash in wavematrix generation
* synth events now work properly without frequency argument
