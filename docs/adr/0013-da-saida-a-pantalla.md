# ADR 0013 · Da saída á pantalla

- **Estado:** Aceptado; concretado polo [ADR 0016](0016-o-historial-o-tamano-e-o-zoom.md)
- **Data:** 2026-11-08
- **Relacionados:** [ADR 0005](0005-fios-e-fotogramas.md), [ADR 0008](0008-temporizador-dos-fotogramas.md), [ADR 0010](0010-texto-con-directwrite.md), [ADR 0011](0011-fin-e-peche-das-sesions.md), [ADR 0012](0012-fachada-do-terminal.md)

## Contexto

- Na subfase 1.6 xúntanse as pezas das anteriores: a sesión de ConPTY e os seus dous fíos (1.4, [ADR 0011](0011-fin-e-peche-das-sesions.md)), o terminal (1.5, [ADR 0012](0012-fachada-do-terminal.md)), os fotogramas (1.2, [ADR 0008](0008-temporizador-dos-fotogramas.md)) e o texto (1.3, [ADR 0010](0010-texto-con-directwrite.md)). O [ADR 0005](0005-fios-e-fotogramas.md) xa decide o modelo: o fío lector le a saída e actualiza a grella baixo un *lock* curto, e avisa á interface con `PostMessage`, só se non hai outro aviso pendente. Faltaba concretar como, e todo o que fai falta para mostrar a saída dun programa de verdade. Antes de escribilo, comprobouse:
  - **Os avisos entre fíos.** A xanela pertence ao fío que a creou: o fío lector non pode debuxar nela. `PostMessageW` deixa unha mensaxe na cola da xanela desde calquera fío e volta de inmediato. `PostThreadMessageW` deixaría a mensaxe na cola do fío, sen xanela, e Microsoft documenta que esas mensaxes se perden mentres o fío está nun bucle modal, como o que fai Windows mentres se move ou se redimensiona a xanela. `SendMessageW` agardaría a que a interface a atendese. As mensaxes desde `WM_APP` (`0x8000`) son para o uso propio dun programa. Unha cola admite como moito 10 000 mensaxes publicadas.
  - **As respostas.** O ConPTY pregunta ao arrancar polos atributos do terminal (`ESC [ c`). Moitos programas preguntan pola cor do fondo (`OSC 11`), por exemplo para escoller un tema claro ou escuro, e despois polos atributos do terminal: como todos os terminais responden a esta, a súa resposta marca o final, e se chega antes ca da cor, o programa sabe que o terminal non responde a cor. As respostas teñen que ir na mesma orde ca as preguntas.
  - **A paleta.** As celas gardan o número da cor, non a cor (1.5). Un programa pode cambiar calquera cor da paleta (`OSC 4`), a do texto (`OSC 10`), a do fondo (`OSC 11`) e a do cursor (`OSC 12`), ou voltar á orixinal (`OSC 104`, `110`, `111` e `112`): `alacritty_terminal` garda os cambios, e o ConPTY 1.25 pásaos sen cambios. Coma no xterm, o texto que xa está na pantalla cambia de cor.
  - **O cursor.** Nesta sesión de Escritorio remoto, Windows desactiva o pestanexo do cursor (`GetSystemMetrics(SM_CARETBLINKINGENABLED)` vale 0). O Windows Terminal fai caso desa métrica e de `GetCaretBlinkTime`, e o Warp non: o seu cursor pestanexa sempre, uns dous fotogramas por segundo da xanela enteira, a maior parte do que gasta en repouso (subfase 0.8). Windows ten tamén un tempo despois do que o cursor deixa de pestanexar (`SPI_GETCARETTIMEOUT`, 5 s por defecto) e un grosor (`SPI_GETCARETWIDTH`, o «grosor do cursor de texto» da accesibilidade): o Windows Terminal e o conhost non usan o primeiro.
  - **As actualizacións sincronizadas.** Mentres un programa ten unha aberta (`ESC [ ? 2026 h`), `vte` garda todo o que escribe, tamén as súas preguntas, e non o le ata que chega o fin ou ata que quen usa o terminal a remata á súa hora, 150 ms despois ([ADR 0012](0012-fachada-do-terminal.md)). O Alacritty remátaa no seu bucle de eventos, que espera a saída cun tempo límite. O fío lector de meiga está bloqueado lendo a saída, sen tempo límite: se o programa non escribe máis, non pode rematala.
  - **As fontes.** Cascadia Mono e Consolas teñen fontes negras e cursivas de verdade: DirectWrite só as simula se a familia non as ten. A fonte dá tamén a posición e o grosor do subliñado e do riscado.
  - **O arranque.** Lanzar o programa, co ConPTY, leva unhas decenas de milisegundos, e o programa tarda moito máis en escribir: unha shell como PowerShell, uns centos.

## Decisión

