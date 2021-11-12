# Mégra Reference - Sound Events and Parameters

* [Sample Events](#sample-events) - play samples
* [Live Buffer Events](#live-buffer-events) - live buffers
* [Simple Synth Events](#simple-synth-events) - simple waves (sine, square etc)
* [Risset Event](#risset-event) - Risset bells
* [Control Events](#control-events)
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


## Live Buffer Events

Live buffer events allow you to play with the live input buffer. The live input buffer is continously recording the last 3 seconds 
(or more or less, if specified at startup) of input of the first input channel. Now there's severaly events that allow you to 
work with it.

### `feedr` - read the live buffer like a regular sample
Using `(feedr)`, you can read from the live buffer directly. All the parameters are the same as in the sample events above. Be careful 
about feedback if you have an open mic !

*Example*:
```lisp
(sx 'ba #t ;; read from live buffer at varying starting points
  (nuc 'fa (feedr :start (bounce 0.01 0.99))))
```

### `freeze` - freeze the live buffer
This event writes the current live buffer (as-is) to a persistent buffer specified by a number, like `(freeze 1)`.
If you use this in a `ctrl` event, you can periodically update the content of the frozen buffer.

```lisp
;; freeze once, to buffer 1
(freeze 1)

;; freeze periodically, every 6 seconds
(sx 'ba #t 
  (nuc 'ta :dur 6000 (ctrl (freeze 1))))
```

### `freezr` - read from frozen buffers
This allows you to read from the buffer previously frozen with `freeze`. You can use it like any regular sample.
First argument specifies the buffer to be read from: `(freezr <bufnum>)`

```lisp
(sx 'ba #t ;; granular sampling on freeze buffer 1 ...
  (nuc 'ba :dur 100 (freezr 1 :start (bounce 0.0 1.0) :atk 1 :sus 100 :rel 100)))
```

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
| cub  | a sine like shape made of two cubic pieces (LFCub in SuperCollider) |
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


## Control Events

Control events allow you to schedule parts that you'd otherwise execute manually.

### Example

```lisp
;; Change between two loops 
(sx 'ba #t 
  (infer 'ta :events
    'a (ctrl (sx 'du #t (cyc 'bu "bd ~ sn ~"))) ;; <-- executing the sync context is automated
    'b (ctrl (sx 'du #t (cyc 'bu "cym cym cym cym")))
    :rules 
    (rule 'a 'b 100 1599)
    (rule 'b 'a 100 1599)))
```


## A Note about Note Names
Note names follow the Common Music 2.x convention, where `'a4` is the concert pitch of 440hz, above the *middle c* which is `'c4`. `'cs4` denotes a sharp note, `'cf4` denotes a flat note. The sharp/flat schema is consistent over all notes.
