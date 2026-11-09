# ADR 0012 · A fachada do terminal

- **Estado:** Aceptado; concretado polos [ADR 0013](0013-da-saida-a-pantalla.md) e [ADR 0015](0015-o-rato-a-seleccion-e-o-portapapeis.md)
- **Data:** 2026-11-08
- **Relacionados:** [ADR 0003](0003-nucleo-vt-alacritty-terminal.md), [ADR 0005](0005-fios-e-fotogramas.md), [ADR 0007](0007-integracion-coa-shell.md)

## Contexto

- O [ADR 0003](0003-nucleo-vt-alacritty-terminal.md) decide usar `alacritty_terminal` como núcleo do terminal, detrás dunha **fachada** propia: o resto de meiga dálle os bytes da saída, consulta as filas que cambiaron e recibe eventos. O [ADR 0007](0007-integracion-coa-shell.md) pon antes do parser un **escáner** propio para as secuencias que `vte` non trata. Na subfase 1.5 constrúese esa fachada. Antes de escribila, comprobouse no código de `alacritty_terminal` 0.26.0 e de `vte` 0.15.0:
  - **Os eventos.** `alacritty_terminal` avisa con `EventListener::send_event(&self, Event)`, e garda o receptor dentro do seu `Term`, sen ningunha función para lelo. Os que importan son as respostas para o programa (`PtyWrite`), o título (`Title` e `ResetTitle`), a campá (`Bell`) e o texto copiado co OSC 52 (`ClipboardStore`). As respostas ás preguntas polas cores (OSC 4, 10, 11 e 12) e polo tamaño en píxeles (`CSI 14 t`) necesitan a paleta e o tamaño da cela, que só coñece o renderizador. Os demais eventos son para o bucle de eventos de Alacritty, que meiga non usa.
  - **As filas que cambiaron.** `Term::damage` dá a pantalla enteira ou, para cada fila, as columnas que cambiaron. O texto que se escribe non se marca cela a cela: márcanse as posicións polas que pasa o cursor, e o rango entre elas. Algúns efectos secundarios non se marcan: escribir sobre unha metade dun carácter ancho borra a outra metade, e esa cela non aparece. Atopouno unha proba aleatoria que compara a copia do que cambia cunha copia enteira. Alacritty non o nota, porque debuxa a pantalla enteira en cada fotograma.
  - **Os modos.** `alacritty_terminal` segue os eventos de foco (`?1004`), pero ignora win32-input-mode (`?9001`), que o ConPTY activa ao arrancar.
  - **As actualizacións sincronizadas** (`CSI ? 2026 h` … `CSI ? 2026 l`): `vte` garda a saída que chega entre as dúas, ata 2 MiB, para mostrala de vez, ou ata que pasan 150 ms. Pero `vte` non ten temporizador: quen o usa ten que consultar a hora límite (`sync_timeout`) e rematala (`stop_sync`).
  - **A memoria.** Cada parser (`Processor`) reserva ao crearse 2 MiB para esa saída (`Vec::with_capacity(0x20_0000)`), aínda que o programa nunca use actualizacións sincronizadas. Son memoria reservada (*commit*) por cada tab; as páxinas non se tocan, así que non entran no *working set*.
  - **O tamaño.** Nunha grella dunha columna, un carácter ancho escríbese fóra da fila e o programa falla. `alacritty_terminal` define o mínimo: dúas columnas (`MIN_COLUMNS`) e unha fila.
  - **O cursor.** `renderable_content` dá o cursor tal como o debuxa Alacritty: sobre a metade dereita dun carácter ancho, pásao á columna anterior, sen comprobar que exista. Borrar caracteres (`CSI P`) pode deixar unha metade dereita soa na primeira columna, co cursor enriba: entón a resta baixa de cero, o que nas compilacións de depuración é un *panic* e en *release* fai desaparecer o cursor. Atopouno a revisión do código.
- O ConPTY 1.25 envía ao arrancar `ESC [ 1 t`, `ESC [ c` (a pregunta polos atributos do terminal), `ESC [ ? 1004 h` e `ESC [ ? 9001 h`, e ao rematar, `ESC [ ? 1004 l` e `ESC [ ? 9001 l`. O que escribe o programa pasa case sen cambios: os fotogramas dun *spinner* chegan tal como os escribe o script.

## Decisión

