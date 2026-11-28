# ADR 0032 · O escáner OSC

- **Estado:** Aceptado
- **Data:** 2026-11-27
- **Relacionados:** [ADR 0003](0003-nucleo-vt-alacritty-terminal.md), [ADR 0007](0007-integracion-coa-shell.md), [ADR 0012](0012-fachada-do-terminal.md), [ADR 0014](0014-o-teclado.md), [ADR 0029](0029-o-titulo-da-tab-e-os-titulos-gardados.md)

## Contexto

- **O que pide o plan.** A subfase 4.1 fai o escáner do [ADR 0007](0007-integracion-coa-shell.md): le OSC 7, 9;9 e 133, o cartafol e as marcas que escribe a shell no seu *prompt*, aínda que a secuencia chegue partida entre lecturas, con probas de anacos aleatorios feitas con `proptest` e un *benchmark*; e custa menos do 1 % do tempo de análise. Da 3.1 quedaron dúas preguntas ([ADR 0029](0029-o-titulo-da-tab-e-os-titulos-gardados.md), decisión 7): se OSC 1 e OSC 21 cambian o título, coma no Windows Terminal, e se o escáner pon un límite ao texto dun OSC, que `vte` garda enteiro mentres chega e conserva despois.
- **O que había.** O escáner do [ADR 0012](0012-fachada-do-terminal.md) só seguía win32-input-mode (`?9001`) e o reinicio (`ESC c`), que o apaga. Parábase en cada ESC: con saída chea de cores, a de `bigfile` (4,4 ESC cada 100 bytes), custaba o 3,2 % do tempo do terminal, e só percorrer os ESC un a un xa levaba arredor do 3 %. `vte` 0.15.0 non trata OSC 1, 7, 9, 21 nin 133; remata un OSC en BEL, en CAN, en SUB ou en calquera ESC, e nos catro casos o despacha; garda o seu texto sen límite e conserva esa memoria; e dentro dunha actualización sincronizada (`?2026`) analiza os bytes despois, ata 150 ms ou 2 MiB máis tarde. O ConPTY que leva meiga pasa os cinco OSC tal cal, en orde: probado cun programa de verdade. O de Windows traga OSC 1.
- **As referencias,** polo seu código ([Tecnoloxías › O cartafol e o prompt que di a shell](../tecnoloxias.md#68-o-cartafol-e-o-prompt-que-di-a-shell)):

| | OSC 7 | OSC 9;9 | OSC 1 e OSC 21 | O texto dun OSC |
|---|---|---|---|---|
| Windows Terminal 1.25 | Ignórao, polo WSL | A ruta ata o primeiro `;`, sen as comiñas | Os dous, o título | Sen límite |
| kitty 0.51 | De calquera equipo | Sen as comiñas, se é unha ruta de POSIX | OSC 21 son as cores | 256 KiB; un máis longo ignórase |
| WezTerm | De calquera equipo | Non | OSC 1, un título da icona aparte | Sen límite |
| Ghostty 1.4 | Só deste equipo | Si, aínda sen facer en Windows | OSC 21 son as cores | 2048 bytes ou 8 MiB |
| xterm 412 | — | — | OSC 1, o nome da icona; OSC 21, o título dun VT520 | 20 000 bytes; un máis longo ignórase |
| GNOME VTE 0.84 | De calquera equipo | — | — | 4096 caracteres; un máis longo ignórase |

## Decisión

1. **O escáner só busca os OSC** (decidido contigo): `?9001` e `ESC c` pasan ao *handler* de `vte` (`Hooked`), que xa os recibe na súa orde e sen custo; e o escáner busca o `]` con `memchr`, moitos bytes á vez, e mira se ten un ESC xusto diante, sen pararse nas outras secuencias nin no texto; nunha lectura con moitos `]` sen ESC, coma unha lista, busca a parella `ESC ]` con `memchr::memmem`, que non depende de cantos `]` haxa. Desde alí le cada OSC coma `vte`, tamén partido entre lecturas: os mesmos controis ignorados, o mesmo número, e o mesmo final, en BEL, CAN, SUB ou calquera ESC, que pode empezar outro OSC. Un OSC dun número que non pode ser 7, 9 nin 133, coma o 8 das ligazóns, sáltao alí mesmo. Só non ve un OSC con bytes entre o ESC e o `]` que non saen do escape en `vte`, coma `ESC \x80 ]`, que ningún programa escribe. Dentro dunha actualización sincronizada, `?9001` vale cando `vte` le a actualización, coma a pantalla; o ConPTY pono ao arrancar e despois de cada `ESC c`. Medido co *benchmark*, coma o le meiga, cada bloque aínda na caché despois de ReadFile, o escáner leva do tempo do terminal o 0,1 % con texto e coa saída de `bigfile`, o 0,25 % con listas e coa saída máis densa en cores, unha cor para cada carácter, e o 0,9 % coa resposta dunha IA gravada, de bloques pequenos, nos que pesa o reloxo que os mide.
2. **OSC 1 e OSC 21 non cambian o título** (decidido contigo), coma ata agora, e coma Alacritty, kitty e Ghostty. Para que o cambiasen na orde dos OSC 0 e 2 e de `CSI 22 t` e `23 t`, que trata `vte`, tamén dentro dunha actualización sincronizada, o escáner tería que cambiarlle os bytes a `vte` e reter o final dunha lectura; e en kitty e Ghostty, OSC 21 son as cores.
3. **Como moito 128 KiB do texto dun OSC** (decidido contigo): o escáner garda ata 128 KiB do texto de cada OSC 7, 9;9 ou 133, onde cabe a ruta máis longa de Windows, 32 767 caracteres, en UTF-8 e entre comiñas; un máis longo ignórao enteiro, sen usar nunca un anaco. Despois dun OSC longo, devolve a memoria que pasa de 4 KiB. `vte` segue recibindo todos os bytes, sen cambios, coma no Windows Terminal: a memoria do seu OSC máis longo queda ata que se pecha a tab.
4. **Que se le de cada un** (decidido contigo):
   - OSC 7: só `file://`; o equipo e a ruta, coas `%XX` descodificadas, e só se quedan en UTF-8; de calquera equipo, coma kitty, WezTerm e VTE, porque WSL e SSH dan o seu. Que facer cun que non sexa este equipo decídese na 4.5 e na 5.6.
   - OSC 9;9: a ruta sen as comiñas de arredor, coma o Windows Terminal e kitty, e todo o que vén despois de `9;9;`, tamén con `;`, que unha ruta de Windows pode levar. Os outros OSC 9, coma o progreso (9;4) ou as notificacións, non son cartafoles.
   - OSC 133: A, B, C e D, e de D, o código de saída se é un número de 32 bits, con signo ou sen el, coma o dá Windows: `3221225477` é o `i32` −1073741819. As outras letras (P, N, L e I) e as opcións (`aid=`, `k=`, `err=`…) ignóranse, coma pide a proposta dos *semantic prompts*.
5. **O estado, na fachada** (presentado contigo): `Terminal::shell()` dá o último cartafol, a última marca e o código de saída da última orde, que segue mentres se mostra o seguinte *prompt*; e `Changes::shell` avisa cando cambian: dicir o mesmo outra vez non é un cambio. Cambian ao ler a saída, antes de debuxala, tamén dentro dunha actualización sincronizada, e as marcas non levan fila. O reinicio da tab ([ADR 0024](0024-o-fin-do-programa-e-o-reinicio.md)) esquéceo, coma os títulos; `ESC c`, non. A xanela úsao desde a 4.5; ata entón, `replay --screen` móstrao.
6. **As probas e a medida** (presentado contigo; como mide o *benchmark*, decidido contigo despois de medilo): `proptest` 1.12, só con `std`, compara o escáner cun `Parser` de `vte` en saídas aleatorias partidas en anacos aleatorios; unha gravación de pwsh, cun *prompt* que escribe OSC 133 e 9;9, próbao con saída de verdade; e unha proba do ConPTY comproba que os pasa tal cal. O *benchmark*, sen dependencias e sen o arnés das probas, mide o escáner e o terminal coa mesma saída, rolda a rolda, e bloque a bloque, coma o fío lector dunha tab: cada bloque cópiase antes nun búfer, coma o escribe ReadFile, e os dous mídense nel, mentres está na caché. A CI execútao e mostra os números, sen límite, coma as medidas do arranque e dos fotogramas.

Isto concreta o [ADR 0007](0007-integracion-coa-shell.md) (o escáner só busca os OSC, e o seu custo medido), o [ADR 0012](0012-fachada-do-terminal.md) (`?9001` deixa o escáner, decisión 6) e o [ADR 0029](0029-o-titulo-da-tab-e-os-titulos-gardados.md) (OSC 1 e OSC 21 seguen sen facer nada, decisión 7).

## Alternativas consideradas

| Alternativa | Por que se descarta |
|---|---|
| Buscar `]` e mirar cara atrás ata o ESC, pasando os bytes que non saen do escape en `vte` | Decidido contigo: tamén ve un OSC con eses bytes despois do ESC, pero con texto cheo de `]`, coma listas ou JSON, custaba o 8,0 %, medido no estudo. O escáner busca tamén `]`, pero só mira o byte de antes, e nunha lectura con moitos pasa á parella. |
| O escáner de antes, ESC a ESC, cos OSC | Decidido contigo: con `bigfile`, o 3,2 %, e coa saída máis densa en cores, o 5,0 %; o obxectivo do 1 % pasaría a ser un 3–5 %. |
| Só `memmem`, para a parella `ESC ]` | Probouse primeiro: atopa o mesmo, pero máis amodo ca `memchr` cun só byte: cos bloques fríos, con cores levaba o 1,3 %, e o escáner de agora, o 1,1 %. |
| Un escáner con instrucións SIMD propias | `meiga-core` non leva código `unsafe`; e medido no estudo, cos bloques fríos e sen saltos, levaba o 1,2 % con `bigfile`, e o escáner de agora, medido igual, o 0,75 %. |
| Medir os bloques seguidos, fríos, coma a primeira versión do *benchmark* | Decidido contigo: o escáner, o primeiro en ler cada bloque, pagaba traelo da memoria, e en meiga leo despois de ReadFile, aínda na caché. Coa saída máis densa en cores, o 1,1 % fronte ao 0,25 %; con `bigfile`, o 0,75 % fronte ao 0,12 %. Medir bloque a bloque mete o reloxo en cada un, que pesa máis ca o escáner nos bloques pequenos. |
| OSC 1 coma título, coma o Windows Terminal e iTerm2 | Decidido contigo: habería que cambiarlle os bytes a `vte` para mantelo na orde dos outros títulos, e o ConPTY de Windows trágao. |
| OSC 1 e OSC 21 coma título, coma o Windows Terminal | Decidido contigo: ademais, en kitty e Ghostty OSC 21 son as cores, e unha pregunta de cores sería o nome da tab. |
| Limitar tamén `vte`: reter o texto de cada OSC e pasarlle só os de ata 1 MiB, coma xterm, VTE e kitty | Decidido contigo: os bytes que recibe `vte` deixarían de ser sempre os mesmos, e unha copia por OSC 52 de máis de 1 MiB non copiaría nada. |
| OSC 7 só sen equipo, con `localhost` ou co nome deste, coma foot e Ghostty | Decidido contigo: que facer cun cartafol doutro equipo decídese cando se use, na 4.5 e na 5.6. |
| OSC 9;9 ata o primeiro `;`, coma o Windows Terminal | Decidido contigo: unha ruta de Windows pode levar `;`. |
| As marcas coa súa fila, coma o Windows Terminal | O plan só pide o cartafol, o estado e o código (4.5); a fila obriga a aplicalas na orde de `vte`, tamén despois dunha actualización sincronizada. |
| `criterion` ou `divan` para o *benchmark* | Moitas máis dependencias, para dúas medidas emparelladas que caben nun arquivo. |

## Consecuencias

**Positivas:**

- A 4.2, a 4.3, a 4.4 e a 4.5 xa teñen onde ler o cartafol e o estado do *prompt* de cada tab.
- O escáner custa menos ca antes, aínda que le máis: medidos igual, con `bigfile`, do 3,2 % ao 0,12 %, e coa saída máis densa en cores, do 5,0 % ao 0,25 %.
- `?9001` séguese exactamente na orde de `vte`, sen copiar as súas regras.
- O escáner próbase contra `vte` con saídas aleatorias partidas en anacos aleatorios, e `proptest` reduce un fallo á saída máis curta que o mostra.

**Negativas e riscos:**

- Cada OSC custa: con ligazóns en cada fila ou cun *prompt* cada 20 filas, a ritmos que ninguén escribe, o escáner leva o 1,7 % e o 2,9 % do tempo do terminal (o de antes, coas ligazóns, o 3,3 %). E nas saídas de bloques pequenos, coma as gravacións pequenas das probas, o reloxo que mide cada bloque pesa máis ca o escáner: do 8 ao 26 % (co de antes, do 14 ao 39 %).
- Dentro dunha actualización sincronizada, `?9001` cambia cando se mostra, non ao chegar.
- `vte` segue gardando enteiro o OSC máis longo, e a súa memoria queda ata que se pecha a tab.
- Non ve un OSC con bytes entre o ESC e o `]` que non saen do escape en `vte`: os controis, que executa, e DEL e os bytes desde 0x80, que ignora.
- `meiga.exe` medra 11 776 bytes, ata 964 096: o escáner novo e `memmem`, de `memchr`.

## Referencias

- `vte` v0.15.0: [o OSC](https://github.com/alacritty/vte/blob/v0.15.0/src/lib.rs#L407-L434), [o seu texto](https://github.com/alacritty/vte/blob/v0.15.0/src/lib.rs#L543-L559), [os OSC que trata](https://github.com/alacritty/vte/blob/v0.15.0/src/ansi.rs#L1329-L1524), [os modos privados](https://github.com/alacritty/vte/blob/v0.15.0/src/ansi.rs#L1603-L1612) e [a actualización sincronizada](https://github.com/alacritty/vte/blob/v0.15.0/src/ansi.rs#L35-L48).
- Windows Terminal v1.25.3131.0: [os OSC](https://github.com/microsoft/terminal/blob/v1.25.3131.0/src/terminal/parser/OutputStateMachineEngine.cpp#L763-L890), [OSC 9;9 e 133](https://github.com/microsoft/terminal/blob/v1.25.3131.0/src/terminal/adapter/adaptDispatch.cpp#L3545-L3746) e [o ConPTY, que pasa o texto](https://github.com/microsoft/terminal/blob/v1.25.3131.0/src/host/_stream.cpp#L389-L463).
- [A proposta dos *semantic prompts*](https://gitlab.freedesktop.org/Per_Bothner/specifications/-/raw/4d2e1d75d4861a1d924895e106f8f016880e12a7/proposals/semantic-prompts.md), [ConEmu](https://conemu.github.io/en/AnsiEscapeCodes.html#ConEmu_specific_OSC) e o [protocolo das cores de kitty](https://sw.kovidgoyal.net/kitty/color-stack/#setting-and-querying-colors).
- [Tecnoloxías › O cartafol e o prompt que di a shell](../tecnoloxias.md#68-o-cartafol-e-o-prompt-que-di-a-shell).
- [Código › meiga-core › `scanner.rs`](../codigo/meiga-core.md#cratesmeiga-coresrctermscannerrs), [`shell_state.rs`](../codigo/meiga-core.md#cratesmeiga-coresrctermshell_staters) e [`handler.rs`](../codigo/meiga-core.md#cratesmeiga-coresrctermhandlerrs).
- [Subfases › 4.1 · Escáner OSC](../subfases/4.1-escaner.md).
