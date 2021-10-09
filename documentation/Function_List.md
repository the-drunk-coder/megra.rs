# Mégra Function Reference

Table of Contents
=================

**Generator Generators**:

* [cyc - Cycle Generator](#cyc---cycle-generator)
* [chop - Chop a sample](#chop---chop-a-sample)
* [friendship - Create Friendship (or Windmill) Generator](#friendship---create-friendship-generator)
* [flower - Create Flower Generator](#flower---create-flower-generator)
* [fully - Create Fully Connected Generator](#fully---create-fully-connected-generator)
* [infer - Infer Generator from Rules](#infer---infer-generator-from-rules)
* [nuc - Nucleus Generator](#nuc---nucleus-generator)
* [learn - Learn Generator from Distribution](#learn---learn-generator-from-distribution)

**Generator Modifiers**:

* [blur - Blur Probabilities](#blur---blur-probabilities)
* [discourage - Stir Up Generator](#discourage---stir-up-generator)
* [encourage - Consolidate Generator](#encourage---consolidate-generator)
* [grow - Enlarge Generator](#grow---enlarge-generator)
* [grown - Enlarge Generator n times](#grown---enlarge-generator-n-times)
* [haste - speed up evaluation](#haste---speed-up-evaluation)
* [life - Manipulate Generator](#life---manipulate-generator)
* [relax - Slow Down Generator](#relax---slow-down-generator)
* [rew - Rewind Generator](#rew---rewind-generator)
* [sharpen - Sharpen Probabilities](#blur---sharpen-probabilities)
* [shrink - Shrink Generator](#shrink---shrink-generator)
* [skip - Skip Events](#skip---skip-events)

**Generator Multiplexers**:

* [xdup - Multiply Generators with Modifiers](#xdup---multiply-generators-independently)
* [xspread - Multiply Generators with Modifiers, Spread over Loudspeakers](#xdup---multiply-generators-independently)
* [ls - Create Generator List](#xdup---multiply-generators-independently)

**Parameter Modifiers**:

* [brownian - Bounded Brownian Motion](#brownian---bounded-brownian-motion)   
* [env - Parameter Envelope](#env---parameter-envelope)
* [exh - Event Stream Manipulator](#exh---event-stream-manipulator)
* [fade - Parameter Fader](#fade---parameter-fader)
* [inh - Event Stream Manipulator](#inh---event-stream-manipulator)
* [bounce - Parameter Oscillator](#bounce---parameter-oscillator)

**Event Stream Modifiers**:

* [pear - Apply Modifiers](#pear---apply-modifiers)
* [apple - Event Stream Manipulator Probablity](#pprob---event-stream-manipulator-probablity)
* [every - Count-Based Generator Manipulators](#evr---count-based-generator-manipulators)

**Misc**:

* [cmp - Compose Generators](#cmp---compose-generators)
* [clear - Clear Session](#clear---clear-session)     
* [ctrl - Control Functions](#ctrl---control-functions)
* [sx - Event Sinks](#sx---multiple-event-sinks)

## `brownian` - Bounded Brownian Motion 

Define a bounded brownian motion on a parameter.

### Parameters

* lower boundary (float)
* upper boundary (float)
* `:wrap` (boolean) (t) - wrap value if it reaches lower or upper boundary
* `:step` (float) (0.1) - step that the parameter will be incremented/decremented

### Syntax

```lisp
(brownian <lower boundary> <upper boundary> :wrap <wrap> :step <step-size>)
```

### Examples
	
```lisp
(sx 'some #t
    (cmp (pear (rate (brownian 0.8 1.2)))
         (nuc 'bass (saw 120 :dur 200))))
```

## `chop` - Chop a sample

Chop a sample into parts, that will be played as a loop.

### Examples

```lisp
;; chop violin sample into 8 parts (each of which is 200ms long)
(sx 'some #t
  (chop 'chops 8 (violin 'a3 :dur 200))) 
```

## `clear` - Clear Session

Stops and deletes all present generators.

### Examples

```lisp
(sx 'some #t
  (cyc 'bear "bd ~ hats ~ sn ~ hats ~"))

(sx 'more #t :sync 'some
  (cyc 'bass "saw:100 ~"))

(clear) ;; clear everything
```

## `cmp` - Compose Generators

### Syntax
```lisp
(cmp <generators>)
```

### Examples

```lisp
;; The plain lisp approach would be:
(sx 'composed #t
    (pear (rev 0.1)
		(every :n 20 (haste 2 0.5)
			(cyc 'bl "bd ~ ~ sn ~ ~"))) )

;; this makes it somewhat inconvenient to add/disable certain parts.

;; With cmp, it can be re-written as:
(sx 'composed #t
    (cmp
		(pear (rev 0.1))
		(every :n 20 (haste 2 0.5))
		(cyc 'bl "bd ~ ~ sn ~ ~")
		))

;; now individual modifiers can easily be commented out

```

## `ctrl` - Control Functions

Executes any function, can be used to conduct execution of generators.

### Parameters

* function

### Syntax

```lisp
(ctrl <function>)
```

### Example

```lisp
;; define some parts
(defpart 'bass 	
		(nuc 'bass (saw 100)))

(defpart 'mid 
	(nuc 'midrange (saw 1000)))
	
(defpart 'treble 
	(nuc 'treble (saw 5000)))

;; Define a score, here as a learned one, even 
;; though any other generator might be used.
(sx 'ga #t 
	(learn 'ta 
		:events
		'a (ctrl (sx 'ba #t 'bass))
		'b (ctrl (sx 'ba #t 'mid))
		'c (ctrl (sx 'ba #t 'treble))
		:sample "ababaabbababcabaababaacabcabac"
		:dur 2400))
```

## `cyc` - Cycle Generator

Generates a cycle (aka loop) from a simple sequencing language.

### Parameters

* name - generator name
* sequence - sequence description
* `:dur` - default space between events 
* `:rep` - probability of repeating an event
* `:max-rep` - limits number of repetitions
* `:rnd` - random connection probability

### Syntax

```lisp
(cyc <name> :dur <duration> :rep <repetition probability> :max-rep <max number of repetitions> :rnd <random connection prob> <sequence>)
```

### Example 
```lisp
;; plain
(sx 'simple #t
  (cyc 'beat "bd ~ hats ~ sn ~ hats ~"))
```

![A plain beat](./beat_plain.svg) 

```lisp
;; with a 40% chance of repetition, 4 times at max
(sx 'simple t
    (cyc 'beat "bd ~ hats ~ sn ~ hats ~" :rep 40 :max-rep 4))
```
![A beat with repetitions](./beat_rep.svg)    

## `env` - Parameter Envelope

Define an envelope on any parameter. Length of list of levels must be one more than length of list of durations.
Durations are step based, so the absolute durations depend on the speed your generator runs at.

### Paramters

* `:v` or `:values` - level points on envelope path
* `:s` or `:steps` - transition durations (in steps)
* `:repeat` (boolean) - loop envelope 

### Syntax

```lisp
(env :values/:v <levels> :steps/:s <durations> :repeat <#t/#f>)
```

### Example

```lisp
(sx 'simple #t
	(cmp (pear (lvl (env :v 0.0 0.4 0.0 :s 20 30)))
         (cyc 'beat "bd ~ hats ~ sn ~ hats ~")))
```

## `every` - Count-Based Generator Manipulators

Every so-and-so steps, do something with the generator.
Does not work as a sound modifier (so far).

### Examples

```lisp
(sx 'simple #t
    (cmp (every :n 20 (skip 2) :n 33 (rewind 2)) ;; <- every 20 steps, skip 2, every 33, rewind 2
         (cyc 'beat "bd ~ hats ~ sn ~ hats ~")))
```

## `exh` - Event Stream Manipulator

Exhibit event type, that is, mute all other events, with a certain probability.

### Parameters

* probablility (int) - exhibit probablility
* filter (filter function) - event type filter

### Syntax
```lisp
(exh <probability> <filter>)
```

### Example
```lisp
(sx 'simple #t 
  (cmp (exh 30 'hats)
       (exh 30 'bd)
       (nuc 'beat (bd) (sn) (hats)))) 
```
## `fade` - Parameter Fader

Fade a parameter (sinusoidal).

### Syntax

`(fade <from> <to> :steps <steps>)`

### Example
```lisp

;; fade cutoff frequency
(sx 'osc #t
    (nuc 'ill (saw 300 :lp-freq (fade 300 2200 :steps 20))))

;; same, but a different position 
(sx 'osc #t
    (cmp
     (pear (lpf (fade 300 2200 :steps 4)))
     (nuc 'ill (saw 300))))

;; fade duration
(sx 'osc #t
    (nuc 'ill (saw 300) :dur (fade 100 400)))

;; fade probablility
(sx 'osc #tt
    (cmp
     (pear :p (fade 0 100) (lvl 0.0))
     (nuc 'ill (saw 300))))
```

## `friendship` - Create Friendship Generator

This creates a directed version of a Friendship- or Windmill graph.

### Syntax

`(friendship <name> <list of events>)`

### Parameters

* `name` - The generator name.
* `list of events` - a list of events (more than 3, odd number)

### Example

```lisp
(sx 'friend #t        
    (cmp
     (pear (atk 1) (rel 90) (dur 100) (rev 0.07))
     (friendship 'ship (saw 'a2) (saw 'c3) (saw 'e3) (saw 'b3) (saw 'd3) (saw 'f3) (saw 'c4))))
```

![A beat with repetitions](./friendship.svg)

## `flower` - Create Flower Generator

Create ... well, look at the examples.

### Syntax:
`(flower <name> [:layers <layers>] <events>)`

### Parameters:

* `name` - generator name
* `:layers` - number of layers
* `events` - list of events (will be padded to appropriate lenght if necessary)

### Examples

Flower with 1 layer:
```lisp
(sx 'a-rose-is-a t
    (flower 'rose (saw 100)
             (saw 200) (saw 300) (saw 400)
             (saw 150) (saw 300) (saw 450)
             (saw 180) (saw 360) (saw 540)))
```
![flower one](./flower_1layer.svg)

Flower with 2 layers:
```lisp
(sx 'a-rose-is-a t
    (flower 'rose :layers 2 (saw 100)
             (saw 200) (saw 300) (saw 400)
             (saw 150) (saw 300) (saw 450)
             (saw 180) (saw 360) (saw 540)))
```
![flower two](./flower_2layer.svg)

## `fully` - Create Fully Connected Generator

Each node follows each other node with equal probablity ... so basically a random generator.

### Syntax
```lisp
(fully <name> <list of events>)
```

### Example

```lisp
(sx 'full t
    (fully 'mel (saw 'a3) (saw 'f2) (saw 'c3) (saw 'e3)))

```

![Fully connected](./fully_connected.svg)    

## `grow` - Enlarge Generator

The growth algorithm allows adding information to an already existing generator.
It does so by picking an event the generator yielded in the past, shaking up the values
a little, and adding it to the generator following certain principles.

### Parameters

* `:var` (float) - variation factor (smaller -> less variation)
* `:method` (symbol) - growth method/mode (see below)
* `:durs` (list of ints) - durations to mix in
* `:rnd` (int) - chance to add random edges after growth

### Examples

```lisp
(sx 'al #t
	(every :n 10 (grow) 
		(nuc 'gae (shortri 120))))
```

### Modes

Each growth mode pushes the generator in a certain direction.

* `'default`
* `'triloop`
* `'quadloop`
* `'flower`
* `'loop` 

## `haste` - Speed Up Evaluation

Speed up evaluation for a specified number of steps, by a certain ratio.

### Examples

```lisp
(sx 'more #t
    (xspread     
     (cmp ;; this is another copy with modifiers ...
          (pear (freq-mul 3.0))
          (every :n 20 (haste 4 0.5))) ;; <- every 20 steps, double-time for four steps. Change to 0.25 for quadruple-time, etc
     ;; this is the "original" 
     (cyc 'one "tri:120 tri:90 tri:100 tri:120 ~ ~ ~ ~")))
```

## `inh` - Event Stream Manipulator

Inhibit event type, that is, mute event of that type, with a certain probability.

### Parameters

* probablility (int) - inhibit probablility
* filter (filter function) - event type filter

### Syntax

```lisp
(inh <probability> <filter>)
```

### Example

```lisp
(sx 'simple #t
  (cmp (inh 30 'hats)
       (inh 30 'bd)
       (inh 30 'sn)
       (nuc 'beat (bd) (sn) (hats))))
```

## `life` - Manipulate Generator

This is one of the more complex generator manipulations. To understand it, it's helpful to play around 
with the `(grow ...)` function first. What the `(life ...)` method does is basically the same, but automated
and bound to resources, in the fashion of a primitive life-modeling algorithm:

* There's a global pool of resources (which is just an abstract number).
* Each generator is assigned an amount of local resources (same as above).
* Each time the generator grows, it comes at a cost, which is subtracted first from the local, then from the global resources.
* If all resources are used up, nothing can grow any further.

Each symbol in the current alphabet is assigned an age (the number of times it has been evaluated), so at a certain, specified
age they can perish, freeing a certain amount of resources (which are added to the local resource pool).

Furthermore, if specified, the generator can be configured to "eat itself up" when a shortage of resources occurs. That means 
that an element will be removed before its time, freeing resources for further growth (which, again, are added to the local resources).

### Parameters

* growth cycle (int)
* average lifespan (int)
* variation (float)
* `:method` - growth method (see `(grow ...)`)
* `:durs` - list of possible durations to choose from
* `:apoptosis` (bool) - if `nil`, symbols never die
* `:autophagia` (bool) - if `t`, the generator will eat its own symbols to generate energy for further growth

```
;; define global resources
(global-resources 30000)
```

### Examples

The algorithm is quite configurable, but to use the default configuration, you can simply use:

```lisp
(sx 'the-circle #t
    (life 10 12 0.4 ;; first arg: growth cycle, second arg: average lifespan, third arg: variation factor
          (cyc 'of-life "tri:100 tri:120 ~ ~ tri:120 tri:180 ~ tri:200")))
```

This means that every ten steps the generator will grow, while the average lifespan of an element is 12 evaluations.
The a variantion factor of 0.4 will be applied when generating the new elements.

You can specify a growth method (see the paragraph on `(grow ...)` for details):

```lisp
(sx 'the-circle #t
    (life 10 12 0.4 :method 'flower
          (cyc 'of-life "tri:100 tri:120 ~ ~ tri:120 tri:180 ~ tri:200")))
```

To add some rhythmical variation, you can mix in other durations (chosen at random):

```lisp
(sx 'the-circle #t
    (life 10 12 0.4 :method 'flower :durs 100 200 200 200 400 100
          (cyc 'of-life "tri:100 tri:120 ~ ~ tri:120 tri:180 ~ tri:200")))
```

Another interesting way to use this is to juxtapose it with a static generator (note the reset flag is nil'd so
we can change parameters without starting from scratch every time):

```lisp
(sx 'the-circle #t
    (xspread
		(pear (pitch-mul 1.5) (life 10 12 0.4 :method 'flower :durs 100 200 200 200 200 400))
		(cyc 'of-life "tri:100 tri:120 ~ ~ tri:120 tri:180 ~ tri:200")))
```


## `nuc` - Nucleus Generator

Generates a one-node repeating generator, i.e. as a starting point for growing.

### Parameters

* name (symbol)
* event(s) (event or list of events) - events to be repeated
* `:dur` - transition duration between events

### Syntax

```lisp
(nuc <name> <event(s)> :dur <duration>)
```

### Example

```lisp
(sx 'just t
  (nuc 'bassdrum (bd) :dur 400))
```
![Just a Bassdrum](./nucleus.svg)    

## `bounce` - Parameter Oscillator

Define oscillation on any parameter. The oscillation curve is a bit bouncy, not really sinusoidal.

### Parameters 

* upper limit - upper limit for oscillation 
* lower limit - lower limit for oscillation 
* `:cycle` - oscillation cycle length in steps

### Syntax

```lisp
(bounce <upper limit> <lower limit> :cycle <cycle length in steps>)
```

### Example

```lisp
(sx 'simple #t
  (nuc 'beat (bd) :dur (bounce 200 600 :steps 80)))
```

## `pear` - Apply Modifiers

Appl-ys and Pears ...

## `relax` - Slow Down Generator

Slows down generator for a specified number of steps, by a certain ratio.

### Examples

```lisp
(sx 'more #t
    (xspread     
     (cmp ;; this is another copy with modifiers ...
          (pear (pitch-mul 3.0))
          (every :n 20 (relax 4 0.5))) ;; <- every 20 steps, half-time for four steps
     ;; this is the "original" 
     (cyc 'one "tri:120 tri:90 tri:100 tri:80 ~ ~ tri:120 tri:90 tri:100 tri:80 ~")))
```

## `rewind` - Rewind Generator

Re-winds the generator by a specified number of steps. The further unfolding might
be different from the previous one, obviously.

### Examples

```lisp
(sx 'more #t
    (xspread     
     (cmp ;; this is another copy with modifiers ...
          (pear (pitch-mul 3.0))
          (every :n 20 (rewind 2))) ;; <- every 20 steps, rewind 2 steps (only the copy)
     ;; this is the "original" 
	 (cyc 'one "tri:120 tri:90 tri:100 tri:80 ~ ~ tri:120 tri:90 tri:100 tri:80 ~")))
```

## `shrink` - Shrink Generator

## `skip` - Skip Events

## `sx` - Event Sink

### Example

```lisp
(sx 'simple #t
  (nuc 'beat (bd) :dur 400))

(sx 'simple2 #t :sync 'simple :shift 200
  (nuc 'beat2 (sn) :dur 400))
  
;; you can solo and mute by tag ...
  
(sx 'solo #t :solo 'bd ;; <-- solo all events tagged 'bd'
  (nuc 'snare (sn) :dur 400)
  (nuc 'hats (hats) :dur 400)
  (nuc 'bass (bd) :dur 400))

(sx 'solo #t :block 'bd ;; <-- block all events tagged 'bd'
  (nuc 'snare (sn) :dur 400)
  (nuc 'hats (hats) :dur 400)
  (nuc 'bass (bd) :dur 400))


```

## `learn` - Learn Generator from Distribution

### Example
```lisp
(sx 'from #t
    (learn 'data
           :events 'x (bd) 'o (sn) 'h (hats)
	       :sample 
		   "xoxoxoxox~~o~h~~~h~h~h~~h~h~~hhh~x~o~x~o~x~o~x
            ~o~xh~h~~hhh~x~o~x~o~x~o~xox~xox~xox~xoxo~xoxo
            ~xoxox~oooo~xxxx~xoxoxox~ohxhohxhohxhxhxhxhxhx
            hxhohohoh"))
```

## `infer` - Infer Generator from Rules

Infer a generator from arbitrary rules. Make sure every event has
at least one exit, otherwise the generator will stop.

Also, exit probablities for each node should add up to 100.

### Parameters

* `name` - generator name
* `:events` - event mapping
* `:type` - pfa type ('naive or 'pfa) (optional)
* `:rules` - transition rules - Format '((<source>) <destination> <probability> <duration (optional)>)

### Example

```lisp

(sx 'con #t 
  (infer 'duct :events 
    'a (ctrl (sx 'ga #t 'a1))
    'b (ctrl (sx 'ga #t 'a2))
    'c (ctrl (sx 'ga #t 'a3))
    'd (ctrl (sx 'ga #t 'a4))
    :rules 
    (rule 'a 'a 80 4800)
    (rule 'a 'b 20 4800)
    (rule 'aaaa 'c 100 4800)
    (rule 'b 'b 80 4800)
    (rule 'b 'c 20 4800)
    (rule 'bbbb 'd 100 4800)
    (rule 'c 'c 80 4800)
    (rule 'c 'd 20 4800)
    (rule 'cccc 'a 100 4800)
    (rule 'd 'd 80 4800)
    (rule 'd 'a 20 4800)
    (rule 'dddd 'b 100 4800)))

```
## `stop` - Stop Event Processing

Stop event processing without deleting generators, thus maintaining current state.

## `xdup` - Multiply Generators Independently

If you want to juxtapose (obvious reference here) a generator with a modified copy of itself,
without re-writing the whole generator. 

### Example
```lisp
(sx 'more #t
    (xdup
     (cmp ;; this is the copy with modifiers ...
      (pear (pitch-mul 2.0) (rev 0.1))
      (every :n 20 (haste 2 0.5)))     
     ;; this is the "original" 
     (cyc 'one "shortri:'f3 shortri:'a3 shortri:'c4 shortri:'e4 ~ ~ shortri:'f3 shortri:'a3 shortri:'c4 shortri:'e4 ~")))
```

## `xspread` - Multiply Generators Independently

If you want to juxtapose (obvious reference here) a generator with a modified copy of itself,
without re-writing the whole generator. As opposed to `xdup`, this one spreads the copies over
the available loudspeakers.

### Example
```lisp
(sx 'more #t
    (xspread
     (cmp ;; this is the copy with modifiers ...
          (pear (pitch-mul 2.0) (rev 0.1))
          (every :n 20 (haste 2 0.5)))
     (cmp ;; this is another copy with modifiers ...
          (pear (pitch-mul 4.02) (rev 0.1))
          (every :n 20 (haste 3 0.5)))     
     ;; this is the "original" 
     (cyc 'one "tri:'f3 shortri:'a3 shortri:'c4 shortri:'e4 ~ ~ shortri:'f3 shortri:'a3 shortri:'c4 shortri:'e4 ~")))
```