1. **Tipos propios.** `meiga-core::term` ofrece `Terminal` e os tipos do que se mostra: `Screen`, `Cell` (`CellChar`, `Color` e `CellFlags`), `Cursor`, `Modes` e `Changes`. Os tipos de `alacritty_terminal` non saen do módulo.
2. **Copiar o que cambia, e comprobalo.** meiga garda un `Screen` e o terminal cópialle o que cambiou desde a copia anterior (`copy_damage`): as filas enteiras que marcou `alacritty_terminal`, ou todas se marcou a pantalla enteira ou se o `Screen` tiña outro tamaño. Cada cela compárase coa que había, e só as distintas quedan marcadas como cambiadas. As celas do cursor, dúas por se está sobre un carácter ancho, márcanse onde estaba e onde está, só se se move ou cambia e só se se debuxa. O cursor lese da grella, non de `renderable_content`: sobre a metade dereita dun carácter ancho debúxase desde a esquerda só se a cela anterior é a súa metade esquerda, coma nas celas copiadas.
3. **Os eventos** van a unha cola que comparten o terminal e o receptor de `alacritty_terminal` (`Arc<Mutex<…>>`): o terminal ten que poder pasar dun fío a outro. `feed` devolve que pasou ao ler (`Changes`: respostas, título, campá e portapapeis), e as respostas recóllense con `take_responses` para envialas ao programa. Un título baleiro conta como ningún.
4. **As respostas** son as de `alacritty_terminal`: os atributos do terminal (DA1, `ESC [ ? 6 c`), o estado e a posición do cursor (DSR) e o tamaño en celas (`CSI 18 t`). As preguntas polas cores e polo tamaño en píxeles quedan sen resposta ata a subfase 1.6.
5. **OSC 52, só para copiar** (`Osc52::OnlyCopy`), e só ao portapapeis: Windows non ten selección primaria, así que `p` e `s` ignóranse. Un programa nunca pode ler o portapapeis.
6. **O escáner só segue `?9001`**, e non `?1004` como dicía o plan: `alacritty_terminal` xa segue os eventos de foco. Le as secuencias coma `vte`, para ver un cambio de modo só cando `vte` o vería, tamén partido entre dúas lecturas, e só segue byte a byte as que empezan por `ESC [ ?`: o resto sáltao buscando o seguinte `ESC` con `memchr`. Recibe os bytes como `&[u8]`, así que non os pode cambiar: o parser recibe os mesmos.
7. **As actualizacións sincronizadas** remátanse desde fóra: o terminal di a súa hora límite (`sync_deadline`) e móstraa cando se lle pide (`end_sync`). Na subfase 1.6, o temporizador dos fotogramas ([ADR 0008](0008-temporizador-dos-fotogramas.md)).
8. **O tamaño mínimo** é o de `alacritty_terminal`: dúas columnas e unha fila.
9. **A configuración:** 10 000 liñas de historial, que mostrará a subfase 1.9, e sen o protocolo de teclado de kitty: co ConPTY, as teclas van en win32-input-mode (subfase 1.7).
10. **Gravacións.** Un formato binario propio, `MEIGAREC` versión 1, garda a saída dun programa en bloques, tal como chegou, co momento de cada un. O exemplo `conpty` de `meiga-win` grava (`--record`), e o exemplo `replay` de `meiga-core` dálle unha gravación a un terminal e mide a que velocidade a le. As probas usan gravacións reais do ConPTY.
11. **Os 2 MiB de cada parser** acéptanse polo de agora. Decídese na subfase 2.2, coa memoria de varias tabs medida.

Isto concreta o [ADR 0003](0003-nucleo-vt-alacritty-terminal.md): que ofrece a fachada e como copia as filas que cambiaron.

## Alternativas consideradas

