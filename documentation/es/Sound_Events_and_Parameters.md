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
| `:tags`     |none|  etiquetas adicionales |


## Live Buffer Events

Los eventos de búfer en vivo le permiten trabajar con el búfer de entrada en vivo. El búfer de entrada en vivo está grabando continuamente los últimos 3 segundos
(o más o menos, si se especifica al inicio) de sonido del primer canal de entrada. Existen varios eventos que te permiten trabajar con eso.

### `feedr` - leer el búfer en vivo como una muestra regular

Usando `(feedr)`, puede leer directamente desde el búfer en vivo. Todos los parámetros son los mismos que en los eventos de *samples* anteriores. Ten cuidado
sobre retroalimantación si tienes un micrófono abierto!

*Ejemplo*:
```lisp
(sx 'ba #t ;; leer desde el búfer en vivo en diferentes puntos de inicio
  (nuc 'fa (feedr :start (bounce 0.01 0.99))))
```

### `freeze` - freeze the live buffer

Este evento escribe el búfer en vivo actual (tal cual) en un búfer persistente especificado por un número, como `(freeze 1)`.
Si usa esto en un evento `ctrl`, puede actualizar periódicamente el contenido del búfer congelado.

```lisp
;; congelar una vez, al búfer 1
(freeze 1)

;; congelar periódicamente, cada 6 segundos
(sx 'ba #t 
  (nuc 'ta :dur 6000 (ctrl (freeze 1))))
```

### `freezr` - read from frozen buffers
Esto le permite leer desde el búfer previamente congelado con `freeze`. Puede usarlo como cualquier *sample* regular.
El primer argumento especifica el búfer desde el que se leerá: `(freezr <bufnum>)`

```lisp
(sx 'ba #t ;; sampling granular en búfer 1 ...
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
| `:tags`     |none|  etiquetas adicionales |

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
| `:tags`     |none|  etiquetas adicionales |


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


## Una nota sobre las nombre de notas
Los nombres de las notas siguen la convención Common Music 2.x, donde `'a4` es el tono de concierto de 440 Hz, por encima de la *do central* que es `'c4`. `'cs4` denota una nota sostenida, `'cf4` denota una nota bemol. El esquema sostenido/bemol es consistente en todas las notas.

## Una nota sobre las etiquetas de eventos
Cada evento contiene ciertas etiquetas por defecto, como el tipo de evento y las etiquetas de búsqueda en el caso de eventos de muestra.
Como habrás visto, puedes agregar etiquetas personalizadas. Todas las etiquetas se pueden usar para filtrar en los respectivos aplicadores o modificadores,
así como solo y silenciar en el contexto de sincronización. Aquí hay un ejemplo:

```lisp
(sx 'ba #t :solo 'bass ;; <-- puede solo o bloquear según las etiquetas 
	(nuc 'fa (bd :tags 'drums) (sn :tags 'drums) (saw 100 :tags 'bass)))

(sx 'ba #t
	(pear :for 'drums (rev 0.2) ;; <-- solo aplica reverberación a la batería
		(nuc 'fa (bd :tags 'drums) (sn :tags 'drums) (saw 100 :tags 'bass))))
```
