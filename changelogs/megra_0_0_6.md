# Changes for Mégra Version 0.0.6:
* add support for WAV files
* automatically downmix stereo/multichannel files to mono for samples
* allow sample folders that start with a digit, but prepend underscore
* wavetable-based sawtooth (experimental)
* all tests passing, immediate growth is fixed 
* `grown` function is brought back to life
* fixed `reverse`
* sample folder configurable at startup
* font size for line numbers and lines doesn't seem to go out of alignment anymore
* fm-based sawtooth synth
* fm-based squarewave synth
* fm-based triangle synth
* configurable filter modes for sampler (including no filter at all)
* 24db highpass- and lowpass filters available
* butterworth highpass- and lowpass filters with configurable order available
* pulsewidth modulators for fmsquare and fmsaw
* two configurable parametric eq bands for sampler, live-sampler and frozen-sampler
* configurable base path
* `nosketch` startup option to avoid sketch spam
* `ctrl` events now can trigger `once` commands
* more flexible envelopes (ADSR, custom, exp and log segments)
* very rudimentary midi input
* lower latency build (blocksize 128) through feature
* noise generators (brown, white)