| Alternativa | Por que se descarta |
|---|---|
| Usar fóra do módulo os tipos de `alacritty_terminal` | O renderizador e as tabs dependerían dunha API 0.x: cada versión nova cambiaría código en varios *crates*. |
| Copiar só as columnas que marca `alacritty_terminal` | Quedan celas vellas na pantalla: a outra metade dun carácter ancho sobre o que se escribe. |
| Copiar e comparar a pantalla enteira en cada copia | O custo dependería do tamaño da pantalla, non do que cambiou: con un só carácter novo, revisaría miles de celas. |
| Marcar como cambiado todo o que copia | O renderizador debuxaría de máis: nun *spinner* que reescribe a súa liña, a liña enteira en vez dunha cela. |
| Marcar sempre as celas do cursor | Unha copia sen cambios xa non sería baleira, e un cursor agochado, como o dun *spinner*, faría debuxar dúas celas en cada fotograma. |
| Ler o cursor con `renderable_content`, coma o renderizador do Alacritty | Cunha metade dereita soa na primeira columna, a súa resta baixa de cero: un *panic* nas compilacións de depuración, e un cursor que desaparece en *release*. |
| A cola dos eventos nun `RefCell` | O terminal non se podería pasar a outro fío. |
| Que o escáner siga byte a byte todas as secuencias `CSI` | Moito máis lento: cunha saída chea de cores, case todas as secuencias son de cor. |
| Seguir tamén `?1004` no escáner | `alacritty_terminal` xa o segue. |
| Gravar en formato *asciicast* (asciinema) | Garda texto en JSON: necesita UTF-8 válido, e un bloque pode cortar un carácter ou unha secuencia en dous, que é xusto o que hai que probar. Ocupa máis e é máis lento de ler. |
| Probas con secuencias escritas a man | Non teñen o que o ConPTY engade nin como parte a saída. As escritas a man complementan as reais, non as substitúen. |
| Cambiar `vte` para non reservar os 2 MiB | Habería que manter unha copia propia de `vte`. Primeiro, medir canto supón con varias tabs (2.2). |

## Consecuencias

**Positivas:**

- O renderizador e, desde a fase 2, as tabs, só coñecen os tipos de meiga.
- Os cambios son exactos: só as celas que cambiaron. Nun *spinner*, unha cela por fotograma, e dúas cando cambian os segundos.
- Un fallo nas marcas de `alacritty_terminal` dentro dunha fila non deixa celas vellas na pantalla, e unha proba aleatoria compróbao en cada execución.
- As probas usan a saída real do ConPTY, e a velocidade do terminal mídese sempre igual coa mesma gravación.

**Negativas e riscos:**

- **2 MiB reservados por tab**, aínda que non se usen. Mitigación: medilos na subfase 2.2; se pesan, propoñerlle a `vte` que os reserve só cando faga falta.
- Comparar as filas enteiras custa en proporción ao seu ancho, uns centos de celas por fila, moito menos ca ler a saída.
- Sen resposta ás preguntas polas cores e polo tamaño en píxeles ata a 1.6: un programa que pregunte espera ata que se canse.
- Os cambios da paleta que pide un programa (OSC 4, 10 e 11) aínda non se aplican: `alacritty_terminal` gárdaos e marca a pantalla enteira, pero as celas gardan o número da cor, non a cor, así que non cambia ningunha. Chegan coa paleta, na subfase 1.6, que terá que redebuxalo todo cando cambie.
- Os atributos secundarios (DA2, `CSI > c`) respóndense coma en `alacritty_terminal`, coa súa versión: `ESC [ > 0 ; 2600 ; 1 c`.
- Aínda non se copian a cor do subliñado (`SGR 58`) nin os caracteres de ancho cero que se combinan co anterior (8.1), nin as ligazóns (`OSC 8`, 8.5).
- O escáner engade traballo por cada `ESC`. Mitigación: `memchr` e saltar as secuencias que non son privadas. O obxectivo de menos do 1 % do tempo do parser é da subfase 4.1, cando o escáner busque tamén OSC 7, 9;9 e 133.

## Referencias

- `alacritty_terminal` 0.26.0, `src/term/mod.rs`: `MIN_COLUMNS`, `Term::damage`, `write_at_cursor` (os caracteres anchos), `RenderableCursor::new`, `identify_terminal`, `device_status` e `clipboard_store`; `src/event.rs`: os eventos.
- `vte` v0.15.0, [`src/ansi.rs`](https://github.com/alacritty/vte/blob/v0.15.0/src/ansi.rs): `Processor`, `SyncState` (`SYNC_BUFFER_SIZE` e `SYNC_UPDATE_TIMEOUT`) e `StdSyncHandler`.
- [Tecnoloxías › `alacritty_terminal` e `vte`](../tecnoloxias.md#34-alacritty_terminal-e-vte-o-núcleo-do-terminal)
- [Código › meiga-core › `term`](../codigo/meiga-core.md#cratesmeiga-coresrctermmodrs)
