# ADR: rexistros de decisións de arquitectura

Un **ADR** (*Architecture Decision Record*) é un documento curto que garda unha decisión importante do proxecto: o contexto no que se tomou, a decisión, as alternativas descartadas e as súas consecuencias. Serve para entender, meses despois, por que o código é como é, e para cambiar unha decisión de forma consciente, sabendo que se perde e que se gaña.

## Regras

- Un ADR por decisión, cun número de catro cifras (`0001`, `0002`…) que nunca se reutiliza.
- Cada ADR segue o [modelo](0000-modelo.md): contexto, decisión, alternativas, consecuencias e referencias.
- Un ADR acéptase ao integrar o PR que o introduce.
- Un ADR aceptado non se reescribe. Se a decisión cambia, escríbese un ADR novo que a substitúe, e o antigo pasa ao estado *Substituído polo ADR NNNN*. Só se corrixen erratas e ligazóns.
- Se un ADR novo só precisa ou cambia unha parte dun anterior, normalmente ao implementalo, o anterior segue vixente e pasa ao estado *Aceptado; concretado polo ADR NNNN*. O ADR novo explica que parte concreta e por que.
- Estados posíbeis: *Proposto*, *Aceptado*, *Aceptado; concretado polo ADR NNNN*, *Substituído polo ADR NNNN* e *Descartado*.

## Índice

| ADR | Decisión | Estado |
|---|---|---|
| [0001](0001-rust.md) | Rust como linguaxe | Aceptado |
| [0002](0002-windows-e-renderizado-cpu.md) | Só Windows, con renderizado por CPU | Aceptado; concretado polo [0009](0009-xanela-que-non-se-ve.md) |
| [0003](0003-nucleo-vt-alacritty-terminal.md) | `alacritty_terminal` como núcleo, detrás dunha fachada | Aceptado |
| [0004](0004-conpty-empaquetado.md) | ConPTY empaquetado con meiga | Aceptado |
| [0005](0005-fios-e-fotogramas.md) | Modelo de fíos e fotogramas | Aceptado; concretado polo [0008](0008-temporizador-dos-fotogramas.md) |
| [0006](0006-licenzas-e-dependencias.md) | Licenzas e dependencias | Aceptado |
| [0007](0007-integracion-coa-shell.md) | Integración coa shell | Aceptado |
| [0008](0008-temporizador-dos-fotogramas.md) | Temporizador dos fotogramas | Aceptado |
| [0009](0009-xanela-que-non-se-ve.md) | Que se debuxa cando a xanela non se ve | Aceptado |

## Como se relacionan

```mermaid
graph TD
    A1["0001 Rust"] --> A3["0003 alacritty_terminal"]
    A1 --> A6["0006 Licenzas"]
    A2["0002 Windows e CPU"] --> A4["0004 ConPTY"]
    A2 --> A5["0005 Fíos e fotogramas"]
    A2 --> A9["0009 Xanela que non se ve"]
    A3 --> A7["0007 Integración coa shell"]
    A4 --> A5
    A5 --> A8["0008 Temporizador dos fotogramas"]
    A5 --> A9
    A6 --> A3
    A6 --> A4
```

Cada frecha significa «condiciona»: por exemplo, a política de licenzas (0006) limita que núcleo de terminal (0003) e que ConPTY (0004) se poden usar.
