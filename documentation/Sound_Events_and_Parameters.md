# Mégra Reference - Sound Events and Parameters

* [Sample Events](#sample-events) - play samples
* [Simple Synth Events](#simple-synth-events) - simple waves (sine, square etc)
* [Risset Event](#risset-event) - Risset bells
* A [Note](#a-note-about-note-names) about Note Names

## Sample Events

**Syntax**: 
```lisp 
(<sample-type> <keywords> <keyword parameters>)
```

**Example** 
```lisp
;; choose the 808 sample from the bd folder
(bd 'bd808 :lpf 1000 :rate 0.9)
```

### Parameters

| Parameter | Default | Description |
|-----------|:-------:|:-----------:|
| `:lvl`       | 0.3     | gain level |
| `:rate`      | 1.0     | sample playback rate |
| `:start`     | 0.0     | start within sample file, ratio |
| `:atk`       | 5       | gain envelope attack, in ms |
| `:rel`       | 5       | gain envelope release, in ms |
| `:sus`       | -       | gain envelope sustain, in ms|
| `:pos`       | 0.0      | stereo position (-1.0 left, 0.0 center 1.0 right) or channel number in multichannel mode |
| `:lpf`   | 19000   | lowpass filter frequency  |
| `:lpq`      | 0.4     | lowpass filter q factor |
| `:lpd`   | 0.0     | lowpass filter distortion|
| `:hpf`   | 20      | highpass filter frequency  |
| `:hpq`      | 0.4     | highpass filter q factor |
| `:pff`   | 1000    | peak filter frequency  |
| `:pfq`      | 10      | peak filter q factor |
| `:pfg`   | 0.0     | peak filter gain |
| `:rev`       | 0.0     | reverb amount |
| `:del`      | 0.0     | delay amount |

## Simple Synth Events 

**Syntax**: 
```lisp 
(sine|saw|sqr|cub|tri <pitch> <keyword parameters>)
```

**Example** 
```lisp
(sine 110) ;; with frequency
(sine 'a2 :rev 0.1) ;; with note name and reverb
```

### Types
| Type |Description|
|-----------|:-------:|
| sine | simple sine wave |
| cub  | a sine like shape made of two cubic pieces (LFCub) |
| tri  | a triangle wave |
| sqr  | a square wave   |
| saw  | a sawtooth wave |

### Parameters

| Parameter | Default | Description |
|-----------|:-------:|:-----------:|
|  pitch     | 43     | pitch - might be frequency in hertz or quoted note name |
| `:lvl`       | 0.3     | gain level |
| `:atk`       | 5       | gain envelope attack, in ms |
| `:rel`       | 5       | gain envelope release, in ms |
| `:sus`       | -       | gain envelope sustain, in ms |
| `:pos`       | 0.0     | (see above) |
| `:lpf`   | 19000   | lowpass filter frequency  |
| `:lpq`      | 0.4     | lowpass filter q factor |
| `:lpd`   | 0.0     | lowpass filter distortion|
| `:rev`       | 0.0     | reverb amount |
| `:del`      | 0.0     | delay amount |
| `:pw`        | 0.5     | pulsewidth (ONLY `sqr`) |

## Risset Event

A simple risset bell event.

**Syntax**: 
```lisp 
(risset <pitch> <keyword parameters>)
```

**Example** 
```lisp
(risset 2000) ;; with frequency
(risset 'a5 :rev 0.1) ;; with note name and reverb
```
### Parameters

| Parameter | Default | Description |
|-----------|:-------:|:-----------:|
|  pitch     | 43     | pitch - might be frequency in hertz or quoted note name |
| `:lvl`       | 0.3     | gain level |
| `:atk`       | 5       | gain envelope attack, in ms |
| `:dec`       | 20       | gain envelope decay, in ms |
| `:sus`       | 50       | gain envelope sustain, in ms |
| `:rel`       | 5       | gain envelope release, in ms |
| `:pos`       | 0.0     | see above |
| `:lpf`   | 19000   | lowpass filter frequency  |
| `:lpq`      | 0.4     | lowpass filter q factor |
| `:lpd`   | 0.0     | lowpass filter distortion|
| `:rev`       | 0.0     | reverb amount |
| `:del`      | 0.0     | echo amount |



## A Note about Note Names
Note names follow the Common Music 2.x convention, where `'a4` is the concert pitch of 440hz, above the *middle c* which is `'c4`. `'cs4` denotes a sharp note, `'cf4` denotes a flat note. The sharp/flat schema is consistent over all notes.
