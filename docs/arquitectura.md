# Arquitectura

> Estado: subfase 1.7. Existen o esqueleto, a integración continua, as ferramentas de medición e os escenarios de carga, e `meiga.exe` xa é un terminal: lanza un programa nunha pseudoconsola, mostra a súa saída nunha xanela que presenta só o que cambia, cun *framebuffer* propio e texto con DirectWrite, e envíalle as teclas coma unha consola de Windows. O que falta, como as tabs, descríbese co deseño acordado e a subfase na que chega. O porqué de cada decisión importante está nos [ADR](adr/README.md).

## 1. Obxectivo e restricións

meiga é un terminal para Windows con tabs, grupos de tabs, explorador de cartafoles e restauración da sesión. A restrición que manda sobre todas as demais é o **rendemento**: o programa debe gastar o mínimo de CPU e de memoria, tamén en máquinas sen GPU ou que se usan por Escritorio remoto.

Nesas máquinas, un terminal que debuxa con DirectX ou Vulkan non ten tarxeta gráfica que faga o traballo, e Windows emula a GPU na CPU (o rasterizador por software WARP). Ademais, o Escritorio remoto ten que comprimir e enviar pola rede cada fotograma que cambia. Por iso meiga debuxa na CPU, pero só o que cambia ([ADR 0002](adr/0002-windows-e-renderizado-cpu.md)).

Deste obxectivo saen os principios de deseño:

