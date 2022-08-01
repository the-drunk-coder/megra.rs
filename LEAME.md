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

Además está fuertemente guiado por la práctica artística del autor, por lo que algunas decisiones de diseño pueden parecer
extraño si está familiarizado con otros sistemas de *live coding*.

## Characterísticas

* ¡Te permite hacer música con cadenas de Markov!
* Sigue un paradigma de *sequencing* en lugar de un paradigma de sintetizador modular.
* Viene con su propio editor (sencillo).
* Funciona con Linux (JACK o PipeWire), Windows 10/11 (WASAPI) y macOS.

## Limitatciones
* No es un lenguaje de programación *turing-complete*. De hecho, es principalmente un montón de métodos fijos en este punto.
* Carga todas tus *samples* en la memoria, así que si tienes muchas muestras, asegúrate de tener RAM suficiente.
* Se centra en *samples*. La síntesis es bastante primitiva y está limitada a algunas formas de onda básicas en este punto.
* Actualmente no le permite crear sintetizadores sofisticados a menos que desee codificarlos en Rust.
* Excepto por algunos nombres básicos de notas, Mégra no refleja la teoría musical (tradicional). No hay escalas, terminaciones de escala, acordes, terminaciones de acordes o afinaciones, ni ayudantes para trabajar con armonía funcional. Tal vez nunca lo haya. 
* El editor es bastante primitivo (puede usarlo en modo REPL e integrarlo en otros editores si lo desea).

Estos problemas se están abordando sin ningún orden en particular.

## Documentación
La documentación (español) se puede encontrar aquí:

https://megra-doc.readthedocs.io/es/latest/

Contiene:
* Información sobre installación & configuración
* Tutoriál
* Documentación de referencia 
