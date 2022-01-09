# megra.rs

(English version: [README.md](https://github.com/the-drunk-coder/megra.rs/blob/main/README.md))

Mégra es un lenguaje de programación específico de dominio (*domain-specific language* en inglés, o DSL, para abreviar) diseñado para **programar música en vivo** 
con elementos estocásticos. Su predecesor se implementó en Common Lisp, ¡este está implementado en Rust puro!

Este archivo LEAME debe contener toda la información necesaria para poner Mégra en funcionamiento.

## Tabla de contenido

* [AVISO](#warning)
* [Características](#características)
* [Limitaciones](#limitaciones)
* [Instalación](#instalación)
* [Configuración Audio](#configuratión-audio)
* [Encontrar y Usar Samples](#encontra-y-usar-samples)
* [Sketchbook](#sketchbook)
* [Ejecutar Mégra y Opciones Inicial](#ejecutar-mégra-y-opciones-inicial)
* [Aprender Mégra](#aprender-mégra)

## AVISO

¡Este programa todavía se encuentra en una etapa de desarrollo relativamente temprana y tiene algunas limitaciones! No ha sido
Excesivamente probado en todas las plataformas.

## Characterísticas

* ¡Te permite hacer música con cadenas de Markov!
* Sigue un paradigma de *sequencing* en lugar de un paradigma de sintetizador modular.
* Viene con su propio editor (sencillo).
* Funciona con Linux (JACK o PipeWire), Windows 10/11 (WASAPI) y macOS.

## Limitatciones
* No es un lenguaje de programación *turing-complete*.
* Carga todas tus *samples* en la memoria, así que si tienes muchas muestras, asegúrate de tener RAM suficiente.
* Se centra en *samples*. La síntesis es bastante primitiva y está limitada a algunas formas de onda básicas en este punto.
* Actualmente no le permite crear sintetizadores sofisticados a menos que desee codificarlos en Rust.
* El editor es bastante primitivo (puede usarlo en modo REPL e integrarlo en otros editores si lo desea).

Estos problemas se están abordando sin ningún orden en particular.

## Instalación
¡Actualmente todavía se necesita `rustc` y `cargo` instalados!

Para instalar esto, vaya a esta página: https://www.rust-lang.org/es/learn/get-started .
Esto lo guiará en el proceso de instalación de las herramientas necesarias para su sistema operativo.

En Windows, eso es un poco molesto porque necesita algunos componentes de VisualStudio que
podrían ocupar mucho espacio.

Cualquier versión de Rust superior a 1.57 (versión estable a principios de 2022) debería funcionar (probada por última vez con 1.57).

Mi objetivo es proporcionar binarios precompilados más adelante.

### Instalar con Cargo
En una terminal, escriba:

```
cargo install megra.rs
```

En Windows, escriba:

```
cargo install megra.rs --features ringbuffer
```

La función *ringbuffer* asegura que pueda usar varios tamaños de bloques (*blocksize*). Sin este función, el tamaño del bloque se fija en 512. Si puede
controlar el tamaño de bloque en su sistema (con JACK y CoreAudio eso es típicamente posible), puede usar la versión sin
*ringbuffer*. Si no está seguro, utilice el *ringbuffer*. Tiene una pequeña penalización en el rendimiento, pero no debería importar en sistemas modernos.

## Configuración Audio

Como se mencionó, la versión *default* de Mégra solo funciona en un *blocksize* fijo de 512, así que si esa es la versión que instaló, asegúrese de
el tamaño de bloque de su sistema es de 512. Cualquier *samplerate* debería funcionar, pero tenga en cuenta que para todas los *samples* que utilice se hace un *resampling* al *samplerate* acutal si los *samplerates* no coinciden. Esto podría aumentar el tiempo de carga. Recomiendo utilizar el *samplerate* en la que se almacenan las *samples*.

## Encontrar y Usar Samples

Si aún no tiene una collección de *samples* a mano, puede encontrar algunas (suficientes para seguir la documentación) aquí:
https://github.com/the-drunk-coder/megra-public-samples

Mégra actualmente solo admite *samples* en formato **FLAC**.

Coloque los *samples* en la carpeta (la carpeta se creará en el primer inicio):

* Linux: `~/.config/megra/samples`.
* Windows: `C:\Users\<username>\AppData\Roaming\parkellipsen\megra\config\samples`
* macOS: `/Users/<username>/Library/Application Support/de.parkellipsen.megra/samples`

Ahora tendrás un evento de sonido para cada *sample*. Eso significa que, si tiene una carpeta llamada `bd` en la carpeta de *samples*, puede llamarlo así:

```(lisp)
(once (bd))
```
También se puede buscar por palabra clave ... si tiene una muestra llamada `jazz.flac` en su carpeta `bd`, puede llamarla así:

```(lisp)
(once (bd 'jazz))
```

Si no se proporciona ninguna palabra clave, se elige un *sample* aleatorio de la carpeta.

También puede cargar *samples* individuales a un *set* a mano usando `(load-sample: set '<set>: path" <path-to-sample> ")`.

Como se mencionó anteriormente, asegúrese de configurar su sistema de audio con el *samplerate* de sus *samples*, de lo contrario, la carga de las muestras será lenta debido al *resampling*.

## Sketchbook

Los archivos generados y leídos por el editor se pueden encontrar en:

* Linux: `~/.config/megra/sketchbook`.
* Windows: `C:\Users\<username>\AppData\Roaming\parkellipsen\megra\config\sketchbook`
* macOS: `/Users/<username>/Library/Application Support/de.parkellipsen.megra/sketchbook`

## Ejecutar Mégra y Opciones Inicial

Ya sea que comience con carga o desde una terminal o un programa de lanzamiento, las opciones son:

```
-v, --version       imprimir versión
-r, --repl          no editor, solo repl (i.e. para integración con otros editores)
-h, --help          imprimir este ayudo
-n, --no-samples    no cargue los samples de la carpeta
-o, --output-mode   modo (stereo, 2ch, 8ch), default: stereo
-l, --list-devices  listar tarjetas de sonido
-d, --device        escoger tarjeta de sonido
--live-buffer-time  capacidad del bufer de entrada, default: 3
--font              editor font (mononoki, ComicMono or custom path, default: mononoki)
--reverb-mode       modo de reverberacion, convolution o freeverb (default/fallback es freeverb). para convolution, se debe proporcionar un IR
--reverb-ir         archivo impulse response, formato FLAC
```

Si se utiliza la opción `-r`, Mégra se inicia en modo REPL (línea de comandos). Si desea integrarlo en su editor favorito, puede ser útil.

## Aprender Mégra

Ahora que debería tener todo en funcionamiento, es hora de aprender a usar este idioma, ¿verdad? ¡Aquí es donde puede empezar!

### Tutorial

La carpeta `documentation` contiene un archivo llamado `es/tutorial/megra_tutorial_basico.megra3`. Puede poner eso en la carpeta *sketchbook* (ver arriba) y
siga directamente el tutorial. Es posible que haya más archivos allí que pueda seguir más tarde.

### Ejemplos

También hay una carpeta llamada `documentation/es/ejemplos/`, una selección creciente de ejemplos interesantes.

### Documentación de referencia (inglés)

La carpeta de documentación contiene dos archivos *markdown*, llamados `documentation/en/Function_List.md`, que es un proyecto en curso para documentar todas las funciones
que Mégra ofrece actualmente. También debería haber un ejemplo para jugar con cada función.

Con respecto a los eventos de sonido, hay un archivo llamado `documentation/en/Sound_Events_and_Parameters.md` que debería cubrir todos los eventos de sonido disponibles actualmente.