| Principio | Que significa na práctica |
|---|---|
| Cero traballo en repouso | Sen temporizadores periódicos, sen *polling* e sen animacións. Se non pasa nada, o programa está durmido. |
| Debuxar só o que cambia | Lévase a conta das celas que cambiaron (*damage tracking*) e só se presentan eses rectángulos. |
| Non debuxar o que non se ve | As tabs en segundo plano só procesan a saída da shell; non se debuxan. Coa xanela minimizada, non se debuxa nada ([ADR 0009](adr/0009-xanela-que-non-se-ve.md)). |
| Agrupar en fotogramas | Como moito un fotograma por intervalo, aínda que a shell escriba miles de veces por segundo. |
| Poucos fíos, e bloqueados | Un fío para a interface e dous por tab. Cando non hai traballo, todos esperan bloqueados sen gastar CPU. |
| Sen *frameworks* pesados | Win32, DirectWrite e GDI directamente: nada de navegadores embebidos nin *runtimes* asíncronos. |
| Medir sempre | Cada fase pecha cunha porta de rendemento con medicións (ver a [sección 8](#8-medición-do-rendemento)). |

## 2. Estrutura do código: un *workspace* con catro *crates*

En Rust, un *crate* é a unidade de compilación: unha biblioteca ou un programa executábel. O código de meiga divídese en catro *crates* dentro dun mesmo *workspace* de Cargo (un conxunto de crates que se compilan xuntos; ver [Tecnoloxías](tecnoloxias.md#3-cargo-e-os-workspaces)).

```mermaid
graph TD
    meiga["meiga<br/>(binario)"] --> render["meiga-render<br/>(renderizador CPU)"]
    meiga --> win["meiga-win<br/>(plataforma Windows)"]
    meiga --> core["meiga-core<br/>(lóxica pura)"]
    render --> core
    win --> core
```

| Crate | Tipo | Responsabilidade | Que pode usar |
|---|---|---|---|
| `meiga-core` | biblioteca | Toda a lóxica que non depende de Windows: modelo de xanelas, tabs e grupos; fachada do terminal; codificación do teclado e do rato; busca difusa; configuración. | Só bibliotecas portábeis. **`unsafe` prohibido** e sen APIs de Windows. |
| `meiga-render` | biblioteca | Renderizado por CPU: caché de glifos con DirectWrite, composición no *framebuffer* e presentación dos rectángulos que cambiaron. | `meiga-core` e as APIs gráficas de Windows. |
| `meiga-win` | biblioteca | Capa de plataforma: ConPTY, detección das shells, sistema de arquivos, portapapeis, xanela Win32 e DPI. | `meiga-core` e as APIs de Windows. |
| `meiga` | binario | Une as pezas: crea a xanela, o bucle de mensaxes e o fluxo de fotogramas. | Os tres anteriores. |

### Por que esta separación

- **Probas rápidas e fiábeis.** `meiga-core` non toca o sistema operativo, así que as súas probas son unitarias puras: non abren xanelas nin procesos e executan en milisegundos.
- **`unsafe` illado.** Chamar ás APIs de Windows desde Rust require bloques `unsafe`, nos que o compilador non pode garantir a seguridade da memoria e confía no programador. Concentralos en `meiga-win` e `meiga-render` fai que o resto do código teña todas as garantías de Rust e que os puntos delicados sexan fáciles de revisar. En `meiga-core`, o compilador prohíbeo con `#![forbid(unsafe_code)]`.
- **Compilación incremental máis rápida.** Un cambio nun crate só recompila ese crate e os que dependen del.
- **Preparado para o futuro.** Un renderizador con GPU (mellora futura F2) sería outra implementación dentro de `meiga-render`, sen tocar a lóxica de `meiga-core`.

A regra de dependencias é estrita: as frechas do diagrama só van cara abaixo, e `meiga-core` nunca depende dos outros tres. Esta regra xa está declarada nos `Cargo.toml` de cada crate, así que Cargo non deixaría compilar un ciclo. A elección de Rust explícase no [ADR 0001](adr/0001-rust.md).

## 3. Fluxo de datos

Así viaxa a saída dun programa ata a pantalla:

```mermaid
sequenceDiagram
    participant S as Programa (pwsh, cmd, wsl…)
    participant P as ConPTY (OpenConsole)
    participant L as Fío lector da sesión
    participant T as Terminal (parser + grella)
    participant U as Fío da interface
    participant F as Framebuffer
    S->>P: escribe texto e secuencias VT
    P->>L: bytes UTF-8 e secuencias VT (ReadFile)
    L->>T: analiza e actualiza a grella (lock)
    L->>P: as respostas ás preguntas do programa
    L-->>U: WM_WAKE, só se non hai outro pendente
    U->>T: copia só as celas que cambiaron (lock)
    U->>F: compón só esas celas
    U->>U: presenta só os rectángulos sucios (BitBlt)
```

- **ConPTY** é a API de Windows que fai de "pseudoterminal": a shell cre que escribe nunha consola normal, e ConPTY tradúceo a un fluxo de texto con secuencias de escape VT, que é o que le meiga. meiga distribúe a súa propia copia de ConPTY ([ADR 0004](adr/0004-conpty-empaquetado.md)).
- **Secuencias VT** (ou ANSI): códigos que van mesturados co texto e indican cores, movementos do cursor, borrados… Por exemplo, `ESC[31m` cambia a cor do texto a vermello.
- **Parser e grella**: o *parser* interpreta as secuencias VT e a grella é a matriz de celas (carácter, cor, estilo) que representa a pantalla do terminal. Os dous veñen de `alacritty_terminal` ([ADR 0003](adr/0003-nucleo-vt-alacritty-terminal.md)), e un escáner propio detecta antes as secuencias da integración coa shell ([ADR 0007](adr/0007-integracion-coa-shell.md)).

| Peza | Subfase |
|---|---|
| Xanela e bucle de mensaxes | 1.1 (feita) |
| *Framebuffer* e presentación parcial | 1.2 (feita) |
| Fontes e caché de glifos | 1.3 (feita) |
| ConPTY | 1.4 (feita) |
| Fachada do terminal (parser e grella) | 1.5 (feita) |
| Da saída á pantalla | 1.6 (feita) |
| Teclado | 1.7 (feita) |
| Varias sesións, unha por tab | 2.2 |

### Sesións: a shell, o ConPTY e os dous fíos

Desde a subfase 1.4, `meiga-win` lanza un programa nunha pseudoconsola e fala con el ([Tecnoloxías › ConPTY](tecnoloxias.md#31-conpty-a-pseudoconsola-de-windows)), e desde a 1.6, `meiga.exe` abre unha sesión ao arrancar, co programa da súa liña de comandos ou, sen el, PowerShell ([Código › meiga › `session.rs`](codigo/meiga.md#cratesmeigasrcsessionrs)).

```mermaid
flowchart LR
    write["Input::write<br/>(as respostas, as teclas e o foco)"] --> input["Entrada pendente<br/>Vec + evento"]
    input --> writer["Fío escritor"]
    writer -- "tubaxe de entrada" --> conpty["ConPTY<br/>(OpenConsole)"]
    conpty <--> shell["Programa<br/>(pwsh, cmd, wsl…)"]
    conpty -- "tubaxe de saída" --> reader["Fío lector<br/>ReadFile de 64 KiB"]
    reader --> output["Output::received<br/>(o terminal)"]
    reader -. "ao final" .-> ended["Output::ended<br/>co código de saída"]
```

1. `Conpty::spawn` crea as dúas tubaxes, a pseudoconsola e un *job*, lanza o programa nela e dentro do *job*, e arranca os dous fíos da sesión. Devolve un `Pty` ([Código › meiga-win](codigo/meiga-win.md#cratesmeiga-winsrcconptyrs)).
2. `Input::write` deixa os bytes na entrada pendente e esperta o escritor, que os escribe na tubaxe de entrada. Nunca espera. `Input` pódese clonar e usar desde calquera fío: o `Pty` dá unha copia (`Pty::input`), coa que a vista do terminal envía as teclas desde a 1.7, e o lector recibe outra con cada bloque, para responder ao programa.
3. O lector le a saída en bloques de ata 64 KiB e pásaos a un `Output`: en `meiga.exe`, o lector da sesión, que llos dá ao terminal.
4. O escritor espera á vez a entrada e o fin do programa. Cando o programa remata, ou cando se destrúe o `Pty`, pecha a pseudoconsola: o ConPTY remata, a tubaxe de saída péchase, e o lector avisa do fin co código de saída do programa. Se pasan 10 s sen que remate ningún dos programas que tiña a sesión ao pechala, está bloqueada, e o escritor remata todos os do *job*: o programa e os que arrancou ([ADR 0011](adr/0011-fin-e-peche-das-sesions.md)).
5. meiga usa o ConPTY que leva ao lado de `meiga.exe` (`conpty.dll` e `OpenConsole.exe`) ou, se non está, o de Windows ([ADR 0004](adr/0004-conpty-empaquetado.md)).

### O terminal: da saída ao que se mostra

Desde a subfase 1.5, o terminal de `meiga-core` le a saída dun programa e garda o que hai que mostrar e o que hai que responder, en tipos de meiga ([ADR 0012](adr/0012-fachada-do-terminal.md)). Desde a 1.6, dálle a saída o fío lector da sesión; as probas danlle saída gravada do ConPTY, e un exemplo de consola mide a que velocidade a le.

```mermaid
flowchart LR
    out["Saída do programa<br/>(o fío lector)"] --> feed["Terminal::feed"]
    feed --> scan["Escáner<br/>?9001"]
    feed --> parser["Parser<br/>vte"]
    parser --> term["Term<br/>alacritty_terminal"]
    term -- "eventos" --> queue["Cola: respostas, título,<br/>campá, portapapeis"]
    queue --> changes["Changes<br/>(o que devolve feed)"]
    term -- "filas marcadas" --> copy["Terminal::copy_damage<br/>compara cela a cela"]
    copy --> screen["Screen<br/>celas, cambios e cursor"]
    screen --> render["A vista do terminal<br/>(o fío da interface)"]
```

1. `Terminal::feed` dálle os bytes ao escáner, que segue os modos que `alacritty_terminal` ignora, e despois, os mesmos bytes, ao parser de `vte`, que actualiza o `Term` de `alacritty_terminal`: a grella, o historial, o cursor e os modos ([Tecnoloxías › `alacritty_terminal` e `vte`](tecnoloxias.md#34-alacritty_terminal-e-vte-o-núcleo-do-terminal)).
2. O que non é a grella chega como eventos a unha cola. `feed` devolve que pasou (`Changes`): hai respostas para o programa, que se recollen con `take_responses` e se lle envían con `Input`; cambiou o título; soou a campá; ou o programa copiou texto ao portapapeis ([Tecnoloxías › As secuencias de control](tecnoloxias.md#35-as-secuencias-de-control)). Desde a 1.6, o terminal responde tamén ás preguntas polas cores e polo tamaño en píxeles, coa súa paleta e o tamaño da cela, no mesmo lugar ca as outras respostas.
3. `Terminal::copy_damage` copia a un `Screen` as filas que marcou `alacritty_terminal`. Compara cada cela coa que había, e só as distintas quedan como cambiadas, xunto coas do cursor se se moveu. Tamén copia a paleta: se o programa a cambiou, todas as celas quedan como cambiadas ([Tecnoloxías › A paleta](tecnoloxias.md#37-a-paleta-e-os-estilos-das-celas)). A vista do terminal debuxa só as cambiadas.
4. Se o programa empeza unha actualización sincronizada, a saída espera ata que a remate ou ata que pase a súa hora límite, que o terminal di (`sync_deadline`): o temporizador dos fotogramas está programado para esa hora, e o fotograma remátaa (`end_sync`). Coa xanela minimizada, sen fotogramas, a interface remátaa igual a esa hora.
5. O exemplo `conpty` de `meiga-win` grava a saída dun programa (`--record`), e o exemplo `replay` de `meiga-core` dálla a un terminal, tan rápido como pode, e mide a velocidade ([Código › meiga-core](codigo/meiga-core.md#cratesmeiga-coreexamplesreplayrs)).

O terminal pasa dun fío a outro: o fío lector dálle a saída e o da interface copia o que cambiou, cada un collendo o mesmo `Mutex` ([ADR 0005](adr/0005-fios-e-fotogramas.md)).

### Da saída á pantalla: o aviso, a copia e o fotograma

Desde a subfase 1.6, as pezas anteriores xúntanse ([ADR 0013](adr/0013-da-saida-a-pantalla.md)):

```mermaid
flowchart LR
    reader["Fío lector<br/>Reader::received"] -- "feed (lock)" --> shared["Shared<br/>o terminal e tres marcas"]
    reader -- "respostas (Input)" --> program["Programa"]
    reader -- "WM_WAKE, só se<br/>non había outro" --> wake["App::wake<br/>(fío da interface)"]
    wake -- "o título" --> title["SetWindowTextW"]
    wake -- "a pantalla" --> frames["Frames::changed"]
    frames --> pacer{"Pasou o<br/>intervalo?"}
    pacer -- "si" --> frame["Fotograma"]
    pacer -- "non" --> timer["Temporizador"]
    timer --> frame
    frame --> update["TerminalView::update<br/>copia o que cambiou (lock)"]
    update --> draw["Compoñer esas celas<br/>e presentalas"]
```

1. O fío lector dálle cada bloque ao terminal, baixo o *lock*, e envía as respostas fóra del. Se a pantalla ou o título cambiaron e a súa marca non estaba posta, póna e envía `WM_WAKE` á xanela con `PostMessageW` ([Tecnoloxías › Avisar á xanela](tecnoloxias.md#36-avisar-á-xanela-desde-outro-fío-postmessage)). Faino tamén cando o programa deixa aberta unha actualización sincronizada, se a xanela aínda non sabe cando remata. Nunca hai máis dun aviso pendente de cada tipo.
2. A interface atende o aviso: se cambiou o título, quita a súa marca e pono na xanela ([Tecnoloxías › O título](tecnoloxias.md#39-o-foco-e-o-título-da-xanela)); se cambiou a pantalla, pide un fotograma, que o *pacer* fai no momento ou ao final do intervalo.
3. O fotograma empeza poñendo ao día a vista do terminal ([Código › meiga › `view.rs`](codigo/meiga.md#cratesmeigasrcviewrs)): baixo o *lock*, copia o que cambiou e quita a marca da pantalla. As zonas que cambiaron son as celas cambiadas de cada fila; se cambiou a paleta, a xanela enteira, marxes incluídas.
4. Despois compón esas zonas: o fondo, os glifos de cada tramo de celas co mesmo estilo, coa fonte normal, negra, cursiva ou negra cursiva, as liñas do subliñado e do riscado, e o cursor. E preséntaas.
5. A vista di tamén cando necesita outro fotograma aínda que non chegue saída: ao final dunha actualización sincronizada ou no seguinte cambio do pestanexo do cursor. O temporizador dos fotogramas vence entón. O cursor só pestanexa se a xanela ten o foco e Windows o permite, e para despois duns segundos sen actividade ([Tecnoloxías › O cursor](tecnoloxias.md#38-o-cursor-de-windows-pestanexo-tempo-e-grosor)): en repouso, non hai ningún temporizador.
6. Coa xanela minimizada, a interface non copia nada: a marca da pantalla segue posta e a saída non a esperta máis. Só remata á súa hora as actualizacións sincronizadas que o programa deixa abertas, sen copiar nin debuxar nada (`update_hidden`), e o temporizador dos fotogramas queda só para iso: mentres unha está aberta, o terminal garda tamén as preguntas do programa, que esperaría a resposta ata que se restaurase a xanela. Ao restaurala, compón todo coa pantalla ao día.
7. Cando o programa remata, o lector dálle ao terminal unha liña co código de saída e oculta o cursor; a xanela queda aberta ata que se pecha.

A grella ten o tamaño da xanela ao arrancar, e as marxes que sobran pintan a cor do fondo: cambiará coa xanela na subfase 1.9.

### O teclado: da tecla ao programa

Desde a subfase 1.7, as teclas chegan ao programa da sesión coma eventos de tecla enteiros, os mesmos que le un programa da consola de Windows ([ADR 0014](adr/0014-o-teclado.md)):

```mermaid
flowchart LR
    win["Windows<br/>as mensaxes das teclas"] --> loop["Bucle de mensaxes<br/>TranslateMessage publica<br/>os caracteres"]
    loop --> kb["Teclado da xanela<br/>xunta tecla e carácter"]
    kb -- "Alt+F4 e Alt+Espazo" --> def["DefWindowProcW"]
    kb -- "Event::Key" --> view["Vista do terminal<br/>TerminalView::key"]
    view --> mode{"O ConPTY pediu<br/>o win32-input-mode?"}
    mode -- "si" --> w32["O evento enteiro<br/>ESC [ Vk ; Sc ; Uc ; Kd ; Cs ; Rc _"]
    mode -- "non" --> vt["VT: o carácter<br/>ou a secuencia da tecla"]
    w32 --> input["Input::write<br/>(sen esperar)"]
    vt --> input
    input --> conpty["ConPTY"]
    conpty --> program["Programa<br/>(PSReadLine…)"]
```

1. Windows envíalle á xanela unha mensaxe cando unha tecla baixa e outra cando sobe, e o bucle de mensaxes chama a `TranslateMessage`, que publica na cola o carácter da tecla, se escribe algún ([Tecnoloxías › O teclado en Windows](tecnoloxias.md#41-o-teclado-en-windows)).
2. O teclado da xanela xunta cada tecla co seu carácter nun evento: a tecla espera polo seu carácter, que é a seguinte mensaxe ([Código › meiga-win › `keyboard.rs`](codigo/meiga-win.md#cratesmeiga-winsrckeyboardrs)). Unha tecla morta baixa sen carácter, e o composto vai coa seguinte. Alt+F4 e Alt+Espazo van a Windows, que pecha a xanela e abre o seu menú; o resto, tamén Alt só, F10 e as teclas que soben, ao programa. As teclas de Windows e as que colle un IME non van a ningures.
3. A vista do terminal convérteo no que le o programa ([Código › meiga-core › `input.rs`](codigo/meiga-core.md#cratesmeiga-coresrcinputrs)): o evento enteiro en **win32-input-mode**, que o ConPTY pide ao arrancar, ou, antes diso ou se o desactiva, o que envía un terminal VT ([Tecnoloxías › As teclas no terminal](tecnoloxias.md#42-as-teclas-no-terminal-win32-input-mode-e-vt)). O ConPTY convérteo de novo nun evento para o programa: así o PSReadLine distingue Maiús+Intro de Intro, e chegan AltGr e as teclas mortas.
4. `Input::write` deixa os bytes na entrada da sesión e esperta o fío escritor, sen esperar: a xanela nunca queda bloqueada porque o programa non le ([Sesións](#sesións-a-shell-o-conpty-e-os-dous-fíos)). A tecla non pide ningún fotograma: o que escriba o programa en resposta chega coma calquera saída ([Da saída á pantalla](#da-saída-á-pantalla-o-aviso-a-copia-e-o-fotograma)).
5. Cando a xanela gaña ou perde o foco, o programa recibe `ESC [ I` ou `ESC [ O`, se o pediu.
6. O texto dun IME, do panel de emoji e do ditado de Windows chega coma caracteres sen tecla, e vai ao programa igual ([Tecnoloxías › O IME](tecnoloxias.md#43-o-ime-e-o-panel-de-emoji)).

### Fotogramas: do cambio á pantalla

Desde a subfase 1.2, o final do camiño funciona igual para calquera contido; desde a 1.6, o que cambia é a saída do programa:

```mermaid
flowchart LR
    ev["Un cambio<br/>(a saída do programa, o pestanexo do cursor)"] --> dmg["Damage<br/>as zonas que cambiaron"]
    dmg --> pacer{"FramePacer<br/>pasou o intervalo?"}
    pacer -- "si" --> frame["Fotograma"]
    pacer -- "non" --> timer["Temporizador<br/>dunha soa vez"]
    timer --> frame
    frame --> comp["Compoñer as zonas<br/>no framebuffer"]
    comp --> pres["Presentalas<br/>na xanela (BitBlt)"]
    paint["WM_PAINT<br/>(pídeo Windows)"] --> copy["Copiar desde<br/>o framebuffer"]
```

1. Cada cambio anota a súa zona no ***damage*** ([Código › meiga-core](codigo/meiga-core.md#cratesmeiga-coresrcdamagers)), que xunta as zonas próximas.
2. O ***pacer*** decide cando se debuxa ([Código › meiga-core](codigo/meiga-core.md#cratesmeiga-coresrcpacingrs)): se pasou o intervalo desde o último fotograma, 33,3 ms por Escritorio remoto e 16,7 ms en local, no momento; se non, cun temporizador dunha soa vez ao final do intervalo. Os cambios que chegan mentres tanto van no mesmo fotograma ([ADR 0005](adr/0005-fios-e-fotogramas.md), [ADR 0008](adr/0008-temporizador-dos-fotogramas.md)).
3. O fotograma **compón** só esas zonas no *framebuffer*, unha *DIB section* ([Código › meiga-render](codigo/meiga-render.md)), e **preséntaas**: copia cada unha á xanela con `BitBlt` ([Tecnoloxías › DIB sections](tecnoloxias.md#24-dib-sections-e-gdi-o-framebuffer-e-a-súa-presentación)).
4. Cando Windows pide pintar (`WM_PAINT`), por exemplo cando a xanela medra ou se restaura, cópiase do *framebuffer*, sen compoñer nada ([Tecnoloxías › Pintar nunha xanela](tecnoloxias.md#25-pintar-nunha-xanela-wm_paint-e-dwm)).
5. Coa xanela minimizada, non se debuxa nada: os cambios esperan e, ao restaurala, píntase enteira unha vez ([ADR 0009](adr/0009-xanela-que-non-se-ve.md)).

O código que une as pezas explícase en [Código › meiga](codigo/meiga.md#cratesmeigasrcframesrs).

### Texto: da cela aos píxeles

Desde a subfase 1.3, meiga debuxa texto, e desde a 1.6, o da grella do terminal ([ADR 0010](adr/0010-texto-con-directwrite.md)):

```mermaid
flowchart LR
    text["Texto<br/>(a grella do terminal)"] --> cells["Celas<br/>CellChar, con unicode-width"]
    cells --> draw["Font::draw<br/>só as celas do recorte"]
    draw --> cache{"Está na caché<br/>de glifos?"}
    cache -- "non" --> raster["DirectWrite rasterízao<br/>(co fallback, se a fonte non o ten)"]
    raster --> slot["Gárdase nun slot<br/>do tamaño da cela"]
    slot --> ink["A súa tinta, coa táboa<br/>de tons, no framebuffer"]
    cache -- "si" --> ink
```

1. O texto divídese en celas ([Código › meiga-core](codigo/meiga-core.md#cratesmeiga-coresrctextrs)): cada carácter ocupa unha, ou dúas se é ancho, segundo `unicode-width`, o mesmo crate que usará o núcleo do terminal ([Tecnoloxías › O ancho dos caracteres](tecnoloxias.md#29-o-ancho-dos-caracteres-unicode-width)).
2. `Font::draw` debuxa unha fila de celas coas súas cores: primeiro o fondo e despois a tinta dos glifos, só das celas que caen dentro do recorte do lenzo, que son as das zonas que cambiaron ([Código › meiga-render](codigo/meiga-render.md#cratesmeiga-rendersrctextrs)).
3. Cada glifo búscase na caché. A primeira vez, DirectWrite rasterízao con antialiasing en grises e gárdase nun *slot* do tamaño dunha cela. Se a fonte non ten o carácter, o *fallback* do sistema escolle outra que o teña ([Tecnoloxías › DirectWrite](tecnoloxias.md#27-directwrite-fontes-e-glifos)).
4. A tinta do glifo, os píxeles con cobertura, convértese en cor cunha táboa cos 256 tons entre o fondo e o texto ([Tecnoloxías › Mesturar o texto co fondo](tecnoloxias.md#30-mesturar-o-texto-co-fondo-gamma-e-contraste)).

O tamaño da cela sae das métricas da fonte e do DPI da xanela: meiga usa Cascadia Mono a 12 puntos ou, se non está, Consolas. A negra, a cursiva e a negra cursiva son outras fontes da mesma familia, que se crean a primeira vez que se usan, coa cela e a liña base da normal, e comparten a súa caché de glifos ([Tecnoloxías › DirectWrite](tecnoloxias.md#27-directwrite-fontes-e-glifos)).

## 4. Fíos

| Fío | Cantos | Que fai | Cando esperta |
|---|---|---|---|
| Interface | 1 | Mensaxes de Windows, entrada do usuario, composición e presentación. | Só cando chega unha mensaxe ou vence o temporizador dun fotograma pendente (`MsgWaitForMultipleObjectsEx`). |
| Lector | 1 por tab coa shell en marcha | Le a saída da shell, actualiza a grella e responde ás preguntas do programa. | Só cando a shell escribe (`ReadFile`). |
| Escritor | 1 por tab coa shell en marcha | Envía á shell o que se teclea, e pecha a pseudoconsola cando a shell remata ou se pecha a tab. | Só cando hai algo que enviar, cando se pecha a tab ou cando remata a shell (`WaitForMultipleObjects`). |

Un fío bloqueado nunha lectura ou nunha espera non consome CPU: o sistema operativo non o planifica ata que hai datos. Por iso o número de fíos non é un problema en si; o problema é que esperten sen motivo. O modelo completo, cos fotogramas e as tabs en segundo plano, está no [ADR 0005](adr/0005-fios-e-fotogramas.md), e o fin e o peche das sesións, no [ADR 0011](adr/0011-fin-e-peche-das-sesions.md).

O fío da interface xa existe desde a subfase 1.1: é o fío principal do programa. Dorme en `MsgWaitForMultipleObjectsEx` mentres non chega ningunha mensaxe nin vence o temporizador dos fotogramas, que só está activo cando hai un fotograma pendente (ver [Tecnoloxías › Temporizadores](tecnoloxias.md#26-temporizadores-e-a-espera-do-fío-da-interface)). Na subfase 1.1 durmía en `GetMessageW`, que só espera mensaxes ([Tecnoloxías › Xanelas Win32](tecnoloxias.md#22-xanelas-win32-e-o-bucle-de-mensaxes)); o temporizador chegou coa 1.2 ([ADR 0008](adr/0008-temporizador-dos-fotogramas.md)). Ao arrancar, Windows crea ademais algúns fíos auxiliares no proceso, por exemplo para cargar as DLL en paralelo, que rematan sós ao pouco tempo se non teñen traballo.

O lector e o escritor existen desde a subfase 1.4, en cada sesión de ConPTY ([Sesións](#sesións-a-shell-o-conpty-e-os-dous-fíos)). Desde a 1.6, `meiga.exe` abre unha sesión ao arrancar: tres fíos que traballan, e os tres dormen mentres o programa non escribe nada. O lector esperta a interface cunha mensaxe, só se non había outra pendente ([Da saída á pantalla](#da-saída-á-pantalla-o-aviso-a-copia-e-o-fotograma)). Varias sesións, unha por tab, chegan na fase 2.

## 5. Compilación

- O perfil `release` optimiza ao máximo: LTO completo, unha soa unidade de xeración de código e `panic = "abort"`.
- O *runtime* de C vai ligado estaticamente, así que `meiga.exe` non necesita que estea instalado o *Visual C++ Redistributable*.
- As versións *release* usan o subsistema gráfico de Windows (sen xanela de consola). As de depuración manteñen a consola para ver diagnósticos.
- O **manifesto da aplicación** vai dentro de `meiga.exe`: escala por monitor (v2), Windows 10 e 11 sen modos de compatibilidade, e sen pedir permisos de administrador. Incrústao o ligador, a petición do `build.rs` de `meiga` (ver [Tecnoloxías › Manifesto](tecnoloxias.md#23-manifesto-da-aplicación-e-dpi)).

Os detalles explícanse en [Tecnoloxías](tecnoloxias.md) e, liña a liña, en [Código › configuración do workspace](codigo/workspace.md).

## 6. Integración continua

Ningún cambio chega a `main` sen pasar por un PR e pola CI:

```mermaid
flowchart LR
    rama["Rama da subfase"] --> pr["Pull request"]
    pr --> win["Build and test (Windows)<br/>ligazóns · PowerShell 7 · arnés de medición · escenarios<br/>fmt · clippy · ConPTY · test · release<br/>zip · DLL · xanela · presentación · teclado · arranque · fotogramas"]
    pr --> scen["Load scenarios (bash)<br/>ShellCheck · escenarios en bash"]
    pr --> deny["Dependency policy (cargo-deny)<br/>licenzas · vulnerabilidades · orixes"]
    win --> art["Artefacto<br/>meiga-…-msvc.zip"]
    win --> ok{"Checks verdes<br/>e revisión"}
    scen --> ok
    deny --> ok
    ok -->|squash merge| main["main"]
```

- As regras da rama `main` (*ruleset*) fan obrigatorio este camiño, tamén para os administradores.
- Dependabot propón cada semana as versións novas dos crates, das actions e de Rust, como PR que pasan polo mesmo proceso.
- Cada luns, a CI revisa de novo as dependencias contra a base de datos de vulnerabilidades, aínda que non cambiase nada, e comproba se hai unha versión estábel do ConPTY máis nova ca a fixada. Esta comprobación non é obrigatoria: unha versión nova non bloquea ningún PR.

Os detalles están en [Tecnoloxías › GitHub Actions](tecnoloxias.md#15-github-actions) e en [Código › .github](codigo/github.md).

## 7. Decisións de arquitectura (ADR)

| ADR | Decisión |
|---|---|
| [0001](adr/0001-rust.md) | Rust como linguaxe. |
| [0002](adr/0002-windows-e-renderizado-cpu.md) | Só Windows, con renderizado por CPU do que cambia. |
| [0003](adr/0003-nucleo-vt-alacritty-terminal.md) | `alacritty_terminal` como núcleo, detrás dunha fachada. |
| [0004](adr/0004-conpty-empaquetado.md) | ConPTY empaquetado con meiga. |
| [0005](adr/0005-fios-e-fotogramas.md) | Fíos bloqueados e como moito un fotograma por intervalo. |
| [0006](adr/0006-licenzas-e-dependencias.md) | Só licenzas permisivas e dependencias mínimas. |
| [0007](adr/0007-integracion-coa-shell.md) | Integración coa shell mediante secuencias OSC. |
| [0008](adr/0008-temporizador-dos-fotogramas.md) | Un temporizador de alta resolución para os fotogramas. |
| [0009](adr/0009-xanela-que-non-se-ve.md) | Non debuxar coa xanela minimizada; as tapadas debúxanse. |
| [0010](adr/0010-texto-con-directwrite.md) | Texto con DirectWrite, nunha caché de glifos do tamaño da cela. |
| [0011](adr/0011-fin-e-peche-das-sesions.md) | O escritor de cada sesión espera tamén polo programa e pecha a pseudoconsola; cada sesión ten un *job*, e unha sesión pechada remata sempre. |
| [0012](adr/0012-fachada-do-terminal.md) | A fachada do terminal: tipos propios, a copia do que cambia cela a cela, as respostas e o escáner de `?9001`. |
| [0013](adr/0013-da-saida-a-pantalla.md) | Da saída á pantalla: un aviso pendente como moito, as respostas desde o fío lector, a paleta, os estilos e un cursor que só pestanexa se Windows o permite e por pouco tempo. |
| [0014](adr/0014-o-teclado.md) | O teclado: eventos de tecla enteiros coma os da consola, nos que a tecla espera polo seu carácter; Alt+F4 e Alt+Espazo para Windows; o win32-input-mode e VT de reserva; e o IME, que se mantén. |

## 8. Medición do rendemento

O rendemento mídese desde fóra, sen tocar o código de meiga, co mesmo arnés para calquera terminal: `tools/perf/measure.ps1`. Cada certo tempo fai unha "fotografía" de todos os procesos e fíos do sistema e reparte o consumo en grupos:

```mermaid
flowchart TD
    subgraph T["Lanzado polo terminal"]
        term["terminal<br/>meiga.exe, warp.exe…"] --> con["console<br/>OpenConsole.exe, un por tab"]
        term --> sh["shells<br/>pwsh, wsl, cmd e os seus programas"]
    end
    subgraph P["Poñer a imaxe na pantalla"]
        dwm["dwm<br/>compón as xanelas"]
        rdp["termservice e wudfhost<br/>Escritorio remoto"]
    end
    sys["system: toda a máquina"]
```

- **Que conta como custo do terminal:** o propio terminal, os seus procesos de consola e o traballo que provoca en `dwm` e no Escritorio remoto ao debuxar. As shells son a carga de traballo, igual para todos os terminais.
- **Métricas:** CPU (en % dun núcleo e en ciclos), espertares por segundo, memoria privada e fíos. Explícanse en [Tecnoloxías › Medición do rendemento](tecnoloxias.md#19-medición-do-rendemento).
- **Onde:** en local, no equipo de desenvolvemento, e non na CI, porque as máquinas da CI son compartidas e darían cifras con moito ruído. Os resultados gárdanse en `.perf/`, fóra de Git. A CI só comproba que o arnés mide ben.
- **O arranque:** `tools/perf/measure-startup.ps1` mide o tempo desde que se crea o proceso de meiga ata que a súa xanela está pintada (ver [Tecnoloxías › Tempo ata o primeiro fotograma](tecnoloxias.md#tempo-ata-o-primeiro-fotograma)). Desde a 1.6, inclúe lanzar a sesión, que vai antes do primeiro fotograma.
- **Os fotogramas:** `tools/perf/measure-frames.ps1` mide canto custan a meiga, a `dwm` e ao Escritorio remoto os fotogramas parciais, os completos e os completos de texto, cos modos de medición de meiga (`--bench`).

O código explícase en [Código › medición](codigo/perf.md).

### Escenarios de carga

Para comparar terminais, todos teñen que facer o mesmo traballo. Os escenarios imitan o uso diario dun terminal, e cada un existe en PowerShell e en bash, para medir a mesma carga nunha tab de pwsh e nunha de Ubuntu (WSL):

| Escenario | Que imita |
|---|---|
| `idle` | O terminal aberto sen facer nada, por exemplo con 5 tabs. Non arranca ningún script. |
| `spinner` | O indicador de progreso dunha ferramenta que traballa: unha liña que se redebuxa 10 veces por segundo. |
| `ai-stream` | Unha IA que pensa e responde palabra a palabra, con cores, emoji e unha zona de estado que se redebuxa 10 veces por segundo. |
| `bigfile` | Imprimir un arquivo de 50 MiB tan rápido como o acepte o terminal. |

```mermaid
flowchart LR
    run["run-scenario.ps1<br/>noutro terminal"] -- "controller e stop" --> dir[(".perf/run/")]
    esc["escenario<br/>no terminal medido"] -- "started-…" --> dir
    run --> mea["measure.ps1<br/>varias medicións"]
    run --> res["resumo, veredicto<br/>e CSV en .perf/"]
```

- `run-scenario.ps1` corre **noutro terminal**, para que o seu traballo non se sume ao do terminal medido. Coordínase cos escenarios con arquivos baleiros nun cartafol que ven Windows e WSL, así que non fan falta portos nin procesos extra.
- Mide varias veces seguidas (por defecto, 3 medicións de 60 s) e di se os resultados son **estábeis**: se os ciclos do terminal varían pouco entre medicións ou, en bigfile, o tempo de imprimir. Se non o son, hai que voltar medir coa máquina máis tranquila.
- Con `-Start`, `run-scenario.ps1` arranca el mesmo un terminal que acepta o programa na súa liña de comandos, como meiga, co escenario como programa: así non hai que escribir nada no terminal medido. Ata a subfase 1.7, era a única forma de medir meiga, que non tiña teclado.
- A CI comproba que os escenarios debuxan exactamente o esperado, en PowerShell e en bash, pero non mide nada.

O código explícase en [Código › escenarios de carga](codigo/escenarios.md). Na subfase 0.8 mediuse en local a liña base co Warp e o Windows Terminal, que fixou os orzamentos de rendemento de cada fase. Esas medicións non se publican, porque describen unha máquina concreta.

## 9. Estado actual

Despois da subfase 1.7:

- Existen os catro crates, coa regra de dependencias xa declarada. `meiga-core` ten as cores e a paleta, os rectángulos, o tamaño da grella, as zonas que cambian (*damage*), o *pacer* dos fotogramas, as celas do texto, o terminal, as comiñas da liña de comandos e as teclas, coma eventos e coma o que le o programa; `meiga-render`, o lenzo, o *framebuffer* e a súa presentación con GDI, e o texto: as fontes con DirectWrite, coa negra, a cursiva e as liñas do subliñado e do riscado, a caché de glifos e a mestura co fondo; e `meiga-win`, a xanela, os seus eventos, o teclado da xanela, o bucle de mensaxes, o temporizador dos fotogramas, os avisos desde outros fíos, os axustes do cursor de Windows e as sesións de ConPTY.
- `meiga.exe` é un terminal: lanza nunha pseudoconsola o programa da súa liña de comandos ou, sen el, PowerShell 7 (`pwsh`) ou, se non está instalado, Windows PowerShell, e mostra a súa saída: as cores (16, 256 e *truecolor*), a negra, a cursiva, o texto tenue, inverso e oculto, cinco subliñados e o riscado, os caracteres anchos e un cursor en forma de barra, que só pestanexa se a xanela ten o foco e Windows o permite, e para despois duns segundos sen actividade. A xanela leva o título que pon o programa. Responde ao programa, tamén ás preguntas polas cores e polo tamaño en píxeles, e aplica os cambios da paleta. Cando o programa remata, mostra o seu código de saída, e a xanela queda aberta ata que se pecha. A grella ten o tamaño da xanela ao arrancar (1.9).
- Pódese escribir nel coma nunha consola de Windows: as teclas chegan ao programa coma eventos enteiros, co teclado español e galego, AltGr e as teclas mortas, e o PSReadLine distingue Maiús+Intro de Intro. Alt+F4 pecha a xanela e Alt+Espazo abre o seu menú. O panel de emoji e o ditado de Windows tamén escriben nel. O rato aínda non chega ao programa (1.8).
- Cada fotograma presenta só as celas que cambian, que se poden ver cun bordo magenta (coa opción `--marks`). Como moito hai un fotograma por intervalo: 30 por segundo por Escritorio remoto e 60 en local. En repouso e coa xanela minimizada non debuxa nada, e os seus tres fíos dormen, tamén coa xanela enfocada. Ao pechala, meiga remata co código 0, e o programa da sesión tamén.
- Ten tres modos de medición, `--bench partial`, `--bench full` e `--bench text`, e un rexistro dos fotogramas, `--trace-frames`.
- `meiga-win` usa o ConPTY do Windows Terminal que vai no paquete ou, se non está, o de Windows. As probas lanzan programas de verdade cos dous, e un exemplo de consola mostra o que escribe calquera programa e grávao. O terminal de `meiga-core` usa `alacritty_terminal` detrás dunha fachada; as súas probas usan saída gravada do ConPTY, e outro exemplo de consola mide a que velocidade le unha gravación.
- `meiga.exe` leva dentro o manifesto da aplicación, ocupa uns 475 KB e só depende de DLL do sistema, entre elas a de DirectWrite. A CI compróbao en cada cambio: as DLL, que abre a xanela e lanza o programa, a escala por monitor, que o título é o que pon o programa, que presenta só o que cambia, que as teclas chegan ao programa co teclado español e co estadounidense, que remata ben, e que o programa remata con ela.
- Están activas as regras de calidade (*lints*), o formato automático, a política de licenzas (`cargo-deny`) e a xeración dos avisos de terceiros (`cargo-about`). As dependencias externas son `windows-sys`, `windows` e `windows-core`, de Microsoft, `unicode-width`, `alacritty_terminal`, co seu parser, `vte`, e `memchr`.
- Cada PR valídase automaticamente en GitHub Actions, tamén as ligazóns da documentación, os scripts de PowerShell, o arnés de medición e os escenarios de carga, e produce un `.zip` descargábel con `meiga.exe` e o ConPTY (`conpty.dll` e `OpenConsole.exe`). `tools/fetch-conpty.ps1` descarga o ConPTY do seu paquete NuGet, cunha versión e un SHA-256 fixados, e comproba que está asinado por Microsoft; cada luns, a CI avisa se hai unha versión nova.
- As decisións de arquitectura están escritas en catorce ADR.
- O arnés de medición mide calquera terminal e as partes de Windows que o poñen na pantalla. `measure-startup.ps1` mide canto tarda meiga en abrir a súa xanela, coa súa sesión, e `measure-frames.ps1`, canto custan os seus fotogramas: parciais, completos e completos de texto.
- Os escenarios de carga imitan o uso diario dun terminal, en PowerShell e en bash, e `run-scenario.ps1` mide un terminal mentres corren e di se os resultados son estábeis. Con `-Start`, arranca el mesmo meiga co escenario como programa.
- Os scripts de `tools/` necesitan PowerShell 7. En Windows PowerShell 5.1 non se executan: só mostran o comando para facelo con `pwsh`.
