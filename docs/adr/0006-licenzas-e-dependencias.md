# ADR 0006 · Licenzas e dependencias

- **Estado:** Aceptado
- **Data:** 2026-11-06
- **Relacionados:** [ADR 0003](0003-nucleo-vt-alacritty-terminal.md), [ADR 0004](0004-conpty-empaquetado.md)

## Contexto

- O código de meiga é público pero con todos os dereitos reservados, e queda aberta a posibilidade de comercializalo (mellora futura F5).
- As licenzas *copyleft* (GPL, LGPL, AGPL e, a nivel de arquivo, MPL) poden obrigar a publicar código ou a cumprir condicións que complican un produto pechado.
- Cada dependencia engade código ao binario, tempo de compilación e risco na cadea de subministración (unha versión comprometida, un mantedor que abandona o proxecto…).

## Decisión

- **Só licenzas permisivas:** Apache-2.0 (tamén coa excepción LLVM), BSD-2-Clause, BSD-3-Clause, BSL-1.0, ISC, MIT, MIT-0, Unicode-3.0 e Zlib. `cargo-deny` bloquea calquera outra en cada PR, tamén nas dependencias indirectas.
- **Avisos de terceiros** xerados con `cargo-about` e incluídos en cada paquete. O ConPTY (MIT, [ADR 0004](0004-conpty-empaquetado.md)) engádese cando se empaquete (subfase 1.4).
- **Dependencias mínimas e xustificadas:**
  - previstas: `alacritty_terminal` (núcleo do terminal), `windows` e `windows-sys` (APIs de Windows), `serde` e `toml` (configuración e sesión) e `memchr` (escáner de secuencias);
  - só para probas e medicións: `criterion` e `proptest`;
  - excluídas: *runtimes* asíncronos, *frameworks* de interface e crates de imaxe.
- Cada dependencia nova xustifícase no PR que a introduce.
- O repositorio mantén "All rights reserved" ata que se decida a licenza do produto (F5).

## Alternativas consideradas

| Alternativa | Por que se descarta |
|---|---|
| Admitir MPL-2.0 | O seu *copyleft* é por arquivo: en moitos casos é compatíbel cun produto pechado, pero obriga a publicar os cambios nos arquivos MPL e require revisar cada caso. Mellor evitalo de entrada. |
| Publicar meiga xa cunha licenza aberta | Pecharía opcións comerciais antes de tempo. A decisión queda para F5. |
| Revisar as licenzas a man | Non escala coas dependencias indirectas, que poden cambiar en cada actualización. |

## Consecuencias

**Positivas:**

- Liberdade para distribuír, e mesmo vender, meiga sen obrigas de publicar o código.
- A revisión é automática: unha dependencia cunha licenza non permitida non pode entrar.
- Poucas dependencias: compilación máis rápida e menos superficie de ataque.

**Negativas e riscos:**

- Algúns crates quedan fóra. Mitigación: buscar alternativas ou escribir o código necesario.
- Hai que manter os avisos de licenza. Mitigación: xéranse automaticamente en cada paquete.

## Referencias

- `deny.toml` e `about.toml`, explicados en [Tecnoloxías › cargo-deny](../tecnoloxias.md#9-cargo-deny) e [Tecnoloxías › cargo-about](../tecnoloxias.md#10-cargo-about-e-handlebars).