1. **Os avisos.** O terminal e tres marcas comparten un `Mutex`: *a pantalla cambiou*, *o título cambiou* e *a xanela sabe cando remata a actualización sincronizada aberta*. O fío lector dálle cada bloque ao terminal e, se algunha marca que toca non estaba posta, póna e envía `WM_WAKE` (`WM_APP`) á xanela con `PostMessageW`, unha soa vez. A xanela quita a marca da pantalla cando copia o que cambiou, no fotograma; a do título, cando o colle; e a da actualización sincronizada, cando ve que xa non hai ningunha aberta. Así nunca hai máis dunha mensaxe pendente de cada tipo, e mentres a xanela está minimizada, que non copia nada, a saída non a esperta máis dunha vez, agás para rematar á súa hora as actualizacións sincronizadas.
2. **O *lock* é curto.** O fío lector tenno mentres o terminal le un bloque, e a interface mentres copia o que cambiou: compoñer e presentar fanse fóra del.
3. **As respostas envíaas o fío lector**, con `Input`, a entrada da sesión, que se pode clonar: xusto despois de ler o bloque que as pide e fóra do *lock*. As preguntas polas cores e polo tamaño en píxeles (`CSI 14 t`) respóndense no mesmo lugar ca as outras, na mesma orde ca as preguntas. As poucas respostas que aparecen ao rematar unha actualización sincronizada envíaas a interface, coa súa copia de `Input`.
4. **A paleta:** as 16 cores do esquema *Campbell* do Windows Terminal, coas de meiga para o texto e o fondo (`#D4D7DD` e `#16181D`), e as 256 do xterm: o cubo de 6 × 6 × 6 e 24 grises. O cursor, da cor do texto. Se un programa cambia a paleta, debúxase todo outra vez, as marxes tamén; se só cambia a cor do cursor, só o cursor.
5. **Os estilos:** a negra e a cursiva, coas fontes negra e cursiva da familia, sen cambiar a cor; o texto tenue (*dim*), coa súa cor un terzo do camiño cara á do fondo; o inverso troca as dúas cores, antes do tenue; o oculto só debuxa o fondo. O subliñado (simple, dobre, ondulado, de puntos e de trazos) e o riscado, da cor do texto, onde di a fonte.
6. **O cursor:** unha barra que pestanexa, coma no Windows Terminal; `ESC [ 0 SP q` volta a ela. O seu grosor é o do cursor de Windows, á escala da xanela. Sen o foco, é un rectángulo baleiro que non pestanexa. Só pestanexa se a xanela ten o foco, se Windows o permite (`SM_CARETBLINKINGENABLED` e un `GetCaretBlinkTime` de 1 a 10 000 ms), e se o programa non pediu un cursor fixo; e para despois de `SPI_GETCARETTIMEOUT` sen actividade, visíbel. Actividade é un cambio na pantalla ou recuperar o foco.
7. **Os fotogramas que pide o terminal.** Antes de cada fotograma, a vista do terminal ponse ao día e di cando necesita outro aínda que non chegue saída: a hora límite dunha actualización sincronizada ou o seguinte cambio do pestanexo. O mesmo temporizador dos fotogramas serve para as dúas cousas; se vence sen ningún fotograma pendente, conta como un cambio, e pasa polo *pacer* coma calquera outro. **Coa xanela minimizada** non hai fotogramas, pero unha actualización sincronizada remátase igual á súa hora: a interface fai só iso, sen copiar nin debuxar nada, e o temporizador queda só para esa hora. Se non, un programa que preguntase algo dentro dunha actualización que non remata esperaría a resposta ata que se restaurase a xanela.
8. **A grella** ten o tamaño da xanela ao arrancar, en celas enteiras, e as marxes que sobran pintan a cor do fondo. Non cambia coa xanela ata a subfase 1.9.
9. **O programa:** o da liña de comandos, `meiga [opcións] [--] [programa [argumentos]]`, cos argumentos entre comiñas segundo as regras de Windows; sen el, PowerShell 7 (`pwsh`) ou, se non está instalado, Windows PowerShell. Se non se pode lanzar, o terminal mostra por que. A sesión lánzase antes do primeiro fotograma.
10. **Cando o programa remata**, o terminal mostra unha liña co seu código de saída e oculta o cursor, e a xanela queda aberta ata que se pecha. meiga sae co código 0.
11. **O título** da xanela é o que pon o programa, ou «meiga» se non pon ningún.
12. **A demo desaparece.** Os modos de medición (`--bench`) quedan, sen cambios.

Isto concreta o [ADR 0005](0005-fios-e-fotogramas.md): como avisa o fío lector á interface e como se envían as respostas. E concreta o [ADR 0012](0012-fachada-do-terminal.md) no que deixaba para esta subfase: as respostas ás preguntas polas cores e polo tamaño en píxeles, os cambios da paleta e o fin das actualizacións sincronizadas co temporizador dos fotogramas.

## Alternativas consideradas

