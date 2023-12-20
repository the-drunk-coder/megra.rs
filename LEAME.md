# megra.rs

(English version: [README.md](https://github.com/the-drunk-coder/megra.rs/blob/main/README.md))

Mégra es un lenguaje de programación específico de dominio (*domain-specific language* en inglés, o DSL, para abreviar) diseñado para **programar música en vivo** 
con elementos estocásticos. Está implementado en Rust puro. Está fuertemente guiado por la práctica artística del autor, pero debería ser fácil de aprender para tod@s.

La [Documentacíon](#documentation) contiene toda la información necesaria para instalar y aprender Mégra.

## Tabla de contenido

* [Características](#características)
* [Limitaciones](#limitaciones)
* [Documentacíon](#documentation)
* [Preguntas & Feedback](#preguntas--feedback)

## Characterísticas

* ¡Te permite hacer música con cadenas de Markov!
* Sigue un paradigma de *sequencing* en lugar de un paradigma de sintetizador modular.
* Es un programa autocontenido con su propio editor (sencillo) y synthe/sampler.
* Funciona con Linux (JACK o PipeWire), Windows 10/11 (WASAPI) y macOS.

## Limitaciones
* Mietras aparece más como un lenguaje de verdad recientemente, no es un lenguaje de programación *turing-complete*. 
* Carga todas tus *samples* en la memoria, así que si tienes muchas muestras, asegúrate de tener RAM suficiente.
* Mientras se ha trabajado en la síntesis, todavía se centra en *samples*. 
* Excepto por algunos nombres básicos de notas, Mégra no refleja la teoría musical, ni occidental ni otra. No hay escalas, terminaciones de escala, acordes, terminaciones de acordes o afinaciones, ni ayudantes para trabajar con armonía funcional. Tal vez nunca lo haya. 
* El editor es bastante primitivo (puede usarlo en modo REPL e integrarlo en otros editores si lo desea).

Estos problemas se están abordando sin ningún orden en particular.

## Documentación

La documentación (inglés) se puede encontrar aquí:

https://megra-doc.readthedocs.io/en/latest/

La documentación (español) se puede encontrar aquí, pero esta un poco atrasado:

https://megra-doc.readthedocs.io/es/latest/

Contiene:
* Información sobre installación & configuración
* Tutoriál
* Documentación de referencia 

## Preguntas & Feedback 

Si tiene preguntas, sugerencias o cree que la documentación podría mejorarse, abra un ticket
en el repositorio de documentación:

[https://github.com/the-drunk-coder/megra-doc/issues](https://github.com/the-drunk-coder/megra-doc/issues)

Si encuentra un error o tiene comentarios o sugerencias sobre Mégra, abra un ticket en el
repositorio principal:

[https://github.com/the-drunk-coder/megra.rs/issues](https://github.com/the-drunk-coder/megra.rs/issues)

No dudes en hacer cualquier pregunta o publicar cualquier comentario.

**Si hiciste un track con Mégra**, también puedes publicarlo allí y yo lo recogeré
en un meta-ticket :).

Si quieres preguntar algo de forma no pública, ¡escríbeme un correo electrónico! Puedes encontrar la dirección en
el archivo `Cargo.toml`!
