# ADR 0003 · `alacritty_terminal` como núcleo, detrás dunha fachada

- **Estado:** Aceptado
- **Data:** 2026-11-06
- **Relacionados:** [ADR 0001](0001-rust.md), [ADR 0006](0006-licenzas-e-dependencias.md), [ADR 0007](0007-integracion-coa-shell.md)

## Contexto

O núcleo dun terminal ten tres pezas:

- o **parser** VT, que interpreta as secuencias de escape que veñen mesturadas co texto;
- a **grella**: a matriz de celas (carácter, cores e estilos) coa pantalla e o historial (*scrollback*);
- a **lóxica de terminal**: modos, cursor, rexións de desprazamento, pantalla alternativa, reaxuste das liñas ao cambiar o tamaño (*reflow*)…

Facelo ben leva moito traballo e está cheo de casos límite; facelo ademais rápido, aínda máis. `alacritty_terminal` é o núcleo do terminal Alacritty, coñecido polo seu rendemento: usa o parser `vte` e unha grella optimizada. Na versión 0.26.0 ten licenza Apache-2.0, e `vte` 0.15.0, Apache-2.0 ou MIT. A súa API é 0.x, así que pode cambiar entre versións.

## Decisión

- Usamos `alacritty_terminal` para o parser, a grella e a lóxica de terminal, cunha versión fixada.
- Só o módulo `meiga-core::term` usa os seus tipos. O resto de meiga fala cunha **fachada** propia: alimentar bytes, consultar as filas que cambiaron e recibir eventos (título, campá, respostas para o PTY, portapapeis…).
- Usamos o seu `Term` e o parser, pero non o seu bucle de eventos nin o seu PTY: meiga xestiona o ConPTY ([ADR 0004](0004-conpty-empaquetado.md)) e os fíos ([ADR 0005](0005-fios-e-fotogramas.md)).
- Antes do parser vai un escáner propio para as secuencias que `vte` non trata ([ADR 0007](0007-integracion-coa-shell.md)).

## Alternativas consideradas

| Alternativa | Por que se descarta, polo de agora |
|---|---|
| Un núcleo propio desde cero | Meses de traballo para chegar á corrección de Alacritty. Queda como posibilidade (mellora futura F4) se a memoria por cela o xustifica. |
| Só o parser `vte` e unha grella propia | Aforraría memoria, pero habería que reimplementar toda a lóxica de terminal. Mesma conclusión: F4. |
| O núcleo de WezTerm (`wezterm-term`) | Non se publica en crates.io: habería que depender do repositorio git de WezTerm, que a política de orixes de `cargo-deny` non permite. |
| libvterm (C) | Obriga a compilar C e a usar FFI: menos seguro e máis difícil de integrar e de compilar. |

## Consecuencias

**Positivas:**

- Un núcleo maduro e rápido desde o primeiro día.
- A fachada permite actualizar `alacritty_terminal`, ou substituílo, tocando un único módulo.

**Negativas e riscos:**

- A API cambia entre versións 0.x. Mitigación: versión fixada, a fachada e actualizacións en PR propios, con probas.
- Cada cela ocupa máis memoria ca nunha grella compacta. Mitigación: medila (subfase 9.4) e decidir F4 con datos.
- `alacritty_terminal` depende de crates para o seu propio PTY e bucle de eventos (por exemplo, `polling` e `miow`) que meiga non usará. Fan a compilación máis longa, pero non engaden traballo en execución, e o código que non se chama elimínase ao ligar.
- `vte` non trata OSC 7, 9;9 nin 133. Mitigación: o escáner previo ([ADR 0007](0007-integracion-coa-shell.md)).

## Referencias

- [`alacritty_terminal` en crates.io](https://crates.io/crates/alacritty_terminal): versión 0.26.0, Apache-2.0.
- `vte` v0.15.0, [`src/ansi.rs`](https://github.com/alacritty/vte/blob/v0.15.0/src/ansi.rs) (función `osc_dispatch`): os códigos OSC que trata.
- [Arquitectura › Fluxo de datos](../arquitectura.md#3-fluxo-de-datos-previsto)