| Alternativa | Por que se descarta |
|---|---|
| `PostThreadMessageW` | As súas mensaxes pérdense mentres Windows move ou redimensiona a xanela, nun bucle modal. |
| `SendMessageW` desde o fío lector | O lector agardaría pola interface, e a interface pode estar agardando polo *lock*. |
| Un aviso por cada bloque de saída | Coa saída masiva, centos de mensaxes por segundo que só fan copiar outra vez o mesmo; a cola podería encherse. |
| Un evento por sesión, esperado na interface xunto coas mensaxes | Tamén funciona, pero `MsgWaitForMultipleObjectsEx` espera como moito 63 obxectos, e con tabs habería que xestionalos. A mensaxe vai pola mesma cola ca o resto. |
| Rematar as actualizacións sincronizadas no fío lector, coma o Alacritty | O fío lector está bloqueado lendo a saída: para rematar unha á súa hora sen que chegue máis saída, necesitaría E/S asíncrona ou outro fío por sesión. A interface xa ten o temporizador. |
| Copiar a pantalla tamén coa xanela minimizada, para rematar as actualizacións sincronizadas | A saída espertaría a interface outra vez, e cada fotograma que non se debuxa copiaría filas para nada. |
| Responder desde o fío da interface | A resposta agardaría a que a interface atenda a mensaxe, e a orde coas outras respostas perderíase. |
| Non responder ás preguntas polas cores | Os programas que preguntan agardan ata que se cansan, e escollen mal o tema. |
| Gardar a cor nas celas, en vez do número | Celas máis grandes, e o texto que xa está na pantalla non cambiaría de cor coma no xterm. |
| A negra como cor brillante (`boldColors` do xterm) | Escollido contigo: a fonte negra, coma o Alacritty e o Warp. |
| Pestanexar sempre, coma o Warp | Dous fotogramas por segundo para sempre, tamén por Escritorio remoto, onde Windows o desactiva. |
| Pestanexar sen límite de tempo, coma o Windows Terminal | Mentres a xanela ten o foco, espertaría dúas veces por segundo para sempre. |
| Un cursor de bloque por defecto, coma o xterm e o Alacritty | Escollido contigo: a barra, coma o Windows Terminal e o Warp. |
| Cambiar xa a grella co tamaño da xanela | Hai que redimensionar tamén o ConPTY, e o texto recolócase: é a subfase 1.9. |
| Lanzar o programa despois do primeiro fotograma | A xanela aparecería antes, pero o programa empezaría despois, e escribiría máis tarde o mesmo tempo. |
| Pechar a xanela cando remata o programa | Perderíase o que escribiu e o seu código. Decídese coas tabs, na subfase 2.7. |

## Consecuencias

**Positivas:**

- **En repouso, 0 espertares:** o cursor só pestanexa ata 5 s despois da última actividade e, por Escritorio remoto, nunca.
- Coa saída masiva, como moito un aviso pendente, e como moito un fotograma por intervalo.
- Coa xanela minimizada, o terminal segue lendo a saída e respondendo, tamén dentro das actualizacións sincronizadas, pero non se debuxa nada nin se copia a pantalla.
- Os programas que preguntan polas cores ou polo tamaño reciben a resposta de inmediato e na orde correcta.
- Un programa pode cambiar a paleta, e todo o texto cambia de cor coma no xterm.

**Negativas e riscos:**

- O fío lector agarda mentres a interface copia o que cambiou. Mitigación: copiar só o que cambiou, sen compoñer nin presentar dentro do *lock*.
- Lanzar o programa atrasa o primeiro fotograma: o arranque de meiga mídese agora co da sesión.
- A grella non segue a xanela: se a xanela medra, sobra espazo; se se fai pequena, córtase. Arránxase na subfase 1.9.
- Aínda non se pode escribir: o teclado chega na subfase 1.7.
- Aínda sen a cor do subliñado (`SGR 58`). Os emoji son dunha soa cor (8.1), e entre as celas dalgunhas liñas de caixa hai un oco (8.2).
- O pestanexo depende da configuración de Windows: lese ao arrancar e cada vez que a xanela recupera o foco.

## Referencias

- Microsoft Learn: [`PostMessageW`](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-postmessagew), [`PostThreadMessageW`](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-postthreadmessagew), [`WM_APP`](https://learn.microsoft.com/en-us/windows/win32/winmsg/wm-app), [`GetCaretBlinkTime`](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-getcaretblinktime), [`GetSystemMetrics`](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-getsystemmetrics) e [`SystemParametersInfoW`](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-systemparametersinfow).
- [*XTerm Control Sequences*](https://invisible-island.net/xterm/ctlseqs/ctlseqs.html): `OSC 4`, `10`, `11`, `12`, `104` e `110`–`112`, e `CSI 14 t`.
- Windows Terminal v1.25.2733.0: `src/cascadia/TerminalCore/terminalrenderdata.cpp`, o pestanexo do cursor, e o esquema *Campbell*.
- [Tecnoloxías › Avisar á xanela desde outro fío](../tecnoloxias.md#36-avisar-á-xanela-desde-outro-fío-postmessage), [› A paleta e os estilos](../tecnoloxias.md#37-a-paleta-e-os-estilos-das-celas), [› O cursor de Windows](../tecnoloxias.md#38-o-cursor-de-windows-pestanexo-tempo-e-grosor) e [› A liña de comandos](../tecnoloxias.md#40-a-liña-de-comandos-en-windows).
- [Código › meiga › `session.rs`](../codigo/meiga.md#cratesmeigasrcsessionrs) e [`view.rs`](../codigo/meiga.md#cratesmeigasrcviewrs).
