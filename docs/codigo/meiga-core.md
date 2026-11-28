# Código › `meiga-core`

> Estado: subfase 4.1. Contén o modelo das xanelas e as súas tabs, a disposición da barra de tabs, que as despraza cando non caben, e o que é cada parte dela para Windows, coma barra de título da xanela, os menús (as súas entradas, o elemento resaltado e onde vai cada parte), o campo de texto (unha liña que se edita, co cursor entre os grafemas, as palabras, desfacer, o pegado, o límite, o rato e a parte do texto que se ve), o arquivo de configuración, os atallos de teclado (as combinacións, as accións e os atallos de meiga, cos do arquivo por riba), os perfís e o contorno dos programas das tabs, as cores e a paleta, coa cor da selección, a xeometría (puntos, tamaños e rectángulos, e o tamaño e as celas da grella), as zonas que cambiaron (*damage*), o ritmo dos fotogramas, as celas do texto, e como cortalo con «…», a liña de comandos dun programa e o seu nome, as teclas, coma eventos e coma o que le o programa, o rato (a cela debaixo do punteiro, os clics seguidos, a roda, en partes e en filas do historial, e os informes para os programas), o texto que se pega, e o terminal: a fachada sobre `alacritty_terminal`, coas súas respostas, a súa paleta, o título do programa e os que este garda, a selección e a vista do historial, o cambio de tamaño, que coloca a pantalla coma o ConPTY e garda á parte a principal mentres se ve a alternativa, o reinicio para un programa novo na mesma tab, o escáner previo, que le o que di a shell de si mesma, e as gravacións. Na 2.1 chegou o modelo, coas súas probas e un exemplo para probalo a man, e na 2.2, para as tabs de `meiga.exe`, `Window::contents_mut` e `Screen::forget`. Na 2.3, para a barra de tabs, chegaron o módulo `ui`, coa disposición da barra, `Rect::contains_point`, `text::fit`, que corta os títulos con «…», e `shell::program_name`, o nome que mostra unha tab sen título. Na 2.4, para o menú da tab, chegaron o módulo `ui::menu`, coas entradas dun menú, o resalte e onde vai cada parte, e `Window::close_others` e `Window::close_to_the_right`, que pechan varias tabs dunha vez. Na 2.5, para os perfís, chegaron os módulos `config`, que le o `config.toml` e di onde está cada erro, `profile`, que xunta as shells que atopa meiga co que di o arquivo e escolle o perfil por defecto, `environment`, o contorno dos programas das tabs, e `sort`, unha ordenación por inserción para as listas curtas; e, na barra, o botón que abre o menú dos perfís e, nos menús, os elementos en negra. Na 2.6, para os atallos de teclado, chegou o módulo `shortcut`, coas combinacións das teclas, as accións e os atallos de meiga, que o arquivo pode engadir, cambiar ou liberar; `config` le a táboa `[shortcuts]` do `config.toml`, e `ControlKeys::alt_gr`, que di se o Ctrl e o Alt son os de AltGr, pasou a ser pública, para que AltGr non dispare ningún atallo. Na 2.7, para o fin do programa dunha tab, chegaron ao terminal `restart`, que o deixa coma o espera un programa novo, co que escribiu o anterior no historial, e `at_empty_row_start`, que di se o cursor está ao principio dunha fila baleira que non continúa a de enriba, tamén cando a vista mostra o historial. Na 2.8, para as moitas tabs, chegaron á disposición da barra o ancho mínimo das tabs, 100 píxeles a escala 100 %, e, cando nin así caben, a franxa entre dous botóns onde van, co seu desprazamento: unha tab, uns píxeles ou o mínimo para que se vexa enteira a tab que se mostra; e as tabs que manteñen o seu ancho mentres se pechan co rato. Na 2.9, para as tabs na barra de título, a disposición da barra leva, coa xanela restaurada, unha banda enriba das tabs, onde se redimensiona a xanela, e, á dereita da barra, os botóns da xanela, minimizar, maximizar ou restaurar e pechar, cun sitio para arrastrar a xanela sempre antes deles, e di o ancho da barra máis estreita; e chegou o módulo `ui::titlebar`, que di que é cada parte da barra para Windows, a barra de título, o bordo de arriba e a súa esquina, un botón da xanela ou a área cliente, tamén cun menú de meiga aberto, e o tamaño mínimo da área cliente. Na 2.10 e no M3 non cambiou o código. Na 3.1, para os títulos dinámicos, chegou ao terminal o módulo `term::title`: o terminal leva el mesmo o título do programa e os que este garda, e non `alacritty_terminal`, que os baleiraba sen avisar nun reinicio da pantalla (`ESC c`); `ESC c` mantén os dous, e `restart`, coma nun terminal novo, quítaos; garda como moito 10 títulos, ignora os de máis de 1000 unidades de UTF-16, e o mesmo título outra vez xa non é un cambio. Chegaron sete probas en `term/title.rs` e sete en `term/tests.rs`. Na 3.2, para o campo de texto, chegou o módulo `ui::field`: unha liña de texto que se edita coma nos campos de texto de Chrome en Windows, sen xanela, e que usarán renomear as tabs e os grupos e o filtro do explorador. O cursor para entre os grafemas de Unicode, que dá unha dependencia nova, `unicode-segmentation`; Ctrl e as frechas van por palabras; desfai e refai ata 100 pasos; pega nunha liña; ten un límite en unidades de UTF-16; colle o rato en celas; e mostra a parte do texto onde está o cursor. `own_character`, de `input.rs`, pasou a ser `pub(crate)`, para que o campo saiba, coma o terminal, se o carácter dunha tecla é seu. Chegaron setenta e dúas probas, todas en `ui/field/`: 55 en `tests.rs`, once en `history.rs` e seis en `segment.rs`. Na 3.3, que renomea as tabs co campo, chegou unha acción, `rename_tab`, con Ctrl+Maiús+R por defecto ([ADR 0031](../adr/0031-o-nome-das-tabs.md)), e a proba dos atallos de meiga compróbaa. Na 4.1, para o que di a shell de si mesma, o escáner previo reescribiuse ([ADR 0032](../adr/0032-o-escaner-osc.md)): xa non segue o win32-input-mode (`?9001`) nin o reinicio da pantalla (`ESC c`), que agora os segue `Hooked`, o *handler* de `vte`, na orde da saída; busca onde empeza cada OSC, `ESC ]`, con `memchr`, e le OSC 7 e OSC 9;9, o cartafol, e OSC 133, as marcas arredor do *prompt* e das ordes, coma `vte`, tamén partidos entre lecturas, e garda como moito 128 KiB do texto de cada un. Chegaron o módulo `term::shell_state`, co cartafol, a última marca e o código de saída da última orde, que dan `Terminal::shell` e `Changes::shell`; `proptest`, unha dependencia só para as probas; unha gravación de pwsh cun *prompt* que escribe OSC 133 e 9;9; e un *benchmark*, `benches/scanner.rs`, que mide o escáner fronte ao terminal. `replay --screen` mostra tamén o que dixo a shell. Chegaron vinte e sete probas, e quitáronse as dez do escáner anterior: dezaoito en `term/scanner/tests.rs`, unha delas con `proptest`, que compara o escáner con `vte`; oito en `term/tests.rs`, cinco delas do win32-input-mode, cos casos das do escáner anterior; e unha en `tests/recordings.rs`.

**Responsabilidade:** toda a lóxica que non depende de Windows: o modelo de xanelas, tabs e grupos; a fachada do terminal; a codificación do teclado e do rato, e, desde a subfase 2.6, os atallos de teclado; a busca difusa; e a configuración, que desde a subfase 2.5 son o `config.toml`, os perfís e o contorno dos programas. Sen `unsafe` e sen APIs de Windows: o rexistro, os arquivos e o contorno de meiga lenos `meiga-win` e o programa, e a este crate só lle chegan os seus datos. Ver [Arquitectura](../arquitectura.md#2-estrutura-do-código-un-workspace-con-catro-crates).

| Arquivo | Contido |
|---|---|
| [`Cargo.toml`](#cratesmeiga-corecargotoml) | O crate e as súas dependencias: `alacritty_terminal`, `memchr`, `toml`, `unicode-segmentation`, desde a 3.2, e `unicode-width`; e, desde a 4.1, `proptest`, só para as probas, e o *benchmark* do escáner. |
| [`src/lib.rs`](#cratesmeiga-coresrclibrs) | A raíz do crate: os módulos. |
| [`src/color.rs`](#cratesmeiga-coresrccolorrs) | `Rgb`, o tipo das cores, e `Palette`, as cores dun terminal: as 16 do esquema *Campbell*, as 256 do xterm e a da selección. |
| [`src/geometry.rs`](#cratesmeiga-coresrcgeometryrs) | `Point`, `Size` e `Rect`: posicións, tamaños e rectángulos en píxeles; `GridSize`, o tamaño da grella en celas; e `GridPoint`, unha cela da grella. |
| [`src/damage.rs`](#cratesmeiga-coresrcdamagers) | `Damage`: as zonas da xanela que cambiaron desde o último fotograma. |
| [`src/pacing.rs`](#cratesmeiga-coresrcpacingrs) | `FramePacer`: cando debuxar o seguinte fotograma. |
| [`src/text.rs`](#cratesmeiga-coresrctextrs) | `CellChar` e `cells`: o texto como celas dunha grella; e `fit`, as celas dun texto que caben nunhas columnas, cortado con «…». |
| [`src/shell.rs`](#cratesmeiga-coresrcshellrs) | `command_line`: a liña de comandos dun programa cos seus argumentos, entre comiñas segundo as regras de Windows; e `program_name`, o nome do programa dunha liña de comandos, o que mostra unha tab sen título. |
| [`src/input.rs`](#cratesmeiga-coresrcinputrs) | `KeyEvent`, unha tecla que baixa ou sobe, coas súas teclas modificadoras, `ControlKeys`, que din tamén se o Ctrl e o Alt son os de AltGr; e `KeyEncoder`, que a converte no que le o programa: en win32-input-mode ou en VT. Os eventos de foco. Desde a 3.2, o campo de texto usa tamén `own_character`, que di se o carácter dunha tecla é seu. |
| [`src/mouse.rs`](#cratesmeiga-coresrcmousers) | O rato: `cell_at`, a cela debaixo do punteiro; `Clicks`, que conta os clics seguidos; `WheelTurns`, que suma os xiros da roda en partes, e `WheelRows`, en filas do historial; `report`, os informes para os programas que piden o rato, en SGR, en UTF-8 ou no formato do primeiro xterm; e `arrow_keys`, a roda coma frechas. |
| [`src/paste.rs`](#cratesmeiga-coresrcpasters) | `Paste`: o texto que le o programa ao pegar, sen caracteres de control, cos fins de liña en CR e, se o pediu, entre as marcas do *bracketed paste*. |
| [`src/config.rs`](#cratesmeiga-coresrcconfigrs) | `parse`: o arquivo de configuración, `config.toml`, lido sen serde, co perfil por defecto, as entradas dos perfís e, desde a 2.6, os atallos de `[shortcuts]`; e `Problem`, cada cousa que ignora, coa súa liña e a súa columna. |
| [`src/profile.rs`](#cratesmeiga-coresrcprofilers) | `Profiles`: os perfís do menú, as shells que atopa meiga co que cambia, oculta ou engade a configuración, e o perfil por defecto; e `Launch`, o que corre a sesión dunha tab: o nome, a liña de comandos e o cartafol, co `--cd` de WSL. |
| [`src/environment.rs`](#cratesmeiga-coresrcenvironmentrs) | `Environment`: as variábeis dun contorno, en UTF-16 coma as garda Windows, as `%NOME%` dun texto e o bloque para `CreateProcessW`; e `prepare`, o contorno dos programas das tabs, con `TERM_PROGRAM`, `COLORTERM` e o `WSLENV`. |
| [`src/sort.rs`](#cratesmeiga-coresrcsortrs) | `insertion_sort_by`: unha ordenación estábel, por inserción, para as listas curtas, máis pequena ca a da biblioteca estándar. |
| [`src/shortcut.rs`](#cratesmeiga-coresrcshortcutrs) | Os atallos de teclado: `Chord`, unha combinación, as teclas modificadoras e unha tecla, a dunha tecla que baixa ou a dun texto coma `ctrl+shift+t`; `Action`, o que fai un atallo, co seu nome no arquivo; e `Shortcuts`, os atallos de meiga cos do `config.toml` por riba, e a acción de cada combinación. Desde a 3.3, `rename_tab`, Ctrl+Maiús+R. |
| [`src/model/mod.rs`](#cratesmeiga-coresrcmodelmodrs) | O modelo das xanelas e as súas tabs: os identificadores, `TabId` e `WindowId`, que nunca se repiten, e os erros. |
| [`src/model/window.rs`](#cratesmeiga-coresrcmodelwindowrs) | `Window`: as tabs dunha xanela, na orde da barra, e a activa, a enfocada e as visíbeis; abrir, pechar, tamén varias dunha vez, mover e activar. |
| [`src/model/tests.rs`](#cratesmeiga-coresrcmodeltestsrs) | As probas do modelo, unha delas con 10 000 cambios ao chou. |
| [`src/term/mod.rs`](#cratesmeiga-coresrctermmodrs) | `Terminal`: a fachada sobre `alacritty_terminal`, cos seus eventos, modos e respostas, a paleta, o cambio de tamaño, a vista do historial e a copia do que cambia, coas celas seleccionadas; desde a 2.7, o reinicio para o programa que empeza outra vez na súa tab; e, desde a 4.1, o que di a shell de si mesma, `Terminal::shell`. |
| [`src/term/resize.rs`](#cratesmeiga-coresrctermresizers) | O cambio de tamaño da pantalla principal, que reparte o texto e coloca a pantalla onde a pon o ConPTY, seguindo a primeira fila e o cursor cunhas marcas; e a pantalla principal á parte mentres se ve a alternativa. |
| [`src/term/handler.rs`](#cratesmeiga-coresrctermhandlerrs) | `Hooked`: o `Handler` entre o parser e o `Term`, que lle pasa as ordes, devolve a pantalla principal cando o programa deixa a alternativa ou reinicia o terminal, e deixa a vista do historial onde está cando se despraza unha rexión que non empeza arriba; desde a 3.1, o título e os títulos gardados non llos pasa: lévaos ao terminal; e, desde a 4.1, segue o win32-input-mode (`?9001`), que `alacritty_terminal` ignora. |
| [`src/term/title.rs`](#cratesmeiga-coresrctermtitlers) | `Titles`, desde a 3.1: o título que lle dá o programa á súa tab e os que garda para dalos outra vez, como moito 10; un título baleiro é ningún, un de máis de 1000 unidades de UTF-16 ignórase, e o mesmo outra vez non é un cambio. |
| [`src/term/screen.rs`](#cratesmeiga-coresrctermscreenrs) | `Screen`: o que mostra o terminal, en tipos de meiga, coa súa paleta, e as celas que cambiaron. |
| [`src/term/selection.rs`](#cratesmeiga-coresrctermselectionrs) | A selección: creala, movela e quitala, o seu texto, e as súas celas, na grella de `alacritty_terminal`. |
| [`src/term/scanner.rs`](#cratesmeiga-coresrctermscannerrs) | `Scanner`: o escáner que le a saída antes do parser. Desde a 4.1, busca nela onde empeza cada OSC, `ESC ]`, e le o que di a shell, o seu cartafol (OSC 7 e 9;9) e as marcas arredor do *prompt* (OSC 133), coma `vte`, tamén partido entre lecturas; ata entón, seguía `?9001`. |
| [`src/term/scanner/tests.rs`](#cratesmeiga-coresrctermscannertestsrs) | Desde a 4.1, as probas do escáner, unha delas con `proptest`, que o compara con `vte` en saídas aleatorias partidas en anacos aleatorios. |
| [`src/term/shell_state.rs`](#cratesmeiga-coresrctermshell_staters) | Desde a 4.1, `ShellState`: o que dixo a shell dunha tab, o seu cartafol (`Folder`), a súa última marca (`Mark`) e o código de saída da última orde. |
| [`src/term/recording.rs`](#cratesmeiga-coresrctermrecordingrs) | `Recorder` e `Recording`: gravar a saída dun programa e lela. |
| [`src/term/tests.rs`](#cratesmeiga-coresrctermtestsrs) | As probas do terminal. |
| [`src/ui/mod.rs`](#cratesmeiga-coresrcuimodrs) | As partes da interface que debuxa meiga, sen Windows: as disposicións da barra de tabs e dos menús, e, desde a 2.9, o que é a barra de tabs para Windows, coma barra de título; `scale`, que leva as súas medidas ao DPI da xanela; e, desde a 3.2, o campo de texto, que edita unha liña de texto. |
| [`src/ui/tabbar.rs`](#cratesmeiga-coresrcuitabbarrs) | `Layout`: onde van as tabs da barra, os seus títulos, os seus botóns de pechar e os separadores, e os botóns que abren unha tab e o menú dos perfís, ao DPI da xanela; desde a 2.8, o ancho mínimo das tabs e, cando nin así caben, a franxa entre os dous botóns que as desprazan, e o seu desprazamento; desde a 2.9, a banda enriba das tabs, os botóns da xanela e os seus glifos, o sitio para arrastrar a xanela e o ancho da barra máis estreita; e `Part`, a parte da barra que hai debaixo dun punto. |
| [`src/ui/titlebar.rs`](#cratesmeiga-coresrcuititlebarrs) | `TitleBar`: a barra de tabs coma barra de título da xanela, e `Hit`, o que é para Windows cada punto dela: a barra de título, que arrastra a xanela, o bordo de arriba e a súa esquina, que a redimensionan, un botón da xanela ou a área cliente, tamén cun menú de meiga aberto; onde se abre con Alt+Espazo o menú da xanela, `menu_corner`; e `min_client`, o tamaño mínimo da área cliente. |
| [`src/ui/menu.rs`](#cratesmeiga-coresrcuimenurs) | `Menu`: as entradas dun menú, elementos, tamén en negra, e liñas, e o resalte, que moven o teclado e o punteiro; e `Layout`: onde van o menú e as súas entradas, ao DPI da xanela, e a entrada que hai debaixo dun punto. |
| [`src/ui/field/mod.rs`](#cratesmeiga-coresrcuifieldmodrs) | Desde a 3.2, `Field`: o campo de texto sen xanela, unha liña que o usuario edita, co texto, o cursor e a selección entre os grafemas; `set_text`, `select_all` e `paste`, que pega nunha liña o que colla no límite; o rato en celas, con `press` e `drag`; `show`, a parte do texto que se ve, que segue o cursor, e onde van o cursor e a selección; e `take_changes`, o que cambiou. `Reply`, o que fixo unha tecla. |
| [`src/ui/field/segment.rs`](#cratesmeiga-coresrcuifieldsegmentrs) | Onde para o cursor dun campo de texto, entre os grafemas de Unicode, con `unicode-segmentation`; e as palabras, para Ctrl e as frechas e o dobre clic: as letras, os números e `_`, os outros signos e os espazos. |
| [`src/ui/field/history.rs`](#cratesmeiga-coresrcuifieldhistoryrs) | `History`: os pasos para desfacer e refacer nun campo de texto, ata 100, e cando se xunta unha edición co paso anterior. |
| [`src/ui/field/keys.rs`](#cratesmeiga-coresrcuifieldkeysrs) | `Field::key`: o que fai cada tecla nun campo de texto, coma nos de Chrome en Windows. |
| [`src/ui/field/tests.rs`](#cratesmeiga-coresrcuifieldtestsrs) | As probas do campo de texto, unha delas con 300 series de 45 accións ao chou. |
| [`tests/recordings.rs`](#cratesmeiga-coretestsrecordingsrs) e `tests/data/` | Probas con gravacións reais do ConPTY. |
| [`benches/scanner.rs`](#cratesmeiga-corebenchesscannerrs) | Desde a 4.1, o *benchmark* do escáner: o que custa fronte ao tempo do terminal, coa mesma saída. |
| [`examples/replay.rs`](#cratesmeiga-coreexamplesreplayrs) | Un exemplo de consola que lle dá unha gravación ao terminal e mide a velocidade. Con `--screen`, mostra o que quedou, desde a 4.1 tamén o que dixo a shell. |
| [`examples/tabs.rs`](#cratesmeiga-coreexamplestabsrs) | Un exemplo de consola para probar o modelo das tabs a man. |

## `crates/meiga-core/Cargo.toml`

```toml
[package]
name = "meiga-core"
description = "Platform-independent core of meiga: model, terminal facade, input encoding, search and configuration."
version.workspace = true
edition.workspace = true
rust-version.workspace = true
license.workspace = true
repository.workspace = true
publish.workspace = true

[dependencies]
# The terminal core, behind the facade of the term module (ADR 0003).
alacritty_terminal.workspace = true
# Finds where each OSC starts (ESC ]) fast, in the scanner that reads the
# output before the parser.
memchr.workspace = true
# Reads config.toml.
toml.workspace = true
# The grapheme clusters of Unicode (UAX #29), between which the caret of a
# text field stops (ADR 0030).
unicode-segmentation.workspace = true
# The width of each character in cells, as alacritty_terminal measures it.
unicode-width.workspace = true

[dev-dependencies]
proptest.workspace = true

# What the scanner costs against the terminal (ADR 0032): cargo bench -p
# meiga-core --bench scanner. Without the test harness, which needs nightly.
[[bench]]
name = "scanner"
harness = false

[lints]
workspace = true
```

- `name`: o nome do crate. No código úsase con guión baixo: `meiga_core`.
- `description`: unha liña que describe o crate.
- As liñas `campo.workspace = true` herdan os valores de `[workspace.package]` do `Cargo.toml` raíz.
- `[dependencies]`, coas versións e as opcións do `Cargo.toml` raíz ([Código › workspace](workspace.md#cargotoml-raíz)):
  - desde a subfase 1.5, **`alacritty_terminal`**, o núcleo do terminal, que trae o seu parser, `vte` ([Tecnoloxías › `alacritty_terminal` e `vte`](../tecnoloxias.md#34-alacritty_terminal-e-vte-o-núcleo-do-terminal)). Só o usa o módulo `term`;
  - desde a 1.5, **`memchr`**, que busca un byte nun texto moitos bytes á vez, con instrucións SIMD, e, con `memchr::memmem`, un anaco de texto. O escáner busca así onde empeza cada OSC, `ESC ]`, coma di o comentario: desde a 4.1, o seu `]` con `memchr` e, nunha lectura con moitos `]` sen ESC, a parella con `memmem` ([`next_start`](#buscar-o-comezo-find_start-next_start-e-start)); ata a 4.1, buscaba o seguinte ESC. `vte` busca o seguinte ESC co mesmo crate, así que non engade ningún crate novo, e `memmem` só engade o seu código a `meiga.exe`;
  - desde a 2.5, **`toml`**, o parser de TOML, sen serde: `default-features = false`, e só as *features* `parse` e `std` ([ADR 0022](../adr/0022-os-perfis-e-a-configuracion.md), decisión 1). Só o usa o módulo `config`, que le con el o `config.toml`: o parser dá cada chave e cada valor co seu sitio no arquivo, e meiga colle a man o que coñece ([Tecnoloxías › Os perfís e a configuración](../tecnoloxias.md#57-os-perfís-e-a-configuración-toml-o-contorno-e-as-shells-de-windows)). Compílanse catro crates máis: `toml_parser`, o parser; `winnow`, a biblioteca de parsers coa que está feito; `toml_datetime`, as datas de TOML; e `serde_spanned`, o `Spanned` que garda o sitio de cada valor, que sen as súas *features* non usa serde. En `meiga.exe`, o parser, con eles, ocupa uns 123 KB;
  - desde a 3.2, **`unicode-segmentation`**, os grafemas de Unicode, segundo UAX #29 e coas táboas de Unicode 17: o cursor dun campo de texto para entre eles ([ADR 0030](../adr/0030-o-campo-de-texto.md), decisión 2; [Tecnoloxías › Os grafemas e as palabras dun campo de texto](../tecnoloxias.md#66-os-grafemas-e-as-palabras-dun-campo-de-texto)). Só o usa o módulo `ui::field`: `GraphemeCursor`, en [`segment.rs`](#cratesmeiga-coresrcuifieldsegmentrs), e `grapheme_indices`, en [`mod.rs`](#cratesmeiga-coresrcuifieldmodrs). É dos mesmos autores ca `unicode-width`; non ten *features* por defecto que quitar, así que o `Cargo.toml` raíz pídeo sen `default-features = false`, nin depende de ningún outro crate; e a súa licenza, MIT ou Apache-2.0, coma a das outras, cumpre a política de licenzas ([ADR 0006](../adr/0006-licenzas-e-dependencias.md)). As súas táboas só chegan a `meiga.exe` cando o programa usa o campo, desde a 3.3: uns 29,7 KB, medidos cunha compilación de proba que o usaba;
  - desde a 1.3, **`unicode-width`**, que dá o ancho de cada carácter en celas, como o mide `alacritty_terminal` ([Tecnoloxías › O ancho dos caracteres](../tecnoloxias.md#29-o-ancho-dos-caracteres-unicode-width)).

  Os cinco son código Rust que non usa APIs de ningún sistema desde o que chama meiga, así que respectan as regras deste crate. `alacritty_terminal` ten tamén un PTY propio con código de Windows, que meiga non usa e que o ligador elimina.
- `[dev-dependencies]`, desde a subfase 4.1: as dependencias que só usan as probas, e que non chegan a `meiga.exe`. **`proptest`** proba moitas entradas ao chou e, cando unha falla, redúcea á máis curta que segue fallando ([ADR 0032](../adr/0032-o-escaner-osc.md), decisión 6). Úsao a proba que compara o escáner con `vte` ([`scanner/tests.rs`](#cratesmeiga-coresrctermscannertestsrs)). O `Cargo.toml` raíz pídeo só coa *feature* `std`, sen as que van por defecto, coma `fork` e `timeout`, que executan os casos noutro proceso e cun tempo máximo, nin as súas dependencias, coma di o seu comentario ([Código › workspace](workspace.md#cargotoml-raíz)). Con el entran no `Cargo.lock` once crates máis, coma `rand`, `rand_xorshift` e `chacha20`, cos que fai os seus números aleatorios. A política de licenzas xa o permitía para as probas ([ADR 0006](../adr/0006-licenzas-e-dependencias.md)); na 2.1, para a proba ao chou do modelo, preferiuse unha secuencia *xorshift* fixa, sen dependencias ([ADR 0018](../adr/0018-o-modelo-das-tabs.md)), e as outras probas ao chou de meiga seguen así.
- `[[bench]]`, desde a 4.1: o *benchmark* do escáner, [`benches/scanner.rs`](#cratesmeiga-corebenchesscannerrs), que se executa con `cargo bench -p meiga-core --bench scanner`, coma di o comentario. `harness = false` quítalle o arnés das probas, que nos *benchmarks* só funciona con Rust *nightly*: é un programa co seu `main`, que mide el mesmo.
- `[lints] workspace = true`: aplica as regras de calidade comúns.

## `crates/meiga-core/src/lib.rs`

```rust
//! Platform-independent core of meiga.
//!
//! This crate holds the logic that does not depend on Windows: the window and
//! tab model, the terminal facade, input encoding, search and configuration.
//! It must stay free of `unsafe` code and of platform APIs, so that it is easy
//! to test and fast to compile.

#![forbid(unsafe_code)]

pub mod color;
pub mod config;
pub mod damage;
pub mod environment;
pub mod geometry;
pub mod input;
pub mod model;
pub mod mouse;
pub mod pacing;
pub mod paste;
pub mod profile;
pub mod shell;
pub mod shortcut;
pub mod sort;
pub mod term;
pub mod text;
pub mod ui;
```

- `lib.rs` é a raíz dunha biblioteca: o compilador empeza por este arquivo, e desde el decláranse os demais módulos.
- As liñas que empezan por `//!` son **documentación do crate**. `cargo doc` convérteas en documentación HTML.
- `#![forbid(unsafe_code)]` é un **atributo interno** (`#!`), que se aplica a todo o crate. Prohíbe calquera bloque `unsafe` en `meiga-core`. Co nivel `forbid`, ningún módulo pode desactivar a regra cun `#[allow]`. Así, esta parte do programa, que será a máis grande, conta con todas as garantías de seguridade de memoria de Rust.
- `pub mod color;` declara o módulo `color`, que está en `src/color.rs`. Con `pub`, os outros crates poden usalo como `meiga_core::color`. O mesmo cos demais; `shell` chegou na subfase 1.6, `input`, na 1.7, `mouse` e `paste`, na 1.8, `model`, na 2.1, `ui`, na 2.3, `config`, `environment`, `profile` e `sort`, na 2.5, e `shortcut`, na 2.6. `term`, desde a 1.5, `model` e `ui` teñen submódulos, así que van en cartafoles: os módulos están en `src/term/mod.rs`, en `src/model/mod.rs` e en `src/ui/mod.rs`.
- As dependencias, como `alacritty_terminal`, poden ter `unsafe` dentro: `forbid` só se aplica ao código deste crate.

## `crates/meiga-core/src/color.rs`

As cores e, desde a subfase 1.6, a paleta dun terminal: a cor coa que se debuxa cada número das celas ([ADR 0013](../adr/0013-da-saida-a-pantalla.md), [Tecnoloxías › A paleta e os estilos](../tecnoloxias.md#37-a-paleta-e-os-estilos-das-celas)). Desde a 1.8, a paleta ten tamén a cor da selección ([ADR 0015](../adr/0015-o-rato-a-seleccion-e-o-portapapeis.md)).

### `Rgb`

```rust
//! Colours, and the palette of a terminal.

/// A colour with 8 bits per channel and no transparency.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Rgb {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Rgb {
    pub const fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }
}
```

Unha cor en RGB, con 8 bits (0–255) por canle: vermello (`r`), verde (`g`) e azul (`b`). Desde a subfase 1.2 úsase para todo o que se debuxa no *framebuffer*, desde a 1.3 tamén para as cores do texto, e desde a 1.6 para a paleta do terminal; máis adiante servirá para os temas e a configuración.

- Está en `meiga-core`, e non nun crate de Windows, porque unha cor non depende de Windows. Cada uso convértea ao seu formato: `meiga-render` convértea nun píxel do *framebuffer* (`0x00RRGGBB`) e, para as brochas de GDI, nun `COLORREF` (`0x00BBGGRR`); ver [Código › meiga-render](meiga-render.md#pixel).
- `#[derive(…)]` fai que o compilador escriba automaticamente estas implementacións:
  - `Clone` e `Copy`: unha cor son 3 bytes, así que se copia ao asignala ou ao pasala a unha función, en vez de *moverse* (en Rust, por defecto, asignar un valor transfire a súa propiedade e o orixinal xa non se pode usar);
  - `Debug`: permite mostrala nas mensaxes de diagnóstico con `{:?}`;
  - `PartialEq` e `Eq`: permiten comparar dúas cores con `==`;
  - `Hash`: permite usala como clave nun `HashMap`, por exemplo nunha táboa cun valor por cor.
- `const fn new`: unha función **constante**, que o compilador pode executar mentres compila. Por iso as cores se poden declarar como constantes: `const BACKGROUND: Rgb = Rgb::new(0x16, 0x18, 0x1d);`.
- Os campos son públicos (`pub`): calquera combinación de tres bytes é unha cor válida, así que non hai ningunha regra que protexer.

```rust
pub const fn mix(self, other: Rgb, amount: u8) -> Rgb {
    const fn channel(from: u8, to: u8, amount: u8) -> u8 {
        let (from, to, amount) = (from as u32, to as u32, amount as u32);
        // Rounded to the nearest. The result is between `from` and `to`,
        // so it fits in a byte.
        ((from * (255 - amount) + to * amount + 127) / 255) as u8
    }
    Rgb::new(
        channel(self.r, other.r, amount),
        channel(self.g, other.g, amount),
        channel(self.b, other.b, amount),
    )
}
```

**`mix`**, desde a subfase 1.6, dá a cor que está a `amount` do camiño desde esta ata `other`, en 255 avos: con 0, esta; con 255, `other`; e con 85, a un terzo do camiño. A vista do terminal úsaa para o texto tenue (*SGR* 2): a cor do texto, un terzo do camiño cara á do fondo ([Código › meiga › `view.rs`](meiga.md#cratesmeigasrcviewrs)).

- Cada canle é unha media ponderada das dúas cores: `from × (255 − amount) + to × amount`, entre 255. Sumar 127, a metade de 255, antes de dividir redondea ao enteiro máis próximo: a división de enteiros só corta os decimais.
- As contas fanse en `u32`, porque os produtos non caben nun `u8`. A conversión é con `as`, e non con `u32::from`, porque unha `const fn` non pode chamar métodos de traits, como `From::from`. O resultado sempre está entre `from` e `to`, así que o `as u8` do final nunca corta nada.
- `channel` é unha función dentro doutra: só a ve `mix`. Tamén é `const fn`, para que `mix` o poida ser: unha cor mesturada pode ser unha constante.

### `Palette`

```rust
pub const CAMPBELL: [Rgb; 16] = [
    Rgb::new(0x0c, 0x0c, 0x0c),
    Rgb::new(0xc5, 0x0f, 0x1f),
    …
    Rgb::new(0xf2, 0xf2, 0xf2),
];

pub const FOREGROUND: Rgb = Rgb::new(0xd4, 0xd7, 0xdd);
pub const BACKGROUND: Rgb = Rgb::new(0x16, 0x18, 0x1d);
pub const SELECTION: Rgb = Rgb::new(0x26, 0x4f, 0x78);
```

- **`CAMPBELL`**: as 16 cores do esquema *Campbell*, o que usan por defecto o Windows Terminal e a consola de Windows desde Windows 10: negro, vermello, verde, amarelo, azul, magenta, cian e branco, e as súas versións brillantes, na orde das secuencias de cor (`30`–`37` e `90`–`97`). Os valores, nunha táboa, en [Tecnoloxías › A paleta](../tecnoloxias.md#37-a-paleta-e-os-estilos-das-celas).
- **`FOREGROUND`** e **`BACKGROUND`**: o texto e o fondo de meiga, `#D4D7DD` e `#16181D`, en vez dos de *Campbell*. Antes estaban na demo de `meiga.exe`, que desaparece na subfase 1.6: agora viven xunto coa paleta, e os modos de medición (`--bench`) tamén as usan.
- **`SELECTION`**, desde a subfase 1.8: o fondo das celas seleccionadas, `#264F78`, o do tema escuro do Visual Studio Code. O texto mantén a súa cor ([ADR 0015](../adr/0015-o-rato-a-seleccion-e-o-portapapeis.md)).

```rust
const CUBE_LEVELS: [u8; 6] = [0x00, 0x5f, 0x87, 0xaf, 0xd7, 0xff];

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Palette {
    pub colors: [Rgb; 256],
    pub foreground: Rgb,
    pub background: Rgb,
    pub cursor: Rgb,
    pub selection: Rgb,
}
```

**`Palette`**, desde a subfase 1.6: as cores dun terminal. As celas gardan o número da súa cor, non a cor ([`Color`](#cratesmeiga-coresrctermscreenrs)), e a paleta dá a cor de cada número ao debuxalas:

| Campo | Que é |
|---|---|
| `colors` | As 256 cores que os programas escollen polo seu número: 0–15 as que teñen nome, 16–231 un cubo de 6 × 6 × 6 e 232–255 grises, de escuro a claro. |
| `foreground`, `background` | O texto e o fondo cando o programa non escolle ningunha cor (`Color::Default`). |
| `cursor` | A cor do cursor. |
| `selection` | Desde a subfase 1.8: o fondo das celas seleccionadas. Os programas non a poden cambiar: non ten número nas secuencias, e o terminal cóllea do tema tal cal ([A paleta](#a-paleta-o-tema-e-os-cambios-do-programa)). |

- Os campos son públicos, como os de `Rgb`: calquera combinación é unha paleta válida.
- Deriva `Clone`, `Debug`, `PartialEq` e `Eq`, pero non `Copy`, ao contrario ca `Rgb`: son 780 bytes, e así cada copia vese no código, cun `clone`.

```rust
impl Palette {
    pub const MEIGA: Palette =
        Palette::new(CAMPBELL, FOREGROUND, BACKGROUND, FOREGROUND, SELECTION);

    pub const fn new(
        named: [Rgb; 16],
        foreground: Rgb,
        background: Rgb,
        cursor: Rgb,
        selection: Rgb,
    ) -> Self {
        let mut colors = [Rgb::new(0, 0, 0); 256];
        let mut index = 0;
        while index < 256 {
            colors[index] = match index {
                0..16 => named[index],
                16..232 => {
                    let cube = index - 16;
                    Rgb::new(
                        CUBE_LEVELS[cube / 36],
                        CUBE_LEVELS[cube / 6 % 6],
                        CUBE_LEVELS[cube % 6],
                    )
                }
                // 8, 18… 238: less than 256.
                _ => {
                    let grey = (8 + 10 * (index - 232)) as u8;
                    Rgb::new(grey, grey, grey)
                }
            };
            index += 1;
        }
        Self {
            colors,
            foreground,
            background,
            cursor,
            selection,
        }
    }
}
```

**`new`** crea a paleta do xterm coas 16 cores `named`: as outras 240 son as mesmas en todos os terminais ([Tecnoloxías › A paleta](../tecnoloxias.md#37-a-paleta-e-os-estilos-das-celas)). O texto, o fondo, o cursor e, desde a subfase 1.8, a selección van tal cal.

- **O cubo**, da 16 á 231: `cube = index − 16`, e os seus tres díxitos en base 6 son o vermello (`cube / 36`), o verde (`cube / 6 % 6`) e o azul (`cube % 6`), así que a cor `16 + 36 r + 6 g + b` ten as compoñentes `r`, `g` e `b`. Cada díxito escolle un nivel de `CUBE_LEVELS`: 0 e, desde `0x5f`, en pasos de `0x28` (40) ata `0xff`. Por exemplo, a 67 é 16 + 36 × 1 + 6 × 2 + 3: `(0x5f, 0x87, 0xaf)`.
- **Os grises**, da 232 á 255: `8 + 10 i`, de 8 a 238. O negro e o branco xa están no cubo, na 16 e na 231. O comentario lembra por que o `as u8` non corta nada: o maior é 238.
- **Por que `const fn`.** Así `MEIGA` é unha constante, que o compilador calcula mentres compila: as 256 cores van no programa como datos, e crear un terminal ou unha pantalla só as copia. `Screen::new`, que tamén é `const`, só pode usar unha paleta constante ([`screen.rs`](#cratesmeiga-coresrctermscreenrs)). Nunha `const fn` non se pode usar `for`, que chama os métodos do trait `Iterator`: por iso o bucle é un `while` cun índice.
- **`MEIGA`** son as cores de meiga ata que haxa temas (subfase 8.3): as 16 de *Campbell*, o texto e o fondo de meiga, o cursor da cor do texto ([ADR 0013](../adr/0013-da-saida-a-pantalla.md)) e, desde a 1.8, o fondo da selección, `SELECTION` ([ADR 0015](../adr/0015-o-rato-a-seleccion-e-o-portapapeis.md)).
- **`Default`** dá `MEIGA`: así os tipos que levan unha paleta, como `Screen`, poden derivar `Default`.

**Probas.** Dúas:

| Proba | Que comproba |
|---|---|
| `the_palette_has_the_256_colours_of_xterm` | As 16 primeiras cores son as de *Campbell*; as esquinas do cubo (a 16, negra, e a 231, branca) e unha cor de dentro (a 67); o primeiro e o último gris (8 e 238); o texto, o fondo, o cursor e, desde a subfase 1.8, a selección de `MEIGA`; e que `Default` é `MEIGA`. |
| `mixing_goes_from_one_colour_to_the_other` | Con 0 e con 255, as dúas cores; con 85, do negro ao branco, o gris 85, e do branco ao negro, o 170; e con 128, case a metade, redondea ao máis próximo: o verde, de 200 a 100, dá 150 (149,8), e non 149. |

## `crates/meiga-core/src/geometry.rs`

Puntos, tamaños e rectángulos en píxeles: os tipos cos que todos os crates falan de posicións na xanela. Desde a subfase 1.4, tamén o tamaño da grella, en celas ([`GridSize`](#gridsize)), e desde a 1.8, unha cela da grella ([`GridPoint`](#gridpoint)).

```rust
pub struct Point {
    pub x: i32,
    pub y: i32,
}

pub struct Size {
    pub width: u32,
    pub height: u32,
}

pub struct Rect {
    pub left: i32,
    pub top: i32,
    pub right: i32,
    pub bottom: i32,
}
```

- **`Point`**: unha posición, en píxeles desde a esquina superior esquerda. As coordenadas son `i32` porque poden ser negativas: por exemplo, o rato fóra da xanela mentres se arrastra.
- **`Size`**: un tamaño, que nunca é negativo (`u32`). `area()` dá os píxeles como `u64`, porque `u32 × u32` pode non caber nun `u32`, e `is_empty()` di se non ten ningún.
- **`Rect`**: un rectángulo. Como o `RECT` de Win32, vai desde `left` e `top`, incluídos, ata `right` e `bottom`, **excluídos**: o seu ancho é `right - left`, e dous rectángulos que comparten un lado non se tocan. Cando `right <= left` ou `bottom <= top`, está baleiro.
- Os tres derivan `Clone`, `Copy`, `Debug`, `Default`, `PartialEq`, `Eq` e `Hash`, como `Rgb`. `Default` dá o valor cero.

| Función de `Rect` | Que fai |
|---|---|
| `EMPTY` | O rectángulo baleiro que devolven as operacións cando non queda ningún píxel. |
| `new` e `from_size` | Un rectángulo cos seus catro lados, ou o dun tamaño desde (0, 0). |
| `at(origin, size)` | O rectángulo dun tamaño coa esquina superior esquerda en `origin`: por exemplo, unha cela na súa posición. |
| `offset(by)` | O mesmo rectángulo, movido. As sumas quedan no máximo de `i32` en vez de desbordar (`saturating_add`). |
| `spanning(a, b)` | O rectángulo máis pequeno que inclúe os píxeles `a` e `b`, en calquera orde: o que se debuxa ao arrastrar o rato desde `a` ata `b`. |
| `width`, `height`, `size` e `area` | As medidas. Dun rectángulo invertido, 0. |
| `intersection` | Os píxeles que están nos dous. Se non hai ningún, `EMPTY`. |
| `union` | O rectángulo máis pequeno que contén os dous. Un rectángulo baleiro non engade nada. |
| `intersects` e `contains` | Se teñen píxeles en común, e se un está enteiro dentro do outro. Un rectángulo baleiro está dentro de calquera. |
| `contains_point(at)` | Desde a subfase 2.3: se o píxel `at` está no rectángulo (máis abaixo). Un rectángulo baleiro non ten ningún. |
| `border(thickness)` | Os catro lados dun bordo de `thickness` píxeles, por dentro: arriba, abaixo, esquerda e dereita. Non se solapan, e un lado queda baleiro se o rectángulo é demasiado pequeno para el. |

Algúns detalles:

```rust
const fn span(from: i32, to: i32) -> u32 {
    if to > from { (to as i64 - from as i64) as u32 } else { 0 }
}

const fn saturate(value: u32) -> i32 {
    if value > i32::MAX as u32 { i32::MAX } else { value as i32 }
}
```

- `span` calcula un ancho ou un alto. A resta faise en `i64`: a diferenza entre dous `i32` pode non caber nun `i32` (por exemplo, de -2 000 000 000 a 2 000 000 000), pero sempre cabe nun `u32`.
- `saturate` converte un tamaño nunha coordenada, quedando en `i32::MAX` se non cabe, en vez de dar un número negativo.
- `spanning` suma 1 co `saturating_add`, que queda no máximo en vez de desbordar.
- Moitas funcións son `const fn`, que se poden usar en constantes, como `Rect::EMPTY`.

```rust
pub const fn contains_point(self, at: Point) -> bool {
    self.left <= at.x && at.x < self.right && self.top <= at.y && at.y < self.bottom
}
```

**`contains_point`**, desde a subfase 2.3, di se o píxel `at` está no rectángulo, cos bordos de sempre: `left` e `top` inclúense, e `right` e `bottom` non. Así, un píxel nunca está en dous rectángulos que só comparten un lado, coma dúas tabs seguidas da barra.

- Nun rectángulo baleiro ou invertido, as dúas condicións dalgún dos eixes non se poden cumprir á vez: non contén ningún punto, sen comprobalo á parte.
- É `const fn`, coma `contains`, que fai o mesmo cun rectángulo enteiro.
- Úsana a disposición da barra de tabs, para saber se un punto está na barra e se está no × dunha tab ([`part_at`](#o-que-hai-debaixo-dun-punto-part_at)), e a barra do programa, para saber se o punteiro está nela ([Código › meiga › `tabbar.rs`](meiga.md#cratesmeigasrctabbarrs)); e, desde a subfase 2.4, a disposición dos menús, para saber en que entrada está un punto ([`entry_at`](#a-entrada-debaixo-dun-punto-entry_at)), e o menú do programa, para saber se un botón baixa dentro del ([Código › meiga › `menu.rs`](meiga.md#cratesmeigasrcmenurs)).

**Probas.** Dez, unha por propiedade: que `right` e `bottom` non se inclúen, que os rectángulos invertidos están baleiros, `from_size` (tamén co tamaño máximo), `spanning` nas dúas ordes, a intersección e a unión (tamén con rectángulos baleiros), `contains`, desde a subfase 2.3 `contains_point` (os píxeles das esquinas de dentro si, os da columna `right`, da fila `bottom` e da columna anterior a `left` non, e un rectángulo baleiro non contén ningún), que `at` e `offset` moven o rectángulo (tamén preto do máximo de `i32`), e que os lados de `border` cobren o bordo sen solaparse nin saír dun rectángulo estreito.

### `GridSize`

```rust
/// A size of the terminal grid, in cells: how many columns and rows of
/// characters fit in it.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct GridSize {
    pub columns: u16,
    pub rows: u16,
}

impl GridSize {
    pub const fn new(columns: u16, rows: u16) -> Self {
        Self { columns, rows }
    }
}
```

Desde a subfase 1.4: o tamaño da grella do terminal, en celas, non en píxeles. É o tamaño que recibe a pseudoconsola ao creala e, desde a 1.9, cando cambia ([Código › meiga-win](meiga-win.md#cratesmeiga-winsrcconptyrs)), e, desde a 1.5, o do núcleo do terminal. Desde a 1.6, sae do tamaño da xanela ao arrancar, dividido polo da cela, e desde a 1.9 segue a xanela, sen a franxa da barra de desprazamento, cando cambia o seu tamaño ou o das celas, coa escala ou co zoom ([ADR 0016](../adr/0016-o-historial-o-tamano-e-o-zoom.md)).

- `u16`: ningunha grella chega a 65 535 columnas ou filas. Cada uso comproba o seu propio límite: o ConPTY, por exemplo, recibe as medidas nun `COORD`, de 16 bits con signo, así que acepta como moito 32 767, e polo menos 1.
- Deriva o mesmo ca os outros tipos do módulo, e os campos son públicos: calquera par de números é un tamaño, e quen o usa decide se lle serve.

### `GridPoint`

```rust
/// A cell of the terminal grid: its column and its row, from the top-left
/// cell, which is (0, 0).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct GridPoint {
    pub column: u16,
    pub row: u16,
}

impl GridPoint {
    pub const fn new(column: u16, row: u16) -> Self {
        Self { column, row }
    }
}
```

Desde a subfase 1.8: unha cela da grella do terminal, en celas e non en píxeles, coma `GridSize`. É a cela debaixo do punteiro do rato ([`cell_at`](#cratesmeiga-coresrcmousers)), a que levan os informes para os programas que piden o rato, e onde empeza e remata unha selección ([`select`](#cratesmeiga-coresrctermselectionrs)).

- A columna e a fila contan desde 0, na esquina superior esquerda. Os informes do rato contan desde 1: a conversión faise ao escribilos.
- A columna vai primeiro, coma o `x` de `Point` e as columnas de `GridSize`.
- `u16`, coma `GridSize`, e deriva o mesmo. Calquera par de números é unha cela: os métodos da selección aceptan tamén as de fóra da grella, e usan a máis próxima.

## `crates/meiga-core/src/damage.rs`

As zonas da xanela que cambiaron desde o último fotograma, o ***damage***. Cada fotograma compón e presenta só estas zonas, como manda o principio de debuxar só o que cambia ([ADR 0002](../adr/0002-windows-e-renderizado-cpu.md)).

```rust
pub const MAX_RECTS: usize = 8;
pub const MERGE_SLACK: u64 = 64 * 64;

pub struct Damage {
    rects: [Rect; MAX_RECTS],
    len: usize,
}
```

- Garda como moito `MAX_RECTS` rectángulos, nun *array* de tamaño fixo: engadir un nunca reserva memoria, como manda o principio de non reservar memoria no camiño quente.
- Cada rectángulo cópiase na pantalla por separado, e cada copia ten un custo fixo, ademais do dos seus píxeles. Por iso dous rectángulos próximos convén xuntalos nun só, aínda que inclúa algúns píxeles que non cambiaron. **`MERGE_SLACK`** é ese custo fixo, contado en píxeles: dous rectángulos xúntanse se o rectángulo que os contén ten como moito 64 × 64 píxeles máis ca os dous xuntos. O valor é unha estimación, para axustar con medicións.

```rust
pub fn add(&mut self, rect: Rect) {
    if rect.is_empty() {
        return;
    }
    let mut rect = rect;
    while let Some((index, extra)) = self.cheapest_merge(rect) {
        if extra > i128::from(MERGE_SLACK) && self.len < MAX_RECTS {
            break;
        }
        rect = rect.union(self.rects[index]);
        self.len -= 1;
        self.rects[index] = self.rects[self.len];
    }
    self.rects[self.len] = rect;
    self.len += 1;
}
```

O algoritmo para engadir un rectángulo:

1. Busca, entre os que xa garda, o que menos crece ao xuntalo co novo (`cheapest_merge`).
2. Se ese crecemento é como moito `MERGE_SLACK`, ou se xa non queda sitio, xúntaos: o novo pasa a ser a unión, e o vello sae da lista. Para sacalo sen mover os demais, o último ocupa o seu sitio.
3. Repite co rectángulo xuntado, que agora é maior e pode tocar outros.
4. Cando xa non convén xuntar máis, gárdao.

- **A garantía:** cada rectángulo engadido acaba **dentro** dun dos que se gardan. Un rectángulo nunca se corta, só se xunta con outros: o que cambiou presentarase sempre.
- `while let` repite mentres o patrón encaixe: mentres haxa algún rectángulo co que comparar.
- `extra_pixels` calcula os píxeles de máis: a área da unión menos as dos dous. É negativo cando se solapan, porque os píxeles comúns se copiarían dúas veces; por iso dous rectángulos que se solapan sempre se xuntan. A conta faise en `i128`, o tipo que cabe con calquera par de áreas de `u64`.
- `rects()` dá os rectángulos gardados (`&self.rects[..self.len]`), e `clear()` esquéceos despois de debuxalos.

**Probas.** Sete casos e unha proba aleatoria:

| Proba | Que comproba |
|---|---|
| `ignores_empty_rectangles` | Os rectángulos baleiros non se gardan. |
| `a_rectangle_inside_another_adds_nothing` | Un rectángulo dentro doutro, en calquera orde, deixa só o grande. |
| `rows_one_after_another_become_one_rectangle` | Tres filas seguidas, aínda que cheguen desordenadas, son un só rectángulo. |
| `distant_changes_stay_apart` | O cursor nunha esquina e un *spinner* na outra quedan separados: xuntalos copiaría media pantalla. |
| `close_changes_are_merged` e `overlapping_changes_are_merged` | Dous rectángulos próximos ou solapados xúntanse. |
| `clear_forgets_every_change` | Despois de `clear`, non queda nada. |
| `every_change_stays_covered_within_the_capacity` | 200 roldas de ata 40 rectángulos pseudoaleatorios: nunca hai máis de `MAX_RECTS`, ningún gardado está baleiro, e cada rectángulo engadido está dentro dun gardado. A secuencia é fixa (un xerador *xorshift* coa mesma semente), así que un fallo pódese repetir. |

## `crates/meiga-core/src/pacing.rs`

**Cando** debuxar o seguinte fotograma: como moito un por intervalo, e o primeiro cambio despois dunha pausa, no momento ([ADR 0005](../adr/0005-fios-e-fotogramas.md)). Desde a subfase 2.3, conta tamén un fotograma que non atopa nada que debuxar ([ADR 0020](../adr/0020-a-barra-de-tabs.md), decisión 5).

```rust
pub const fn auto_interval(remote: bool) -> Duration {
    let fps = if remote { 30 } else { 60 };
    Duration::from_nanos(1_000_000_000 / fps)
}
```

O intervalo de `max_fps = "auto"`: 30 fotogramas por segundo (33,3 ms) nas sesións de Escritorio remoto e 60 (16,7 ms) en local. Decidiuse coas medicións da subfase 1.10 ([ADR 0017](../adr/0017-os-fotogramas-por-escritorio-remoto.md)): por Escritorio remoto, 30 aínda que a sesión permita 60. `Duration` é un tempo en segundos e nanosegundos, da biblioteca estándar.

```rust
pub enum Pace {
    Now,
    After(Duration),
    Scheduled,
}

pub struct FramePacer {
    last_frame: Option<Instant>,
    due: Option<Instant>,
}
```

- **`Pace`**, a resposta: debuxar agora (`Now`), despois dun tempo (`After`, co tempo que falta: hai que pór un temporizador) ou nada, porque xa hai un fotograma programado que mostrará tamén este cambio (`Scheduled`).
- **`FramePacer`** lembra cando foi o último fotograma (`last_frame`) e cando vence o programado, se o hai (`due`).

```rust
pub fn request(&mut self, now: Instant, interval: Duration) -> Pace {
    if let Some(due) = self.due {
        return if now >= due { Pace::Now } else { Pace::Scheduled };
    }
    let Some(due) = self.last_frame.and_then(|last| last.checked_add(interval)) else {
        return Pace::Now;
    };
    if now >= due {
        return Pace::Now;
    }
    self.due = Some(due);
    Pace::After(due - now)
}
```

Algo cambiou no instante `now`: cando se debuxa?

1. Se xa hai un fotograma programado, normalmente a resposta é `Scheduled`. Pero se xa pasou a súa hora (o temporizador vai tarde, por exemplo porque chegaban mensaxes sen parar e nunca lle tocou), `Now`: non se espera por el.
2. Se nunca houbo fotograma, `Now`.
3. Se xa pasou o intervalo desde o último, `Now`: a primeira tecla despois dunha pausa vese sen atraso.
4. Se non, prográmase para cando remate o intervalo: `After` co tempo que falta.

- `now` vén de fóra, en vez de ler o reloxo dentro: así as probas usan instantes inventados e dan sempre o mesmo resultado.
- `is_scheduled()` di se hai un fotograma programado, `cancel()` esquéceo (por exemplo, ao minimizar a xanela) e `updated(now)` anota que un fotograma puxo ao día a escena en `now`, debuxase algo ou non: o seguinte agarda o intervalo. Ata a 2.3 chamábase `drawn`, e o programa só o chamaba se o fotograma debuxaba algo. Un fotograma sen nada que debuxar cancelaba o programado, e o seguinte cambio voltaba ser `Now`: a saída que non cambia nada na pantalla, coma o mesmo título unha e outra vez, espertaba a interface con cada bloque, e non como moito unha vez por intervalo. Atopouno a revisión do código da 2.3 ([Código › meiga › `frames.rs`](meiga.md#cratesmeigasrcframesrs)).

**Probas.** Seis: o primeiro cambio debúxase no momento; un cambio dentro do intervalo espera o que falta (23 ms dun intervalo de 33 se pasaron 10), e os seguintes xa están programados; despois do intervalo, no momento; un temporizador que vai tarde non atrasa o fotograma; `updated` e `cancel` rematan a espera (`updating_and_cancelling_end_the_wait`, que ata a 2.3 se chamaba `drawing_and_cancelling_end_the_wait`); e `auto` dá 33 333 333 ns en remoto e 16 666 666 ns en local.

## `crates/meiga-core/src/text.rs`

O texto como celas dunha grella: que caracteres ocupan unha cela, cales dúas e cales ningunha ([Tecnoloxías › O ancho dos caracteres](../tecnoloxias.md#29-o-ancho-dos-caracteres-unicode-width)). Desde a subfase 2.3, tamén como cortar un texto, con «…», para que caiba nunhas columnas: os títulos das tabs.

```rust
pub enum CellChar {
    Blank,
    Narrow(char),
    WideLeft(char),
    WideRight(char),
}
```

O que mostra unha cela:

| Variante | Que é |
|---|---|
| `Blank` | Só o fondo, como un espazo. |
| `Narrow(c)` | Un carácter que ocupa esta cela. |
| `WideLeft(c)` | A metade esquerda dun carácter que ocupa esta cela e a seguinte. |
| `WideRight(c)` | A metade dereita dun carácter que ocupa a cela anterior e esta. |

- **Por que dúas metades.** Así cada cela se pode debuxar soa: cando só cambia unha das dúas, ou cando só unha está dentro da zona que cambiou, só se debuxa esa. Na caché de glifos, cada metade ten o seu *slot* ([Código › meiga-render](meiga-render.md#glyphcache)).
- Deriva `Clone`, `Copy`, `Debug`, `Default`, `PartialEq`, `Eq` e `Hash`, como `Rgb`. O valor por defecto, desde a subfase 1.5, é `Blank` (`#[default]`): o dunha cela baleira do terminal ([`Cell`](#cratesmeiga-coresrctermscreenrs)).

```rust
pub fn cells(text: &str) -> impl Iterator<Item = CellChar> + '_ {
    text.chars().flat_map(|c| {
        let (first, second) = match c.width() {
            Some(1) if c == ' ' => (Some(CellChar::Blank), None),
            Some(1) => (Some(CellChar::Narrow(c)), None),
            Some(2) => (Some(CellChar::WideLeft(c)), Some(CellChar::WideRight(c))),
            _ => (None, None),
        };
        first.into_iter().chain(second)
    })
}

pub fn width(text: &str) -> usize {
    cells(text).count()
}
```

- **`cells`** dá as celas dun texto, unha tras outra. O ancho de cada carácter é o que dá `unicode-width` (`c.width()`, do seu trait `UnicodeWidthChar`), o mesmo que usa `alacritty_terminal` para a grella do terminal:
  - o espazo é unha cela en branco;
  - un carácter de ancho 1, unha cela;
  - un de ancho 2, as súas dúas metades;
  - os de control (`None`) e os que non teñen ancho (`Some(0)`), como os acentos que se combinan coa letra anterior, ningunha. Xuntalos co carácter anterior, nun grafema, é traballo da subfase 8.1. As letras con acento xa compostas, como `á`, son un só carácter.
- `flat_map` converte cada carácter en cero, unha ou dúas celas. `first.into_iter().chain(second)` encadea dous `Option`, que se percorren como listas de cero ou un elementos, sen reservar memoria: quen chama decide onde gardar as celas, por exemplo nun `Vec` con `collect`.
- `impl Iterator<Item = CellChar> + '_`: devolve un iterador sen dicir o seu tipo exacto, que toma prestado o texto (`'_`).
- **`width`** conta as celas que ocupa un texto. Desde a subfase 2.4, a disposición dos menús mide con ela os textos dos seus elementos ([`Layout::new`](#facer-o-menú-layoutnew)). Desde a 3.2, o campo de texto mostra o seu texto con `cells`, e mide con `width` onde van o cursor, a selección e cada grafema ([`ui/field/mod.rs`](#cratesmeiga-coresrcuifieldmodrs)).

```rust
pub fn fit(text: &str, columns: usize, out: &mut Vec<CellChar>) {
    out.clear();
    out.extend(cells(text).take(columns.saturating_add(1)));
    if out.len() <= columns {
        return;
    }
    out.truncate(columns.saturating_sub(1));
    if matches!(out.last(), Some(CellChar::WideLeft(_))) {
        out.pop();
    }
    if columns > 0 {
        out.push(CellChar::Narrow('…'));
    }
}
```

**`fit`**, desde a subfase 2.3, pon en `out` as celas de `text` que caben en `columns` columnas: todas, se caben, e se non, as primeiras e unha elipse (`…`) na última columna ([ADR 0020](../adr/0020-a-barra-de-tabs.md), decisión 4). A barra de tabs corta así o título de cada tab, coas columnas que caben no seu sitio ([Código › meiga › `tabbar.rs`](meiga.md#cratesmeigasrctabbarrs)), e, desde a 2.4, o menú do programa, os textos dos seus elementos, que só se cortan se a xanela é máis estreita ca o menú ([Código › meiga › `menu.rs`](meiga.md#cratesmeigasrcmenurs)); e, desde a 3.2, o exemplo `field` de `meiga-win`, a liña de debaixo do campo ([Código › meiga-win › `examples/field.rs`](meiga-win.md#cratesmeiga-winexamplesfieldrs)). Paso a paso:

1. Baleira `out` e colle as celas do texto, como moito unha máis das que caben: abonda para saber se o texto cabe, e `take` para o iterador alí, sen percorrer o resto dun título longo.
2. Se caben todas, xa está.
3. Se non, deixa as `columns - 1` primeiras: a última columna é para a elipse.
4. Se a última que queda é a metade esquerda dun carácter ancho, porque a súa metade dereita quedou fóra, quítaa tamén: medio carácter non se pode debuxar. Entón o texto ocupa unha columna menos das que hai.
5. Pon a elipse, un carácter dunha cela, se hai polo menos unha columna. Sen columnas, non queda nada.

- Por exemplo: `meiga-tabs` en 5 columnas é `meig…`; `ab` nunha columna, `…`, e en ningunha, nada; e `a中文` en 4 columnas, `a中…`, e en 3, `a…`, porque `中` xa non cabe enteiro antes da elipse.
- **Por que contar celas.** A barra escribe os títulos coa fonte do terminal, que ten todos os caracteres do mesmo ancho: cortar un título é contar celas, sen medilo con DirectWrite ([ADR 0020 › Alternativas consideradas](../adr/0020-a-barra-de-tabs.md#alternativas-consideradas)).
- **Sen reservar memoria.** Quen chama garda `out` entre un texto e outro, coma a barra, que usa o mesmo `Vec` para todos os títulos: `clear` baleirao sen soltar a súa memoria, e só se reserva máis cando un título ocupa máis celas ca os anteriores.
- `saturating_add` e `saturating_sub` evitan desbordar nos extremos: con `usize::MAX` columnas, e sen ningunha. `matches!` compara a última cela co patrón sen movela.

**Probas.** Cinco: as letras, `ñ` e `€` ocupan unha cela, e o espazo é unha cela en branco; `中` e `😀` ocupan dúas cada un, e `日本語 abc`, dez; os caracteres de control (o tabulador) e os que non teñen ancho (un acento que se combina e o espazo de ancho cero) non ocupan ningunha; e, desde a subfase 2.3, dúas de `fit`. Nelas, `fitted` empeza cun `out` que xa ten unha cela, para comprobar tamén que `fit` o baleira:

| Proba | Que comproba |
|---|---|
| `text_that_fits_keeps_all_its_cells` | `pwsh` en 4 columnas e en 10 queda enteiro, e un texto baleiro non ten ningunha cela. |
| `longer_text_ends_with_an_ellipsis_in_its_last_cell` | `meiga-tabs` en 5 columnas é `meig…`; `ab` en 1, `…`, e en 0, nada; e un carácter ancho que perdería a súa metade dereita queda fóra: `a中文` en 4 columnas é `a中…`, e en 3, `a…`. |

## `crates/meiga-core/src/shell.rs`

Desde a subfase 1.6: a liña de comandos dun programa. `meiga.exe` recibe o programa que ten que lanzar e os seus argumentos xa divididos (`meiga [opcións] [--] [programa [argumentos]]`), pero a sesión lánzao con `CreateProcessW`, que recibe un só texto: hai que xuntalos outra vez, de forma que o programa os divida igual ([Tecnoloxías › A liña de comandos](../tecnoloxias.md#40-a-liña-de-comandos-en-windows), [ADR 0013](../adr/0013-da-saida-a-pantalla.md)). Úsao `Options::parse` ([Código › meiga › `options.rs`](meiga.md#cratesmeigasrcoptionsrs)) e, desde a subfase 2.5, `profiles::detect`, que fai así as liñas de comandos dos perfís que atopa, coas rutas completas dos seus programas ([Código › meiga-win › `profiles.rs`](meiga-win.md#cratesmeiga-winsrcprofilesrs)).

Está en `meiga-core`, e non nun crate de Windows, aínda que as regras son as de Windows: só traballa con texto, sen chamar a ningunha API, así que se proba sen lanzar ningún programa. O módulo é para as shells e as súas liñas de comandos; desde a subfase 2.3 ten dúas funcións: `command_line` e `program_name`, o nome do programa dunha liña de comandos.

```rust
pub fn command_line<S: AsRef<str>>(program: &str, args: &[S]) -> Option<String> {
    if program.contains('"') {
        return None;
    }
    let mut line = String::with_capacity(
        program.len() + args.iter().map(|arg| arg.as_ref().len() + 3).sum::<usize>() + 2,
    );
    if needs_quotes(program) || program.is_empty() {
        line.push('"');
        line.push_str(program);
        line.push('"');
    } else {
        line.push_str(program);
    }
    for arg in args {
        line.push(' ');
        push_argument(&mut line, arg.as_ref());
    }
    Some(line)
}

fn needs_quotes(arg: &str) -> bool {
    arg.contains([' ', '\t'])
}
```

**`command_line`** dá a liña de comandos de `program` con `args`, separados por espazos, ou `None` se o nome do programa ten comiñas.

- **O programa** non se le coma os argumentos. `CreateProcessW` busca o programa ata o primeiro espazo ou tabulador ou, se empeza por comiñas, ata as seguintes, sen ningunha barra invertida especial; e o *runtime* de C do programa le igual o seu primeiro argumento. Por iso o nome vai tal cal, sen escapar nada, e entre comiñas só se ten espazos ou tabuladores: `C:\Program Files\PowerShell\7\pwsh.exe` vai como `"C:\Program Files\PowerShell\7\pwsh.exe"`.
- **Por que `None` coas comiñas.** Un nome con comiñas non se pode escribir de forma que se lea igual: unhas comiñas no medio rematarían o nome antes de tempo, e `\"` non as escapa. Ademais, en Windows ningún arquivo pode ter comiñas no nome, así que ese programa non existe. Con `None`, quen chama dio claramente, en vez de arriscarse a lanzar outro programa: `Options::parse` convérteo no erro `The name of a program cannot have quotes`.
- **Un programa baleiro** vai entre comiñas, `""`: sen elas, a liña empezaría polo espazo de antes do primeiro argumento, e ese argumento pasaría a ser o programa. Con elas, o programa é un nome baleiro, que non se pode lanzar, e o terminal mostra o erro.
- `S: AsRef<str>`: os argumentos poden ser `String`, como os ten `Options`, ou `&str`, como nas probas, sen copialos.
- `with_capacity` reserva dunha vez o habitual: cada argumento cun espazo e dúas comiñas (3 bytes máis), e o programa coas súas (2). Só as barras e as comiñas que hai que escapar fan medrar o texto despois.
- `contains([' ', '\t'])`: `contains` acepta como patrón un *array* de caracteres, e busca calquera deles.

```rust
fn push_argument(line: &mut String, arg: &str) {
    let quote = arg.is_empty() || needs_quotes(arg);
    if quote {
        line.push('"');
    }
    let mut backslashes = 0;
    for c in arg.chars() {
        match c {
            '\\' => backslashes += 1,
            '"' => {
                // The backslashes before it, and the quote, are escaped.
                line.extend(std::iter::repeat_n('\\', backslashes + 1));
                backslashes = 0;
            }
            _ => backslashes = 0,
        }
        line.push(c);
    }
    if quote {
        // The closing quote must not be escaped by the backslashes.
        line.extend(std::iter::repeat_n('\\', backslashes));
        line.push('"');
    }
}
```

**Os argumentos** escríbense coas regras do *runtime* de C de Microsoft, coas que case todos os programas dividen a súa liña de comandos, e que tamén segue `CommandLineToArgvW`. Nelas, os espazos e os tabuladores separan os argumentos, salvo entre comiñas, e unha barra invertida só é especial xusto antes dunhas comiñas: `2n` barras e unhas comiñas son `n` barras e o principio ou o fin dun texto entre comiñas; `2n + 1` barras e unhas comiñas son `n` barras e unhas comiñas dentro do argumento. Así que `push_argument` escribe:

| Argumento | Na liña de comandos | Por que |
|---|---|---|
| `-NoProfile` | `-NoProfile` | Sen espazos, tabuladores nin comiñas, vai tal cal. |
| `C:\my scripts\a b.ps1` | `"C:\my scripts\a b.ps1"` | Os espazos separarían os argumentos: vai entre comiñas. As barras que non van antes dunhas comiñas quedan como están. |
| Un argumento baleiro | `""` | Sen as comiñas, non quedaría nada, e o argumento desaparecería. |
| `say "ola"` | `"say \"ola\""` | Unhas comiñas dentro do argumento escríbense `\"`. |
| `a\"b` | `a\\\"b` | As barras xusto antes dunhas comiñas dóbranse, e despois vai `\"`: tres barras e unhas comiñas son unha barra e unhas comiñas. |
| `a\\b` | `a\\b` | Sen comiñas despois, as barras quedan como están. |
| `C:\a b\` | `"C:\a b\\"` | Ao final dun argumento entre comiñas, as barras dóbranse: se non, a última escaparía as comiñas que o pechan, e o argumento seguiría. |
| `C:\a\` | `C:\a\` | Sen comiñas ao final, a barra queda como está. |

- `backslashes` conta as barras seguidas que acaba de copiar. Se despois veñen unhas comiñas, engade outras tantas e unha máis, a que escapa as comiñas; con calquera outro carácter, volta a 0. Ao final dun argumento entre comiñas, engade outras tantas antes das comiñas que o pechan.
- As comiñas escápanse tamén nun argumento sen espazos, que vai sen comiñas: `a"b` vai como `a\"b`.
- `std::iter::repeat_n('\\', n)` dá o mesmo carácter `n` veces, e `extend` engádeos todos ao texto.
- **`cmd.exe` é a excepción**: le o resto da súa liña coas súas propias regras, e non coas do *runtime* de C ([Tecnoloxías › A liña de comandos](../tecnoloxias.md#40-a-liña-de-comandos-en-windows)).

```rust
pub fn program_name(command_line: &str) -> &str {
    let line = command_line.trim_start_matches([' ', '\t']);
    let program = match line.strip_prefix('"') {
        Some(quoted) => quoted
            .split_once('"')
            .map_or(quoted, |(program, _)| program),
        None => line
            .split_once([' ', '\t'])
            .map_or(line, |(program, _)| program),
    };
    let file = program
        .rsplit_once(['\\', '/'])
        .map_or(program, |(_, file)| file);
    match file.len().checked_sub(4) {
        Some(stem) if file.is_char_boundary(stem) && file[stem..].eq_ignore_ascii_case(".exe") => {
            &file[..stem]
        }
        _ => file,
    }
}
```

**`program_name`**, desde a subfase 2.3, dá o nome do programa dunha liña de comandos: o que mostra unha tab mentres o seu programa non lle dá un título, coma `pwsh` ou `cmd` ([ADR 0020](../adr/0020-a-barra-de-tabs.md), decisión 4). Ata a 2.4, cada tab collíao ao abrirse, da liña de comandos que lanzou a súa sesión, ou que intentou lanzar por última vez, que lle devolvía `session::start`: a do programa que se lle deu a `meiga.exe`, na primeira tab, ou a da shell por defecto. Desde a 2.5, as tabs dos perfís mostran o nome do seu perfil, coma «PowerShell» ou «Ubuntu», e `program_name` só dá o do programa da liña de comandos de meiga, na primeira tab, a través de [`Launch::command_line`](#como-empeza-launch-e-command_line) ([ADR 0022](../adr/0022-os-perfis-e-a-configuracion.md), decisión 11; [Código › meiga › `tabs.rs`](meiga.md#cratesmeigasrctabsrs)). Paso a paso:

1. Salta os espazos e os tabuladores do principio.
2. Colle o primeiro argumento coma o le `CreateProcessW`, o mesmo que escribe `command_line`: se empeza por comiñas, ata as seguintes, ou ata o final se non as hai; se non, ata o primeiro espazo ou tabulador. Sen ningunha barra invertida especial.
3. Del, o nome do arquivo: o que vai despois da última barra, `\` ou `/`, sen o cartafol.
4. Sen `.exe`, en maiúsculas ou en minúsculas (`eq_ignore_ascii_case`): `wsl.EXE` é `wsl`. As outras extensións quedan: `build.cmd` segue sendo `build.cmd`.

- Devolve un anaco do mesmo texto (`&str`), sen copialo: a tab cópiao unha vez, con `to_owned`.
- `strip_prefix` dá o texto sen as comiñas do principio, se as ten. `split_once` e `rsplit_once` parten o texto na primeira ou na última aparición dun patrón, e `map_or` dá o texto enteiro se non aparece. Coma en `command_line`, un *array* de caracteres é un patrón que busca calquera deles.
- **`checked_sub` e `is_char_boundary`.** `.exe` son catro bytes, e `checked_sub` dá `None` cos nomes máis curtos, que non o poden levar. `file[stem..]` corta o texto nun byte, e Rust para o programa (*panic*) se ese byte está no medio dun carácter de varios: en `中文`, de seis bytes, o corte caería no medio de `中`. A garda comproba primeiro que o byte empeza un carácter; se non, o nome non remata en `.exe`, e queda enteiro.

**Probas.** Cinco:

| Proba | Que comproba |
|---|---|
| `simple_arguments_stay_as_they_are` | `pwsh` só, e `pwsh -NoProfile -File C:\perf\spinner.ps1`, coas barras como están. |
| `spaces_tabs_and_empty_arguments_go_in_quotes` | O programa `C:\Program Files\PowerShell\7\pwsh.exe` vai entre comiñas, e tamén un argumento con espazos, un baleiro e un cun tabulador. |
| `quotes_and_the_backslashes_before_them_are_escaped` | As comiñas dentro dun argumento, as barras antes delas, as que non van antes de ningunhas, e as do final dun argumento con e sen comiñas: os exemplos da táboa. |
| `a_program_cannot_have_quotes` | Un programa con comiñas dá `None`, e un baleiro, `""`. |
| `the_name_of_a_program_is_its_file_without_the_folder_and_exe` | Desde a subfase 2.3: `pwsh` é `pwsh`; `cmd /c echo ola`, `cmd`; `  wsl.EXE\t-e bash`, `wsl`, sen os brancos do principio; `"C:\Program Files\PowerShell\7\pwsh.exe" -NoProfile`, `pwsh`; `C:\tools\build.cmd x`, `build.cmd`; e `tools/x.exe`, `x`. Cunhas comiñas que non se pechan, `"a b`, o nome chega ata o final, `a b`; unha liña baleira dá un nome baleiro; e `ñ.exe`, `ñ`. |

## `crates/meiga-core/src/input.rs`

Desde a subfase 1.7: as teclas que meiga lle envía ao programa ([ADR 0014](../adr/0014-o-teclado.md)). Windows describe cada tecla que baixa ou sobe cun evento, e o teclado da xanela ([Código › meiga-win › `keyboard.rs`](meiga-win.md#cratesmeiga-winsrckeyboardrs)) dálle a meiga eses eventos. Este módulo convérteos nos bytes que le o programa: o evento enteiro en **win32-input-mode**, se o ConPTY o activou, ou o que enviaría un terminal VT ([Tecnoloxías › As teclas no terminal](../tecnoloxias.md#42-as-teclas-no-terminal-win32-input-mode-e-vt)). Está en `meiga-core` porque só traballa con números e bytes: as súas probas non necesitan ningún teclado.

### `ControlKeys`

```rust
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct ControlKeys(u16);

impl ControlKeys {
    pub const RIGHT_ALT: Self = Self(0x0001);
    pub const LEFT_ALT: Self = Self(0x0002);
    pub const RIGHT_CTRL: Self = Self(0x0004);
    pub const LEFT_CTRL: Self = Self(0x0008);
    pub const SHIFT: Self = Self(0x0010);
    pub const NUM_LOCK: Self = Self(0x0020);
    pub const SCROLL_LOCK: Self = Self(0x0040);
    pub const CAPS_LOCK: Self = Self(0x0080);
    pub const ENHANCED: Self = Self(0x0100);

    pub const fn empty() -> Self {
        Self(0)
    }

    pub const fn bits(self) -> u16 {
        self.0
    }

    pub const fn contains(self, other: Self) -> bool {
        self.0 & other.0 == other.0
    }

    pub const fn alt(self) -> bool {
        self.0 & (Self::LEFT_ALT.0 | Self::RIGHT_ALT.0) != 0
    }

    pub const fn ctrl(self) -> bool {
        self.0 & (Self::LEFT_CTRL.0 | Self::RIGHT_CTRL.0) != 0
    }

    pub const fn shift(self) -> bool {
        self.contains(Self::SHIFT)
    }

    pub const fn alt_gr(self) -> bool {
        let all = Self::LEFT_ALT.0 | Self::RIGHT_ALT.0 | Self::LEFT_CTRL.0 | Self::RIGHT_CTRL.0;
        self.0 & all == Self::LEFT_CTRL.0 | Self::RIGHT_ALT.0
    }
}

impl BitOr for ControlKeys {
    type Output = Self;

    fn bitor(self, other: Self) -> Self {
        Self(self.0 | other.0)
    }
}

impl BitOrAssign for ControlKeys {
    fn bitor_assign(&mut self, other: Self) {
        self.0 |= other.0;
    }
}
```

- As teclas modificadoras e as de bloqueo dun evento, cos bits que usa Windows nos eventos da consola (`dwControlKeyState`). Gardar os mesmos bits fai que o win32-input-mode os envíe tal cal, con `bits`.
- `ENHANCED` marca unha **tecla estendida**, das que non tiñan os primeiros teclados do PC: as frechas, Insert, Supr, Inicio, Fin, RePáx e AvPáx de fóra do teclado numérico, o Ctrl e o Alt dereitos, e o Intro e a `/` do teclado numérico ([Tecnoloxías › O teclado en Windows](../tecnoloxias.md#41-o-teclado-en-windows)).
- `alt` e `ctrl` son certos con calquera dos dous, o esquerdo ou o dereito; `contains`, se están todas as de outro valor. `BitOr` e `BitOrAssign` permiten xuntalas con `|` e `|=`.
- **`alt_gr`** di se o Ctrl e o Alt premidos son só os de AltGr: Windows dá AltGr coma o Ctrl esquerdo e o Alt dereito. Se ademais está o Ctrl dereito ou o Alt esquerdo, hai un Ctrl ou un Alt de verdade. Úsaa [`push_vt_character`](#keyencoder) e, desde a subfase 2.6, [`Chord::of`](#a-combinación-dunha-tecla-chordof), para que AltGr non dispare ningún atallo ([ADR 0023](../adr/0023-os-atallos-de-teclado.md), decisión 10). Ata a 2.5 era privada; desde a 2.6 é pública, coma as outras. `check-shortcuts.ps1` comproba na CI que AltGr+2 chega ao programa, mentres Ctrl+Alt+2, co Alt esquerdo, é un atallo do seu `config.toml` ([Código › ferramentas › `check-shortcuts.ps1`](ferramentas.md#toolscheck-shortcutsps1)).
- É un *newtype* sobre `u16`: o número non se confunde con outro, e os bits só se nomean coas constantes. As funcións son `const`, así que tamén se poden usar para definir outras constantes, coma nas probas.

### `KeyEvent`

```rust
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct KeyEvent {
    pub key: u16,
    pub scan: u16,
    pub character: u16,
    pub down: bool,
    pub controls: ControlKeys,
    pub repeat: u16,
}
```

Unha tecla que baixou ou subiu, coma a describe Windows:

| Campo | Que é |
|---|---|
| `key` | O código de tecla virtual (`VK_…`): que tecla é na distribución do teclado. 0 para un carácter sen tecla, coma os dun IME. |
| `scan` | O código de exploración: onde está a tecla no teclado. |
| `character` | O carácter que escribe, unha unidade UTF-16, ou 0. Un carácter de fóra do plano básico, coma a maioría dos emoji, chega en dous eventos, cada un cunha metade (*surrogate*). |
| `down` | Se baixou. Cada tecla que baixa sobe despois, noutro evento. |
| `controls` | As teclas modificadoras e de bloqueo. |
| `repeat` | Cantas veces baixou: máis dunha se o teclado a repetiu máis rápido do que a xanela a colleu. |

É `Copy`, de 12 bytes: vai por valor no evento da xanela, `Event::Key` ([Código › meiga-win › `event.rs`](meiga-win.md#cratesmeiga-winsrceventrs)). Desde a subfase 1.8, a roda do rato tamén crea eventos de tecla: as frechas, na pantalla alternativa ([`arrow_keys`](#cratesmeiga-coresrcmousers)). E desde a 2.6, cada tecla que baixa ten a súa combinación, [`Chord::of`](#a-combinación-dunha-tecla-chordof), coa que a vista busca o seu atallo. E desde a 3.2, o campo de texto recibe os eventos enteiros, tamén os das teclas que soben ([`Field::key`](#key-a-entrada-das-teclas)).

### `KeyEncoder`

```rust
#[derive(Debug, Default)]
pub struct KeyEncoder {
    high_surrogate: Option<u16>,
}

impl KeyEncoder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn encode(&mut self, event: &KeyEvent, modes: &Modes, out: &mut Vec<u8>) {
        if modes.win32_input {
            // The ConPTY joins the two halves of a character by itself.
            self.high_surrogate = None;
            push_win32(event, out);
            return;
        }
        // A VT terminal only sends something when a key goes down, and the
        // character that Alt brings when it goes up: the one of the code
        // typed with it on the numeric keypad. Only with Num Lock on, as
        // Windows Terminal.
        if !event.down {
            if event.key == vk::MENU
                && event.controls.contains(ControlKeys::NUM_LOCK)
                && let Some(character) = char::from_u32(u32::from(event.character))
                && character != '\0'
            {
                push_utf8(out, character);
            }
            return;
        }
        let start = out.len();
        self.push_vt(event, modes, out);
        let end = out.len();
        for _ in 1..event.repeat {
            out.extend_from_within(start..end);
        }
    }
}
```

**`encode`** engade a `out` o que le o programa para `event`, cos modos que activou ([`Modes`](#changes-e-modes)). Quen chama reutiliza o mesmo `Vec`, así que codificar unha tecla non reserva memoria.

- **Co win32-input-mode,** o evento vai enteiro, tamén ao subir e tamén as teclas modificadoras soas ([`push_win32`](#os-axudantes-das-secuencias)). As dúas metades dun carácter van en dous eventos, e o ConPTY xúntaas.
- **Sen el,** só envían algo as teclas que baixan, coma nun terminal VT. Se o teclado repetiu a tecla, `extend_from_within` copia o que escribiu a primeira vez as veces que falten, sen calculalo de novo.
- **A excepción é Alt cando sobe,** se trae un carácter: o do código que se escribiu con el no teclado numérico, coma Alt+0225, que escribe á. Windows dálle ese carácter á tecla Alt cando sobe, e as teclas do teclado numérico non escriben nada ([Código › meiga-win › `keyboard.rs`](meiga-win.md#cratesmeiga-winsrckeyboardrs)). Vai tal cal, sen o ESC de Alt, e só co Bloq Num activado, coma no Windows Terminal. `char::from_u32` descarta unha metade dun carácter, que un código así non pode dar.
- **`high_surrogate`**, o único estado: a primeira metade dun carácter, que espera pola segunda en VT. Cambiar ao win32-input-mode descártaa.

```rust
fn push_vt(&mut self, event: &KeyEvent, modes: &Modes, out: &mut Vec<u8>) {
    // The first half of a character only waits for the second one.
    let high_surrogate = self.high_surrogate.take();
    // A key that brings a character that is not its own sends the
    // character, not its sequence.
    if !own_character(event.key, event.character) {
        self.push_vt_character(event, high_surrogate, out);
        return;
    }
    let controls = event.controls;
    let (alt, ctrl) = (controls.alt(), controls.ctrl());
    // The parameter of the modifiers in the sequences of xterm: 1, plus
    // 1 for Shift, 2 for Alt and 4 for Ctrl.
    let modifiers = 1 + u16::from(controls.shift()) + 2 * u16::from(alt) + 4 * u16::from(ctrl);
    // Alt adds ESC before the keys that have no parameter for it.
    let with_alt = |out: &mut Vec<u8>, bytes: &[u8]| {
        if alt {
            out.push(ESC);
        }
        out.extend_from_slice(bytes);
    };
    match event.key {
        vk::UP => push_cursor_key(out, b'A', modifiers, modes.app_cursor),
        vk::DOWN => push_cursor_key(out, b'B', modifiers, modes.app_cursor),
        vk::RIGHT => push_cursor_key(out, b'C', modifiers, modes.app_cursor),
        vk::LEFT => push_cursor_key(out, b'D', modifiers, modes.app_cursor),
        vk::HOME => push_cursor_key(out, b'H', modifiers, modes.app_cursor),
        vk::END => push_cursor_key(out, b'F', modifiers, modes.app_cursor),
        vk::INSERT => push_tilde_key(out, 2, modifiers),
        vk::DELETE => push_tilde_key(out, 3, modifiers),
        vk::PRIOR => push_tilde_key(out, 5, modifiers),
        vk::NEXT => push_tilde_key(out, 6, modifiers),
        // F1 to F4 always go as in the application cursor mode.
        vk::F1..=vk::F4 => {
            let last = b'P' + (event.key - vk::F1) as u8;
            push_cursor_key(out, last, modifiers, true);
        }
        vk::F5..=vk::F12 => {
            let number = F5_TO_F12[usize::from(event.key - vk::F5)];
            push_tilde_key(out, number, modifiers);
        }
        // Backspace sends DEL, and Ctrl+Backspace, BS.
        vk::BACK => with_alt(out, if ctrl { b"\x08" } else { b"\x7f" }),
        vk::TAB if controls.shift() => out.extend_from_slice(b"\x1b[Z"),
        vk::TAB => with_alt(out, b"\t"),
        vk::RETURN => with_alt(
            out,
            if ctrl {
                b"\n"
            } else if modes.line_feed_new_line {
                b"\r\n"
            } else {
                b"\r"
            },
        ),
        vk::ESCAPE => with_alt(out, &[ESC]),
        vk::PAUSE => out.push(0x1a),
        // Ctrl+Break.
        vk::CANCEL => out.push(0x03),
        _ => self.push_vt_character(event, high_surrogate, out),
    }
}
```

**`push_vt`** escribe o que envía o xterm cando a tecla baixa unha vez:

| Tecla | Envía |
|---|---|
| Frechas, Inicio e Fin | `ESC [ A`–`D`, `H` e `F`, ou `ESC O …` no modo aplicación (`app_cursor`). |
| Insert, Supr, RePáx e AvPáx | `ESC [ 2 ~`, `3 ~`, `5 ~` e `6 ~`. |
| F1–F4 | `ESC O P`–`S`, sempre coma no modo aplicación. |
| F5–F12 | `ESC [ n ~`, cos números de `F5_TO_F12`: 15, 17, 18, 19, 20, 21, 23 e 24. |
| Retroceso | `DEL` (`0x7F`); con Ctrl, `BS` (`0x08`). Windows escribe ao revés, `BS` sen Ctrl e `DEL` con el, pero os terminais VT, coma o Windows Terminal, envían `DEL`. |
| Tab e Maiús+Tab | `HT` e `ESC [ Z`. |
| Intro | `CR`; co modo *LNM* (`line_feed_new_line`), `CR LF`; con Ctrl, `LF`. |
| Esc | `ESC`. |
| Pausa | `SUB` (`0x1A`), o Ctrl+Z, coma no Windows Terminal: Pausa non ten secuencia VT. |
| Ctrl+Inter | `ETX` (`0x03`), coma Ctrl+C. |
| As demais | O seu carácter, en `push_vt_character`. |

- **`modifiers`** é o parámetro das modificadoras do xterm: 1, máis 1 por Maiús, 2 por Alt e 4 por Ctrl. Con algunha, as frechas e as teclas con `~` lévano: Ctrl+frecha esquerda é `ESC [ 1 ; 5 D`.
- **`with_alt`** pon un ESC diante, coma fan os terminais VT con Alt, nas teclas que non teñen o parámetro.
- `high_surrogate` cóllese ao principio con `take`: se esta tecla non é a segunda metade dun carácter, a primeira pérdese, porque xa non ten con que xuntarse.
- **Unha tecla que trae un carácter que non é seu** envía o carácter, non a súa secuencia ([`own_character`](#os-axudantes-das-secuencias)). Pasa cando unha tecla morta non se xunta coa seguinte: Windows escribe os dous caracteres coa segunda tecla, o primeiro chega con ela e o segundo chega despois, só, coa tecla do seu código de exploración. Despois de ´, Intro trae o ´, e o `CR` chega só: sen esta comprobación irían dous `CR`. E se a seguinte é o 7 do teclado numérico, o 7 chega coa tecla Inicio, e iría `ESC [ H`.

```rust
fn push_vt_character(
    &mut self,
    event: &KeyEvent,
    high_surrogate: Option<u16>,
    out: &mut Vec<u8>,
) {
    let (alt, ctrl) = (event.controls.alt(), event.controls.ctrl());
    let unit = event.character;
    // Windows writes a space for Ctrl+Space, and nothing for some keys
    // with Ctrl, such as Ctrl+2: a VT terminal sends control codes.
    if ctrl && (unit == 0 || unit == u16::from(b' ')) {
        // But AltGr writes characters: with a key that writes nothing
        // with it, as a dead key, it sends nothing.
        if unit == 0 && event.controls.alt_gr() {
            return;
        }
        if let Some(code) = control_code(event.key) {
            if alt {
                out.push(ESC);
            }
            out.push(code);
        }
        return;
    }
    let character = match unit {
        0 => return,
        0xd800..=0xdbff => {
            self.high_surrogate = Some(unit);
            return;
        }
        0xdc00..=0xdfff => {
            // A second half without its first one is lost.
            let Some(high) = high_surrogate else {
                return;
            };
            let code = 0x10000 + (u32::from(high - 0xd800) << 10) + u32::from(unit - 0xdc00);
            char::from_u32(code)
        }
        _ => char::from_u32(u32::from(unit)),
    };
    let Some(character) = character else {
        return;
    };
    // AltGr writes its characters with Ctrl and Alt down: they go as
    // they are. With Alt, a VT terminal sends ESC first.
    let alt_gr = ctrl && alt && character > ' ' && character != '\x7f';
    if alt && !alt_gr {
        out.push(ESC);
    }
    push_utf8(out, character);
}
```

**`push_vt_character`** envía o carácter da tecla en UTF-8:

- **Ctrl sen carácter.** Windows dá o código de control de Ctrl cunha letra (Ctrl+C escribe `0x03`), pero nada para Ctrl+2 e un espazo para Ctrl+Espazo. Para esas, [`control_code`](#os-axudantes-das-secuencias) dá o que envía un terminal VT, e se a tecla non ten ningún, non se envía nada.
- **AltGr sen carácter** non envía nada. AltGr é o Ctrl esquerdo e o Alt dereito ([`alt_gr`](#controlkeys)), pero serve para escribir caracteres. Unha tecla que con AltGr non escribe nada é unha tecla morta, coma o ~ de AltGr+4 no teclado español, cuxo carácter vai coa tecla seguinte, ou unha sen carácter con AltGr, coma a A. Ctrl+Alt co Alt esquerdo segue enviando os códigos de control: Ctrl+Alt+A é `ESC` e `0x01`.
- **As dúas metades.** A primeira (`0xD800`–`0xDBFF`) gárdase; a segunda (`0xDC00`–`0xDFFF`) xúntase con ela no carácter enteiro. Unha segunda metade sen a primeira, ou unha primeira que non vai seguida da segunda, pérdese.
- **AltGr** escribe os seus caracteres con Ctrl e Alt premidos ([Tecnoloxías › O teclado en Windows](../tecnoloxias.md#41-o-teclado-en-windows)): un carácter imprimíbel con Ctrl e Alt é de AltGr e vai tal cal. Con Alt só, vai un ESC diante: Alt+f é `ESC f`.
- [`push_utf8`](#os-axudantes-das-secuencias) escribe o carácter en UTF-8.

### Os eventos de foco

```rust
pub fn focus_sequence(focused: bool, modes: &Modes) -> &'static [u8] {
    match (modes.focus_events, focused) {
        (false, _) => b"",
        (true, true) => b"\x1b[I",
        (true, false) => b"\x1b[O",
    }
}
```

**`focus_sequence`** di que enviarlle ao programa cando a xanela gaña ou perde o foco: `ESC [ I` ou `ESC [ O` se o pediu (`?1004`, que segue `alacritty_terminal`), e nada se non. Úsaa a vista do terminal ([Código › meiga › `view.rs`](meiga.md#cratesmeigasrcviewrs)).

### Os axudantes das secuencias

```rust
fn push_win32(event: &KeyEvent, out: &mut Vec<u8>) {
    out.extend_from_slice(b"\x1b[");
    for number in [
        event.key,
        event.scan,
        event.character,
        u16::from(event.down),
        event.controls.bits(),
    ] {
        push_decimal(out, number);
        out.push(b';');
    }
    // The ConPTY stores a 0 as it is, and a program could lose the key.
    push_decimal(out, event.repeat.max(1));
    out.push(b'_');
}

fn push_cursor_key(out: &mut Vec<u8>, last: u8, modifiers: u16, application: bool) {
    if modifiers > 1 {
        out.extend_from_slice(b"\x1b[1;");
        push_decimal(out, modifiers);
    } else if application {
        out.extend_from_slice(b"\x1bO");
    } else {
        out.extend_from_slice(b"\x1b[");
    }
    out.push(last);
}

fn push_tilde_key(out: &mut Vec<u8>, number: u16, modifiers: u16) {
    out.extend_from_slice(b"\x1b[");
    push_decimal(out, number);
    if modifiers > 1 {
        out.push(b';');
        push_decimal(out, modifiers);
    }
    out.push(b'~');
}
```

- **`push_win32`** escribe o evento coma `ESC [ Vk ; Sc ; Uc ; Kd ; Cs ; Rc _`, sempre cos seis números, coma o Windows Terminal. As repeticións van polo menos a 1: o ConPTY garda un 0 tal cal, e un programa que conta as repeticións non vería a tecla.
- **`push_cursor_key`**, para as frechas, Inicio, Fin e F1–F4: `ESC [ X`, ou `ESC O X` no modo aplicación; con modificadoras, `ESC [ 1 ; m X`, tamén no modo aplicación.
- **`push_tilde_key`**, para Insert, Supr, RePáx, AvPáx e F5–F12: `ESC [ n ~`, e `ESC [ n ; m ~` con modificadoras.

```rust
fn control_code(key: u16) -> Option<u8> {
    Some(match key {
        // Ctrl+Space and Ctrl+2, which is Ctrl+@: NUL.
        vk::SPACE | 0x32 => 0x00,
        // Ctrl+3 to Ctrl+7, which are Ctrl+[, \, ], ^ and _: ESC to US.
        0x33..=0x37 => (key - 0x33) as u8 + ESC,
        // Ctrl+8: DEL.
        0x38 => 0x7f,
        // Ctrl+A to Ctrl+Z, when the layout gives no character with them,
        // as with Ctrl+Alt: 1 to 26.
        0x41..=0x5a => (key - 0x40) as u8,
        _ => return None,
    })
}

pub(crate) fn push_decimal(out: &mut Vec<u8>, value: u16) {
    let mut digits = [0; 5];
    let mut start = digits.len();
    let mut rest = value;
    loop {
        start -= 1;
        digits[start] = b'0' + (rest % 10) as u8;
        rest /= 10;
        if rest == 0 {
            break;
        }
    }
    out.extend_from_slice(&digits[start..]);
}
```

- **`control_code`** dá o código de control de Ctrl cunha tecla que non o ten en Windows: Ctrl+Espazo e Ctrl+2, `NUL`; Ctrl+3 a Ctrl+7, de `ESC` a `US` (son Ctrl+[, \, ], ^ e _ no teclado estadounidense); Ctrl+8, `DEL`; e Ctrl+A–Z, do 1 ao 26, para cando a distribución non dá ningún, coma con Ctrl+Alt.
- **`push_decimal`** escribe as cifras de atrás cara a diante nun *array* de 5, as que ten como moito un `u16`, sen `format!`, que reservaría un texto. Desde a subfase 1.8 é `pub(crate)`, visíbel en todo o crate pero non fóra del: tamén escribe os números dos informes do rato en SGR ([`mouse.rs`](#cratesmeiga-coresrcmousers)).
- O módulo **`vk`** ten os códigos de tecla virtual que a codificación VT trata á parte, e `F5_TO_F12`, os números das teclas de función, que saltan algúns por razóns históricas. `meiga-core` non depende de `windows-sys`, así que define os seus.

```rust
pub(crate) fn own_character(key: u16, unit: u16) -> bool {
    match key {
        vk::BACK => matches!(unit, 0 | 0x08 | 0x7f),
        vk::TAB => matches!(unit, 0 | 0x09),
        vk::RETURN => matches!(unit, 0 | 0x0a | 0x0d),
        vk::ESCAPE => matches!(unit, 0 | 0x1b),
        vk::CANCEL => matches!(unit, 0 | 0x03),
        vk::PAUSE | vk::PRIOR..=vk::DOWN | vk::INSERT | vk::DELETE | vk::F1..=vk::F12 => unit == 0,
        _ => true,
    }
}

fn push_utf8(out: &mut Vec<u8>, character: char) {
    let mut utf8 = [0; 4];
    out.extend_from_slice(character.encode_utf8(&mut utf8).as_bytes());
}
```

- **`own_character`** di se o carácter dun evento é da súa tecla, ou se non trae ningún: só entón envían a súa secuencia as teclas que a teñen ([`push_vt`](#keyencoder)). Retroceso escribe `BS`, e con Ctrl `DEL`; Tab, `HT`; Intro, `CR`, e con Ctrl `LF`; Esc, `ESC`; e Ctrl+Inter, `ETX`. As frechas, Inicio, Fin, Insert, Supr, RePáx, AvPáx, Pausa e as teclas de función non escriben ningún. As outras teclas non teñen secuencia, e envían sempre o seu carácter. Desde a subfase 3.2 é `pub(crate)`, coma `push_decimal`, e o seu comentario di que tamén a usa o campo de texto, coa mesma regra: unha tecla que trae un carácter que non é seu escribe ese carácter, e a tecla chega despois co seu e fai o seu traballo ([`Field::key`](#key-a-entrada-das-teclas)).
- **`push_utf8`** escribe o carácter en UTF-8 nun *array* de 4 bytes, sen reservar memoria. Úsano `encode`, para o carácter de Alt cando sobe, e `push_vt_character`.

**Probas.** Doce:

| Proba | Que comproba |
|---|---|
| `win32_input_mode_sends_the_whole_key_events` | Eventos que enviou o Windows Terminal co teclado español: a A ao baixar e ao subir, AltGr+2 (@), Maiús+Intro, a frecha esquerda, unha tecla morta e Maiús só; as repeticións, e un 0 que vai coma 1; e os números máis grandes. |
| `vt_sends_the_characters_in_utf8_when_keys_go_down` | a, ñ e €, en UTF-8; unha tecla morta non envía nada, e a seguinte, á; nin as teclas que soben nin Maiús só envían nada. |
| `vt_joins_the_two_halves_of_a_character` | 😀 en dous eventos dá os seus 4 bytes, tamén cos eventos de subir no medio e con Alt; unha metade soa pérdese. |
| `vt_sends_the_character_of_an_alt_code_when_alt_goes_up` | Alt+0225: Alt e as teclas do teclado numérico non envían nada ao baixar, e Alt envía á ao subir; sen Bloq Num, nada. |
| `vt_cursor_keys_follow_the_application_mode_and_the_modifiers` | As frechas, Inicio e Fin, co modo aplicación e coas modificadoras. |
| `vt_editing_and_function_keys_send_their_sequences` | Insert, Supr, RePáx, AvPáx, F1, F4, F5, F10 e F12, con e sen modificadoras. |
| `vt_backspace_tab_enter_and_escape_send_control_codes` | Retroceso, Tab, Intro (tamén co modo *LNM*), Esc, Pausa e Ctrl+Inter, con Maiús, Ctrl e Alt. |
| `vt_keys_with_a_character_that_is_not_theirs_send_the_character` | Despois de ´, Intro, que trae o ´, e o `CR` que chega só envían `´` e `CR`; con Retroceso, `´` e `DEL`; e co 7 do teclado numérico, que chega coa tecla Inicio, `´7`. |
| `vt_ctrl_sends_control_codes` | Ctrl+C, Ctrl+Espazo, Ctrl+2, Ctrl+3, Ctrl+7, Ctrl+8 e Ctrl+Alt+A, tamén con AltGr e o Ctrl dereito; unha tecla sen código de control non envía nada. |
| `vt_alt_sends_escape_first_but_altgr_does_not` | Alt+a é `ESC a`; AltGr+2 e Ctrl+Alt+2 envían `@`, e o `[` de AltGr, `[`, sen ESC; e o ~ de AltGr+4, unha tecla morta, non envía nada ata a tecla seguinte. |
| `vt_repeats_the_keys_that_repeated` | Unha tecla repetida tres veces envía tres veces o seu, tamén unha frecha; con 0 repeticións, unha. |
| `focus_events_only_go_if_the_program_asks_for_them` | `ESC [ I` e `ESC [ O` co modo, e nada sen el. |

## `crates/meiga-core/src/mouse.rs`

Desde a subfase 1.8: o rato, coma o conta meiga e coma o recibe un programa que o pide ([ADR 0015](../adr/0015-o-rato-a-seleccion-e-o-portapapeis.md)). A xanela dá os eventos do rato en píxeles, cos botóns e as teclas modificadoras ([Código › meiga-win › `event.rs`](meiga-win.md#cratesmeiga-winsrceventrs)), e o rato da vista decide se seleccionan texto ou van ao programa ([Código › meiga › `mouse.rs`](meiga.md#cratesmeigasrcmousers)). Este módulo ten o que non depende de Windows: a cela debaixo do punteiro, os clics seguidos, as partes da roda e, desde a subfase 1.9, as filas do historial que move ([ADR 0016](../adr/0016-o-historial-o-tamano-e-o-zoom.md)), e os informes, no formato que pediu o programa. Coma `input.rs`, só traballa con números e bytes: as súas probas non necesitan ningún rato.

**Os informes e o ConPTY.** Un programa pide os clics (`?1000`), tamén os movementos cun botón premido (`?1002`) ou todos os movementos (`?1003`), e o formato dos informes: o do primeiro xterm, o mesmo en UTF-8 (`?1005`) ou SGR (`?1006`) ([Tecnoloxías › O rato nos programas](../tecnoloxias.md#45-o-rato-nos-programas-modos-e-formatos)). O ConPTY pásalles os informes tal cal aos programas que len a entrada coma un terminal, coma os de WSL: por iso meiga os escribe no formato que pediu o programa. Para os programas da consola de Windows, o ConPTY pide el mesmo SGR, e converte os informes en eventos do rato de Windows.

```rust
pub const WHEEL_DELTA: i32 = 120;

const MAX_X10: u16 = 127 - 33;

const MAX_UTF8: u16 = 0x7ff - 33;
```

- **`WHEEL_DELTA`**: o xiro que Windows conta coma unha parte enteira (*notch*) da roda: 120. As mensaxes da roda dan o xiro en 120 avos dunha parte. Úsano [`WheelTurns` e, desde a subfase 1.9, `WheelRows`](#a-roda-wheelturns-wheelrows-e-arrow_keys).
- **`MAX_X10`** e **`MAX_UTF8`**: a columna e a fila máis grandes, desde 0, que caben nun informe no formato do primeiro xterm e no de UTF-8 ([`report`](#os-informes-report)).

### `Button`, `Modifiers`, `Wheel` e `MouseAction`

```rust
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Button {
    Left,
    Middle,
    Right,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Modifiers {
    pub shift: bool,
    pub alt: bool,
    pub ctrl: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Wheel {
    Up,
    Down,
    Left,
    Right,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MouseAction {
    Press(Button),
    Release(Button),
    Move(Option<Button>),
    Wheel(Wheel),
}
```

| Tipo | Que é |
|---|---|
| `Button` | Un dos tres botóns: o esquerdo, o do medio e o dereito. |
| `Modifiers` | As teclas modificadoras que estaban premidas co evento: Maiús, Alt e Ctrl, sen distinguir a esquerda da dereita, porque os informes non o fan. As mensaxes do rato de Windows levan Maiús e Ctrl, e a xanela engade Alt ([Tecnoloxías › O rato en Windows](../tecnoloxias.md#44-o-rato-en-windows)). |
| `Wheel` | Cara a onde vai unha parte da roda: adiante é `Up`, e a inclinación cara á dereita, `Right`. |
| `MouseAction` | O que fixo o rato, coma o recibe un programa: premer ou soltar un botón, moverse a outra cela, cun botón premido ou sen ningún (`Move(None)`), ou xirar a roda unha parte. |

- `Button` e `Modifiers` están en `meiga-core`, e non en `meiga-win`, porque tamén os levan os eventos da xanela (`Event::MouseDown`, `MouseUp`, `MouseMoved` e `Wheel`): a xanela e os informes falan dos mesmos botóns, sen conversións.
- Todos son `Copy`: uns poucos bytes, que van por valor.

### Que pediu o programa: `is_tracked` e `wants`

```rust
pub const fn is_tracked(modes: &Modes) -> bool {
    modes.mouse_clicks || modes.mouse_drag || modes.mouse_motion
}

pub const fn wants(action: MouseAction, modes: &Modes) -> bool {
    match action {
        MouseAction::Press(_) | MouseAction::Release(_) | MouseAction::Wheel(_) => {
            is_tracked(modes)
        }
        MouseAction::Move(Some(_)) => modes.mouse_drag || modes.mouse_motion,
        MouseAction::Move(None) => modes.mouse_motion,
    }
}
```

- **`is_tracked`** di se o programa pediu o rato, con calquera dos tres modos de [`Modes`](#changes-e-modes): `mouse_clicks` (`?1000`), `mouse_drag` (`?1002`) ou `mouse_motion` (`?1003`). Mentres tanto, os clics van ao programa en vez de seleccionar texto, salvo con Maiús, e o punteiro é a frecha.
- **`wants`** di se o programa pediu `action`: os clics, as solturas e a roda, con calquera dos tres; os movementos cun botón premido, con `?1002` ou `?1003`; e os movementos sen ningún, só con `?1003`. `alacritty_terminal` só deixa activo un dos tres, o último que se pediu: `?1003` non activa tamén `mouse_drag`, e por iso os movementos cun botón miran os dous.

### Os informes: `report`

```rust
pub fn report(
    action: MouseAction,
    at: GridPoint,
    modifiers: Modifiers,
    modes: &Modes,
    out: &mut Vec<u8>,
) -> bool {
    if !wants(action, modes) {
        return false;
    }
    let button = |button: Button| match button {
        Button::Left => 0,
        Button::Middle => 1,
        Button::Right => 2,
    };
    let mut code: u8 = match action {
        MouseAction::Press(pressed) => button(pressed),
        // Without SGR, a release does not say which button it was.
        MouseAction::Release(released) if modes.sgr_mouse => button(released),
        MouseAction::Release(_) => 3,
        MouseAction::Move(down) => 32 + down.map_or(3, button),
        MouseAction::Wheel(Wheel::Up) => 64,
        MouseAction::Wheel(Wheel::Down) => 65,
        MouseAction::Wheel(Wheel::Left) => 66,
        MouseAction::Wheel(Wheel::Right) => 67,
    };
    for (down, value) in [
        (modifiers.shift, 4),
        (modifiers.alt, 8),
        (modifiers.ctrl, 16),
    ] {
        if down {
            code += value;
        }
    }
    if modes.sgr_mouse {
        out.extend_from_slice(b"\x1b[<");
        push_decimal(out, u16::from(code));
        out.push(b';');
        push_decimal(out, at.column.saturating_add(1));
        out.push(b';');
        push_decimal(out, at.row.saturating_add(1));
        out.push(if matches!(action, MouseAction::Release(_)) {
            b'm'
        } else {
            b'M'
        });
        return true;
    }
    let max = if modes.utf8_mouse { MAX_UTF8 } else { MAX_X10 };
    if at.column > max || at.row > max {
        return false;
    }
    out.extend_from_slice(b"\x1b[M");
    // At most 67 + 4 + 8 + 16 + 32 = 127: it is always one byte.
    out.push(32 + code);
    for value in [at.column, at.row] {
        let value = u32::from(value) + 33;
        match char::from_u32(value) {
            Some(character) if value >= 0x80 => {
                let mut utf8 = [0; 4];
                out.extend_from_slice(character.encode_utf8(&mut utf8).as_bytes());
            }
            // Up to 127, the byte, in both formats.
            _ => out.push(value as u8),
        }
    }
    true
}
```

**`report`** engade a `out` o informe de `action` na cela `at`, coas teclas `modifiers`, no formato que pediu o programa, e di se o engadiu: non o fai se o programa non pediu `action`, nin se a cela non cabe no formato.

1. **O número** do que fixo o rato, o mesmo nos tres formatos ([Tecnoloxías › O rato nos programas](../tecnoloxias.md#45-o-rato-nos-programas-modos-e-formatos)): premer o botón esquerdo, o do medio ou o dereito, 0, 1 e 2; soltalo, o botón en SGR, e 3 nos outros formatos, que non din cal; moverse, 32 máis o botón, ou 35 sen ningún (`map_or(3, button)`); e a roda, de 64 a 67. Maiús suma 4, Alt 8 e Ctrl 16.
2. **En SGR**: `ESC [ < número ; columna ; fila M`, en decimal, coa columna e a fila desde 1, e `m` en vez de `M` ao soltar. Non ten límite: `saturating_add` só evita desbordar na columna ou na fila 65 535.
3. **Nos outros dous formatos**: `ESC [ M` e tres valores, cada un con 32 máis, para que non sexan caracteres de control: o número, a columna e a fila. A columna e a fila contan desde 1, así que levan 33 máis ca a cela. Ata 127, cada valor é un byte; desde 128, só no formato de UTF-8, un carácter en UTF-8, de dous bytes.

- **Os límites.** O primeiro xterm escribía cada valor nun byte, ata 255: a columna 223. Pero o ConPTY le a entrada coma UTF-8, e un byte solto desde 128 chegaría ao programa coma outro carácter. Así que, nese formato, meiga só escribe os valores ata 127: `MAX_X10`, 127 − 33 = 94, as 95 primeiras columnas e filas. No de UTF-8, os caracteres de ata dous bytes chegan ata `0x7FF`: `MAX_UTF8`, 0x7FF − 33 = 2 014, as 2 015 primeiras. Fóra delas, o informe non se envía ([ADR 0015](../adr/0015-o-rato-a-seleccion-e-o-portapapeis.md)).
- **O número sempre é un byte**: como moito 67 da roda, máis 28 das tres modificadoras, máis 32, 127. Por iso `code` é un `u8`, e `32 + code` nunca desborda.
- `Some(character) if value >= 0x80` é un brazo de `match` cunha garda: só os valores desde 128 van en UTF-8, e o resto, tal cal (`value as u8`, que xa cabe). Ningún valor chega ás metades de UTF-16 (`0xD800`–`0xDFFF`), que non son caracteres: `char::from_u32` sempre dá `Some`.
- Os números de SGR escríbeos [`push_decimal`](#os-axudantes-das-secuencias), de `input.rs`, sen reservar memoria. Quen chama reutiliza `out`, e envía cada informe nunha escritura, porque o ConPTY le cada escritura enteira ([Código › meiga › `mouse.rs`](meiga.md#cratesmeigasrcmousers)).

### A cela debaixo do punteiro: `cell_at`

```rust
pub fn cell_at(at: Point, cell: Size, grid: GridSize) -> (GridPoint, Side) {
    let width = i64::from(cell.width.max(1));
    let height = i64::from(cell.height.max(1));
    let (x, y) = (i64::from(at.x), i64::from(at.y));
    let column = (x.max(0) / width).min(i64::from(grid.columns.max(1)) - 1);
    let row = (y.max(0) / height).min(i64::from(grid.rows.max(1)) - 1);
    let side = if x >= width * i64::from(grid.columns) || x.max(0) % width > width / 2 {
        Side::Right
    } else {
        Side::Left
    };
    // Both are below the columns and the rows of the grid.
    (GridPoint::new(column as u16, row as u16), side)
}
```

**`cell_at`** dá a cela dunha grella de `grid` celas de `cell` píxeles que está debaixo do punto `at`, en píxeles desde a esquina superior esquerda da grella, e o lado da cela onde está ([`Side`](#word_delimiters-selectionkind-e-side)).

- **A cela**: a columna é `x` entre o ancho da cela, e a fila, `y` entre o alto. Un punto fóra da grella dá a cela máis próxima (`max(0)` e `min`): unha selección que se arrastra fóra da xanela chega ata o bordo.
- **O lado**: a metade esquerda chega ata a metade da cela, incluída; cunha cela de 10 píxeles, do 0 ao 5. Á dereita da grella, o lado dereito da última columna: unha selección arrastrada alén do final da fila colle a súa última cela.
- As contas fanse en `i64`, onde caben unha coordenada `i32`, un tamaño `u32` e o seu produto. `max(1)` evita dividir entre 0 mentres non se sabe o tamaño da cela, e que unha grella sen columnas ou sen filas dea −1. Despois de `min`, a columna e a fila caben nun `u16`: o `as u16` non corta nada.

### Os clics seguidos: `ClickSettings` e `Clicks`

```rust
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ClickSettings {
    pub time: Duration,
    pub width: u32,
    pub height: u32,
}

impl Default for ClickSettings {
    fn default() -> Self {
        Self {
            time: Duration::from_millis(500),
            width: 4,
            height: 4,
        }
    }
}
```

**`ClickSettings`**: cando toma Windows dous clics por un dobre clic: o tempo do dobre clic (`GetDoubleClickTime`) e o tamaño do rectángulo arredor do primeiro clic onde ten que caer o segundo (`SM_CXDOUBLECLK` e `SM_CYDOUBLECLK`). Os valores por defecto son os de Windows: 500 ms e 4 × 4 píxeles. meiga le os de Windows ao arrancar, e outra vez cada vez que a xanela recupera o foco, porque poden cambiar mentres corre ([Código › meiga-win › `window.rs`](meiga-win.md#cratesmeiga-winsrcwindowrs), [Tecnoloxías › O rato en Windows](../tecnoloxias.md#44-o-rato-en-windows)).

```rust
#[derive(Clone, Copy, Debug, Default)]
pub struct Clicks {
    last: Option<LastClick>,
}

#[derive(Clone, Copy, Debug)]
struct LastClick {
    button: Button,
    at: Point,
    when: Instant,
    count: u8,
}

impl Clicks {
    …
    pub fn press(
        &mut self,
        button: Button,
        at: Point,
        now: Instant,
        settings: &ClickSettings,
    ) -> u8 {
        let count = match self.last {
            Some(last)
                if last.button == button
                    && now.saturating_duration_since(last.when) <= settings.time
                    && at.x.abs_diff(last.at.x) <= settings.width / 2
                    && at.y.abs_diff(last.at.y) <= settings.height / 2 =>
            {
                last.count % 3 + 1
            }
            _ => 1,
        };
        self.last = Some(LastClick {
            button,
            at,
            when: now,
            count,
        });
        count
    }

    pub fn reset(&mut self) {
        self.last = None;
    }
}
```

**`Clicks`** conta os clics dun botón: o segundo seguido é un dobre clic, e o terceiro, un triplo clic. Windows só detecta os dobres, e só nas xanelas dunha clase con `CS_DBLCLKS`: meiga cóntaos todos, coas mesmas medidas ([ADR 0015](../adr/0015-o-rato-a-seleccion-e-o-portapapeis.md)).

- **`press`** conta un clic de `button` en `at`, no instante `now`: 1, ou un máis ca o anterior se foi do mesmo botón, dentro do tempo do dobre clic e preto del, como moito a metade do rectángulo cara a cada lado: 2 píxeles, co de Windows. Despois do 3 volta o 1 (`last.count % 3 + 1`): o cuarto clic seguido é outra vez un clic só. O rato da vista selecciona celas cun clic, palabras con dous e liñas con tres.
- Cada clic compárase co anterior: o tempo e a distancia cóntanse desde el. `LastClick`, privado, lémbrao: o botón, onde e cando foi, e a súa conta.
- `now` vén de fóra, coma en [`FramePacer`](#cratesmeiga-coresrcpacingrs): as probas usan instantes inventados. `saturating_duration_since` dá 0 se `now` fose anterior, en vez de fallar, e `abs_diff` dá a distancia entre dous `i32` coma un `u32`, sen desbordar.
- **`reset`** esquece o último clic: o seguinte é un clic só.

### A roda: `WheelTurns`, `WheelRows` e `arrow_keys`

```rust
#[derive(Clone, Copy, Debug, Default)]
pub struct WheelTurns {
    vertical: i32,
    horizontal: i32,
}

impl WheelTurns {
    …
    pub fn add(&mut self, delta: i16, horizontal: bool) -> i32 {
        let total = if horizontal {
            &mut self.horizontal
        } else {
            &mut self.vertical
        };
        let delta = i32::from(delta);
        if (*total > 0 && delta < 0) || (*total < 0 && delta > 0) {
            *total = 0;
        }
        *total += delta;
        let notches = *total / WHEEL_DELTA;
        *total -= notches * WHEEL_DELTA;
        notches
    }
    …
}
```

**`WheelTurns`** suma os xiros da roda ata facer partes enteiras: algúns ratos e *touchpads* xiran menos dunha parte cada vez ([Tecnoloxías › O rato en Windows](../tecnoloxias.md#44-o-rato-en-windows)).

- **`add`** suma un xiro de `delta`, en 120 avos dunha parte, positivo adiante ou cara á dereita, e devolve as partes enteiras que completa, con signo. O que sobra queda para o seguinte xiro: tres anacos de 50 dan 0, 0 e 1 parte, e quedan 30.
- **Un xiro no outro sentido empeza de cero**: o que sobraba no sentido anterior esquécese.
- A división de enteiros corta cara a 0, así que o que sobra ten o signo do total e nunca chega a unha parte.
- A roda e a inclinación cara aos lados suman por separado (`vertical` e `horizontal`). `delta` é un `i16`, coma o trae a mensaxe de Windows, e o total, un `i32`. `reset` esquece o que sobraba.
- Por cada parte, o rato da vista envíalle ao programa un informe, se pediu o rato, ou frechas, na pantalla alternativa. Desde a subfase 1.9, con Ctrl, cada parte é un punto de zoom; e, na pantalla principal, se a roda despraza unha pantalla, unha pantalla do historial. Se despraza liñas, as filas cóntaas `WheelRows`.

```rust
#[derive(Clone, Copy, Debug, Default)]
pub struct WheelRows {
    total: i64,
}

impl WheelRows {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add(&mut self, delta: i16, rows: u32) -> i32 {
        let delta = i64::from(delta) * i64::from(rows);
        if (self.total > 0 && delta < 0) || (self.total < 0 && delta > 0) {
            self.total = 0;
        }
        self.total += delta;
        let whole = self.total / i64::from(WHEEL_DELTA);
        self.total -= whole * i64::from(WHEEL_DELTA);
        i32::try_from(whole).unwrap_or(if whole < 0 { i32::MIN } else { i32::MAX })
    }
}
```

**`WheelRows`**, desde a subfase 1.9, suma os xiros da roda en filas do historial, a un número de filas por parte: as liñas que din as preferencias de Windows ([ADR 0016](../adr/0016-o-historial-o-tamano-e-o-zoom.md)).

- **`add`** suma un xiro de `delta`, en 120 avos dunha parte, positivo adiante, a `rows` filas por parte, e devolve as filas enteiras que completa, con signo: positivas cara arriba, coma en [`Scroll::Rows`](#o-historial-e-a-súa-vista).
- **En 120 avos dunha fila** (`total`): cada xiro suma `delta` por `rows`, así que unha parte enteira son `rows` filas, e os anacos dunha parte moven unha fila en canto chegan a ela: con 3 filas por parte, cada 40. Contando partes enteiras, coma `WheelTurns`, a vista moveríase de `rows` en `rows` filas ao completar cada parte, e cun *touchpad*, que xira menos dunha parte cada vez, iría a saltos.
- **Un xiro no outro sentido empeza de cero**, coma en `WheelTurns`.
- As contas fanse en `i64`, onde caben un `i16` por un `u32` e as súas sumas. As filas, nun `i32`, quedan no máximo ou no mínimo (`unwrap_or`) se non caben, o que só pasaría cun número de filas por parte enorme.
- O rato da vista úsao na pantalla principal, cando a roda despraza liñas, e move a vista as filas que devolve ([Código › meiga › `mouse.rs`](meiga.md#cratesmeigasrcmousers)).

```rust
pub const fn arrow_keys(wheel: Wheel) -> [KeyEvent; 2] {
    let (key, scan) = match wheel {
        Wheel::Up => (0x26, 0x48),
        Wheel::Down => (0x28, 0x50),
        Wheel::Left => (0x25, 0x4b),
        Wheel::Right => (0x27, 0x4d),
    };
    let down = KeyEvent {
        key,
        scan,
        character: 0,
        down: true,
        controls: ControlKeys::ENHANCED,
        repeat: 1,
    };
    let mut up = down;
    up.down = false;
    [down, up]
}
```

**`arrow_keys`** dá a frecha de `wheel`, ao baixar e ao subir, para os programas da pantalla alternativa que non pediron o rato, coma `less`: desprázanse coas frechas. É o modo `?1007` (`alternate_scroll` en `Modes`), activo ao principio, coma no xterm e no Windows Terminal ([Tecnoloxías › O rato nos programas](../tecnoloxias.md#45-o-rato-nos-programas-modos-e-formatos)).

- Son eventos de tecla coma os do teclado ([`KeyEvent`](#keyevent)): o código de tecla virtual de cada frecha (`VK_UP` é `0x26`…), o seu código de exploración, e `ENHANCED`, porque son as frechas de fóra do teclado numérico ([`ControlKeys`](#controlkeys)). O mesmo [`KeyEncoder`](#keyencoder) das teclas convérteos no que le o programa: co win32-input-mode, cada evento enteiro; en VT, a secuencia da frecha, `ESC [ A`…, ou `ESC O A`… no modo aplicación.
- As frechas de cada parte son as liñas das preferencias de Windows, ou as filas da pantalla se a roda despraza unha pantalla; cara aos lados, os caracteres. Cóntaas o rato da vista ([Código › meiga › `mouse.rs`](meiga.md#cratesmeigasrcmousers)).
- `let mut up = down;` copia o evento, que é `Copy`, e despois cambia un campo da copia: nunha `const fn` tamén se pode.

**Probas.** Nove:

| Proba | Que comproba |
|---|---|
| `the_program_gets_only_what_it_asked_for` | Sen modos, nada, e `is_tracked` é falso; con `?1000`, o clic (`ESC [ M` e tres bytes), pero non os movementos; con `?1002`, os movementos cun botón, pero non sen el; e con `?1003`, tamén sen botón. |
| `sgr_reports_say_the_button_and_the_cell_from_1` | En SGR, na columna 120 e na fila 30 (119 e 29 desde 0): premer o botón esquerdo e o do medio, soltar o dereito (co botón e `m`), moverse con e sen botón, a roda adiante e cara á dereita, as tres modificadoras, que suman 28, e Ctrl coa roda cara atrás. E sen límite: a columna 1 000 e a fila 301. |
| `the_first_format_does_not_say_which_button_went_up_and_has_a_limit` | No primeiro formato, soltar o botón dereito é 3, sen o botón; a columna 94 desde 0 é o byte 127; e a 95, de columna ou de fila, non se envía. |
| `utf8_reports_reach_further` | En UTF-8, a columna 94 é o byte 127, coma no primeiro formato; a 95, o carácter 128, en dous bytes; a última que cabe, `MAX_UTF8`, o carácter `0x7FF`; e a seguinte non se envía. |
| `the_cell_under_the_pointer_and_its_side` | Con celas de 10 × 20 píxeles nunha grella de 8 × 4: a metade esquerda chega ata a metade da cela, incluída; e fóra da grella, por riba, á esquerda, á dereita e por baixo, a cela máis próxima, co lado dereito á dereita da grella. |
| `clicks_in_a_row_count_up_to_three` | Tres clics seguidos dan 1, 2 e 3, o segundo 2 píxeles máis aló, e o cuarto, 1; tarde, lonxe ou con outro botón, 1; e despois de `reset`, 1. |
| `the_wheel_adds_up_whole_notches` | Unha parte enteira e tres cara atrás; un *touchpad* que dá unha parte en tres anacos de 50; o que sobra esquécese ao cambiar de sentido; a roda e a inclinación por separado; e `reset`. |
| `the_wheel_moves_the_history_by_rows_also_with_parts_of_a_notch` | Desde a subfase 1.9, con 3 filas por parte: unha parte adiante dá 3 filas, e dúas cara atrás, −6; un *touchpad* que xira de 30 en 30 move unha fila cada 40 (0, 1 e, con 20 máis, outra); e os 30 que sobran esquécense ao cambiar de sentido: −30 e −10 dan unha fila cara abaixo. |
| `the_wheel_on_the_alternate_screen_sends_arrows` | A frecha de arriba baixa e sobe, coma tecla estendida; e `KeyEncoder` converte a de abaixo en `ESC [ B`, e no modo aplicación, en `ESC O B`. |

## `crates/meiga-core/src/paste.rs`

Desde a subfase 1.8: o que le o programa cando se pega texto ([ADR 0015](../adr/0015-o-rato-a-seleccion-e-o-portapapeis.md), [Tecnoloxías › Pegar no terminal](../tecnoloxias.md#47-pegar-no-terminal-o-filtro-e-o-bracketed-paste)). Pegar é teclear: o texto vai á entrada do programa, e un carácter de control sería unha tecla, coma Esc ou Ctrl+C, e cada fin de liña, un Intro. O texto chega do portapapeis de Windows ([Código › meiga-win › `clipboard.rs`](meiga-win.md#cratesmeiga-winsrcclipboardrs)), e o módulo que copia e pega ([Código › meiga › `clipboard.rs`](meiga.md#cratesmeigasrcclipboardrs)) pásao por eiquí, cos modos do terminal, antes de envialo. Coma no Windows Terminal:

- o texto perde os caracteres de control, salvo o tabulador e os fins de liña, para que non poida enviar unha tecla coma Esc nin rematar o *bracketed paste* antes de tempo;
- cada fin de liña pasa a ser un CR, a tecla Intro;
- e, se o programa non pediu o *bracketed paste* (`?2004`), un texto dunha soa liña perde os brancos do final, para que un comando copiado co seu fin de liña non se execute só.

```rust
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Paste {
    pub text: String,
    pub bracketed: bool,
}

impl Paste {
    pub fn new(clipboard: &str, modes: &Modes) -> Self {
        let bracketed = modes.bracketed_paste;
        let clipboard = if bracketed {
            clipboard
        } else {
            trim(clipboard)
        };
        let mut text = String::with_capacity(clipboard.len());
        let mut after_cr = false;
        for character in clipboard.chars() {
            match character {
                // CR and LF, CRLF and LF CR... each line ends with one CR.
                '\n' if after_cr => {}
                '\r' | '\n' => text.push('\r'),
                '\t' => text.push('\t'),
                // C0, DEL and C1.
                '\0'..='\x1f' | '\x7f'..='\u{9f}' => {}
                _ => text.push(character),
            }
            after_cr = character == '\r';
        }
        Self { text, bracketed }
    }
    …
}
```

**`Paste`** é o texto listo para pegar (`text`), e se vai marcado coma texto pegado (`bracketed`): se o programa pediu o *bracketed paste*. **`new`** faino desde o texto do portapapeis (`clipboard`), co filtro do Windows Terminal (`FilterStringForPaste`):

| Carácter | Que pasa |
|---|---|
| CR, LF e CR LF | Un CR, o que envía a tecla Intro: un LF só chegaría ao PSReadLine coma Ctrl+Intro. `after_cr` lembra se o anterior foi un CR, para que o LF de CR LF non engada outro. LF e CR, nesa orde, son dous fins de liña. |
| O tabulador | Queda. |
| Os outros de control: C0 (`0x00`–`0x1F`), DEL (`0x7F`) e C1 (`U+0080`–`U+009F`) | Desaparecen. Un `ESC [ 201 ~` dentro do texto perde o ESC, e o resto é texto: non remata o pegado antes de tempo. E algúns programas len os C1 coma secuencias: `U+009B` é CSI, coma `ESC [`. |
| O resto | Queda: letras, acentos, emoji… |

- Sen *bracketed paste*, o texto pasa antes por `trim`, que quita os brancos do final dunha soa liña (máis abaixo). Co *bracketed paste*, queda enteiro: decide o programa.
- `'\0'..='\x1f' | '\x7f'..='\u{9f}'` son dous rangos de caracteres nun patrón. Os brazos compróbanse en orde: o CR, o LF e o tabulador, que tamén están no primeiro rango, xa se trataron antes.
- `String::with_capacity(clipboard.len())`: o texto nunca medra, porque cada carácter queda, desaparece ou, un LF, pasa a ser un CR do mesmo tamaño. Unha reserva chega.

```rust
pub fn is_multiline(&self) -> bool {
    self.text.contains('\r')
}

pub fn is_empty(&self) -> bool {
    self.text.is_empty() && !self.bracketed
}

pub fn write_to(&self, out: &mut Vec<u8>) {
    if self.bracketed {
        out.extend_from_slice(b"\x1b[200~");
    }
    out.extend_from_slice(self.text.as_bytes());
    if self.bracketed {
        out.extend_from_slice(b"\x1b[201~");
    }
}
```

- **`is_multiline`**: se ten máis dunha liña, é dicir, algún CR. Sen *bracketed paste*, unha shell executa cada liña ao lela: entón meiga pregunta antes de pegar, nunha caixa de mensaxe coas primeiras liñas (`!paste.bracketed && paste.is_multiline()`, en [Código › meiga › `clipboard.rs`](meiga.md#cratesmeigasrcclipboardrs)). Unha soa liña co seu fin de liña non pregunta: sen *bracketed paste*, `trim` xa lle quitou o fin de liña.
- **`is_empty`**: se non hai nada que enviar. Co *bracketed paste*, un texto baleiro vai igual, só coas marcas: algúns programas, ao recibilo, len unha imaxe do portapapeis. Sen el, un texto baleiro, ou só con brancos, non envía nada.
- **`write_to`** engade a `out` o que le o programa: o texto, entre `ESC [ 200 ~` e `ESC [ 201 ~` co *bracketed paste*. O ConPTY pásalles as marcas tal cal aos programas que len a entrada coma un terminal, coma bash en WSL, e quítallas aos da consola de Windows, coma o PSReadLine, que reciben o texto coma teclas ([ADR 0015](../adr/0015-o-rato-a-seleccion-e-o-portapapeis.md)).

```rust
fn trim(text: &str) -> &str {
    let trimmed = text.trim_end_matches(['\t', '\n', '\x0b', '\x0c', '\r', ' ']);
    if trimmed.contains(['\n', '\r']) {
        text
    } else {
        trimmed
    }
}
```

- **`trim`** quita os brancos do final (o tabulador, LF, VT, FF, CR e o espazo) só se o texto ten unha liña: se despois de quitalos queda algún fin de liña, ten varias, e devólveo como estaba. Os brancos do principio quedan.
- Devolve un anaco do mesmo texto (`&str`), sen copialo. `trim_end_matches` e `contains` aceptan coma patrón un *array* de caracteres, coma en [`shell.rs`](#cratesmeiga-coresrcshellrs).

**Probas.** Catro:

| Proba | Que comproba |
|---|---|
| `each_end_of_line_becomes_a_cr` | CR LF, LF e CR dan un CR cada un, e LF CR, dous; un texto con CR ten varias liñas, e un sen el, unha. |
| `control_characters_go_away_but_tabs_and_text_stay` | Co *bracketed paste*, un `ESC [ 201 ~` dentro do texto perde o ESC e non remata o pegado; BEL, DEL e os C1 `U+0085` e `U+009B` desaparecen; o tabulador, ñ, € e 😀 quedan. |
| `without_bracketed_paste_a_single_line_loses_the_blanks_at_its_end` | Sen *bracketed paste*, `  ls -l \t\r\n` queda en `  ls -l`, cos espazos do principio; unha liña só de brancos non envía nada; varias liñas quedan coma están, cun CR ao final de cada unha; e co *bracketed paste*, unha liña conserva os seus brancos e o seu fin de liña. |
| `an_empty_clipboard_only_goes_with_bracketed_paste` | Un portapapeis baleiro non envía nada sen *bracketed paste*, e só as marcas con el. |

## `crates/meiga-core/src/config.rs`

Desde a subfase 2.5: o arquivo de configuración, `%APPDATA%\meiga\config.toml`, co perfil por defecto e as entradas que cambian, ocultan ou engaden perfís ([ADR 0022](../adr/0022-os-perfis-e-a-configuracion.md), decisións 1, 2 e 3; [Arquitectura › Os perfís e a configuración](../arquitectura.md#os-perfís-e-a-configuración)). Desde a 2.6, tamén os atallos que engaden, cambian ou liberan os de meiga, na táboa `[shortcuts]` ([ADR 0023](../adr/0023-os-atallos-de-teclado.md), decisións 1 e 3; [Arquitectura › Os atallos de teclado](../arquitectura.md#os-atallos-de-teclado)). O arquivo é opcional, e meiga leo unha vez, ao arrancar: o programa ábreo e dálle os seus bytes a `parse` ([Código › meiga › `shells.rs`](meiga.md#cratesmeigasrcshellsrs)). Este módulo non abre ningún arquivo, só le bytes, así que se proba con textos.

```toml
default_profile = "Ubuntu"

[[profiles]]
name = "Command Prompt"
hidden = true

[[profiles]]
name = "Build"
command = 'pwsh -NoExit -File "%USERPROFILE%\build.ps1"'
directory = 'C:\src\meiga'

[shortcuts]
"ctrl+t" = "new_tab"
"ctrl+6" = "none"
```

Este arquivo, o do comentario do módulo (`//!`), fai de Ubuntu o perfil por defecto, oculta o Command Prompt e engade un perfil, «Build», que corre un script de PowerShell nun cartafol; e, desde a 2.6, fai de Ctrl+T outro atallo para abrir unha tab, ademais de Ctrl+Maiús+T, e libera Ctrl+6, que con meiga mostra a sexta tab, para que chegue outra vez ao programa ([`shortcut.rs`](#cratesmeiga-coresrcshortcutrs)). As cadeas entre comiñas simples van tal cal, sen escapes: así se escriben as rutas de Windows ([Tecnoloxías › Os perfís e a configuración](../tecnoloxias.md#57-os-perfís-e-a-configuración-toml-o-contorno-e-as-shells-de-windows)). As combinacións van entre comiñas: TOML non admite un `+` nunha chave sen elas.

**Un erro non para nada** ([ADR 0022](../adr/0022-os-perfis-e-a-configuracion.md), decisión 2; [ADR 0023](../adr/0023-os-atallos-de-teclado.md), decisión 3). meiga ignora o que está mal, e segue co resto:

| Que está mal | Que ignora meiga |
|---|---|
| O arquivo pasa de 1 MiB, non é texto UTF-8 ou non é TOML, coma con `hidden = tru` ou cunha chave repetida, tamén unha combinación escrita dúas veces exactamente igual | O arquivo enteiro, e usa os seus valores por defecto. |
| Un valor doutro tipo, coma `hidden = "yes"`, ou unha cadea baleira | Ese valor. |
| Unha chave que meiga non coñece, coma `colour` | Esa chave. |
| Unha entrada de `[[profiles]]` sen `name`, cun `name` mal ou co nome doutra | A entrada enteira. |
| Unha entrada de `[shortcuts]` cunha combinación mal, coma `"ctrl+a+b"`, ou cunha acción mal, coma `"quit"`, un número ou unha cadea baleira | Esa entrada. |
| Unha combinación que xa tiña unha entrada anterior, escrita doutra maneira, coma `"Ctrl+T"` despois de `"ctrl+t"` | A entrada seguinte: vale a primeira. |

Cada un é un `Problem`, coa súa liña e a súa columna, que a xanela mostra nunha caixa de mensaxe cando xa está na pantalla (decisión 8). Así, unha errata nunha entrada non desfai o resto do arquivo, ao contrario ca no Windows Terminal, que cun erro de sintaxe ou de tipo usa só os seus valores por defecto.

**Sen serde.** O crate `toml` adoita encher as estruturas dun programa con serde e `derive`, pero así a lectura para no primeiro erro, e o código ocupa máis: nun programa de proba, 193 KB, fronte aos 116 KB do parser só ([ADR 0022 › Alternativas consideradas](../adr/0022-os-perfis-e-a-configuracion.md#alternativas-consideradas)). meiga usa só o parser: `DeTable::parse` dá a táboa do arquivo, con cada chave e cada valor nun `Spanned`, que garda o rango de bytes do arquivo onde está; e o lector deste módulo, `Reader`, colle a man os valores que coñece, e anota un erro, co seu sitio, por cada cousa que ignora.

### `Position`, `Problem` e `Setting`

```rust
pub const MAX_SIZE: usize = 1 << 20;

const BOM: &[u8] = b"\xef\xbb\xbf";

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Position {
    pub line: u32,
    pub column: u32,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Problem {
    pub at: Option<Position>,
    pub message: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Setting<T> {
    pub value: T,
    pub at: Position,
}
```

- **`MAX_SIZE`**: o arquivo máis grande que le meiga, 1 MiB (1 048 576 bytes). Unha configuración ten uns centos de bytes: o límite só evita ler e analizar ao arrancar un arquivo enorme, que non sería unha configuración. É público: o programa le como moito un byte máis, que abonda para saber que o arquivo pasa do límite sen lelo enteiro.
- **`BOM`**: a marca de orde dos bytes de UTF-8 (*byte order mark*), `EF BB BF`, que algúns editores escriben ao principio dun arquivo.
- **`Position`**: un sitio do arquivo, a liña e a columna, as dúas desde 1, coma nos editores. A columna conta caracteres, e non bytes ([`position`](#a-liña-e-a-columna-position)). `PartialOrd` e `Ord`, derivados, comparan os campos na súa orde: primeiro a liña, e despois a columna. É `Copy`: dous `u32`.
- **`Problem`**: algo mal na configuración, que meiga ignora: onde está (`at`), se está nun sitio do arquivo, e a mensaxe, en inglés coma o resto da interface. `Problem::new` acepta coma mensaxe calquera texto que se converta nun `String` (`impl Into<String>`): un texto fixo ou un `format!`.
- **`Setting<T>`**: un valor do arquivo e o seu sitio. Só o garda `default_profile`: o seu erro chega despois, cando [`Profiles::merge`](#os-perfís-do-menú-profiles-e-merge) ve que non é un perfil visíbel, e necesita o sitio do valor.

```rust
impl fmt::Display for Problem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.at {
            Some(at) => write!(
                f,
                "Line {}, column {}: {}",
                at.line, at.column, self.message
            ),
            None => f.write_str(&self.message),
        }
    }
}
```

`Display` escribe un erro coma unha liña da caixa: `` Line 3, column 10: invalid boolean, expected `true` `` ou, se non ten sitio, só a mensaxe, coma `the file has more than 1 MiB` ([ADR 0022](../adr/0022-os-perfis-e-a-configuracion.md), decisión 8).

### `ProfileEntry` e `Config`

```rust
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProfileEntry {
    pub name: String,
    pub command: Option<String>,
    pub directory: Option<String>,
    pub hidden: bool,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Config {
    pub default_profile: Option<Setting<String>>,
    pub profiles: Vec<ProfileEntry>,
    pub shortcuts: Vec<shortcut::Entry>,
}
```

O que di o arquivo, coas chaves que coñece meiga ([ADR 0022](../adr/0022-os-perfis-e-a-configuracion.md), decisión 3; e, desde a 2.6, [ADR 0023](../adr/0023-os-atallos-de-teclado.md), decisión 1):

| Chave | Valor | Que é |
|---|---|---|
| `default_profile` | Unha cadea | O nome do perfil por defecto, o que abren o +, Ctrl+Maiús+T e a primeira tab ([ADR 0022](../adr/0022-os-perfis-e-a-configuracion.md), decisión 5). |
| `[[profiles]]` | Unha lista de táboas | As entradas que cambian, ocultan ou engaden perfís, na orde do arquivo: os `ProfileEntry`. |
| `name` | Unha cadea | O nome dun perfil que atopa meiga, para cambialo ou ocultalo, ou o dun perfil novo. Sen el, non hai entrada. |
| `command` | Unha cadea | A liña de comandos, coma a le o programa, coas variábeis `%NOME%`. Un perfil novo necesítaa. |
| `directory` | Unha cadea | O cartafol onde empeza, coas variábeis `%NOME%`; `~` é o persoal. |
| `hidden` | `true` ou `false` | Se o menú o deixa fóra. Sen el, `false`. |
| `[shortcuts]` | Unha táboa | Desde a 2.6: os atallos que engaden, cambian ou liberan os de meiga, cunha combinación de chave e o nome dunha acción, ou `"none"`, de valor. Son os [`shortcut::Entry`](#os-atallos-entry-e-shortcuts), na orde do arquivo, cada combinación unha vez ([`shortcuts`](#os-atallos-do-arquivo-shortcuts)). |

- Os textos gárdanse coma no arquivo: as variábeis expándeas `Profiles::merge`, co contorno de meiga, e `~` convérteo `Folder::of` ([`profile.rs`](#cratesmeiga-coresrcprofilers)). Os atallos, en cambio, gárdanse xa lidos, coma combinacións e accións: o programa só ten que xuntalos cos de meiga, con [`Shortcuts::new`](#os-atallos-entry-e-shortcuts).
- **`Config`** deriva `Default`: sen arquivo, ou cun arquivo que meiga ignora enteiro, a configuración é a baleira, sen perfil por defecto, entradas nin atallos.
- As táboas en liña tamén son entradas: `profiles = [{ name = "a" }, { name = "b", hidden = false }]` é o mesmo ca dúas `[[profiles]]`. Para o parser, os dous son unha lista de táboas. Igual cos atallos: `shortcuts = { "ctrl+shift+u" = "profile_4" }` é o mesmo ca unha táboa `[shortcuts]` con esa liña.
- Os atallos chegaron na 2.6, coa táboa `[shortcuts]`, e concretan a decisión 3 do ADR 0022. As cores e `max_fps` chegarán na 8.4 (decisión 3): ata entón, son chaves que meiga non coñece.

### Ler o arquivo: `parse`

```rust
pub fn parse(bytes: &[u8]) -> Result<(Config, Vec<Problem>), Problem> {
    if bytes.len() > MAX_SIZE {
        return Err(Problem::new(None, "the file has more than 1 MiB"));
    }
    let bytes = bytes.strip_prefix(BOM).unwrap_or(bytes);
    let text = str::from_utf8(bytes).map_err(|error| {
        // The text is valid up to where the error is.
        let valid = str::from_utf8(&bytes[..error.valid_up_to()]).unwrap_or_default();
        Problem::new(
            Some(position(valid, valid.len())),
            "the file is not UTF-8 text",
        )
    })?;
    let table = DeTable::parse(text).map_err(|error| {
        let at = error.span().map(|span| position(text, span.start));
        Problem::new(at, error.message())
    })?;
    let mut reader = Reader {
        text,
        problems: Vec::new(),
    };
    let config = reader.config(table.get_ref());
    let mut problems = reader.problems;
    sort(&mut problems);
    Ok((config, problems))
}
```

**`parse`** le a configuración en `bytes`, o contido do arquivo. Devolve a configuración, cos erros do que ignorou, na orde do arquivo; ou, nun `Err`, o erro que fai ignorala enteira. Paso a paso:

1. **O tamaño.** Con máis de `MAX_SIZE` bytes, o arquivo ignórase enteiro, cun erro sen sitio. Mírase primeiro, antes de percorrer o texto.
2. **A marca.** Se o arquivo empeza polo BOM, quítase (`strip_prefix`): con ela ou sen ela, é o mesmo arquivo, e as columnas da primeira liña cóntanse sen ela.
3. **UTF-8.** `str::from_utf8` comproba que o arquivo é texto UTF-8. Se non o é, o erro di onde: `valid_up_to` dá cantos bytes do principio son válidos, e o erro vai ao final dese texto, no primeiro byte que non vale. Ese texto sempre é válido, así que `unwrap_or_default` non actúa nunca: só evita un `unwrap`.
4. **TOML.** `DeTable::parse` le a táboa do texto. Un erro de sintaxe, coma `hidden = tru`, ou unha chave repetida, fai ignorar o arquivo enteiro, coa mensaxe do parser (`message`) e o sitio onde empeza o erro (`span`), se o dá. `get_ref` dá a táboa que garda o `Spanned`.
5. **Os valores.** O `Reader` percorre a táboa, colle o que coñece e anota os erros do que ignora.
6. **A orde.** [`sort`](#os-erros-na-orde-do-arquivo-sort) pon os erros na orde do arquivo.

- `map_err` converte os erros de `std` e de `toml` nun `Problem`, e o `?` devólveo no momento, coma o `Err` de `parse`.
- O programa mostra nunha caixa os erros, ou o erro que fai ignorar o arquivo, e di que fixo: ignoralos, ou ignorar o arquivo, e usar os seus valores por defecto ([Código › meiga › `shells.rs`](meiga.md#cratesmeigasrcshellsrs)).

### Os erros na orde do arquivo: `sort`

```rust
pub fn sort(problems: &mut [Problem]) {
    let key = |problem: &Problem| (problem.at.is_none(), problem.at);
    insertion_sort_by(problems, |a, b| key(a).cmp(&key(b)));
}
```

**`sort`** pon os erros na orde do arquivo: polo seu sitio, e os que non teñen ningún, ao final.

- **Por que hai que ordenalos.** A táboa que dá `DeTable::parse` é un `Map` de `toml`, que é un `BTreeMap` sen a *feature* `preserve_order`, e meiga non a activa: dá as chaves na orde dos seus nomes, e non na do arquivo. Así, `colour` vai antes ca `default_profile`, aínda que estea despois no arquivo, e, nunha entrada, `command` vai antes ca `name`. O lector atopa os erros nesa orde. E desde a 2.6, cando a orde decide algo, ordena el antes: as entradas de `[shortcuts]`, das que vale a primeira de cada combinación ([`shortcuts`](#os-atallos-do-arquivo-shortcuts)).
- **A chave** de cada erro é `(problem.at.is_none(), problem.at)`: primeiro os que teñen sitio, porque `false` vai antes ca `true`, pola liña e a columna; despois, os que non teñen, coma `all the profiles are hidden, so meiga shows them all`. Dous `Option` compáranse polos seus valores, e dous `Position`, pola liña e despois pola columna.
- **Estábel**: dous erros no mesmo sitio quedan na orde na que se atoparon ([`sort.rs`](#cratesmeiga-coresrcsortrs)).
- É pública: o programa xunta os erros do arquivo cos de `Profiles::merge`, e ordénaos outra vez con ela. Unha soa función para os dous sitios compila unha soa copia da ordenación ([ADR 0022](../adr/0022-os-perfis-e-a-configuracion.md), decisión 12).

### O lector: `Reader`

```rust
struct Reader<'a> {
    text: &'a str,
    problems: Vec<Problem>,
}

type Key<'i> = Spanned<std::borrow::Cow<'i, str>>;
type Value<'i> = Spanned<DeValue<'i>>;
```

- **`Reader`**, privado: o texto do arquivo, para dar o sitio de cada erro, e os erros que anota.
- **`Key`** e **`Value`**: unha chave e un valor da táboa, cada un nun `Spanned`: `get_ref` dá a chave ou o valor, e `span`, o rango de bytes do arquivo onde está. A chave é un `Cow<str>`, que pode ser un anaco do propio texto, sen copialo, coma os textos dos menús ([`Entry`](#entry-os-elementos-e-as-liñas)). `DeValue` é un valor de TOML: unha cadea, un número, un booleano, unha data, unha lista ou unha táboa.

```rust
fn config(&mut self, table: &DeTable<'_>) -> Config {
    let mut config = Config::default();
    for (key, value) in table {
        match key.get_ref().as_ref() {
            "default_profile" => {
                config.default_profile = self.string(key, value).map(|name| Setting {
                    value: name,
                    at: self.at(value.span()),
                });
            }
            "profiles" => config.profiles = self.profiles(key, value),
            "shortcuts" => config.shortcuts = self.shortcuts(key, value),
            _ => self.unknown(key),
        }
    }
    config
}
```

- **`config`**: a táboa principal do arquivo. Cada chave que coñece vai ao seu campo, e calquera outra é un erro no sitio da chave: `` unknown key `colour` ``. Desde a 2.6, `shortcuts` vai a `config.shortcuts`, que le [`shortcuts`](#os-atallos-do-arquivo-shortcuts); ata entón, era unha chave que meiga non coñecía.
- `default_profile` garda o seu valor co sitio do valor, e non co da chave, coma os erros dos valores: en `default_profile = "Ubuntu"`, a columna 19.

```rust
fn profiles(&mut self, key: &Key<'_>, value: &Value<'_>) -> Vec<ProfileEntry> {
    let DeValue::Array(array) = value.get_ref() else {
        self.problem(
            value.span(),
            format!(
                "`{}` has to be an array of tables ([[profiles]]), not {}",
                key.get_ref(),
                kind(value.get_ref())
            ),
        );
        return Vec::new();
    };
    let mut entries = Vec::<ProfileEntry>::with_capacity(array.len());
    for item in array {
        let DeValue::Table(table) = item.get_ref() else {
            self.problem(
                item.span(),
                format!("a profile has to be a table, not {}", kind(item.get_ref())),
            );
            continue;
        };
        let Some((entry, name)) = self.profile(table, item.span()) else {
            continue;
        };
        if entries.iter().any(|other| other.name == entry.name) {
            self.problem(
                name,
                format!("another profile is already named `{}`", entry.name),
            );
        } else {
            entries.push(entry);
        }
    }
    entries
}
```

**`profiles`**: as entradas de `[[profiles]]`.

1. `profiles` ten que ser unha lista. Se non o é, coma en `profiles = { name = "a" }`, é un erro, e non hai ningunha entrada. `let … else` colle a lista co patrón, ou sae da función.
2. Cada elemento ten que ser unha táboa. Un que non o é, coma en `profiles = ["cmd"]`, é un erro, e sáltase.
3. `profile` le a táboa. Sen nome, non hai entrada.
4. **Un nome repetido.** Se outra entrada xa ten ese nome, esta ignórase enteira, cun erro no seu nome: `` another profile is already named `a` ``. A primeira queda. Os nomes distinguen maiúsculas, coma ao buscar os perfís: `a` e `A` son dous. Así, cada perfil recibe como moito unha entrada, e non dúas que se contradín.

- `with_capacity(array.len())` reserva unha vez para todas as entradas. Mirar o nome nas anteriores percorre a lista, con `any`: con poucas entradas, nada.

```rust
fn profile(
    &mut self,
    table: &DeTable<'_>,
    span: Range<usize>,
) -> Option<(ProfileEntry, Range<usize>)> {
    let mut name = None;
    let mut named = false;
    let (mut command, mut directory, mut hidden) = (None, None, false);
    for (key, value) in table {
        match key.get_ref().as_ref() {
            "name" => {
                named = true;
                name = self.string(key, value).map(|name| (name, value.span()));
            }
            "command" => command = self.string(key, value),
            "directory" => directory = self.string(key, value),
            "hidden" => hidden = self.boolean(key, value).unwrap_or(hidden),
            _ => self.unknown(key),
        }
    }
    // A name that is wrong already has its problem.
    if !named {
        self.problem(span, "a profile needs a `name`");
    }
    let (name, at) = name?;
    Some((
        ProfileEntry {
            name,
            command,
            directory,
            hidden,
        },
        at,
    ))
}
```

**`profile`**: unha entrada, que ocupa `span` no arquivo. Devolve a entrada e o sitio do seu nome, para o erro dun nome repetido; ou `None`, sen nome.

- Cada chave que coñece garda o seu valor, se está ben. Un valor mal só se perde el: `command = ""` deixa a entrada sen `command`, e `hidden = "yes"`, con `hidden` en `false`, o valor que tiña (`unwrap_or(hidden)`).
- **Sen nome.** `named` distingue as dúas maneiras de non ter nome. Sen a chave `name`, o erro vai no sitio da entrada, que nunha `[[profiles]]` é a súa cabeceira: `` Line 1, column 1: a profile needs a `name` ``. Cun `name` mal, coma `name = 7`, `string` xa anotou o seu erro, e non hai outro: o comentario lémbrao. Nos dous casos, `name?` devolve `None`, e a entrada enteira ignórase: sen nome, non se sabe que perfil cambia.

```rust
fn string(&mut self, key: &Key<'_>, value: &Value<'_>) -> Option<String> {
    match value.get_ref() {
        DeValue::String(text) if text.is_empty() => {
            self.problem(value.span(), format!("`{}` cannot be empty", key.get_ref()));
            None
        }
        DeValue::String(text) => Some(text.to_string()),
        other => {
            self.problem(
                value.span(),
                format!(
                    "`{}` has to be a string, not {}",
                    key.get_ref(),
                    kind(other)
                ),
            );
            None
        }
    }
}

fn unknown(&mut self, key: &Key<'_>) {
    self.problem(key.span(), format!("unknown key `{}`", key.get_ref()));
}

fn problem(&mut self, span: Range<usize>, message: impl Into<String>) {
    let at = self.at(span);
    self.problems.push(Problem::new(Some(at), message));
}

fn at(&self, span: Range<usize>) -> Position {
    position(self.text, span.start)
}
```

- **`string`**: o texto dun valor, se é unha cadea que non está baleira. Unha cadea baleira non é un nome, nin unha liña de comandos, nin un cartafol: é un erro, `` `command` cannot be empty ``, coma un valor doutro tipo, `` `directory` has to be a string, not an array ``. `to_string` copia o texto nun `String`, que xa non depende do arquivo.
- **`boolean`** fai o mesmo con `true` e `false`: `` `hidden` has to be true or false, not a string ``.
- **`unknown`**: o erro dunha chave que meiga non coñece, no sitio da chave.
- **`problem`** anota un erro no comezo de `span`, e **`at`** dá ese sitio.

```rust
const fn kind(value: &DeValue<'_>) -> &'static str {
    match value {
        DeValue::String(_) => "a string",
        DeValue::Integer(_) => "an integer",
        DeValue::Float(_) => "a float",
        DeValue::Boolean(_) => "a boolean",
        DeValue::Datetime(_) => "a date",
        DeValue::Array(_) => "an array",
        DeValue::Table(_) => "a table",
    }
}
```

**`kind`**: que é un valor, coma o din os erros (`not an integer`), cunha frase para cada tipo de TOML. É unha `const fn`.

### Os atallos do arquivo: `shortcuts`

Desde a subfase 2.6: as entradas de `[shortcuts]`, cada unha cunha combinación de chave e o nome dunha acción de valor ([ADR 0023](../adr/0023-os-atallos-de-teclado.md), decisións 1 e 3). Este método do lector le as combinacións e as accións con [`shortcut.rs`](#cratesmeiga-coresrcshortcutrs), e deixa como moito unha entrada por combinación.

```rust
fn shortcuts(&mut self, key: &Key<'_>, value: &Value<'_>) -> Vec<shortcut::Entry> {
    let DeValue::Table(table) = value.get_ref() else {
        self.problem(
            value.span(),
            format!(
                "`{}` has to be a table ([shortcuts]), not {}",
                key.get_ref(),
                kind(value.get_ref())
            ),
        );
        return Vec::new();
    };
    // The table gives its keys in the order of their text: the entries
    // go in the order of the file, so that the first one of a chord
    // counts.
    let mut items: Vec<(&Key<'_>, &Value<'_>)> = table.iter().collect();
    insertion_sort_by(&mut items, |a, b| a.0.span().start.cmp(&b.0.span().start));
    let mut entries = Vec::<(shortcut::Entry, u32)>::with_capacity(items.len());
```

1. **Unha táboa.** `shortcuts` ten que ser unha táboa: `[shortcuts]` ou, en liña, `shortcuts = { "ctrl+shift+u" = "profile_4" }`. Se non o é, coma en `shortcuts = ["ctrl+t"]`, é un erro no sitio do valor, `` `shortcuts` has to be a table ([shortcuts]), not an array ``, e non hai ningún atallo.
2. **A orde do arquivo.** A táboa dá as súas chaves na orde dos seus textos, coma di o comentario ([`sort`](#os-erros-na-orde-do-arquivo-sort)): `"Ctrl+T"` iría antes ca `"alt+a"` e ca `"ctrl+t"`, porque as maiúsculas van antes ca as minúsculas. `items` garda cada entrada coma un par de referencias, á chave e ao valor, e [`insertion_sort_by`](#cratesmeiga-coresrcsortrs) ordénaas polo byte onde empeza cada chave, `span().start`: así quedan na orde do arquivo, e vale a primeira entrada de cada combinación.
3. **`entries`** gardará as entradas que están ben, cada unha coa liña da súa chave, para o erro das que a repitan. `with_capacity` reserva unha vez para todas.

```rust
    for (key, value) in items {
        let text = key.get_ref();
        let chord = match Chord::parse(text) {
            Ok(chord) => chord,
            Err(_) if text.trim().is_empty() => {
                self.problem(key.span(), "a shortcut cannot be empty");
                continue;
            }
            Err(error) => {
                self.problem(key.span(), format!("`{text}` is not a shortcut: {error}"));
                continue;
            }
        };
        let Some(name) = self.string(key, value) else {
            continue;
        };
        let action = match Action::parse(&name) {
            Ok(action) => action,
            Err(error) => {
                self.problem(value.span(), format!("`{name}` {error}"));
                continue;
            }
        };
        if let Some((_, line)) = entries.iter().find(|(entry, _)| entry.chord == chord) {
            let message = format!("`{text}` repeats the shortcut of line {line}");
            self.problem(key.span(), message);
            continue;
        }
        let line = self.at(key.span()).line;
        entries.push((shortcut::Entry { chord, action }, line));
    }
    entries.into_iter().map(|(entry, _)| entry).collect()
}
```

4. **A combinación.** [`Chord::parse`](#ler-unha-combinación-chordparse) le a chave. Se está mal, a entrada ignórase, cun erro no sitio da chave: se a chave está baleira, ou só ten brancos, `a shortcut cannot be empty`; se non, o texto da chave e o motivo ([`ChordError`](#por-que-non-é-unha-combinación-chorderror)), coma `` `ctrl+a+b` is not a shortcut: it has more than one key ``.
5. **O nome da acción.** O valor ten que ser unha cadea que non está baleira ([`string`](#o-lector-reader)), co erro de sempre no sitio do valor, que eiquí nomea a combinación: `` `ctrl+e` has to be a string, not an integer ``.
6. **A acción.** [`Action::parse`](#os-nomes-das-accións-actions-none-e-actionparse) le o nome. Se está mal, o erro vai no sitio do valor, co nome e o motivo ([`ActionError`](#por-que-non-é-unha-acción-actionerror)): `` `quit` is not an action ``. `none` está ben: é unha entrada sen acción, que libera a combinación.
7. **Unha combinación repetida.** Se unha entrada anterior xa ten a mesma combinación, esta ignórase, cun erro no sitio da súa chave que di a liña da primeira: `` `Ctrl+T` repeats the shortcut of line 2 ``. Dúas combinacións son a mesma se teñen os mesmos modificadores e a mesma tecla, escritas como sexa: `"ctrl+t"`, `"Ctrl+T"` e `"T + CTRL"`.
8. Se todo está ben, a entrada gárdase, coa liña da súa chave, que dá `at`.
9. **As entradas**, sen as liñas: `into_iter` e `map` quitan o número a cada unha, e `collect` fai o vector.

- **Só se ignora a entrada mal** (decisión 3), coma nos perfís, e cada entrada dá como moito un erro, o primeiro: unha combinación mal cunha acción mal só dá o da combinación.
- **Só as entradas que están ben contan para as repeticións.** Unha combinación cunha acción mal, coma `"ctrl+y" = "nwe_tab"`, non fai repetida a seguinte, `"Ctrl+Y" = "duplicate_tab"`, que vale. As entradas con `none` si contan.
- **Escrita exactamente igual**, unha combinación dúas veces son a mesma chave para TOML, e `DeTable::parse` falla: o arquivo enteiro ignórase ([`parse`](#ler-o-arquivo-parse)), coma con calquera chave repetida.
- **Vale a primeira**, coma cun perfil co nome doutro ([ADR 0022](../adr/0022-os-perfis-e-a-configuracion.md), decisión 2). `find` percorre as entradas anteriores: con poucas, nada.
- Os atallos non se aplican eiquí: `Config::shortcuts` garda as entradas, e o programa pono por riba dos de meiga con [`Shortcuts::new`](#os-atallos-entry-e-shortcuts), que, con cada combinación unha vez, non ten que decidir cal vale.

### A liña e a columna: `position`

```rust
fn position(text: &str, offset: usize) -> Position {
    let mut offset = offset.min(text.len());
    while !text.is_char_boundary(offset) {
        offset -= 1;
    }
    let before = &text[..offset];
    let start = before.rfind('\n').map_or(0, |newline| newline + 1);
    let line = before.bytes().filter(|&byte| byte == b'\n').count() + 1;
    let column = before[start..].chars().count() + 1;
    Position {
        line: u32::try_from(line).unwrap_or(u32::MAX),
        column: u32::try_from(column).unwrap_or(u32::MAX),
    }
}
```

**`position`** dá a liña e a columna do byte `offset` de `text`. O parser e `from_utf8` contan en bytes, e os editores, en liñas e caracteres:

1. O byte queda dentro do texto (`min`) e, se cae no medio dun carácter de varios bytes, recúa ata o seu comezo: así, `&text[..offset]` nunca corta un carácter, que sería un *panic*.
2. A liña: os LF de antes, máis 1. Un arquivo con CR LF conta igual: o CR queda ao final da liña anterior.
3. A columna: os caracteres desde o último LF, máis 1. `chars().count()` conta caracteres, e non bytes: `Ñ` ocupa dous bytes en UTF-8, pero é unha columna, coma nos editores ([ADR 0022](../adr/0022-os-perfis-e-a-configuracion.md), decisión 2).

- Cada erro percorre o texto ata o seu sitio, sen gardar onde empeza cada liña: cun arquivo pequeno e uns poucos erros, abonda.
- `try_from` deixa a liña e a columna en `u32::MAX` se non cupesen nun `u32`, en vez de cortalas; con 1 MiB, nunca pasa.

**Probas.** Oito. `read` le un texto que ten que ser unha configuración, e dá os seus erros coma texto; `fails`, o erro dun arquivo que meiga ignora enteiro; `entry`, unha entrada só co nome; e, desde a 2.6, `shortcut`, unha entrada de `[shortcuts]`, cunha combinación que ten que estar ben e unha acción.

| Proba | Que comproba |
|---|---|
| `an_empty_file_says_nothing` | Un arquivo baleiro, e un só cun comentario e CR LF, dan a configuración por defecto, sen erros. |
| `it_takes_the_default_profile_and_the_profiles_in_their_order` | O arquivo do exemplo, sen os atallos: `Ubuntu`, na liña 1, columna 19, e as dúas entradas na súa orde, coas comiñas e as barras invertidas das cadeas entre comiñas simples tal cal, e ningún atallo. E as táboas en liña, `profiles = [{ name = "a" }, { name = "b", hidden = false }]`, son dúas entradas. |
| `wrong_values_and_unknown_keys_are_ignored_with_their_place` | Nun arquivo con CR LF, un `default_profile` que é un número, unha chave `colour`, un `command` baleiro, un `hidden = "yes"`, unha chave `shortcut` e un `directory` que é unha lista dan seis erros, na orde do arquivo, cada un co seu sitio: o do valor nos valores, e o da chave nas chaves. O que está ben queda: as dúas entradas, sen eses valores, e sen perfil por defecto. |
| `a_profile_needs_a_name_of_its_own` | Unha entrada sen `name` dá o erro na súa cabeceira, `Line 1, column 1`; unha con `name = 7`, só o do tipo; e a segunda chamada `a`, `` another profile is already named `a` `` no seu nome, e ignórase, co seu `hidden`. `profiles = { name = "a" }`, unha táboa, non é unha lista de táboas, e `profiles2`, na liña seguinte, é unha chave que meiga non coñece; e `profiles = ["cmd"]` ten un elemento que non é unha táboa. |
| `shortcuts_go_in_the_order_of_the_file_and_the_first_of_a_chord_counts` | Desde a 2.6. `"ctrl+t" = "new_tab"`, `"ctrl+6" = "none"` e `"alt+a" = "tab_2"` quedan na orde do arquivo, con `NewTab`, sen acción e con `Tab(1)`; e `"Ctrl+T"`, despois, que na orde dos textos iría primeiro, é un erro: `` Line 5, column 1: `Ctrl+T` repeats the shortcut of line 2 ``. Unha táboa en liña, `shortcuts = { "ctrl+shift+u" = "profile_4" }`, tamén vale, con `Profile(3)`; e o mesmo texto dúas veces non é TOML: o arquivo enteiro ignórase, cun erro na liña 3, columna 1. |
| `wrong_shortcuts_are_ignored_with_their_place` | Desde a 2.6. Nove erros, na orde do arquivo: catro na columna 1, a da combinación, `ctrl+a+b`, con dúas teclas, unha chave baleira, `win+e` e `alt+f4`; e cinco na columna 12, a da acción, `quit`, `tab_0`, un número, unha cadea baleira e `nwe_tab`. Só queda `"Ctrl+Y" = "duplicate_tab"`, despois de `"ctrl+y" = "nwe_tab"`: vale a primeira entrada dunha combinación que está ben. E `shortcuts = ["ctrl+t"]`, unha lista, dá un erro na columna 13, e ningún atallo. |
| `a_file_that_is_not_toml_is_ignored_whole` | `hidden = tru` dá o erro do parser, `` Line 3, column 10: invalid boolean, expected `true` ``; unha cadea entre comiñas simples que non se pecha despois dun `Ñ`, o erro ao final da liña, na columna 21, que en bytes sería a 22; e unha chave repetida, un erro na liña 2, columna 1. |
| `the_file_is_utf8_with_or_without_its_mark_and_not_too_large` | Co BOM, o arquivo lese igual, e `default_profile = "ñ"` ten o valor na columna 19; un byte que non é UTF-8 (`E1`), `Line 2, column 20: the file is not UTF-8 text`; e un arquivo de `MAX_SIZE + 1` bytes, `the file has more than 1 MiB`, pero un de `MAX_SIZE`, non. |

**Como se comproba.** Ademais destas probas, `check-profiles.ps1` dálle a meiga na CI un `config.toml` con dous erros, unha chave que non coñece e un `hidden` que non é `true` nin `false`, e comproba a caixa que os mostra, coa liña e a columna de cada un; e despois, un arquivo que non é TOML ([Código › ferramentas](ferramentas.md#toolscheck-profilesps1)). Unha copia de meiga estragada a propósito que conta as columnas en bytes pasa a comprobación, porque o seu `config.toml` só ten ASCII antes dos seus erros, pero falla `a_file_that_is_not_toml_is_ignored_whole`; e unha que as conta desde 0 falla as dúas.

Desde a 2.6, `check-shortcuts.ps1` dálle a meiga na CI un `config.toml` cunha táboa `[shortcuts]` con catro erros: unha combinación con dúas teclas, `"ctrl+t+w"`; unha acción que non existe, `open_explorer`; `profile_0`; e `"Alt+O"`, despois de `"alt+o"`. Comproba a caixa que os mostra, cada un coa súa liña e a súa columna, a da combinación ou a da acción; e, despois, que Alt+O pecha as outras tabs, coma di a súa primeira liña, e non a tab que se mostra, coma diría a segunda ([Código › ferramentas › `check-shortcuts.ps1`](ferramentas.md#toolscheck-shortcutsps1)). As copias de meiga estragadas a propósito que non len `[shortcuts]`, que non anotan o erro dunha combinación, que poñen o erro dunha acción no sitio da combinación, ou nas que vale a última entrada dunha combinación, fallan a comprobación.

## `crates/meiga-core/src/profile.rs`

Desde a subfase 2.5: os perfís das tabs ([ADR 0022](../adr/0022-os-perfis-e-a-configuracion.md); [Arquitectura › Os perfís e a configuración](../arquitectura.md#os-perfís-e-a-configuración)). Un perfil é unha forma de abrir unha tab: un nome, que mostran o menú dos perfís e a tab, ata que o seu programa pon un título; unha liña de comandos; e o cartafol onde empeza, o persoal se a configuración non di outro ([Tecnoloxías › Os perfís e a configuración](../tecnoloxias.md#57-os-perfís-e-a-configuración-toml-o-contorno-e-as-shells-de-windows)). Este módulo xunta as shells que atopa meiga, que lle dá `meiga-win` ([Código › meiga-win › `profiles.rs`](meiga-win.md#cratesmeiga-winsrcprofilesrs)), co que cambia, oculta ou engade o `config.toml` ([`config.rs`](#cratesmeiga-coresrcconfigrs)); escolle o perfil por defecto; e di como empeza unha sesión cada perfil, cos de WSL no cartafol persoal de Linux. Non le o rexistro nin mira ningún arquivo: recibe as shells xa atopadas, así que se proba coas dun equipo inventado.

### Os nomes dos perfís

```rust
pub const POWERSHELL: &str = "PowerShell";
pub const WINDOWS_POWERSHELL: &str = "Windows PowerShell";
pub const COMMAND_PROMPT: &str = "Command Prompt";
pub const GIT_BASH: &str = "Git Bash";
```

Os nomes dos perfís que atopa meiga, os mesmos do Windows Terminal, na orde do menú; as distribucións de WSL, entre Command Prompt e Git Bash, levan o seu propio, coma `Ubuntu` ([ADR 0022](../adr/0022-os-perfis-e-a-configuracion.md), decisión 4). `profiles::detect` dállos aos seus perfís, e [`merge`](#os-perfís-do-menú-profiles-e-merge) busca con eles o perfil por defecto. Son os nomes que se escriben no `config.toml` para cambiar ou ocultar un deses perfís: `name = "Command Prompt"`.

### `Folder`, `Profile` e `Launch`

```rust
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Folder {
    Home,
    Path(String),
}

impl Folder {
    pub fn of(directory: String) -> Self {
        if directory == "~" {
            Self::Home
        } else {
            Self::Path(directory)
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Profile {
    pub name: String,
    pub command: String,
    pub folder: Folder,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Launch {
    pub name: String,
    pub command: String,
    pub folder: Option<String>,
}
```

| Tipo | Que é |
|---|---|
| `Folder` | Onde empeza un perfil: `Home`, o cartafol persoal, que nos de WSL é o de Linux, e nos outros, o do usuario en Windows, `%USERPROFILE%`; ou `Path`, un cartafol de Windows ou, nos de WSL, calquera que acepte `wsl.exe --cd`. |
| `Profile` | Un perfil do menú: o seu nome; a liña de comandos, co programa e os seus argumentos entre comiñas coma os le o programa; e o cartafol. |
| `Launch` | O que corre unha sesión: o nome que mostra a súa tab mentres o programa non pon un título, a liña de comandos, e o cartafol de Windows onde empeza, ou `None`, o de meiga. |

- **`Folder::of`** fai o cartafol dun `directory` do arquivo, xa coas variábeis expandidas: `~` é o persoal, e calquera outro texto, tamén `~/src`, é un `Path`. Move o texto ao `Path`, sen copialo.
- **Por que `Home`, e non a ruta.** O cartafol persoal depende do programa: nos de WSL é o de Linux, que só coñece `wsl.exe`, e nos outros, o de Windows. Un perfil di «o persoal», e [`launch`](#como-empeza-launch-e-command_line) decide cal.
- **Por que dous tipos.** `Profile` é o que mostra o menú e cambia a configuración; `Launch`, o que necesita a sesión, xa coa liña de comandos de WSL e un cartafol de Windows. Cada tab garda o seu `Launch`: a barra mostra o seu `name` mentres o programa non pon un título ([ADR 0022](../adr/0022-os-perfis-e-a-configuracion.md), decisión 11), e *Duplicate tab* lanza outra vez o mesmo, co mesmo nome, a mesma liña de comandos e o mesmo cartafol (decisión 10; [Código › meiga › `tabs.rs`](meiga.md#cratesmeigasrctabsrs)).

### Como empeza: `launch` e `command_line`

```rust
impl Profile {
    pub fn launch(&self, home: Option<&str>, system: &str) -> Launch {
        let command = wsl_command(&self.command, &self.folder, system);
        let folder = match (&self.folder, &command) {
            // WSL has the folder in its command line, and starts in the home
            // folder, which no folder of Linux keeps in use.
            (_, Some(_)) | (Folder::Home, None) => home.map(str::to_owned),
            (Folder::Path(path), None) => Some(path.clone()),
        };
        Launch {
            name: self.name.clone(),
            command: command.unwrap_or_else(|| self.command.clone()),
            folder,
        }
    }
}
```

**`launch`** di como empeza unha sesión do perfil, con `home`, o cartafol persoal do usuario en Windows, se Windows dá un, e `system`, o cartafol do sistema, onde está `wsl.exe` ([ADR 0022](../adr/0022-os-perfis-e-a-configuracion.md), decisión 10):

| O perfil | A liña de comandos | O cartafol de Windows |
|---|---|---|
| O `wsl.exe` do sistema, sen `--cd` nin `~` | A súa, co cartafol do perfil coma `--cd`: en `Home`, `~`, o persoal de Linux ([`wsl_command`](#wsl-e-o-seu-cartafol-wsl_command)). | O persoal. |
| Calquera outro, en `Home` | A súa. | O persoal. |
| Calquera outro, nun `Path` | A súa. | Ese cartafol. |

- **WSL, co cartafol na liña de comandos.** `CreateProcessW` só acepta cartafoles de Windows, e o persoal de Linux só o coñece `wsl.exe`. Así que o cartafol vai coma `--cd`, e o proceso de `wsl.exe` empeza no persoal de Windows: coma di o comentario, así non deixa en uso ningún cartafol de Linux.
- Un `wsl.exe` que xa leva `--cd` ou `~` é «calquera outro»: a súa liña de comandos queda, e empeza no cartafol de Windows do perfil.
- Sen `USERPROFILE`, `home` é `None`, e a sesión empeza no cartafol de meiga.
- `clone` copia o nome e a liña de comandos: o perfil queda no menú, e o `Launch` vai á tab. O programa chámao co perfil escollido no menú, e co por defecto para o +, Ctrl+Maiús+T e a primeira tab ([Código › meiga › `shells.rs`](meiga.md#cratesmeigasrcshellsrs)).

```rust
impl Launch {
    pub fn command_line(command: &str) -> Self {
        Self {
            name: shell::program_name(command).to_owned(),
            command: command.to_owned(),
            folder: None,
        }
    }
}
```

**`command_line`**: o `Launch` do programa da liña de comandos de meiga, que vai á primeira tab en vez do perfil por defecto (decisión 5). O seu nome é o do programa, coma ata a 2.4 o de todas as tabs ([`program_name`](#cratesmeiga-coresrcshellrs)), e empeza no cartafol de meiga, `None`, onde estaba o usuario ao lanzalo (decisión 10). Úsao o programa ao abrir a primeira tab ([Código › meiga › `app.rs`](meiga.md#cratesmeigasrcapprs)).

### Os perfís do menú: `Profiles` e `merge`

```rust
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Profiles {
    list: Vec<Profile>,
    default: usize,
}

impl Profiles {
    pub fn list(&self) -> &[Profile] {
        &self.list
    }

    pub const fn default_index(&self) -> usize {
        self.default
    }

    pub fn default_profile(&self) -> &Profile {
        &self.list[self.default]
    }
}
```

- **`list`**: os perfís visíbeis, na orde do menú. Nunca está baleira. O menú dos perfís ten un elemento por perfil, co seu nome, e escollelo abre unha tab con el ([Código › meiga › `tabs.rs`](meiga.md#cratesmeigasrctabsrs)). Desde a subfase 2.6, os atallos `profile_1`…`profile_9`, Ctrl+Maiús+1…9, abren os perfís deses sitios da lista ([`Action`](#as-accións-action)).
- **`default`**: a posición do perfil por defecto na lista. `default_index` dálla ao menú, que pon ese elemento en negra ([`Entry::bold`](#entry-os-elementos-e-as-liñas)), e `default_profile` dá o perfil, o que abren o +, Ctrl+Maiús+T e a primeira tab.
- Os campos son privados, e ningún método deixa a lista baleira nin o por defecto fóra dela: `default_profile` le `&self.list[self.default]` sen comprobar nada.

```rust
pub fn merge(
    detected: Vec<Profile>,
    config: &Config,
    environment: &Environment,
) -> (Self, Vec<Problem>) {
    let mut profiles: Vec<(Profile, bool)> = detected
        .into_iter()
        .map(|profile| (profile, false))
        .collect();
    for entry in &config.profiles {
        let command = entry
            .command
            .as_deref()
            .map(|text| environment.expand(text));
        let folder = entry
            .directory
            .as_deref()
            .map(|text| Folder::of(environment.expand(text)));
        if let Some((profile, hidden)) = profiles
            .iter_mut()
            .find(|(profile, _)| profile.name == entry.name)
        {
            if let Some(command) = command {
                profile.command = command;
            }
            if let Some(folder) = folder {
                profile.folder = folder;
            }
            *hidden = entry.hidden;
        } else if let Some(command) = command {
            let name = entry.name.clone();
            let folder = folder.unwrap_or(Folder::Home);
            profiles.push((
                Profile {
                    name,
                    command,
                    folder,
                },
                entry.hidden,
            ));
        }
    }
```

**`merge`** xunta os perfís que atopou meiga, `detected`, na súa orde, coas entradas do arquivo ([ADR 0022](../adr/0022-os-perfis-e-a-configuracion.md), decisión 3). Devolve os perfís e os erros que só se ven eiquí:

1. Cada perfil atopado vai cun `false`: non está oculto.
2. Para cada entrada, na orde do arquivo, `command` e `directory` expanden as súas variábeis `%NOME%` co contorno de meiga ([`expand`](#expandir-as-variábeis-expand)), e `directory` pasa a ser un `Folder`.
3. **Co nome dun perfil**, a entrada cambia a súa liña de comandos e o seu cartafol, se os leva, e decide se está oculto: unha entrada sen `hidden` déixao visíbel.
4. **Cun nome novo e `command`**, engade un perfil ao final, despois dos atopados, no persoal se non leva `directory`; tamén oculto, se o di.
5. **Cun nome novo e sen `command`**, nada, e sen erro: é unha entrada para outro equipo, que ten esa shell. Así, o mesmo arquivo vale nos dous.

- **Os nomes non se repiten.** `profiles::detect` non repite ningún, e `config::parse` ignora as entradas co nome doutra: cada entrada atopa como moito un perfil, e un perfil engadido nunca ten o nome doutro. Os nomes distinguen maiúsculas: unha entrada `ubuntu` non cambia `Ubuntu` e, se leva `command`, engade outro perfil.
- `iter_mut().find` busca o perfil, e dá referencias mutábeis ao perfil e ao seu `bool`, para cambialos no sitio.
- O `bool` vai xunto a cada perfil, nunha tupla, e non en `Profile`: só importa mentres se fai a lista, e os perfís que saen xa son os visíbeis.

```rust
    let mut problems = Vec::new();
    let mut list: Vec<Profile> = profiles
        .iter()
        .filter(|(_, hidden)| !hidden)
        .map(|(profile, _)| profile.clone())
        .collect();
    if list.is_empty() && !profiles.is_empty() {
        problems.push(Problem::new(
            None,
            "all the profiles are hidden, so meiga shows them all",
        ));
        list = profiles
            .iter()
            .map(|(profile, _)| profile.clone())
            .collect();
    }
    if list.is_empty() {
        // No shell was found: Command Prompt, which every Windows has.
        list.push(Profile {
            name: COMMAND_PROMPT.to_owned(),
            command: "cmd.exe".to_owned(),
            folder: Folder::Home,
        });
    }
```

6. **A lista**: os perfís que non están ocultos, na súa orde.
7. **Todos ocultos.** Un menú baleiro non abriría nada, e o + non tería perfil. É un erro, sen sitio, porque non está nun lugar do arquivo, e móstranse todos, coma se ningún estivese oculto (decisión 5).
8. **Ningún perfil.** Se meiga non atopou ningunha shell, nin o arquivo engadiu ningunha, Command Prompt, `cmd.exe`, que ten calquera Windows. Sen ruta: `CreateProcessW` búscao, tamén no cartafol do sistema.

```rust
    let index_of = |name: &str| list.iter().position(|profile| profile.name == name);
    let chosen = config.default_profile.as_ref().and_then(|setting| {
        let index = index_of(&setting.value);
        if index.is_none() {
            let name = &setting.value;
            let message = if profiles.iter().any(|(profile, _)| profile.name == *name) {
                format!("the profile `{name}` is hidden, so it cannot be the default one")
            } else {
                let names: Vec<&str> =
                    list.iter().map(|profile| profile.name.as_str()).collect();
                format!(
                    "there is no profile named `{name}`; the profiles are: {}",
                    names.join(", ")
                )
            };
            problems.push(Problem::new(Some(setting.at), message));
        }
        index
    });
    let default = chosen
        .or_else(|| index_of(POWERSHELL))
        .or_else(|| index_of(WINDOWS_POWERSHELL))
        .unwrap_or(0);
    (Self { list, default }, problems)
}
```

9. **O perfil por defecto** ([ADR 0022](../adr/0022-os-perfis-e-a-configuracion.md), decisión 5): o de `default_profile`, se está na lista; se non, PowerShell, Windows PowerShell ou o primeiro, coma no Windows Terminal. Concreta a decisión 12 do [ADR 0019](../adr/0019-varias-sesions-e-tabs-en-segundo-plano.md): as tabs novas corren o perfil por defecto, e non sempre PowerShell.
10. **Un `default_profile` que non está na lista** é un erro, no sitio do seu valor: se é un perfil oculto, `` the profile `Command Prompt` is hidden, so it cannot be the default one ``; e, se non existe, `` there is no profile named `ubuntu`; the profiles are: … ``, cos nomes dos visíbeis, para escoller un. Por iso `config::parse` garda o sitio deste valor nun [`Setting`](#position-problem-e-setting): o erro só se sabe eiquí, despois de xuntar os perfís.

- `and_then` só busca se o arquivo ten `default_profile`, e cada `or_else` só busca se o anterior non atopou nada.
- O programa xunta estes erros cos do arquivo, e móstraos na mesma caixa (decisión 8; [Código › meiga › `shells.rs`](meiga.md#cratesmeigasrcshellsrs)).
- **Cada vez que se abre o menú dos perfís** e, desde a subfase 2.6, cada vez que un atallo abre un perfil, coma Ctrl+Maiús+1 ([ADR 0023](../adr/0023-os-atallos-de-teclado.md), decisión 6), o programa atopa outra vez as shells, e volta chamar `merge` coa mesma configuración, para mostrar ou abrir as que se instalaron mentres tanto (decisión 4). Os erros, que xa mostrou ao arrancar, descártaos.

### WSL e o seu cartafol: `wsl_command`

```rust
fn wsl_command(command: &str, folder: &Folder, system: &str) -> Option<String> {
    // The program ends at the first space or quote after its first
    // character, which can be the opening quote.
    let start = usize::from(command.starts_with('"'));
    let end = command
        .get(1..)?
        .find([' ', '"'])
        .map_or(command.len(), |at| at + 1);
    let program = command.get(start..end)?;
    let (parent, file) = match program.rsplit_once(['\\', '/']) {
        Some((parent, file)) => (Some(parent), file),
        None => (None, program),
    };
    if !file.eq_ignore_ascii_case("wsl") && !file.eq_ignore_ascii_case("wsl.exe") {
        return None;
    }
    // Another program with that name.
    if parent.is_some_and(|parent| !parent.eq_ignore_ascii_case(system)) {
        return None;
    }
    let arguments = command.get(end + 1..).unwrap_or_default();
    if arguments.contains("--cd") {
        return None;
    }
    if let Some(tilde) = arguments.find('~')
        && arguments[tilde + 1..]
            .chars()
            .next()
            .is_none_or(|c| c == ' ')
    {
        return None;
    }
    let directory = match folder {
        Folder::Home => "~".to_owned(),
        // `wsl.exe --cd` takes these as folders of Linux.
        Folder::Path(path) if path.starts_with("//wsl$") || path.starts_with("//wsl.localhost") => {
            path.replace('/', "\\")
        }
        Folder::Path(path) => path.clone(),
    };
    Some(format!(r#""{program}" --cd "{directory}" {arguments}"#))
}
```

**`wsl_command`**, privada: a liña de comandos que empeza `command` en `folder`, se é o `wsl.exe` do cartafol do sistema, coma fai o Windows Terminal (`MangleStartingDirectoryForWSL`): co cartafol coma `--cd`, porque WSL pode empezar nun cartafol de Linux, e `CreateProcessW` non. `None` cos outros programas, e cun `wsl.exe` que xa leva `--cd`, ou `~` coma argumento, que empeza no persoal. Paso a paso:

1. **O programa**, desde o principio ata o primeiro espazo ou comiñas despois do primeiro carácter, que poden ser as comiñas que o abren; sen elas. `start` é 1 se empeza por comiñas, e `end`, onde remata. `get` dá `None`, e non un *panic*, cunha liña baleira. Nun programa entre comiñas, tamén o remata un espazo: unha ruta con espazos, coma `"C:\Program Files\…"`, queda cortada, pero tampouco sería a do cartafol do sistema.
2. **O arquivo**, despois da última barra, ten que chamarse `wsl` ou `wsl.exe`, en maiúsculas ou en minúsculas.
3. **O seu cartafol**, se o leva, ten que ser o do sistema, sen distinguir maiúsculas: `C:\tools\wsl.exe` é outro programa co mesmo nome. Sen cartafol, `wsl` e `wsl.exe` son o do sistema, onde os atopa `CreateProcessW`.
4. **Os argumentos**, despois do espazo ou das comiñas que rematan o programa. Se xa levan `--cd`, ou un `~` só, ao final ou antes dun espazo, quen escribiu a liña xa escolleu onde empeza, e queda como está. Só conta o primeiro `~`, coma no Windows Terminal: en `wsl ls ~/src`, `~/src` é un argumento, e non o persoal.
5. **O cartafol**: en `Home`, `~`; un que empeza por `//wsl$` ou `//wsl.localhost`, un cartafol de Linux visto desde Windows, coas barras invertidas, que é como o recoñece `wsl.exe --cd`; e calquera outro, tal cal: un de Windows, coma `C:\src`, ou un de Linux, coma `/tmp`.
6. **A liña nova**: o programa entre comiñas, `--cd` co cartafol entre comiñas, e os argumentos.

| Liña de comandos | Cartafol | Liña nova |
|---|---|---|
| `C:\WINDOWS\system32\wsl.exe -d Ubuntu` | `Home` | `"C:\WINDOWS\system32\wsl.exe" --cd "~" -d Ubuntu` |
| `wsl` | `C:\src` | `"wsl" --cd "C:\src" `, cun espazo ao final |
| `"c:\windows\SYSTEM32\WSL.EXE" -e bash` | `Home` | `"c:\windows\SYSTEM32\WSL.EXE" --cd "~"  -e bash`, con dous espazos antes de `-e` |
| `wsl.exe` | `//wsl$/Ubuntu/home` | `"wsl.exe" --cd "\\wsl$\Ubuntu\home" ` |
| `wsl ls ~/src` | `Home` | `"wsl" --cd "~" ls ~/src` |
| `cmd.exe /c wsl.exe`, `C:\tools\wsl.exe -d Ubuntu`, `wsl --cd /tmp`, `wsl ~` e `wsl ~ -d Ubuntu` | Calquera | A mesma: outro programa, un `wsl.exe` doutro cartafol, un que xa ten `--cd` e dous que xa empezan no persoal. |

- **Os espazos de máis.** Os argumentos empezan despois do carácter que remata o programa: cun programa entre comiñas, polo espazo que o segue, e a liña nova leva dous espazos seguidos; sen argumentos, remata nun espazo. Non cambian nada: os argumentos sepáranse con calquera número de espazos.
- `if let … && …` (unha cadea de `let`) mira o `~` e o carácter que o segue nunha soa condición: `is_none_or` é certo ao final do texto, onde non hai ningún, e cun espazo.
- `eq_ignore_ascii_case` compara sen distinguir maiúsculas só en ASCII: abonda para `wsl.exe` e para o cartafol do sistema.

**Probas.** Seis. `detected` dá os perfís dun equipo con todas as shells: PowerShell 7 en `%ProgramFiles%`, Windows PowerShell e Command Prompt no cartafol do sistema, Ubuntu e Git Bash; `entry`, unha entrada só co nome; `names`, os nomes da lista; `default_named`, un `default_profile` na liña 1, columna 19; e `environment`, un contorno só con `USERPROFILE`. As tres probas que fan unha configuración a man dan os campos que usan, e os outros con `..Config::default()`: desde a 2.6, `Config` ten tamén os atallos, que quedan baleiros.

| Proba | Que comproba |
|---|---|
| `without_a_configuration_powershell_is_the_default` | Sen configuración, os perfís son os atopados, sen erros, e o por defecto é PowerShell; sen el, Windows PowerShell, o primeiro; sen os dous, Command Prompt, tamén o primeiro; e, sen ningunha shell, a lista só ten Command Prompt, `cmd.exe`. |
| `the_configuration_changes_hides_and_adds_profiles` | Un arquivo oculta Command Prompt, dálle a Ubuntu un cartafol con `%USERPROFILE%`, engade «Build», cunha liña de comandos con `%USERPROFILE%`, e cambia a liña de comandos de PowerShell e o seu cartafol, `~`; e unha entrada «Debian», oculta e sen `command` nin perfil dese nome, ignórase sen erro. Os perfís son PowerShell, Windows PowerShell, Ubuntu, Git Bash e Build, coas variábeis expandidas, e o por defecto, Build. |
| `a_default_profile_that_is_hidden_or_not_there_is_a_problem` | Sen PowerShell e con Command Prompt oculto, un `default_profile` que é Command Prompt dá o erro do perfil oculto; e `ubuntu`, en minúsculas, o do perfil que non existe, cos nomes dos visíbeis: os nomes distinguen maiúsculas. Nos dous casos, o por defecto é Windows PowerShell. |
| `if_all_the_profiles_are_hidden_they_are_all_shown` | Cos cinco ocultos, un erro sen sitio, e móstranse todos, co por defecto, PowerShell. |
| `profiles_start_in_the_home_folder_or_in_theirs` | Un perfil en `Home` empeza no cartafol persoal, e un nun `Path`, nese cartafol; sen cartafol persoal, no de meiga. E o programa da liña de comandos de meiga, `"C:\tools\my app.exe" -x`, chámase `my app`, coa mesma liña de comandos, e empeza no cartafol de meiga. |
| `wsl_gets_its_folder_as_cd_as_in_windows_terminal` | Os casos da táboa de [`wsl_command`](#wsl-e-o-seu-cartafol-wsl_command), co cartafol de Windows de cada un: o persoal nos que reciben `--cd` e nos que quedan como están en `Home`, e `C:\src` en `wsl --cd /tmp` cun `Path`. |

**Como se comproba.** Ademais destas probas, `check-profiles.ps1` comproba na CI que a primeira tab corre o perfil por defecto do seu `config.toml`, coas variábeis da súa liña de comandos e do seu cartafol expandidas, nese cartafol; que o menú dos perfís non ten o perfil oculto; que cada perfil que atopa meiga corre a liña de comandos que espera, no cartafol persoal, e os de WSL, con `--cd "~"`; e que *Duplicate tab* repite a liña de comandos e o cartafol ([Código › ferramentas](ferramentas.md#toolscheck-profilesps1)). As copias de meiga estragadas a propósito que ignoran o cartafol, as variábeis, o perfil por defecto ou `hidden`, ou que lanzan WSL sen `--cd`, fallan a comprobación.

## `crates/meiga-core/src/environment.rs`

Desde a subfase 2.5: o contorno dos programas das tabs ([ADR 0022](../adr/0022-os-perfis-e-a-configuracion.md), decisión 9; [Arquitectura › Os perfís e a configuración](../arquitectura.md#os-perfís-e-a-configuración)): o de meiga, coma llo dá Windows ao arrancar, coas variábeis que lles din aos programas en que terminal corren, e sen as do Windows Terminal, que meiga herda cando se abre desde el. O programa fai o bloque unha vez, ao arrancar, e dállo a todas as sesións ([Código › meiga › `shells.rs`](meiga.md#cratesmeigasrcshellsrs)), e `Conpty::spawn` pásallo a `CreateProcessW` con `CREATE_UNICODE_ENVIRONMENT` ([Código › meiga-win › `conpty.rs`](meiga-win.md#cratesmeiga-winsrcconptyrs)). Ler o contorno de meiga, con `GetEnvironmentStringsW`, é traballo de `meiga-win` ([› `profiles.rs`](meiga-win.md#cratesmeiga-winsrcprofilesrs)): este módulo só traballa coas variábeis, así que se proba con contornos feitos a man.

**As variábeis de Windows** ([Tecnoloxías › Os perfís e a configuración](../tecnoloxias.md#57-os-perfís-e-a-configuración-toml-o-contorno-e-as-shells-de-windows)). Cada unha é un texto `NOME=valor`. Un nome pode ter calquera carácter menos `=` e o nulo, e un valor, calquera menos o nulo. Os nomes non distinguen maiúsculas, e poden empezar por `=`: Windows garda o cartafol actual de cada unidade en variábeis coma `=C:`, que usa `cmd`. meiga gárdaas coma as ten Windows, en UTF-16: así, cada variábel volta ao bloque tal cal, tamén se non é UTF-16 válido.

### As variábeis: `Environment`

```rust
pub const TERM_PROGRAM: &str = "TERM_PROGRAM";
pub const TERM_PROGRAM_VERSION: &str = "TERM_PROGRAM_VERSION";
pub const COLORTERM: &str = "COLORTERM";

const WINDOWS_TERMINAL: [&str; 2] = ["WT_SESSION", "WT_PROFILE_ID"];

const WSLENV: &str = "WSLENV";

const NAME: &str = "meiga";

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Environment {
    vars: Vec<Vec<u16>>,
}
```

| Constante | Que é |
|---|---|
| `TERM_PROGRAM`, `TERM_PROGRAM_VERSION` e `COLORTERM` | Os nomes das variábeis que lle din a un programa en que terminal corre, coma en VS Code e WezTerm: o seu nome, a súa versión, e que mostra cores de 24 bits. |
| `WINDOWS_TERMINAL` | As do Windows Terminal: un programa que as ve cre que corre nel. |
| `WSLENV` | A variábel que nomea as que WSL pasa de Windows aos programas de Linux ([`wslenv`](#o-contorno-dos-programas-prepare-e-wslenv)). |
| `NAME` | O valor de `TERM_PROGRAM`: `meiga`. |

**`Environment`** é un conxunto de variábeis, cada unha coma `NOME=valor`, en UTF-16 e sen o seu nulo, nun `Vec`. Deriva `Default`, un contorno baleiro, coma o das probas.

```rust
pub fn from_block(block: &[u16]) -> Self {
    let vars = block
        .split(|&unit| unit == 0)
        .take_while(|var| !var.is_empty())
        .map(<[u16]>::to_vec)
        .collect();
    Self { vars }
}

pub fn get(&self, name: &str) -> Option<String> {
    let var = &self.vars[self.position(name)?];
    let value = var.get(name_length(var) + 1..).unwrap_or_default();
    Some(String::from_utf16_lossy(value))
}

pub fn set(&mut self, name: &str, value: &str) {
    let var = name
        .encode_utf16()
        .chain(Some(u16::from(b'=')))
        .chain(value.encode_utf16())
        .collect();
    match self.position(name) {
        Some(index) => self.vars[index] = var,
        None => self.vars.push(var),
    }
}

pub fn remove(&mut self, name: &str) {
    if let Some(index) = self.position(name) {
        self.vars.remove(index);
    }
}
```

- **`from_block`**: as variábeis dun bloque coma o dá `GetEnvironmentStringsW`: cada unha remata nun nulo, e unha baleira remata o bloque. `split` parte o bloque nos nulos, `take_while` para na primeira variábel baleira, e `to_vec` copia cada unha, porque o bloque de Windows hai que soltalo despois.
- **`get`**: o valor dunha variábel, sen distinguir maiúsculas no nome: `PATH` atopa `Path`. O valor empeza despois do primeiro `=` que segue ao nome: en `x=a=b`, o de `x` é `a=b`. Un carácter que non é UTF-16 válido pasa a ser U+FFFD (`from_utf16_lossy`), porque un `String` sempre é texto válido. Unha variábel sen `=`, que Windows non dá, tería un valor baleiro (`unwrap_or_default`).
- **`set`**: pon `name=value` no sitio da variábel que ten ese nome, en maiúsculas ou en minúsculas, ou ao final, se non hai ningunha. O nome queda coma se lle dá: despois de `set("PATH", …)`, a variábel chámase `PATH`, e non `Path`. `encode_utf16` converte o texto, e `chain` xúntao co `=` e co valor, sen textos intermedios.
- **`remove`**: quita a variábel, se está.

### Expandir as variábeis: `expand`

```rust
pub fn expand(&self, text: &str) -> String {
    let mut expanded = String::with_capacity(text.len());
    let mut rest = text;
    while let Some(start) = rest.find('%') {
        expanded.push_str(&rest[..start]);
        let after = &rest[start + 1..];
        let Some(end) = after.find('%') else {
            rest = &rest[start..];
            break;
        };
        let name = &after[..end];
        let value = if name.is_empty() {
            None
        } else {
            self.get(name)
        };
        match value {
            Some(value) => {
                expanded.push_str(&value);
                rest = &after[end + 1..];
            }
            None => {
                expanded.push('%');
                rest = after;
            }
        }
    }
    expanded.push_str(rest);
    expanded
}
```

**`expand`** substitúe cada `%NOME%` de `text` polo valor da variábel, coma `ExpandEnvironmentStringsW`. Úsao `Profiles::merge` con `command` e con `directory` ([ADR 0022](../adr/0022-os-perfis-e-a-configuracion.md), decisión 3). Paso a paso:

1. Copia o texto ata o seguinte `%`.
2. Busca o `%` que o pecha. Se non hai, o resto, desde o primeiro, queda tal cal.
3. O que hai entre os dous é o nome. Se hai unha variábel con ese nome, sen distinguir maiúsculas, vai o seu valor, e segue despois do segundo `%`. O valor non se volta expandir: un `%` dentro del queda.
4. Se non hai ningunha, ou se o nome está baleiro (`%%`), o primeiro `%` queda tal cal, e o segundo pode empezar o nome seguinte.

Nun contorno con `USERPROFILE`, o cartafol persoal, e `A=x`:

| Texto | Queda | Por que |
|---|---|---|
| `%userprofile%\src\%A%` | O cartafol persoal, e `\src\x` | Os nomes non distinguen maiúsculas. |
| `%NONE%A%` | `%NONEx` | `NONE` non é unha variábel: o seu primeiro `%` queda, e o segundo empeza `%A%`. |
| `100%` | `100%` | Un `%` sen outro queda tal cal. |
| `%%A%` | `%x` | `%%` é un nome baleiro, que non é unha variábel: o primeiro queda, e o segundo empeza `%A%`. |
| `ñ%A%ñ` | `ñxñ` | `find` dá posicións onde empeza un carácter: os de varios bytes non se cortan. |

- `with_capacity(text.len())` reserva o tamaño do texto, o habitual se as variábeis non son longas.
- `let … else` colle a posición do segundo `%`, ou deixa o resto e sae do bucle.

### O bloque: `block`

```rust
pub fn block(&self) -> Vec<u16> {
    let mut vars: Vec<&[u16]> = self.vars.iter().map(Vec::as_slice).collect();
    insertion_sort_by(&mut vars, |first, second| compare_names(first, second));
    let length = vars.iter().map(|var| var.len() + 1).sum::<usize>();
    let mut block = Vec::with_capacity(length + 2);
    for var in vars {
        block.extend_from_slice(var);
        block.push(0);
    }
    // An empty block has two nulls too.
    if block.is_empty() {
        block.push(0);
    }
    block.push(0);
    block
}
```

**`block`**: as variábeis coma as recibe `CreateProcessW` con `CREATE_UNICODE_ENVIRONMENT`: cada unha rematada nun nulo, e outro nulo ao final.

- **Ordenadas polo nome, sen distinguir maiúsculas**, coma pide Windows nos bloques do contorno ([Tecnoloxías › Os perfís e a configuración](../tecnoloxias.md#57-os-perfís-e-a-configuración-toml-o-contorno-e-as-shells-de-windows)). Os nomes compáranse coma maiúsculas ([`compare_names`](#os-nomes-name_length-compare_names-e-uppercase)): `=` vai antes ca as letras; `_`, despois delas, porque está despois de `Z`, aínda que estea antes de `a`; e `Ñ`, despois de `z`. Por exemplo: `=D:`, `a`, `A_B`, `b`, `z` e `Ñ`.
- **Ordenadas por inserción** ([`sort.rs`](#cratesmeiga-coresrcsortrs)): o contorno que dá Windows xa vén case ordenado, e só se moven as variábeis que engade `set` ao final, que recúan ata o seu sitio. Ordénanse referencias ás variábeis (`&[u16]`), sen copialas: cada unha cópiase unha vez, no bloque.
- O bloque reserva dunha vez o que ocupa: `length`, as variábeis cos seus nulos, e os dous do final.
- **Un contorno sen variábeis** dá tamén dous nulos, coma di o comentario: `Conpty::spawn` só acepta un bloque que remata en dous nulos, porque Windows le o bloque ata a primeira variábel baleira, e sen eles pasaría do seu final ([Código › meiga-win › `conpty.rs`](meiga-win.md#cratesmeiga-winsrcconptyrs)).

### Os nomes: `name_length`, `compare_names` e `uppercase`

```rust
fn position(&self, name: &str) -> Option<usize> {
    self.vars
        .iter()
        .position(|var| uppercase(&var[..name_length(var)]).eq(uppercase_str(name)))
}
```

```rust
fn name_length(var: &[u16]) -> usize {
    var.iter()
        .skip(1)
        .position(|&unit| unit == u16::from(b'='))
        .map_or(var.len(), |at| at + 1)
}

fn compare_names(first: &[u16], second: &[u16]) -> Ordering {
    let first = uppercase(&first[..name_length(first)]);
    first.cmp(uppercase(&second[..name_length(second)]))
}

fn uppercase(name: &[u16]) -> impl Iterator<Item = char> + '_ {
    char::decode_utf16(name.iter().copied())
        .map(|c| c.unwrap_or(char::REPLACEMENT_CHARACTER))
        .flat_map(char::to_uppercase)
}

fn uppercase_str(name: &str) -> impl Iterator<Item = char> + '_ {
    name.chars().flat_map(char::to_uppercase)
}
```

- **`position`**, o método privado que usan `get`, `set` e `remove`: onde está a variábel `name`, comparando os nomes coma maiúsculas.
- **`name_length`**: onde remata o nome dun `NOME=valor`: no primeiro `=` despois do primeiro carácter, que pode ser o `=` dun nome coma `=C:`. `skip(1)` sáltao, e `at + 1` volta contalo. Sen `=`, todo é o nome.
- **`compare_names`**: a orde de dúas variábeis polos seus nomes, coma maiúsculas.
- **`uppercase`**: os caracteres dun nome en UTF-16, en maiúsculas. `decode_utf16` xunta as dúas unidades dos caracteres que ocupan dúas, e un que non é válido pasa a ser U+FFFD. **`uppercase_str`** fai o mesmo cun `&str`.
- **Sen textos novos.** `eq` e `cmp` de `Iterator` comparan os caracteres un a un, mentres saen dos iteradores: ningunha comparación reserva memoria.
- **As maiúsculas de Unicode.** `to_uppercase` usa as táboas de Unicode, e non só ASCII: `ñ` e `Ñ` son o mesmo nome, coma en Windows. Dá un iterador, porque algúns caracteres son varios en maiúsculas, e `flat_map` xúntaos. Comparar só en ASCII aforraría só 6 656 bytes de `meiga.exe`, medidos na 2.5: quedou a comparación de Unicode.

### O contorno dos programas: `prepare` e `wslenv`

```rust
pub fn prepare(environment: &mut Environment, version: &str) {
    for name in WINDOWS_TERMINAL {
        environment.remove(name);
    }
    environment.set(TERM_PROGRAM, NAME);
    environment.set(TERM_PROGRAM_VERSION, version);
    environment.set(COLORTERM, "truecolor");
    let current = environment.get(WSLENV);
    let wslenv = wslenv(
        current.as_deref(),
        &[TERM_PROGRAM, TERM_PROGRAM_VERSION, COLORTERM],
        &WINDOWS_TERMINAL,
    );
    environment.set(WSLENV, &wslenv);
}
```

**`prepare`** fai de `environment` o contorno dos programas das tabs ([ADR 0022](../adr/0022-os-perfis-e-a-configuracion.md), decisión 9). O programa chámao unha vez, ao arrancar, cunha copia do contorno de meiga e coa súa versión, `CARGO_PKG_VERSION`, e garda o bloque:

| Variábel | Valor | Por que |
|---|---|---|
| `WT_SESSION` e `WT_PROFILE_ID` | Ningún | Son do Windows Terminal, e meiga herdaas cando se abre desde el: con elas, os seus programas crerían que corren no Windows Terminal. |
| `TERM_PROGRAM` | `meiga` | O terminal no que corre o programa, coma `vscode` en VS Code. |
| `TERM_PROGRAM_VERSION` | A versión de meiga | Coma en VS Code e WezTerm. |
| `COLORTERM` | `truecolor` | Que o terminal mostra cores de 24 bits ([Tecnoloxías › A paleta](../tecnoloxias.md#37-a-paleta-e-os-estilos-das-celas)). |
| `WSLENV` | O que tiña, cos tres nomes anteriores e sen os do Windows Terminal | Para que WSL lles pase as tres variábeis aos programas de Linux. |

- **Os valores que xa había cámbianse**: un programa que corre en meiga non corre en VS Code, aínda que meiga se abrise desde el, e `COLORTERM=24bit` pasa a ser `truecolor`.
- **Sen `TERM`.** O ConPTY fala cos programas de Windows, que non o usan, e WSL pono para os de Linux.
- **O contorno de meiga, unha vez.** As tabs reciben o contorno que tiña meiga ao arrancar, co que lle deu quen o lanzou, coma o `APPDATA` das comprobacións; o Windows Terminal faino de novo desde o rexistro para cada tab. Un `PATH` que cambia despois non chega ás tabs novas ata reiniciar meiga ([ADR 0022 › Consecuencias](../adr/0022-os-perfis-e-a-configuracion.md#consecuencias)).

```rust
fn wslenv(current: Option<&str>, add: &[&str], drop: &[&str]) -> String {
    let named = |item: &str, name: &str| {
        item.split_once('/')
            .map_or(item, |(item, _)| item)
            .eq_ignore_ascii_case(name)
    };
    let mut items: Vec<&str> = current
        .unwrap_or_default()
        .split(':')
        .filter(|item| !item.is_empty() && !drop.iter().any(|name| named(item, name)))
        .collect();
    for name in add {
        if !items.iter().any(|item| named(item, name)) {
            items.push(name);
        }
    }
    items.join(":")
}
```

**`wslenv`**, privada: o valor novo do `WSLENV`. O `WSLENV` nomea as variábeis que WSL pasa de Windows aos programas de Linux, separadas por `:`, cada unha cunhas opcións despois dunha `/`, coma `USERPROFILE/p`, que traduce a ruta ([Tecnoloxías › Os perfís e a configuración](../tecnoloxias.md#57-os-perfís-e-a-configuración-toml-o-contorno-e-as-shells-de-windows)). Paso a paso:

1. Parte o valor de agora nos `:`, e quita os elementos baleiros, coma o de `::` ou o dun `:` ao final, e os de `drop`, coas súas opcións: tamén `WT_SESSION/u`.
2. Engade ao final os nomes de `add` que aínda non están, con ou sen opcións. Os que xa están quedan coma estaban, coas súas opcións e as súas maiúsculas: se o `WSLENV` xa ten `term_program/u`, quen o puxo escolleu como pasa, e meiga respéctao.
3. Xunta outra vez os elementos con `:`.

- **`named`** di se un elemento é a variábel `name`: o seu nome, sen as opcións, que separa `split_once('/')`, e sen distinguir maiúsculas en ASCII.
- **Ao final**, coma WezTerm, e non diante, coma o Windows Terminal: Microsoft recomenda engadir os nomes aos que xa ten o `WSLENV`, e non cambialos.
- Sen `WSLENV`, o valor novo só ten os tres nomes.

**Probas.** Cinco. `environment` fai un contorno cun bloque coma o de Windows, e `vars` le as variábeis dun bloque, despois de comprobar que remata en dous nulos.

| Proba | Que comproba |
|---|---|
| `names_do_not_distinguish_case_and_can_start_with_an_equals_sign` | Nun contorno con `=C:`, `Path`, `EMPTY=` e `x=a=b`: `=c:` atopa `=C:`; `PATH`, `Path`; `empty`, un valor baleiro; `X`, `a=b`, porque o primeiro `=` despois do nome o remata; e `missing`, nada. Despois de cambiar `PATH`, engadir `New` e quitar `=C:`, o bloque ten `EMPTY=`, `New=1`, `PATH=C:\bin` e `x=a=b`. |
| `the_block_goes_in_the_order_of_the_names_and_ends_with_two_nulls` | `b`, `A_B`, `a`, `=D:`, `Ñ` e `z` van no bloque coma `=D:`, `a`, `A_B`, `b`, `z` e `Ñ`; un contorno baleiro dá dous nulos; e un bloque de Windows remata na súa primeira variábel baleira: o que vai despois non se le. |
| `variables_expand_as_windows_expands_them` | Os textos da táboa de [`expand`](#expandir-as-variábeis-expand), e un baleiro. |
| `the_programs_know_that_they_run_in_meiga_and_not_in_windows_terminal` | Un contorno con `WT_SESSION`, `wt_profile_id`, en minúsculas, `WSLENV=WT_SESSION:WT_PROFILE_ID:`, coma o deixa o Windows Terminal cando estaba baleiro, e `COLORTERM=24bit` ten despois de `prepare` `COLORTERM=truecolor`, `Path`, `TERM_PROGRAM=meiga`, `TERM_PROGRAM_VERSION` coa versión e o `WSLENV` só cos tres nomes; e un contorno sen `WSLENV` recíbeo cos tres. |
| `wslenv_keeps_what_it_has_and_its_flags` | `USERPROFILE/p:term_program/u` queda, e só recibe `TERM_PROGRAM_VERSION` e `COLORTERM` ao final; `A::WT_SESSION/u:B` perde o elemento baleiro e `WT_SESSION/u`, e recibe `C`; e sen `WSLENV`, só `C`. |

**Como se comproba.** Ademais destas probas, `check-profiles.ps1` dálle a meiga na CI as variábeis do Windows Terminal, `TERM_PROGRAM=vscode`, `COLORTERM=24bit` e `WSLENV=WT_SESSION:COLORTERM/u:WT_PROFILE_ID/p:MEIGA_CHECK_WORK/p`, e comproba que o programa da primeira tab recibe `TERM_PROGRAM=meiga`, a versión de meiga, `COLORTERM=truecolor` e `WSLENV=COLORTERM/u:MEIGA_CHECK_WORK/p:TERM_PROGRAM:TERM_PROGRAM_VERSION`, sen `WT_SESSION` nin `WT_PROFILE_ID` ([Código › ferramentas](ferramentas.md#toolscheck-profilesps1)). As copias de meiga estragadas a propósito que non poñen `TERM_PROGRAM`, que deixan as variábeis do Windows Terminal ou que non cambian o `WSLENV` fallan a comprobación.

## `crates/meiga-core/src/sort.rs`

Desde a subfase 2.5: unha ordenación para as listas curtas de meiga: as variábeis do contorno, os erros da configuración, as versións de PowerShell e as distribucións de WSL e, desde a 2.6, os atallos do `config.toml` ([ADR 0022](../adr/0022-os-perfis-e-a-configuracion.md), decisión 12; [Tecnoloxías › Os perfís e a configuración](../tecnoloxias.md#57-os-perfís-e-a-configuración-toml-o-contorno-e-as-shells-de-windows)).

**Por que unha propia.** As ordenacións da biblioteca estándar están feitas para listas longas. `sort_by`, a estábel, é *driftsort*, que xunta varios algoritmos, coma un *quicksort* e outro para os anacos pequenos (*smallsort*); e, coma calquera función xenérica, compílase unha vez para cada tipo que ordena e cada comparación ([Tecnoloxías › Xenéricos e contadores atómicos](../tecnoloxias.md#53-xenéricos-e-contadores-atómicos)). Na 2.5, mentres se facía o código, `meiga.exe` tiña cinco copias dela, uns 33 KB. Unha ordenación por inserción ocupa uns centos de bytes por copia e, con unhas decenas de elementos, é tan rápida coma ela; cunha lista que xa está case ordenada, coma o contorno que dá Windows, máis. Na 2.5, cos cinco sitios ordenando con ela, e os dous que ordenan erros cunha soa función, [`config::sort`](#os-erros-na-orde-do-arquivo-sort), `meiga.exe` quedou 32 768 bytes máis pequeno. Na 2.6 chegou un sexto, os atallos do arquivo, cunha copia máis, tamén pequena.

```rust
pub fn insertion_sort_by<T>(items: &mut [T], mut compare: impl FnMut(&T, &T) -> Ordering) {
    for end in 1..items.len() {
        // The items before `end` are sorted: the one at `end` goes back past
        // the ones that go after it.
        let mut at = end;
        while at > 0 && compare(&items[at - 1], &items[at]) == Ordering::Greater {
            items.swap(at - 1, at);
            at -= 1;
        }
    }
}
```

**`insertion_sort_by`** ordena `items` coma os ordena `compare`, coma `slice::sort_by`. Paso a paso, coma di o comentario:

1. As posicións antes de `end` xa están ordenadas: ao principio, só a primeira.
2. O elemento en `end` recúa, cambiando o sitio co anterior (`swap`), mentres o anterior vai despois del.
3. Cando chega ao seu sitio, as posicións ata `end` están ordenadas, e segue co seguinte.

- **Estábel.** Un elemento só recúa mentres o anterior vai despois del (`Greater`): ante un igual, para. Así, dous elementos iguais quedan na orde na que estaban, coma con `sort_by`: dous erros no mesmo sitio, na orde na que se atoparon.
- **Canto traballa.** Cunha lista xa ordenada, unha comparación por elemento, e ningún cambio de sitio; cun elemento fóra do seu sitio, tantos cambios coma posicións recúa; e, no peor caso, cunha lista ao revés, uns n²/2. Con listas de decenas de elementos, nada.
- **Sen reservar memoria**: os elementos cambian de sitio dentro da mesma lista.
- `compare` é `FnMut`: calquera función ou *closure* que compare dous elementos, tamén unha que cambie algo seu ao chamala. É xenérica, coma `sort_by`: cada uso compila a súa copia, pero pequena.

| Quen a usa | Que ordena |
|---|---|
| [`config::sort`](#os-erros-na-orde-do-arquivo-sort) | Os erros da configuración, na orde do arquivo. |
| [`Reader::shortcuts`](#os-atallos-do-arquivo-shortcuts), en `config.rs` | Desde a 2.6: as entradas de `[shortcuts]`, na orde do arquivo, para que valla a primeira de cada combinación. |
| [`Environment::block`](#o-bloque-block) | As variábeis do contorno, polo nome, sen distinguir maiúsculas. |
| `profiles::detect`, en `meiga-win` | As versións de PowerShell, a máis nova primeiro, e as distribucións de WSL, a por defecto primeiro e as outras polo nome ([Código › meiga-win › `profiles.rs`](meiga-win.md#cratesmeiga-winsrcprofilesrs)). |

**Probas.** Dúas:

| Proba | Que comproba |
|---|---|
| `it_sorts_as_the_stable_sort_of_the_standard_library` | Listas de 0 a 39 elementos, con chaves do 0 ao 7, saídas dunha secuencia *xorshift* fixa, e a posición de cada elemento: quedan igual ca con `sort_by_key` da biblioteca estándar, que é estábel. Con tantas chaves repetidas, os elementos iguais só quedan na orde das súas posicións se a ordenación é estábel. |
| `it_follows_the_order_that_it_is_given` | `[3, 1, 4, 1, 5, 9, 2, 6]`, coa comparación ao revés, queda `[9, 6, 5, 4, 3, 2, 1, 1]`. |

## `crates/meiga-core/src/shortcut.rs`

Desde a subfase 2.6: os atallos de teclado, as teclas que meiga colle para si e o que fai cada unha ([ADR 0023](../adr/0023-os-atallos-de-teclado.md); [Arquitectura › Os atallos de teclado](../arquitectura.md#os-atallos-de-teclado); [Tecnoloxías › Os atallos de teclado](../tecnoloxias.md#58-os-atallos-de-teclado-as-teclas-virtuais-altgr-e-os-doutros-programas)). Un atallo é unha combinación, unhas teclas modificadoras e unha tecla, e unha acción. meiga ten os seus, coma o Windows Terminal e Chrome, e a táboa `[shortcuts]` do `config.toml` engade outros, cámbialles a acción ou libéraos ([ADR 0023](../adr/0023-os-atallos-de-teclado.md), decisión 1):

```toml
[shortcuts]
"ctrl+t" = "new_tab"            # another shortcut for an action
"ctrl+6" = "none"               # Ctrl+6 reaches the program again
"ctrl+shift+u" = "profile_4"
```

Este arquivo, o do comentario do módulo (`//!`), fai de Ctrl+T outro atallo para abrir unha tab, ademais de Ctrl+Maiús+T; libera Ctrl+6, que con meiga mostra a sexta tab, para que chegue outra vez ao programa; e fai que Ctrl+Maiús+U abra o cuarto perfil do menú dos perfís.

**A sintaxe** é a do Windows Terminal, coma di o comentario do módulo ([ADR 0023](../adr/0023-os-atallos-de-teclado.md), decisión 2): as teclas modificadoras `ctrl`, `alt` e `shift`, en calquera orde, e unha tecla, unidas con `+`, sen distinguir maiúsculas. A tecla é unha letra, un díxito, `f1`–`f24`, `numpad0`–`numpad9` ou un dos nomes de [`NAMES`](#as-teclas-virtuais-e-os-seus-nomes-names), e vai polo seu código de tecla virtual, que di que tecla é na distribución do teclado, e non polo carácter que escribe: `ctrl+shift+2` é a mesma combinación con calquera distribución, aínda que o 2 con Maiús escriba `@` nunhas e `"` noutras, coma no teclado estadounidense e no español ([Tecnoloxías › O teclado en Windows](../tecnoloxias.md#41-o-teclado-en-windows)). AltGr, que Windows dá coma o Ctrl esquerdo e o Alt dereito, escribe caracteres: ningún atallo o ten, coma no Windows Terminal (decisión 10).

O módulo ten tres partes:

| Parte | Que é |
|---|---|
| [`Chord`](#a-combinación-chord) | Unha combinación: a dunha tecla que baixa, ou a dun texto do arquivo. |
| [`Action`](#as-accións-action) | O que fai un atallo, co seu nome no arquivo. |
| [`Shortcuts`](#os-atallos-entry-e-shortcuts) | Os atallos dunha xanela, os de meiga, [`defaults`](#os-atallos-de-meiga-defaults), cos do arquivo por riba, e a acción de cada combinación. |

Non le o arquivo nin o teclado. `config.rs` le a táboa, e garda as súas entradas ([`shortcuts`](#os-atallos-do-arquivo-shortcuts)); o programa fai con elas os atallos da xanela, unha vez, ao crear as súas tabs; e a vista do terminal busca a combinación de cada tecla que baixa, e fai a súa acción ([Código › meiga › `view.rs`](meiga.md#cratesmeigasrcviewrs)). Así que se proba con textos e con eventos de tecla feitos a man. Usa [`KeyEvent`](#keyevent), de `input.rs`, para a combinación dunha tecla, e `std::fmt`, para as mensaxes dos erros.

### As teclas virtuais e os seus nomes: `NAMES`

```rust
const VK_TAB: u16 = 0x09;
const VK_SPACE: u16 = 0x20;
const VK_PRIOR: u16 = 0x21;
const VK_NEXT: u16 = 0x22;
const VK_END: u16 = 0x23;
const VK_HOME: u16 = 0x24;
const VK_UP: u16 = 0x26;
const VK_DOWN: u16 = 0x28;
const VK_APPS: u16 = 0x5d;
const VK_NUMPAD0: u16 = 0x60;
const VK_ADD: u16 = 0x6b;
const VK_SUBTRACT: u16 = 0x6d;
const VK_F1: u16 = 0x70;
const VK_F4: u16 = 0x73;
const VK_F10: u16 = 0x79;
const VK_OEM_PLUS: u16 = 0xbb;
const VK_OEM_MINUS: u16 = 0xbd;
```

**Os `VK_…`**: os códigos de tecla virtual que usan os atallos de meiga, cos nomes e os valores de Windows: Tab, Espazo, RePáx (`PRIOR`), AvPáx (`NEXT`), Fin, Inicio, ↑, ↓, a tecla de menú (`APPS`), o 0, o + e o − do teclado numérico (`NUMPAD0`, `ADD` e `SUBTRACT`), F1, F4 e F10, e as teclas que escriben + e - (`OEM_PLUS` e `OEM_MINUS`). Úsanos `NAMES`, [`key_code`](#o-código-dunha-tecla-key_code-strip_prefix_ignore_case-e-number), [`Chord::parse`](#ler-unha-combinación-chordparse) e [`defaults`](#os-atallos-de-meiga-defaults). Son privados. `meiga-core` non depende de `windows-sys`, e o módulo `vk` de `input.rs`, cos códigos que trata á parte a codificación VT, é privado: coma el, este módulo define os seus ([`input.rs`](#os-axudantes-das-secuencias)).

```rust
pub const NAMES: [(&str, u16); 32] = [
    ("enter", 0x0d),
    ("tab", VK_TAB),
    ("space", VK_SPACE),
    ("backspace", 0x08),
    ("menu", VK_APPS),
    ("app", VK_APPS),
    ("insert", 0x2d),
    ("delete", 0x2e),
    ("home", VK_HOME),
    ("end", VK_END),
    ("pgdn", VK_NEXT),
    ("pagedown", VK_NEXT),
    ("pgup", VK_PRIOR),
    ("pageup", VK_PRIOR),
    ("esc", 0x1b),
    ("escape", 0x1b),
    ("left", 0x25),
    ("right", 0x27),
    ("up", VK_UP),
    ("down", VK_DOWN),
    ("numpad_plus", VK_ADD),
    ("numpad_add", VK_ADD),
    ("numpad_minus", VK_SUBTRACT),
    ("numpad_subtract", VK_SUBTRACT),
    ("numpad_multiply", 0x6a),
    ("numpad_divide", 0x6f),
    ("numpad_period", 0x6e),
    ("numpad_decimal", 0x6e),
    ("plus", VK_OEM_PLUS),
    ("comma", 0xbc),
    ("minus", VK_OEM_MINUS),
    ("period", 0xbe),
];
```

**`NAMES`**: os nomes das teclas que non son unha letra, un díxito nin unha tecla de función, cos seus códigos. Son os do Windows Terminal, sen os das teclas do navegador (decisión 2):

| Nomes | Tecla | Código |
|---|---|---|
| `enter` | Intro | `0x0D` |
| `tab` | Tab | `0x09` |
| `space` | Espazo | `0x20` |
| `backspace` | Retroceso | `0x08` |
| `menu`, `app` | A tecla de menú | `0x5D` |
| `insert`, `delete` | Insert e Supr | `0x2D` e `0x2E` |
| `home`, `end` | Inicio e Fin | `0x24` e `0x23` |
| `pgdn`, `pagedown` | AvPáx | `0x22` |
| `pgup`, `pageup` | RePáx | `0x21` |
| `esc`, `escape` | Esc | `0x1B` |
| `left`, `right`, `up`, `down` | As frechas | `0x25`, `0x27`, `0x26` e `0x28` |
| `numpad_plus`, `numpad_add` | O + do teclado numérico | `0x6B` |
| `numpad_minus`, `numpad_subtract` | O − do teclado numérico | `0x6D` |
| `numpad_multiply`, `numpad_divide` | O * e a / do teclado numérico | `0x6A` e `0x6F` |
| `numpad_period`, `numpad_decimal` | A tecla decimal do teclado numérico | `0x6E` |
| `plus`, `comma`, `minus`, `period` | As teclas que escriben `+`, `,`, `-` e `.` en calquera distribución | `0xBB`, `0xBC`, `0xBD` e `0xBE` |

- Os nomes van en minúsculas, pero [`key_code`](#o-código-dunha-tecla-key_code-strip_prefix_ignore_case-e-number) compáraos sen distinguir maiúsculas: `PageUp` tamén vale. Algunhas teclas teñen dous nomes, coma `pgup` e `pageup`.
- Non están as letras, os díxitos, `f1`–`f24` nin `numpad0`–`numpad9`, que `key_code` calcula.
- A tecla `+` chámase `plus`: o `+` separa as partes dunha combinación.
- É pública, pero polo de agora só a usan este módulo e as súas probas.

### A combinación: `Chord`

```rust
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Chord {
    modifiers: u8,
    key: u16,
}
```

```rust
pub const CTRL: u8 = 1;
pub const ALT: u8 = 2;
pub const SHIFT: u8 = 4;

pub const fn new(modifiers: u8, key: u16) -> Self {
    Self {
        modifiers: modifiers & (Self::CTRL | Self::ALT | Self::SHIFT),
        key,
    }
}
```

- **`Chord`**: unha combinación, as teclas modificadoras que están abaixo e a tecla que baixa. `modifiers` garda Ctrl, Alt e Maiús en tres bits, `CTRL`, `ALT` e `SHIFT`, ou ningún; e `key`, o código de tecla virtual da tecla. Non distingue o Ctrl esquerdo do dereito, nin o Alt: `ctrl+t` vale cos dous Ctrl.
- **`new`** fai unha combinación, e quítalles aos modificadores calquera outro bit (`&`): dúas combinacións coas mesmas teclas son sempre iguais. Os campos son privados, así que toda combinación pasa por el.
- **Ningún método dá os campos**: ningún código os necesita, porque as combinacións compáranse enteiras.
- É `Copy`, e vai por valor. `PartialEq` e `Eq` comparan os dous campos: así atopa [`Shortcuts::action`](#os-atallos-entry-e-shortcuts) a combinación dunha tecla entre as dos atallos, e `config.rs`, unha combinación repetida ([`shortcuts`](#os-atallos-do-arquivo-shortcuts)). `Hash` deixaría usala de chave nun mapa, pero os atallos van nunha lista.
- Todas as funcións de `Chord` son `const fn`, menos `parse`.

### A combinación dunha tecla: `Chord::of`

```rust
pub const fn of(event: &KeyEvent) -> Option<Self> {
    let controls = event.controls;
    if event.key == 0 || controls.alt_gr() {
        return None;
    }
    let mut modifiers = 0;
    if controls.ctrl() {
        modifiers |= Self::CTRL;
    }
    if controls.alt() {
        modifiers |= Self::ALT;
    }
    if controls.shift() {
        modifiers |= Self::SHIFT;
    }
    Some(Self::new(modifiers, event.key))
}
```

**`of`** dá a combinación dunha tecla que baixa: Ctrl, Alt e Maiús, os que están abaixo, e a súa tecla ([`ControlKeys`](#controlkeys)). Dá `None` se ningún atallo a pode ter: cun carácter sen tecla, dun IME ou do panel de emoji, que chega coma a tecla 0 ([`KeyEvent`](#keyevent)); e cunha tecla con AltGr, tamén con Maiús. [`alt_gr`](#controlkeys) di se o Ctrl e o Alt que están abaixo son só os de AltGr, o Ctrl esquerdo e o Alt dereito. AltGr escribe caracteres, coma `@`, `#`, `€` ou os corchetes no teclado español, e nunca dispara un atallo ([ADR 0023](../adr/0023-os-atallos-de-teclado.md), decisión 10).

- **Calquera outro Ctrl e Alt é Ctrl+Alt.** Cos dous Ctrl e o Alt dereito, ou co Ctrl dereito e o Alt esquerdo, xa non é só AltGr, e a combinación dispara os atallos `ctrl+alt+…` que poña o arquivo. O Windows Terminal mira só que estean o Ctrl esquerdo e o Alt dereito, e non os outros dous (decisión 10).
- As teclas de bloqueo e a marca das teclas estendidas non contan: Ctrl+Maiús+T é a mesma combinación co Bloq Num activado.
- Non mira se a tecla baixa: a vista chámaa só coas que baixan, tamén cando repiten ([Código › meiga › `view.rs`](meiga.md#cratesmeigasrcviewrs)).
- É unha `const fn`, coma `alt_gr`, `ctrl`, `alt` e `shift`.

### Ler unha combinación: `Chord::parse`

```rust
pub fn parse(text: &str) -> Result<Self, ChordError> {
    let mut modifiers = 0;
    let mut key = None;
    for part in text.split('+') {
        let part = part.trim();
        if part.is_empty() {
            return Err(ChordError::Empty);
        }
        if part.eq_ignore_ascii_case("ctrl") {
            modifiers |= Self::CTRL;
        } else if part.eq_ignore_ascii_case("alt") {
            modifiers |= Self::ALT;
        } else if part.eq_ignore_ascii_case("shift") {
            modifiers |= Self::SHIFT;
        } else if part.eq_ignore_ascii_case("win") {
            return Err(ChordError::Windows);
        } else {
            let code = key_code(part).ok_or_else(|| ChordError::Unknown(part.to_owned()))?;
            if key.replace(code).is_some() {
                return Err(ChordError::TwoKeys);
            }
        }
    }
    let key = key.ok_or(ChordError::NoKey)?;
    // Windows closes the window with Alt+F4 and, with Alt+Space, asks
    // the window for its menu, which meiga opens below the bar: they
    // never reach meiga as keys (ADR 0014 and ADR 0027).
    if modifiers & (Self::ALT | Self::CTRL) == Self::ALT && matches!(key, VK_F4 | VK_SPACE) {
        return Err(ChordError::ForWindows);
    }
    Ok(Self::new(modifiers, key))
}
```

**`parse`** le unha combinación escrita coma no Windows Terminal, coma `ctrl+shift+t`. Paso a paso:

1. **As partes.** `split('+')` parte o texto en cada `+`, e `trim` quita os brancos arredor de cada parte, que non contan: `Shift + CTRL + T` é `ctrl+shift+t`.
2. **Unha parte baleira** é un erro, `Empty`: `""`, `ctrl+` ou `ctrl++`. Por iso a tecla + escríbese `plus`.
3. **Os modificadores.** `ctrl`, `alt` e `shift`, sen distinguir maiúsculas (`eq_ignore_ascii_case`), engaden o seu bit. Repetidos, son o mesmo: `ctrl+ctrl+t` é `ctrl+t`.
4. **`win`** é un erro, `Windows`: as teclas Windows son de Windows (decisión 2).
5. **A tecla.** Calquera outra parte é a tecla: [`key_code`](#o-código-dunha-tecla-key_code-strip_prefix_ignore_case-e-number) dá o seu código, ou é un erro, `Unknown`, coa parte. Unha segunda tecla é outro erro, `TwoKeys`: `replace` garda a nova en `key`, e devolve a que había.
6. **Sen tecla**, só con modificadores, é un erro, `NoKey`.
7. **Alt+F4 e Alt+Espazo**, con Maiús ou sen el, son un erro, `ForWindows`: Windows pecha a xanela con Alt+F4 e, con Alt+Espazo, pídelle á xanela o seu menú, que desde a 2.9 abre meiga debaixo da barra ([ADR 0027](../adr/0027-a-barra-de-tabs-coma-barra-de-titulo.md), decisión 6); nunca lle chegan a meiga coma teclas, coma di o comentario ([ADR 0014](../adr/0014-o-teclado.md), decisión 5). A condición, Alt sen Ctrl, é a mesma coa que o teclado da xanela llos deixa a Windows ([Código › meiga-win › `keyboard.rs`](meiga-win.md#cratesmeiga-winsrckeyboardrs)). Con Ctrl, son outras combinacións: `ctrl+alt+f4` vale.
8. **A combinación**, con `new`.

- O erro é o da primeira parte mal, de esquerda a dereita: `return` e `?` devólveno no momento.
- Máis permisiva ca o Windows Terminal, que distingue maiúsculas nos nomes das teclas, salvo nas letras, e non admite brancos (decisión 2). Pero con menos teclas: sen `win`, nin `vk()`, `sc()` ou un carácter calquera, que depende da distribución do teclado.
- `ok_or_else` só fai o erro, e copia a parte nun `String` con `to_owned`, cando a parte non é unha tecla.

### Por que non é unha combinación: `ChordError`

```rust
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ChordError {
    Empty,
    NoKey,
    TwoKeys,
    Unknown(String),
    Windows,
    ForWindows,
}

impl fmt::Display for ChordError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Empty => f.write_str("a part is empty (the key + is `plus`)"),
            Self::NoKey => f.write_str("it has no key, only modifier keys"),
            Self::TwoKeys => f.write_str("it has more than one key"),
            Self::Unknown(part) => write!(f, "`{part}` is not a key"),
            Self::Windows => f.write_str("the Windows keys are for Windows"),
            Self::ForWindows => f.write_str("Alt+F4 and Alt+Space are for Windows"),
        }
    }
}
```

| Variante | Cando | Mensaxe |
|---|---|---|
| `Empty` | Non hai nada entre dous `+`, ou antes ou despois dun. | `` a part is empty (the key + is `plus`) `` |
| `NoKey` | Só ten modificadores, coma `ctrl+shift`. | `it has no key, only modifier keys` |
| `TwoKeys` | Ten máis dunha tecla, coma `ctrl+a+b`. | `it has more than one key` |
| `Unknown` | Unha parte non é un modificador nin unha tecla, coma `hyper`, `ñ` ou `f25`. Garda a parte. | `` `hyper` is not a key `` |
| `Windows` | Ten unha tecla Windows, `win`. | `the Windows keys are for Windows` |
| `ForWindows` | É Alt+F4 ou Alt+Espazo, con Maiús ou sen el. | `Alt+F4 and Alt+Space are for Windows` |

- `Display` escribe o motivo, en inglés coma o resto da interface. O erro da caixa de mensaxe leva diante a combinación: `` `ctrl+a+b` is not a shortcut: it has more than one key `` ([`shortcuts`](#os-atallos-do-arquivo-shortcuts)).
- Non é `Copy`: `Unknown` leva un `String`.

### O código dunha tecla: `key_code`, `strip_prefix_ignore_case` e `number`

```rust
fn key_code(name: &str) -> Option<u16> {
    let bytes = name.as_bytes();
    if let [byte] = bytes {
        return match byte.to_ascii_uppercase() {
            byte @ (b'0'..=b'9' | b'A'..=b'Z') => Some(u16::from(byte)),
            _ => None,
        };
    }
    if let Some(number) = strip_prefix_ignore_case(name, "f").and_then(number) {
        return match number {
            1..=24 => Some(VK_F1 + number - 1),
            _ => None,
        };
    }
    if let Some(digit) = strip_prefix_ignore_case(name, "numpad")
        .map(|rest| rest.strip_prefix('_').unwrap_or(rest))
        .and_then(number)
    {
        return match digit {
            0..=9 => Some(VK_NUMPAD0 + digit),
            _ => None,
        };
    }
    NAMES
        .iter()
        .find(|(known, _)| known.eq_ignore_ascii_case(name))
        .map(|&(_, code)| code)
}
```

**`key_code`** dá o código de tecla virtual da tecla `name`, sen distinguir maiúsculas, ou `None`. Proba catro formas, nesta orde:

1. **Un só byte**: unha letra ou un díxito, que teñen de código o seu carácter en ASCII, en maiúsculas: `t` e `T` son `0x54`, e `1`, `0x31`. O patrón `[byte]` casa cun texto dun só byte, e `byte @ (…)` colle o byte en maiúsculas se está nun dos dous rangos. Calquera outro carácter, coma `,` ou `-`, non é unha tecla: escríbese co seu nome, `comma` ou `minus`. Un carácter de varios bytes en UTF-8, coma `ñ`, non entra eiquí nin está entre os nomes: tampouco é unha tecla.
2. **`f` e un número** do 1 ao 24: F1–F24, que teñen códigos seguidos desde `VK_F1`, `0x70`; F24 é `0x87`. Outro número, coma `f0`, `f25` ou `f65535`, non é unha tecla: o `match` dá `None`, e `return` sae sen mirar os nomes, onde tampouco está.
3. **`numpad` e un díxito**, con `_` ou sen el: o díxito do teclado numérico, `VK_NUMPAD0` máis o díxito. `numpad7` e `numpad_7` son `0x67`; `numpad10` e `numpad65535` non son teclas. `strip_prefix('_')` quita o guión baixo, se o hai, e `unwrap_or` deixa o resto como estaba se non.
4. **Os nomes** de [`NAMES`](#as-teclas-virtuais-e-os-seus-nomes-names), sen distinguir maiúsculas: o primeiro que coincide. Eiquí chegan tamén `numpad_plus` e os outros nomes do teclado numérico, porque `plus` non é un número; e `f` só, dun byte, é a tecla F, do primeiro paso.

- **O `match` mira o número antes de sumalo.** `number`, máis abaixo, acepta calquera número que caiba nun `u16`, ata 65 535, e as sumas, `VK_F1 + number - 1` e `VK_NUMPAD0 + digit`, tamén son `u16`: cun número coma 65 535, pasarían do máximo (*overflow*), con `f` desde o 65 424, e con `numpad` desde o 65 440. Por iso a suma vai no brazo do rango, e só se fai despois de comprobalo. `(1..=24).contains(&number).then_some(VK_F1 + number - 1)` diría o mesmo, pero `then_some` recibe o seu valor xa calculado, mesmo cando vai dar `None`: a suma faríase sempre. Nunha compilación de depuración, coma a das probas, o *overflow* sería un *panic*; e en *release*, que non o comproba, a suma daría a volta, e `then_some` tiraría ese valor errado.

```rust
fn strip_prefix_ignore_case<'a>(text: &'a str, prefix: &str) -> Option<&'a str> {
    let start = text.get(..prefix.len())?;
    start
        .eq_ignore_ascii_case(prefix)
        .then(|| &text[prefix.len()..])
}

fn number(digits: &str) -> Option<u16> {
    if digits.is_empty() || !digits.bytes().all(|byte| byte.is_ascii_digit()) {
        return None;
    }
    digits.parse().ok()
}
```

- **`strip_prefix_ignore_case`**: `text` sen `prefix` ao principio, se empeza por el, sen distinguir maiúsculas: `F12` dá `12`. `str::strip_prefix` distingue maiúsculas, e por iso esta. `get(..prefix.len())` dá `None`, e non un *panic*, se o texto é máis curto ca o prefixo, ou se ese byte cae no medio dun carácter de varios bytes; `then` só corta o texto se o principio coincide. O resultado é un anaco do mesmo texto, co seu tempo de vida, `'a`, sen copialo.
- **`number`**: o número que escriben `digits`, se todos son díxitos, e non demasiados para un `u16`, ata 65 535. `str::parse` aceptaría un `+` diante, coma en `+1`, e por iso se miran antes os bytes; con demasiados díxitos falla, e `ok` dá `None`. Os ceros diante non contan: `f01` é F1. Úsana `key_code` e [`Action::parse`](#os-nomes-das-accións-actions-none-e-actionparse).

### As accións: `Action`

```rust
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Action {
    NewTab,
    CloseTab,
    NextTab,
    PreviousTab,
    Tab(u16),
    LastTab,
    Profile(u16),
    ProfileMenu,
    TabMenu,
    RenameTab,
    DuplicateTab,
    CloseOtherTabs,
    CloseTabsToTheRight,
    Copy,
    Paste,
    ScrollUp,
    ScrollDown,
    ScrollPageUp,
    ScrollPageDown,
    ScrollToTop,
    ScrollToBottom,
    ZoomIn,
    ZoomOut,
    ZoomReset,
}
```

O que fai un atallo, co seu nome no `config.toml` ([ADR 0023](../adr/0023-os-atallos-de-teclado.md), decisión 4):

| Acción | Nome | Que fai |
|---|---|---|
| `NewTab` | `new_tab` | Abre unha tab co perfil por defecto, coma o botón +. |
| `CloseTab` | `close_tab` | Pecha a tab que se mostra. |
| `NextTab` | `next_tab` | Mostra a tab seguinte, ou a primeira despois da última. |
| `PreviousTab` | `previous_tab` | Mostra a anterior, ou a última antes da primeira. |
| `Tab(n)` | `tab_1`, `tab_2`… | Mostra a tab dese sitio da barra, contado desde 0: `tab_1` é `Tab(0)`. Se alí non hai ningunha, non pasa nada. |
| `LastTab` | `last_tab` | Mostra a última tab. |
| `Profile(n)` | `profile_1`, `profile_2`… | Abre unha tab co perfil dese sitio do menú dos perfís, contado desde 0: `profile_1` é `Profile(0)`. |
| `ProfileMenu` | `profile_menu` | Abre o menú dos perfís, coma o botón que está xunto ao +. |
| `TabMenu` | `tab_menu` | Abre o menú da tab que se mostra, debaixo dela. |
| `RenameTab` | `rename_tab` | Desde a 3.3: renomea a tab que se mostra, coma o seu menú; o seu nome edítase na súa tab ([ADR 0031](../adr/0031-o-nome-das-tabs.md)). |
| `DuplicateTab` | `duplicate_tab` | Duplica a tab que se mostra, coma o seu menú. |
| `CloseOtherTabs` | `close_other_tabs` | Pecha todas as tabs menos a que se mostra, coma o seu menú. |
| `CloseTabsToTheRight` | `close_tabs_to_the_right` | Pecha as tabs á dereita da que se mostra, coma o seu menú. |
| `Copy` | `copy` | Copia a selección, se a hai. |
| `Paste` | `paste` | Pega o portapapeis. |
| `ScrollUp` e `ScrollDown` | `scroll_up` e `scroll_down` | Move a vista do historial unha fila arriba ou abaixo. |
| `ScrollPageUp` e `ScrollPageDown` | `scroll_page_up` e `scroll_page_down` | Unha páxina arriba ou abaixo. |
| `ScrollToTop` e `ScrollToBottom` | `scroll_to_top` e `scroll_to_bottom` | Ao principio do historial, ou ao final. |
| `ZoomIn` e `ZoomOut` | `zoom_in` e `zoom_out` | Fai a fonte un punto máis grande, ou máis pequena. |
| `ZoomReset` | `zoom_reset` | Leva a fonte ao seu tamaño do principio. |

- **O que fai cada unha é cousa do programa**: a vista do terminal fai as do portapapeis, o historial e o zoom, e pásalles as outras ás tabs ([Código › meiga › `view.rs`](meiga.md#cratesmeigasrcviewrs) e [› `tabs.rs`](meiga.md#cratesmeigasrctabsrs)). Este módulo só as nomea.
- **A tecla vai ao programa** cando a acción non se pode facer: copiar sen selección, e mover o historial na pantalla alternativa, que non ten historial (decisión 4). Así, Ctrl+C sen selección segue chegando ao programa.
- **`Tab` e `Profile`** levan un sitio, desde 0, nun `u16`. Unha tab ou un perfil que non existe non fai nada, e a tecla tampouco lle chega ao programa: é a dun atallo de meiga, teña ou non onde ir (decisión 11).
- É `Copy`: vai por valor nos atallos e nas entradas do arquivo.

### Os nomes das accións: `ACTIONS`, `NONE` e `Action::parse`

```rust
const ACTIONS: [(&str, Action); 22] = [
    ("new_tab", Action::NewTab),
    ("close_tab", Action::CloseTab),
    ("next_tab", Action::NextTab),
    ("previous_tab", Action::PreviousTab),
    ("last_tab", Action::LastTab),
    ("profile_menu", Action::ProfileMenu),
    ("tab_menu", Action::TabMenu),
    ("rename_tab", Action::RenameTab),
    ("duplicate_tab", Action::DuplicateTab),
    ("close_other_tabs", Action::CloseOtherTabs),
    ("close_tabs_to_the_right", Action::CloseTabsToTheRight),
    ("copy", Action::Copy),
    ("paste", Action::Paste),
    ("scroll_up", Action::ScrollUp),
    ("scroll_down", Action::ScrollDown),
    ("scroll_page_up", Action::ScrollPageUp),
    ("scroll_page_down", Action::ScrollPageDown),
    ("scroll_to_top", Action::ScrollToTop),
    ("scroll_to_bottom", Action::ScrollToBottom),
    ("zoom_in", Action::ZoomIn),
    ("zoom_out", Action::ZoomOut),
    ("zoom_reset", Action::ZoomReset),
];

pub const NONE: &str = "none";
```

- **`ACTIONS`**: os nomes das 22 accións sen número (21 ata a 3.2, sen `rename_tab`), coma os escribe o `config.toml`: en minúsculas, cun `_` entre as palabras (decisión 2). É privada.
- **`NONE`**: `none`, o nome que libera unha combinación: a súa tecla vai entón ao programa. É pública, pero polo de agora só a usa `parse`.

```rust
pub fn parse(name: &str) -> Result<Option<Self>, ActionError> {
    if name == NONE {
        return Ok(None);
    }
    if let Some((_, action)) = ACTIONS.iter().find(|(known, _)| *known == name) {
        return Ok(Some(*action));
    }
    let numbered = |prefix: &str, action: fn(u16) -> Self| {
        let digits = name.strip_prefix(prefix)?;
        Some(match number(digits) {
            Some(0) => Err(ActionError::FromOne),
            Some(place) => Ok(Some(action(place - 1))),
            None => Err(ActionError::Unknown),
        })
    };
    numbered("tab_", Self::Tab)
        .or_else(|| numbered("profile_", Self::Profile))
        .unwrap_or(Err(ActionError::Unknown))
}
```

**`parse`** le o nome dunha acción, coma o escribe o `config.toml`. Devolve `Ok(Some(acción))`; `Ok(None)` con `none`, que libera a combinación; ou un erro. Paso a paso:

1. `none`: `Ok(None)`.
2. Un nome de `ACTIONS`: `find` percorre a lista.
3. `tab_` ou `profile_` e un número desde 1: `Tab` ou `Profile`, co número menos 1, o sitio contado desde 0. `tab_0` e `profile_0` teñen un erro de seu, `FromOne`, que lembra que se contan desde 1. Calquera outra cousa despois do prefixo é `Unknown`: `tab_`, `tab_x`, `tab_+1` ou `tab_99999`, que non cabe nun `u16` ([`number`](#o-código-dunha-tecla-key_code-strip_prefix_ignore_case-e-number)).
4. Calquera outro nome: `Unknown`.

- **Distingue maiúsculas**, ao contrario ca as combinacións: `New_Tab`, `None` e `Tab_1` non son accións. As accións escríbense en minúsculas (decisión 2).
- **`numbered`**, unha *closure*, proba un prefixo: o `?` sae dela con `None` se o nome non empeza por el, e entón `or_else` proba o seguinte; se ningún coincide, `unwrap_or` dá `Unknown`. Recibe a variante, `Self::Tab` ou `Self::Profile`, coma unha función `fn(u16) -> Self`: o nome dunha variante cun campo é tamén a función que a fai.
- O número máis grande é 65 535: `tab_65535` é `Tab(65534)`.

### As que repiten: `Action::repeats`

```rust
pub const fn repeats(self) -> bool {
    matches!(
        self,
        Self::NextTab
            | Self::PreviousTab
            | Self::ScrollUp
            | Self::ScrollDown
            | Self::ScrollPageUp
            | Self::ScrollPageDown
            | Self::ScrollToTop
            | Self::ScrollToBottom
            | Self::ZoomIn
            | Self::ZoomOut
    )
}
```

**`repeats`**: se a acción se fai outra vez mentres se mantén a súa tecla, cada vez que o teclado a repite ([ADR 0023](../adr/0023-os-atallos-de-teclado.md), decisión 12): a tab seguinte e a anterior, as seis do historial, e o zoom, menos o tamaño do principio. As outras, non: unha tecla mantida abre unha soa tab, pecha unha soa ou pega unha vez.

- As teclas dun atallo non lle chegan ao programa mentres repiten nin cando soben, repitan ou non a acción: iso faino a vista, que apunta as teclas que colle ata que soben ([Código › meiga › `view.rs`](meiga.md#cratesmeigasrcviewrs)).
- Ata a 2.5, a vista decidía pola tecla: repetían RePáx, AvPáx, Inicio, Fin, ↑, ↓, as teclas do zoom e Tab. Desde a 2.6, unha combinación pode ter calquera acción, e decide a acción.
- `matches!` compara a acción cos patróns. É unha `const fn`.

### Por que non é unha acción: `ActionError`

```rust
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ActionError {
    Unknown,
    FromOne,
}

impl fmt::Display for ActionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Unknown => "is not an action",
            Self::FromOne => "is not an action: the tabs and the profiles count from 1",
        })
    }
}
```

- **`Unknown`**: un nome que non é unha acción. **`FromOne`**: `tab_0` ou `profile_0`.
- `Display` escribe o motivo sen o nome, que pon diante `config.rs` ([`shortcuts`](#os-atallos-do-arquivo-shortcuts)): `` `quit` is not an action ``, ou `` `tab_0` is not an action: the tabs and the profiles count from 1 ``. O `match` escolle o texto, e un só `write_str` escríbeo.
- É `Copy`: non leva datos.

### Os atallos: `Entry` e `Shortcuts`

```rust
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Entry {
    pub chord: Chord,
    pub action: Option<Action>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Shortcuts {
    list: Vec<(Chord, Action)>,
}

impl Default for Shortcuts {
    fn default() -> Self {
        Self { list: defaults() }
    }
}
```

- **`Entry`**: unha entrada de `[shortcuts]`, coma a deixa `config.rs` ([`shortcuts`](#os-atallos-do-arquivo-shortcuts)): a combinación e a súa acción, ou `None` se a combinación queda libre, e a súa tecla vai ao programa. Os campos son públicos: `Config::shortcuts` garda unha lista delas.
- **`Shortcuts`**: os atallos dunha xanela, unha acción por cada combinación que colle meiga, nunha lista de pares. Unha lista, e non un mapa: buscar nela percorre unhas cincuenta combinacións, e só cando baixa unha tecla; en repouso, nada.
- **`Default`**: os atallos de meiga sen arquivo, os de [`defaults`](#os-atallos-de-meiga-defaults). Polo de agora só o usan as probas: o programa usa sempre `new`, tamén cunha lista baleira.

```rust
pub fn new(entries: &[Entry]) -> Self {
    let mut list = defaults();
    for entry in entries {
        list.retain(|(chord, _)| *chord != entry.chord);
        if let Some(action) = entry.action {
            list.push((entry.chord, action));
        }
    }
    Self { list }
}

pub fn action(&self, chord: Chord) -> Option<Action> {
    self.list
        .iter()
        .find(|(known, _)| *known == chord)
        .map(|&(_, action)| action)
}

pub fn list(&self) -> &[(Chord, Action)] {
    &self.list
}
```

**`new`**: os atallos de meiga cos de `entries` por riba, na súa orde ([ADR 0023](../adr/0023-os-atallos-de-teclado.md), decisión 1). Cada entrada quita primeiro calquera par coa súa combinación (`retain`) e despois, se ten acción, pon o seu ao final:

| Entrada | Que fai |
|---|---|
| Unha combinación nova cunha acción, `"ctrl+t" = "new_tab"` | Engádea. Ctrl+T abre unha tab, e Ctrl+Maiús+T tamén: unha acción que recibe outra combinación mantén as súas. |
| Unha combinación de meiga cunha acción, `"ctrl+shift+w" = "close_other_tabs"` | Cámbialle a acción. |
| Unha combinación de meiga con `none`, `"ctrl+6" = "none"` | Libéraa: a súa tecla vai ao programa. |
| Unha combinación que meiga non ten, con `none` | Nada. |

- **Cada combinación queda como moito unha vez**: as de meiga non se repiten, e cada entrada quita a súa antes de poñela. Se `entries` tivese a mesma combinación dúas veces, valería a última; pero `config::parse` só deixa a primeira de cada combinación, e as outras son erros (decisión 3).
- **`action`**: a acción de `chord`, se meiga a colle, ou `None`, e a tecla vai ao programa. `find` percorre a lista ata a súa combinación, e `map` copia a acción do par.
- **`list`**: todas as combinacións que colle meiga, e as súas accións. Polo de agora só a usan as probas.
- O programa fai os atallos dunha xanela unha vez, ao crear as súas tabs, coa configuración que leu ao arrancar, e dállos á vista do terminal ([Código › meiga › `shells.rs`](meiga.md#cratesmeigasrcshellsrs)). Un cambio no arquivo necesita reiniciar meiga, ata que chegue a recarga, na 8.4.

### Os atallos de meiga: `defaults`

```rust
fn defaults() -> Vec<(Chord, Action)> {
    const CTRL: u8 = Chord::CTRL;
    const SHIFT: u8 = Chord::SHIFT;
    let key = |modifiers, key: u8| Chord::new(modifiers, u16::from(key));
    let mut list = vec![
        (key(CTRL | SHIFT, b'T'), Action::NewTab),
        (key(CTRL | SHIFT, b'W'), Action::CloseTab),
        (Chord::new(CTRL, VK_TAB), Action::NextTab),
        (Chord::new(CTRL | SHIFT, VK_TAB), Action::PreviousTab),
        (Chord::new(CTRL, VK_NEXT), Action::NextTab),
        (Chord::new(CTRL, VK_PRIOR), Action::PreviousTab),
        (key(CTRL, b'9'), Action::LastTab),
        (Chord::new(CTRL | SHIFT, VK_SPACE), Action::ProfileMenu),
        (Chord::new(0, VK_APPS), Action::TabMenu),
        (Chord::new(SHIFT, VK_APPS), Action::TabMenu),
        (Chord::new(SHIFT, VK_F10), Action::TabMenu),
        (key(CTRL | SHIFT, b'R'), Action::RenameTab),
        (key(CTRL | SHIFT, b'D'), Action::DuplicateTab),
        (key(CTRL, b'C'), Action::Copy),
        (key(CTRL | SHIFT, b'C'), Action::Copy),
        (key(CTRL, b'V'), Action::Paste),
        (key(CTRL | SHIFT, b'V'), Action::Paste),
        (Chord::new(CTRL | SHIFT, VK_UP), Action::ScrollUp),
        (Chord::new(CTRL | SHIFT, VK_DOWN), Action::ScrollDown),
        (Chord::new(SHIFT, VK_PRIOR), Action::ScrollPageUp),
        (Chord::new(CTRL | SHIFT, VK_PRIOR), Action::ScrollPageUp),
        (Chord::new(SHIFT, VK_NEXT), Action::ScrollPageDown),
        (Chord::new(CTRL | SHIFT, VK_NEXT), Action::ScrollPageDown),
        (Chord::new(CTRL | SHIFT, VK_HOME), Action::ScrollToTop),
        (Chord::new(CTRL | SHIFT, VK_END), Action::ScrollToBottom),
        (Chord::new(CTRL, VK_OEM_PLUS), Action::ZoomIn),
        (Chord::new(CTRL, VK_ADD), Action::ZoomIn),
        (Chord::new(CTRL, VK_OEM_MINUS), Action::ZoomOut),
        (Chord::new(CTRL, VK_SUBTRACT), Action::ZoomOut),
        (key(CTRL, b'0'), Action::ZoomReset),
        (Chord::new(CTRL, VK_NUMPAD0), Action::ZoomReset),
    ];
    for place in 0..8 {
        list.push((key(CTRL, b'1' + place), Action::Tab(u16::from(place))));
    }
    for place in 0..9 {
        list.push((
            key(CTRL | SHIFT, b'1' + place),
            Action::Profile(u16::from(place)),
        ));
    }
    list
}
```

**`defaults`**: os atallos de meiga, os do Windows Terminal para as tabs, os perfís, os menús, copiar e pegar, o historial e o zoom; os de Chrome para mostrar unha tab polo seu sitio, a última, a anterior e a seguinte ([ADR 0023](../adr/0023-os-atallos-de-teclado.md), decisións 4 a 9); e, desde a 3.3, Ctrl+Maiús+R, que renomea a tab que se mostra ([ADR 0031](../adr/0031-o-nome-das-tabs.md), decisión 1):

| Combinacións | Acción |
|---|---|
| Ctrl+Maiús+T | `NewTab` |
| Ctrl+Maiús+W | `CloseTab` |
| Ctrl+Tab e Ctrl+AvPáx | `NextTab` |
| Ctrl+Maiús+Tab e Ctrl+RePáx | `PreviousTab` |
| Ctrl+1…Ctrl+8 | `Tab(0)`…`Tab(7)` |
| Ctrl+9 | `LastTab` |
| Ctrl+Maiús+1…Ctrl+Maiús+9 | `Profile(0)`…`Profile(8)` |
| Ctrl+Maiús+Espazo | `ProfileMenu` |
| A tecla de menú, só ou con Maiús, e Maiús+F10 | `TabMenu` |
| Ctrl+Maiús+R | `RenameTab` |
| Ctrl+Maiús+D | `DuplicateTab` |
| Ctrl+C e Ctrl+Maiús+C | `Copy` |
| Ctrl+V e Ctrl+Maiús+V | `Paste` |
| Ctrl+Maiús+↑ e Ctrl+Maiús+↓ | `ScrollUp` e `ScrollDown` |
| Maiús+RePáx e Ctrl+Maiús+RePáx | `ScrollPageUp` |
| Maiús+AvPáx e Ctrl+Maiús+AvPáx | `ScrollPageDown` |
| Ctrl+Maiús+Inicio e Ctrl+Maiús+Fin | `ScrollToTop` e `ScrollToBottom` |
| Ctrl e a tecla `+`, e Ctrl e o + do teclado numérico | `ZoomIn` |
| Ctrl e a tecla `-`, e Ctrl e o − do teclado numérico | `ZoomOut` |
| Ctrl+0, e Ctrl e o 0 do teclado numérico | `ZoomReset` |

- **48 combinacións** (47 ata a 3.2): as 31 do `vec!`, e as 8 de Ctrl+1…8 e as 9 de Ctrl+Maiús+1…9, que engaden os dous `for`. `b'1' + place` é o díxito de cada sitio, coma byte.
- **`key`**, unha *closure*, fai a combinación dunha letra ou dun díxito co seu byte en ASCII, coma `b'T'`, que é o seu código; as outras teclas van con `Chord::new` e as constantes `VK_…`. `CTRL` e `SHIFT` son nomes curtos para `Chord::CTRL` e `Chord::SHIFT`.
- **Todas as teclas que xa collía meiga** son atallos, e o arquivo pode cambialas (decisión 9): copiar e pegar, desde a 1.8; o historial e o zoom, desde a 1.9; Ctrl+Maiús+T, Ctrl+Maiús+W, Ctrl+Tab e Ctrl+Maiús+Tab, desde a 2.2; o menú da tab, desde a 2.4; e o dos perfís, desde a 2.5. Son novas Ctrl+1…9, Ctrl+Maiús+1…9, Ctrl+RePáx, Ctrl+AvPáx e Ctrl+Maiús+D.
- **Ctrl+T e Ctrl+W non están**: van ao programa (decisión 9). Tampouco están Esc, que, cunha selección, quítaa a vista antes de mirar os atallos, nin Maiús+Insert nin F10 só, que van ao programa.
- **Ningunha ten Alt**: as combinacións con Alt seguen indo ao programa, salvo as que o arquivo faga atallos (decisión 2, que concreta a decisión 5 do [ADR 0014](../adr/0014-o-teclado.md)). Ctrl+1…9 non levan Alt, porque Ctrl+Alt é o AltGr dos teclados que o usan para escribir, e só usan os díxitos da fila de arriba: os do teclado numérico pode engadilos o arquivo (decisión 5).
- **O zoom si ten o teclado numérico**, `VK_ADD`, `VK_SUBTRACT` e `VK_NUMPAD0`, coma desde a 1.9 ([ADR 0016](../adr/0016-o-historial-o-tamano-e-o-zoom.md), decisión 12).
- **`close_other_tabs` e `close_tabs_to_the_right` non teñen combinación**: o arquivo pode darlles unha; se non, fanse co menú da tab.
- Cada chamada fai unha lista nova: o programa chámaa unha vez por xanela, a través de `Shortcuts::new`.

**Probas.** Sete. `chord` le unha combinación que ten que estar ben, e `event` fai unha tecla que baixa, cun código e unhas teclas modificadoras, sen carácter nin código de exploración.

| Proba | Que comproba |
|---|---|
| `chords_are_written_as_in_windows_terminal` | `ctrl+shift+t` é Ctrl, Maiús e a tecla `0x54`, a T. `Shift + CTRL + T`, en calquera orde, con maiúsculas e con brancos, é o mesmo, e `ctrl+ctrl+t` é `ctrl+t`. `alt+shift+1`, `f1` e `ctrl+F24` (`0x87`) dan os seus códigos, e `numpad7` e `numpad_7`, `0x67`. `ctrl+pgup` e `ctrl+PageUp` son a mesma combinación, coma `menu` e `app`, e `ctrl+numpad_add` e `ctrl+numpad_plus`; e `ctrl+plus` é Ctrl e `VK_OEM_PLUS`. Alt+F4 con Ctrl, `ctrl+alt+f4`, vale, porque non é o de Windows; e cada nome de `NAMES`, en maiúsculas, dá o seu código. |
| `wrong_chords_say_why` | `""`, `ctrl+` e `ctrl++` teñen unha parte baleira; `ctrl+shift`, ningunha tecla; e `ctrl+a+b`, dúas. `hyper`, `ñ`, `f0`, `f25` e `numpad10` non son teclas; e, coma di o comentario, tampouco o son `f65535` e `numpad65535`, cuns números que non caben despois do primeiro código: sen o `match` de [`key_code`](#o-código-dunha-tecla-key_code-strip_prefix_ignore_case-e-number), a súa suma faría un *panic* na proba. `win+e` ten unha tecla Windows; e `alt+f4` e `alt+shift+space` son de Windows. E a mensaxe de `hyper`: `` `hyper` is not a key ``. |
| `a_key_that_goes_down_has_the_chord_of_its_modifier_keys_but_altgr_none` | O Ctrl esquerdo e Maiús, co Bloq Num activado, coa T, son `ctrl+shift+t`; e o Ctrl dereito e o Alt esquerdo co 1, `ctrl+alt+1`. AltGr, o Ctrl esquerdo e o Alt dereito, co 1, non ten combinación, nin con Maiús; pero os dous Ctrl co Alt dereito, si: `ctrl+alt+1`. Un carácter sen tecla, a tecla 0, tampouco ten. |
| `actions_are_read_by_their_names` | `none` libera a combinación; `new_tab`, `tab_1`, `tab_12` e `profile_9` son `NewTab`, `Tab(0)`, `Tab(11)` e `Profile(8)`; `tab_0` e `profile_0` din que se contan desde 1; `tab_`, `tab_+1`, `tab_x`, `tab_99999`, `New_Tab`, `None` e un nome baleiro non son accións; e cada nome de `ACTIONS` dá a súa acción. |
| `the_shortcuts_of_meiga_leave_ctrl_t_and_ctrl_w_to_the_program` | Os atallos de meiga: Ctrl+Maiús+T e Ctrl+Maiús+W, pero non Ctrl+T nin Ctrl+W; Ctrl+Tab, Ctrl+Maiús+Tab, Ctrl+AvPáx e Ctrl+RePáx; Ctrl+1 e Ctrl+8, a primeira e a oitava tab, e Ctrl+9, a última; Ctrl+Maiús+1 e Ctrl+Maiús+9, o primeiro e o noveno perfil, pero non Ctrl+Alt+1; Ctrl+Maiús+D e Ctrl+Maiús+Espazo; desde a 3.3, Ctrl+Maiús+R, pero non Ctrl+R; a tecla de menú, só e con Maiús, e Maiús+F10, pero non F10; Ctrl+C e Ctrl+Maiús+V, pero non Maiús+Insert; Maiús+RePáx, Ctrl+Maiús+AvPáx e Ctrl+Maiús+Inicio; Ctrl e o − e o 0 do teclado numérico; e non Ctrl+Maiús e a tecla `+`. E ningunha combinación ten dúas accións. |
| `the_file_adds_changes_and_frees_shortcuts` | Catro entradas: `ctrl+t` con `NewTab`, que se engade, e Ctrl+Maiús+T segue abrindo unha tab; `ctrl+6` sen acción, que se libera, mentres Ctrl+5 segue mostrando a quinta tab; `ctrl+shift+w` con `CloseOtherTabs`, que cambia de acción; e `ctrl+alt+x` sen acción, que meiga non tiña, e non cambia nada. A lista ten tantos atallos coma a de meiga: un máis e un menos. |
| `only_moving_between_tabs_the_history_and_the_zoom_repeat` | Todas as accións con nome, as 22 de `ACTIONS`: `repeats` é certo se, e só se, o nome está en `repeating`, `next_tab`, `previous_tab`, as seis `scroll_…`, `zoom_in` e `zoom_out`; as outras doce, coma `new_tab`, `rename_tab`, `paste` ou `zoom_reset`, non repiten. Se unha falla, a mensaxe di o seu nome. E `Tab(0)` e `Profile(0)`, que non teñen un nome en `ACTIONS`, tampouco repiten. |

**Como se comproba.** Ademais destas probas, `check-shortcuts.ps1` tecléalle a meiga na CI as combinacións, cun `config.toml` que libera Ctrl+6, dálle a Ctrl+Alt+2 a segunda tab e a Ctrl+8 a última, no canto da oitava, e a Alt+O e a Alt+R `close_other_tabs` e `close_tabs_to_the_right`, dúas accións sen combinación de seu ([Código › ferramentas › `check-shortcuts.ps1`](ferramentas.md#toolscheck-shortcutsps1)). O programa de cada tab apunta os códigos das teclas que recibe, ao baixar e ao subir, e a comprobación mira, entre outras cousas, que Ctrl+T, Ctrl+W e Ctrl+6 chegan enteiras ao programa; que Ctrl+Maiús+T abre unha soa tab mentres a T repite; que AltGr+2, o Ctrl esquerdo e o Alt dereito, chega ao programa, mentres Ctrl+Alt+2, co Alt esquerdo, mostra a segunda tab; que Ctrl+Maiús+D duplica a tab que se mostra, Ctrl+1, Ctrl+2 e Ctrl+3 mostran as tabs deses sitios, e Ctrl+9 a última; que Ctrl+4, con tres tabs, e Ctrl+Maiús+3, con dous perfís, non fan nada, e o seu díxito non chega ao programa; que Ctrl+AvPáx e Ctrl+RePáx dan a volta á barra, e Ctrl+RePáx segue mentres repite; que Ctrl+Maiús+2 e Ctrl+Maiús+1 abren os perfís deses sitios do menú; que Ctrl+8 mostra a última de cinco tabs; e que Alt+R pecha as tabs da dereita e Alt+O as outras, e o programa só recibe o Alt. As copias de meiga estragadas a propósito que collen Ctrl+T, que non liberan as combinacións con `none`, que non engaden as entradas do arquivo ou deixan diante as de meiga, que non miran Maiús ou AltGr en `Chord::of`, nas que `new_tab` repite ou `previous_tab` non, ou sen Ctrl+Maiús+D, Ctrl+RePáx e Ctrl+AvPáx, ou con Ctrl+9 coma a novena tab, fallan a comprobación. Desde a 3.3, [`check-rename.ps1`](ferramentas.md#toolscheck-renameps1) comproba que Ctrl+Maiús+R abre o campo do nome da tab que se mostra, e que, mentres está aberto, Ctrl+Maiús+T non abre ningunha tab.

## `crates/meiga-core/src/model/mod.rs`

Desde a subfase 2.1, o modelo das xanelas e as súas tabs: que tabs ten cada xanela, en que orde, cal está activa, e que pasa ao abrir, pechar, mover ou activar unha, sen ningunha interface ([ADR 0018](../adr/0018-o-modelo-das-tabs.md)). Na 2.1 só o usan as súas probas e o exemplo `tabs`; desde a 2.2, o programa terá unha sesión e unha vista en cada tab. Desde a 2.4, pecha tamén varias tabs dunha vez, para o menú dunha tab ([ADR 0021](../adr/0021-os-menus-e-o-menu-da-tab.md), decisión 11).

```rust
mod window;

#[cfg(test)]
mod tests;

pub use window::{Close, Closed, Switch, Tab, Window};
```

- O módulo ten dous arquivos máis: `window.rs`, coa xanela e as súas tabs, e `tests.rs`, coas probas, que só se compilan con `cargo test` (`#[cfg(test)]`).
- `pub use` publica os tipos de `window.rs` coma se estivesen eiquí: fóra do crate úsanse coma `meiga_core::model::Window`, sen saber en que arquivo están. `Closed` chegou na 2.4.
- O comentario do módulo (`//!`) explica as tres preguntas que o modelo responde por separado ([ADR 0018](../adr/0018-o-modelo-das-tabs.md), decisión 9): a tab activa, a que marca a barra (`Window::active`); a enfocada, a que recibe o teclado (`Window::focused`); e as visíbeis, as que se debuxan (`Window::is_visible`).

### `TabId` e `WindowId`

```rust
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct TabId(u64);

impl TabId {
    fn next() -> Self {
        static NEXT: AtomicU64 = AtomicU64::new(1);
        Self(take(&NEXT))
    }

    pub const fn get(self) -> u64 {
        self.0
    }

    pub const fn from_raw(number: u64) -> Self {
        Self(number)
    }
}

fn take(counter: &AtomicU64) -> u64 {
    counter.fetch_add(1, Ordering::Relaxed)
}
```

- **`TabId`** identifica unha tab mentres meiga corre. É un `u64` dentro dun tipo propio (un *newtype*): o compilador non deixa confundilo cun número calquera nin cun `WindowId`. O campo é privado: fóra do módulo, un identificador só se fai a partir dun número, con `from_raw`.
- **`next`** dá o seguinte número dun contador `static`, que existe unha soa vez en todo o programa e empeza en 1. É privada: só o modelo dá números novos, cando crea unha tab.
- **`take`** súmalle 1 ao contador e devolve o valor de antes nunha soa operación atómica: dous fíos que abran tabs á vez nunca reciben o mesmo número, e unha tab pechada non devolve o seu. Chega a orde de memoria `Relaxed`, porque só importa que os números non se repitan ([Tecnoloxías › Xenéricos e contadores atómicos](../tecnoloxias.md#53-xenéricos-e-contadores-atómicos)).
- **`get`** dá o número, por exemplo para envialo nunha mensaxe a outro fío (2.2), e **`from_raw`**, desde a revisión da 2.1, volta facer o identificador co número que chega: as mensaxes de Windows levan números, e non tipos de Rust. Un número inventado non rompe o modelo: o modelo dálle a cada tab nova un número do seu contador e só el pon tabs na xanela, así que `from_raw` non pode crear unha tab nin facer que dúas compartan un número, e un número que non é dunha tab da xanela non atopa nada (`Error::NoSuchTab`).
- Os `derive`: `Clone` e `Copy` copian o identificador coma un número; `Debug` móstrao nas mensaxes das probas; `PartialEq` e `Eq` compáranos; e `Hash` deixa usalos de chave nun mapa, coma fan as probas.
- **`WindowId`** é igual para as xanelas, co seu propio contador. Polo de agora só o garda cada `Window` (`Window::id`), porque meiga ten unha xanela: prepara as varias xanelas (F6).

### `Error`

```rust
pub enum Error {
    NoSuchTab(TabId),
    OutOfRange { index: usize, count: usize },
}
```

Por que unha xanela rexeitou un cambio. A xanela queda como estaba.

- **`NoSuchTab`:** a xanela non ten esa tab, porque xa se pechou ou porque é doutra xanela.
- **`OutOfRange`:** a posición, contada desde 0, está despois da última tab. `index` é a posición pedida, e `count`, cantas tabs hai.

`Display` escribe o erro en inglés, coma as outras mensaxes de meiga: `the window has no tab 7`, `position 3 is out of range for 3 tabs` ou, cunha tab, `for 1 tab`. E implementa `std::error::Error`, o *trait* dos erros de Rust, así que pode ir xunto con outros erros, por exemplo nun `Box<dyn Error>`.

## `crates/meiga-core/src/model/window.rs`

A xanela e as súas tabs.

### `Tab`, `Switch` e `Close`

```rust
pub struct Tab<T> {
    id: TabId,
    content: T,
}

pub struct Switch {
    pub from: TabId,
    pub to: TabId,
}

#[must_use = "the program has to close the content of the tab and show the tab that is active now"]
pub enum Close<T> {
    Removed {
        content: T,
        activated: Option<TabId>,
    },
    LastTab,
}
```

- **`Tab<T>`** é unha tab: o seu identificador e o seu contido, do tipo que escolla o programa ([Tecnoloxías › Xenéricos e contadores atómicos](../tecnoloxias.md#53-xenéricos-e-contadores-atómicos)). Os campos son privados: fóra do módulo só se len, con `id()` e `content()`. Só o modelo crea tabs, ao crear a xanela e ao abrir unha ([Abrir e pechar](#abrir-e-pechar)).
- **`Switch`:** a tab activa cambiou, de `from` a `to`. Desde a 2.2, o programa redebuxará todo e cambiará a prioridade dos dous fíos lectores.
- **`Close<T>`,** o que fixo pechar unha tab: `Removed`, a tab saíu da xanela, co seu contido e, se era a activa, a tab que ocupa o seu lugar (`activated`); ou `LastTab`, era a única, segue na xanela, e a xanela ten que pecharse con ela. Ao pechar varias tabs dunha vez, desde a 2.4, a resposta é un [`Closed`](#pechar-varias-tabs-closed-close_others-e-close_to_the_right).
- **`#[must_use]`** en `Close`, desde a revisión da 2.1: o compilador avisa se quen pecha unha tab ignora a resposta, que pode traer a sesión da tab e a tab que pasa a ser a activa, ou pedir que se peche a xanela. Por ir no tipo, avisa tamén cando o erro sae cun `?`: en `window.close(id)?;`, o `?` usa o `Result`, pero o `Close` que queda dentro ignórase, e iso si o ve o compilador.
- `Close` deriva `PartialEq` e `Eq`, se o contido os ten: as probas comparan a resposta enteira cun `assert_eq!`.

### `Window`: os datos e as consultas

```rust
pub struct Window<T> {
    id: WindowId,
    tabs: Vec<Tab<T>>,
    active: usize,
}
```

- **`tabs`:** as tabs, na orde da barra, nun vector, contiguas en memoria. Nunca está baleiro.
- **`active`:** a posición da tab activa en `tabs`. Gárdase a posición, e non o identificador, porque o programa consultará a activa en cada tecla e en cada fotograma: así é un acceso directo ([ADR 0018](../adr/0018-o-modelo-das-tabs.md), decisión 11).
- Os campos son privados, e ningún método deixa a xanela sen tabs nin a activa fóra delas. Son as súas **invariantes**, as regras que sempre se cumpren, e as probas compróbanas despois de cada cambio.

| Método | Que fai |
|---|---|
| `new(content)` | Crea unha xanela cunha tab, a activa, con ese contido. |
| `try_new_with(content)` | Igual, pero o contido faino unha función que recibe o identificador da tab; se falla, devolve o seu erro, e non hai xanela ([Abrir e pechar](#abrir-e-pechar)). |
| `id()` | O identificador da xanela. |
| `tabs()` | As tabs, na orde da barra, nunha *slice* para lelas. |
| `tab_count()` | Cantas tabs ten: polo menos unha. Chámase así, e non `len`, porque Clippy pide un `is_empty` xunto a cada `len`, e unha xanela nunca está baleira. |
| `index_of(id)` | A posición dunha tab, desde 0, ou `None` se non está. |
| `get(id)` e `get_mut(id)` | O contido dunha tab, para lelo ou para cambialo. |
| `contents_mut()` | Desde a 2.2: o contido de todas as tabs, cos seus identificadores, na orde da barra, para cambialo. As tabs e a súa orde non cambian. O programa úsao coas tabs que non se ven ([Código › meiga › `tabs.rs`](meiga.md#cratesmeigasrctabsrs)). |
| `active()` e `active_index()` | A tab activa, e a súa posición. |
| `focused()` e `focused_mut()` | A tab enfocada, a que recibe o teclado, e o seu contido, para cambialo. Hoxe, a activa. |
| `is_visible(id)` | Se unha tab se debuxa. Hoxe, só a activa. |

- Buscar polo identificador (`index_of`, `get` e `get_mut`) percorre o vector ata atopala, con `position` e `find`: con decenas de tabs, uns nanosegundos.
- `active()` le o vector na posición gardada (`&self.tabs[self.active]`), que as invariantes manteñen dentro del.
- `focused` e `is_visible` respóndense coa activa: o modelo non garda nada máis ata a vista dividida (decisión 9). Na 2.2, o programa só mostra unha tab, e usa `active` para todo; `focused` e `is_visible` quedan para cando a enfocada e as visíbeis poidan ser outras.
- Non hai un `tabs_mut`: cunha *slice* mutábel, o programa podería cambiar a orde das tabs sen que a xanela o soubese, e a posición da activa apuntaría a outra tab. O contido cámbiase con `get_mut`, `focused_mut` e, desde a 2.2, `contents_mut`, que só dan o contido: `contents_mut` dá pares `(TabId, &mut T)`, cun iterador sobre o vector (`iter_mut().map(…)`), sen reservar memoria.

### Abrir e pechar

```rust
pub fn open(&mut self, content: T) -> TabId {
    self.push(Tab::new(content))
}

pub fn try_open_with<E>(
    &mut self,
    content: impl FnOnce(TabId) -> Result<T, E>,
) -> Result<TabId, E> {
    Ok(self.push(Tab::try_with(content)?))
}

fn push(&mut self, tab: Tab<T>) -> TabId {
    let id = tab.id;
    self.tabs.push(tab);
    id
}
```

**Abrir** pon a tab nova ao final da barra (decisión 6, decidido contigo) e devolve o seu identificador. Non a activa: o botón + e Ctrl+Maiús+T actívana despois con `activate`, e a restauración da sesión (7.2) abrirá as tabs sen activalas.

```rust
fn try_with<E>(content: impl FnOnce(TabId) -> Result<T, E>) -> Result<Self, E> {
    let id = TabId::next();
    Ok(Self {
        id,
        content: content(id)?,
    })
}
```

**Abrir cun contido que sabe o seu identificador,** desde a revisión da 2.1: `try_open_with` e `try_new_with`, a da xanela nova, reciben unha función que fai o contido, e `Tab::try_with` dálle o identificador da tab antes de que a tab exista. Desde a 2.2, esa función creará a sesión, cuxo fío lector enviará o identificador cos seus avisos, así que o necesita ao nacer.

- `impl FnOnce(TabId) -> Result<T, E>` acepta calquera función ou *closure* que reciba un `TabId` e devolva o contido ou un erro ([Tecnoloxías › Xenéricos e contadores atómicos](../tecnoloxias.md#53-xenéricos-e-contadores-atómicos)). `FnOnce` chámase unha vez, así que pode levar dentro o que move ao contido. O tipo do erro, `E`, escólleo quen chama: o modelo non sabe que é unha sesión nin como falla.
- Se a función falla, o `?` devolve o erro antes de `push`, e a xanela queda como estaba; a xanela nova non chega a existir. O identificador pérdese: o contador non o devolve, coma cando se pecha unha tab, e con 64 bits non se esgota.
- `open` e `new` usan `Tab::new`, co contido xa feito, e as catro maneiras de crear unha tab rematan igual: só `push` e `with_tab`, a da xanela nova, poñen tabs na xanela, sempre recén creadas e co seu identificador novo.

```rust
pub fn close(&mut self, id: TabId) -> Result<Close<T>, Error> {
    let index = self.find(id)?;
    if self.tabs.len() == 1 {
        return Ok(Close::LastTab);
    }
    let tab = self.tabs.remove(index);
    let activated = match index.cmp(&self.active) {
        Ordering::Less => {
            self.active -= 1;
            None
        }
        Ordering::Greater => None,
        Ordering::Equal => {
            self.active = index.min(self.tabs.len() - 1);
            Some(self.tabs[self.active].id)
        }
    };
    Ok(Close::Removed {
        content: tab.content,
        activated,
    })
}
```

**Pechar:**

1. `find` busca a posición da tab ou, se non está, devolve `Error::NoSuchTab`; o `?` devolve o erro no momento, sen cambiar nada.
2. Se é a única tab, non se quita: `Close::LastTab`, e a xanela pecharase con ela (decisión 4).
3. Se non, `remove` sácaa do vector, e as que estaban á súa dereita corren un posto cara á esquerda. A posición da activa axústase comparando a da tab pechada coa da activa (`cmp`):
   - **á esquerda da activa** (`Less`): a activa correu un posto, e a súa posición baixa un;
   - **á dereita** (`Greater`): nada cambia;
   - **era a activa** (`Equal`): a tab da súa dereita ocupou a súa posición e pasa a ser a activa, coma en Chrome (decisión 5, decidido contigo). Se era a última, a posición quedou despois da nova última, e `min` colle esta, a que estaba á súa esquerda.
4. A resposta leva o contido da tab, para que o programa peche a súa sesión, e a tab activada, se cambiou.

### Pechar varias tabs: `Closed`, `close_others` e `close_to_the_right`

Desde a subfase 2.4, para os elementos *Close other tabs* e *Close tabs to the right* do menú dunha tab ([ADR 0021](../adr/0021-os-menus-e-o-menu-da-tab.md), decisión 11). Concreta o [ADR 0018](../adr/0018-o-modelo-das-tabs.md): o modelo pecha varias tabs á vez, e di cal queda activa.

```rust
#[derive(Debug, PartialEq, Eq)]
#[must_use = "the program has to close the contents of the tabs and show the tab that is active now"]
pub struct Closed<T> {
    pub contents: Vec<T>,
    pub activated: Option<TabId>,
}
```

- **`Closed<T>`,** o que fixo pechar varias tabs dunha vez, coma `Close::Removed` para unha: `contents`, o contido das tabs que saíron da xanela, na orde da barra, para que o programa peche as súas sesións; e `activated`, se a activa era unha delas, a tab que é a activa agora.
- Non hai un `LastTab`: estes dous métodos nunca pechan todas as tabs, porque a que reciben sempre queda.
- **`#[must_use]`**, coma en `Close` e polas mesmas razóns: o compilador avisa se quen pecha as tabs ignora a resposta, tamén cando o erro sae cun `?` ([`Tab`, `Switch` e `Close`](#tab-switch-e-close)).
- Os campos son públicos, coma os de `Switch`: é só unha resposta, que o programa desfai cun patrón, `Closed { contents, activated }` ([Código › meiga › `tabs.rs`](meiga.md#cratesmeigasrctabsrs)). Deriva `PartialEq` e `Eq`, se o contido os ten, coma `Close`, para que as probas comparen a resposta enteira.

```rust
pub fn close_others(&mut self, keep: TabId) -> Result<Closed<T>, Error> {
    let index = self.find(keep)?;
    let activated = (index != self.active).then_some(keep);
    // The tab is taken out, and put back in the same memory.
    let kept = self.tabs.remove(index);
    let contents = self.tabs.drain(..).map(|tab| tab.content).collect();
    self.tabs.push(kept);
    self.active = 0;
    Ok(Closed {
        contents,
        activated,
    })
}
```

**Pechar as outras** pecha todas as tabs menos `keep`, tamén as da súa esquerda, coma *Close other tabs* en Chrome e no Windows Terminal. `keep` queda soa, e activa:

1. `find` busca a posición da tab ou, se non está, devolve `Error::NoSuchTab` sen cambiar nada, coma en `close`.
2. Se `keep` non era a activa, pasa a selo, e a resposta dío: `then_some` dá `Some(keep)` se a condición se cumpre e, se non, `None`.
3. `remove` saca `keep` do vector, e `drain(..)` saca todas as demais, en orde, nun iterador: `map` quédase co contido de cada unha, e `collect` fai con eles o vector da resposta.
4. `keep` volta ao vector, que quedou baleiro pero conserva a súa memoria: coma di o comentario, `push` non reserva nada. É a única tab, na posición 0, e a activa.

Cunha soa tab, non pecha nada: `contents` vai baleiro, e `activated`, `None`. O menú tampouco o deixa escoller: cunha soa tab, *Close other tabs* está desactivado ([ADR 0021](../adr/0021-os-menus-e-o-menu-da-tab.md), decisión 2).

```rust
pub fn close_to_the_right(&mut self, id: TabId) -> Result<Closed<T>, Error> {
    let index = self.find(id)?;
    let contents = self
        .tabs
        .drain(index + 1..)
        .map(|tab| tab.content)
        .collect();
    let activated = (self.active > index).then(|| {
        self.active = index;
        id
    });
    Ok(Closed {
        contents,
        activated,
    })
}
```

**Pechar as da dereita** pecha as tabs que están á dereita de `id`, coma *Close tabs to the right* en Chrome e no Windows Terminal:

1. `find`, coma en `close_others`.
2. `drain(index + 1..)` saca as tabs despois de `id`, e `collect` garda o seu contido. `id` e as da súa esquerda quedan onde estaban.
3. Se a activa era unha das que saíron, a súa posición estaba despois de `index`: a activa pasa a ser `id`, a máis próxima das que quedan, e a resposta dío. `then` só chama a *closure*, que cambia a posición, se a condición se cumpre; `then_some`, en `close_others`, recibe o valor xa feito. Se a activa era `id` ou estaba á súa esquerda, segue sendo a mesma, na mesma posición.

Desde a última tab, non pecha nada; tampouco se pode escoller: na última tab, *Close tabs to the right* está desactivado.

- **Nun paso.** O modelo quita as tabs dunha vez e responde unha vez, así que o programa mostra como moito unha tab, a que queda activa, nun único fotograma completo; se a activa non cambiou, só debuxa outra vez a barra. As sesións péchanse sen esperar, cando o programa solta os contidos, coma ao pechar unha tab ([ADR 0021](../adr/0021-os-menus-e-o-menu-da-tab.md), decisión 11).
- Ningún dos dous deixa a xanela sen tabs nin a activa fóra delas: as invariantes seguen ([`Window`: os datos e as consultas](#window-os-datos-e-as-consultas)).

### Mover

```rust
pub fn move_tab(&mut self, id: TabId, to: usize) -> Result<(), Error> {
    let from = self.find(id)?;
    let count = self.tabs.len();
    if to >= count {
        return Err(Error::OutOfRange { index: to, count });
    }
    if from < to {
        self.tabs[from..=to].rotate_left(1);
    } else {
        self.tabs[to..=from].rotate_right(1);
    }
    self.active = moved(self.active, from, to);
    Ok(())
}
```

- `move` é unha palabra reservada de Rust, así que o método chámase `move_tab`.
- Unha posición despois da última devolve `Error::OutOfRange` antes de cambiar nada.
- **Rotar** unha parte do vector move a tab sen sacala nin metela, e sen reservar memoria: cara á dereita, `rotate_left(1)` sobre as posicións de `from` a `to` leva a primeira, a tab que se move, ao final, e corre as demais un posto cara á esquerda; cara á esquerda, `rotate_right(1)` sobre as de `to` a `from` fai o contrario. Para mover unha tab á súa propia posición, a parte ten unha tab, e nada cambia.

```rust
const fn moved(index: usize, from: usize, to: usize) -> usize {
    if index == from {
        to
    } else if from < index && index <= to {
        index - 1
    } else if to <= index && index < from {
        index + 1
    } else {
        index
    }
}
```

**`moved`** di onde acaba a tab que estaba en `index`: a que se move, en `to`; as que estaban entre as dúas posicións, un posto á esquerda se a tab foi cara á dereita, ou á dereita se foi cara á esquerda; e as demais, onde estaban. A xanela úsaa coa posición da activa, que así segue sendo a mesma tab (decisión 7). É unha `const fn`, porque só fai contas.

### Activar

```rust
pub fn activate(&mut self, id: TabId) -> Result<Option<Switch>, Error> {
    let index = self.find(id)?;
    Ok(self.activate_at(index))
}

#[must_use = "the program has to show the tab that is active now"]
pub fn activate_next(&mut self) -> Option<Switch> {
    self.activate_at((self.active + 1) % self.tabs.len())
}

#[must_use = "the program has to show the tab that is active now"]
pub fn activate_previous(&mut self) -> Option<Switch> {
    let count = self.tabs.len();
    self.activate_at((self.active + count - 1) % count)
}

fn activate_at(&mut self, index: usize) -> Option<Switch> {
    if index == self.active {
        return None;
    }
    let from = self.tabs[self.active].id;
    self.active = index;
    Some(Switch {
        from,
        to: self.tabs[index].id,
    })
}
```

- **`activate`** activa unha tab polo seu identificador; **`activate_next`** e **`activate_previous`**, a seguinte e a anterior. O resto da división (`%`) dá a volta: desde a última, a seguinte é a primeira, e desde a primeira, a anterior é a última. Para a anterior, súmase `count` antes de restar 1, para non baixar de 0. Cunha soa tab, as dúas dan a mesma tab, e nada cambia.
- **`activate_at`** fai o cambio e responde cun `Switch`, de que tab a que tab, ou con `None` se xa era a activa.
- **`#[must_use]`** en `activate_next` e `activate_previous` fai que o compilador avise se quen chama ignora a resposta: desde a 2.2, quen active unha tab terá que mostrala. `activate` devolve un `Result`, que o compilador tampouco deixa ignorar enteiro; pero, se o erro sae cun `?` (`window.activate(id)?;`), o `Option<Switch>` que queda dentro pódese ignorar sen aviso. Un `#[must_use]` en `Switch` non o evitaría: o aviso dun tipo non pasa a través de `Option`. `Close` si o leva, porque se devolve sen `Option` ([`Tab`, `Switch` e `Close`](#tab-switch-e-close)). O exemplo `tabs`, que non debuxa, descarta a resposta a propósito, con `let _ =` e con `map(|_| ())`.

## `crates/meiga-core/src/model/tests.rs`

As probas do modelo: vinte e catro, que levan uns milisegundos.

```rust
fn check<T>(window: &Window<T>) {
    assert!(window.tab_count() >= 1, "a window without tabs");
    assert_eq!(window.tabs().len(), window.tab_count());
    assert!(window.active_index() < window.tab_count());
    let active = window.active().id();
    assert_eq!(window.tabs()[window.active_index()].id(), active);
    assert_eq!(window.focused().id(), active);
    let ids: HashSet<TabId> = window.tabs().iter().map(Tab::id).collect();
    assert_eq!(ids.len(), window.tab_count(), "a repeated identifier");
    for (index, tab) in window.tabs().iter().enumerate() {
        assert_eq!(window.index_of(tab.id()), Some(index));
        assert_eq!(window.is_visible(tab.id()), tab.id() == active);
    }
}
```

- **`check`** comproba as invariantes: polo menos unha tab, a activa entre elas, a enfocada igual ca a activa, ningún identificador repetido, a posición de cada tab igual ca a que dá `index_of`, e só a activa visíbel. Cada proba chámaa despois de cada cambio, tamén dos que a xanela rexeita.
- **`window(&["a", "b", "c"])`** crea unha xanela cunha tab por nome, a primeira activa, e devolve tamén os seus identificadores; tamén comproba as invariantes, ao crear a xanela e despois de abrir cada tab. **`names`** dá os contidos de esquerda a dereita, para comparalos cunha lista. **`removed`** e **`switched`** escriben as respostas de pechar e de activar, e, desde a 2.4, **`removed_tabs`**, a de pechar varias tabs.

Vinte e tres probas comproban cada regra:

- unha xanela nova ten unha tab, e é a activa; unha tab ábrese ao final, e non se activa;
- un contido pode saber o identificador da súa tab, ao crear a xanela e ao abrir unha, e un contido que non se pode facer non abre nada; un identificador volta do seu número, e o número 0, que o contador nunca dá, non atopa ningunha tab;
- os identificadores nunca se repiten, nin noutra xanela nin despois de pechar unha tab;
- pechar a activa activa a da dereita ou, se era a última, a da esquerda; pechar outra deixa a activa, tamén cando a pechada estaba á súa esquerda e a activa corre un posto; e a única tab queda, e a xanela péchase con ela;
- desde a 2.4, pechar varias dunha vez: pechar as outras deixa unha soa tab, que pasa a ser a activa se non o era, e responde o contido das demais, na orde da barra; pechar as da dereita activa a tab se a activa era unha delas, e deixa a activa se estaba á esquerda; e cunha soa tab, ou desde a última, non pechan nada;
- mover corre as tabs do medio, e a activa segue sendo a mesma tab: cando se move ela, cando outra a cruza desde a esquerda ou desde a dereita, e cando outra se move sen cruzala;
- unha posición despois da última, unha tab xa pechada e unha tab doutra xanela rexéitanse, e a xanela queda igual; desde a 2.4, tamén unha tab xa pechada ao pechar as outras ou as da dereita;
- activar responde o cambio, ou nada se xa era a activa; a seguinte e a anterior dan a volta; e cunha soa tab non cambian nada;
- o teclado e a pantalla seguen a activa (`focused`, `focused_mut` e `is_visible`), e `get_mut` cambia o contido doutra tab;
- e as mensaxes dos erros.

**A proba ao chou** (`random_changes_keep_the_rules_and_match_a_plain_list`) fai 500 cambios en cada unha de 20 xanelas, 10 000 en total: abrir, con `open` ou con `try_open_with`, e ás veces despois dun intento cun contido que non se pode facer; pechar; mover (ás veces a unha posición despois da última); activar unha tab, a seguinte ou a anterior; pechar ou activar unha tab xa pechada; e, desde a 2.4, pechar todas as tabs menos unha, ou as da dereita dunha, cada un nun cambio de cada 64: poucas veces, para que a barra poida medrar. Os cambios saen dunha secuencia *xorshift* fixa, coma nas probas de `damage.rs` e do terminal: se falla, falla sempre igual. Fai os mesmos cambios nunha lista simple, cos contidos das tabs e o contido da activa, coas regras escritas da forma máis directa: ao pechar as outras, a lista queda só coa tab que se mantén, e ao pechar as da dereita, córtase despois dela (`split_off`). Os identificadores das tabs pechadas de varias en varias van tamén á lista das pechadas (`closed_ids`), coas que se proba despois pechar e activar unha tab xa pechada. Despois de cada cambio comproba as invariantes, que a xanela e a lista coinciden, e que as respostas son as que tocan. Ao final comproba que a barra pasou por unha tab, por dúas e por dez ou máis, para que os extremos se proben tamén.

Comprobouse tamén ao revés, con catro copias do modelo estragadas a propósito: unha activa a tab da esquerda ao pechar a activa, outra non corre a activa ao pechar unha tab á súa esquerda, outra axusta mal a activa ao mover, e a última non dá a volta coa seguinte. Despois da revisión, outras dúas: unha dálle ao contido outro identificador ca o da súa tab, e outra fai mal o identificador co seu número. As seis fallan as probas.

## `crates/meiga-core/src/term/mod.rs`

O terminal: a fachada sobre `alacritty_terminal` ([ADR 0003](../adr/0003-nucleo-vt-alacritty-terminal.md) e [ADR 0012](../adr/0012-fachada-do-terminal.md)). meiga dálle o que escribe o programa e le del o que hai que mostrar e o que hai que responder, en tipos propios. Desde a subfase 1.6 ten tamén a paleta coa que se debuxan as celas, e responde ás preguntas polas cores e polo tamaño en píxeles ([ADR 0013](../adr/0013-da-saida-a-pantalla.md)). Desde a 1.8 ten a selección, que vive na grella de `alacritty_terminal` e que a copia do que cambia marca nas celas ([ADR 0015](../adr/0015-o-rato-a-seleccion-e-o-portapapeis.md)). Desde a 1.9, cando cambia o tamaño, coloca a pantalla onde a pon o ConPTY e garda á parte a pantalla principal mentres se ve a alternativa, e move a vista do historial ([ADR 0016](../adr/0016-o-historial-o-tamano-e-o-zoom.md)). Desde a 2.7, para o programa que empeza outra vez na mesma tab, volta ao estado no que o espera un programa novo, co que escribiu o anterior no historial; e di se o cursor está ao principio da súa fila, para a liña do fin do programa ([ADR 0024](../adr/0024-o-fin-do-programa-e-o-reinicio.md)). Desde a 3.1, leva el mesmo o título do programa e os que este garda, que xa non lle chegan a `alacritty_terminal` ([ADR 0029](../adr/0029-o-titulo-da-tab-e-os-titulos-gardados.md), [`term/title.rs`](#cratesmeiga-coresrctermtitlers)). Desde a 4.1, garda o que di a shell de si mesma, que atopa o escáner ([ADR 0032](../adr/0032-o-escaner-osc.md), [`term/shell_state.rs`](#cratesmeiga-coresrctermshell_staters)), e segue el mesmo o win32-input-mode, no [`Hooked`](#cratesmeiga-coresrctermhandlerrs). Só este módulo usa os tipos de `alacritty_terminal`: cambialo por outro núcleo, ou actualizalo a unha versión cunha API distinta, só cambia este módulo. Como encaixa co resto, en [Arquitectura › O terminal](../arquitectura.md#o-terminal-da-saída-ao-que-se-mostra), [› Da saída á pantalla](../arquitectura.md#da-saída-á-pantalla-o-aviso-a-copia-e-o-fotograma), [› O rato, a selección e o portapapeis](../arquitectura.md#o-rato-a-selección-e-o-portapapeis), [› O historial, o tamaño e o zoom](../arquitectura.md#o-historial-o-tamaño-e-o-zoom) e [› O fin do programa e o reinicio](../arquitectura.md#o-fin-do-programa-e-o-reinicio).

```rust
mod handler;
pub mod recording;
mod resize;
mod scanner;
mod screen;
mod selection;
mod shell_state;
mod title;

pub use scanner::{Report, Scanner};
pub use screen::{Cell, CellFlags, Color, Columns, Cursor, CursorShape, Screen};
pub use selection::{SelectionKind, Side};
pub use shell_state::{Folder, Mark, ShellState};
```

- Os outros arquivos do cartafol `src/term/` son os submódulos de `term`.
- `recording` é público: o exemplo `replay`, as probas e, desde a 4.1, o *benchmark* usan as gravacións como `meiga_core::term::recording`. `scanner`, `screen` e, desde a subfase 1.8, `selection` son privados, coma, desde a 1.9, `handler` e `resize`, desde a 3.1, `title`, e, desde a 4.1, `shell_state`, pero os tipos de `screen` e de `selection` reexpórtanse con `pub use`: fóra úsanse como `meiga_core::term::Screen` ou `meiga_core::term::SelectionKind`, sen saber en que arquivo están. Desde a 4.1, tamén os de `shell_state`, `Folder`, `Mark` e `ShellState`, o que dá [`Terminal::shell`](#terminal); e os de `scanner`, `Scanner` e `Report`, para que o [*benchmark*](#cratesmeiga-corebenchesscannerrs) mida o escáner só, desde fóra do crate. Os de `title`, non: fóra da fachada, o título é un `Option<&str>`.
- Desde a 1.9, `use alacritty_terminal::grid::{self, Dimensions, Grid};` trae tamén o módulo `grid` (`self`): o `Scroll` de `alacritty_terminal` escríbese `grid::Scroll`, porque `Scroll` é o tipo de meiga ([O historial e a súa vista](#o-historial-e-a-súa-vista)). E `use handler::Hooked;` trae o `Handler` que recibe a saída do programa ([`term/handler.rs`](#cratesmeiga-coresrctermhandlerrs)). Desde a 3.1, `use title::Titles;` trae os títulos do programa, que garda o terminal ([`term/title.rs`](#cratesmeiga-coresrctermtitlers)). Ata a 4.1, `use scanner::Scanner;` traía o escáner, que agora trae o `pub use`.
- Desde a 2.7, `use std::io::Write;` trae o trait `Write`, co que [`restart`](#o-fin-do-programa-e-o-reinicio-restart-e-at_empty_row_start) escribe con `write!` nun `Vec<u8>` as secuencias das tabulacións.

### Constantes

```rust
pub const SCROLLBACK_LINES: usize = 10_000;

pub const MIN_SIZE: GridSize =
    GridSize::new(term::MIN_COLUMNS as u16, term::MIN_SCREEN_LINES as u16);

const DEFAULT_CURSOR: CursorStyle = CursorStyle {
    shape: ansi::CursorShape::Beam,
    blinking: true,
};
```

- **`SCROLLBACK_LINES`**: as liñas de historial que se gardan por riba da pantalla, só na pantalla principal: a alternativa non ten. Desde a subfase 1.9, a vista móstraas ([O historial e a súa vista](#o-historial-e-a-súa-vista)); ata a 1.8, gardábanse sen poder velas. `alacritty_terminal` non as reserva ao crear o terminal: a memoria do historial medra a medida que se enche, de 1 000 en 1 000 liñas. Cada cela ocupa 24 bytes, tamén as baleiras: un historial cheo de 100 columnas son 24 MB ([ADR 0016 › Consecuencias](../adr/0016-o-historial-o-tamano-e-o-zoom.md#consecuencias)). Desde a 1.9, úsaa tamén [`new_primary`](#a-pantalla-principal-á-parte), para a pantalla principal nova despois dun reinicio na alternativa.
- **`MIN_SIZE`**: a grella máis pequena que admite `alacritty_terminal`, as súas constantes `MIN_COLUMNS` e `MIN_SCREEN_LINES`: 2 columnas e 1 fila. Un carácter ancho necesita dúas columnas: nunha grella dunha soa columna, `alacritty_terminal` falla (*panic*) ao escribilo. `as u16` converte os dous valores, que son `usize`, porque nunha constante non se pode usar `try_from`; son 2 e 1, así que caben.
- **`DEFAULT_CURSOR`**, desde a subfase 1.6: o cursor cando o programa non escolle ningún, unha barra que pestanexa, coma no Windows Terminal ([ADR 0013](../adr/0013-da-saida-a-pantalla.md)). Vai na configuración de `alacritty_terminal` (`default_cursor_style`), que o usa mentres o programa non pide outro, e outra vez despois de `ESC [ 0 SP q`, dun reinicio (RIS) ou, desde a subfase 2.7, de [`restart`](#o-fin-do-programa-e-o-reinicio-restart-e-at_empty_row_start). Se o programa só pide un cursor fixo ou que pestanexe (`?12l` e `?12h`), a barra segue, e só cambia o pestanexo. É privada porque `CursorStyle` é un tipo de `alacritty_terminal`: fóra da fachada, o cursor é un [`Cursor`](#cratesmeiga-coresrctermscreenrs). Que pestanexe de verdade decídeo a vista do terminal, segundo Windows ([Tecnoloxías › O cursor de Windows](../tecnoloxias.md#38-o-cursor-de-windows-pestanexo-tempo-e-grosor)).
- **`RESTART`**, desde a subfase 2.7: as secuencias coas que [`restart`](#o-fin-do-programa-e-o-reinicio-restart-e-at_empty_row_start) deixa os modos, as cores e o cursor coma nun terminal novo, e pasa ao historial a pantalla principal. Ata a 3.1 deixaba tamén o título, cun OSC 0 baleiro; desde entón quítao `restart`, sen secuencias. O seu código, e o que fai cada secuencia, están alí.

### `Changes` e `Modes`

```rust
pub struct Changes {
    pub responses: bool,
    pub title: bool,
    pub bell: bool,
    pub clipboard: bool,
    pub shell: bool,
}
```

- **`Changes`**: o que pasou ao ler a saída, ademais dos cambios na pantalla: hai respostas para o programa (`take_responses`), cambiou o título (`title`), soou a campá, hai texto para o portapapeis (`take_clipboard`) ou, desde a subfase 4.1, cambiou o que di a shell (`shell`). Devólveno `feed` e `end_sync`, e quen chama só ten que facer algo máis cando un campo é `true`. Desde a subfase 3.1, `title` só é `true` se o título é outro: dar outra vez o mesmo non é un cambio, coma di o seu comentario ([ADR 0029](../adr/0029-o-titulo-da-tab-e-os-titulos-gardados.md), decisión 2). Ata entón, cada OSC 0 ou 2, e cada `CSI 23 t` cun título gardado, era un cambio, tamén co mesmo título. Un título distinto e outra vez o de antes, na mesma lectura, si son un cambio, aínda que o título remate igual ([`take_changed`](#o-cambio-take_changed-e-replace)). Desde a 4.1, `shell`, coma o título: a shell dixo outro cartafol, outra marca ou outro código de saída, que dá [`Terminal::shell`](#terminal); dicir o mesmo outra vez non é un cambio, coma di o seu comentario ([`apply`](#o-que-dixo-a-shell-shellstate)). Cambia cando o escáner le a saída, en `feed`, tamén dentro dunha actualización sincronizada: `end_sync` nunca o trae.
- **`Modes`**: os modos que activou o programa e que cambian o que meiga lle envía: como se codifican as teclas (win32-input-mode, as frechas e o teclado numérico en modo aplicación), os eventos de foco, o pegado entre marcas (*bracketed paste*), que eventos do rato quere e en que formato, se a roda envía frechas, se está na pantalla alternativa, se se ve o cursor, e se o salto de liña volta tamén á primeira columna. Os números de cada modo están en [Tecnoloxías › As secuencias de control](../tecnoloxias.md#35-as-secuencias-de-control). O win32-input-mode (`win32_input`) ségueo o propio terminal, porque `alacritty_terminal` non o coñece: desde a subfase 4.1, no [`Hooked`](#os-métodos-de-handler), e ata entón, no escáner; o resto dáo `alacritty_terminal`. Desde a subfase 1.8, os modos do rato deciden se os clics seleccionan texto ou van ao programa, e en que formato ([`mouse.rs`](#cratesmeiga-coresrcmousers)), e o *bracketed paste*, como se pega o texto ([`paste.rs`](#cratesmeiga-coresrcpasters)). Desde a 1.9, na pantalla alternativa (`alt_screen`), que non ten historial, as teclas do historial van ao programa ([Código › meiga › `view.rs`](meiga.md#cratesmeigasrcviewrs)).

### Os eventos: `Pending` e `Listener`

```rust
#[derive(Default)]
struct Pending {
    changes: Changes,
    responses: Vec<u8>,
    questions: Vec<(usize, Question)>,
    clipboard: Option<String>,
}

enum Question {
    Color(usize, Arc<dyn Fn(ansi::Rgb) -> String + Send + Sync>),
    TextAreaSize,
}

struct Listener(Arc<Mutex<Pending>>);
```

Mentres le a saída, `alacritty_terminal` avisa do que pasa chamando a `send_event` do seu `EventListener`, un trait que implementa `Listener`. `send_event` recibe `&self`, unha referencia que non deixa cambiar nada, así que os eventos van a unha cola, `Pending`, que o `Listener` comparte co `Terminal` nun `Arc<Mutex<…>>`:

- **`Arc`** é un punteiro con contador de referencias que se pode compartir entre fíos: o `Terminal` e o seu `Listener` apuntan á mesma cola, que se destrúe cando se destrúen os dous.
- **`Mutex`** permite cambiar o que garda a través dunha referencia compartida, bloqueándoo mentres tanto. Ninguén máis o bloquea, así que nunca hai que esperar. Fai falta porque o terminal ten que poder pasar dun fío a outro (`Send`): `RefCell`, a alternativa sen bloqueo, non o permite dentro dun `Arc`.

| Evento | Que fai meiga |
|---|---|
| `PtyWrite` | Unha resposta para o programa: os atributos do dispositivo (DA1 e DA2), o estado e a posición do cursor (DSR), o tamaño en celas… Engádese a `responses`. |
| `ColorRequest(index, answer)` | Desde a subfase 1.6: unha pregunta por unha cor (OSC 4, 10, 11 ou 12 con `?`). Anótase en `questions` (`Question::Color`), coa posición na que irá a súa resposta: o final de `responses` nese momento. |
| `TextAreaSizeRequest` | Desde a 1.6: unha pregunta polo tamaño da grella en píxeles (`CSI 14 t`). Anótase igual (`Question::TextAreaSize`). |
| `Title` e `ResetTitle` | Ata a subfase 3.1: `Title`, o título que lle daba o programa á xanela (OSC 0 e 2), ou un gardado que daba outra vez (`ESC [ 23 t`); e `ResetTitle`, un gardado que era ningún. O `Listener` gardaba o título en `Pending`, e un título baleiro era coma non ter ningún. Desde a 3.1, `alacritty_terminal` xa non os envía: o [`Hooked`](#os-métodos-de-handler) non lle pasa o título nin os gardados, que leva o terminal ([`term/title.rs`](#cratesmeiga-coresrctermtitlers), [ADR 0029](../adr/0029-o-titulo-da-tab-e-os-titulos-gardados.md), decisión 2). Se chegasen, ignoraríanse coma os demais. |
| `Bell` | A campá (BEL). |
| `ClipboardStore(Clipboard, …)` | Texto para o portapapeis (OSC 52). Windows non ten selección primaria: só se garda o do portapapeis. |
| Os demais | Ignóranse. Ler o portapapeis nunca se permite (`Osc52::OnlyCopy`), así que nunca se pide; e, coma di o comentario desde a 3.1, o título non vén de eiquí. `CursorBlinkingChange` tampouco fai falta: cada copia compara o cursor, tamén o seu pestanexo. O resto son para o bucle de eventos do Alacritty, que meiga non usa. |

**As preguntas que `alacritty_terminal` non sabe responder.** As cores dependen da paleta de meiga, e o tamaño en píxeles, do tamaño dunha cela, que coñece quen debuxa: `alacritty_terminal` envíaas como eventos, en vez de respondelas. O `Listener` só ten a cola, así que as anota en `questions`, e o terminal respóndeas cando remata de ler ([As preguntas](#as-preguntas-polas-cores-e-polo-tamaño)). Cada `Question` garda o que fai falta para respondela:

- **`Color(index, answer)`**: o número da cor nas cores de `alacritty_terminal` (de 0 a 255, as da paleta; 256, 257 e 258, o texto, o fondo e o cursor) e a función que escribe a resposta, que trae o evento. É un `Arc<dyn Fn…>` porque así a dá `alacritty_terminal`; e `Send` e `Sync` deixan que a cola, e o terminal con ela, siga podendo pasar dun fío a outro.
- **`TextAreaSize`** non garda nada: a resposta sae do tamaño da grella e do dunha cela. O evento trae tamén unha función que a escribiría, pero meiga non a usa (ver máis abaixo).

```rust
struct TermSize(GridSize);

impl Dimensions for TermSize { … }
```

`alacritty_terminal` pide o tamaño a través do trait `Dimensions`: as filas que se ven (`screen_lines`), as columnas e as filas en total. `TermSize` envolve un `GridSize` para implementalo: implementalo directamente en `GridSize`, que é público, mostraría `alacritty_terminal` fóra da fachada. O historial non conta: `total_lines` é igual a `screen_lines`. Ata a subfase 1.5 chamábase `Size`; na 1.6 cambiou de nome porque o módulo usa agora o `Size` de [`geometry.rs`](#cratesmeiga-coresrcgeometryrs), o tamaño dunha cela en píxeles, e os dous nomes chocarían. Desde a 1.9, úsano tamén os cambios de tamaño de [`term/resize.rs`](#cratesmeiga-coresrctermresizers).

### `Terminal`

```rust
pub struct Terminal {
    term: Term<Listener>,
    primary: Option<Grid<TermCell>>,
    region_below_top: bool,
    win32_input: bool,
    parser: Processor,
    scanner: Scanner,
    pending: Arc<Mutex<Pending>>,
    size: GridSize,
    titles: Titles,
    shell: ShellState,
    theme: Palette,
    cell: Size,
    damaged: Vec<LineDamageBounds>,
    shown_selection: Option<SelectionRange>,
}
```

| Campo | Que é |
|---|---|
| `term` | O terminal de `alacritty_terminal`: a grella, o historial, o cursor e os modos. |
| `primary` | Desde a subfase 1.9: a pantalla principal, coa súa grella e o seu historial, mentres está á parte porque o tamaño cambiou cando se vía a alternativa; `None` o resto do tempo. Recibe o tamaño do terminal cando o programa volta a ela ([`term/resize.rs`](#cratesmeiga-coresrctermresizers)). `TermCell` é o `Cell` de `alacritty_terminal`, co nome cambiado, porque `Cell` é o de meiga. |
| `region_below_top` | Desde a revisión do código da subfase 1.9: a rexión de desprazamento do programa empeza máis abaixo da primeira fila da pantalla. `alacritty_terminal` non di onde empeza a súa, e [`Hooked`](#hooked) séguea eiquí, para deixar a vista do historial onde está ([O historial e a súa vista](#o-historial-e-a-súa-vista)). |
| `win32_input` | Desde a subfase 4.1: o win32-input-mode está activo. `alacritty_terminal` non coñece ese modo, e [`Hooked`](#os-métodos-de-handler) ségueo eiquí, na orde da saída. Ata a 4.1, sabíao o escáner. |
| `parser` | O parser de `vte` (`Processor`), que le os bytes e chama aos métodos de `term`. Garda o estado entre lecturas: unha secuencia partida entre dous bloques, ou un carácter UTF-8 a medias. Tamén garda a saída dunha actualización sincronizada mentres non remata. |
| `scanner` | O escáner propio, que busca na saída, antes do parser, o que di a shell ([`scanner.rs`](#cratesmeiga-coresrctermscannerrs)). Ata a 4.1, seguía o win32-input-mode. |
| `pending` | A cola dos eventos, compartida co `Listener`. |
| `size` | O tamaño da grella. |
| `titles` | Desde a subfase 3.1: o título que deu o programa, os que gardou e se o título cambiou desde que o terminal colleu os seus cambios ([`Titles`](#titles)). Ata a 3.1 era `title`, o último título que deu o programa, que `take_changes` collía da cola dos eventos. |
| `shell` | Desde a subfase 4.1: o último que dixo a shell, o seu cartafol, a súa última marca e o código de saída da última orde, e se cambiou desde que o terminal colleu os seus cambios ([`ShellState`](#o-que-dixo-a-shell-shellstate)). |
| `theme` | Desde a subfase 1.6: as cores do tema, `Palette::MEIGA` ata que haxa temas. As que cambia o programa non van eiquí: gárdaas `alacritty_terminal` ([A paleta](#a-paleta-o-tema-e-os-cambios-do-programa)). |
| `cell` | Desde a 1.6: o tamaño dunha cela en píxeles, para responder a `CSI 14 t`; 0 × 0 mentres non se sabe. |
| `damaged` | As filas danadas, nun `Vec` que se reutiliza en cada copia para non reservar memoria. |
| `shown_selection` | Desde a subfase 1.8: a selección que mostra a pantalla, como a deixou a última copia, coas súas celas na grella (`SelectionRange`). `alacritty_terminal` non dana as celas da selección cando esta cambia: a copia compáraa coa nova ([`copy_damage`](#copy_damage-copiar-o-que-cambiou)). |

```rust
const _: () = {
    const fn send<T: Send>() {}
    send::<Terminal>();
};
```

**Unha comprobación ao compilar.** Desde a subfase 1.6, o fío lector da sesión alimenta o terminal e o fío da xanela copia del ([Código › meiga › `session.rs`](meiga.md#cratesmeigasrcsessionrs)): `Terminal` ten que poder pasar dun fío a outro, ser `Send`. `send::<Terminal>()` só compila se o é, e un `const _` avalíase ao compilar, sen deixar nada no programa. Se un cambio mete no terminal algo que non é `Send`, como un `Rc`, o erro sae eiquí, e non no crate que o usa, lonxe da causa.

```rust
pub fn new(size: GridSize) -> Self {
    let size = at_least_min_size(size);
    let pending = Arc::new(Mutex::new(Pending::default()));
    let config = Config {
        scrolling_history: SCROLLBACK_LINES,
        default_cursor_style: DEFAULT_CURSOR,
        osc52: Osc52::OnlyCopy,
        kitty_keyboard: false,
        semantic_escape_chars: selection::WORD_DELIMITERS.to_owned(),
        ..Config::default()
    };
    Self {
        term: Term::new(config, &TermSize(size), Listener(Arc::clone(&pending))),
        primary: None,
        region_below_top: false,
        win32_input: false,
        parser: Processor::new(),
        …
        titles: Titles::default(),
        shell: ShellState::default(),
        theme: Palette::MEIGA,
        cell: Size::new(0, 0),
        damaged: Vec::new(),
        shown_selection: None,
    }
}

pub fn feed(&mut self, bytes: &[u8]) -> Changes {
    let shell = &mut self.shell;
    self.scanner.scan(bytes, |report| shell.apply(report));
    let mut hooked = Hooked {
        term: &mut self.term,
        primary: &mut self.primary,
        region_below_top: &mut self.region_below_top,
        titles: &mut self.titles,
        win32_input: &mut self.win32_input,
    };
    self.parser.advance(&mut hooked, bytes);
    self.take_changes()
}
```

- **`new`** crea un terminal de polo menos `MIN_SIZE`, coa configuración de meiga:
  - 10 000 liñas de historial;
  - desde a subfase 1.6, o cursor por defecto, `DEFAULT_CURSOR`;
  - `Osc52::OnlyCopy`: os programas poden escribir no portapapeis, pero non lelo;
  - sen o protocolo de teclado de kitty: meiga usa o win32-input-mode, que é o que pide o ConPTY ([`input.rs`](#cratesmeiga-coresrcinputrs));
  - desde a subfase 1.8, os caracteres que rematan unha palabra nun dobre clic (`semantic_escape_chars`), os de meiga, [`WORD_DELIMITERS`](#word_delimiters-selectionkind-e-side): `alacritty_terminal` úsaos para estender as seleccións de palabras. `to_owned` copia o texto nun `String`, que é o que garda a configuración;
  - o resto, cos valores por defecto (`..Config::default()`).

  Ten as cores de meiga (`Palette::MEIGA`), ningunha selección e, desde a subfase 1.9, ningunha pantalla principal á parte (`primary: None`) e a rexión de desprazamento na pantalla enteira (`region_below_top: false`); desde a 3.1, ningún título nin ningún gardado (`Titles::default()`); desde a 4.1, o win32-input-mode apagado (`win32_input: false`) e nada do que di a shell (`ShellState::default()`); e aínda non sabe o tamaño da cela: 0 × 0 ata que llo dean con `set_cell_size`. `Processor::new()` reserva xa o búfer das actualizacións sincronizadas, de 2 MiB, que Windows conta como memoria privada aínda que non se use. Que facer con el decidirase na subfase 2.2, coa memoria por tab ([ADR 0012 › Consecuencias](../adr/0012-fachada-do-terminal.md#consecuencias)).
- **`feed`** le o que escribiu o programa: primeiro o escáner e despois o parser, cos mesmos bytes, sen copialos nin cambialos. Devolve os `Changes`.
  - Desde a subfase 4.1, o escáner dálle o que di a shell, `Report` a `Report`, a [`ShellState::apply`](#o-que-dixo-a-shell-shellstate), cunha *closure* que só toma prestado o campo `shell` (`let shell = &mut self.shell;`), mentres le o escáner, que é outro campo. Así, o que di a shell conta ao ler a saída, antes ca a pantalla, tamén dentro dunha actualización sincronizada, que `vte` garda sen ler ata que remata ([ADR 0032](../adr/0032-o-escaner-osc.md), decisión 5). Compróbao a proba `what_the_shell_says_counts_at_once_also_in_a_synchronized_update`.
  - Desde a subfase 1.9, o parser non lle dá a saída directamente ao `Term`, senón a un [`Hooked`](#hooked): pásalle as ordes e, cando o programa deixa a pantalla alternativa ou reinicia o terminal, devolve no momento a pantalla principal que deixou á parte un cambio de tamaño, antes da saída que vén despois. Tamén deixa a vista do historial onde está cando se despraza unha rexión que non empeza arriba ([O historial e a súa vista](#o-historial-e-a-súa-vista)). E, desde a 3.1, o título e os gardados non llos pasa ao `Term`: lévaos aos `titles` do terminal ([`term/title.rs`](#cratesmeiga-coresrctermtitlers)). Desde a 4.1, segue tamén o win32-input-mode, que `alacritty_terminal` ignora ([`term/handler.rs`](#cratesmeiga-coresrctermhandlerrs)). `Hooked` toma prestados cinco campos do terminal, `term`, `primary`, `region_below_top`, desde a 3.1, `titles` e, desde a 4.1, `win32_input`, mentres le o parser, que é outro: o compilador permite as seis referencias mutábeis porque son campos distintos.

| Método | Que fai |
|---|---|
| `size` | O tamaño da grella. |
| `set_theme(theme)` | Desde a subfase 1.6: cambia as cores do tema. As que cambiou o programa seguen coa súa. Polo de agora só o usan as probas: os temas chegan na subfase 8.3. |
| `set_cell_size(cell)` | Desde a 1.6: o tamaño dunha cela en píxeles, para responder a `CSI 14 t`. Mentres non o ten, non responde. `meiga.exe` dállo ao crear o terminal, co tamaño da cela da fonte. |
| `palette` | Desde a 1.6: as cores coas que se debuxan as celas ([A paleta](#a-paleta-o-tema-e-os-cambios-do-programa)). |
| `take_responses(into)` | Move as respostas ao final de `into`, para envialas ao programa. `Vec::append` deixa a cola baleira pero coa súa memoria: a seguinte resposta non reserva nada. |
| `take_clipboard` | O último texto que o programa copiou ao portapapeis; despois, ningún. Se copiou varios antes de recollelos, só queda o último, como no portapapeis. Desde a subfase 1.8, a xanela recólleo e pono no portapapeis de Windows, só se ten o foco e está en primeiro plano ([ADR 0015](../adr/0015-o-rato-a-seleccion-e-o-portapapeis.md), [Código › meiga › `app.rs`](meiga.md#cratesmeigasrcapprs)). |
| `title` | O título que deu o programa, se deu algún: nunca un texto baleiro. Desde a subfase 3.1 dállo [`Titles::current`](#o-título-current-e-set); ata entón, o campo `title`. |
| `shell` | Desde a subfase 4.1: o que dixo a shell, prestado: o seu último cartafol, a súa última marca e o código de saída da última orde ([`ShellState`](#o-que-dixo-a-shell-shellstate)). A xanela usarao desde a 4.5; ata entón, móstrao `replay --screen` ([`examples/replay.rs`](#cratesmeiga-coreexamplesreplayrs)). |
| `modes` | Os modos, traducidos de `TermMode` (uns *bitflags*) aos campos de `Modes`. `win32_input` é o campo do terminal, que segue o `Hooked` desde a subfase 4.1; ata entón, preguntábaselle ao escáner. |
| `resize(size)` | Cambia o tamaño da grella, con polo menos `MIN_SIZE`, e todo queda danado. Ata a subfase 1.8, `alacritty_terminal` repartía as liñas longas da pantalla principal (*reflow*) e colocaba a pantalla á súa maneira, e a selección só se ía se cambiaba o número de columnas. Desde a 1.9, a pantalla queda onde a pon o ConPTY, e a selección vaise sempre ([O tamaño](#o-tamaño-resize)). |
| `sync_deadline` | Cando hai que mostrar a actualización sincronizada (`?2026`) que empezou o programa, aínda que non a remate: `vte` gárdaa como moito 150 ms desde o último `?2026h`, ou ata encher o búfer. Mentres tanto, a saída non chega á pantalla. |
| `end_sync` | Mostra a saída da actualización sincronizada que agarda, e devolve os `Changes`. É para cando vence: o terminal non ten temporizador. Desde a subfase 1.6, remátaa a xanela: antes de cada fotograma consulta `sync_deadline`, e o temporizador dos fotogramas vence a esa hora, tamén coa xanela minimizada, que non debuxa ([Código › meiga › `session.rs`](meiga.md#cratesmeigasrcsessionrs)). Desde a 1.9, coma `feed`: a saída pasa polo `Hooked`, co mesmo préstamo dos campos do terminal, desde a 3.1 tamén os `titles` e, desde a 4.1, `win32_input`: dentro dunha actualización sincronizada, o win32-input-mode cambia cando esta se mostra. O escáner, non: xa leu eses bytes no `feed` que os trouxo, e `end_sync` nunca trae `Changes::shell`. Desde a 2.7, chámao tamén `restart`, antes de reiniciar o terminal. |

Desde a subfase 1.8, `Terminal` ten tamén os métodos da selección, noutro arquivo do módulo: `select`, `select_to`, `clear_selection`, `has_selection` e `selection_text` ([`term/selection.rs`](#cratesmeiga-coresrctermselectionrs)). Desde a 1.9, ten tamén os do historial: `scroll`, `scroll_to` e `history` ([O historial e a súa vista](#o-historial-e-a-súa-vista)). E desde a 2.7, `restart` e `at_empty_row_start`, para o fin do programa ([O fin do programa e o reinicio](#o-fin-do-programa-e-o-reinicio-restart-e-at_empty_row_start)).

### O tamaño: `resize`

Desde a subfase 1.9 ([ADR 0016](../adr/0016-o-historial-o-tamano-e-o-zoom.md)). A vista do terminal chámao co tamaño da grella que enche a xanela, como moito unha vez por intervalo dos fotogramas, e despois cambia o tamaño do ConPTY co *lock* do terminal aínda collido: ningunha saída do programa chega entre os dous ([Código › meiga › `view.rs`](meiga.md#cratesmeigasrcviewrs)).

```rust
pub fn resize(&mut self, size: GridSize) {
    let size = at_least_min_size(size);
    if size == self.size {
        return;
    }
    self.size = size;
    self.region_below_top = false;
    if self.term.mode().contains(TermMode::ALT_SCREEN) {
        if self.primary.is_none() {
            self.primary = Some(resize::set_aside_primary(&mut self.term));
        }
        // Neither ConPTY nor alacritty_terminal reflow the alternate
        // screen.
        self.term.resize(TermSize(size));
        self.term.selection = None;
    } else {
        resize::resize(&mut self.term, size);
    }
}
```

- **Co mesmo tamaño, nada**, tamén despois de `at_least_min_size`: nin garda á parte a pantalla principal nin quita a selección.
- **Na pantalla principal**, [`resize::resize`](#resize-a-pantalla-onde-a-pon-o-conpty) reparte o texto e coloca a pantalla onde a pon o ConPTY.
- **Na pantalla alternativa**, a primeira vez, [`set_aside_primary`](#a-pantalla-principal-á-parte) garda á parte a principal, sen repartila. Despois, `term.resize` cambia a alternativa, que nin o ConPTY nin `alacritty_terminal` reparten: córtase ou énchese pola dereita e por abaixo. `alacritty_terminal` reparte tamén a grella que ocupa o sitio da principal, que, baleira e sen historial, non custa nada. Os seguintes cambios xa atopan a principal á parte (`is_none`), e esta recibe o tamaño do terminal unha vez, cando o programa volta a ela, coma no ConPTY ([`term/handler.rs`](#cratesmeiga-coresrctermhandlerrs)).
- **A selección vaise sempre**, coma no Windows Terminal: `alacritty_terminal` quítaa se cambian as columnas, pero se só cambian as filas, móvea co texto. Na pantalla principal, quítaa `resize::resize`.
- **A rexión de desprazamento volta ser a pantalla enteira**: `alacritty_terminal` faino sempre que cambia o tamaño, nas dúas pantallas. Desde a revisión do código, `region_below_top`, que a segue, volta a `false` ([`Hooked`](#hooked)).

### O historial e a súa vista

Desde a subfase 1.9: o historial, as filas que saíron da pantalla principal por arriba, vese coa roda, coas teclas e coa barra de desprazamento ([ADR 0016](../adr/0016-o-historial-o-tamano-e-o-zoom.md), [Tecnoloxías › O historial](../tecnoloxias.md#49-o-historial-de-alacritty_terminal-a-vista-e-o-reflow)). A vista é o que se mostra: a pantalla, ou unhas filas máis arriba. `alacritty_terminal` gárdaa na súa grella, coma as filas por riba da pantalla onde empeza (`display_offset`), e a copia do que cambia copia as filas que se ven ([`copy_row`](#os-axudantes)). Fóra da fachada, móvese e lese con tipos de meiga.

```rust
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Scroll {
    Rows(i32),
    PageUp,
    PageDown,
    Top,
    Bottom,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct History {
    pub rows: usize,
    pub offset: usize,
}
```

- **`Scroll`**: canto mover a vista:

  | Variante | Ata onde |
  |---|---|
  | `Rows(filas)` | Unhas filas cara arriba, cara ás máis vellas, ou cara abaixo, cun número negativo. |
  | `PageUp` e `PageDown` | Unha pantalla cara arriba ou cara abaixo. |
  | `Top` | Á primeira fila do historial. |
  | `Bottom` | Á pantalla, onde escribe o programa. |

- **`History`**: o historial e a parte del que mostra a vista: `rows`, as filas de historial por riba da pantalla, e `offset`, cantas filas por riba da pantalla empeza a vista, 0 cando mostra a pantalla. Na pantalla alternativa, sen historial, os dous son 0. Con el debúxase o polgar da barra de desprazamento ([Código › meiga › `scrollbar.rs`](meiga.md#cratesmeigasrcscrollbarrs), [Tecnoloxías › A barra de desprazamento](../tecnoloxias.md#52-a-barra-de-desprazamento-e-o-zoom)).
- `alacritty_terminal` ten o seu `Scroll`, coas mesmas cinco variantes (`Delta` en vez de `Rows`), pero non sae da fachada ([ADR 0012](../adr/0012-fachada-do-terminal.md)): neste módulo é `grid::Scroll`.

```rust
pub fn scroll(&mut self, scroll: Scroll) -> bool {
    let before = self.term.grid().display_offset();
    self.term.scroll_display(match scroll {
        Scroll::Rows(rows) => grid::Scroll::Delta(rows),
        Scroll::PageUp => grid::Scroll::PageUp,
        Scroll::PageDown => grid::Scroll::PageDown,
        Scroll::Top => grid::Scroll::Top,
        Scroll::Bottom => grid::Scroll::Bottom,
    });
    self.term.grid().display_offset() != before
}

pub fn scroll_to(&mut self, offset: usize) -> bool {
    let before = self.term.grid().display_offset();
    let offset = offset.min(self.term.grid().history_size());
    // Both fit in an i32: the history has at most SCROLLBACK_LINES rows.
    self.scroll(Scroll::Rows(offset as i32 - before as i32))
}

pub fn history(&self) -> History {
    let grid = self.term.grid();
    History {
        rows: grid.history_size(),
        offset: grid.display_offset(),
    }
}
```

| Método | Que fai |
|---|---|
| `scroll(scroll)` | Move a vista, sen pasar da primeira fila do historial nin da pantalla, e di se se moveu: entón hai que debuxar. Úsano as teclas e a roda, a barra cando pasa páxinas e a selección que se despraza soa; e as teclas que van ao programa, e pegar, levan con el a vista á pantalla (`Bottom`). |
| `scroll_to(offset)` | Leva a vista a `offset` filas por riba da pantalla, como moito ao principio do historial, e di se se moveu. Úsao a barra mentres se arrastra o polgar. |
| `history` | O historial e onde está a vista. Con el, cada fotograma pon ao día o polgar da barra. |

- `scroll_display` de `alacritty_terminal` fai o traballo: non pasa dos extremos e, se move a vista, dana a pantalla enteira. A seguinte copia compara entón todas as filas que se ven, e só marca as celas que cambian ([`copy_damage`](#copy_damage-copiar-o-que-cambiou)).
- `scroll_to` move a vista a diferenza entre `offset` e onde está. As dúas contas caben nun `i32`: o historial ten como moito `SCROLLBACK_LINES` filas, e `offset` xa non pasa del (`min`).
- Na pantalla alternativa, que non ten historial, a vista nunca se move.

- **A saída non move a vista** ([ADR 0016](../adr/0016-o-historial-o-tamano-e-o-zoom.md)): mentres mostra o historial, as filas que entran nel súbena tamén, e segue mostrando as mesmas filas. Faino `alacritty_terminal` ao desprazar a grella. Compróbao a proba `the_output_does_not_move_the_view_of_the_history`.
- **Tampouco cunha rexión de desprazamento.** Cunha rexión que non empeza na primeira fila (`ESC [ superior ; inferior r`), as filas que saen dela non entran no historial, pero `alacritty_terminal` sube igual a vista: con cada fila que se despraza, a vista mostraría a de enriba, e podería pasar do principio do historial, onde non hai filas que mostrar. Desde a revisión do código, [`Hooked`](#manter-a-vista-view_to_keep-e-keep_view) déixaa onde estaba. Compróbano as probas `the_rows_that_leave_a_region_below_the_top_do_not_move_the_view` e `a_new_size_a_reset_or_deccolm_make_the_region_the_whole_screen_again`.

### A paleta: o tema e os cambios do programa

```rust
pub fn palette(&self) -> Palette {
    let mut palette = self.theme.clone();
    let colors = self.term.colors();
    for (index, color) in palette.colors.iter_mut().enumerate() {
        if let Some(changed) = colors[index] {
            *color = rgb(changed);
        }
    }
    for (name, color) in [
        (NamedColor::Foreground, &mut palette.foreground),
        (NamedColor::Background, &mut palette.background),
        (NamedColor::Cursor, &mut palette.cursor),
    ] {
        if let Some(changed) = colors[name] {
            *color = rgb(changed);
        }
    }
    palette
}
```

Desde a subfase 1.6. Un programa pode cambiar calquera cor da paleta (OSC 4), a do texto (OSC 10), a do fondo (OSC 11) e a do cursor (OSC 12), e voltar á orixinal (OSC 104, 110, 111 e 112) ([Tecnoloxías › A paleta](../tecnoloxias.md#37-a-paleta-e-os-estilos-das-celas)). `alacritty_terminal` garda eses cambios en `term.colors()`, unha táboa cun `Option` para cada unha das 256 cores e para as que teñen nome (o texto, o fondo, o cursor…): `None` mentres o programa non a cambia, e outra vez `None` cando volta á orixinal.

- **`palette`** dá a paleta coa que se debuxa: a do tema, coas cores que cambiou o programa por riba. A táboa de `alacritty_terminal` admite como índice un número ou un `NamedColor`, e os nomes van despois das 256 cores. Desde a subfase 1.8, a cor da selección é sempre a do tema: non ten nome nesa táboa, e ningunha secuencia a cambia.
- **Por que o tema vai aparte.** Se os cambios do programa se escribisen no tema, voltar á cor orixinal non sabería cal era, e cambiar de tema borraría o que cambiou o programa. Así, `set_theme` só cambia o tema, e as cores do programa seguen por riba.
- `for (name, color) in [(…, &mut palette.foreground), …]` percorre un *array* de pares, cada un cunha referencia mutábel a un campo distinto da mesma paleta: o compilador permíteo porque non se solapan.
- Devolve unha paleta nova en cada chamada, sen reservar memoria: son 780 bytes na pila. `copy_damage` chámaa en cada copia, e `take_changes`, se hai preguntas.
- `rgb` converte cada cor de `alacritty_terminal` nun `Rgb` de meiga ([Os axudantes](#os-axudantes)).

### As preguntas polas cores e polo tamaño

```rust
fn take_changes(&mut self) -> Changes {
    let mut pending = lock(&self.pending);
    if !pending.questions.is_empty() {
        let palette = self.palette();
        let questions = mem::take(&mut pending.questions);
        let responses = mem::take(&mut pending.responses);
        let mut answered = Vec::with_capacity(responses.len() + 32 * questions.len());
        let mut from = 0;
        for (at, question) in &questions {
            let at = (*at).clamp(from, responses.len());
            answered.extend_from_slice(&responses[from..at]);
            from = at;
            if let Some(answer) = self.answer(question, &palette) {
                answered.extend_from_slice(answer.as_bytes());
                pending.changes.responses = true;
            }
        }
        answered.extend_from_slice(&responses[from..]);
        pending.responses = answered;
    }
    …
}
```

Desde a subfase 1.6, o terminal responde tamén ás preguntas que lle deixa `alacritty_terminal` ([Os eventos](#os-eventos-pending-e-listener)): as das cores e a do tamaño en píxeles. `feed` e `end_sync` rematan os dous en `take_changes`, que as responde antes de devolver os `Changes`, coas cores que hai cando remata de ler. As que chegan dentro dunha actualización sincronizada respóndense cando esta remata: no `feed` que trae o seu fin, ou en `end_sync`.

**A orde.** As respostas teñen que ir na mesma orde ca as preguntas. Moitos programas preguntan pola cor do fondo e despois polos atributos do terminal (DA1, `ESC [ c`), á que responden todos: se esta resposta chega primeiro, entenden que o terminal non sabe responder a outra ([Tecnoloxías › As secuencias de control](../tecnoloxias.md#35-as-secuencias-de-control), [ADR 0013](../adr/0013-da-saida-a-pantalla.md)). Por iso cada pregunta garda onde vai a súa resposta: os bytes que había en `responses` cando chegou. `take_changes` constrúe as respostas outra vez, en orde: as de `alacritty_terminal` ata a posición da primeira pregunta, a súa resposta, as seguintes ata a posición da segunda, e así ata o final. Compróbao a proba `questions_about_colours_are_answered_with_the_palette`, cunha cor entre dúas preguntas DA1.

- `clamp(from, responses.len())`: as posicións só medran e nunca pasan do final, porque mentres se le a saída as respostas só se engaden ao final; `clamp` garante que, aínda así, os rangos nunca fallan.
- `pending.changes.responses` só pasa a `true` se houbo algunha resposta: unha pregunta que o terminal non sabe responder non avisa de nada.
- Reserva sitio para 32 bytes máis por pregunta, o bastante para unha resposta habitual. Só pasa cando houbo preguntas, poucas veces, así que non importa reservar un `Vec` novo.

```rust
fn answer(&self, question: &Question, palette: &Palette) -> Option<String> {
    match question {
        Question::Color(index, answer) => {
            let color = match *index {
                index @ 0..256 => palette.colors[index],
                index if index == NamedColor::Foreground as usize => palette.foreground,
                index if index == NamedColor::Background as usize => palette.background,
                index if index == NamedColor::Cursor as usize => palette.cursor,
                _ => return None,
            };
            Some(answer(ansi::Rgb {
                r: color.r,
                g: color.g,
                b: color.b,
            }))
        }
        Question::TextAreaSize => (!self.cell.is_empty()).then(|| {
            let height = u64::from(self.size.rows) * u64::from(self.cell.height);
            let width = u64::from(self.size.columns) * u64::from(self.cell.width);
            format!("\x1b[4;{height};{width}t")
        }),
    }
}
```

- **Unha cor**: a da paleta, se o número vai de 0 a 255, ou a do texto, a do fondo ou a do cursor. Con outro número, que `alacritty_terminal` non envía, non hai resposta. A resposta escríbea a función que veu coa pregunta, coma a pregunta: co mesmo número e co mesmo final (BEL ou `ESC \`), e cada compoñente coas súas dúas cifras hexadecimais repetidas, catro en total: `#16181D` é `rgb:1616/1818/1d1d`.
- **O tamaño en píxeles**, `ESC [ 4 ; alto ; ancho t`: as filas polo alto dunha cela, e as columnas polo seu ancho. É o da grella, sen as marxes que sobran na xanela. `then` dá `Some` co resultado da *closure* só se a condición se cumpre: sen o tamaño da cela, `None`, porque unha resposta de 0 píxeles sería falsa.
- **Por que non a resposta de `alacritty_terminal`.** A súa función calcula o mesmo, pero multiplicando números de 16 bits (os de `WindowSize`), e o produto pode non caber neles: un *panic* nas compilacións de depuración, e unha resposta errada en *release*. meiga multiplica en `u64`, onde un `u16` por un `u32` sempre cabe.

### O fin do programa e o reinicio: `restart` e `at_empty_row_start`

Desde a subfase 2.7 ([ADR 0024](../adr/0024-o-fin-do-programa-e-o-reinicio.md), [Arquitectura › O fin do programa e o reinicio](../arquitectura.md#o-fin-do-programa-e-o-reinicio)). Cando o programa dunha tab remata sen un fin limpo, a sesión escribe no terminal como rematou, nunha fila súa, e debaixo o que se pode facer entón; e Intro ou *Restart tab* lanzan outra vez o programa, na mesma tab e no mesmo terminal ([Código › meiga › `session.rs`](meiga.md#cratesmeigasrcsessionrs) e [› `tabs.rs`](meiga.md#cratesmeigasrctabsrs)). Para iso, o terminal ten dous métodos: `restart`, que o deixa coma o espera un programa novo, e `at_empty_row_start`, que di se a liña do fin pode ir na fila do cursor.

```rust
const RESTART: &[u8] = concat!(
    // The primary screen, the default colours of the text, and all the
    // screen for the program: the scrolling region, the origin mode, the
    // wrapping, the insert mode and the line feed.
    "\x1b[?1049l\x1b[0m\x1b[r\x1b[?6l\x1b[?7h\x1b[4l\x1b[20l",
    // The keys and the mouse: the cursor keys and the keypad, the reports
    // of the mouse, the focus events, the bracketed paste, the
    // win32-input-mode and the wheel on the alternate screen.
    "\x1b[?1l\x1b>\x1b[?1000;1002;1003;1005;1006l\x1b[?1004l\x1b[?2004l\x1b[?9001l\x1b[?1007h",
    // The characters: the four sets, ASCII, and the first one in use.
    "\x1b(B\x1b)B\x1b*B\x1b+B\x0f",
    // The cursor, with its default style and shown; no hyperlink.
    "\x1b[ q\x1b[?25h\x1b]8;;\x07",
    // The colours of the theme: the palette, the text, the background and
    // the cursor.
    "\x1b]104\x07\x1b]110\x07\x1b]111\x07\x1b]112\x07",
    // What the primary screen shows goes to the history.
    "\x1b[2J",
)
.as_bytes();
```

**`RESTART`**: as secuencias que deixan os modos, as cores e o cursor coma nun terminal novo, e que pasan ao historial a pantalla principal, sen o reinicio (`ESC c`) que tamén a borra. `concat!` xunta os textos ao compilar nun só `&'static str`, e `as_bytes` dá os seus bytes, coma os que escribe un programa: `restart` dállos a `feed`. Cada grupo leva un comentario. O que fai cada secuencia en `alacritty_terminal`:

| Secuencias | Que deixan |
|---|---|
| `?1049l` | A pantalla principal, se se vía a alternativa, coma cando a deixa o programa, e o [`Hooked`](#os-métodos-de-handler) devolve tamén a principal que gardase á parte un cambio de tamaño: o resto de `RESTART` cambia xa a principal. A alternativa, coa liña do fin se o programa rematou nela, xa non se ve, e baléirase cando un programa pasa outra vez a ela. |
| `ESC [ 0 m` | As cores e os estilos por defecto, para o texto que vén e para as celas que borra `ESC [ 2 J`, ao final. |
| `ESC [ r`, `?6l`, `?7h`, `ESC [ 4 l` e `ESC [ 20 l` | A pantalla enteira para o programa: a rexión de desprazamento, que [`follow_region`](#seguir-a-rexión-follow_region-e-follow_column_mode) segue; sen o modo de orixe (DECOM); co paso á fila seguinte despois da última columna (DECAWM); sen inserir (IRM); e sen que o salto de liña volte á primeira columna (*LNM*). |
| `?1l` e `ESC >` | As frechas e o teclado numérico en modo normal, e non no de aplicación. |
| `?1000;1002;1003;1005;1006l` | Sen informes do rato, nin os seus formatos UTF-8 e SGR: os clics seleccionan texto. |
| `?1004l`, `?2004l` e `?9001l` | Sen os eventos de foco, o *bracketed paste* nin o win32-input-mode. Este último non o coñece `alacritty_terminal`: apágao o [`Hooked`](#os-métodos-de-handler), coma os outros modos, desde a subfase 4.1, e ata entón, o escáner, que le os mesmos bytes; o ConPTY novo actívao outra vez ao empezar. |
| `?1007h` | A roda envía frechas na pantalla alternativa, coma nun terminal novo. |
| `ESC ( B`, `ESC ) B`, `ESC * B`, `ESC + B` e SI | Os catro xogos de caracteres, de G0 a G3, en ASCII, e G0 en uso (SI, `0x0F`): un programa que deixou en uso o das liñas (`ESC ( 0`, ou `ESC ) 0` e SO) non deixa ao seguinte cun `q` que se ve coma `─`. |
| `ESC [ SP q` | O cursor por defecto, [`DEFAULT_CURSOR`](#constantes): DECSCUSR sen número é o 0, coma `ESC [ 0 SP q`. |
| `?25h` | O cursor visíbel: a liña do fin ocultárao. |
| `OSC 8 ; ;` | Sen ligazón (*hyperlink*) para o texto que vén: `ESC [ 0 m` non a quita. |
| `OSC 104`, `OSC 110`, `OSC 111` e `OSC 112` | As cores do tema: as 256 da paleta, a do texto, a do fondo e a do cursor ([A paleta](#a-paleta-o-tema-e-os-cambios-do-programa)). |
| `ESC [ 2 J` | ED 2: o que mostra a pantalla principal vai ao historial, e a pantalla queda en branco (ver máis abaixo). |

- **A orde importa.** `?1049l` vai primeiro: en `alacritty_terminal`, os xogos de caracteres e as cores do texto van co cursor de cada pantalla, e teñen que cambiar no da principal. E `ESC [ 0 m` vai antes ca `ESC [ 2 J`, porque `alacritty_terminal` borra as celas co fondo do cursor: así quedan co fondo por defecto, e non coa última cor do programa.
- As OSC rematan en BEL (`\x07`). `ESC [ 2 J` non move o cursor: lévao á esquina `restart`, despois das tabulacións.
- **Ata a subfase 3.1**, `RESTART` levaba tamén `OSC 0 ;`, despois das cores: un título baleiro, que era ningún ([Os eventos](#os-eventos-pending-e-listener)), e a tab mostraba o nome do seu perfil ata que o programa novo puxese un. Pero os títulos gardados seguían en `alacritty_terminal`, e un `CSI 23 t` do programa novo traía un título do vello ([ADR 0029](../adr/0029-o-titulo-da-tab-e-os-titulos-gardados.md), decisión 4). Desde a 3.1, o título e os gardados non pasan polo `Term`, e `restart` quítaos el mesmo, sen ningunha secuencia (ver máis abaixo).

```rust
pub fn restart(&mut self) {
    // What the last program left in a synchronized update is shown
    // first. It has ended: its answers are dropped.
    if self.sync_deadline().is_some() {
        self.end_sync();
    }
    self.feed(RESTART);
    // A tab stop every 8 columns, from the first one: alacritty_terminal
    // ignores DECST8C (`CSI ? 5 W`).
    let mut tabs = Vec::with_capacity(8 * (usize::from(self.size.columns) / 8 + 1) + 8);
    tabs.extend_from_slice(b"\x1b[3g");
    for column in (1..=self.size.columns).step_by(8) {
        let _ = write!(tabs, "\x1b[{column}G\x1bH");
    }
    // Then the cursor at the top-left corner, which is also the one that
    // `ESC 8` restores until the program saves another one.
    tabs.extend_from_slice(b"\x1b[H\x1b7");
    self.feed(&tabs);
    self.scroll(Scroll::Bottom);
    self.titles = Titles::default();
    self.shell = ShellState::default();
    let mut pending = lock(&self.pending);
    pending.responses.clear();
    pending.clipboard = None;
}
```

**`restart`** deixa o terminal coma o espera un programa novo, para a sesión que empeza outra vez na súa tab ([ADR 0024](../adr/0024-o-fin-do-programa-e-o-reinicio.md), decisión 7): o que mostraba a pantalla principal pasa ao historial, por riba dunha pantalla limpa co cursor na esquina de arriba á esquerda, e os modos, as cores, o cursor e as tabulacións son os dun terminal novo, sen título e, desde a subfase 3.1, sen títulos gardados ([ADR 0029](../adr/0029-o-titulo-da-tab-e-os-titulos-gardados.md), decisión 4), nin, desde a 4.1, nada do que dixo a shell ([ADR 0032](../adr/0032-o-escaner-osc.md), decisión 5). O que mostraba a alternativa, se o programa rematou nela, pérdese, coma cando un programa sae dela. O historial queda, e a vista volta á pantalla. Así empeza tamén o ConPTY novo, cunha pantalla limpa e o seu cursor arriba á esquerda: o terminal queda coma o seu, coma nunha tab nova, e o que escribe o programa cae nas mesmas filas nos dous. Paso a paso:

1. **A actualización sincronizada.** Se o último programa deixou unha aberta (`?2026h`), `vte` aínda garda a súa saída. `end_sync` móstraa primeiro, para que vaia ao historial co resto, e para que `RESTART` chegue ao `Term` e non ao búfer da actualización. Despois, xa non queda ningunha aberta. A sesión xa a remata ao escribir a liña do fin ([Código › meiga › `session.rs`](meiga.md#cratesmeigasrcsessionrs)), pero `restart` non depende diso.
2. **`RESTART`**, por `feed`, coma a saída dun programa: pasa polo escáner e polo [`Hooked`](#hooked). O seu primeiro ESC remata tamén un OSC que o último programa deixase a medias, no escáner e en `vte`: o que dicía esquécese nos pasos 6 e 7.
3. **As tabulacións**, unha cada 8 columnas desde a primeira, coma nun terminal novo, onde `alacritty_terminal` as pon así ([`TabStops::new`](https://github.com/alacritty/alacritty/blob/94e7c8874e526b1e67b349d9ba30ddf81669119e/alacritty_terminal/src/term/mod.rs#L2323-L2327), en `term/mod.rs`). `ESC [ 3 g` (TBC) quítaas todas e, en cada columna 1, 9, 17…, `ESC [ columna G` (CHA) leva alí o cursor e `ESC H` (HTS) pon unha. Noutros terminais abondaría con `CSI ? 5 W` (DECST8C), pero `alacritty_terminal` ignórao: `vte` chama con el a [`set_tabs(8)`](https://github.com/alacritty/vte/blob/3b3da71c34cc1256c7e20981cf03f8eb95e08ffc/src/ansi.rs#L1580), que `Term` non implementa, e queda [o método do trait, que non fai nada](https://github.com/alacritty/vte/blob/3b3da71c34cc1256c7e20981cf03f8eb95e08ffc/src/ansi.rs#L616), os dous en `src/ansi.rs` de `vte` 0.15.0.
4. **O cursor**, despois das tabulacións, que o moven, e no mesmo `feed`: `ESC [ H` (CUP) lévao á esquina de arriba á esquerda, e `ESC 7` (DECSC) gárdao alí, coas cores por defecto e os xogos de caracteres en ASCII, que `alacritty_terminal` garda con el. Un `ESC 8` (DECRC) do programa novo lévao alí, coma nun terminal novo, ata que garde outro.
5. **A vista volta á pantalla** (`Scroll::Bottom`). Se mostraba o historial, `ESC [ 2 J` subiuna coas filas que entraron nel, e seguiría mostrándoas.
6. **Sen títulos**, desde a subfase 3.1: uns [`Titles`](#titles) novos, sen título nin gardados, coma nun terminal novo. Van despois dos dous `feed`, para que tampouco quede un título que o último programa deixase nunha actualización sincronizada aberta, que mostra o paso 1. Tampouco queda ningún cambio do título por avisar: os `feed` xa colleron os seus, e os `Titles` novos non teñen ningún. Ata a 3.1, quitaba o título o `OSC 0 ;` de `RESTART`, e os gardados quedaban.
7. **Sen o que dixo a shell**, desde a subfase 4.1: un [`ShellState`](#o-que-dixo-a-shell-shellstate) novo, sen cartafol, marca nin código, coma nunha tab nova ([ADR 0032](../adr/0032-o-escaner-osc.md), decisión 5). Vai tamén despois dos dous `feed`, para que non quede o que dicía un OSC que o último programa deixou a medias, e que rematou o primeiro ESC de `RESTART`. Nin queda ningún cambio por avisar: o que diga a shell nova é un cambio, aínda que a vella dixese o mesmo. Compróbao a proba `a_restart_forgets_what_the_shell_said_and_a_reset_does_not`.
8. **O que quedaba pendente vaise**: as respostas ás preguntas que o último programa deixou nunha actualización sincronizada, e o texto que copiou ao portapapeis. O programa que as fixo xa rematou, e o novo non as espera. `restart` tampouco devolve os `Changes`: a sesión esquece tamén os avisos que a xanela non colleu ([Código › meiga › `session.rs`](meiga.md#cratesmeigasrcsessionrs)).

- **`write!` nun `Vec`.** O trait `std::io::Write` deixa escribir texto con formato nun `Vec<u8>`, sen un `String` no medio. Nun `Vec` nunca falla, e `let _ =` ignora o `Result`. Cada tabulación leva 5 bytes e as cifras da súa columna: a capacidade, 8 bytes por tabulación e 8 máis, abonda mentres as columnas teñen ata tres cifras, e nunha grella máis ancha o `Vec` medra.
- **Cun tamaño novo.** As tabulacións son as das columnas que ten o terminal ao reiniciarse. A tab dálle despois o tamaño da xanela ([Código › meiga › `tabs.rs`](meiga.md#cratesmeigasrctabsrs)), e `alacritty_terminal` corta as que sobran ou engade as que faltan, tamén cada 8 columnas desde a primeira ([`TabStops::resize`](https://github.com/alacritty/alacritty/blob/94e7c8874e526b1e67b349d9ba30ddf81669119e/alacritty_terminal/src/term/mod.rs#L2337-L2346)).

**Por que o historial, e non `ESC c`.** O reinicio enteiro, RIS (`ESC c`), deixaría o terminal novo dunha vez, pero en `alacritty_terminal` borra tamén o historial: [`reset_state`](https://github.com/alacritty/alacritty/blob/94e7c8874e526b1e67b349d9ba30ddf81669119e/alacritty_terminal/src/term/mod.rs#L1833-L1858), en `term/mod.rs`, chama a [`Grid::reset`](https://github.com/alacritty/alacritty/blob/94e7c8874e526b1e67b349d9ba30ddf81669119e/alacritty_terminal/src/grid/mod.rs#L335-L352), en `grid/mod.rs`, que baleira o historial (`clear_history`) e as filas da pantalla. Perderíase o que escribiu o programa que rematou, e por que rematou, que é o que mostraba a tab (ADR 0024, alternativas). En cambio, `ESC [ 2 J` (ED 2) na pantalla principal [chama a `clear_viewport`](https://github.com/alacritty/alacritty/blob/94e7c8874e526b1e67b349d9ba30ddf81669119e/alacritty_terminal/src/term/mod.rs#L1788-L1803), que [despraza a pantalla cara arriba](https://github.com/alacritty/alacritty/blob/94e7c8874e526b1e67b349d9ba30ddf81669119e/alacritty_terminal/src/grid/mod.rs#L309-L333) ata a última fila con algo escrito: esas filas entran no historial, e as baleiras de debaixo non. Na alternativa, ED 2 só a borraría, pero `?1049l` xa a deixou. Os modos voltan coma no reinicio do Windows Terminal que non borra o texto, pero o texto vai ao historial, e non queda na pantalla: o programa novo empeza nunha pantalla limpa, coma o ConPTY, sen o texto dun programa que non coñece, e o de antes queda a unha roda de distancia (ADR 0024, decisión 7). Ademais, `reset_state` non volta ás cores que cambiou o programa: farían falta igual as OSC de `RESTART`. Nin quitaría o título: desde a subfase 3.1, `ESC c` mantén o título e os gardados ([ADR 0029](../adr/0029-o-titulo-da-tab-e-os-titulos-gardados.md), decisións 1 e 3). Ata a 3.1, en cambio, baleirábaos en `alacritty_terminal` sen avisar, e facía falta igual o OSC 0. Desde a 4.1, `ESC c` tampouco esquece o que dixo a shell ([ADR 0032](../adr/0032-o-escaner-osc.md), decisión 5).

- **Se a pantalla principal está baleira e sen historial**, ED 2 mete igual unha fila en branco no historial: `clear_viewport` busca cara atrás a última cela con algo e, se chega ao principio da grella sen atopala, conta a primeira fila. Con historial, para nel, e non mete ningunha. O ConPTY de Windows fai iso en cada tab ao arrancar: borra a pantalla, que aínda está baleira, e o historial de cada tab empeza cunha fila en branco (atopado no mantemento M2, porque `check-exit.ps1` fallaba no seu paso 6 con ese ConPTY: [Ferramentas › `check-exit.ps1`](ferramentas.md#toolscheck-exitps1)). meiga deixará de metela na 8.6; o ConPTY que vai con meiga non borra a pantalla ao arrancar.

```rust
pub fn at_empty_row_start(&self) -> bool {
    let grid = self.term.grid();
    let point = grid.cursor.point;
    if point.column.0 != 0 || !grid[point.line].is_clear() {
        return false;
    }
    let above = Line(point.line.0 - 1);
    above < grid.topmost_line()
        || !grid[above][grid.last_column()]
            .flags
            .contains(Flags::WRAPLINE)
}
```

**`at_empty_row_start`** di se unha liña escrita no cursor queda nunha fila súa: o cursor está ao principio dunha fila baleira, e a de enriba non continúa nela. Pregúntao a sesión cando o programa remata sen un fin limpo: se non, a liña do fin empeza cun salto de liña ([ADR 0024](../adr/0024-o-fin-do-programa-e-o-reinicio.md), decisión 3). Mira a grella, e non a vista: vale tamén cando a vista mostra o historial e non a fila do cursor.

- **Baleira** é sen texto nin cores, coma unha fila borrada coas cores por defecto. [`Row::is_clear`](https://github.com/alacritty/alacritty/blob/94e7c8874e526b1e67b349d9ba30ddf81669119e/alacritty_terminal/src/grid/row.rs#L155-L160), en `grid/row.rs` de `alacritty_terminal`, mira que o sexan todas as súas celas ([`is_empty`](https://github.com/alacritty/alacritty/blob/94e7c8874e526b1e67b349d9ba30ddf81669119e/alacritty_terminal/src/term/cell.rs#L226-L239), en `term/cell.rs`): un espazo ou unha tabulación, coas cores do texto e do fondo por defecto, sen inverso, subliñados nin riscado, sen as marcas dos caracteres anchos nin caracteres de ancho cero, e sen `WRAPLINE`, a marca da última cela dunha fila que continúa na seguinte. Así, despois dun CR sen LF, coma o que deixa unha barra de progreso, o cursor está ao principio dunha fila con texto, e a liña do fin vai na seguinte, sen tapalo. Ata a revisión do código da 2.7, só miraba a columna, e a liña escribíase enriba dese texto.
- **A de enriba** pode estar no historial, coas liñas negativas: `grid[above]` indexa a grella polas súas liñas, sen mirar a vista. Se ten `WRAPLINE` na súa última columna, a fila do cursor é a continuación dunha liña longa: a liña do fin continuaría esa liña, e ao cambiar o ancho xuntaríanse. Se non hai fila enriba, porque a do cursor é a primeira da grella ([`topmost_line`](https://github.com/alacritty/alacritty/blob/94e7c8874e526b1e67b349d9ba30ddf81669119e/alacritty_terminal/src/grid/mod.rs#L504-L506), a primeira do historial ou, sen historial, a da pantalla), non hai nada que continuar, e a grella non ten esa fila: indexala fallaría.
- **`Line(point.line.0 - 1)`**, e non `point.line - 1`: `Line` resta tanto un `i32` coma un `usize`, e cun número sen tipo Rust non sabe cal escoller.
- **Despois do último carácter dunha fila chea**, o cursor queda na última columna, esperando para pasar á seguinte: non está ao principio, e o salto de liña leva a liña do fin á fila seguinte, sen deixar unha en branco.
- **Por que non `cursor`.** [`cursor`](#cursor-e-take_changes) dá o cursor coma se debuxa: se a vista mostra o historial e a fila do cursor queda fóra, un cursor oculto na columna 0. Ata a 2.6, a sesión miraba esa columna, e coa vista no historial a liña do fin ía na fila do texto do programa, detrás del. Compróbano a proba `the_cursor_is_at_the_start_of_an_empty_row_also_with_the_view_up` e, en `session.rs`, `the_end_is_on_a_row_of_its_own_also_with_the_view_in_the_history`.

### `copy_damage`: copiar o que cambiou

```rust
pub fn copy_damage(&mut self, screen: &mut Screen) {
    let resized = screen.size() != self.size;
    if resized {
        screen.resize(self.size);
    }
    self.damaged.clear();
    let full = match self.term.damage() {
        TermDamage::Full => true,
        TermDamage::Partial(lines) => {
            self.damaged.extend(lines);
            resized
        }
    };
    let selection = self.selection_range();
    let grid = self.term.grid();
    if full {
        for row in 0..self.size.rows {
            copy_row(grid, screen, row, selection.as_ref());
        }
    } else {
        for damage in &self.damaged {
            match u16::try_from(damage.line) {
                Ok(row) if row < self.size.rows => {
                    copy_row(grid, screen, row, selection.as_ref());
                }
                _ => {}
            }
        }
        if selection != self.shown_selection {
            let display_offset = grid.display_offset();
            for range in [&self.shown_selection, &selection].into_iter().flatten() {
                for row in selection::shown_rows(range, display_offset, self.size.rows) {
                    copy_row(grid, screen, row, selection.as_ref());
                }
            }
        }
    }
    self.shown_selection = selection;
    let cursor = self.cursor();
    if cursor != screen.cursor() {
        screen.mark_cursor_changed();
        screen.set_cursor(cursor);
        screen.mark_cursor_changed();
    }
    screen.set_palette(&self.palette());
    self.term.reset_damage();
}
```

Copia na `Screen` o que cambiou desde a copia anterior ([ADR 0012](../adr/0012-fachada-do-terminal.md)), desde a subfase 1.6 a paleta ([ADR 0013](../adr/0013-da-saida-a-pantalla.md)), e desde a 1.8 a selección ([ADR 0015](../adr/0015-o-rato-a-seleccion-e-o-portapapeis.md)):

1. Se a `Screen` ten outro tamaño, cámbiao: todas as súas celas quedan en branco e cambiadas, e cópianse todas as filas.
2. `term.damage()` dá as filas danadas desde a última copia: `Full`, todas, ou `Partial`, a lista das filas danadas. `alacritty_terminal` dá `Full` cando cambia moito: ao borrar a pantalla, ao desprazala (cada salto de liña na última fila), ao cambiar á pantalla alternativa ou de tamaño, ao mover a vista do historial (desde a subfase 1.9), nun reinicio…
3. Cada fila danada cópiase **enteira** con `copy_row`, que compara cela a cela e só marca as que cambiaron. **Por que enteira:** `alacritty_terminal` dana as celas que escribe, pero non sempre outras da mesma fila que cambian con elas. Ao escribir sobre a metade dereita dun carácter ancho, por exemplo, borra tamén a esquerda, e esa non queda danada. Atopouno a proba aleatoria do terminal. Copiar e comparar unha fila custa pouco, e marcar só as celas distintas evita debuxar o que non cambiou, tamén co dano `Full`: despois de borrar a pantalla, as filas que xa estaban en branco non se marcan.
4. **A selección**, desde a subfase 1.8. `selection_range` dá as súas celas na grella, e `copy_row` marca as que están nela con `CellFlags::SELECTED` ([Os axudantes](#os-axudantes)). Pero `alacritty_terminal` tampouco dana as celas da selección cando esta cambia: se non é a que mostra a pantalla (`shown_selection`), cópianse outra vez as filas das dúas, a vella e a nova ([`shown_rows`](#cratesmeiga-coresrctermselectionrs)), e `copy_row` só marca as celas que entran ou saen dela. Así, arrastrar o rato só debuxa as celas onde cambia a selección. Cando a selección se move co texto, porque a pantalla se desprazou, `alacritty_terminal` xa dá `Full`. Ao final, `shown_selection` garda a nova, tamén despois dunha copia enteira.
5. **O cursor.** Se cambiou (a posición, a forma ou o pestanexo), márcanse as súas celas onde estaba, antes de cambialo, e onde está, despois, con [`mark_cursor_changed`](#cratesmeiga-coresrctermscreenrs): dúas celas, por se está sobre un carácter ancho, e só unha na última columna, e só se se debuxa. A vista do terminal debuxa o cursor por riba delas. Oculto (`Hidden`), non marca nada: un *spinner* que oculta o cursor só redebuxa os seus caracteres, e unha copia sen saída nova non marca nada.
6. **A paleta.** `set_palette` compara a paleta do terminal coa da `Screen`: se cambiou unha das 256 cores, a do texto, a do fondo ou a da selección, cambian todas as celas; se só cambiou a do cursor, só as do cursor. **Por que aparte:** cando o programa cambia unha cor, `alacritty_terminal` dana a pantalla enteira e as filas cópianse, pero as celas gardan o número da súa cor, que é o mesmo: comparadas, ningunha cambiou. E cando só cambia a do cursor, non dana nada. Compróbao a proba `a_change_of_the_palette_changes_every_cell`.
7. `reset_damage` esquece o dano, para a seguinte copia.

- `Ok(row) if row < self.size.rows => …` é un brazo de `match` cunha garda (`if`): só se copian as filas que existen.
- `[&self.shown_selection, &selection].into_iter().flatten()` percorre as dúas seleccións, a vella e a nova, e salta as que son `None`: unha referencia a un `Option` percórrese coma unha lista de cero ou un elementos. Unha fila que está nas dúas cópiase dúas veces, pero a segunda xa non cambia nada.
- `selection.as_ref()` dá un `Option<&SelectionRange>`, o que recibe `copy_row`: unha referencia, sen copiar a selección. Ao final, a selección pasa a ser `shown_selection`. A comparación `!=` mira as celas, non os puntos onde se fixo a selección: arrastrar unha selección de palabras dentro da mesma palabra, por exemplo, non copia nada.

### `cursor` e `take_changes`

- **`cursor`** le o cursor da grella (`grid.cursor.point`) e do estado do terminal. Desde a subfase 1.6 é público: ata a 2.6, cando o programa remataba, a sesión miraba con el se o cursor estaba ao principio dunha fila, para escribir nunha fila propia a liña co código de saída ([Código › meiga › `session.rs`](meiga.md#cratesmeigasrcsessionrs)). Desde a 2.7, mira [`at_empty_row_start`](#o-fin-do-programa-e-o-reinicio-restart-e-at_empty_row_start), que vale tamén coa vista no historial, e fóra do crate xa ninguén usa `cursor`.
  - a forma é `Hidden` se o programa o ocultou (`?25l`); se non, a que pediu (`cursor_style()`), que tamén di se pestanexa, ou `DEFAULT_CURSOR` se non pediu ningunha;
  - sobre a metade dereita dun carácter ancho, pasa á columna anterior, para debuxalo sobre o carácter enteiro, pero só se esa cela é a súa metade esquerda: coma en `copy_row`, unha metade dereita soa é unha cela en branco, e o cursor debúxase nela;
  - `point_to_viewport` converte a posición na grella, que inclúe o historial, na fila que se ve: se queda fóra, porque desde a subfase 1.9 a vista pode mostrar o historial, o cursor non se debuxa.

  **Por que non `renderable_content()`**, que dá o cursor como o debuxa Alacritty: sobre unha metade dereita, pásao á columna anterior sen comprobar que exista. Borrar caracteres (`ESC [ P`) pode deixar unha metade dereita soa na primeira columna, co cursor enriba, e entón a resta baixa de cero: nas compilacións de depuración, que comproban os desbordamentos tamén nas dependencias, é un *panic*; en *release*, a columna dá a volta e o cursor desaparece. Atopouno a revisión do código, e compróbao a proba `the_cursor_on_a_lone_right_half_stays_in_its_cell`.
- **`take_changes`**, desde a subfase 1.6, responde primeiro as preguntas pendentes ([As preguntas](#as-preguntas-polas-cores-e-polo-tamaño)). Despois colle os `Changes` da cola e déixaa baleira: `mem::take` devolve o valor e deixa no seu sitio o valor por defecto. Desde a 3.1, o aviso do título non vén da cola, senón dos `titles`, e, desde a 4.1, o do que di a shell, do `shell`:

  ```rust
  let changes = mem::take(&mut pending.changes);
  Changes {
      title: self.titles.take_changed(),
      shell: self.shell.take_changed(),
      ..changes
  }
  ```

  [`take_changed`](#o-cambio-take_changed-e-replace) di se o título cambiou desde a última vez, e esquéceo, e o [de `ShellState`](#o-que-dixo-a-shell-shellstate), se cambiou o que di a shell; `..changes` dá aos outros campos os valores de `changes`. Ata a 3.1, o aviso do título viña na cola, coma os outros, e o título, en `Pending`: `take_changes` movíao ao campo `title` do terminal só se cambiara.

### Os axudantes

- **`rgb`**, desde a subfase 1.6, converte unha cor de `alacritty_terminal` nun `Rgb` de meiga. Úsana `palette` e `color`.
- **`lock`** bloquea a cola. Se un fío fallase (*panic*) mentres a tiña bloqueada, o `Mutex` quedaría «envelenado» (*poisoned*), e `lock()` daría un erro desde entón. Ningún cambio da cola pode quedar a medias, así que `unwrap_or_else(PoisonError::into_inner)` cóllea igual.
- **`at_least_min_size`**: o tamaño, con polo menos 2 columnas e 1 fila.
- **`copy_row`** copia unha fila que se ve e marca as celas que cambiaron. Desde a subfase 1.8 recibe tamén a selección (`selection`), e as celas que están nela levan `CellFlags::SELECTED`:

  ```rust
  let selected = selection.map_or(Columns::default(), |range| {
      selected_cells(range, point.line, sources, columns)
  });
  …
  for ((cell, source), column) in cells.iter_mut().zip(sources).zip(0..columns) {
      let char = if source.flags.contains(Flags::WIDE_CHAR) {
          CellChar::WideLeft(source.c)
      } else if source.flags.contains(Flags::WIDE_CHAR_SPACER) {
          wide.map_or(CellChar::Blank, CellChar::WideRight)
      } else if source.flags.contains(Flags::LEADING_WIDE_CHAR_SPACER)
          || matches!(source.c, ' ' | '\t')
      {
          CellChar::Blank
      } else {
          CellChar::Narrow(source.c)
      };
      wide = source.flags.contains(Flags::WIDE_CHAR).then_some(source.c);
      let mut flags = flags(source.flags);
      if selected.contains(column) {
          flags = flags | CellFlags::SELECTED;
      }
      …
  }
  ```

  - `viewport_to_point` converte a fila que se ve na liña da grella, contando o historial (`display_offset`), e `&grid[point.line][..]` son as súas celas. Desde a subfase 1.9, a vista pode mostrar o historial, e as súas filas cópianse coma as da pantalla.
  - Percorre á vez as celas da `Screen` e as de `alacritty_terminal` (`zip`), e converte cada unha:

    | Cela de `alacritty_terminal` | `CellChar` |
    |---|---|
    | Co *flag* `WIDE_CHAR` | `WideLeft(c)`. |
    | Co *flag* `WIDE_CHAR_SPACER` | `WideRight(c)`, co carácter da cela anterior se esta é a súa metade esquerda; se non, `Blank`. |
    | Co *flag* `LEADING_WIDE_CHAR_SPACER`, un espazo ou un tabulador | `Blank`. |
    | Calquera outra | `Narrow(c)`. |

  - **As metades soltas.** Inserir ou borrar caracteres (`ESC [ @` e `ESC [ P`) move as celas dunha fila e pode separar as dúas metades dun carácter ancho: unha metade dereita pode quedar sen a esquerda, mesmo na columna 0. `wide` lembra o carácter da cela anterior, se é unha metade esquerda; sen ela, a metade dereita é unha cela en branco.
  - **`LEADING_WIDE_CHAR_SPACER`**: cando un carácter ancho non cabe ao final dunha fila, vai á seguinte, e a última cela queda baleira con este *flag*.
  - **O tabulador**: `alacritty_terminal` garda `\t` na cela onde empezou, para poder copiar o texto con tabuladores; vese como unha cela en branco.
  - Cada cela nova compárase coa que había, e só se cambia se é distinta. `changed` acumula as columnas, da primeira á última que cambiaron, e `mark_changed` engádeas aos cambios da fila.
  - **A selección.** `selected` son as columnas desta fila que están na selección, ningunha se non hai selección (`map_or`). `SELECTED` engádese aos estilos da cela, así que unha cela que entra ou sae da selección é distinta da que había, e márcase coma calquera outro cambio. `let mut flags = flags(source.flags);` chama á función `flags` e garda o resultado nunha variábel co mesmo nome, que oculta a función ata o final da volta do bucle.
- **`selected_cells`**, desde a subfase 1.8, dá esas columnas: as que colle a selección na liña da grella `line` ([`selected_columns`](#cratesmeiga-coresrctermselectionrs)), e un carácter ancho enteiro se a selección colle só unha das súas metades:

  ```rust
  fn selected_cells(
      range: &SelectionRange,
      line: Line,
      sources: &[TermCell],
      columns: u16,
  ) -> Columns {
      let mut selected = selection::selected_columns(range, line, columns);
      if selected.is_empty() {
          return selected;
      }
      let flags_at = |column: u16| {
          sources
              .get(usize::from(column))
              .map_or(Flags::empty(), |cell| cell.flags)
      };
      if selected.start > 0 && flags_at(selected.start).contains(Flags::WIDE_CHAR_SPACER) {
          selected.start -= 1;
      }
      if selected.end < columns && flags_at(selected.end - 1).contains(Flags::WIDE_CHAR) {
          selected.end += 1;
      }
      selected
  }
  ```

  - Se a selección empeza na metade dereita dun carácter ancho (`WIDE_CHAR_SPACER`), empeza unha columna antes; e se remata na metade esquerda (`WIDE_CHAR`), unha columna despois. O texto da selección xa leva o carácter enteiro: así a pantalla mostra o mesmo que se copia ([ADR 0015](../adr/0015-o-rato-a-seleccion-e-o-portapapeis.md)).
  - `flags_at` le os *flags* dunha cela, ou ningún se a columna non existe: `get` nunca falla.
- **`color`**: as 16 cores con nome (`Named`) son as 16 primeiras da paleta, de `Indexed(0)` a `Indexed(15)`. Os outros nomes son as cores por defecto do texto e do fondo, e outras que só usa o renderizador do Alacritty (a do cursor, as atenuadas…), que ningunha secuencia lle dá a unha cela: `Default`. `Indexed` pasa igual, e `Spec` (*truecolor*), con `rgb`. `Ok(index @ 0..16)` é un patrón: o valor ten que estar entre 0 e 15, e `@` gárdao en `index`.
- **`flags`** traduce os estilos cunha táboa (`MAP`) de 11 pares, só os que debuxa meiga: os *flags* internos de `alacritty_terminal`, como os das metades dos caracteres anchos, non pasan. `fold` xúntaos con `|`. `SELECTED` non está na táboa: non é un estilo de `alacritty_terminal`, e pono `copy_row`.

As probas están en [`tests.rs`](#cratesmeiga-coresrctermtestsrs).

## `crates/meiga-core/src/term/resize.rs`

Desde a subfase 1.9: o tamaño novo da pantalla principal, coa pantalla onde a colocan o anfitrión do ConPTY e o Windows Terminal, para que as filas do programa e as do terminal sexan as mesmas ([ADR 0016](../adr/0016-o-historial-o-tamano-e-o-zoom.md)). Chámano [`Terminal::resize`](#o-tamaño-resize) e, cando o programa volta á pantalla principal, [`Hooked`](#cratesmeiga-coresrctermhandlerrs).

**O problema.** Cando cambia o tamaño, o ConPTY volta repartir as liñas longas do seu búfer (*reflow*), que non ten historial: o texto queda arriba, e medra cara abaixo. Non lle escribe nada ao terminal: supón que colocou o texto coma el, e a seguinte vez que un programa lle pide os datos do búfer, pregúntalle onde ten o cursor (`ESC [ 6 n`) e colle esa posición ([Tecnoloxías › Cambiar o tamaño do ConPTY](../tecnoloxias.md#48-cambiar-o-tamaño-do-conpty-o-seu-reflow-e-a-pregunta-polo-cursor)). `alacritty_terminal` reparte a pantalla e o historial xuntos, e mantén abaixo o cursor: cando a pantalla medra, trae liñas do historial, e cando as liñas se parten, empurra as de arriba ao historial, aínda que haxa filas baleiras debaixo ([Tecnoloxías › O historial](../tecnoloxias.md#49-o-historial-de-alacritty_terminal-a-vista-e-o-reflow)). Entón, o que escribe despois o programa cae noutras filas, e queda lixo na pantalla.

**A solución.** Despois do *reflow* de `alacritty_terminal`, a pantalla móvese onde a pon o Windows Terminal (`Terminal::UserResize`): a súa primeira fila é aquela onde foi parar a primeira de antes, salvo que o texto e o cursor xa non caiban debaixo dela; entón, a súa última fila é a máis baixa con texto, ou a do cursor se está máis abaixo. O cursor sempre queda na pantalla, e as liñas que pasaron ao historial quedan nel. Para saber onde foron parar a primeira fila e o cursor, `resize` marca as súas celas antes de repartir: non se pode fiar do cursor de `alacritty_terminal`, que, cando moitas filas debaixo del se parten, sae da pantalla por arriba e queda na primeira fila, que xa non é a súa.

**Na pantalla alternativa**, o ConPTY non reparte a principal: dálle o último tamaño unha vez, cando o programa volta a ela. meiga garda á parte a pantalla principal ata entón, e así tampouco reparte o seu historial mentres corre un programa a pantalla completa.

**Como se comproba.** Ademais das probas de [`tests.rs`](#cratesmeiga-coresrctermtestsrs), `check-resize.ps1` compara na CI, despois de cada cambio de tamaño, a pantalla de meiga co búfer da consola do ConPTY, que le un script de PowerShell dentro de meiga con `ReadConsoleOutputCharacterW`: a 70 × 20, 45 × 14, 90 × 18 e 60 × 24 celas; na pantalla alternativa, a 50 × 16; e, despois dun cambio de tamaño coa caixa de mensaxe aberta, a 66 × 15 ([Código › ferramentas](ferramentas.md#toolscheck-resizeps1)).

### As marcas: `MARK` e `CURSOR`

```rust
const MARK: Flags = Flags::from_bits_retain(0x8000);

const CURSOR: char = '\u{FFFF}';
```

- **`MARK`**: un *flag* das celas que `alacritty_terminal` 0.26.0 non usa, o último dos 16 bits de `Flags` (`0x8000`): os seus, os estilos e as marcas dos caracteres anchos e das liñas partidas, chegan ata `0x4000`. Marca as dúas celas que segue o cambio de tamaño: a primeira da pantalla e a do cursor. `from_bits_retain` crea o valor con ese bit aínda que non sexa un *flag* coñecido, e pódese usar nunha constante. Nunca chega á pantalla: `resize` quita as marcas antes de rematar, e `copy_row` só copia os estilos da súa táboa ([Os axudantes](#os-axudantes)).
- **`CURSOR`**: o carácter da cela do cursor mentres está marcada, U+FFFF, un dos que Unicode reserva para usos internos (*noncharacters*), que os programas non escriben. Fai que a cela estea ocupada. Para `alacritty_terminal`, unha cela cun espazo, sen cores nin estilos, está baleira, tamén con `MARK`, e o *reflow* non conserva as celas baleiras do final das filas que parte: a do cursor podería perderse. Con `CURSOR`, o *reflow* consérvaa coma texto, e pasa á fila seguinte cos brancos de antes dela, coma fai o ConPTY cos brancos antes do cursor. Ao final, a cela recupera o seu carácter.

### `resize`: a pantalla onde a pon o ConPTY

```rust
pub(super) fn resize(term: &mut Term<Listener>, size: GridSize) {
    let at_bottom = term.grid().display_offset() == 0;
    let rows = i32::from(size.rows);
    let old_rows = term.grid().screen_lines() as u16;
    let marks = Marks::set(term.grid_mut());
    // The width first, with the same rows: alacritty_terminal removes no row
    // when it reflows.
    if usize::from(size.columns) != term.grid().columns() {
        term.resize(TermSize(GridSize::new(size.columns, old_rows)));
    }
    if let Some(cursor) = marks.find(term.grid()).cursor {
        term.grid_mut().cursor.point = cursor;
    }
    if size.rows < old_rows {
        // alacritty_terminal removes the rows at the bottom: before that, the
        // screen starts where it has to, so that they are the ones that
        // ConPTY removes too.
        let top = window_top(term.grid(), rows, marks.find(term.grid()).top);
        move_screen(term.grid_mut(), top);
        term.resize(TermSize(size));
    } else {
        term.resize(TermSize(size));
        let top = window_top(term.grid(), rows, marks.find(term.grid()).top);
        move_screen(term.grid_mut(), top);
        // Back to the size, without the rows at the bottom.
        let grid = term.grid_mut();
        let columns = grid.columns();
        grid.resize::<Color>(false, usize::from(size.rows), columns);
    }
    let found = marks.find(term.grid());
    marks.remove(term.grid_mut(), found);
    term.selection = None;
    let grid = term.grid_mut();
    grid.scroll_display(if at_bottom {
        Scroll::Bottom
    } else {
        // Where it was, as long as the history still reaches it.
        Scroll::Delta(0)
    });
}
```

**`resize`** cambia o tamaño da pantalla principal de `term` a `size`, que non é o que ten: nin [`Terminal::resize`](#o-tamaño-resize) nin [`bring_back_primary`](#a-pantalla-principal-á-parte) o chaman co mesmo tamaño. Paso a paso:

1. **As marcas.** Lembra se a vista mostraba a pantalla (`at_bottom`), e marca a primeira cela da pantalla e a do cursor ([`Marks::set`](#seguir-as-celas-marks-e-found)).
2. **Primeiro o ancho, coas mesmas filas.** Se cambian as columnas, `alacritty_terminal` reparte a pantalla e o historial coas filas de antes: ao repartir, non quita ningunha fila, e as que non caben na pantalla quedan no historial. Se cambiase tamén a altura na mesma chamada, `alacritty_terminal` quitaría ou traería filas primeiro, á súa maneira, co cursor abaixo, antes de saber onde foi parar o texto.
3. **O cursor, desde as marcas.** O cursor pasa á cela marcada con `CURSOR`, que pode estar no historial. `alacritty_terminal` sobe o cursor unha fila por cada fila nova debaixo del e, se moitas filas debaixo del se parten e o empurran por riba da pantalla, déixao na primeira fila, que xa non é a súa. Compróbao a proba `the_cursor_keeps_its_row_when_many_rows_below_it_wrap`.
4. **As filas.** [`window_top`](#onde-empeza-a-pantalla-window_top-e-last_text_row) di onde ten que empezar a pantalla, e [`move_screen`](#mover-a-pantalla-move_screen) móvea alí:
   - **con menos filas**, a pantalla móvese primeiro, e despois `alacritty_terminal` quita as filas que sobran abaixo: así son as mesmas que quita o ConPTY;
   - **con máis filas, ou coas mesmas**, primeiro cambia o tamaño: `alacritty_terminal` medra trazendo filas do historial, co cursor abaixo. Despois, a pantalla móvese: as filas que trouxo voltan ao historial, e entran filas baleiras abaixo. Se ten que empezar máis arriba, no historial, `move_screen` faina máis alta, e `Grid::resize` sen *reflow* (`false`) devólvea ao seu tamaño, sen as filas de abaixo. Coas mesmas filas, o primeiro `term.resize` non fai nada: o ancho xa cambiou.
5. **O final.** As marcas vanse, e a cela do cursor recupera o seu carácter ([`Marks::remove`](#seguir-as-celas-marks-e-found)). A selección vaise, coma no Windows Terminal: `alacritty_terminal` só a quita se cambian as columnas. E a vista queda abaixo se estaba abaixo, ou onde estaba, mentres o historial chegue a ela: `Delta(0)` non a move ningunha fila, pero axústaa ao historial, que pode ser máis curto. Compróbao a proba `a_resize_keeps_the_view_inside_the_history`.

- `old_rows` cabe nun `u16`, porque as filas da grella veñen dun `GridSize`; e `rows` é un `i32`, coma as liñas de `alacritty_terminal` (`Line`), negativas no historial.
- `Scroll`, neste arquivo, é o de `alacritty_terminal`, e o último `scroll_display` é o da grella, non o do `Term`: non dana nada, pero o cambio de tamaño xa danou a pantalla enteira.
- `::<Color>`, en `Grid::resize` e en `Grid::scroll_up`, di o tipo co que a grella compara as celas cando borra filas (`ResetDiscriminant`): nas celas de `alacritty_terminal`, a cor do fondo.

### A pantalla principal á parte

```rust
pub(super) fn set_aside_primary(term: &mut Term<Listener>) -> Grid<Cell> {
    let lines = term.screen_lines();
    let columns = term.columns();
    // alacritty_terminal only gives the primary screen through swap_alt,
    // which clears the grid that becomes the alternate screen: the alternate
    // screen goes out before, and comes back after.
    let alternate = mem::replace(term.grid_mut(), Grid::new(lines, columns, 0));
    term.swap_alt();
    let primary = mem::replace(term.grid_mut(), Grid::new(lines, columns, 0));
    term.swap_alt();
    *term.grid_mut() = alternate;
    primary
}
```

**`set_aside_primary`** garda á parte a pantalla principal, mentres se ve a alternativa, e devólvea. No seu sitio deixa unha grella baleira sen historial (`Grid::new(lines, columns, 0)`), que segue o tamaño do terminal: ata que o programa volta á pantalla principal, un cambio de tamaño só cambia a alternativa e esa grella, que, baleira, se reparte sen custo.

- **Por que dous `swap_alt`.** `alacritty_terminal` garda a pantalla que non se ve nun campo privado, e só a dá `swap_alt`, que cambia dunha pantalla á outra. Pero, cando cambia á alternativa, borra a grella que pasa a ser a alternativa. Así que primeiro sae a alternativa, cunha grella baleira no seu sitio; o primeiro `swap_alt` mostra a principal, que sae tamén, con outra grella baleira no seu sitio; o segundo volta á pantalla alternativa, e borra a primeira grella baleira; e ao final a alternativa volta ao seu sitio. O terminal segue na pantalla alternativa, e cada `swap_alt` quita a selección e dana a pantalla enteira.
- `mem::replace` pon un valor no sitio doutro e devolve o que había: as grellas saen do `Term` sen copialas.
- Unha copia de meiga que non garda á parte a pantalla principal falla `check-resize.ps1`: `The primary screen, after the alternate one changed size: … line 1 is '26 …' in meiga and '25 …' in the console.`

```rust
pub(super) fn bring_back_primary(term: &mut Term<Listener>, primary: Grid<Cell>) -> bool {
    let size = GridSize::new(term.columns() as u16, term.screen_lines() as u16);
    let fits = primary.columns() == term.columns() && primary.screen_lines() == term.screen_lines();
    *term.grid_mut() = primary;
    if !fits {
        resize(term, size);
    }
    !fits
}
```

**`bring_back_primary`** devolve ao `Term` a pantalla principal que estaba á parte, cando o programa acaba de deixar a alternativa ([`term/handler.rs`](#cratesmeiga-coresrctermhandlerrs)), no sitio da grella baleira, que xa ten o tamaño do terminal. Coma fai entón o ConPTY, recibe ese tamaño, unha vez: se non é o que tiña, [`resize`](#resize-a-pantalla-onde-a-pon-o-conpty) reparte o texto e coloca a pantalla, coma se o tamaño cambiase nese momento. Os tamaños do medio non contan. Compróbao a proba `on_the_alternate_screen_the_primary_one_gets_the_last_size_when_the_program_leaves`.

- O tamaño do terminal lese antes de devolver a principal: o `Term` mide a grella que ten. Os `as u16` non cortan nada: as columnas e as filas viñeron dun `GridSize`.
- Desde a revisión do código, di se o tamaño cambiou: entón `alacritty_terminal` puxo a rexión de desprazamento na pantalla enteira, e [`Hooked`](#os-métodos-de-handler) fai o mesmo coa súa copia.

```rust
pub(super) fn new_primary(term: &mut Term<Listener>) {
    let lines = term.screen_lines();
    let columns = term.columns();
    *term.grid_mut() = Grid::new(lines, columns, SCROLLBACK_LINES);
}
```

**`new_primary`** pon unha pantalla principal nova despois dun reinicio (`ESC c`) na pantalla alternativa, cando a principal estaba á parte. O reinicio de `alacritty_terminal` volta á pantalla principal e baléiraa, co seu historial, pero a que baleira é a grella sen historial que ocupaba o seu sitio. A principal á parte vaise ([`Hooked`](#cratesmeiga-coresrctermhandlerrs)), e no sitio da grella sen historial queda unha nova, baleira e con historial (`SCROLLBACK_LINES`), coma despois dun reinicio na pantalla principal. `Grid::new` non reserva o historial: medra a medida que se enche. Compróbao a proba `a_reset_on_the_alternate_screen_leaves_an_empty_primary_screen_with_history`.

### Seguir as celas: `Marks` e `Found`

```rust
struct Marks {
    cursor_char: char,
    pending_wrap: bool,
    cursor_first: bool,
}

struct Found {
    top: Option<i32>,
    cursor: Option<Point>,
}
```

| Campo | Que é |
|---|---|
| `Marks::cursor_char` | O carácter que tiña a cela do cursor, que volta a ela ao final. |
| `Marks::pending_wrap` | O cursor esperaba despois da última columna para pasar á fila seguinte (`input_needs_wrap`): o seguinte carácter vai á fila de abaixo. |
| `Marks::cursor_first` | O cursor estaba na primeira cela da pantalla: as dúas marcas son unha. |
| `Found::top` | A fila da cela que era a primeira da pantalla, na pantalla ou no historial, se a atopou. |
| `Found::cursor` | A cela do cursor, se a atopou. |

Os tres métodos de `Marks` (`impl Marks`):

```rust
fn set(grid: &mut Grid<Cell>) -> Self {
    let first = Point::new(Line(0), Column(0));
    let cursor = grid.cursor.point;
    let pending_wrap = grid.cursor.input_needs_wrap;
    // Writing through the index of the row makes the cell occupied.
    grid[first].flags.insert(MARK);
    let cell = &mut grid[cursor];
    cell.flags.insert(MARK);
    Self {
        cursor_char: mem::replace(&mut cell.c, CURSOR),
        pending_wrap,
        cursor_first: cursor == first,
    }
}
```

**`set`** marca a primeira cela da pantalla con `MARK`, e a do cursor con `MARK` e `CURSOR`, e garda o que fai falta para desfacelo. `mem::replace` pon `CURSOR` na cela e devolve o carácter que tiña.

- **As celas ocupadas.** Cada fila de `alacritty_terminal` lembra ata que columna ten celas escritas (`occ`), e cando a borra só mira esas. Escribir a través do índice da fila (`grid[…]`) conta a cela coma escrita: se a fila se borra, a marca vaise con ela.

```rust
fn find(&self, grid: &Grid<Cell>) -> Found {
    let mut found = Found {
        top: None,
        cursor: None,
    };
    let top = -(grid.history_size() as i32);
    let rows = grid.screen_lines() as i32;
    for line in (top..rows).rev() {
        for (column, cell) in grid[Line(line)].into_iter().enumerate() {
            if !cell.flags.contains(MARK) {
                continue;
            }
            let is_cursor = cell.c == CURSOR;
            if is_cursor {
                found.cursor = Some(Point::new(Line(line), Column(column)));
            }
            if !is_cursor || self.cursor_first {
                found.top = Some(line);
            }
        }
        if found.top.is_some() && found.cursor.is_some() {
            break;
        }
    }
    found
}
```

**`find`** busca as celas marcadas de abaixo arriba, desde a última fila da pantalla ata a primeira do historial, e para cando atopou as dúas: case sempre están preto da pantalla. A do cursor é a que ten `CURSOR`; a outra, a que era a primeira, ou a do cursor se eran a mesma (`cursor_first`). `resize` chámaa tres veces, porque cada paso move as filas: despois do *reflow*, polo cursor; antes de mover a pantalla, pola primeira fila; e ao final, para quitar as marcas.

- `grid[Line(line)]` é unha fila, e `into_iter` percorre as súas celas; `enumerate` dá a columna de cada unha.
- `-(grid.history_size() as i32)` é a primeira fila do historial: ten como moito `SCROLLBACK_LINES`, e cabe nun `i32`.

```rust
fn remove(self, grid: &mut Grid<Cell>, found: Found) {
    if let Some(line) = found.top {
        for cell in &mut grid[Line(line)] {
            cell.flags.remove(MARK);
        }
    }
    let Some(point) = found.cursor else {
        return;
    };
    let cell = &mut grid[point];
    cell.c = self.cursor_char;
    cell.flags.remove(MARK);
    if !(0..grid.screen_lines() as i32).contains(&point.line.0) {
        return;
    }
    let last_column = Column(grid.columns() - 1);
    let cursor = &mut grid.cursor;
    // The next character goes after the one of the cell, maybe on the
    // same row now.
    if self.pending_wrap && point.column < last_column {
        cursor.point = Point::new(point.line, point.column + 1);
        cursor.input_needs_wrap = false;
    } else {
        cursor.point = point;
        cursor.input_needs_wrap = self.pending_wrap;
    }
}
```

**`remove`** quita as marcas: a da primeira cela, en toda a súa fila, porque o *reflow* pode levala a outra columna, cando a liña empezaba no historial e se xunta co seu anaco ao ensanchar; e a da cela do cursor, que recupera o seu carácter. Despois, se esa cela está na pantalla, o cursor pasa a ela:

- se esperaba despois da última columna, e a súa cela xa non está na última, vai á cela seguinte, e xa non espera: o seguinte carácter vai despois do da cela, quizais agora na mesma fila;
- se non, queda na cela, e espera se esperaba. Coma no xterm, un cursor que esperaba segue esperando aínda que a fila se estreite; no ConPTY, non.

Compróbao a proba `the_cursor_keeps_its_place_after_the_last_character_of_a_full_row`. `remove` recibe `self` por valor: despois, as marcas xa non serven. `let Some(point) = found.cursor else { return; };` (*let-else*) sae se non atopou a cela do cursor.

### Onde empeza a pantalla: `window_top` e `last_text_row`

```rust
fn window_top(grid: &Grid<Cell>, rows: i32, first: Option<i32>) -> i32 {
    let cursor = grid.cursor.point.line.0;
    let bottom = last_text_row(grid).map_or(cursor, |last| max(last, cursor));
    let mut top = bottom - rows + 1;
    if let Some(first) = first {
        top = max(top, first);
    }
    min(top, cursor)
}

fn last_text_row(grid: &Grid<Cell>) -> Option<i32> {
    let rows = grid.screen_lines() as i32;
    (0..rows).rev().find(|&line| {
        grid[Line(line)]
            .into_iter()
            .any(|cell| !matches!(cell.c, ' ' | '\t'))
    })
}
```

**`window_top`**: onde ten que empezar unha pantalla de `rows` filas, coma unha fila da pantalla de agora ou, negativa, do historial. Coma no Windows Terminal:

1. a fila máis baixa que ten que caber, `bottom`, é a última con texto ou a do cursor, a que estea máis abaixo;
2. a pantalla empeza onde foi parar a primeira fila, `first`, salvo que así `bottom` non caiba: entón, tan abaixo como faga falta (`bottom - rows + 1`);
3. e nunca debaixo do cursor: se o texto que hai debaixo del non cabe, a pantalla empeza na súa fila, e as filas que non caben vanse por abaixo.

Sen `first`, se a marca non apareceu, a pantalla remata en `bottom`.

**`last_text_row`**: a fila máis baixa da pantalla con texto, coma a mide o ConPTY: os espazos e os tabuladores non contan, aínda que teñan cor. Para `alacritty_terminal`, un espazo con cor ocupa a cela.

### Mover a pantalla: `move_screen`

```rust
fn move_screen(grid: &mut Grid<Cell>, top: i32) {
    let rows = grid.screen_lines();
    match top.cmp(&0) {
        Ordering::Greater => {
            let count = top as usize;
            grid.scroll_up::<Color>(&(Line(0)..Line(rows as i32)), count);
            grid.cursor.point.line -= count;
            grid.saved_cursor.point.line = max(grid.saved_cursor.point.line - count, Line(0));
        }
        Ordering::Less => {
            let columns = grid.columns();
            grid.resize::<Color>(false, rows + (-top) as usize, columns);
        }
        Ordering::Equal => {}
    }
}
```

**`move_screen`** move a pantalla para que a súa primeira fila sexa `top`:

- **máis abaixo**, con `top` positivo: `Grid::scroll_up`, coa pantalla enteira coma rexión, sobe as `top` primeiras filas ao historial, e entran filas baleiras abaixo, coma cando o programa escribe na última fila. O cursor sobe con elas, e tamén o cursor gardado (`ESC 7`), como moito ata a primeira fila;
- **máis arriba**, con `top` negativo, unha fila do historial: `Grid::resize` sen *reflow* (`false`) fai a pantalla `-top` filas máis alta, e énchea coas filas do historial que hai enriba dela, co cursor máis abaixo. Despois, `resize` devólvea ao tamaño novo, quitando as filas de abaixo.

Na súa rama, `top` e `-top` son positivos: os `as usize` non perden nada.

**Onde meiga e o ConPTY non coinciden.** Hai casos raros, que corrixe a seguinte escritura do programa ([ADR 0016 › Consecuencias](../adr/0016-o-historial-o-tamano-e-o-zoom.md#consecuencias)): unha liña que empezaba no historial xúntase coa súa parte da primeira fila ao ensanchar, coma no Windows Terminal, e no ConPTY non, porque non ten historial; os espazos con cor ao final dunha liña ocupan celas en `alacritty_terminal`, e o ConPTY quítaos ao repartir; e un cursor que esperaba despois da última columna segue esperando, coma no xterm, e no ConPTY non.

As probas están en [`tests.rs`](#cratesmeiga-coresrctermtestsrs), coas do terminal: cinco do cambio de tamaño e tres da pantalla principal á parte.

## `crates/meiga-core/src/term/handler.rs`

Desde a subfase 1.9: o terminal de `alacritty_terminal` coma o ve o parser. O parser de `vte` chama a un `Handler` por cada orde da saída ([Tecnoloxías › `alacritty_terminal` e `vte`](../tecnoloxias.md#34-alacritty_terminal-e-vte-o-núcleo-do-terminal)). Ata a 1.8 era o propio `Term`; desde a 1.9 é `Hooked`, que lle pasa ao `Term` as ordes e, cando o programa deixa a pantalla alternativa ou reinicia o terminal, devolve a pantalla principal que un cambio de tamaño deixou á parte ([`term/resize.rs`](#cratesmeiga-coresrctermresizers), [ADR 0016](../adr/0016-o-historial-o-tamano-e-o-zoom.md)). Desde a revisión do código, tamén deixa a vista do historial onde está cando se despraza unha rexión que non empeza arriba ([Manter a vista](#manter-a-vista-view_to_keep-e-keep_view)). E, desde a subfase 3.1, o título do programa e os que este garda non llos pasa ao `Term`, que os baleiraba nun reinicio (`ESC c`) sen avisar: lévaos ao terminal, que os garda ([`term/title.rs`](#cratesmeiga-coresrctermtitlers), [ADR 0029](../adr/0029-o-titulo-da-tab-e-os-titulos-gardados.md)). Ata a 3.1, pasáballe todas as ordes. E, desde a 4.1, segue o win32-input-mode (`?9001`), que `alacritty_terminal` ignora e que ata entón seguía o escáner ([ADR 0032](../adr/0032-o-escaner-osc.md), decisión 1).

**Por que dentro do parser.** A pantalla principal ten que voltar no momento en que o programa deixa a alternativa: a saída que vén despois de `?1049l`, na mesma lectura ou na mesma actualización sincronizada (`?2026`), xa é para ela, e ten que caer nas súas filas, co tamaño novo. `feed` só volta cando o parser leu o bloque enteiro; `vte` garda a saída dunha actualización sincronizada e dálla ao `Handler` dunha vez, cando remata ou cando vence (`end_sync`); e ningún evento de `alacritty_terminal` avisa do cambio de pantalla. Despois de ler, sería tarde. Compróbao a proba `the_output_after_leaving_the_alternate_screen_reaches_the_primary_one`, nos tres casos.

**O win32-input-mode, eiquí.** O ConPTY actívao ao arrancar, e outra vez despois de cada `ESC c`, para que meiga lle envíe eventos de tecla enteiros ([`input.rs`](#cratesmeiga-coresrcinputrs)), coma di o comentario do módulo. `alacritty_terminal` non o coñece, e ata a 4.1 seguíao o escáner, que lía as secuencias coas regras de `vte` para velo só cando o vería `vte`: parábase en cada ESC, e con saída chea de cores custaba o 3,1–3,4 % do tempo do terminal. Pero `vte` xa le esas secuencias, e dálle ao `Handler` `?9001h` e `?9001l`, coma un modo privado que non coñece, e `ESC c`, coma `reset_state`, na orde da saída e sen custo ningún. Desde a 4.1, `Hooked` ségueo eiquí, e o escáner só busca os OSC ([`scanner.rs`](#cratesmeiga-coresrctermscannerrs)). Dentro dunha actualización sincronizada, o modo cambia cando `vte` le a actualización, coma a pantalla, e non cando chega; o ConPTY só o cambia ao arrancar e despois de cada `ESC c` ([ADR 0032](../adr/0032-o-escaner-osc.md), decisión 1).

### `Hooked`

```rust
const WIN32_INPUT_MODE: u16 = 9001;

pub(super) struct Hooked<'a> {
    pub term: &'a mut Term<Listener>,
    pub primary: &'a mut Option<Grid<Cell>>,
    pub region_below_top: &'a mut bool,
    pub titles: &'a mut Titles,
    pub win32_input: &'a mut bool,
}
```

- **`Hooked`**: o terminal, a pantalla principal que un cambio de tamaño deixou á parte, desde a revisión do código, se a rexión de desprazamento do programa empeza máis abaixo da primeira fila da pantalla, desde a subfase 3.1, o que o terminal garda do programa, o seu título e os que gardou ([`Titles`](#titles)), e, desde a 4.1, se o win32-input-mode está activo. Son os campos `term`, `primary`, `region_below_top`, `titles` e `win32_input` do [`Terminal`](#terminal), prestados mentres le o parser. `feed` e `end_sync` crean un en cada lectura: son cinco referencias, sen copiar nada.
- `'a` é o tempo de vida dos cinco préstamos: `Hooked` non pode durar máis ca eles. Os campos son públicos, pero o tipo só se ve no módulo `term` (`pub(super)`).
- **`WIN32_INPUT_MODE`**, desde a 4.1: o número do modo privado do win32-input-mode, 9001, que `vte` non coñece. Ata a 4.1, a mesma constante estaba no escáner.
- `Cell`, neste arquivo, é a cela de `alacritty_terminal`. E `Titles` vén do módulo irmán, `title` (`use super::title::Titles;`).

### Manter a vista: `view_to_keep` e `keep_view`

Desde a revisión do código da subfase 1.9. Cando se despraza unha rexión que non empeza na primeira fila da pantalla, as filas que saen dela non entran no historial, pero `alacritty_terminal` sube igual a vista ([O historial e a súa vista](#o-historial-e-a-súa-vista)): se a vista non está na pantalla, `Grid::scroll_up` súmalle as filas que despraza, entren ou non no historial (só se borra a rexión enteira, non a move). As ordes que poden desprazar así gardan antes onde está a vista e, despois, déixana alí.

```rust
impl Hooked<'_> {
    #[inline]
    fn view_to_keep(&self, below_top: bool) -> Option<usize> {
        if !below_top {
            return None;
        }
        let offset = self.term.grid().display_offset();
        (offset != 0).then_some(offset)
    }

    #[inline]
    fn keep_view(&mut self, view: Option<usize>) {
        let Some(offset) = view else {
            return;
        };
        let moved = self.term.grid().display_offset();
        if moved != offset {
            // Both are at most SCROLLBACK_LINES: they fit in an i32.
            self.term
                .scroll_display(Scroll::Delta(offset as i32 - moved as i32));
        }
    }
}
```

- **`view_to_keep(below_top)`**: onde está a vista, cantas filas por riba da pantalla, se a orde despraza desde unha fila que non é a primeira (`below_top`) e a vista non está na pantalla; se non, `None`. Coa vista na pantalla, `alacritty_terminal` non a move, e non hai nada que manter. Fóra dese caso, o custo é ler un `bool`.
- **`keep_view(view)`**: despois da orde, se a vista se moveu, lévaa onde estaba coa diferenza (`Delta`), negativa: cara abaixo. As dúas contas caben nun `i32`, porque a vista non pasa de `SCROLLBACK_LINES` filas. `scroll_display` dana a pantalla enteira, pero só cando a vista estaba arriba e a rexión se desprazou. A vista volta a unha fila que xa mostraba, así que tampouco pasa do principio do historial.
- **Orde a orde.** Ao final da lectura sería tarde: na mesma lectura, outras ordes poden meter filas no historial, que si teñen que subir a vista, e despois xa non se sabe canto subiu por cada unha.

### Seguir a rexión: `follow_region` e `follow_column_mode`

```rust
#[inline]
fn follow_region(&mut self, top: usize, bottom: Option<usize>) {
    if top < bottom.unwrap_or_else(|| self.term.screen_lines()) {
        *self.region_below_top = top > 1;
    }
}

#[inline]
fn follow_column_mode(&mut self, mode: PrivateMode) {
    if matches!(mode, PrivateMode::Named(NamedPrivateMode::ColumnMode)) {
        self.follow_region(1, None);
    }
}
```

Desde a revisión do código da subfase 1.9. `alacritty_terminal` garda a rexión de desprazamento nun campo privado, `scroll_region`, e non di onde empeza: `region_below_top` séguea.

- **`follow_region`** fai coa copia o mesmo ca `alacritty_terminal` coa rexión cando o programa a pon desde a fila `top`, onde a primeira é 1, ata `bottom` ou, sen `bottom`, ata a última fila: se `top` non está por riba desa fila, `alacritty_terminal` ignora a orde, e a copia tampouco cambia. Se non, a rexión empeza máis abaixo da primeira fila se `top > 1`. `unwrap_or_else` só le o alto da pantalla se fai falta.
- **`follow_column_mode`**: DECCOLM (`?3h` e `?3l`) cambiaba o terminal a 132 ou a 80 columnas. `alacritty_terminal` non cambia as columnas, pero borra a pantalla e pon a rexión na pantalla enteira, coma `CSI 1 r`, desde dentro do `Term`, sen pasar polo `Handler`. `matches!` compara o modo co patrón de DECCOLM. Cunha soa fila, `1` non está por riba da última, e `alacritty_terminal` deixa a rexión que había: `follow_region(1, None)` tamén. Atopouno a segunda volta da revisión do código.
- **A copia segue todos os cambios.** `alacritty_terminal` só cambia a rexión en catro sitios: ao crear o terminal, ao cambiar o seu tamaño, que a pon na pantalla enteira, no reinicio e en `set_scrolling_region`, que tamén chama DECCOLM. Deixar a pantalla alternativa non a cambia. `region_below_top` segue os catro: `false` ao crear o [`Terminal`](#terminal), en [`Terminal::resize`](#o-tamaño-resize) e cando `bring_back_primary` cambia o tamaño, en `reset_state`, e `follow_region` con `set_scrolling_region` e con DECCOLM ([Os métodos de `Handler`](#os-métodos-de-handler)).

### Os métodos de `Handler`

```rust
macro_rules! pass {
    ($(fn $name:ident(&mut self $(, $argument:ident: $type:ty)*);)*) => {
        $(
            #[inline]
            fn $name(&mut self $(, $argument: $type)*) {
                <Term<Listener> as Handler>::$name(self.term $(, $argument)*);
            }
        )*
    };
}

// A method that Handler adds in a new version would do nothing here, without
// any error: the lint asks for all of them.
#[deny(clippy::missing_trait_methods)]
impl Handler for Hooked<'_> {
    pass! {
        fn set_cursor_style(&mut self, style: Option<CursorStyle>);
        fn set_cursor_shape(&mut self, shape: CursorShape);
        fn goto(&mut self, line: i32, column: usize);
        fn goto_line(&mut self, line: i32);
        fn goto_col(&mut self, column: usize);
        fn insert_blank(&mut self, count: usize);
        fn move_up(&mut self, rows: usize);
        fn move_down(&mut self, rows: usize);
        fn identify_terminal(&mut self, intermediate: Option<char>);
        fn device_status(&mut self, kind: usize);
        fn move_forward(&mut self, columns: usize);
        fn move_backward(&mut self, columns: usize);
        fn move_down_and_cr(&mut self, rows: usize);
        fn move_up_and_cr(&mut self, rows: usize);
        fn backspace(&mut self);
        fn carriage_return(&mut self);
        fn bell(&mut self);
        fn substitute(&mut self);
        fn set_horizontal_tabstop(&mut self);
        fn scroll_down(&mut self, rows: usize);
        fn insert_blank_lines(&mut self, count: usize);
        fn erase_chars(&mut self, count: usize);
        fn delete_chars(&mut self, count: usize);
        fn move_backward_tabs(&mut self, count: u16);
        fn move_forward_tabs(&mut self, count: u16);
        fn save_cursor_position(&mut self);
        fn restore_cursor_position(&mut self);
        fn clear_line(&mut self, mode: LineClearMode);
        fn clear_screen(&mut self, mode: ClearMode);
        fn clear_tabs(&mut self, mode: TabulationClearMode);
        fn set_tabs(&mut self, interval: u16);
        fn reverse_index(&mut self);
        fn terminal_attribute(&mut self, attribute: Attr);
        fn set_mode(&mut self, mode: Mode);
        fn unset_mode(&mut self, mode: Mode);
        fn report_mode(&mut self, mode: Mode);
        fn report_private_mode(&mut self, mode: PrivateMode);
        fn set_keypad_application_mode(&mut self);
        fn unset_keypad_application_mode(&mut self);
        fn set_active_charset(&mut self, index: CharsetIndex);
        fn configure_charset(&mut self, index: CharsetIndex, charset: StandardCharset);
        fn set_color(&mut self, index: usize, color: Rgb);
        fn dynamic_color_sequence(&mut self, prefix: String, index: usize, terminator: &str);
        fn reset_color(&mut self, index: usize);
        fn clipboard_store(&mut self, clipboard: u8, base64: &[u8]);
        fn clipboard_load(&mut self, clipboard: u8, terminator: &str);
        fn decaln(&mut self);
        fn text_area_size_pixels(&mut self);
        fn text_area_size_chars(&mut self);
        fn set_hyperlink(&mut self, hyperlink: Option<Hyperlink>);
        fn set_mouse_cursor_icon(&mut self, icon: CursorIcon);
        fn report_keyboard_mode(&mut self);
        fn push_keyboard_mode(&mut self, mode: KeyboardModes);
        fn pop_keyboard_modes(&mut self, count: u16);
        fn set_keyboard_mode(&mut self, mode: KeyboardModes, behavior: KeyboardModesApplyBehavior);
        fn set_modify_other_keys(&mut self, mode: ModifyOtherKeys);
        fn report_modify_other_keys(&mut self);
        fn set_scp(&mut self, path: ScpCharPath, update: ScpUpdateMode);
    }

    #[inline]
    fn set_title(&mut self, title: Option<String>) {
        self.titles.set(title);
    }

    #[inline]
    fn push_title(&mut self) {
        self.titles.push();
    }

    #[inline]
    fn pop_title(&mut self) {
        self.titles.pop();
    }

    #[inline]
    fn input(&mut self, c: char) {
        let view = self.view_to_keep(*self.region_below_top);
        <Term<Listener> as Handler>::input(self.term, c);
        self.keep_view(view);
    }

    #[inline]
    fn put_tab(&mut self, count: u16) {
        let view = self.view_to_keep(*self.region_below_top);
        <Term<Listener> as Handler>::put_tab(self.term, count);
        self.keep_view(view);
    }

    #[inline]
    fn linefeed(&mut self) {
        let view = self.view_to_keep(*self.region_below_top);
        <Term<Listener> as Handler>::linefeed(self.term);
        self.keep_view(view);
    }

    #[inline]
    fn newline(&mut self) {
        let view = self.view_to_keep(*self.region_below_top);
        <Term<Listener> as Handler>::newline(self.term);
        self.keep_view(view);
    }

    #[inline]
    fn scroll_up(&mut self, rows: usize) {
        let view = self.view_to_keep(*self.region_below_top);
        <Term<Listener> as Handler>::scroll_up(self.term, rows);
        self.keep_view(view);
    }

    #[inline]
    fn delete_lines(&mut self, count: usize) {
        let view = self.view_to_keep(self.term.grid().cursor.point.line.0 != 0);
        <Term<Listener> as Handler>::delete_lines(self.term, count);
        self.keep_view(view);
    }

    #[inline]
    fn set_scrolling_region(&mut self, top: usize, bottom: Option<usize>) {
        <Term<Listener> as Handler>::set_scrolling_region(self.term, top, bottom);
        self.follow_region(top, bottom);
    }

    #[inline]
    fn set_private_mode(&mut self, mode: PrivateMode) {
        <Term<Listener> as Handler>::set_private_mode(self.term, mode);
        self.follow_column_mode(mode);
        if mode == PrivateMode::Unknown(WIN32_INPUT_MODE) {
            *self.win32_input = true;
        }
    }

    #[inline]
    fn unset_private_mode(&mut self, mode: PrivateMode) {
        <Term<Listener> as Handler>::unset_private_mode(self.term, mode);
        self.follow_column_mode(mode);
        if mode == PrivateMode::Unknown(WIN32_INPUT_MODE) {
            *self.win32_input = false;
        }
        if !self.term.mode().contains(TermMode::ALT_SCREEN)
            && let Some(primary) = self.primary.take()
            && resize::bring_back_primary(self.term, primary)
        {
            *self.region_below_top = false;
        }
    }

    #[inline]
    fn reset_state(&mut self) {
        <Term<Listener> as Handler>::reset_state(self.term);
        *self.region_below_top = false;
        *self.win32_input = false;
        if self.primary.take().is_some() {
            resize::new_primary(self.term);
        }
    }
}
```

- **`pass!`**, unha macro declarativa (`macro_rules!`), escribe os métodos que só lle pasan a orde ao `Term`: recibe as súas firmas, coma no trait, e escribe cada un cun corpo que chama ao mesmo método do `Term`, cos mesmos argumentos. `$name:ident` e `$argument:ident` recollen identificadores, `$type:ty` un tipo, e `$( … )*` repítese para cada método e para cada argumento.
- `<Term<Listener> as Handler>::$name(self.term …)` chama ao método do trait `Handler` do `Term` coa sintaxe completa, que non deixa dúbida de cal é. `self.term` xa é unha referencia mutábel ao `Term`.
- `#[inline]` suxírelle ao compilador que meta cada método no sitio da chamada: o parser é xenérico no seu `Handler`, e así pasar polo `Hooked` non ten por que engadir unha chamada.
- **Todos os métodos.** Os 71 métodos de `Handler` en `vte` 0.15.0 teñen unha implementación por defecto que non fai nada. Se unha versión nova engadise outro, `Hooked` usaría esa, e a orde non chegaría ao `Term`, sen ningún erro. O *lint* de Clippy `missing_trait_methods`, do grupo *restriction*, que non está activo por defecto, avisa de cada método dun trait que unha implementación non escribe; co nivel `deny`, só neste `impl`, é un erro, e a CI falla ([Tecnoloxías › *Lints*](../tecnoloxias.md#7-lints-rustc-e-clippy)). `pass!` escribe 58 deles, e os outros trece fan algo máis; ata a subfase 3.1, 61 e dez.
- **`set_title`, `push_title` e `pop_title`**, desde a subfase 3.1: OSC 0 e OSC 2, `CSI 22 t` e `CSI 23 t` ([Tecnoloxías › Os títulos da tab](../tecnoloxias.md#65-os-títulos-da-tab)). Non van ao `Term`, senón aos títulos do terminal: [`set`](#o-título-current-e-set), [`push` e `pop`](#os-gardados-push-e-pop) de `Titles`, que deciden tamén se o título cambia ([ADR 0029](../adr/0029-o-titulo-da-tab-e-os-titulos-gardados.md), decisión 2). `vte` dálle a `set_title` o texto do OSC sen os espazos dos extremos, e chama a `push_title` e `pop_title` sen o segundo número da secuencia. Ata a 3.1, `pass!` pasáballos ao `Term`, que gardaba o título e ata 4096 gardados, e avisaba con `Title` ou `ResetTitle` ([Os eventos](#os-eventos-pending-e-listener)); e o seu reinicio baleirábaos sen avisar.
- **`input`, `put_tab`, `linefeed`, `newline` e `scroll_up`**, desde a revisión do código: as ordes que desprazan a rexión de desprazamento cara arriba, desde a súa primeira fila. Un carácter despois da última columna pasa á fila seguinte (`input`), e tamén un tabulador (`put_tab`); ao final da rexión, iso despráza, coma unha liña nova (`linefeed`, que fan LF, VT, FF, `ESC D` e `ESC E`). `newline` é unha liña nova e, no modo `LNM`, un retorno de carro, pero o parser de `vte` 0.15 non o chama. E `CSI n S` despraza a rexión `n` filas (`scroll_up`). Se a rexión empeza máis abaixo da primeira fila da pantalla (`region_below_top`), a vista queda onde estaba ([Manter a vista](#manter-a-vista-view_to_keep-e-keep_view)).
- **`delete_lines`** (`CSI n M`) borra `n` filas desde a do cursor e sobe as de debaixo, dentro da rexión: despraza desde a fila do cursor. Se non é a primeira da pantalla, non entra nada no historial, aínda que a rexión empece arriba, e a vista queda onde estaba. `cursor.point.line.0` é a fila do cursor na pantalla, desde 0.
- **`set_scrolling_region`** (`ESC [ superior ; inferior r`) pásalle a orde ao `Term`, e [`follow_region`](#seguir-a-rexión-follow_region-e-follow_column_mode) segue a rexión nova.
- **`set_private_mode`** e **`unset_private_mode`**, con DECCOLM (`?3h` e `?3l`), seguen tamén a rexión que pon o `Term` ([`follow_column_mode`](#seguir-a-rexión-follow_region-e-follow_column_mode)).
- **O win32-input-mode**, desde a subfase 4.1: despois de pasarlle a orde ao `Term`, que ignora os modos que non coñece, `set_private_mode` actívao con `?9001h`, e `unset_private_mode` desactívao con `?9001l`. `vte` dálle `PrivateMode::Unknown(9001)`, porque non o coñece, e chama a estes métodos unha vez por cada parámetro dunha secuencia privada que remata en `h` ou en `l`, co seu primeiro número: `?1004;9001h` activa os dous modos, `?9001:1h` activa este, e `?1:9001h`, non. Unha secuencia sen `?`, con outra marca, con outro byte intermedio ou con máis de 32 números non chega eiquí. Compróbano as cinco probas do win32-input-mode de [`tests.rs`](#cratesmeiga-coresrctermtestsrs), tamén coa saída que escribe o ConPTY ao arrancar, partida en todas as posicións.
- **`unset_private_mode`**: despois de que o `Term` desactive o modo, se xa non está na pantalla alternativa e hai unha principal á parte, devólvea con [`bring_back_primary`](#a-pantalla-principal-á-parte), que lle dá o tamaño do terminal. Só pasa con `?1049l`, o único modo co que `alacritty_terminal` deixa a pantalla alternativa. `take` colle a principal e deixa `None` no seu sitio. Se o tamaño da principal cambia, a rexión volta ser a pantalla enteira.
- **`reset_state`**: o reinicio (`ESC c`) de `alacritty_terminal` deixa tamén a pantalla alternativa e baleira a principal, co seu historial, e a rexión volta ser a pantalla enteira. Se a principal estaba á parte, o reinicio baleirou a grella que ocupaba o seu sitio: entón a principal á parte vaise, e [`new_primary`](#a-pantalla-principal-á-parte) pon unha nova, con historial. Desde a subfase 3.1, o título e os gardados quedan, coma di o seu comentario: coma no Windows Terminal, en xterm e en Kitty, e coma o título da consola do ConPTY ([ADR 0029](../adr/0029-o-titulo-da-tab-e-os-titulos-gardados.md), decisións 1 e 3). `reset_state` non toca os `titles`, e `alacritty_terminal` só baleira os seus, que xa non usa ninguén. Compróbao a proba `a_reset_keeps_the_title_and_the_saved_ones`, tamén coa principal á parte e dentro dunha actualización sincronizada. Desde a 4.1, apaga tamén o win32-input-mode, coma todos os modos, na orde da saída: despois de `?9001h` e `ESC c`, na mesma lectura ou noutra, está apagado, e o ConPTY actívao outra vez despois do reinicio. `ESC ( c`, que escolle un xogo de caracteres, non é un reinicio, e non chega eiquí; `ESC`, os bytes dun `á`, que `vte` ignora despois dun ESC, e `c`, si.
- `if … && let Some(primary) = self.primary.take() && resize::bring_back_primary(…)` encadea condicións e un `if let` (*let chains*, da edición 2024): cada unha só se avalía se se cumpre a anterior, así que `take` só se chama fóra da pantalla alternativa, e `bring_back_primary` só se hai unha principal á parte.
- Un programa que activa a pantalla alternativa estando xa nela (`?1049h` dúas veces) non cambia nada en `alacritty_terminal`: a principal segue á parte ata que o programa volta a ela ([ADR 0016 › Consecuencias](../adr/0016-o-historial-o-tamano-e-o-zoom.md#consecuencias)).

## `crates/meiga-core/src/term/title.rs`

Desde a subfase 3.1: o título que lle dá o programa á súa tab, e os que garda para dalos outra vez máis tarde ([ADR 0029](../adr/0029-o-titulo-da-tab-e-os-titulos-gardados.md), [Arquitectura › O terminal](../arquitectura.md#o-terminal-da-saída-ao-que-se-mostra), [Tecnoloxías › Os títulos da tab](../tecnoloxias.md#65-os-títulos-da-tab)). O programa pon o título con OSC 0 ou OSC 2, e gárdao e dáo outra vez con `CSI 22 t` e `CSI 23 t` (XTWINOPS), coma vim, que o garda ao entrar e o dá outra vez ao saír. O parser pásalle esas catro ordes ao [`Hooked`](#os-métodos-de-handler), e este, aos `Titles` do [`Terminal`](#terminal), que dá o título con `title` e di que cambiou con [`Changes::title`](#changes-e-modes).

**Por que o terminal, e non `alacritty_terminal`.** Ata a 3.1 levábaos `alacritty_terminal`, que avisaba á fachada con `Title` ou `ResetTitle` ([Os eventos](#os-eventos-pending-e-listener)), e a fachada levaba unha copia do título. Pero o seu reinicio da pantalla (`ESC c`) baleiraba por dentro o título e os gardados, sen avisar: a tab mantiña o título, pero se despois o programa o gardaba e o daba outra vez, a tab quedaba co nome do seu perfil. O reinicio da sesión na súa tab quitaba o título, cun OSC 0 baleiro, pero non os gardados: un `CSI 23 t` do programa novo traía un título do vello. E un título non tiña límite de lonxitude, e cada `CSI 22 t` gardaba outra copia del, ata 4096. Levalos no terminal arranxa as tres cousas, e unha versión nova de `alacritty_terminal` que os trate doutra maneira non cambia nada en meiga ([ADR 0029](../adr/0029-o-titulo-da-tab-e-os-titulos-gardados.md), decisión 2).

**Coma os outros terminais**, polo seu código: un reinicio da pantalla mantén o título no Windows Terminal, en xterm, en Kitty, en WezTerm, en xterm.js, en VS Code e na xanela de Alacritty, aínda que o modelo de `alacritty_terminal` o baleire; xterm e Kitty manteñen tamén os gardados, e gardan como moito 10; o Windows Terminal non garda títulos; e o Windows Terminal ignora os títulos de máis de 1000 unidades de UTF-16, e xterm, os de máis de 1000 bytes. O detalle, coas ligazóns ao seu código e ao do ConPTY, está en [Tecnoloxías › Os títulos da tab](../tecnoloxias.md#65-os-títulos-da-tab).

```rust
//! The title that the program gives to its tab, and the ones that it saves
//! to give again later (ADR 0029). The terminal keeps them, and not
//! `alacritty_terminal`, which empties them on a reset (`ESC c`) without
//! telling it: the tab kept its title, but a program that then saved it
//! and gave it again got none, and the tab showed the name of its profile.
//!
//! As in Windows Terminal and the other terminals, a reset keeps the title,
//! and, as in xterm and Kitty, the saved ones too: they are of the window,
//! not of the screen. A new program in the tab (ADR 0024) starts without
//! them, as in a new tab.

use std::collections::VecDeque;
use std::mem;
```

- O comentario do módulo (`//!`) di o mesmo ca os parágrafos de enriba, e o que fan os dous reinicios. O da pantalla, `ESC c`, mantén o título, coma o Windows Terminal e os outros terminais, e tamén os gardados, coma xterm e Kitty: son da xanela, coma o título, e non da pantalla (decisións 1 e 3). Un programa novo na tab empeza sen eles, coma nunha tab nova (decisión 4): iso faino [`restart`](#o-fin-do-programa-e-o-reinicio-restart-e-at_empty_row_start), que pon uns `Titles` novos.
- `use` trae `VecDeque`, para os gardados, e `mem`, para `mem::take`.
- O módulo é privado (`mod title;`, en [`term/mod.rs`](#cratesmeiga-coresrctermmodrs)), e `Titles` e as constantes son `pub(super)`: só os ve o módulo `term`. `mod.rs` garda uns `Titles` no terminal, e `handler.rs` cámbiaos; as constantes só se usan eiquí.

### Os límites: `SAVED_TITLES` e `MAX_TITLE`

```rust
pub(super) const SAVED_TITLES: usize = 10;

pub(super) const MAX_TITLE: usize = 1000;
```

- **`SAVED_TITLES`**: os títulos que pode gardar o programa con `CSI 22 t`, como moito 10, coma en xterm e Kitty; gardar outro quita o máis vello ([ADR 0029](../adr/0029-o-titulo-da-tab-e-os-titulos-gardados.md), decisión 5). Un programa garda un ou dous. `alacritty_terminal` gardaba ata 4096 (`TITLE_STACK_MAX_DEPTH`).
- **`MAX_TITLE`**: o título máis longo, 1000 unidades de UTF-16, coma as conta Windows: `x`, `ñ` ou `中` contan unha, e un emoji coma `😀`, fóra do plano básico de Unicode, conta dúas. Un título máis longo ignórase, e a tab queda co título de antes, coma no Windows Terminal e no anfitrión da consola do ConPTY, que mantén o seu (decisión 6): así, a tab e a consola seguen dicindo o mesmo. xterm ignora os títulos de máis de 1000 bytes.
- **Canto ocupan.** Unha unidade de UTF-16 son como moito 3 bytes en UTF-8, que é o que garda un `String`: `中` ocupa 3 bytes e conta unha unidade, e `😀`, 4 bytes e dúas. Un título ocupa entón como moito 3000 bytes, e o título e os 10 gardados dunha tab, uns 33 KB. Sen límites, cun título de 1 MB e 4096 gardados, ata 4 GB.
- **O que non limitan.** `vte` garda todo o texto dun OSC mentres chega, sen límite, e conserva despois esa memoria no analizador da tab: un título enorme, aínda que meiga o ignore, ocupa esa memoria ata que se pecha a tab. E meiga conta o título sen os espazos dos extremos, que `vte` xa quitou, e o ConPTY, con eles: un título de case 1000 unidades con espazos nos extremos pode cambiar en meiga e non na consola ([ADR 0029 › Consecuencias](../adr/0029-o-titulo-da-tab-e-os-titulos-gardados.md#consecuencias)).

### `Titles`

```rust
#[derive(Default)]
pub(super) struct Titles {
    current: Option<String>,
    saved: VecDeque<Option<String>>,
    changed: bool,
}
```

| Campo | Que é |
|---|---|
| `current` | O título que deu o programa, se deu algún. `None` é «sen título»: a tab mostra o nome do seu perfil. Nunca é un texto baleiro, nin un de máis de `MAX_TITLE` unidades. |
| `saved` | Os títulos que gardou, o máis novo ao final, como moito `SAVED_TITLES`. Cada un é un `Option`: tamén se garda «sen título». |
| `changed` | O título cambiou desde que o terminal colleu os seus cambios por última vez. |

- **`#[derive(Default)]`**: uns `Titles` sen título, sen gardados e sen cambios, os dun terminal novo. Fanos [`Terminal::new`](#terminal) e, para o programa que empeza outra vez na tab, [`restart`](#o-fin-do-programa-e-o-reinicio-restart-e-at_empty_row_start). Un `VecDeque` baleiro non reserva memoria ata que o programa garda o primeiro título.
- **`VecDeque`**, unha cola dobre: o programa garda e dá outra vez polo mesmo extremo, o do máis novo, coma nunha pila, e, cando xa non caben, o máis vello sae polo outro, sen mover os demais. `alacritty_terminal` gardábaos nun `Vec`, e quitaba o primeiro con `remove(0)`, que move todos os outros.

### O título: `current` e `set`

```rust
pub fn current(&self) -> Option<&str> {
    self.current.as_deref()
}

pub fn set(&mut self, title: Option<String>) {
    let title = title.filter(|title| !title.is_empty());
    if title.as_deref().is_some_and(too_long) {
        return;
    }
    self.replace(title);
}
```

- **`current`**: o título, prestado, sen copialo: o que dá [`Terminal::title`](#terminal). `as_deref` converte o `&Option<String>` nun `Option<&str>`.
- **`set`** atende OSC 0 e OSC 2, desde [`Hooked::set_title`](#os-métodos-de-handler). Paso a paso:
  1. `vte` dálle o texto do OSC sen os espazos dos extremos, coas súas partes xuntas con `;`. Se queda baleiro, coma con `OSC 0 ;` ou cun título só de espazos, é «sen título» (`filter`), coma no Windows Terminal: a tab mostra o nome do seu perfil, ou o do programa da liña de comandos de meiga ([`Launch`](#folder-profile-e-launch) e [`program_name`](#cratesmeiga-coresrcshellrs)), e a xanela, «meiga» ([ADR 0029](../adr/0029-o-titulo-da-tab-e-os-titulos-gardados.md), decisión 2).
  2. Un título de máis de `MAX_TITLE` unidades ignórase: dío [`too_long`](#os-títulos-longos-too_long), e `set` volta sen cambiar nada, nin o título nin `changed` (decisión 6). «Sen título» nunca é longo: `as_deref` presta o texto sen copialo, e `is_some_and` só chama a `too_long` se hai un.
  3. Se non, [`replace`](#o-cambio-take_changed-e-replace) pono, e anota o cambio só se é outro.
- `vte` sempre lle dá un texto (`Some`), aínda que sexa baleiro; `None`, que o trait `Handler` tamén permite, sería tamén «sen título». Ata a 3.1, o título baleiro filtrábao o `Listener`, ao recibir o evento `Title` ([Os eventos](#os-eventos-pending-e-listener)).

### Os gardados: `push` e `pop`

```rust
pub fn push(&mut self) {
    if self.saved.len() == SAVED_TITLES {
        self.saved.pop_front();
    }
    self.saved.push_back(self.current.clone());
}

pub fn pop(&mut self) {
    if let Some(title) = self.saved.pop_back() {
        self.replace(title);
    }
}
```

- **`push`** atende `CSI 22 t`: garda ao final unha copia do título, tamén «sen título» (`None`), que se dá outra vez coma calquera outro: entón a tab mostra outra vez o nome do seu perfil, coma en Alacritty (decisión 5). Se xa hai `SAVED_TITLES`, quita antes o máis vello, o do principio (`pop_front`): un programa que garda sen parar non enche a memoria, e quedan os 10 últimos.
- **`pop`** atende `CSI 23 t`: quita o último gardado, o máis novo (`pop_back`), e dáo outra vez con `replace`, que só anota o cambio se é outro título. Sen ningún gardado, non fai nada: nin cambia o título nin avisa (decisión 5).
- O título que se dá outra vez non pasa por `set`: xa pasou cando se puxo, así que nin é baleiro nin longo.
- **Sen o segundo número.** `vte` chama a `push_title` e a `pop_title` sen o segundo número da secuencia, que en xterm escolle o título da xanela, o da icona ou os dous. meiga só ten un, e sempre o garda e o dá outra vez: `CSI 22 ; 1 t`, que en xterm garda só o da icona, gárdao tamén. Os programas gardan e dan outra vez en parellas, coma vim, así que dá o mesmo ([ADR 0029 › Consecuencias](../adr/0029-o-titulo-da-tab-e-os-titulos-gardados.md#consecuencias)).

### O cambio: `take_changed` e `replace`

```rust
pub fn take_changed(&mut self) -> bool {
    mem::take(&mut self.changed)
}

fn replace(&mut self, title: Option<String>) {
    if self.current != title {
        self.current = title;
        self.changed = true;
    }
}
```

- **`replace`**, privado, pon o título que dan `set` e `pop` só se é outro, e entón marca `changed`. Compara os dous `Option`: un título e «sen título» son distintos, e dous títulos, se o seu texto é distinto. Dar outra vez o mesmo título, cun OSC ou desde os gardados, non é un cambio, e a tab non ten nada que coller (decisión 2).
- **`take_changed`** di se o título cambiou desde a última vez que se lle preguntou, e esquéceo: `mem::take` devolve o valor e deixa `false`. Pregúntao [`take_changes`](#cursor-e-take_changes) ao final de cada `feed` e de cada `end_sync`, para `Changes::title`.
- **Un cambio, aínda que remate igual.** Un título distinto e outra vez o de antes, antes de que o terminal colla os cambios, son un cambio, aínda que o título remate coma estaba: `changed` xa quedou marcado. A tab colle entón o mesmo título, e a sesión, que tamén compara, di que non é outro ([Código › meiga › `session.rs`](meiga.md#cratesmeigasrcsessionrs)).
- **Ata a 3.1**, cada OSC 0 ou 2, e cada `CSI 23 t` cun título gardado, era un cambio, tamén co mesmo título: `alacritty_terminal` avisaba sempre, e só comparaba a sesión, ao coller o título.

### Os títulos longos: `too_long`

```rust
fn too_long(title: &str) -> bool {
    title.encode_utf16().nth(MAX_TITLE).is_some()
}
```

- **`too_long`** di se `title` ten máis de `MAX_TITLE` unidades de UTF-16. `encode_utf16` dá as unidades unha a unha, sen copiar o texto nin reservar memoria, e `nth(MAX_TITLE)` salta as 1000 primeiras e dá a seguinte, a 1001.ª, se a hai: entón o título é longo. Coma di o comentario, conta como moito unha máis ca o límite: dun título de 1 MB só le o principio.
- **Sen atallo.** Un texto ten polo menos tantos bytes en UTF-8 coma unidades en UTF-16, así que un título de ata 1000 bytes nunca é longo, e unha primeira versión miraba antes `title.len() > MAX_TITLE`. Pero non facía falta, porque `nth` xa para ao final dun título curto, e o seu mutante, `>` por `>=`, sobrevivía ás probas, porque daba o mesmo: quitouse ([Tecnoloxías › A cobertura e os mutantes](../tecnoloxias.md#64-a-cobertura-e-os-mutantes)).

**Probas.** Sete, cos `Titles` sós, sen terminal nin parser. `titled(título)` dá uns `Titles` con ese título, co seu cambio xa collido:

| Proba | Que comproba |
|---|---|
| `a_title_is_a_change_only_when_it_is_another_one` | Ao principio, sen título nin cambio. `pwsh` é un cambio, que se colle unha vez; `pwsh` outra vez, non. E `vim` e outra vez `pwsh`, antes de coller os cambios, son un cambio, e o título é `pwsh`. |
| `an_empty_title_is_no_title` | Con `pwsh`, un título baleiro e `None` deixan «sen título», cun cambio; e outra vez, xa sen cambio. |
| `a_title_of_more_than_1000_utf16_code_units_is_ignored` | Con catro caracteres que ocupan un, dous, tres e catro bytes en UTF-8, `x`, `ñ`, `中` e `😀`, o último de dúas unidades en UTF-16: un título de 1000 unidades, mil dos tres primeiros ou 500 emoji, cabe, e é un cambio; cun `x` máis, ou tres veces máis longo, ignórase, sen cambio. |
| `saved_titles_come_back_newest_first` | Con `pwsh` e `vim` gardados e `less` de título, `pop` dá outra vez `vim`, e despois `pwsh`, cada un cun cambio; sen ningún gardado, `pop` non cambia nada. |
| `no_title_can_be_saved_and_given_again` | Gardado «sen título», e despois de `vim`, `pop` deixa outra vez «sen título», cun cambio. |
| `giving_again_the_same_title_is_not_a_change` | `pwsh`, gardado e dado outra vez, segue sendo `pwsh`, sen cambio. |
| `saving_more_than_10_titles_drops_the_oldest` | Gardados once títulos, do `0` ao `10`, e con `last` de título, `pop` dá do `10` ao `1`; despois, sen o `0`, que se foi, non cambia nada. |

**Como se comproba.** Ademais destas probas, sete do terminal comproban as mesmas regras a través de `feed`, coa saída dun programa, tamén cos dous reinicios ([`tests.rs`](#cratesmeiga-coresrctermtestsrs)). En `meiga-win`, a proba de integración `the_conpty_passes_the_titles_of_the_program_unchanged` comproba que o ConPTY empaquetado lle pasa ao terminal OSC 0, OSC 2, `CSI 22 t`, OSC 1, OSC 21, `CSI 23 t` e `ESC c` tal cal, enteiros, nesa orde e sen nada entre eles, e despois do reinicio só `ESC [ ? 1004 h ESC [ ? 9001 h`, sen título: un título que o ConPTY dese pola súa conta cambiaría o da tab ([Código › meiga-win › `tests/conpty.rs`](meiga-win.md#cratesmeiga-wintestsconptyrs)). E `check-tabbar.ps1` comproba na CI, no seu paso 12, coa xanela de meiga, que un reinicio da pantalla mantén o título da tab e só debuxa o terminal, que os títulos gardados despois do reinicio e antes del se dan outra vez, e que un título de 1001 caracteres se ignora, sen debuxar nada na barra ([Código › ferramentas › `check-tabbar.ps1`](ferramentas.md#toolscheck-tabbarps1)).

As trece comprobacións con xanela pasan coa compilación *release* da 3.1, e `check-tabbar.ps1` dá as súas doce liñas `OK:` tamén cunha copia de `meiga.exe` sen o seu ConPTY ao lado, coa consola de Windows. Desde a revisión, o seu paso 7 comproba tamén que o mesmo título unha e outra vez, nunha tab oculta, non esperta a xanela, como moito 20 veces por segundo: desde a 3.1 non é un cambio para o terminal ([`take_changed`](#o-cambio-take_changed-e-replace) e `Changes::title`), así que alí non a esperta nada; ata a 3.1, espertábaa unha vez por intervalo dos fotogramas. Así, a compilación de `main`, sen a 3.1, falla `check-tabbar.ps1` no paso 7, despois de catro liñas `OK:`, porque esperta a xanela decenas de veces por segundo, antes de chegar ao paso 12; e tamén o falla unha copia de meiga na que `replace` marca sempre un cambio, de xeito que o mesmo título outra vez é un cambio, coma ata a 3.1. As outras tres copias estragadas a man fallan o paso 12, cada unha no seu punto: a que co reinicio volta ao nome do perfil e a que non ten límite para a lonxitude dos títulos debuxan a barra, e a que co reinicio baleira os títulos gardados non trae outra vez o gardado antes del. A copia sen cambios pasa.

`cargo mutants --in-diff`, co *diff* da 3.1, fai 28 mutantes, e as probas cóllenos todos. Só coas probas, a cobertura das liñas de `title.rs` é do 100 %, e todas as liñas que cambian en `term/mod.rs` e `term/handler.rs` están cubertas ([Tecnoloxías › A cobertura e os mutantes](../tecnoloxias.md#64-a-cobertura-e-os-mutantes)).

## `crates/meiga-core/src/term/screen.rs`

O que mostra o terminal, en tipos de meiga: as celas, as súas cores e estilos, o cursor, a paleta e as celas que cambiaron. Desde a subfase 1.6, a vista do terminal de `meiga.exe` debuxa desde eiquí, sen coñecer `alacritty_terminal` ([Código › meiga › `view.rs`](meiga.md#cratesmeigasrcviewrs)). Desde a 1.8, as celas din tamén se están na selección.

```rust
pub enum Color {
    Default,
    Indexed(u8),
    Rgb(Rgb),
}

impl Color {
    pub fn rgb(self, palette: &Palette, default: Rgb) -> Rgb {
        match self {
            Color::Default => default,
            Color::Indexed(index) => palette.colors[usize::from(index)],
            Color::Rgb(rgb) => rgb,
        }
    }
}
```

- **`Color`**: a cor do texto ou do fondo dunha cela, tal como a deu o programa: a por defecto, unha das 256 da paleta (0–7 as normais, 8–15 as brillantes, 16–231 un cubo de 6 × 6 × 6 e 232–255 grises), ou unha dada polos seus compoñentes (*truecolor*).
  - Desde a subfase 1.6, **`rgb`** convértea na cor que se debuxa, coa paleta da pantalla. A cor por defecto depende de onde está, a do texto ou a do fondo, así que a dá quen chama (`default`). `usize::from(index)`: un `u8` sempre está entre as 256 cores, así que o índice nunca falla.
  - **Por que o número, e non a cor.** Se o programa cambia a paleta, o texto que xa está na pantalla cambia de cor, coma no xterm; e as celas son máis pequenas ([ADR 0013](../adr/0013-da-saida-a-pantalla.md)).
- **`CellFlags`**: os estilos dunha cela, un bit cada un nun `u16`: negra, atenuada, cursiva, cinco subliñados (simple, dobre, ondulado, de puntos e de trazos), riscada, inversa (o texto e o fondo intercambian as cores) e oculta (só se debuxa o fondo). `contains`, `is_empty` e o operador `|` (o trait `BitOr`) chegan para combinalos e preguntar por eles.
  - Desde a subfase 1.8, **`SELECTED`**, o bit 11: a cela está na selección, e o seu fondo ten a cor da selección da paleta, co texto na súa cor ([ADR 0015](../adr/0015-o-rato-a-seleccion-e-o-portapapeis.md)). Non o pon ningún programa: márcao a fachada ao copiar as celas ([`copy_row`](#os-axudantes)). Ir nos estilos fai que unha cela que entra ou sae da selección sexa unha cela que cambiou, sen outro mecanismo.
- **`Cell`**: unha cela: o carácter (`CellChar`, de [`text.rs`](#cratesmeiga-coresrctextrs)), a cor do texto, a do fondo e os estilos. A cela por defecto (`Default`) é unha cela en branco, coas cores por defecto e sen estilos.
- **`CursorShape`**: bloque, subliñado, barra vertical, o contorno dun bloque, ou oculto.
- **`Cursor`**: a fila, a columna, a forma e se pestanexa: por defecto si, salvo que o programa pida un cursor fixo. meiga só o fai pestanexar se Windows o permite ([Tecnoloxías › O cursor de Windows](../tecnoloxias.md#38-o-cursor-de-windows-pestanexo-tempo-e-grosor)).
- **`Columns`**: as columnas dunha fila, de `start` (incluída) a `end` (excluída); ningunha se `end` non está despois de `start`. `union` dá as que cobren as dúas. Desde a subfase 1.8, `contains` di se unha columna está entre elas: `copy_row` pregúntao, cela a cela, coas columnas seleccionadas.

```rust
pub struct Screen {
    size: GridSize,
    cells: Vec<Cell>,
    changed: Vec<Columns>,
    all_changed: bool,
    cursor: Cursor,
    palette: Palette,
}
```

**`Screen`** é unha copia do que mostra o terminal, e das celas que cambiaron desde o último `clear_changes`. Está pensada para gardarse entre fotogramas: o terminal só copia nela o que cambiou, e a súa memoria reutilízase.

- `cells`: as celas, fila tras fila, nun só `Vec`: a da fila `f` e a columna `c` está na posición `f × columnas + c`. Unha soa reserva de memoria, coas filas seguidas.
- `changed`: as columnas que cambiaron en cada fila. É un só intervalo por fila: se cambian as columnas 2 e 7, debúxanse tamén as do medio. Chega para o que fan os programas, que escriben seguido, e non reserva memoria.
- `all_changed`: cambiaron todas as celas, por exemplo porque cambiou o tamaño ou a paleta.
- `palette`, desde a subfase 1.6: as cores coas que se debuxan as celas, as do terminal na última copia. A vista do terminal debuxa desde a súa `Screen` sen o *lock* do terminal ([ADR 0013](../adr/0013-da-saida-a-pantalla.md)): por iso a copia leva tamén as cores.

| Método | Que fai |
|---|---|
| `new` | Unha pantalla baleira, de 0 × 0, coas cores de meiga (`Palette::MEIGA`). É `const` e non reserva nada: a memoria chega co primeiro cambio de tamaño. |
| `size`, `cursor` | O tamaño e o cursor. |
| `palette` | Desde a 1.6: as cores coas que se debuxan as celas, as do tema cos cambios do programa. |
| `row(row)` | As celas dunha fila, ou ningunha se está fóra. |
| `all_changed` | Se cambiaron todas as celas. |
| `changes()` | As filas que cambiaron, coas súas columnas; se cambiaron todas, cada fila enteira. |
| `clear_changes` | Esquece os cambios, despois de debuxalos. |
| `row_text(row)` | O texto dunha fila sen os espazos do final: cada carácter unha vez, tamén os anchos, e as celas en branco como espazos. Úsano as probas e o exemplo `replay`. |
| `mark_cursor_changed` | Desde a 1.6: engade aos cambios as celas do cursor, se se debuxa (máis abaixo). |
| `forget` | Desde a 2.2: esquece o que mostra, e a seguinte copia dun terminal cópiao enteiro (máis abaixo). |

Estes cinco só os usa a fachada: son `pub(super)`, visíbeis no módulo pai, `term`, e non fóra del.

| Método | Que fai |
|---|---|
| `resize(size)` | Cambia o tamaño: todas as celas en branco e cambiadas. Reutiliza a memoria se cabe. |
| `cells_mut(row, columns)` | As celas dunha fila, para cambialas. |
| `mark_changed(row, columns)` | Engade columnas aos cambios dunha fila. |
| `set_cursor` | Cambia o cursor. |
| `set_palette` | Desde a 1.6: colle a paleta do terminal, e marca o que cambia con ela (máis abaixo). |

- `changes()` devolve `impl Iterator<…> + '_`, como `cells` en `text.rs`: un iterador que toma prestada a pantalla, sen reservar memoria. `filter_map` salta as filas sen cambios, e `then_some` converte a condición nun `Option`.

```rust
pub fn mark_cursor_changed(&mut self) {
    let Cursor {
        row, column, shape, ..
    } = self.cursor;
    if shape != CursorShape::Hidden {
        let end = column.saturating_add(2).min(self.size.columns);
        self.mark_changed(row, Columns::new(column, end));
    }
}

pub(super) fn set_palette(&mut self, palette: &Palette) {
    if self.palette.colors != palette.colors
        || self.palette.foreground != palette.foreground
        || self.palette.background != palette.background
        || self.palette.selection != palette.selection
    {
        self.all_changed = true;
    } else if self.palette.cursor != palette.cursor {
        self.mark_cursor_changed();
    } else {
        return;
    }
    self.palette.clone_from(palette);
}
```

- **`mark_cursor_changed`** marca as celas do cursor: dúas, por se está sobre un carácter ancho, e só unha na última columna (`min`); ningunha se está oculto. Ata a subfase 1.5, `copy_damage` marcaba el mesmo estas celas. Na 1.6 o código pasou eiquí, e é público, porque a vista do terminal tamén o necesita cando o cursor cambia de aspecto sen moverse: ao pestanexar, e ao gañar ou perder o foco a xanela, cando pasa a ser un rectángulo baleiro ou volta á súa forma ([Código › meiga › `view.rs`](meiga.md#cratesmeigasrcviewrs)). `let Cursor { row, column, shape, .. } = self.cursor;` desestrutura o cursor en tres variábeis; `..` ignora o resto dos campos.
- **`set_palette`** colle a paleta do terminal, en cada copia:
  - se cambiaron as 256 cores, a do texto, a do fondo ou, desde a subfase 1.8, a da selección, cambian todas as celas: calquera pode ter unha cor que cambiou. Tamén cambian as marxes da xanela, que teñen a cor do fondo: con `all_changed`, a vista do terminal redebuxa a xanela enteira. A da selección só cambia co tema (`set_theme`), que polo de agora só cambian as probas;
  - se só cambiou a do cursor, só cambian as celas do cursor: ningunha outra a usa;
  - se non cambiou nada, o caso de case todas as copias, non copia nada.

  `clone_from` copia a paleta nova sobre a que xa había, sen crear outra.

```rust
pub fn forget(&mut self) {
    self.size = GridSize::new(0, 0);
}
```

- **`forget`**, desde a subfase 2.2, deixa o tamaño en 0 × 0 sen tocar as celas: na seguinte copia, `copy_damage` ve outro tamaño e copia todo, coma despois dun cambio de tamaño ([`copy_damage`](#copy_damage-copiar-o-que-cambiou), paso 1). `resize` borra as celas do `Vec` sen soltar a súa memoria, así que se a grella do outro terminal cabe nela, non reserva nada; e marca todas as celas como cambiadas. A vista do terminal chámao cando pasa a mostrar outra tab ([Código › meiga › `view.rs`](meiga.md#cratesmeigasrcviewrs)): sen el, `copy_damage` só copiaría as filas que o outro terminal marcou desde a súa última copia, e quedarían as da tab de antes.

**Probas.** Cinco:

| Proba | Que comproba |
|---|---|
| `a_new_screen_is_empty` | Unha pantalla nova mide 0 × 0, sen filas nin cambios. |
| `after_resizing_every_cell_is_blank_and_changed` | Despois de `resize`, todas as celas están en branco e cambiadas, e `clear_changes` esquece os cambios. |
| `changes_add_up_until_they_are_cleared` | Dous cambios na mesma fila xúntanse nun intervalo (2–4 e 6–7 dan 2–7), e unha fila que non existe ignórase. |
| `the_text_of_a_row_shows_each_character_once` | `row_text` mostra os caracteres anchos unha vez, e as celas en branco como espazos, sen os do final. |
| `flags_combine` | Os estilos combínanse con `\|`, e `contains` comproba todos os que se piden. |

## `crates/meiga-core/src/term/selection.rs`

Desde a subfase 1.8: a selección, o texto que se marca co rato, que mostra a pantalla e que copia o portapapeis ([ADR 0015](../adr/0015-o-rato-a-seleccion-e-o-portapapeis.md)). É outro submódulo de `term`, cos métodos de `Terminal` que a crean, a moven, a quitan e a len: os tipos da selección de `alacritty_terminal` quedan dentro da fachada, coma os demais ([ADR 0012](../adr/0012-fachada-do-terminal.md)). Quen a move é o rato da vista ([Código › meiga › `mouse.rs`](meiga.md#cratesmeigasrcmousers)), e quen leva o seu texto ao portapapeis, o módulo que copia e pega ([Código › meiga › `clipboard.rs`](meiga.md#cratesmeigasrcclipboardrs)).

**A selección vive na grella.** `alacritty_terminal` garda a selección no seu `Term` (`term.selection`), cos puntos da súa grella, que contan tamén o historial. Así segue o texto: cando a pantalla se despraza, sobe con el; e desaparece cando o programa borra a parte da pantalla onde está, cambia á pantalla alternativa ou reinicia o terminal. Escribir enriba das súas celas non a quita. Desde a subfase 1.9, desaparece tamén cando cambia o tamaño ([`resize`](#o-tamaño-resize)); e, como vive na grella, colle tamén filas do historial, e segue o seu texto cando se move a vista. Gardala fóra da grella obrigaría a facer todo isto outra vez ([ADR 0015 › Alternativas consideradas](../adr/0015-o-rato-a-seleccion-e-o-portapapeis.md#alternativas-consideradas)).

### `WORD_DELIMITERS`, `SelectionKind` e `Side`

```rust
pub(super) const WORD_DELIMITERS: &str = " \t\"'`()[]{}<>,;|│";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SelectionKind {
    Cells,
    Words,
    Lines,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Side {
    Left,
    Right,
}
```

- **`WORD_DELIMITERS`**: os caracteres que rematan unha palabra nun dobre clic: o espazo e o tabulador, as comiñas (`"`, `'` e `` ` ``), as parénteses, os corchetes, as chaves e os ángulos, e `,`, `;`, `|` e `│`, a liña vertical das táboas debuxadas con liñas de caixa. Unha ruta, un URL ou unha posición nun arquivo, coma `C:\x\f.rs:12:5`, son unha soa palabra. Os de `alacritty_terminal` (`SEMANTIC_ESCAPE_CHARS`) teñen tamén `:`, que partiría esa posición, e non teñen `;`. É `pub(super)`: `Terminal::new` dállos a `alacritty_terminal` na súa configuración.
- **`SelectionKind`**: como medra unha selección desde onde empeza, segundo os clics que a empezan:

  | Variante | Clics | Que selecciona |
  |---|---|---|
  | `Cells` | Un | Desde un lado dunha cela ata un lado doutra. |
  | `Words` | Dous | Palabras enteiras, que rematan nos caracteres de `WORD_DELIMITERS`. Nunha paréntese, un corchete, unha chave ou un ángulo, ata o que o pecha ou o abre; nos outros caracteres de `WORD_DELIMITERS`, ou nun destes sen parella, só a súa cela. |
  | `Lines` | Tres | Liñas enteiras, con todas as filas dunha liña que non colleu nunha. |

- **`Side`**: unha metade dunha cela. Unha selección de celas empeza e remata nun lado dunha cela: unha cela entra nela cando se pasa da súa metade. Dáo [`cell_at`](#a-cela-debaixo-do-punteiro-cell_at), de `mouse.rs`. Desde a 3.2, úsao tamén o campo de texto, para o lado da cela onde está o punteiro ([`press` e `drag`](#o-rato-press-e-drag)).
- `alacritty_terminal` ten os seus propios tipos para isto, `SelectionType` e o seu `Side`, que é o seu `Direction`. meiga ten os seus para que eses non saian da fachada, e o `From<Side> for Direction` do final do arquivo converte o lado.

### Crear, mover e quitar a selección

```rust
impl Terminal {
    pub fn select(&mut self, kind: SelectionKind, point: GridPoint, side: Side) {
        let point = self.grid_point(point);
        let selection = match kind {
            SelectionKind::Words => self.word_at(point),
            SelectionKind::Cells => Selection::new(SelectionType::Simple, point, side.into()),
            SelectionKind::Lines => Selection::new(SelectionType::Lines, point, side.into()),
        };
        self.term.selection = Some(selection);
    }

    pub fn select_to(&mut self, point: GridPoint, side: Side) {
        let point = self.grid_point(point);
        if let Some(selection) = &mut self.term.selection {
            selection.update(point, side.into());
        }
    }

    pub fn clear_selection(&mut self) -> bool {
        let had = self.has_selection();
        self.term.selection = None;
        had
    }

    pub fn has_selection(&self) -> bool {
        self.selection_range().is_some()
    }
    …
}
```

| Método | Que fai |
|---|---|
| `select(kind, point, side)` | Empeza unha selección de `kind` no lado `side` da cela `point` da pantalla, en vez da que había. Unha de celas está baleira ata que remata noutro lado; unha de palabras ou de liñas xa ten a palabra ou a liña de `point`. |
| `select_to(point, side)` | Move o final da selección ao lado `side` da cela `point`: a selección vai desde onde empezou ata alí, tamén cara atrás, con palabras ou liñas enteiras se é diso. Sen selección, non fai nada. |
| `clear_selection` | Quita a selección, e di se había unha con celas: así quen chama sabe se ten que debuxar outra vez. |
| `has_selection` | Se hai unha selección con celas: un clic que non se moveu non ten ningunha. |

- **As clases de `alacritty_terminal`** (`SelectionType`): `Simple` para as celas, `Semantic` para as palabras e `Lines` para as liñas. A selección rectangular, `Block`, aínda non se usa.
- `alacritty_terminal` garda a selección coma dous puntos da grella, o do comezo e o do final, cada un co seu lado, e calcula as súas celas cada vez que se piden. Unha selección de celas que empeza e remata no mesmo lado da mesma cela, ou que vai do lado dereito dunha ao esquerdo da seguinte, non ten ningunha.
- `side.into()` converte o `Side` de meiga no de `alacritty_terminal`. `if let Some(selection) = &mut self.term.selection` toma prestada a selección para cambiala no seu sitio.
- O rato da vista chama a `select` cando baixa o botón esquerdo, coa clase dos clics que contou ([`Clicks`](#os-clics-seguidos-clicksettings-e-clicks)), e a `select_to` cando o rato pasa a outra cela ou a outro lado e, desde a subfase 1.9, cando a vista se move mentres a selección medra, coa roda ou soa, para que remate debaixo do punteiro.

```rust
pub fn selection_text(&self) -> Option<String> {
    let mut text = self.term.selection_to_string()?;
    // alacritty_terminal ends a selection of lines with LF; pasted
    // into a shell, it would run the line.
    if text.ends_with('\n') {
        text.pop();
    }
    Some(text).filter(|text| !text.is_empty())
}

pub(super) fn selection_range(&self) -> Option<SelectionRange> {
    self.term
        .selection
        .as_ref()
        .and_then(|selection| selection.to_range(&self.term))
}
```

- **`selection_text`**: o texto da selección, de `selection_to_string` de `alacritty_terminal`: as súas filas rematan con LF, salvo a última e as dunha liña que non colleu nunha fila, e sen os brancos do final; un carácter ancho vai unha vez. `alacritty_terminal` remata con LF a selección de liñas: pegada nunha shell, executaría a liña, así que se quita. Unha selección sen texto, coma unha cela baleira despois do final da liña, dá `None`. O portapapeis recíbeo con CR LF, que pon `write_text` ([Código › meiga-win › `clipboard.rs`](meiga-win.md#cratesmeiga-winsrcclipboardrs)).
- O `?` despois de `selection_to_string()` devolve `None` se non hai selección, ou se non ten celas; e `filter` converte un texto baleiro en `None`.
- **`selection_range`**, só para a fachada (`pub(super)`): as celas da selección na grella, se ten algunha. `to_range` calcúlaas cada vez: estende as palabras e as liñas, e axústaas á grella. É un `SelectionRange` de `alacritty_terminal`, coa primeira e a última cela, incluídas. Úsano `has_selection` e [`copy_damage`](#copy_damage-copiar-o-que-cambiou).

### O dobre clic: `word_at`

```rust
fn word_at(&self, point: Point) -> Selection {
    // As alacritty_terminal does, but only once: it would search for
    // the other bracket at every frame, maybe through all the history.
    if let Some(other) = self.term.bracket_search(point) {
        return cells(point.min(other), point.max(other));
    }
    // alacritty_terminal would also take the words on both sides.
    let cell = &self.term.grid()[point];
    if !cell
        .flags
        .intersects(Flags::WIDE_CHAR_SPACER | Flags::LEADING_WIDE_CHAR_SPACER)
        && WORD_DELIMITERS.contains(cell.c)
    {
        return cells(point, point);
    }
    Selection::new(SelectionType::Semantic, point, Direction::Left)
}
```

**`word_at`** dá a selección dun dobre clic en `point`:

1. **Nunha paréntese, un corchete, unha chave ou un ángulo**, `bracket_search` busca o que o pecha ou o abre, saltando as parellas que hai dentro, e a selección vai dun ao outro, os dous incluídos. `alacritty_terminal` fai o mesmo nas súas seleccións de palabras, pero buscándoo outra vez cada vez que calcula as celas, en cada fotograma, quizais por todo o historial: meiga búscao unha vez, e garda unha selección de celas.
2. **Nun dos caracteres de `WORD_DELIMITERS`**, coma un espazo ou unha coma, ou nunha paréntese sen parella, só a súa cela: `alacritty_terminal` collería tamén as palabras dos dous lados. A metade dereita dun carácter ancho (`WIDE_CHAR_SPACER`) e a cela baleira que deixa ao final dunha fila un carácter ancho que non colleu (`LEADING_WIDE_CHAR_SPACER`) gardan un espazo, pero non son espazos: nelas, o dobre clic selecciona a palabra.
3. **No resto**, unha selección de palabras de `alacritty_terminal` (`Semantic`), que vai cara aos dous lados ata os caracteres de `WORD_DELIMITERS`, dentro da mesma liña aínda que ocupe varias filas, e que, ao arrastrar, segue collendo palabras enteiras.

```rust
fn cells(start: Point, end: Point) -> Selection {
    let mut selection = Selection::new(SelectionType::Simple, start, Direction::Left);
    selection.update(end, Direction::Right);
    selection
}
```

- **`cells`** fai unha selección de celas desde o lado esquerdo de `start` ata o dereito de `end`: as dúas incluídas.
- `point.min(other)` e `point.max(other)`: os puntos de `alacritty_terminal` compáranse pola liña e despois pola columna, así que o primeiro é o que vai antes no texto.

### As celas da pantalla e as da grella

```rust
fn grid_point(&self, point: GridPoint) -> Point {
    let row = point.row.min(self.size.rows - 1);
    let column = point.column.min(self.size.columns - 1);
    term::viewport_to_point(
        self.term.grid().display_offset(),
        Point::new(usize::from(row), Column(usize::from(column))),
    )
}
```

**`grid_point`** converte a cela da pantalla no punto da grella de `alacritty_terminal`, coma `copy_row`: a fila que se ve, contando o historial que hai por riba (`display_offset`, que desde a subfase 1.9 non é 0 mentres a vista mostra o historial). Unha cela fóra da pantalla pasa a ser a máis próxima: a resta nunca baixa de 0, porque a grella ten polo menos `MIN_SIZE`.

```rust
pub(super) fn selected_columns(range: &SelectionRange, line: Line, columns: u16) -> Columns {
    if line < range.start.line || line > range.end.line {
        return Columns::default();
    }
    // The columns of the grid are the ones of the screen, so they fit in a
    // u16.
    let column = |column: Column| column.0.min(usize::from(columns)) as u16;
    let start = if line == range.start.line {
        column(range.start.column)
    } else {
        0
    };
    let end = if line == range.end.line {
        column(range.end.column).saturating_add(1)
    } else {
        columns
    };
    Columns::new(start, end.min(columns))
}

pub(super) fn shown_rows(
    range: &SelectionRange,
    display_offset: usize,
    rows: u16,
) -> std::ops::Range<u16> {
    let offset = i32::try_from(display_offset).unwrap_or(i32::MAX);
    let row = |line: i32| line.saturating_add(offset).clamp(0, i32::from(rows)) as u16;
    row(range.start.line.0)..row(range.end.line.0.saturating_add(1))
}
```

- **`selected_columns`**: as columnas que a selección colle na fila da grella `line`, dunha fila de `columns` columnas: ningunha fóra das súas filas; desde a súa primeira columna na primeira fila, e desde 0 nas outras; e ata a súa última columna, incluída, na última fila, e ata o final nas outras. Úsaa `copy_row`, a través de [`selected_cells`](#os-axudantes), que engade as outras metades dos caracteres anchos.
- **`shown_rows`**: as filas da pantalla, de `rows`, nas que está a selección, cando a pantalla mostra a grella desde `display_offset` liñas por riba do seu final. As liñas do historial son negativas, e a fila da pantalla é a liña máis `display_offset`; as que quedan fóra da pantalla córtanse (`clamp`). Úsaa `copy_damage`, para copiar outra vez as filas da selección vella e da nova.
- Os `as u16` despois de `min` e de `clamp` non cortan nada: o valor xa cabe. `saturating_add` e `unwrap_or(i32::MAX)` evitan desbordar cos números máis grandes.

As probas da selección están en [`tests.rs`](#cratesmeiga-coresrctermtestsrs), coas do terminal.

## `crates/meiga-core/src/term/scanner.rs`

O escáner que le a saída do programa antes do parser, para atopar o que di a shell de si mesma e `vte` ignora ([ADR 0007](../adr/0007-integracion-coa-shell.md) e [ADR 0032](../adr/0032-o-escaner-osc.md), [Tecnoloxías › O cartafol e o prompt que di a shell](../tecnoloxias.md#68-o-cartafol-e-o-prompt-que-di-a-shell)): o seu cartafol, con OSC 7 ou OSC 9;9, e onde está entre o seu *prompt* e a saída dunha orde, con OSC 133. Só le os bytes: o parser recibe exactamente os mesmos. O que atopa dállo, en orde, a quen o chama, e o terminal gárdao nun [`ShellState`](#cratesmeiga-coresrctermshell_staters) ([`feed`](#terminal)).

**Ata a subfase 4.1** seguía o win32-input-mode (`?9001`) e o reinicio que o apaga (`ESC c`): lía as secuencias de control coas regras de `vte`, e parábase en cada ESC. Con saída chea de cores, coma a de `bigfile`, con 4,4 ESC cada 100 bytes, custaba o 3,1–3,4 % do tempo do terminal, e só percorrer os ESC un a un xa levaba o 3,2 %: lendo tamén os OSC, non podía quedar por baixo do 1 % que pide o plan. Desde a 4.1, `?9001` e `ESC c` ségueos o [`Hooked`](#os-métodos-de-handler), que os recibe de `vte` na súa orde e sen custo, e o escáner só busca os OSC (decisión 1).

```rust
//! The scanner that reads the output of the program before the parser, to
//! find what the shell says about itself, which `vte` ignores (ADR 0007 and
//! ADR 0032): its folder, with OSC 7 or OSC 9;9, and where it is between its
//! prompt and the output of a command, with OSC 133. It only reads the
//! bytes: the parser gets them unchanged.
//!
//! An OSC starts at `ESC ]`. The scanner looks for its `]` with `memchr`,
//! many bytes at a time, and checks that an ESC comes right before it, so it
//! does not stop at the other sequences, as the colours, nor at the text; in
//! a read with many `]` without an ESC, as a list, it looks for the pair with
//! `memchr::memmem` instead. From there it reads the OSC as `vte` does, also
//! when it comes split between reads, and it ends it where `vte` ends it: at
//! BEL, CAN, SUB or any ESC. The only OSC that it misses is one with bytes
//! between the ESC and the `]` that do not end the escape in `vte`: the
//! controls, which it runs, and DEL and the bytes from 0x80, which it
//! ignores, as `ESC \x80 ]`. No program writes them.

use memchr::memmem::Finder;

use super::shell_state::{Folder, Mark};
```

- O comentario do módulo (`//!`) di o mesmo ca os parágrafos de enriba, e como o fai: atopa onde empeza cada OSC sen pararse no resto da saída, e le o OSC coma `vte`.
- `use` trae `Finder`, o buscador de `memmem`, e os tipos do que di a shell, `Folder` e `Mark`, do módulo irmán, [`shell_state`](#cratesmeiga-coresrctermshell_staters).
- O módulo é privado (`mod scanner;`, en [`term/mod.rs`](#cratesmeiga-coresrctermmodrs)), pero `Scanner` e `Report` reexpórtanse con `pub use`: o [*benchmark*](#cratesmeiga-corebenchesscannerrs) mide o escáner só, desde fóra do crate.

**Coma `vte`.** O escáner ten que ver un OSC só cando o ve `vte`, e co mesmo texto, tamén cando chega partido entre dúas lecturas. Por iso le cada un coas regras de `vte` 0.15.0 ([o OSC](https://github.com/alacritty/vte/blob/v0.15.0/src/lib.rs#L407-L434) e [o seu texto](https://github.com/alacritty/vte/blob/v0.15.0/src/lib.rs#L543-L559), en `src/lib.rs`):

- un OSC empeza en `ESC ]`, tamén no medio doutra secuencia ou doutra cadea: en `vte`, un ESC sempre empeza unha secuencia nova;
- dentro del, `vte` ignora os controis de 0x00 a 0x1f, menos BEL, CAN, SUB e ESC, e garda o resto, tamén DEL e os bytes desde 0x80;
- o seu número son os bytes ata o primeiro `;`, e compárase coma texto: `07` non é `7`;
- e remata en BEL, en CAN, en SUB ou en calquera ESC, sen esperar polo `\` de ST: nos catro casos, `vte` despacha o OSC, e o ESC empeza a secuencia seguinte, que pode ser outro OSC.

### Os límites: `MAX_TEXT`, `KEPT_CAPACITY` e `LONE_BRACKETS`

```rust
const MAX_TEXT: usize = 128 * 1024;

const KEPT_CAPACITY: usize = 4 * 1024;

const LONE_BRACKETS: usize = 16;

const BEL: u8 = 0x07;
const CAN: u8 = 0x18;
const SUB: u8 = 0x1a;
const ESC: u8 = 0x1b;
```

- **`MAX_TEXT`**: o texto dun OSC que garda o escáner, como moito 128 KiB (decisión 3). Cabe a ruta máis longa de Windows, 32 767 caracteres de UTF-16: en UTF-8, cada un ocupa como moito 3 bytes, 98 301 en total, uns 96 KiB, tamén coas súas comiñas e o `9;`. Un OSC máis longo ignórase enteiro, coma en xterm, en GNOME VTE e en kitty: o escáner nunca usa un anaco del, que sería outro cartafol. `vte` non ten límite, e segue recibindo todos os bytes. Só a usan este arquivo e as súas probas.
- **`KEPT_CAPACITY`**: a memoria que conserva o texto entre dous OSC, 4 KiB. Abonda para unha ruta habitual, que o *prompt* dá outra vez despois de cada orde sen reservar memoria; despois dun OSC máis longo, [`end_text`](#o-texto-read_text-e-end_text) devolve o resto. `vte`, en cambio, conserva a memoria do seu OSC máis longo ata que se pecha a tab ([ADR 0032 › Consecuencias](../adr/0032-o-escaner-osc.md#consecuencias)).
- **`LONE_BRACKETS`**: cantos `]` sen un ESC xusto antes deixa pasar o escáner, un a un, antes de buscar a parella `ESC ]` enteira: 16 ([`next_start`](#buscar-o-comezo-find_start-next_start-e-start)).
- `BEL`, `CAN`, `SUB` e `ESC`: os catro bytes que rematan un OSC en `vte`.

### O que atopa: `Report`, `Kind` e `State`

```rust
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Report {
    Folder(Folder),
    Mark(Mark),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Kind {
    Url,
    ConEmu,
    Mark,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum State {
    Ground { escape: bool },
    Number,
    Text(Kind),
}
```

- **`Report`**: o que dixo a shell, na orde na que o dixo: un cartafol, de OSC 7 ou OSC 9;9, ou unha marca, de OSC 133. Os seus tipos, `Folder` e `Mark`, son os de [`shell_state.rs`](#cratesmeiga-coresrctermshell_staters), que os garda.
- **`Kind`**: os OSC que le: `Url`, OSC 7, o cartafol coma URL; `ConEmu`, OSC 9, de ConEmu, que con 9;9 é o cartafol coma ruta de Windows; e `Mark`, OSC 133, as marcas arredor do *prompt* e das ordes.
- **`State`**: onde está o escáner, tamén entre dúas lecturas. Dos estados de `vte`, só lle importan os do OSC: o resto da saída é `Ground` para el.

  | Estado | Onde está |
  |---|---|
  | `Ground { escape }` | Buscando o seguinte `ESC ]`. `escape`: a última lectura rematou nun ESC, e a seguinte pode empezar co seu `]`. |
  | `Number` | Despois de `ESC ]`: o número do OSC, ata o seu primeiro `;`. |
  | `Text(kind)` | O texto dun OSC que le, ata o seu final. |

### `Scanner`

```rust
#[derive(Clone, Debug)]
pub struct Scanner {
    finder: Finder<'static>,
    state: State,
    number: [u8; 3],
    number_len: usize,
    text: Vec<u8>,
    too_long: bool,
}

impl Default for Scanner {
    fn default() -> Self {
        Self::new()
    }
}

impl Scanner {
    pub fn new() -> Self {
        Self {
            finder: Finder::new(b"\x1b]"),
            state: State::Ground { escape: false },
            number: [0; 3],
            number_len: 0,
            text: Vec::new(),
            too_long: false,
        }
    }
}
```

| Campo | Que é |
|---|---|
| `finder` | O buscador de `memmem` para a parella `ESC ]`. `Finder::new` prepárao unha vez, ao crear o escáner, e non en cada lectura; `'static`, porque a parella é un texto do programa. |
| `state` | Onde está ([`State`](#o-que-atopa-report-kind-e-state)). |
| `number` e `number_len` | O número do OSC, mentres aínda pode ser un dos que le, 7, 9 ou 133: como moito tres cifras. |
| `text` | O texto do OSC, sen os bytes que `vte` ignora nel, como moito `MAX_TEXT` bytes. Un `Vec` que se reutiliza dun OSC a outro. |
| `too_long` | O texto pasou de `MAX_TEXT`: o OSC ignórase. |

- `impl Default for Scanner` dá o mesmo ca `new`: Clippy pídeo a un tipo público con `new` sen argumentos (`new_without_default`).
- Un escáner novo non reserva memoria para o texto: o `Vec` medra co primeiro OSC que le.

### Ler a saída: `scan`

```rust
pub fn scan(&mut self, bytes: &[u8], mut report: impl FnMut(Report)) {
    let mut rest = bytes;
    while !rest.is_empty() {
        rest = match self.state {
            State::Ground { escape } => self.find_start(rest, escape),
            State::Number => self.read_number(rest),
            State::Text(kind) => self.read_text(rest, kind, &mut report),
        };
    }
}
```

- **`scan`** le os seguintes bytes da saída, e dálle a `report`, en orde, o que dixo a shell neles. Cada estado le o que pode de `rest` e devolve o que queda despois: buscar o comezo dun OSC, ler o seu número ou ler o seu texto. Cada volta le polo menos un byte ou cambia de estado: o bucle sempre avanza, e remata cando non queda nada.
- `impl FnMut(Report)`: `report` é calquera función ou *closure* que recibe un `Report` e pode cambiar o que captura. O terminal pásalle unha que o aplica ao seu `ShellState`, e o *benchmark*, unha que conta. `scan` é xenérica, así que o compilador fai unha copia dela para cada unha, sen chamadas indirectas.
- Os bytes nunca se copian: `rest` é un anaco de `bytes`, cada vez máis curto.

### Buscar o comezo: `find_start`, `next_start` e `start`

```rust
#[inline]
fn find_start<'a>(&mut self, bytes: &'a [u8], escape: bool) -> &'a [u8] {
    if escape && bytes[0] == b']' {
        self.start();
        return &bytes[1..];
    }
    let mut from = 0;
    let mut lone = 0;
    while let Some(at) = self.next_start(bytes, from, &mut lone) {
        from = at + 2;
        match bytes.get(from) {
            // The number of an OSC that the scanner does not read, as the
            // 8 of the links: it goes on as text, until the next ESC ].
            // An ESC that ends it right away can start that one.
            Some(&byte)
                if !matches!(
                    byte,
                    b'1' | b'7' | b'9' | 0x00..=0x06 | 0x08..=0x17 | 0x19 | 0x1c..=0x1f
                ) => {}
            _ => {
                self.start();
                return &bytes[from..];
            }
        }
    }
    self.state = State::Ground {
        escape: bytes.last() == Some(&ESC),
    };
    &[]
}

#[inline]
fn next_start(&self, bytes: &[u8], mut from: usize, lone: &mut usize) -> Option<usize> {
    while *lone < LONE_BRACKETS {
        let at = from + memchr::memchr(b']', &bytes[from..])?;
        // A `]` at the start goes with an ESC of the last read, which
        // `escape` follows.
        if at > 0 && bytes[at - 1] == ESC {
            return Some(at - 1);
        }
        *lone += 1;
        from = at + 1;
    }
    self.finder.find(&bytes[from..]).map(|at| from + at)
}

fn start(&mut self) {
    self.state = State::Number;
    self.number_len = 0;
}
```

- **O camiño rápido.** Fóra dun OSC, nada importa ata o seguinte `ESC ]`, e `next_start` atópao lendo moitos bytes á vez, con instrucións SIMD: o escáner non se para nas cores, nos movementos do cursor nin no texto. Case toda a saída pasa só por eiquí.
- **`next_start`** busca o `]` con `memchr::memchr`, e mira se o byte de antes é un ESC: entón devolve onde está o ESC. Buscar un byte é máis rápido ca buscar a parella con `memmem`, e `]` é raro na maior parte da saída, tamén na chea de cores, onde ESC non o é: cada cor leva un ESC, e ningún `]`. A primeira versión buscaba só a parella con `memmem`: atopaba o mesmo, pero coa saída de cores do *benchmark*, `colours`, levaba o 1,3 % do tempo do terminal, e agora o 1,1 % ([ADR 0032 › Alternativas consideradas](../adr/0032-o-escaner-osc.md#alternativas-consideradas)).
- **Moitos `]` sen ESC**, coma en listas ou en JSON, serían unha parada cada un. `lone` cóntaos e, despois de `LONE_BRACKETS`, o resto do anaco vai a `memmem`, que busca a parella enteira e non depende de cantos `]` haxa. `lone` empeza en 0 cada vez que `scan` chama a `find_start`: ao principio dunha lectura en `Ground`, e despois de cada OSC que empeza a ler. Un OSC que se salta non o pon a 0.
- **As dúas buscas atopan os mesmos comezos.** `memmem` busca a parella desde `from`, e `memchr`, un `]` desde `from` cun ESC antes: só serían distintas se `from` quedase entre un ESC e o seu `]`, e nunca queda, porque é o principio do anaco, o byte despois dun `]` ou o despois dun `ESC ]`. Un `]` no principio do anaco non é un comezo (`at > 0`), coma di o comentario: vai cun ESC da lectura anterior, que xa mirou `escape`, ou con ningún. Compróbano a proba `many_brackets_without_an_esc_do_not_hide_an_osc`, con 0, 1, 15, 16, 17 e 200 `]` antes de cada OSC, e a de referencia, con listas e lecturas longas ([`scanner/tests.rs`](#cratesmeiga-coresrctermscannertestsrs)).
- **Un OSC que non pode ser dos que le** sáltao alí mesmo: se o byte despois de `ESC ]` non é `1`, `7`, `9` nin un dos controis que `vte` ignora nun OSC, o número non pode ser 7, 9 nin 133, e a busca segue desde alí, sen ler o número. Así pasan o 8 das ligazóns, o 0 e o 2 dos títulos ou o 52 do portapapeis. O seu texto non importa: un OSC dos que le o escáner só pode empezar noutro `ESC ]`. A busca segue xusto despois do `]`, coma di o comentario: se o OSC saltado remata nun ESC que empeza outro, coma en `ESC ] ESC ] 133 ; A`, ese é o seguinte.
- **O byte seguinte pode non chegar aínda**: se o anaco remata xusto despois de `ESC ]`, `bytes.get(from)` dá `None`, e o OSC empeza igual (`start`): o número chega na lectura seguinte.
- **Un ESC ao final da lectura.** Se non hai máis comezos, o escáner queda en `Ground` e anota se o último byte é un ESC (`escape`). A lectura seguinte pode empezar co `]` do OSC, coma cando o ConPTY parte a saída entre os dous: entón o OSC empeza antes de buscar nada. Se empeza por outro byte, o ESC non empezaba ningún OSC. Unha lectura baleira non cambia nada: `scan` non chama a `find_start` sen bytes.
- `bytes[0]` non falla, porque `find_start` sempre ten bytes; e en `next_start`, `from` nunca pasa do final: `&bytes[from..]` pode quedar baleiro, e entón non hai nada que atopar, e `?` volta con `None`.
- `#[inline]` suxírelle ao compilador que meta `find_start` e `next_start` no sitio da chamada: son o camiño de case toda a saída.
- **`start`**: `ESC ]`, empeza un OSC: o escáner pasa a ler o seu número, desde cero.

**O que non ve.** `vte` tamén empeza un OSC se entre o ESC e o `]` hai bytes que non saen do escape: os controis, que executa, e DEL e os bytes desde 0x80, que ignora, coma en `ESC \x80 ]`. O escáner só mira o byte xusto antes do `]`, e non o ve. Ningún programa os escribe, e mirar cara atrás ata o ESC, pasando eses bytes, custaba o 8,0 % do tempo do terminal con texto cheo de `]` (decisión 1 e alternativas). Compróbao a proba `it_misses_an_osc_with_bytes_that_vte_ignores_after_the_esc`.

### O número: `read_number` e `start_text`

```rust
fn read_number<'a>(&mut self, bytes: &'a [u8]) -> &'a [u8] {
    for (at, &byte) in bytes.iter().enumerate() {
        match byte {
            // The controls that `vte` ignores in an OSC.
            0x00..=0x06 | 0x08..=0x17 | 0x19 | 0x1c..=0x1f => {}
            // An OSC with only a number: none that the scanner reads.
            BEL | CAN | SUB => {
                self.state = State::Ground { escape: false };
                return &bytes[at + 1..];
            }
            // It ends, and a new sequence starts, maybe another OSC.
            ESC => {
                self.state = State::Ground { escape: false };
                return &bytes[at..];
            }
            b';' => {
                self.state = match &self.number[..self.number_len] {
                    b"7" => self.start_text(Kind::Url),
                    b"9" => self.start_text(Kind::ConEmu),
                    b"133" => self.start_text(Kind::Mark),
                    _ => State::Ground { escape: false },
                };
                return &bytes[at + 1..];
            }
            _ => {
                if self.number_len == self.number.len() {
                    self.state = State::Ground { escape: false };
                    return &bytes[at + 1..];
                }
                self.number[self.number_len] = byte;
                self.number_len += 1;
                // Any other OSC goes on as text, until the next ESC ].
                if !matches!(
                    &self.number[..self.number_len],
                    b"7" | b"9" | b"1" | b"13" | b"133"
                ) {
                    self.state = State::Ground { escape: false };
                    return &bytes[at + 1..];
                }
            }
        }
    }
    &[]
}

fn start_text(&mut self, kind: Kind) -> State {
    self.too_long = false;
    State::Text(kind)
}
```

- **O número, coma o le `vte`**: os bytes ata o primeiro `;`, sen os controis que ignora nun OSC. Ten que ser exactamente `7`, `9` ou `133`: `vte` compara o número coma texto, e `07`, `9` cun espazo detrás ou `1337` son outros números.
- **Mentres pode ser un deles.** Cada cifra vai a `number` e, se o que leva xa non é o principio de 7, 9 ou 133 (`7`, `9`, `1`, `13` ou `133`), o OSC é outro, e o escáner volta a buscar o seguinte `ESC ]`: o resto do OSC é texto para el. Tampouco garda máis de tres cifras: unha cuarta, coma o 7 de `1337`, xa non cabe (`number_len == number.len()`).
- **O final.** BEL, CAN ou SUB rematan un OSC que só tiña número, coma `ESC ] 7 BEL`: sen texto, non di nada. Un ESC remátao tamén, pero non se le: `return &bytes[at..]` devolve o anaco desde el, e `find_start` mírao coma o posíbel comezo doutro OSC.
- **`;`** remata o número: se é 7, 9 ou 133, `start_text` esquece se o OSC anterior era longo de máis, e o escáner pasa a ler o texto deste (`Text(kind)`), que empeza baleiro: [`end_text`](#o-texto-read_text-e-end_text) baleirou o do anterior ao rematalo. Se non, volta a `Ground`.
- **Partido entre lecturas**, `number` e `number_len` gardan as cifras, e o estado segue en `Number`.

### O texto: `read_text` e `end_text`

```rust
fn read_text<'a>(
    &mut self,
    bytes: &'a [u8],
    kind: Kind,
    report: &mut impl FnMut(Report),
) -> &'a [u8] {
    for (at, &byte) in bytes.iter().enumerate() {
        match byte {
            0x00..=0x06 | 0x08..=0x17 | 0x19 | 0x1c..=0x1f => {}
            BEL | CAN | SUB => {
                self.end_text(kind, report);
                return &bytes[at + 1..];
            }
            ESC => {
                self.end_text(kind, report);
                return &bytes[at..];
            }
            _ => {
                if self.text.len() == MAX_TEXT {
                    self.too_long = true;
                } else {
                    self.text.push(byte);
                }
            }
        }
    }
    &[]
}

fn end_text(&mut self, kind: Kind, report: &mut impl FnMut(Report)) {
    self.state = State::Ground { escape: false };
    if !self.too_long
        && let Some(said) = read(kind, &self.text)
    {
        report(said);
    }
    self.text.clear();
    self.text.shrink_to(KEPT_CAPACITY);
}
```

- **`read_text`** garda o texto do OSC despois do `;` do número, sen os controis que `vte` ignora nel; o resto vai ao texto, tamén `;`, DEL e os bytes desde 0x80. Ata o final do OSC, o mesmo ca en `vte`: BEL, CAN, SUB ou calquera ESC, tamén o de `ESC \` (ST) e o dun ESC que empeza outra secuencia, coma `ESC [ 0 m`. `vte` despacha o OSC nos catro casos, e o escáner tamén o le. Cun ESC, `read_text` volta sen lelo, coma `read_number`.
- **Máis de `MAX_TEXT` bytes**: o texto deixa de medrar, e `too_long` marca o OSC, que se ignora cando remata. O escáner segue lendo ata o final, sen gardar nada máis, porque só alí sabe onde empeza o seguinte.
- **`end_text`**: o OSC rematou. Se non era longo, [`read`](#ler-o-texto-read-url_folder-decode-e-hex) di o que di o texto, se di algo que garda o terminal, e vai a `report`. Despois, o texto baléirase, e `shrink_to(KEPT_CAPACITY)` devolve a memoria que pase de 4 KiB: unha ruta habitual, que o *prompt* dá outra vez despois de cada orde, non reserva memoria outra vez, e despois dun OSC longo só quedan 4 KiB. Se a capacidade xa é menor, `shrink_to` non fai nada.
- `if !self.too_long && let Some(said) = read(kind, &self.text)` encadea unha condición e un `if let` (*let chains*, da edición 2024): `read` só se chama se o OSC non era longo.

### Ler o texto: `read`, `url_folder`, `decode` e `hex`

```rust
fn read(kind: Kind, text: &[u8]) -> Option<Report> {
    match kind {
        Kind::Url => url_folder(text).map(Report::Folder),
        Kind::ConEmu => conemu_folder(text).map(Report::Folder),
        Kind::Mark => mark(text).map(Report::Mark),
    }
}

fn url_folder(text: &[u8]) -> Option<Folder> {
    const SCHEME: &[u8] = b"file://";
    let (scheme, rest) = text.split_at_checked(SCHEME.len())?;
    if !scheme.eq_ignore_ascii_case(SCHEME) {
        return None;
    }
    let slash = memchr::memchr(b'/', rest)?;
    let (host, path) = rest.split_at(slash);
    Some(Folder::Url {
        host: decode(host)?,
        path: decode(path)?,
    })
}

fn decode(text: &[u8]) -> Option<String> {
    let mut bytes = Vec::with_capacity(text.len());
    let mut rest = text;
    while let Some((&byte, after)) = rest.split_first() {
        if byte == b'%'
            && let [high, low, ..] = after
            && let (Some(high), Some(low)) = (hex(*high), hex(*low))
        {
            bytes.push(high << 4 | low);
            rest = &after[2..];
        } else {
            bytes.push(byte);
            rest = after;
        }
    }
    String::from_utf8(bytes).ok()
}

const fn hex(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => Some(byte - b'0'),
        b'a'..=b'f' => Some(byte - b'a' + 10),
        b'A'..=b'F' => Some(byte - b'A' + 10),
        _ => None,
    }
}
```

- **`read`** le o texto segundo o OSC, e o que di, un `Folder` ou unha `Mark`, vai nun `Report`; `None` se non di nada que garde o terminal. Non usa o `Scanner`: a proba de referencia le con ela o texto que lle dá `vte`, e así só compara onde empeza e onde remata cada OSC ([`scanner/tests.rs`](#cratesmeiga-coresrctermscannertestsrs)).
- **`url_folder`**, OSC 7: `file://equipo/ruta` (decisión 4). O esquema, `file://`, vale tamén en maiúsculas (`eq_ignore_ascii_case`), coma nas URL; outro, coma `http://`, non é un cartafol. A primeira `/` despois del separa o equipo da ruta, que empeza nela: `file:///home/ana` é a ruta `/home/ana`, sen equipo, e `file://ana-pc/C:/x` é `/C:/x`, de `ana-pc`. Sen `/`, coma en `file://ana-pc`, non hai ruta. Vale calquera equipo, coma en kitty, WezTerm e GNOME VTE, porque WSL e SSH dan o seu: que facer cun que non é este decídese na 4.5 e na 5.6.
- `split_at_checked` parte o texto despois do esquema, ou dá `None` se é máis curto, e `?` volta con ese `None` desde a función.
- **`decode`**: cada `%XX`, con dúas cifras hexadecimais, é o byte XX, coma nas URL: `%20` é un espazo. Un `%` sen dúas cifras detrás queda coma está, coma en `%4` ou `%zz`. O resultado ten que ser UTF-8 (`String::from_utf8`): se non, `None`, e o OSC non di nada, coma cun `%ff` só. `let [high, low, ..] = after` comproba, sen índices que poidan fallar, que quedan polo menos dous bytes.
- **`hex`**: o valor dunha cifra hexadecimal, en minúsculas ou en maiúsculas. É `const fn`, coma as funcións que se poden avaliar ao compilar.

### O cartafol de ConEmu e as marcas: `conemu_folder`, `mark` e `exit_code`

```rust
fn conemu_folder(text: &[u8]) -> Option<Folder> {
    let path = text.strip_prefix(b"9;")?;
    let path = match path {
        [b'"', inside @ .., b'"'] => inside,
        _ => path,
    };
    if path.is_empty() {
        return None;
    }
    String::from_utf8(path.to_vec()).ok().map(Folder::Windows)
}

fn mark(text: &[u8]) -> Option<Mark> {
    let mut fields = text.split(|&byte| byte == b';');
    match fields.next()? {
        b"A" => Some(Mark::Prompt),
        b"B" => Some(Mark::Command),
        b"C" => Some(Mark::Output),
        b"D" => Some(Mark::Done(fields.next().and_then(exit_code))),
        _ => None,
    }
}

fn exit_code(field: &[u8]) -> Option<i32> {
    let text = std::str::from_utf8(field).ok()?;
    text.parse::<i32>()
        .ok()
        .or_else(|| text.parse::<u32>().ok().map(u32::cast_signed))
}
```

- **`conemu_folder`**, OSC 9;9: o texto dun OSC 9 empeza por `9;`, e o resto é a ruta de Windows, tal cal ou entre comiñas, coma a escriben os *prompts* de PowerShell e cmd: `ESC ] 9 ; 9 ; "C:\Users" ESC \`. Os outros OSC 9, coma o progreso (`9;4`) ou as notificacións, non son cartafoles. As comiñas quítanse só se están nos dous extremos, coma no Windows Terminal e en kitty; e a ruta é todo o que vén despois, tamén con `;`, que unha ruta de Windows pode levar: o Windows Terminal córtaa no primeiro (decisión 4). Unha ruta baleira, tamén `""`, ou que non é UTF-8, non é un cartafol.
- O patrón `[b'"', inside @ .., b'"']` casa cun anaco de polo menos dous bytes que empeza e remata en `"`, e garda en `inside` os do medio: unha `"` soa non casa, e é a ruta.
- **`mark`**, OSC 133: a primeira parte, ata o primeiro `;`, é a letra. A, o *prompt* empeza; B, o *prompt* remata, e empeza a orde que se escribe; C, a orde corre, e empeza a súa saída; e D, a orde rematou. As outras letras da proposta dos *semantic prompts* (P, N, L e I), e as opcións despois da letra (`aid=`, `k=`, `err=`…), ignóranse, coma pide a proposta. A letra vai soa: `Ax` ou `a` non son marcas.
- De D, a segunda parte é o código de saída, se é un número (`exit_code`); se non o é, ou se non hai, a orde rematou sen código: `Done(None)`.
- **`exit_code`**: un número que cabe nun `i32`, con signo ou sen el, ou un de 32 bits sen signo, coma os dá Windows, que queda no `i32` que garda Windows: `3221225477`, `0xC0000005`, é −1073741819. `u32::cast_signed` toma os mesmos 32 bits coma un número con signo. `+7` é 7, porque `parse` admite o signo; `1` cun espazo diante non é un número; e `4294967296` non cabe en 32 bits.

**O que custa.** Medido co [*benchmark*](#cratesmeiga-corebenchesscannerrs), o escáner leva do tempo do terminal o 0,5 % con texto, o 0,6–0,7 % con listas, o 0,75 % cunha gravación de `bigfile` e o 0,45 % coa resposta dunha IA gravada, `ai-stream.mrec`; e coa saída máis densa en cores, unha cor para cada carácter, o 1,1 %. O escáner de antes levaba o 3,3 % con `bigfile`. Cada OSC custa máis ca o texto: cun *prompt* cada 20 filas, que le enteiro, arredor do 2,8 %, e con dúas ligazóns en cada fila, que salta, arredor do 1,7 %, a ritmos que ninguén escribe.

**Probas.** Dezaoito, nun arquivo aparte: [`scanner/tests.rs`](#cratesmeiga-coresrctermscannertestsrs). Ata a 4.1, as dez do escáner anterior ían ao final deste arquivo; as súas regras, as do win32-input-mode, compróbanas agora cinco probas do terminal, a través de `feed` ([`tests.rs`](#cratesmeiga-coresrctermtestsrs)).

**Como se comproba.** A proba de referencia compara o escáner con `vte` en saídas aleatorias partidas en anacos aleatorios, e `proptest` reduce un fallo á saída máis curta que o mostra. Unha gravación de pwsh, cun *prompt* que escribe OSC 133 e 9;9, próbao con saída de verdade ([`tests/recordings.rs`](#cratesmeiga-coretestsrecordingsrs)); e, en `meiga-win`, a proba `the_conpty_passes_what_the_shell_says_unchanged` comproba que o ConPTY empaquetado lle pasa ao terminal OSC 133, OSC 9;9 e OSC 7 tal cal, enteiros, en orde e sen nada entre eles ([Código › meiga-win › `tests/conpty.rs`](meiga-win.md#cratesmeiga-wintestsconptyrs)). `cargo mutants` atopou que cambiar calquera dos dous límites, `MAX_TEXT` ou `KEPT_CAPACITY`, pasaba sen que ningunha proba o vise, porque as probas usaban as mesmas constantes: desde `c23211c`, dúas probas compróbanos con números escritos a man, a ruta máis longa de Windows e a memoria dunha ruta habitual ([Tecnoloxías › A cobertura e os mutantes](../tecnoloxias.md#64-a-cobertura-e-os-mutantes)).

## `crates/meiga-core/src/term/scanner/tests.rs`

Desde a subfase 4.1: as dezaoito probas do escáner, nun arquivo aparte para que `scanner.rs` sexa máis curto. É un submódulo de `scanner` (`#[cfg(test)] mod tests;`, ao final de `scanner.rs`), así que ve tamén o seu privado, coma `Kind`, `read`, `LONE_BRACKETS` ou o campo `text`. Rust busca o arquivo dun submódulo de `scanner.rs` no cartafol `scanner/`, ao seu lado.

```rust
use alacritty_terminal::vte::{Parser, Perform};
use proptest::prelude::*;

use super::*;
```

- `use super::*;` trae todo o de `scanner.rs`, tamén o privado e o que este importa, coma `Folder` e `Mark`.
- `Parser` é o parser de `vte`, o mesmo que hai dentro de `alacritty_terminal`, e `Perform`, o trait que recibe o que le: son a referencia da última proba. E `proptest::prelude::*` trae o que usa `proptest`: a macro `proptest!`, `Strategy`, `prop_oneof!`, `any`…

### Os axudantes: `scanned`, `windows`, `url` e `done`

```rust
fn scanned(parts: &[&[u8]]) -> Vec<Report> {
    let mut scanner = Scanner::new();
    let mut found = Vec::new();
    for part in parts {
        scanner.scan(part, |report| found.push(report));
    }
    found
}
```

- **`scanned`**: o que atopa un escáner novo en `parts`, lidas unha despois doutra, coma lecturas: así se proba unha saída partida.
- **`windows`**, **`url`** e **`done`** escriben os `Report` que se esperan: un cartafol de Windows, un de URL co seu equipo e a súa ruta, e o fin dunha orde co seu código.

### As probas de cada regra

| Proba | Que comproba |
|---|---|
| `it_finds_the_folder_of_osc_7_and_osc_9_9` | OSC 7 de `ana-pc`, cun `%20`, dá o equipo e a ruta `/C:/x y`; un sen equipo, rematado en `ESC \`, `/home/ana`; e OSC 9;9, entre comiñas e sen elas, a ruta de Windows. |
| `it_finds_the_marks_of_osc_133` | A, B, C, D co código 0 e D sen código, seguidos, dan as cinco marcas, en orde. |
| `the_exit_code_is_a_number_of_32_bits` | `1`, `-1`, `+7` e os dous extremos dun `i32` son eses números; sen signo, coma os dá Windows, `3221225477` é −1073741819, e `4294967295`, −1. `4294967296`, `-2147483649`, `x`, nada e `1` cun espazo diante non son códigos: a orde remata sen el. As opcións despois do código ignóranse, e unha opción no sitio do código (`aid=7`) non é un código. |
| `other_letters_and_options_of_osc_133_are_ignored` | A e B coas súas opcións son marcas; P, N, L, I, `a`, `Ax`, `AB`, nada e `;A`, non. |
| `what_is_not_a_folder_is_ignored` | Non son cartafoles: OSC 7 con `http://`, con `file:/`, sen ruta, baleiro ou só co número; os outros OSC 9, o progreso, unha notificación ou `9;99`; unha ruta baleira, tamén entre comiñas; un texto que non é UTF-8, tamén despois de descodificar os `%XX`; e os OSC doutros números, coma os le `vte`: `07`, `9` cun espazo detrás, `1337` e `0`. |
| `the_url_is_percent_decoded` | `%41%62%2f` é `Ab/`; un `%` sen dúas cifras queda, en `%4`, en `%zz%` e no primeiro de `%%41`, que dá `%A`; o equipo descodifícase tamén, e o esquema vale en maiúsculas; e os caracteres de varios bytes, en `%XX` ou tal cal, chegan enteiros. |
| `only_the_quotes_around_the_path_go` | As comiñas só se quitan se están nos dous extremos; unha `"` soa é a ruta; e a ruta é todo o que vén despois de `9;9;`, tamén con `;`, con comiñas ou sen elas. |
| `an_osc_ends_as_in_vte` | BEL, ST, o ESC doutra secuencia (`ESC [ 0 m`), un ESC só ao final, CAN e SUB rematan un OSC, e o OSC conta. E o ESC que remata un pode empezar o seguinte: de tres OSC seguidos, os dous primeiros rematan no ESC do seguinte. |
| `the_controls_that_vte_ignores_in_an_osc_are_not_part_of_it` | Un LF no medio do número e un CR antes da letra non están: `1`, LF, `33;`, CR e `A` é a marca A. Na ruta, NUL non está, e DEL si, coma en `vte`. |
| `other_oscs_and_sequences_do_not_hide_the_next_one` | Despois dun título, unha ligazón cos seus dous OSC 8, unha copia ao portapapeis, OSC 1, OSC 21, OSC 1337, unha cor *truecolor*, texto con `]` e `[1]`, unha DCS e unha APC, atópase o OSC 7 do final; e un `]` sen ESC xusto antes non empeza ningún OSC. |
| `it_finds_them_when_they_come_split` | Unha saída coma a dun *prompt* de pwsh, con cores, as marcas, os dous cartafoles, unha ligazón e un título, dá os seus sete `Report` en orde: nunha lectura, partida en dúas en todas as posicións posíbeis, e byte a byte. |
| `an_esc_at_the_end_of_a_read_can_start_one_in_the_next` | Un ESC ao final dunha lectura e un `]` ao principio da seguinte empezan un OSC, tamén despois doutro ESC só, e cunha lectura baleira no medio; pero non se a seguinte empeza por outro byte, coma `x` ou `[`. |
| `it_misses_an_osc_with_bytes_between_its_esc_and_its_bracket` | O que non ve, adrede: `ESC`, 0x80 e `]`, e `ESC`, LF e `]`, bytes que non saen do escape en `vte`, que os le coma OSC. Outro ESC non é un deses bytes: o OSC empeza no último. |
| `a_text_longer_than_the_maximum_is_ignored_whole` | Unha ruta que, co seu `9;`, ocupa `MAX_TEXT` bytes cabe; cun byte máis, lida en anacos de 1000 bytes, ignórase enteira, e o OSC 133 que vén despois conta. Ao final, o texto devolveu a súa memoria, ata `KEPT_CAPACITY`. |
| `many_brackets_without_an_esc_do_not_hide_an_osc` | Con 0, 1, 15, 16, 17 e 200 `[1]` antes dun OSC 133, e outros tantos `]` antes dun OSC 9;9, atópanse os dous: antes de que o escáner pase a buscar a parella, e despois. E un `]` ao principio dunha lectura, despois doutra que non rematou nun ESC, non empeza ningún OSC. |
| `the_longest_path_of_windows_fits` | 32 767 caracteres de tres bytes en UTF-8, `中`, entre comiñas, son un cartafol. |
| `the_text_keeps_the_memory_of_a_usual_path_and_gives_back_the_rest` | Despois dunha ruta de 3000 caracteres, o texto conserva a súa memoria, e outra igual non pide máis; despois dunha de 100 000, quédanlle como moito 4 KiB. |
| `it_agrees_with_vte` | **A referencia**: 2 000 saídas aleatorias, partidas en anacos aleatorios, co escáner e con `vte` ([máis abaixo](#a-proba-de-referencia-it_agrees_with_vte)). |

### A proba de referencia: `it_agrees_with_vte`

```rust
#[derive(Default)]
struct Reference {
    found: Vec<Report>,
    cut: bool,
}

impl Perform for Reference {
    fn osc_dispatch(&mut self, params: &[&[u8]], _bell_terminated: bool) {
        if params.len() == 16 {
            self.cut = true;
        }
        let kind = match params.first().copied() {
            Some(b"7") => Kind::Url,
            Some(b"9") => Kind::ConEmu,
            Some(b"133") => Kind::Mark,
            _ => return,
        };
        // An OSC with only its number has no text.
        if params.len() < 2 {
            return;
        }
        if let Some(report) = read(kind, &params[1..].join(&b';')) {
            self.found.push(report);
        }
    }
}

const fn stays_in_the_escape(byte: u8) -> bool {
    matches!(byte, 0x00..=0x17 | 0x19 | 0x1c..=0x1f | 0x7f..=0xff)
}
```

- **`Reference`**: o que le `vte` nunha saída, coma ten que velo o escáner. Implementa `Perform`, o trait co que o parser de `vte` dá o que le, e del só `osc_dispatch`: os outros métodos teñen unha implementación por defecto que non fai nada. `vte` chámao con cada OSC que remata, cos seus parámetros, partidos nos `;`. Se o primeiro é `7`, `9` ou `133` e hai máis, `join` xúntaos outra vez con `;`, que é o texto que garda o escáner, e a mesma [`read`](#ler-o-texto-read-url_folder-decode-e-hex) le ese texto. Así, a proba compara onde empeza e onde remata cada OSC, e que bytes leva; o que di cada texto compróbano as outras probas.
- **`cut`**: `vte` garda como moito 16 parámetros dun OSC, e o texto despois deles pérdese. Un OSC con 16 marca a saída, que non se pode comparar.
- **`stays_in_the_escape`**: os bytes que non saen do escape en `vte` despois dun ESC: os controis, menos CAN, SUB e ESC, que executa, e DEL e os bytes desde 0x80, que ignora. O escáner non ve un OSC despois deles.

```rust
fn part() -> impl Strategy<Value = Vec<u8>> {
    let pieces: &[&[u8]] = &[
        b"\x1b",
        b"\x1b]",
        b"\x1b]1",
        …
        b"\r\n",
    ];
    let texts: &[&str] = &[
        "7;file://ana-pc/C:/x%20y",
        …
        "07;file://h/x",
    ];
    let ends: &[&[u8]] = &[
        b"\x07", b"\x1b\\", b"\x1b", b"\x18", b"\x1a", b"\x1b[0m", b"",
    ];
    prop_oneof![
        3 => prop::sample::select(pieces).prop_map(<[u8]>::to_vec),
        3 => (prop::sample::select(texts), prop::sample::select(ends))
            .prop_map(|(text, end)| [b"\x1b]", text.as_bytes(), end].concat()),
        2 => "[ -~\u{e1}\u{4e2d}]{0,12}".prop_map(String::into_bytes),
        // Lists, with more ] than the scanner looks for one by one.
        1 => "[\\[\\]0-9,]{8,48}".prop_map(String::into_bytes),
        2 => any::<u8>().prop_map(|byte| vec![byte]),
    ]
}
```

**`part`**: os anacos cos que se fan as saídas, unha `Strategy` de `proptest`, que di como facer un valor ao chou. `prop_oneof!` escolle unha das cinco maneiras, co seu peso:

| Peso | Anaco |
|---|---|
| 3 de 11 | Un de 27 anacos que importan (`select`): ESC, `ESC ]` e `ESC ]` cos comezos dos números (`1`, `13`, `133`, `133;`, `7`, `7;`, `9;` e `9;9;`), outros OSC (un título, unha ligazón, o portapapeis e OSC 1337), unha DCS e unha APC, `ESC [`, dúas cores, ST, `ESC c`, `]`, `;`, BEL, CAN, SUB e CRLF. |
| 3 de 11 | Un OSC enteiro: `ESC ]`, un de 18 textos, uns que le o escáner e outros que ignora, coma `07;file://h/x`, e un de sete finais, BEL, ST, ESC, CAN, SUB, `ESC [ 0 m` ou ningún. |
| 2 de 11 | Texto: de 0 a 12 caracteres, ASCII imprimíbeis, `á` ou `中`, cunha expresión regular. |
| 1 de 11 | Unha lista: de 8 a 48 caracteres de `[`, `]`, cifras e comas, con moitos `]` sen ESC, coma di o comentario. |
| 2 de 11 | Un byte calquera. |

```rust
proptest! {
    #![proptest_config(ProptestConfig::with_cases(2_000))]

    #[test]
    fn it_agrees_with_vte(
        parts in prop::collection::vec(part(), 0..40),
        // Short pieces, and long ones, with many ] in a read.
        sizes in prop::collection::vec(prop_oneof![3 => 1usize..24, 1 => 24usize..2048], 1..16),
    ) {
        let mut output = parts.concat();
        // Out of what the scanner misses.
        for at in 1..output.len() {
            if output[at - 1] == ESC && stays_in_the_escape(output[at]) {
                output[at] = b'x';
            }
        }
        let mut reference = Reference::default();
        Parser::new().advance(&mut reference, &output);
        prop_assume!(!reference.cut);
        let mut scanner = Scanner::new();
        let mut found = Vec::new();
        let mut rest = output.as_slice();
        for &size in sizes.iter().cycle() {
            if rest.is_empty() {
                break;
            }
            let (piece, after) = rest.split_at(size.min(rest.len()));
            scanner.scan(piece, |report| found.push(report));
            rest = after;
        }
        prop_assert_eq!(found, reference.found, "{:?}", output);
    }
}
```

- **`it_agrees_with_vte`**: 2 000 casos (`with_cases`). Cada un xunta ata 39 anacos nunha saída, e cada byte que segue a un ESC e deixaría `vte` no escape pasa a ser un `x`, coma di o comentario: así a saída queda fóra do único que o escáner non ve, adrede. `vte` le a saída enteira, nunha lectura; e o escáner, en anacos de ata 15 tamaños aleatorios, un detrás doutro e outra vez desde o primeiro (`cycle`): tres de cada catro curtos, de 1 a 23 bytes, e un longo, de 24 a 2047 bytes, con moitos `]` nunha lectura. O escáner ten que atopar exactamente o mesmo ca `vte`, na mesma orde (`prop_assert_eq!`), e a mensaxe dun fallo leva a saída (`{:?}`). `prop_assume!` descarta os casos con `cut`.
- **`proptest`** fai os casos ao chou e, se un falla, redúceo (*shrinking*): quita anacos e simplifica os que quedan mentres o caso siga fallando, e mostra o máis pequeno que falla. Ao contrario das outras probas ao chou de meiga, que repiten sempre a mesma secuencia *xorshift*, cada execución proba casos novos.

## `crates/meiga-core/src/term/shell_state.rs`

Desde a subfase 4.1: o que di de si mesma a shell dunha tab ([ADR 0007](../adr/0007-integracion-coa-shell.md) e [ADR 0032](../adr/0032-o-escaner-osc.md), decisión 5): o seu cartafol, e onde está entre o seu *prompt* e a saída dunha orde, co código de saída da última. O [escáner](#cratesmeiga-coresrctermscannerrs) atópao na saída, e o terminal gárdao eiquí, ata que empeza outro programa na tab. Dáo [`Terminal::shell`](#terminal), e [`Changes::shell`](#changes-e-modes) avisa cando cambia. A xanela usarao desde a 4.5, coa integración coa shell; ata entón, móstrao `replay --screen` ([`examples/replay.rs`](#cratesmeiga-coreexamplesreplayrs)).

```rust
//! What the shell of a tab says about itself (ADR 0007 and ADR 0032): its
//! folder, and where it is between its prompt and the output of a command,
//! with the exit code of the last one. The scanner finds it in the output
//! ([`super::scanner`]), and the terminal keeps it here, until a new program
//! starts in the tab.

use std::mem;

use super::scanner::Report;
```

- O comentario do módulo (`//!`) di o mesmo ca o parágrafo de enriba.
- `use` trae `mem`, para `mem::take`, e `Report`, o que atopa o escáner.
- O módulo é privado (`mod shell_state;`, en [`term/mod.rs`](#cratesmeiga-coresrctermmodrs)), pero os seus tipos reexpórtanse con `pub use`: fóra úsanse como `meiga_core::term::ShellState`, `meiga_core::term::Folder` e `meiga_core::term::Mark`.

### O cartafol e as marcas: `Folder` e `Mark`

```rust
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Folder {
    Windows(String),
    Url { host: String, path: String },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mark {
    Prompt,
    Command,
    Output,
    Done(Option<i32>),
}
```

- **`Folder`**: o cartafol da shell, de dúas maneiras:
  - `Windows`: unha ruta de Windows, de OSC 9;9, coma a dan PowerShell e cmd;
  - `Url`: o equipo e a ruta dunha URL, de OSC 7, coma a dan as shells de Linux. En WSL, a ruta é de Linux; e o equipo pode ser outro, coma nunha sesión de SSH, ou ningún, coma en `file:///home/ana`.

  Os dous van en UTF-8, coma os escribiu a shell: meiga non mira se o cartafol existe. Que facer cun cartafol doutro equipo, ou cunha ruta de Linux, decídese cando se use, na 4.5 e na 5.6 ([ADR 0032](../adr/0032-o-escaner-osc.md), decisión 4). Non é o `Folder` dos perfís, onde empeza un perfil ([`profile.rs`](#folder-profile-e-launch)): este é onde está a shell.
- **`Mark`**: as marcas de OSC 133, que a shell escribe arredor do seu *prompt* e das ordes:

  | Variante | Letra | Que di |
  |---|---|---|
  | `Prompt` | A | Empeza o *prompt*. |
  | `Command` | B | Remata o *prompt*, e empeza a orde que se escribe. |
  | `Output` | C | A orde corre, e empeza a súa saída. |
  | `Done(code)` | D | A orde rematou, co seu código de saída, se a shell o deu. |

- `Mark` é `Copy`, porque é pequena: `mark` dáa sen prestala. `Folder` non, porque leva texto: `folder` présta o cartafol, sen copialo.

### O que dixo a shell: `ShellState`

```rust
#[derive(Clone, Debug, Default)]
pub struct ShellState {
    folder: Option<Folder>,
    mark: Option<Mark>,
    exit_code: Option<i32>,
    changed: bool,
}

impl ShellState {
    pub const fn folder(&self) -> Option<&Folder> {
        self.folder.as_ref()
    }

    pub const fn mark(&self) -> Option<Mark> {
        self.mark
    }

    pub const fn exit_code(&self) -> Option<i32> {
        self.exit_code
    }

    pub(super) fn apply(&mut self, report: Report) {
        match report {
            Report::Folder(folder) => {
                if self.folder.as_ref() != Some(&folder) {
                    self.folder = Some(folder);
                    self.changed = true;
                }
            }
            Report::Mark(mark) => {
                if self.mark != Some(mark) {
                    self.mark = Some(mark);
                    self.changed = true;
                }
                if let Mark::Done(code) = mark
                    && self.exit_code != code
                {
                    self.exit_code = code;
                    self.changed = true;
                }
            }
        }
    }

    pub(super) fn take_changed(&mut self) -> bool {
        mem::take(&mut self.changed)
    }
}
```

| Campo | Que é |
|---|---|
| `folder` | O último cartafol que deu a shell, se deu algún. |
| `mark` | A última marca que escribiu. |
| `exit_code` | O código de saída da última orde que rematou (D), se a shell o deu. |
| `changed` | Algo cambiou desde que o terminal colleu os seus cambios por última vez. |

- **`#[derive(Default)]`**: un estado sen nada e sen cambios, o dunha tab nova. Fano [`Terminal::new`](#terminal) e, para o programa que empeza outra vez na tab, [`restart`](#o-fin-do-programa-e-o-reinicio-restart-e-at_empty_row_start).
- **`folder`**, **`mark`** e **`exit_code`** len o estado; son `const fn`, coma [`Terminal::shell`](#terminal), que o presta.
- **`apply`** colle un `Report` do escáner. Un cartafol ou unha marca só se gardan, e só son un cambio, se son outros: dicir o mesmo outra vez non o é, coma co título ([`replace`](#o-cambio-take_changed-e-replace)). Así, un *prompt* que dá o seu cartafol despois de cada orde non avisa de nada se o cartafol non cambiou. `self.folder.as_ref() != Some(&folder)` compara sen copiar.
- **O código de saída** só o cambia D, e queda mentres se mostra o seguinte *prompt* e corre a seguinte orde, ata o D seguinte. Un D sen código deixa a orde sen el, coma no Windows Terminal: o código de antes era doutra orde. O mesmo D outra vez, co mesmo código, non é un cambio.
- **`take_changed`** di se algo cambiou desde a última vez que se lle preguntou, e esquéceo: `mem::take` devolve o valor e deixa `false`. Pregúntao [`take_changes`](#cursor-e-take_changes) ao final de cada `feed` e de cada `end_sync`, para `Changes::shell`.
- `apply` e `take_changed` son `pub(super)`: só os chama o módulo `term`. Fóra, o estado só se le.
- **As marcas non levan fila**: o terminal sabe en que punto está a shell, pero non en que fila da pantalla escribiu cada marca. Para levala, habería que aplicar cada marca na orde de `vte`, tamén despois dunha actualización sincronizada; e o plan só pide o cartafol, o estado e o código ([ADR 0032 › Alternativas consideradas](../adr/0032-o-escaner-osc.md#alternativas-consideradas)).

**Probas.** Non ten probas propias: compróbano tres probas do terminal a través de `feed`, coa saída dun programa ([`tests.rs`](#cratesmeiga-coresrctermtestsrs)), e a gravación de pwsh, coa saída de verdade ([`tests/recordings.rs`](#cratesmeiga-coretestsrecordingsrs)).

## `crates/meiga-core/src/term/recording.rs`

Gravacións do que escribe un programa no terminal, para reproducilo nas probas e nas medicións sen executar o programa outra vez ([ADR 0012](../adr/0012-fachada-do-terminal.md)). Cada bloque gárdase tal como chegou da pseudoconsola, co momento en que chegou: así pódese reproducir cos mesmos anacos e as mesmas pausas.

O formato, cos números en *little-endian*:

| Bytes | Contido |
|---|---|
| 8 | `MEIGAREC` (`MAGIC`). |
| 2 | A versión do formato: 1 (`VERSION`). |
| 2 + 2 | As columnas e as filas da pseudoconsola. |

E despois, para cada bloque:

| Bytes | Contido |
|---|---|
| 8 | Os microsegundos desde que empezou a gravación. |
| 4 | A lonxitude do bloque. |
| Esa lonxitude | Os bytes do bloque. |

- **`Recorder<W: Write>`** escribe unha gravación en calquera saída que implemente `Write`: un arquivo, con `create`, ou un `Vec<u8>`, nas probas.
  - `new` escribe a cabeceira e empeza a contar o tempo (`Instant::now()`).
  - `record` engade un bloque, co tempo desde o comezo. Un bloque de 4 GiB ou máis non cabe na súa lonxitude (`u32`) e dá un erro.
  - `blocks` e `bytes` contan os bloques e os bytes gravados.
  - `finish` escribe o que quede na memoria e devolve a saída. `create` usa un `BufWriter`, que xunta as escrituras pequenas nunha grande. Destruílo tamén escribe o que quede, pero sen poder informar dun erro.
- **`Recording`** le unha gravación enteira e compróbaa:
  - `read` le un arquivo, e `from_bytes` comproba os datos: o tamaño da cabeceira, a marca `MEIGAREC`, a versión, e que ningún bloque estea cortado. Os erros son `io::Error` do tipo `InvalidData`, cunha mensaxe que di que falla, e en que byte.
  - Garda os datos tal como están e, de cada bloque, só o tempo e onde empeza e remata (`Span`): ler os bloques non os copia.
  - `blocks` dá os bloques en orde (`Block`: o tempo e os bytes, prestados dos datos); `size`, o tamaño da pseudoconsola; `output_len`, os bytes de saída sen as cabeceiras; e `duration`, cando chegou o último bloque.
  - `first_chunk` dá os primeiros bytes como un *array* de tamaño fixo, se os hai, e o patrón `[magic @ .., v0, v1, c0, c1, r0, r1]` sepáraos sen índices: os 8 primeiros en `magic`, e os seguintes un a un.

**Probas.** Catro:

| Proba | Que comproba |
|---|---|
| `a_recording_gives_back_its_blocks_and_size` | Tres bloques, un deles baleiro, gravados nun `Vec` e lidos outra vez: os mesmos bytes, o tamaño, a lonxitude e os tempos en orde. |
| `a_recording_without_blocks_is_valid` | Unha gravación só coa cabeceira é válida, e dura 0. |
| `other_data_is_rejected` | Datos máis curtos ca a cabeceira, sen a marca `MEIGAREC` ou doutra versión dan cadanseu erro. |
| `a_cut_block_is_rejected` | Un bloque cortado, na súa cabeceira ou nos seus bytes, dá `the block at byte 14 is cut`. |

## `crates/meiga-core/src/term/tests.rs`

As cincuenta e catro probas do terminal, nun arquivo aparte para que `mod.rs` sexa máis curto. É un submódulo de `term` (`#[cfg(test)] mod tests;`, ao final de `mod.rs`), así que ve tamén os seus tipos privados. Once axudantes: `copy` copia o terminal enteiro nunha `Screen` nova; `responses` recolle as respostas; `narrow` crea unha cela cun carácter, sen cores nin estilos; desde a subfase 1.8, `marks` mostra as celas seleccionadas dunha fila, `#` as que levan `CellFlags::SELECTED` e `.` as outras; desde a 1.9, `texts` dá o texto de cada fila que se ve, `cursor_at`, a fila e a columna do cursor, e `ten_lines`, un terminal de 10 × 3 coas liñas do 1 ao 10, 7 delas no historial; desde a revisión do código da 1.9, `assert_the_rows_go_into_the_history`, que, coa vista dúas filas arriba, comproba que dúas liñas novas ao final da pantalla meten dúas filas no historial e que a vista segue mostrando as mesmas; desde a 3.1, `title_changed`, os `Changes` só cun título que cambiou; e, desde a 4.1, `win32_input_after`, se o win32-input-mode está activo nun terminal novo despois dunhas lecturas, e `shell_changed`, os `Changes` só cun cambio do que di a shell. Desde a 1.8, cinco probas comproban a selección, e a proba aleatoria cámbiaa tamén. Desde a 1.9, trece comproban o cambio de tamaño, a pantalla principal á parte e a vista do historial, e a proba aleatoria move tamén a vista. Desde a 2.7, dúas comproban o reinicio e se o cursor está ao principio dunha fila baleira. Desde a 3.1, sete comproban o título e os títulos gardados a través de `feed`, coa saída dun programa ([ADR 0029](../adr/0029-o-titulo-da-tab-e-os-titulos-gardados.md); as dos `Titles` sós están en [`term/title.rs`](#cratesmeiga-coresrctermtitlers)): co código do M3, cinco delas fallan, e as outras dúas, `saving_no_title_gives_back_no_title` e `osc_1_and_21_do_not_change_the_title`, describen o que xa pasaba. E desde a 4.1 ([ADR 0032](../adr/0032-o-escaner-osc.md)), cinco comproban o win32-input-mode, que desde entón segue o [`Hooked`](#os-métodos-de-handler): cos casos das dez probas do escáner anterior, agora a través de `feed`, e co que trae o cambio, o reinicio na orde da saída e a actualización sincronizada. Outras tres comproban o que di a shell.

| Proba | Que comproba |
|---|---|
| `the_output_reaches_the_screen` | O texto e o salto de liña chegan ás celas, e o cursor queda despois do texto: unha barra que pestanexa, porque o programa non escolleu outro cursor. A copia leva as cores de meiga (`Palette::MEIGA`). |
| `only_what_changed_is_copied_again` | Sen saída, unha copia non marca nada; despois de escribir noutra fila, só cambian o texto novo e as celas do cursor, onde estaba e onde está. |
| `the_program_gets_its_answers` | As respostas a DA1, DSR 5, DSR 6, `ESC [ 18 t` e DA2, que leva a versión de `alacritty_terminal` (`2600`); e a `ESC [ 14 t`, ningunha mentres o terminal non sabe o tamaño da cela e, cunha cela de 9 × 20 píxeles nunha grella de 10 × 3, `ESC [ 4 ; 60 ; 90 t`. |
| `questions_about_colours_are_answered_with_the_palette` | As preguntas pola cor 1 da paleta (OSC 4) e polas do texto, o fondo e o cursor (10, 11 e 12) respóndense coas cores de meiga, cada unha co final da súa pregunta, BEL ou `ESC \`; o cursor ten a cor do texto. Despois de cambiar o programa o fondo, a resposta ten a cor nova, e vai entre as dúas respostas a DA1, na orde das preguntas. E cun tema que cambia a cor 1, a resposta é a do tema. |
| `a_change_of_the_palette_changes_every_cell` | Cambiar a cor 1 da paleta (OSC 4) cambia todas as celas, aínda que seguen co mesmo número de cor; voltar á do tema (OSC 104), tamén; cambiar a do cursor (OSC 12) só cambia as dúas celas do cursor; e despois, unha copia sen cambios non marca nada. |
| `the_title_and_the_bell` | O título de OSC 0 e 2, sen os espazos dos extremos; gardalo e recuperalo (`ESC [ 22 t` e `ESC [ 23 t`); un título baleiro é ningún; e a campá. |
| `the_same_title_again_is_not_a_change` | Desde a subfase 3.1. Nun terminal novo, `OSC 0 ;` e un título só de espazos, por OSC 2, non cambian nada: xa non había título. `pwsh` é un cambio; `pwsh` outra vez, por OSC 2, e `pwsh` cun espazo en cada extremo, por OSC 0, que `vte` dá sen eles, non. E `vim` e outra vez `pwsh`, na mesma lectura, son un cambio, aínda que o título remate igual. |
| `a_reset_keeps_the_title_and_the_saved_ones` | Desde a 3.1. Con `pwsh` gardado e `vim` de título, `ESC c` non cambia nada, e o título segue sendo `vim`. Gardado despois do reinicio, e con `less` de título, un `CSI 23 t` dá outra vez `vim`; outro, `pwsh`, o gardado antes do reinicio; e un terceiro, xa sen gardados, non cambia nada. E na pantalla alternativa, con `top` de título e a principal á parte por un cambio de tamaño: gardar o título, `ESC c` e `full`, dentro dunha actualización sincronizada, non cambian nada ata que a remata `end_sync`; despois, o título é `full`, xa na pantalla principal, e `CSI 23 t` dá outra vez `top`. |
| `a_restart_forgets_the_title_and_the_saved_ones` | Desde a 3.1. Con `pwsh` e `vim` gardados, e `late` de título nunha actualización sincronizada que o programa deixou aberta, `restart` deixa o terminal sen título: tampouco queda `late`, que mostra primeiro. Sen nada gardado, `CSI 23 t` non cambia nada; e `vim`, o título do programa novo, é un cambio, aínda que o dese o anterior. |
| `a_title_longer_than_1000_characters_is_ignored` | Desde a 3.1. Con `pwsh` de título, un de 1001 `x` non cambia nada, e `pwsh` queda. Ignorado, tampouco se garda: despois del, `CSI 22 t` garda `pwsh`, o título que quedou, e, despois de `vim`, `CSI 23 t` dá outra vez `pwsh`. 500 emoji, 1000 unidades de UTF-16, con dous espazos en cada extremo, que `vte` quita antes de que meiga as conte, caben, tamén partidos entre dúas lecturas no medio dun emoji, coma poden chegar do ConPTY: a primeira lectura non cambia nada, e a segunda pon o título. Cun `x` máis, 1001 unidades, ignórase. |
| `the_program_saves_at_most_10_titles` | Desde a 3.1. O programa garda once títulos, do `0` ao `10`, e pon `last`: dez `CSI 23 t` dan do `10` ao `1`, cada un cun cambio, e o seguinte non cambia nada: o `0`, o máis vello, xa non estaba. |
| `saving_no_title_gives_back_no_title` | Desde a 3.1. Nun terminal novo, `CSI 22 t` garda «sen título», e `vim` é un cambio; `CSI 23 t` deixa outra vez «sen título», tamén cun cambio: a tab mostra outra vez o nome do seu perfil. Co código do M3, xa pasaba. |
| `osc_1_and_21_do_not_change_the_title` | Desde a 3.1. Con `pwsh` de título, OSC 1, o título da icona, e OSC 21, o de DECSWT, non cambian nada: `vte` non os pasa ([ADR 0029](../adr/0029-o-titulo-da-tab-e-os-titulos-gardados.md), decisión 7). Co código do M3, tamén pasaba. |
| `osc_52_only_copies` | O texto en base64 de OSC 52 chega ao portapapeis; ler o portapapeis, a selección primaria e un texto que non é base64 non fan nada nin responden. |
| `the_modes_follow_the_program` | Os modos por defecto; activar e desactivar cada un; os modos do rato, e os seus formatos, exclúense entre si; e un reinicio (RIS) déixaos como ao principio. |
| `win32_input_mode_follows_its_sequences_also_split` | Desde a subfase 4.1. O que escribe o ConPTY ao arrancar, `CSI 1 t`, DA1, `?1004h`, `?9001h` e `?25l`, activa o win32-input-mode, partido en dúas lecturas en todas as posicións posíbeis, e byte a byte. Entre outros modos, `?1004;9001h` actívao, e `?9001;1004l` desactívao; `?1004h` só, non. |
| `only_a_private_set_or_reset_changes_win32_input_mode` | Desde a 4.1. Sen `?`, con outra marca (`>`), con outro byte intermedio (`$`), con dous `?` ou con outro byte final (`p`), nada. Só o primeiro número de cada parámetro é un modo: `?9001:1h` actívao, e `?1:9001h`, non. E con 32 números, o último 9001, actívase; con 33, `vte` ignora a secuencia. |
| `controls_cancel_or_restart_the_sequence_of_win32_input_mode` | Desde a 4.1. CAN no medio cancela a secuencia; un ESC empeza outra, que o activa; e un salto de liña no medio execútase, e a secuencia segue ata activalo. Un OSC remata nun ESC, que empeza a secuencia; e `ESC`, 0x80 e `[ ? 9001 h` activan o modo, porque `vte` ignora o 0x80 despois do ESC. |
| `a_reset_turns_win32_input_mode_off_in_the_order_of_the_output` | Desde a 4.1. `ESC c` desactiva o modo, na lectura seguinte ou na mesma; e `ESC c`, `?1004h` e `?9001h`, coma o escribe o ConPTY despois dun reinicio, déixao activo. `ESC ( c`, que escolle un xogo de caracteres, non é un reinicio; `ESC`, `á` e `c` si, porque `vte` ignora os bytes de `á` despois do ESC. |
| `in_a_synchronized_update_win32_input_mode_changes_when_the_update_is_shown` | Desde a 4.1. Co modo activo, `?9001l` dentro dunha actualización sincronizada non o desactiva ata que `end_sync` a mostra; e `?9001h` noutra non o activa ata que o programa a remata, con `?2026l`. |
| `the_shell_says_its_folder_its_marks_and_the_exit_code` | Desde a 4.1. Nun terminal novo, nin cartafol, nin marca, nin código. OSC 9;9 entre comiñas é un cambio, co cartafol de Windows; o mesmo sen as comiñas, non. OSC 7 é outro cartafol, co equipo e a ruta. A, o *prompt* e B, nunha lectura, son un cambio, e a marca é B; a orde, C e a súa saída, outro, coa marca C; e D co código 3, outro, coa marca e o código. O A seguinte é un cambio, e o código segue sendo 3; o mesmo A outra vez, non. Un D sen código deixa a orde sen el. E os OSC non deixan nada na pantalla: `PS> dir`, `out` e unha fila baleira. |
| `what_the_shell_says_counts_at_once_also_in_a_synchronized_update` | Desde a 4.1. Dentro dunha actualización sincronizada, A e OSC 9;9 son un cambio no mesmo `feed`, e a marca xa é A; `end_sync` xa non trae ningún. |
| `a_restart_forgets_what_the_shell_said_and_a_reset_does_not` | Desde a 4.1. Despois de OSC 9;9 e D co código 1, `ESC c` non cambia nada, e o cartafol e o código quedan. Con outro OSC 9;9 a medias, `restart` deixa o terminal sen cartafol, sen marca e sen código: tampouco queda o cartafol a medias, que remata o primeiro ESC de `RESTART`. E o cartafol da shell nova é un cambio, aínda que a vella dese o mesmo. |
| `colours_and_styles_reach_the_cells` | As cores de 16, de 256 e *truecolor*, do texto e do fondo, e todos os estilos, tamén os cinco subliñados. |
| `a_wide_character_takes_two_cells` | Un carácter ancho ocupa as súas dúas metades; se non cabe ao final da fila, vai á seguinte. |
| `a_tab_leaves_blank_cells` | O tabulador deixa celas en branco. |
| `the_cursor_follows_the_program` | Ao principio, unha barra que pestanexa; as formas do cursor (DECSCUSR), fixas e que pestanexan; o pestanexo (`?12h` e `?12l`); voltar ao cursor de meiga (`ESC [ 0 SP q`); ocultalo e mostralo; e sobre a metade dereita dun carácter ancho, debúxase desde a esquerda. |
| `the_cursor_on_a_lone_right_half_stays_in_its_cell` | Despois de borrar a metade esquerda dun carácter ancho (`ESC [ P`), a metade dereita que queda é unha cela en branco, e o cursor debúxase nela, tanto na primeira columna coma despois dun carácter estreito. Antes da revisión do código, na primeira columna era un *panic*. |
| `the_cells_of_the_cursor_are_changes_when_it_is_drawn` | As celas do cursor márcanse só se cambia e se se debuxa: ao ocultalo, só onde estaba; mentres está oculto, nada; e na última columna, unha soa cela. |
| `a_synchronized_update_waits_for_its_end` | A saída dunha actualización sincronizada non chega á pantalla ata que remata ou vence (como moito, 150 ms); `end_sync` móstraa e entrega as súas respostas. |
| `after_a_resize_everything_is_copied_again` | Despois de cambiar o tamaño, a copia ten o tamaño novo e todas as celas cambiadas, co texto no seu sitio. |
| `a_forgotten_screen_takes_all_of_another_terminal` | Desde a subfase 2.2. Unha pantalla copiada dun terminal con `uno` e `dous` recibe a copia doutro, con `tres`, que xa se copiara noutra pantalla e non cambiou desde entón, coma unha tab que xa se mostrou: sen `forget`, segue co `dous` do primeiro; despois de `forget`, ten todas as celas cambiadas, o mesmo tamaño, `tres` na primeira fila, a segunda baleira e o cursor despois de `tres`. |
| `the_grid_is_never_smaller_than_two_columns_and_one_row` | Un terminal de 0 × 0 ou de 1 × 0 mide 2 × 1, e escribir caracteres anchos e inserir celas e liñas nel non falla. |
| `a_taller_screen_adds_blank_rows_and_leaves_the_history_alone` | Desde a subfase 1.9: un terminal de 10 × 3 con `a` a `e`, dúas no historial, pasa a ter 5 filas: `c`, `d` e `e` seguen arriba, con dúas filas baleiras abaixo, o cursor segue despois do `e`, e o historial segue con dúas filas. `alacritty_terminal` traería del `a` e `b`. |
| `a_shorter_screen_keeps_the_cursor_and_the_rows_above_it` | De 5 filas a 3, se o texto cabe, só se van as filas baleiras de abaixo: `a` e `b` seguen arriba, co cursor, e nada vai ao historial. Se non cabe, de `a`, `b` e `c` a 2 filas, a primeira vai ao historial; e alí segue cando a pantalla volta a 5 filas, con `b` e `c` arriba e as baleiras abaixo. |
| `narrower_lines_wrap_downwards_and_join_again_when_wider` | `abcdef` e `xy`, de 8 columnas a 4: `abcdef` pártese en dúas filas, que baixan cara á fila baleira, e nada vai ao historial; o cursor segue despois de `xy`. De volta a 8, as dúas filas xúntanse, a fila baleira volta abaixo, e un `z` escríbese despois de `xy`. |
| `the_cursor_keeps_its_row_when_many_rows_below_it_wrap` | En 4 × 4, con `x`, `a`, `bbbb` e `cccc` e o cursor no `a`: a 2 columnas, as dúas últimas liñas ocupan catro filas e, coma no ConPTY, a pantalla empeza na fila do cursor, e a última, que xa non cabe, vaise. O cursor segue no `a`, e un `A` escríbese enriba del. |
| `the_cursor_keeps_its_place_after_the_last_character_of_a_full_row` | Co cursor esperando despois de `abcd`, nunha fila chea de 4 columnas: a 6 columnas, un `e` vai despois do `d`, na mesma fila; a 2 columnas, `abcd` ocupa dúas filas, e o `e` vai á seguinte. |
| `on_the_alternate_screen_the_primary_one_gets_the_last_size_when_the_program_leaves` | Na pantalla alternativa, a 10 × 5 e a 4 × 2, `ALT` segue arriba, sen repartir. Se o programa volta á principal co tamaño que tiña, 10 × 3, esta non cambia, sen rastro dos tamaños do medio: `c`, `d` e `e`, o cursor despois do `e` e dúas filas no historial. E se volta despois de pasar a 10 × 5, recibe ese tamaño coma na pantalla principal: dúas filas baleiras abaixo, e o historial onde estaba. |
| `the_output_after_leaving_the_alternate_screen_reaches_the_primary_one` | Despois de pasar a 10 × 4 na pantalla alternativa, a `X` que segue a `?1049l` chega á principal, xa co tamaño novo, despois do `b`: na mesma lectura, dentro dunha actualización sincronizada (`?2026`) e nunha que remata `end_sync`, á súa hora. |
| `a_reset_on_the_alternate_screen_leaves_an_empty_primary_screen_with_history` | Un reinicio (`ESC c`) na pantalla alternativa, despois dun cambio de tamaño, volta á principal, baleira e sen historial; e esta garda historial: con seis liñas en catro filas, dúas van a el. |
| `a_restart_moves_the_screen_to_the_history_and_leaves_a_new_terminal` | Desde a subfase 2.7. Nun terminal de 20 × 4, un programa escribe do `1` ao `6`, dous deles no historial, e cambia o que espera un programa novo: o cursor na fila 3 e na columna 5, vermello e en negra, gardado alí; sen tabulacións; o xogo das liñas en G1, e en uso; unha rexión da segunda á terceira fila, co modo de orixe; sen *wrap*, inserindo e co *LNM*; un cursor de subliñado fixo, e oculto; outra cor 1 da paleta, e outras cores do fondo e do cursor; e un título. Coa vista unha fila arriba, `restart` mete as catro filas da pantalla no historial, que ten 6, leva a vista á pantalla, que queda baleira, e deixa o cursor, os modos e a paleta dun terminal novo, e ningún título; no principio do historial están do `1` ao `4`. Despois, o terminal compórtase coma un novo: `q`, un tabulador e `x` levan o `x` á columna 9, a da seguinte tabulación, e o `q` vai en ASCII e coas cores por defecto; un `y` na primeira columna non empuxa o `x`, porque xa non insire, e `last` vai á cuarta fila, sen rexión nin modo de orixe; un LF na terceira columna da segunda fila deixa o cursor na terceira columna da seguinte, sen *LNM*; 23 letras desde o principio da segunda fila pasan á terceira; e `ESC 8` leva o cursor á esquina de arriba á esquerda, onde un `z` vai coas cores por defecto. E na pantalla alternativa, cos modos do programa para as teclas e o rato, sen as frechas da roda, con outra cor do texto e cunha actualización sincronizada aberta cunha pregunta (DA1): `restart` remata a actualización sen ningunha resposta, volta á pantalla principal, cos modos e a paleta dun terminal novo, e mete no historial a súa única fila con texto, `un`; `ALT`, na alternativa, xa non se ve. |
| `the_cursor_is_at_the_start_of_an_empty_row_also_with_the_view_up` | Desde a 2.7. Nun terminal novo, o cursor está ao principio dunha fila baleira, e non hai fila enriba. Con `ten_lines`, o cursor está despois do `10`, e non ao principio da súa fila. Coa vista no principio do historial, `cursor` dá a columna 0, a dun cursor que non se ve, pero `at_empty_row_start` segue dicindo que non. Despois dun CR, tampouco: a fila ten o `10`; e despois dun LF, si. Desde a revisión do código da 2.7, unha fila cun espazo de fondo azul non está baleira, nin tampouco a que continúa a de enriba: dez letras enchen unha fila, e un espazo pasa á seguinte, onde un CR deixa o cursor. Despois dun CR e un LF, si. |
| `the_view_moves_through_the_history` | Con `ten_lines`, a vista móvese dúas filas cara arriba, unha pantalla, ao principio do historial, unha pantalla cara abaixo, a unha fila por riba da pantalla e, con `scroll_to(100)`, ao principio outra vez, e volta á pantalla; en cada paso vense as filas que tocan. Mentres non se ve a fila do cursor, o cursor non se debuxa (`Hidden`); e nos extremos, `scroll` di que non se moveu. |
| `the_output_does_not_move_the_view_of_the_history` | Coa vista dúas filas arriba, en `6`, `7` e `8`, dúas liñas novas non a moven: segue mostrando as mesmas filas, agora 4 por riba da pantalla, nun historial de 9. |
| `the_rows_that_leave_a_region_below_the_top_do_not_move_the_view` | Desde a revisión do código da subfase 1.9. Con `ten_lines` e a vista dúas filas arriba, en `6`, `7` e `8`, seis saídas non meten nada no historial nin moven a vista: dez liñas novas ao final dunha rexión que empeza na segunda fila (`ESC [ 2 ; 3 r`); tres `CSI S`; caracteres que pasan á fila seguinte ao final da rexión; un tabulador despois da última columna; unha rexión que non vale (`ESC [ 1 ; 1 r`), que deixa a de antes; e, sen rexión, dous `CSI M` co cursor na segunda fila. Desde o principio do historial, a vista tampouco pasa del. E unha rexión que empeza arriba (`ESC [ 1 ; 2 r`) mete as súas filas no historial, e a vista sobe con elas. |
| `a_new_size_a_reset_or_deccolm_make_the_region_the_whole_screen_again` | Desde a revisión do código. Despois dunha rexión que empeza na segunda fila, a pantalla enteira volta ser a rexión, e dúas liñas novas ao final da pantalla meten dúas filas no historial, coa vista nas mesmas filas: despois dun cambio de tamaño; dunha rexión da pantalla alternativa, cando o programa volta á principal, que recibe outro tamaño; dun reinicio (`ESC c`); e de DECCOLM, `?3h` e `?3l`, que `alacritty_terminal` só segue para borrar a pantalla e a rexión. |
| `a_resize_keeps_the_view_inside_the_history` | Coa vista no principio do historial, despois de pasar a 10 × 8, a 2 × 2 e a 10 × 3, a vista nunca pasa do historial, e copiar o terminal non falla. E coa vista abaixo, un cambio de tamaño déixaa abaixo. |
| `a_selection_of_cells_marks_them_and_gives_their_text` | Un clic que non se move non selecciona nada, e a copia non cambia nada. Arrastrando desde o lado esquerdo da segunda cela ata o dereito da sexta, o texto é `la mu`, e só esas celas quedan marcadas e cambiadas; ata o lado esquerdo da terceira cela da fila seguinte, `la mundo`, un LF e `ad`, e só cambian as celas que entran; e cara atrás, ata o lado esquerdo da primeira, `o`. `clear_selection` di se había selección, e despois só cambia a cela que estaba seleccionada, co texto no seu sitio. |
| `a_double_click_selects_a_word_a_path_or_up_to_the_other_bracket` | Un dobre clic selecciona unha palabra (`cat`, e `ok` e `si`, dentro dunhas parénteses e dunhas comiñas); unha ruta enteira, coas súas barras invertidas e os seus `:` (`C:\x\f.rs:12:5`); nunha paréntese, ata a outra, desde as dúas; nun espazo ou nunha coma, só iso; despois do final do texto, ningún texto; e en calquera das metades dun carácter ancho, a palabra (`中文`). |
| `a_triple_click_selects_the_rows_of_a_line` | Un triplo clic nunha liña que non colle nunha fila marca as súas dúas filas, e o seu texto vai sen LF ao final; arrastrar ata a liña seguinte engádea, despois dun LF. |
| `a_wide_character_is_selected_whole` | Unha selección que empeza na metade dereita dun carácter ancho, ou que remata na esquerda, marca o carácter enteiro, e o texto lévao unha vez. |
| `the_selection_moves_with_the_text_and_goes_away_with_it` | Cando a pantalla se despraza, a selección sobe co seu texto. Cando o programa borra unha das súas filas, desaparece, tamén da outra fila, que non cambiou. Escribir enriba dela non a quita, pero borrar a pantalla si. |
| `copying_only_the_damage_gives_the_same_screen_as_copying_everything` | **A invariante da copia.** 300 casos aleatorios, de ata 79 anacos dunha lista de 70: texto, caracteres anchos e combinados, movementos do cursor, borrados, inserir e borrar caracteres e liñas, desprazamentos e rexións, a pantalla alternativa, estilos, reinicios… Cada caso lese en anacos de ata 16 bytes, por veces cambiando o tamaño, con grellas de 2 × 1 a 80 × 24. Despois de cada anaco, a pantalla copiada só co dano ten que ser igual, cela a cela e no cursor, á dun segundo terminal copiado enteiro. Esta proba atopou o oco do dano de `alacritty_terminal` cos caracteres anchos. Desde a subfase 1.8, antes de cada anaco cambia tamén a selección, igual nos dous terminais: un de cada oito empeza unha, de calquera clase; dous de cada oito móvena; e un quítaa, ás veces con celas fóra da grella. Así comproba tamén que a copia marca as celas onde cambia a selección. Desde a 1.9, outro de cada oito move a vista do historial, igual nos dous terminais, con calquera `Scroll`: de dúas filas cara abaixo a dúas cara arriba, unha pantalla ou ata un extremo. |

## `crates/meiga-core/src/ui/mod.rs`

Desde a subfase 2.3: as partes da interface que debuxa o propio meiga, coma disposicións sen Windows: onde vai cada parte, á escala da xanela, e que parte hai debaixo dun punto ([ADR 0020](../adr/0020-a-barra-de-tabs.md) e [ADR 0021](../adr/0021-os-menus-e-o-menu-da-tab.md)). Debuxalas e seguir o rato é traballo do programa, con `meiga-render`; eiquí só se calculan rectángulos, así que se proban con coordenadas exactas, sen xanela nin fontes (ADR 0020, decisión 2). Hai dúas: a barra de tabs e, desde a 2.4, os menús, que escalan as súas medidas coas mesmas funcións. Desde a 2.9, a barra de tabs é a barra de título da xanela, e `titlebar` di que é para Windows cada punto dela, coa súa disposición ([ADR 0027](../adr/0027-a-barra-de-tabs-coma-barra-de-titulo.md)): non ten medidas propias nin debuxa nada. Desde a 3.2, ten tamén o campo de texto, `field`, que non é unha disposición: edita unha liña de texto, coas súas teclas e o seu rato en celas, e di que parte do texto se ve nas súas celas ([ADR 0030](../adr/0030-o-campo-de-texto.md), [`ui/field/mod.rs`](#cratesmeiga-coresrcuifieldmodrs)).

```rust
//! The parts of the interface that meiga draws itself, without Windows: the
//! layouts, which say where each part goes, at the scale of the window, and
//! what is under a point; and the text field, which edits a line of text.

pub mod field;
pub mod menu;
pub mod tabbar;
pub mod titlebar;
```

- O comentario do módulo (`//!`) di para que é, coma o parágrafo de enriba. Desde a 3.2 fala tamén do campo de texto, que non é unha disposición.
- `pub mod field;`, desde a 3.2, `pub mod menu;`, `pub mod tabbar;` e, desde a 2.9, `pub mod titlebar;` declaran os submódulos, que están en `src/ui/field/mod.rs`, nun cartafol porque ten os seus propios submódulos, en `src/ui/menu.rs`, desde a 2.4, en `src/ui/tabbar.rs` e en `src/ui/titlebar.rs`, e fanos públicos: fóra do crate, os seus tipos úsanse co seu camiño, coma `meiga_core::ui::tabbar::Layout`, `meiga_core::ui::titlebar::TitleBar` ou `meiga_core::ui::field::Field`. Ao contrario ca `model`, `ui` non reexporta nada con `pub use`: as dúas disposicións chámanse `Layout`, e lense co nome do seu módulo diante, `tabbar::Layout` e `menu::Layout`. `titlebar` usa a da barra, con `use super::tabbar::…`.

### Os axudantes: `BASE_DPI`, `scale` e `coord`

Desde a subfase 2.4 están eiquí, e non en `tabbar.rs`, porque os usan a barra e os menús: os dous submódulos tráenos con `use super::{coord, scale};`. `titlebar`, desde a 2.9, non os usa: as medidas que necesita xa veñen escaladas na disposición da barra. `field`, desde a 3.2, tampouco: conta en celas, e non en píxeles. Son privados, pero os submódulos de `ui` venos, porque en Rust un módulo ve o que é privado nos seus antepasados; fóra de `ui`, non se ven.

```rust
const BASE_DPI: u32 = 96;

fn scale(pixels: u32, dpi: u32) -> u32 {
    (pixels.saturating_mul(dpi).saturating_add(BASE_DPI / 2) / BASE_DPI).max(1)
}

fn coord(value: u64) -> i32 {
    i32::try_from(value).unwrap_or(i32::MAX)
}
```

- **`BASE_DPI`**: os puntos por polgada da escala 100 %, 96, na que están en píxeles as medidas das partes.
- **`scale`**: `pixels` a escala 100 %, a `dpi`: multiplicado por `dpi` e dividido entre 96, redondeado ao máis próximo, e polo menos 1 ([ADR 0020](../adr/0020-a-barra-de-tabs.md), decisión 2, e [ADR 0021](../adr/0021-os-menus-e-o-menu-da-tab.md), decisión 3). Sumar a metade de 96 antes de dividir redondea, coma en [`Rgb::mix`](#rgb): a división de enteiros só corta os decimais, e unha metade sobe. Con `max(1)`, ningunha medida desaparece, nin sequera a un DPI moi baixo; e `saturating_mul` e `saturating_add` quedan no máximo dun `u32` en vez de desbordar. A 120 DPI (125 %), os 32 do alto da fila das tabs son 40, e o trazo, 1,25, queda en 1; a 144 DPI (150 %), o trazo, 1,5, sobe a 2, e os 11 píxeles entre o resalte e o texto dun menú, 16,5, a 17.
- **`coord`**: un número de píxeles coma unha coordenada, un `i32`, ou `i32::MAX` se non cabe. Ningunha parte da xanela chega tan lonxe, pero así as contas non desbordan: as da barra, con calquera índice de tab, e as dos menús, con calquera número de entradas. Desde a 2.8, as posicións das tabs, que mentres a franxa as despraza poden ser negativas, convérteas [`clamped`](#os-axudantes-scale-coord-signed-clamped-centered-e-centered_size), en `tabbar.rs`, que queda tamén no mínimo dun `i32`.

## `crates/meiga-core/src/ui/tabbar.rs`

Desde a subfase 2.3: a disposición da barra de tabs ([ADR 0020](../adr/0020-a-barra-de-tabs.md), [Arquitectura › A barra de tabs](../arquitectura.md#a-barra-de-tabs)). Para un número de tabs, un ancho e un DPI, di onde van as tabs, os seus títulos, os seus botóns de pechar (×) e os separadores entre elas, o botón que abre unha tab (+) e, desde a 2.5, xunto a el, o que abre o menú dos perfís (▾) ([ADR 0022](../adr/0022-os-perfis-e-a-configuracion.md), decisión 6); e que parte da barra hai debaixo dun punto. Desde a 2.8, as tabs teñen un ancho mínimo e, cando nin así caben, van nunha franxa entre dous botóns de desprazamento (‹ e ›): a disposición garda canto está desprazada a franxa, e desprázaa unha tab, uns píxeles ou o mínimo para que se vexa enteira unha tab ([ADR 0025](../adr/0025-moitas-tabs-e-os-indicadores.md), [Arquitectura › Moitas tabs e os indicadores](../arquitectura.md#moitas-tabs-e-os-indicadores)). Desde a 2.9, a barra é a barra de título da xanela ([ADR 0027](../adr/0027-a-barra-de-tabs-coma-barra-de-titulo.md), [Arquitectura › As tabs na barra de título](../arquitectura.md#as-tabs-na-barra-de-título)): a disposición ten tamén o alto da banda que vai enriba das tabs coa xanela restaurada, e di onde van os botóns da xanela, minimizar, maximizar ou restaurar e pechar, e os seus glifos, cun sitio para arrastrar a xanela sempre antes deles; o que é cada parte da barra para Windows dío [`ui/titlebar.rs`](#cratesmeiga-coresrcuititlebarrs), con esta disposición. A barra do programa debúxaa con estes rectángulos, segue con eles o rato e marca as zonas que cambian ([Código › meiga › `tabbar.rs`](meiga.md#cratesmeigasrctabbarrs)).

**A forma da barra**, coas medidas do Windows Terminal en píxeles a escala 100 % ([ADR 0020](../adr/0020-a-barra-de-tabs.md), decisións 1, 2 e 3; [ADR 0025](../adr/0025-moitas-tabs-e-os-indicadores.md), decisións 1 e 2; [ADR 0027](../adr/0027-a-barra-de-tabs-coma-barra-de-titulo.md), decisións 2, 3 e 5; [Tecnoloxías › As barras de tabs](../tecnoloxias.md#55-as-barras-de-tabs-chrome-e-o-windows-terminal)):

- unha fila de tabs de 32 píxeles de alto, de todo o ancho da xanela, enriba do terminal e sempre visíbel, tamén cunha soa tab. Desde a 2.9, a barra vai no sitio da barra de título de Windows, e coa xanela restaurada leva enriba da fila unha banda do grosor do marco da xanela, onde Windows a redimensiona: a 96 DPI, de 8 píxeles, e a barra mide 40. Maximizada, non hai banda, e a barra é a fila, coma ata a 2.8, cando ía debaixo da barra de título de Windows;
- as tabs, unha tras outra desde a esquerda, todas do mesmo ancho, ata 240. Se non caben, encóllense todas por igual, ata 100, coma no Windows Terminal; ata a 2.7, sen mínimo;
- o +, un cadrado de 28, xusto despois da última tab e, desde a 2.5, unido a el, o ▾, de 24 de ancho e tan alto coma el, coma o botón dividido do Windows Terminal;
- desde a 2.8, se nin de 100 caben, a barra desprázaas: as tabs van nunha franxa entre dous botóns de desprazamento, o ‹ á esquerda e o › á dereita, de 24 de ancho e tan altos coma o +, que só aparecen entón; a franxa mostra a parte da fila das tabs que di o seu desprazamento, e corta as dos seus lados; e o + e o ▾ van despois do ›, sempre no mesmo sitio: na 2.8, ao final da barra, e desde a 2.9, antes do sitio para arrastrar a xanela;
- desde a 2.9, á dereita da barra, os tres botóns da xanela, minimizar, maximizar ou restaurar e pechar, de 46 de ancho, coma no Windows 11 e no Windows Terminal, e do alto da barra enteira, tamén por riba da banda; no medio de cada un, o seu glifo, nun cadrado de 10, co trazo do × e do +;
- desde a 2.9, entre a separación da dereita do ▾ e os botóns da xanela, un sitio para arrastrar a xanela de polo menos 45, coma no Windows Terminal: mentres as tabs caben, o que non usan queda tamén alí, e mentres non, a franxa deixa xusto ese sitio;
- 4 píxeles de separación por riba das tabs, debaixo da banda, á esquerda da primeira, entre a última e o +, e á dereita do ▾ (ata a 2.4, do +); mentres as tabs non caben, á esquerda do ‹ e entre o › e o +, no canto de á esquerda da primeira tab e despois da última;
- cada tab debúxase desde esa separación de enriba ata o bordo de abaixo da barra, para que a activa, coa cor do fondo do terminal, se vexa unida a el; pero, para o rato, ocupa todo o alto da fila das tabs: desde a 2.9, a banda non é da tab, nin dos botóns da barra;
- o ×, un cadrado de 24 á dereita da tab, a 4 píxeles do seu bordo dereito e centrado no seu alto, se cabe, e desde a 2.8 cabe sempre; e o título, desde 8 píxeles do bordo esquerdo da tab ata o ×;
- e todas as medidas, escaladas co DPI da xanela, redondeadas e de polo menos un píxel; desde a 2.9, menos a banda, que chega xa en píxeles á escala da xanela, coma a mide Windows.

**Un exemplo**, o das probas: tres tabs nunha xanela de 983 píxeles a 96 DPI, a escala 100 %, maximizada: sen banda, a barra é a fila das tabs. Os 983 son os 800 deste exemplo ata a 2.8 e os 183 do sitio para arrastrar a xanela e dos seus botóns, así que as tabs e os botóns da barra quedan onde estaban. As tabs caben enteiras, de 240 píxeles:

| Parte | Método | Rectángulo |
|---|---|---|
| A barra | `bar()` | (0, 0, 983, 32) |
| A primeira tab, como se debuxa | `tab(0)` | (4, 4, 244, 32) |
| A terceira | `tab(2)` | (484, 4, 724, 32) |
| O título da primeira | `title(0)` | (12, 4, 216, 32) |
| O seu × | `close(0)` | (216, 6, 240, 30) |
| O que resalta nel o punteiro | `close_highlight(0)` | (218, 8, 238, 28) |
| A súa cruz | `cross(0)` | (224, 14, 232, 22) |
| O separador entre a primeira e a segunda | `separator(1)` | (244, 10, 245, 26) |
| O que cambia co aspecto da primeira | `tab_area(0)` | (4, 0, 245, 32) |
| O + | `new_tab()` | (728, 4, 756, 32) |
| O que resalta nel o punteiro | `new_highlight()` | (730, 6, 754, 30) |
| O seu signo + | `plus()` | (736, 12, 747, 23) |
| O ▾ | `profiles()` | (756, 4, 780, 32) |
| O que resalta nel o punteiro | `profiles_highlight()` | (758, 6, 778, 30) |
| A súa frecha | `chevron()` | (764, 16, 772, 20) |
| O botón de minimizar a xanela | `caption(CaptionButton::Minimize)` | (845, 0, 891, 32) |
| O seu glifo | `glyph(CaptionButton::Minimize)` | (863, 11, 873, 21) |
| O de maximizar ou restaurar | `caption(CaptionButton::Maximize)` | (891, 0, 937, 32) |
| O de pechar | `caption(CaptionButton::Close)` | (937, 0, 983, 32) |
| O seu glifo | `glyph(CaptionButton::Close)` | (955, 11, 965, 21) |
| Os tres botóns da xanela | `captions()` | (845, 0, 983, 32) |

Os rectángulos van coma en [`Rect`](#cratesmeiga-coresrcgeometryrs), (`left`, `top`, `right`, `bottom`), e os dous últimos non se inclúen: a primeira tab remata no píxel 243, e a segunda empeza no 244. Á dereita do ▾, de 780 a 845, a barra está baleira: a separación de 4, e 61 píxeles para arrastrar a xanela, os 45 do mínimo e os 16 que as tabs, xa co seu ancho máximo, non usan. Ata a 2.8, de 780 a 800, ao final da barra.

**Restaurada**, desde a 2.9, coa banda de 8 píxeles que ten a xanela a 96 DPI, a mesma barra mide 40 de alto: todo o que vai debaixo da banda baixa 8, e os botóns da xanela chegan ata arriba:

| Parte | Método | Rectángulo |
|---|---|---|
| A barra | `bar()` | (0, 0, 983, 40) |
| A primeira tab | `tab(0)` | (4, 12, 244, 40) |
| O seu × | `close(0)` | (216, 14, 240, 38) |
| O separador entre a primeira e a segunda | `separator(1)` | (244, 18, 245, 34) |
| O que cambia co aspecto da primeira | `tab_area(0)` | (4, 8, 245, 40) |
| O + | `new_tab()` | (728, 12, 756, 40) |
| O ▾ | `profiles()` | (756, 12, 780, 40) |
| O botón de pechar a xanela | `caption(CaptionButton::Close)` | (937, 0, 983, 40) |
| O seu glifo | `glyph(CaptionButton::Close)` | (955, 15, 965, 25) |

O que cambia co aspecto dunha tab empeza debaixo da banda, que non cambia con ela; e o glifo vai no medio do botón, que agora mide 40 de alto.

**Un exemplo con moitas tabs**, desde a 2.8, tamén o das probas: na mesma xanela maximizada, as tabs teñen un sitio de 736 píxeles, 983 − 3 × 4 − 28 − 24 − 45 − 3 × 46 ([`room`](#a-barra-e-o-ancho-das-tabs-bar-room-viewport-e-fit)); ata a 2.8, nunha xanela de 800, 800 − 3 × 4 − 28 − 24. Sete caben, de 105. Oito, de 92, xa non: miden 100, e van na franxa, de 736 − 2 × 24 = 688 píxeles ([`viewport`](#a-barra-e-o-ancho-das-tabs-bar-room-viewport-e-fit)), onde caben seis tabs enteiras e 88 píxeles da sétima. A fila das oito mide 800, así que a franxa se despraza ata 112, 800 − 688; con trinta, a fila mide 3000, e a franxa desprázase ata 2312 ([`max_scroll`](#canto-se-despraza-max_scroll-e-can_scroll)). Oito tabs, coa franxa sen desprazar:

| Parte | Método | Rectángulo |
|---|---|---|
| O ‹ | `scroll_button(Side::Left)` | (4, 4, 28, 32) |
| O que resalta nel o punteiro | `scroll_highlight(Side::Left)` | (6, 6, 26, 30) |
| A súa frecha | `scroll_chevron(Side::Left)` | (14, 14, 18, 22) |
| A franxa | `strip()` | (28, 0, 716, 32) |
| A primeira tab | `tab(0)` | (28, 4, 128, 32) |
| A oitava, fóra da franxa | `tab(7)` | (728, 4, 828, 32) |
| O que cambia co aspecto da sétima, que a franxa corta | `tab_area(6)` | (628, 0, 716, 32) |
| O que cambia co aspecto da oitava | `tab_area(7)` | baleiro |
| O › | `scroll_button(Side::Right)` | (716, 4, 740, 32) |
| A súa frecha | `scroll_chevron(Side::Right)` | (726, 14, 730, 22) |
| O + | `new_tab()` | (744, 4, 772, 32) |
| O ▾ | `profiles()` | (772, 4, 796, 32) |

Os botóns quedan aí sexan oito tabs ou cen, e estea a franxa onde estea; de 796 a 800 vai a separación da dereita, e de 800 a 845, o sitio para arrastrar a xanela, xusto os 45 do mínimo; ata a 2.8, a barra remataba no 800. Coa franxa desprazada 50 píxeles, a primeira tab vai de 28 − 50 = −22 a 78 ([`tab`](#a-franxa-e-as-tabs-strip-start-tab-e-tab_area)): só se debuxan, e só reciben o rato, os seus 50 píxeles da dereita, desde o 28.

### As medidas

```rust
const ROW: u32 = 32;
const MAX_TAB: u32 = 240;
const MIN_TAB: u32 = 100;
const GAP: u32 = 4;
const PADDING: u32 = 8;
const CLOSE: u32 = 24;
const CLOSE_HIGHLIGHT: u32 = 20;
const CROSS: u32 = 8;
const NEW: u32 = 28;
const NEW_HIGHLIGHT: u32 = 24;
const PLUS: u32 = 10;
const PROFILES: u32 = 24;
const PROFILES_HIGHLIGHT: u32 = 20;
const CHEVRON: u32 = 4;
const SCROLL: u32 = 24;
const SCROLL_HIGHLIGHT: u32 = 20;
const CAPTION: u32 = 46;
const GLYPH: u32 = 10;
const GLYPH_OFFSET: u32 = 2;
const DRAG: u32 = 45;
const SEPARATOR: u32 = 16;
const STROKE: u32 = 1;
```

As medidas, en píxeles a escala 100 %, que é 96 DPI ([`BASE_DPI`](#os-axudantes-base_dpi-scale-e-coord), que desde a subfase 2.4 está en `ui/mod.rs`). [`Layout::new`](#crear-e-cambiar-new-row_at-min_width-set_band-e-resize) escálaas ao DPI da xanela.

| Constante | A 100 % | Que é |
|---|---|---|
| `ROW` | 32 | O alto da fila das tabs, o da fila de tabs do Windows Terminal: coa xanela maximizada, a barra enteira, e restaurada, a barra sen a banda de enriba. Ata a 2.8 chamábase `HEIGHT`, e era o alto da barra, que non tiña banda. |
| `MAX_TAB` | 240 | O ancho máximo dunha tab: o `TabViewItemMaxWidth` do `TabView` de WinUI, que usa o Windows Terminal. |
| `MIN_TAB` | 100 | Desde a 2.8: o ancho mínimo dunha tab, o `TabViewItemMinWidth` do mesmo `TabView` ([ADR 0025](../adr/0025-moitas-tabs-e-os-indicadores.md), decisión 1). Ten sitio para o × e para uns 8 ou 9 caracteres do título, segundo a escala. Se as tabs non caben nin así, a barra desprázaas. |
| `GAP` | 4 | A separación por riba das tabs, debaixo da banda, á esquerda da primeira, entre a última e o +, e á dereita do ▾ (ata a 2.4, do +). Desde a 2.8, mentres as tabs non caben, á esquerda do ‹ e entre o › e o +, no canto de á esquerda da primeira tab e despois da última. Tamén a que queda á dereita do ×. |
| `PADDING` | 8 | Desde o bordo esquerdo dunha tab ata o seu título. Nunha tab sen ×, que desde a 2.8 non hai a ningunha escala de Windows, tamén desde o título ata o bordo dereito. |
| `CLOSE` | 24 | O lado do ×, un cadrado á dereita da tab: todo el recibe o rato. |
| `CLOSE_HIGHLIGHT` | 20 | O lado do cadrado que resalta o punteiro, no medio do ×. |
| `CROSS` | 8 | O lado do cadrado da cruz, tamén no medio do ×. |
| `NEW` | 28 | O lado do +, un cadrado tan alto coma as tabs. |
| `NEW_HIGHLIGHT` | 24 | O lado do cadrado que resalta o punteiro, no medio do +. |
| `PLUS` | 10 | O lado do signo +, no medio do botón. Medra un píxel se non ten a paridade do trazo ([`new`](#crear-e-cambiar-new-row_at-min_width-set_band-e-resize)). |
| `PROFILES` | 24 | Desde a 2.5: o ancho do ▾, que abre o menú dos perfís, á dereita do + e tan alto coma el. Todo el recibe o rato. |
| `PROFILES_HIGHLIGHT` | 20 | O ancho do rectángulo que resalta o punteiro, no medio do ▾. O seu alto é o do resalte do +, `NEW_HIGHLIGHT`. |
| `CHEVRON` | 4 | O alto da frecha do ▾, que mide o dobre de ancho, 8, para que as súas dúas liñas se xunten a calquera escala ([`chevron`](#o-botón-dos-perfís-profiles-profiles_highlight-e-chevron)). Desde a 2.8, tamén o ancho das frechas do ‹ e do ›, que miden o dobre de alto ([`scroll_chevron`](#os-botóns-de-desprazamento-scroll_button-scroll_highlight-e-scroll_chevron)). |
| `SCROLL` | 24 | Desde a 2.8: o ancho do ‹ e do ›, os botóns de desprazamento, que son coma o ▾: tan altos coma o +, e todo o botón recibe o rato ([ADR 0025](../adr/0025-moitas-tabs-e-os-indicadores.md), decisión 2). |
| `SCROLL_HIGHLIGHT` | 20 | O ancho do rectángulo que resalta o punteiro, no medio do ‹ e do ›. O seu alto, coma no ▾, é o do resalte do +, `NEW_HIGHLIGHT`. |
| `CAPTION` | 46 | Desde a 2.9: o ancho de cada botón da xanela, coma no Windows Terminal; o seu alto é o da barra enteira, tamén por riba da banda ([ADR 0027](../adr/0027-a-barra-de-tabs-coma-barra-de-titulo.md), decisión 3). Todo o botón recibe o rato ([`caption`](#os-botóns-da-xanela-captionbutton-caption-captions-glyph-e-glyph_offset)). |
| `GLYPH` | 10 | Desde a 2.9: o lado do cadrado do glifo de cada botón da xanela, no seu medio, coma os glifos de 10 × 10 da fonte Segoe Fluent Icons, que usan o Windows 11 e o Windows Terminal. |
| `GLYPH_OFFSET` | 2 | Desde a 2.9: o glifo de restaurar son dous cadrados, e o de atrás vai estes píxeles máis arriba e máis á dereita ca o de diante. |
| `DRAG` | 45 | Desde a 2.9: o sitio mínimo para arrastrar a xanela, entre a separación da dereita do ▾ e os botóns da xanela, coma no Windows Terminal ([ADR 0027](../adr/0027-a-barra-de-tabs-coma-barra-de-titulo.md), decisión 5). |
| `SEPARATOR` | 16 | O alto das liñas que separan dúas tabs inactivas, no medio do alto das tabs. Teñen un píxel de ancho a calquera escala. |
| `STROKE` | 1 | O grosor das liñas da cruz e do signo + e, desde a 2.5, da frecha do ▾; desde a 2.8, tamén das do ‹ e do ›; e, desde a 2.9, das dos glifos dos botóns da xanela. |

Son privadas: fóra do módulo só se ven escaladas, a través de `Layout`. A 150 % (144 DPI), os botóns da xanela miden 69 de ancho, os seus glifos 15, cun trazo de 2, o cadrado de atrás do de restaurar vai 3 píxeles máis arriba e á dereita, e o sitio para arrastrar a xanela mide 68.

### `Part` e `Side`

```rust
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Part {
    Tab(usize),
    Close(usize),
    New,
    Profiles,
    Scroll(Side),
    Caption(CaptionButton),
    Empty,
}
```

A parte da barra que hai nun punto, a resposta de [`part_at`](#o-que-hai-debaixo-dun-punto-part_at):

| Variante | Que é |
|---|---|
| `Tab(índice)` | A tab nesa posición, desde 0 e na orde da barra, fóra do seu ×: tamén na separación de enriba, por riba do que se debuxa, pero, desde a 2.9, non na banda. Desde a 2.8, mentres as tabs non caben, só a súa parte dentro da franxa. |
| `Close(índice)` | O × da tab nesa posición. |
| `New` | O +, en todo o alto da fila das tabs: ata a 2.8, a barra enteira, e desde a 2.9, debaixo da banda. |
| `Profiles` | Desde a 2.5: o ▾, tamén en todo o alto da fila das tabs. |
| `Scroll(lado)` | Desde a 2.8: o botón de desprazamento dese lado, o ‹ ou o ›, tamén en todo o alto da fila das tabs. Só os hai mentres as tabs non caben. |
| `Caption(botón)` | Desde a 2.9: o botón da xanela dese tipo, minimizar, maximizar ou restaurar, ou pechar ([`CaptionButton`](#os-botóns-da-xanela-captionbutton-caption-captions-glyph-e-glyph_offset)), en todo o alto da barra, tamén na banda. |
| `Empty` | A barra fóra das tabs e dos botóns: á esquerda da primeira tab, entre a última e o +, e á dereita do ▾. Desde a 2.8, mentres as tabs non caben, á esquerda do ‹ e entre o › e o +; e, mentres as tabs manteñen o seu ancho despois dun peche co rato, na franxa despois da última. Desde a 2.9, tamén a banda, fóra dos botóns da xanela, e o sitio para arrastrar a xanela, entre o ▾ e eles: para Windows, a banda é o bordo de arriba da xanela, e o resto, a barra de título ([`ui/titlebar.rs`](#cratesmeiga-coresrcuititlebarrs)). |

```rust
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Side {
    Left,
    Right,
}
```

Desde a 2.8, **`Side`** é un lado da franxa, cara ao que a despraza un botón:

| Variante | Que é |
|---|---|
| `Left` | Cara á primeira tab: o ‹, á esquerda da franxa. |
| `Right` | Cara á última: o ›, á dereita. |

- As tabs van pola súa posición, e non polo seu identificador: a disposición non coñece o modelo, só cantas tabs hai. A barra do programa busca a tab desa posición na xanela ([`Window::tabs`](#window-os-datos-e-as-consultas)), e cos botóns do rato decide que facer ([ADR 0020](../adr/0020-a-barra-de-tabs.md), decisión 6): desde a subfase 2.4, o dereito abre o menú da tab ([ADR 0021](../adr/0021-os-menus-e-o-menu-da-tab.md), decisión 5); desde a 2.5, o esquerdo, ao soltarse enriba do ▾ no que se premeu, o menú dos perfís; desde a 2.8, o esquerdo nun botón de desprazamento despraza a franxa unha tab, e outra mentres segue premido ([ADR 0025](../adr/0025-moitas-tabs-e-os-indicadores.md), decisión 4); e, desde a 2.9, o esquerdo, ao soltarse enriba do botón da xanela no que se premeu, minimiza, maximiza ou restaura, ou pecha a xanela ([ADR 0027](../adr/0027-a-barra-de-tabs-coma-barra-de-titulo.md), decisión 4).
- Os dous son `Copy`, coma os tipos de [`mouse.rs`](#button-modifiers-wheel-e-mouseaction): uns poucos bytes, que van por valor.
- Non é o `Side` da selección, unha metade dunha cela ([`term/selection.rs`](#word_delimiters-selectionkind-e-side)), aínda que teña as mesmas variantes: cada un úsase co seu módulo, `tabbar::Side` e `term::Side`.

### `Layout`: os campos

```rust
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Layout {
    row: u32,
    max_tab: u32,
    min_tab: u32,
    gap: u32,
    padding: u32,
    close: u32,
    close_highlight: u32,
    cross: u32,
    new: u32,
    new_highlight: u32,
    plus: u32,
    profiles: u32,
    profiles_highlight: u32,
    chevron: u32,
    scroll_button: u32,
    scroll_highlight: u32,
    caption: u32,
    glyph: u32,
    glyph_offset: u32,
    drag: u32,
    separator: u32,
    stroke: u32,
    band: u32,
    width: u32,
    tabs: usize,
    tab_width: u32,
    overflow: bool,
    scroll: u32,
}
```

Onde van as partes da barra, para un número de tabs, un ancho, unha escala e, desde a 2.9, a banda enriba das tabs:

| Campo | Que é |
|---|---|
| De `row` a `stroke` | As medidas, unha por constante, xa escaladas ao DPI da xanela. A de `SCROLL` chámase `scroll_button`, porque `scroll` é o desprazamento; `chevron`, desde a 2.8, é o lado curto das tres frechas; e `caption`, `glyph`, `glyph_offset` e `drag`, desde a 2.9, as dos botóns da xanela e do sitio para arrastrala. Ata a 2.8, `row` chamábase `height`. |
| `band` | Desde a 2.9: o alto da banda enriba da fila das tabs, onde o punteiro redimensiona a xanela, en píxeles á escala da xanela, sen escalar eiquí ([`new`](#crear-e-cambiar-new-row_at-min_width-set_band-e-resize)). 0 coa xanela maximizada, e tamén sen a barra de título de meiga. |
| `width` | O ancho da barra: o da área cliente da xanela. |
| `tabs` | Cantas tabs hai. |
| `tab_width` | O ancho de cada tab, que sae dos outros ([`fit`](#a-barra-e-o-ancho-das-tabs-bar-room-viewport-e-fit)). |
| `overflow` | Desde a 2.8: se as tabs non caben nin co ancho mínimo, e van na franxa entre os botóns de desprazamento. |
| `scroll` | Desde a 2.8: mentres as tabs non caben, cantos píxeles da fila das tabs oculta a franxa á súa esquerda. Mentres caben, 0. |

```rust
pub const fn height(&self) -> u32 {
    self.band.saturating_add(self.row)
}

pub const fn band(&self) -> u32 {
    self.band
}

pub const fn width(&self) -> u32 {
    self.width
}

pub const fn tabs(&self) -> usize {
    self.tabs
}

pub const fn tab_width(&self) -> u32 {
    self.tab_width
}

pub const fn overflows(&self) -> bool {
    self.overflow
}

pub const fn scroll(&self) -> u32 {
    self.scroll
}

pub const fn stroke(&self) -> u32 {
    self.stroke
}
```

- **As medidas escálanse unha vez**, ao facer a disposición, e non en cada consulta. Cambiar o ancho ou o número de tabs só volta calcular o ancho das tabs e, desde a 2.8, se caben e ata onde chega o desprazamento ([`resize`](#crear-e-cambiar-new-row_at-min_width-set_band-e-resize)); desde a 2.9, cambiar a banda non calcula nada ([`set_band`](#crear-e-cambiar-new-row_at-min_width-set_band-e-resize)); e cambiar o DPI fai outra disposición.
- Os campos son privados: ningún método deixa o ancho das tabs sen corresponder ao ancho da barra e ao número de tabs, salvo, desde a 2.8, [`keep_widths`](#as-tabs-no-seu-sitio-ao-pechar-co-rato-keep_widths), que o fai a propósito mentres se pechan tabs co rato. Lense con `height`, `width`, `tabs`, `tab_width`, `stroke`, desde a 2.8, `overflows` e `scroll`, e, desde a 2.9, `band`, que son `const fn`. O alto da barra é tamén onde empeza o terminal.
- **`height`**, desde a 2.9, é o alto da barra enteira, a banda máis a fila das tabs: 32 a 96 DPI coa xanela maximizada, e 40 restaurada, coa banda de 8. `saturating_add` non desborda con ningunha banda. Ata a 2.8 lía o campo `height`, que era o alto da fila.
- **`band`**, desde a 2.9: o alto da banda. `ui/titlebar.rs` mira con el se un punto libre da barra está na banda ([`hit`](#o-que-é-cada-punto-hit)), e a barra do programa, ao facer outra disposición co DPI novo, cóllea para a nova.
- Deriva o mesmo ca `Part`. É `Copy`: vinte e seis números de 32 bits, vinte e un ata a 2.8, dezasete ata a 2.7 e catorce ata a 2.4, un `usize` e, desde a 2.8, un `bool`, que a barra do programa copia ao debuxar cada tab. Desde a 2.9, o programa cópiaa tamén para a xanela despois de cada evento, nun [`TitleBar`](#cratesmeiga-coresrcuititlebarrs). `PartialEq` compara dúas disposicións enteiras, coma fai unha proba coa banda e sen ela ([Probas](#os-axudantes-scale-coord-signed-clamped-centered-e-centered_size)).

### Crear e cambiar: `new`, `row_at`, `min_width`, `set_band` e `resize`

```rust
pub fn new(dpi: u32, width: u32, tabs: usize, band: u32) -> Self {
    let size = |pixels| scale(pixels, dpi);
    let stroke = size(STROKE);
    let plus = size(PLUS);
    let mut layout = Self {
        row: size(ROW),
        max_tab: size(MAX_TAB),
        min_tab: size(MIN_TAB),
        gap: size(GAP),
        padding: size(PADDING),
        close: size(CLOSE),
        close_highlight: size(CLOSE_HIGHLIGHT),
        cross: size(CROSS),
        new: size(NEW),
        new_highlight: size(NEW_HIGHLIGHT),
        plus: plus + plus.abs_diff(stroke) % 2,
        profiles: size(PROFILES),
        profiles_highlight: size(PROFILES_HIGHLIGHT),
        chevron: size(CHEVRON),
        scroll_button: size(SCROLL),
        scroll_highlight: size(SCROLL_HIGHLIGHT),
        caption: size(CAPTION),
        glyph: size(GLYPH),
        glyph_offset: size(GLYPH_OFFSET),
        drag: size(DRAG),
        separator: size(SEPARATOR),
        stroke,
        band,
        width,
        tabs,
        tab_width: 0,
        overflow: false,
        scroll: 0,
    };
    layout.fit();
    layout
}

pub fn row_at(dpi: u32) -> u32 {
    scale(ROW, dpi)
}

pub fn min_width(dpi: u32) -> u32 {
    let layout = Self::new(dpi, 0, 0, 0);
    layout
        .fixed()
        .saturating_add(layout.scroll_button.saturating_mul(2))
        .saturating_add(layout.min_tab)
}

pub const fn set_band(&mut self, band: u32) {
    self.band = band;
}

pub fn resize(&mut self, width: u32, tabs: usize) {
    self.width = width;
    self.tabs = tabs;
    self.fit();
}
```

- **`new`** fai a barra de `tabs` tabs nunha xanela de `width` píxeles de ancho, a `dpi`, e, desde a 2.9, cunha banda de `band` píxeles enriba da fila das tabs. Escala cada medida con [`scale`](#os-axudantes-base_dpi-scale-e-coord), a través da *closure* `size`, e despois dálles ás tabs o seu ancho, que necesita as medidas xa escaladas: por iso `tab_width` empeza en 0, `overflow` en `false` e `scroll` en 0, e [`fit`](#a-barra-e-o-ancho-das-tabs-bar-room-viewport-e-fit) chámase cando a disposición xa existe. Ata a 2.7, `fit_tabs` daba o ancho, e `new` gardábao en `tab_width`.
- **A banda non se escala.** É o grosor do marco que Windows lle dá á xanela, `SM_CXPADDEDBORDER` máis `SM_CYSIZEFRAME`, que meiga-win xa pide ao DPI da xanela, e que non é unha medida de 100 % escalada: 8 píxeles a 96 DPI, e 11 a 144, onde 8 escalado darían 12 ([`Window::resize_band`](meiga-win.md#cratesmeiga-winsrcwindowrs)). Por iso `new` a garda tal cal, e a barra do programa pásalla así, 0 coa xanela maximizada ou sen a barra de título de meiga. Ata a 2.8, `new` recibía tres argumentos, sen a banda.
- **O signo + ten os brazos iguais.** A barra do programa debúxao cunha liña horizontal e outra vertical de `stroke` píxeles, que se cruzan no medio dun cadrado de `plus` de lado. Os brazos, o que sobresae cada liña a cada lado da outra, miden `(plus - stroke) / 2` se esa resta é par; se é impar, un brazo tería un píxel máis ca o outro, así que o lado medra un píxel: `abs_diff` dá a diferenza sen signo, e `% 2`, 1 se é impar. A 100 %, o trazo ten 1 e o lado 10, que medra a 11, con brazos de 5; a 150 % (144 DPI), o trazo ten 2 e o lado 15, que medra a 16, con brazos de 7; e a 125 % (120 DPI), o trazo ten 1 e o lado 13, que queda.
- **`row_at`** dá o alto da fila das tabs a un DPI sen facer a disposición: 32 a 96 DPI e 40 a 120. Ata a 2.8 chamábase `height_at`, e daba o alto da barra, que era o mesmo, e só o usaban as probas; desde a 2.9 úsao tamén [`min_client`](#o-tamaño-mínimo-min_client), para o alto mínimo da área cliente, coa banda.
- **`min_width`**, desde a 2.9: o ancho da barra máis estreita a `dpi`: o que non é das tabs ([`fixed`](#a-barra-e-o-ancho-das-tabs-bar-room-viewport-e-fit)), cos botóns da xanela e o sitio para arrastrala, máis unha franxa que mostra enteira unha tab das máis estreitas entre os dous botóns de desprazamento: a 100 %, 3 × 4 + 28 + 24 + 45 + 3 × 46 + 2 × 24 + 100 = 395; a 125 % (120 DPI), 495; e a 150 % (144 DPI), 593. Fai unha disposición de ancho 0, sen tabs nin banda, só para ler as súas medidas escaladas, e suma sen desbordar (`saturating_add` e `saturating_mul`). Nesa barra, con dúas tabs ou máis, que non caben, a franxa mide xusto unha tab das máis estreitas, 100 a 100 %; e unha soa tab cabe sen franxa, de 148. `min_client` dálle este ancho á área cliente, e a xanela non pode ser máis estreita ([ADR 0027](../adr/0027-a-barra-de-tabs-coma-barra-de-titulo.md), decisión 9).
- **`set_band`**, desde a 2.9: a banda mide agora `band` píxeles. Só cambia ese campo: o ancho das tabs, o seu desprazamento e todo o que vai á dereita e á esquerda quedan coma estaban, e o que vai debaixo da banda, as tabs, o +, o ▾ e os botóns de desprazamento, baixa ou sobe con ela, porque se mide desde `band` en cada consulta ([`top`](#a-franxa-e-as-tabs-strip-start-tab-e-tab_area)). Sen banda, a disposición é igual á que fai `new` sen ela. A barra do programa chámao cando cambia a banda: ao maximizar e restaurar a xanela, e ao cambiar de escala, despois de facer a disposición nova co DPI novo e a banda vella ([Código › meiga › `tabbar.rs`](meiga.md#cratesmeigasrctabbarrs)). É `const fn`, coma os lectores.
- **`resize`**: a barra ten agora `width` píxeles de ancho e `tabs` tabs. As medidas quedan, e só cambian o ancho das tabs e, desde a 2.8, se caben e o desprazamento, que queda onde estaba se a franxa aínda chega alí ([`fit`](#a-barra-e-o-ancho-das-tabs-bar-room-viewport-e-fit)). A barra do programa chámao cando cambia o ancho da xanela e cando se abren ou se pechan tabs; desde a 2.8, salvo cando unha tab se pecha co rato e as outras manteñen o seu ancho ([`keep_widths`](#as-tabs-no-seu-sitio-ao-pechar-co-rato-keep_widths)), que `resize` lles dá outra vez despois. Cando cambia o DPI, a barra do programa fai outra disposición con `new`, que empeza sen desprazamento, e no seguinte fotograma a franxa móvese ata a tab que se mostra ([ADR 0025](../adr/0025-moitas-tabs-e-os-indicadores.md), decisión 5).

### A barra e o ancho das tabs: `bar`, `room`, `viewport` e `fit`

```rust
pub fn bar(&self) -> Rect {
    Rect::new(0, 0, coord(self.width.into()), coord(self.height().into()))
}

const fn fixed(&self) -> u32 {
    self.gap
        .saturating_mul(3)
        .saturating_add(self.new)
        .saturating_add(self.profiles)
        .saturating_add(self.drag)
        .saturating_add(self.caption.saturating_mul(3))
}

const fn room(&self) -> u32 {
    self.width.saturating_sub(self.fixed())
}

const fn viewport(&self) -> u32 {
    self.room().saturating_sub(2 * self.scroll_button)
}

fn fit(&mut self) {
    let tabs = u32::try_from(self.tabs.max(1)).unwrap_or(u32::MAX);
    let width = (self.room() / tabs).min(self.max_tab);
    self.overflow = width < self.min_tab;
    self.tab_width = if self.overflow { self.min_tab } else { width };
    self.scroll = self.scroll.min(self.max_scroll());
}
```

- **`bar`**: a barra enteira, desde (0, 0), co ancho da xanela e o alto da barra: desde a 2.9, desde o bordo de arriba da banda, co alto de [`height`](#layout-os-campos), a banda máis a fila. No exemplo, (0, 0, 983, 32), e coa banda de 8, (0, 0, 983, 40). A barra do programa mira con ela se un punto está na barra ([`contains_point`](#cratesmeiga-coresrcgeometryrs)), e márcaa enteira cando cambia a disposición.
- **`fixed`**, privada, desde a 2.9: o ancho da barra que non é das tabs: as tres separacións, á esquerda da primeira tab, despois da última e á dereita do ▾, o + e o ▾, o sitio mínimo para arrastrar a xanela e os tres botóns da xanela. A 100 %, 3 × 4 + 28 + 24 + 45 + 3 × 46 = 247; a 150 %, 3 × 6 + 42 + 36 + 68 + 3 × 69 = 371. Úsano `room` e [`min_width`](#crear-e-cambiar-new-row_at-min_width-set_band-e-resize). As contas saturan, en vez de desbordar, e son `const fn`, coma `saturating_mul` e `saturating_add`.
- **`room`**, privada, desde a 2.8: o sitio das tabs mentres caben, o ancho da barra menos `fixed`: a 100 %, en 983 píxeles, 983 − 247 = 736. Ata a 2.8 restaba só as tres separacións e os dous botóns, sen o sitio para arrastrar a xanela nin os seus botóns: en 800 píxeles, 800 − 3 × 4 − 28 − 24 = 736. Ata a 2.7 calculábao `fit_tabs`, e ata a 2.4, sen o ▾, eran 760. É `const fn`, coma `saturating_sub`. Desde a 2.9, as tabs teñen 183 píxeles menos ca na 2.8 nunha xanela do mesmo ancho, a 100 %, pero a xanela xa non ten a barra de título de Windows, cos mesmos botóns ([ADR 0027 › Consecuencias](../adr/0027-a-barra-de-tabs-coma-barra-de-titulo.md#consecuencias)).
- **`viewport`**, privada, desde a 2.8: o ancho da franxa mentres as tabs non caben, o sitio menos o ‹ e o ›: 736 − 2 × 24 = 688, seis tabs enteiras e 88 píxeles da sétima. As tres separacións seguen, pero noutros sitios: á esquerda do ‹, entre o › e o +, e á dereita do ▾. Nunha xanela máis estreita ca `fixed` e os dous botóns de desprazamento, 295 píxeles a 100 %, 0.
- **`fit`**, privada, que desde a 2.8 substitúe a `fit_tabs`: dálles ás tabs o seu ancho, e di se caben ([ADR 0020](../adr/0020-a-barra-de-tabs.md), decisión 3, e [ADR 0025](../adr/0025-moitas-tabs-e-os-indicadores.md), decisión 1):
  1. O sitio repártese entre as tabs, e cada unha ten como moito `max_tab`: con tres, 245, que quedan en 240; con sete, 105.
  2. Se así medisen menos de `min_tab`, non caben (`overflow`), e todas miden `min_tab`, 100: con oito, que medirían 92, e con trinta, que medirían 24. Así, a escala 100 %, nunha xanela de 983 píxeles caben sete tabs, e oito xa non: a súa fila mide 800, e a franxa, 688; ata a 2.8, o mesmo nunha de 800. Unhas tabs que medirían xusto `min_tab` caben, porque a comparación é `<`.
  3. O desprazamento queda, ata onde pode chegar a franxa coas tabs e o ancho novos ([`max_scroll`](#canto-se-despraza-max_scroll-e-can_scroll)): se agora caben, 0.

  Mentres caben, o que sobra da división queda á dereita do ▾, e o sitio para arrastrar a xanela mide máis ca o mínimo; mentres non, o + e o ▾ van ao final da franxa, e o sitio para arrastrar a xanela mide xusto `drag`, 45 a 100 % ([ADR 0027](../adr/0027-a-barra-de-tabs-coma-barra-de-titulo.md), decisión 5). Ata a 2.8, o que sobraba quedaba coa barra baleira, e, mentres as tabs non cabían, o + e o ▾ ían ao final da barra. Ata a 2.7, sen mínimo: con moitas tabs, estreitábanse ata quedar sen × e sen título, e, nunha xanela máis estreita ca os dous botóns e as súas separacións, ata 0 ([ADR 0020 › Consecuencias](../adr/0020-a-barra-de-tabs.md#consecuencias)). `saturating_sub` deixa o sitio en 0 en vez de desbordar, e `max(1)` evita dividir entre 0 sen tabs. Nunha xanela máis estreita ca `fixed`, a división dá 0: as tabs miden `min_tab` e van nunha franxa de 0 píxeles. Cun número de tabs que non cabe nun `u32`, `try_from` dá `u32::MAX`, e a división tamén 0: non caben. Ao contrario ca `fit_tabs`, que só daba o ancho, `fit` cambia tres campos: recibe `&mut self`, e non dá nada.

  Desde a 2.9, coa barra de título de meiga, Windows non deixa a área cliente máis estreita ca [`min_width`](#crear-e-cambiar-new-row_at-min_width-set_band-e-resize), 395 a 100 %. Só sen ela, se Windows non deixa poñela, pode selo: entón, por debaixo dos 295 de `fixed` e os botóns de desprazamento, a franxa queda en 0, o + e o ▾ quedan quietos, e o sitio para arrastrar encolle co ancho da xanela, ata que os botóns da xanela, que entón non se debuxan, quedan enriba do ▾, por debaixo de 246, e do +, por debaixo de 222: alí `part_at` dá un botón da xanela, e o + e o ▾ non reciben o rato.

### A franxa e as tabs: `strip`, `start`, `tab` e `tab_area`

```rust
pub fn strip(&self) -> Rect {
    if !self.overflow {
        return self.bar();
    }
    let left = u64::from(self.gap) + u64::from(self.scroll_button);
    Rect::new(
        coord(left),
        0,
        coord(left + u64::from(self.viewport())),
        coord(self.height().into()),
    )
}

fn start(&self) -> i64 {
    if self.overflow {
        i64::from(self.strip().left) - i64::from(self.scroll)
    } else {
        i64::from(self.gap)
    }
}
```

- **`strip`**, desde a 2.8: a parte da barra onde van as tabs. Mentres caben, toda a barra; mentres non, a franxa entre o ‹ e o ›, tan alta coma a barra, desde `gap` + `scroll_button`, 28 a 100 %, e co ancho de `viewport`: en 983 píxeles, (28, 0, 716, 32), e coa banda de 8, (28, 0, 716, 40), que cobre tamén a banda. A barra do programa recorta a ela o que debuxa das tabs e as zonas que marca, caiban ou non: mentres caben, recortar á barra non corta nada.
- **`start`**, privada, desde a 2.8: onde empeza a primeira tab, ou onde empezaría: a `gap` píxeles do bordo esquerdo da barra mentres caben e, mentres non, no bordo esquerdo da franxa, menos o desprazamento. É un `i64`, porque pode ser negativo: cun desprazamento de 50, 28 − 50 = −22. `tab` e `part_at` miden desde el.

```rust
pub fn tab(&self, index: usize) -> Rect {
    let left = self.start().saturating_add(signed(
        (index as u64).saturating_mul(u64::from(self.tab_width)),
    ));
    Rect::new(
        clamped(left),
        coord(self.top().into()),
        clamped(left.saturating_add(self.tab_width.into())),
        coord(self.height().into()),
    )
}

pub fn tab_area(&self, index: usize) -> Rect {
    let tab = self.tab(index);
    let band = coord(self.band.into());
    Rect::new(tab.left, band, tab.right.saturating_add(1), tab.bottom)
        .intersection(self.strip())
}

const fn top(&self) -> u32 {
    self.band.saturating_add(self.gap)
}

const fn tab_height(&self) -> u32 {
    self.row.saturating_sub(self.gap)
}
```

- **`tab`**: a tab en `index` como se debuxa: desde onde empeza a primeira, máis `index` × `tab_width`, co ancho dunha tab, e desde `gap` píxeles por debaixo do bordo de arriba da fila das tabs (`top`) ata o bordo de abaixo da barra. Ata a 2.8, desde `gap` píxeles por debaixo do bordo de arriba da barra, que era o da fila. Coa banda de 8, a primeira tab do exemplo vai en (4, 12, 244, 40). Para o rato, a tab ocupa tamén a separación de enriba, pero non a banda ([`part_at`](#o-que-hai-debaixo-dun-punto-part_at)). Desde a 2.8, mentres as tabs se desprazan, pode quedar fóra da franxa, en parte ou enteira: con oito tabs, a oitava vai de 728 a 828, máis alá da franxa, que remata no 716; e, co desprazamento de 50, a primeira vai de −22 a 78. A barra do programa só debuxa a súa parte na franxa.
  - As contas fanse en `u64`, saturando, e desde a 2.8 en `i64`, porque `start` pode ser negativo: [`signed`](#os-axudantes-scale-coord-signed-clamped-centered-e-centered_size) converte o produto, e [`clamped`](#os-axudantes-scale-coord-signed-clamped-centered-e-centered_size) converte o resultado nunha coordenada, tamén cara aos negativos. Ata a 2.7, as tabs empezaban sempre en `gap`, e convertíaas [`coord`](#os-axudantes-base_dpi-scale-e-coord). `tab` acepta calquera índice, tamén o da tab despois da última, onde empeza o + mentres as tabs caben ([`new_tab`](#o-botón-que-abre-unha-tab-new_tab-new_highlight-e-plus)).
  - Desde a 2.4, o menú que abre o teclado vai debaixo da tab que se mostra, coma a dá `tab` ([`Anchor::Below`](#onde-se-abre-anchor-e-place)). Desde a 2.8, a barra do programa despraza antes a franxa ata ela ([`reveal`](#desprazar-a-franxa-scroll_by-step-reveal-e-scroll_to)), e o menú vai debaixo da súa parte na franxa ([ADR 0025](../adr/0025-moitas-tabs-e-os-indicadores.md), decisión 5).
- **`tab_area`**: o que cambia cando a tab en `index` cambia de aspecto, porque pasa a estar activa ou resaltada, ou deixa de estalo: toda a súa columna da fila das tabs, desde o bordo de arriba da fila, e un píxel máis á dereita, a primeira columna da tab seguinte, onde vai o separador entre as dúas, que depende tamén do aspecto desta. Recórtase á barra e, desde a 2.8, á franxa: na primeira tab do exemplo, (4, 0, 245, 32); con oito tabs, na sétima, que a franxa corta no 716, (628, 0, 716, 32), e na oitava, fóra da franxa, un rectángulo baleiro, que a barra do programa non marca ([ADR 0025](../adr/0025-moitas-tabs-e-os-indicadores.md), decisión 10). A barra do programa marca así as tabs que gañan ou perden o resalte do punteiro (decisión 8 do [ADR 0020](../adr/0020-a-barra-de-tabs.md)). Desde a 2.9, empeza debaixo da banda, que non cambia co aspecto da tab: coa banda de 8, (4, 8, 245, 40) na primeira, e (628, 8, 716, 40) na sétima de oito; ata a 2.8, desde o bordo de arriba da barra.
- **`top`**, privada, desde a 2.9: onde empezan as tabs como se debuxan, o +, o ▾ e os botóns de desprazamento: a separación debaixo da banda, `band` + `gap`, 4 sen banda e 12 coa de 8. Ata a 2.8, todos empezaban en `gap`. Como todos a miden desde `band`, [`set_band`](#crear-e-cambiar-new-row_at-min_width-set_band-e-resize) móveos sen cambiar ningún outro campo.
- **`tab_height`**, privada: o alto das tabs como se debuxan, o da fila menos a separación de enriba, 28 a 100 %, coa banda ou sen ela; ata a 2.8, o da barra, que era a fila. O × e os separadores céntranse nel.

### O botón de pechar: `close`, `close_highlight` e `cross`

```rust
pub fn close(&self, index: usize) -> Option<Rect> {
    let tab = self.tab(index);
    let gap = coord(self.gap.into());
    let right = tab.right - gap;
    let left = right - coord(self.close.into());
    if left < tab.left + gap {
        return None;
    }
    let top = tab.top + coord((self.tab_height().saturating_sub(self.close) / 2).into());
    Some(Rect::new(left, top, right, top + coord(self.close.into())))
}

pub fn close_highlight(&self, index: usize) -> Option<Rect> {
    self.close(index)
        .map(|close| centered(close, self.close_highlight))
}

pub fn cross(&self, index: usize) -> Option<Rect> {
    self.close(index).map(|close| centered(close, self.cross))
}
```

**`close`** dá o × da tab en `index`, ou `None` se a tab é demasiado estreita para el:

1. Remata `gap` píxeles antes do bordo dereito da tab, e mide `close` de ancho.
2. Se así empezase a menos de `gap` píxeles do bordo esquerdo, non cabe: a tab é máis estreita ca o × e as dúas separacións dos seus lados, 32 píxeles a 100 %, e queda sen el ([ADR 0020](../adr/0020-a-barra-de-tabs.md), decisión 3). Ata a 2.7, as tabs que se encollían tanto pechábanse co botón do medio. Desde a 2.8, a máis estreita mide 100 ([ADR 0025](../adr/0025-moitas-tabs-e-os-indicadores.md), decisión 1), e o × cabe sempre: `None` só sae a 1 e a 2 DPI, onde o redondeo de `scale` deixa a tab máis estreita en 1 e 2 píxeles, e o × coas súas separacións en 3.
3. Vai centrado no alto da tab que se debuxa ([`tab_height`](#a-franxa-e-as-tabs-strip-start-tab-e-tab_area)): na primeira tab do exemplo, (216, 6, 240, 30), con 2 píxeles por riba e 2 por debaixo; desde a 2.9, coa banda de 8, baixa con ela, a (216, 14, 240, 38).

- **`close_highlight`** e **`cross`**: o cadrado que resalta o punteiro, de 20 a 100 %, e o da cruz, de 8, no medio do × ([`centered`](#os-axudantes-scale-coord-signed-clamped-centered-e-centered_size)); `None` se a tab non ten ×. A barra do programa debuxa a cruz coma dúas diagonais de `stroke` píxeles de grosor, en chanzos dun píxel, coma, desde a 2.5, a frecha do ▾.
- Para o rato, o × é todo o seu cadrado, aínda que o punteiro só resalte o do medio ([`part_at`](#o-que-hai-debaixo-dun-punto-part_at)); desde a 2.8, só a parte del que está na franxa. O × dunha tab cortada pode quedar fóra da franxa, en parte ou enteiro: `close` dá o seu cadrado enteiro, e a barra do programa recórtao á franxa ao debuxalo e ao marcalo.

### O título e os separadores: `title` e `separator`

```rust
pub fn title(&self, index: usize) -> Rect {
    let tab = self.tab(index);
    let left = tab.left + coord(self.padding.into());
    let right = self
        .close(index)
        .map_or(tab.right - coord(self.padding.into()), |close| close.left);
    Rect::new(left, tab.top, right.max(left), tab.bottom)
}

pub fn separator(&self, index: usize) -> Option<Rect> {
    if index == 0 || index >= self.tabs {
        return None;
    }
    let tab = self.tab(index);
    let top = tab.top + coord((self.tab_height().saturating_sub(self.separator) / 2).into());
    Some(Rect::new(
        tab.left,
        top,
        tab.left + 1,
        top + coord(self.separator.into()),
    ))
}
```

- **`title`**: o sitio do título da tab en `index`: desde `padding` píxeles do seu bordo esquerdo ata o ×, ou, se a tab non ten ×, ata `padding` píxeles do bordo dereito; e o alto da tab que se debuxa. Nunca vai ao revés: se non hai sitio, `right.max(left)` deixa un rectángulo baleiro. No exemplo, (12, 4, 216, 32); con sete tabs de 105, o da última, (642, 4, 711, 32); e nunha tab de 100, 64 píxeles, entre os 8 da esquerda e o ×. Ata a 2.7, con trinta tabs de 24, sen ×, era (12, 4, 20, 32), e con tabs de ancho 0, (12, 4, 12, 32), baleiro.
  - A barra do programa corta o título ás columnas que caben neste rectángulo ([`text::fit`](#cratesmeiga-coresrctextrs)) e debúxao centrado no alto da tab, recortado a el. Cando un título cambia, só redebuxa este rectángulo (decisión 8 do [ADR 0020](../adr/0020-a-barra-de-tabs.md)). Desde a 2.8, as primeiras columnas poden ser dun indicador, coa cela baleira que o separa do título ([ADR 0025](../adr/0025-moitas-tabs-e-os-indicadores.md), decisión 7); e o rectángulo que se redebuxa recórtase á franxa: o dunha tab fóra dela non debuxa nada (decisión 10).
- **`separator`**: a liña á esquerda da tab en `index`, entre ela e a anterior: un píxel de ancho, na primeira columna da tab, e `separator` de alto, centrada no alto da tab que se debuxa. No exemplo, a da segunda tab é (244, 10, 245, 26), e coa banda de 8, (244, 18, 245, 34). A primeira tab non ten ningunha, nin hai ningunha despois da última: `None`. A barra do programa só a debuxa entre dúas tabs que non están activas nin resaltadas.

### O botón que abre unha tab: `new_tab`, `new_highlight` e `plus`

```rust
pub fn new_tab(&self) -> Rect {
    let gap = coord(self.gap.into());
    let left = if self.overflow {
        self.strip().right + coord(self.scroll_button.into()) + gap
    } else {
        self.tab(self.tabs).left + gap
    };
    let top = coord(self.top().into());
    Rect::new(
        left,
        top,
        left + coord(self.new.into()),
        top + coord(self.new.into()),
    )
}

pub fn new_highlight(&self) -> Rect {
    centered(self.new_tab(), self.new_highlight)
}

pub fn plus(&self) -> Rect {
    centered(self.new_tab(), self.plus)
}
```

- **`new_tab`**: o +, un cadrado de `new` píxeles. Mentres as tabs caben, `gap` píxeles despois da última: onde empezaría unha tab máis (`tab(self.tabs)`), máis a separación. Desde a 2.8, mentres non caben, `gap` píxeles despois do ›, á dereita da franxa, sempre no mesmo sitio, sexan as tabs que sexan e estea a franxa onde estea ([ADR 0025](../adr/0025-moitas-tabs-e-os-indicadores.md), decisión 2): en 983 píxeles, (744, 4, 772, 32). Na 2.8, a franxa chegaba ata el, e o + e o ▾ ían ao final da barra; desde a 2.9, despois deles van o sitio para arrastrar a xanela e os seus botóns. Vai á altura das tabs, desde `gap` por debaixo do bordo de arriba da fila ([`top`](#a-franxa-e-as-tabs-strip-start-tab-e-tab_area)): a 100 %, de 4 a 32, e coa banda de 8, de 12 a 40; ata a 2.8, sempre desde `gap`. No exemplo, (728, 4, 756, 32); con sete tabs de 105, (743, 4, 771, 32), co ▾ en (771, 4, 795, 32), e dos 50 píxeles que quedan ata o 845, onde empezan os botóns da xanela, 4 son a separación da dereita, 45 o sitio mínimo para arrastrar a xanela, e 1 o que sobra da división; ata a 2.8, ata o 800, só a separación e ese píxel. Ata a 2.7, dez tabs de 73 cabían, e o + ía en (738, 4, 766, 32).
- **`new_highlight`** e **`plus`**: o cadrado que resalta o punteiro, de 24 a 100 %, e o do signo +, de 11, no medio do botón: no exemplo, (730, 6, 754, 30) e (736, 12, 747, 23).

### O botón dos perfís: `profiles`, `profiles_highlight` e `chevron`

Desde a subfase 2.5: o ▾, que abre o menú dos perfís, unido ao +, coma o botón dividido do Windows Terminal ([ADR 0022](../adr/0022-os-perfis-e-a-configuracion.md), decisión 6; [Tecnoloxías › Os perfís e a configuración](../tecnoloxias.md#57-os-perfís-e-a-configuración-toml-o-contorno-e-as-shells-de-windows)). Concreta a decisión 9 do [ADR 0020](../adr/0020-a-barra-de-tabs.md), na que o + abría o shell por defecto: agora abre o perfil por defecto, e o ▾, calquera.

```rust
pub fn profiles(&self) -> Rect {
    let new = self.new_tab();
    Rect::new(
        new.right,
        new.top,
        new.right + coord(self.profiles.into()),
        new.bottom,
    )
}

pub fn profiles_highlight(&self) -> Rect {
    centered_size(self.profiles(), self.profiles_highlight, self.new_highlight)
}

pub fn chevron(&self) -> Rect {
    centered_size(self.profiles(), 2 * self.chevron, self.chevron)
}
```

- **`profiles`**: o ▾, unido ao +, sen separación: empeza no seu bordo dereito, co seu alto, e mide `profiles` de ancho, 24 a 100 %. No exemplo, (756, 4, 780, 32), e coa banda de 8, (756, 12, 780, 40); desde a 2.8, mentres as tabs non caben, vai co + despois do ›: en 983 píxeles, (772, 4, 796, 32), e na 2.8, en 800, ao final da barra. Os dous vense coma un botón dividido, pero cada un ten o seu resalte e a súa acción: o + abre unha tab co perfil por defecto, e o ▾, o menú dos perfís, que se abre debaixo dos dous ([`Anchor::Below`](#onde-se-abre-anchor-e-place); [Código › meiga › `tabbar.rs`](meiga.md#cratesmeigasrctabbarrs)).
- **`profiles_highlight`**: o rectángulo que resalta o punteiro, no medio do ▾: `profiles_highlight` de ancho, 20 a 100 %, e tan alto coma o resalte do +, 24, para que os dous queden á mesma altura. Entre os dous resaltes quedan 4 píxeles sen resaltar, 2 de cada botón. No exemplo, (758, 6, 778, 30).
- **`chevron`**: o rectángulo da frecha, que apunta abaixo: `chevron` de alto, 4 a 100 %, e o dobre de ancho, no medio do botón. No exemplo, (764, 16, 772, 20). A barra do programa debúxaa coma a cruz do ×: dúas liñas de `stroke` píxeles, en chanzos dun píxel, desde as dúas esquinas de arriba, a 45 graos, ata o bordo de abaixo. Nun cadrado, crúzanse; nun rectángulo dúas veces máis ancho ca alto, xúntanse no medio do bordo de abaixo, a calquera escala: 10 × 5 a 125 % (120 DPI), e 12 × 6 a 150 % (144 DPI).

### Os botóns de desprazamento: `scroll_button`, `scroll_highlight` e `scroll_chevron`

Desde a subfase 2.8: o ‹ e o ›, que desprazan a franxa cara á primeira tab e cara á última, aos seus dous lados, só mentres as tabs non caben ([ADR 0025](../adr/0025-moitas-tabs-e-os-indicadores.md), decisión 2). Son coma o ▾: de 24 píxeles de ancho, tan altos coma o +, co mesmo resalte, e cunha frecha no medio, que apunta ao seu lado. Coma no Windows Terminal e en Firefox, quedan sempre no mesmo sitio, e o + e o ▾ van despois do ›.

```rust
pub fn scroll_button(&self, side: Side) -> Option<Rect> {
    if !self.overflow {
        return None;
    }
    let strip = self.strip();
    let width = coord(self.scroll_button.into());
    let left = match side {
        Side::Left => strip.left - width,
        Side::Right => strip.right,
    };
    let top = coord(self.top().into());
    Some(Rect::new(
        left,
        top,
        left + width,
        top + coord(self.new.into()),
    ))
}

pub fn scroll_highlight(&self, side: Side) -> Option<Rect> {
    self.scroll_button(side)
        .map(|button| centered_size(button, self.scroll_highlight, self.new_highlight))
}

pub fn scroll_chevron(&self, side: Side) -> Option<Rect> {
    self.scroll_button(side)
        .map(|button| centered_size(button, self.chevron, 2 * self.chevron))
}
```

- **`scroll_button`**: o botón que despraza as tabs cara a `side`, ou `None` mentres caben. Vai pegado á franxa, sen separación, o ‹ á súa esquerda e o › á súa dereita, e mide `scroll_button` de ancho, 24 a 100 %; coma o +, empeza `gap` píxeles por debaixo do bordo de arriba da fila das tabs ([`top`](#a-franxa-e-as-tabs-strip-start-tab-e-tab_area)), e mide `new` de alto. En 983 píxeles, (4, 4, 28, 32) e (716, 4, 740, 32); coa banda de 8, o ‹ en (4, 12, 28, 40). Ata a 2.8, sempre desde `gap`. Para o rato, coma os outros botóns da barra, ocupa todo o alto da fila das tabs, e desde a 2.9 non a banda ([`part_at`](#o-que-hai-debaixo-dun-punto-part_at)).
- **`scroll_highlight`**: o rectángulo que resalta o punteiro, no medio do botón, coma o do ▾: `scroll_highlight` de ancho, 20 a 100 %, e tan alto coma o resalte do +, 24. No ‹, (6, 6, 26, 30). A barra do programa non o debuxa mentres a franxa non se pode desprazar cara a ese lado ([`can_scroll`](#canto-se-despraza-max_scroll-e-can_scroll)): o botón é entón máis escuro, e o punteiro non o resalta.
- **`scroll_chevron`**: o rectángulo da frecha, `chevron` de ancho, 4 a 100 %, e o dobre de alto, no medio do botón: (14, 14, 18, 22) no ‹ e (726, 14, 730, 22) no ›. É a frecha do ▾ xirada: a barra do programa debuxa dúas liñas de `stroke` píxeles, en chanzos dun píxel, desde as dúas esquinas do lado contrario ata o medio do lado do botón, onde se xuntan a calquera escala: 5 × 10 a 125 % e 6 × 12 a 150 %.
- Os tres dan `None` mentres as tabs caben: a barra do programa non os debuxa nin os marca.

### Os botóns da xanela: `CaptionButton`, `caption`, `captions`, `glyph` e `glyph_offset`

Desde a subfase 2.9: os botóns da xanela, minimizar, maximizar ou restaurar e pechar, á dereita da barra, coma no Windows 11 e no Windows Terminal ([ADR 0027](../adr/0027-a-barra-de-tabs-coma-barra-de-titulo.md), decisión 3). Debúxaos a barra do programa, que os fai actuar cando o botón esquerdo sobe enriba do mesmo no que baixou, coma o + e o × (decisión 4; [Código › meiga › `tabbar.rs`](meiga.md#cratesmeigasrctabbarrs)); para Windows, son os botóns da súa xanela ([`ui/titlebar.rs`](#cratesmeiga-coresrcuititlebarrs)). Ata a 2.8, os botóns da xanela eran os da barra de título de Windows, enriba da barra de tabs.

```rust
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CaptionButton {
    Minimize,
    Maximize,
    Close,
}

impl CaptionButton {
    pub const ALL: [Self; 3] = [Self::Minimize, Self::Maximize, Self::Close];

    const fn on_the_right(self) -> u32 {
        match self {
            Self::Minimize => 2,
            Self::Maximize => 1,
            Self::Close => 0,
        }
    }
}
```

**`CaptionButton`** é un botón da xanela:

| Variante | Que é |
|---|---|
| `Minimize` | Minimiza a xanela. |
| `Maximize` | Maximiza a xanela ou, se xa está maximizada, restáuraa: un só botón, co glifo dun cadrado ou, maximizada, dos dous do de restaurar. |
| `Close` | Pecha a xanela. |

- **`ALL`**: os tres, de esquerda a dereita. `part_at` percórreos para ver se un punto está nun, e a barra do programa, para debuxalos.
- **`on_the_right`**, privada: cantos botóns da xanela van á dereita deste, 2, 1 ou 0. `caption` mide desde o bordo dereito da barra.
- É `Copy`, coma `Part` e `Side`: un byte. `Part::Caption` leva un. O programa convérteos nas súas accións, `TabAction::Minimize`, `TabAction::ToggleMaximize` e `TabAction::CloseWindow`, que lle piden a Windows que minimice, maximice ou restaure a xanela (`WM_SYSCOMMAND`), ou que a peche (`WM_CLOSE`), coma os seus botóns ([Código › meiga › `tabs.rs`](meiga.md#cratesmeigasrctabsrs)).

```rust
pub fn caption(&self, button: CaptionButton) -> Rect {
    let width = i64::from(self.caption);
    let right = i64::from(self.width) - i64::from(button.on_the_right()) * width;
    Rect::new(
        clamped(right - width),
        0,
        clamped(right),
        coord(self.height().into()),
    )
}

pub fn captions(&self) -> Rect {
    self.caption(CaptionButton::Minimize)
        .union(self.caption(CaptionButton::Close))
}

pub fn glyph(&self, button: CaptionButton) -> Rect {
    centered(self.caption(button), self.glyph)
}

pub const fn glyph_offset(&self) -> u32 {
    self.glyph_offset
}
```

- **`caption`**: o botón `button` da xanela, de `caption` píxeles de ancho, 46 a 100 %, contados desde o bordo dereito da barra, con tantos botóns á súa dereita coma di `on_the_right`; e do alto da barra enteira, desde o seu bordo de arriba, tamén por riba da banda, ata o de abaixo (decisión 3). No exemplo, o de minimizar en (845, 0, 891, 32), o de maximizar en (891, 0, 937, 32) e o de pechar en (937, 0, 983, 32); coa banda de 8, o de pechar en (937, 0, 983, 40); e a 150 % (144 DPI), nunha xanela de 2195 píxeles, de 69 × 48: o de minimizar en (1988, 0, 2057, 48) e o de pechar en (2126, 0, 2195, 48).
  - Non dependen das tabs nin da franxa: sempre no mesmo sitio, contra o bordo dereito da barra e o de arriba. Maximizada, o último píxel da dereita na fila de arriba da pantalla é o botón de pechar, coma no Windows 11: o punteiro levado contra a esquina cae nel.
  - As contas fanse en `i64`, e [`clamped`](#os-axudantes-scale-coord-signed-clamped-centered-e-centered_size) convérteas en coordenadas: nunha xanela máis estreita ca os tres botóns, que coa barra de título de meiga Windows non deixa, os da esquerda empezan en negativo.
- **`captions`**: os tres xuntos, desde o de minimizar ata o de pechar (`union`): no exemplo, (845, 0, 983, 32). A barra do programa marca este rectángulo cando a xanela gaña ou perde o foco, que só cambia a cor dos tres glifos (decisión 8).
- **`glyph`**: o cadrado do glifo do botón, de `glyph` píxeles, 10 a 100 %, no seu medio ([`centered`](#os-axudantes-scale-coord-signed-clamped-centered-e-centered_size)): no exemplo, o de minimizar en (863, 11, 873, 21) e o de pechar en (955, 11, 965, 21); coa banda de 8, o de pechar baixa á metade da barra máis alta, (955, 15, 965, 25); e a 150 %, de 15, en (2153, 16, 2168, 31). A barra do programa debuxa nel, con liñas de `stroke` píxeles e as esquinas cadradas, unha liña horizontal no medio para minimizar, o contorno do cadrado para maximizar, dous cadrados para restaurar, e as dúas diagonais da × para pechar, coma a cruz do × das tabs.
- **`glyph_offset`**: canto vai o cadrado de atrás do glifo de restaurar desde o de diante, tantos píxeles cara arriba coma cara á dereita: 2 a 100 %, e 3 a 150 %. O de diante vai na esquina de abaixo á esquerda do cadrado do glifo.

### Canto se despraza: `max_scroll` e `can_scroll`

```rust
pub fn max_scroll(&self) -> u32 {
    if !self.overflow {
        return 0;
    }
    let row = (self.tabs as u64).saturating_mul(self.tab_width.into());
    u32::try_from(row.saturating_sub(self.viewport().into())).unwrap_or(u32::MAX)
}

pub fn can_scroll(&self, side: Side) -> bool {
    match side {
        Side::Left => self.scroll > 0,
        Side::Right => self.scroll < self.max_scroll(),
    }
}
```

- **`max_scroll`**, desde a 2.8: ata onde se pode desprazar a franxa, o que mide a fila das tabs, `tabs` × `tab_width`, menos o ancho da franxa. Con oito tabs en 983 píxeles (ata a 2.8, en 800), a fila mide 800, e 800 − 688 = 112; con trinta, 3000 − 688 = 2312, e entón a última tab vai de 616 a 716, e remata onde empeza o ›. Mentres caben, 0. As contas fanse en `u64`, saturando, e un resultado que non cabe nun `u32` queda en `u32::MAX`. Nunha xanela máis estreita ca os botóns, a franxa mide 0, e o máximo é a fila enteira: con dúas tabs, 200.
- **`can_scroll`**, desde a 2.8: se a franxa se pode desprazar cara a `side`: cara á esquerda mentres oculta algo á súa esquerda, e cara á dereita mentres non chega ao máximo. Mentres as tabs caben, cara a ningún. Con oito tabs sen desprazar, só cara á dereita. A barra do programa debuxa así cada botón, máis escuro e sen resalte cando non pode ([ADR 0025](../adr/0025-moitas-tabs-e-os-indicadores.md), decisión 2), e deixa de repetir o clic ao chegar ao final (decisión 4).
- O desprazamento queda sempre entre 0 e `max_scroll`, porque só o cambian `scroll_to`, que o limita, e `fit`, que o recorta; salvo mentres as tabs manteñen o seu ancho despois dun peche co rato, cando pode quedar por riba ([`keep_widths`](#as-tabs-no-seu-sitio-ao-pechar-co-rato-keep_widths)), e `can_scroll(Side::Right)` é falso.

### Desprazar a franxa: `scroll_by`, `step`, `reveal` e `scroll_to`

Desde a subfase 2.8, tres formas de desprazar a franxa, unha para cada cousa que a despraza ([ADR 0025](../adr/0025-moitas-tabs-e-os-indicadores.md), decisións 3, 4 e 5): uns píxeles, para a roda; unha tab, para os botóns; e o mínimo para que se vexa enteira unha tab, para a que se mostra. As tres din se a franxa se moveu, e mentres as tabs caben non fan nada, porque `max_scroll` é 0.

```rust
pub fn scroll_by(&mut self, pixels: i64) -> bool {
    let target = i64::from(self.scroll).saturating_add(pixels).max(0);
    self.scroll_to(target.unsigned_abs())
}
```

- **`scroll_by`**: despraza a franxa `pixels` píxeles cara á última tab ou, se son negativos, cara á primeira, ata onde pode: o desprazamento novo non baixa de 0 (`max(0)`), e `scroll_to` non o deixa pasar de `max_scroll`. `unsigned_abs` dá o mesmo número nun `u64`, porque xa non é negativo. Con oito tabs, desde 0: +50 deixa a primeira tab en (−22, 4, 78, 32); −80, a franxa en 0, e non en −30; −1, xa en 0, non a move; e +1000, no 112. A barra do programa despraza así a franxa coa roda, o ancho dunha tab por cada paso, 120, e as partes dun paso en píxeles (decisión 3): ao contrario ca os botóns, a roda non deixa as tabs aliñadas coa franxa.

```rust
pub fn step(&mut self, side: Side) -> bool {
    if self.tab_width == 0 {
        return false;
    }
    let width = u64::from(self.tab_width);
    let scroll = u64::from(self.scroll);
    let target = match side {
        // The first tab that is all in the strip, and the one before.
        Side::Left => scroll.div_ceil(width).saturating_sub(1) * width,
        Side::Right => {
            let viewport = u64::from(self.viewport());
            // The first tab that is not all in the strip.
            let hidden = (scroll + viewport) / width;
            ((hidden + 1) * width).saturating_sub(viewport)
        }
    };
    self.scroll_to(target)
}
```

**`step`** despraza a franxa unha tab cara a `side`, coma os botóns: ata que a primeira tab que oculta por ese lado, aínda que sexa en parte, queda enteira nela, coma en Firefox ([ADR 0025](../adr/0025-moitas-tabs-e-os-indicadores.md), decisión 4). Na fila, a tab en `i` vai de `i` × `width` a (`i` + 1) × `width`, e a franxa mostra o que vai de `scroll` a `scroll` + `viewport`:

- **Cara á esquerda**, `scroll.div_ceil(width)`, a división redondeada cara arriba, é a primeira tab que empeza dentro da franxa, a primeira enteira nela; a anterior é a primeira que a franxa oculta á esquerda, aínda que sexa en parte, e a franxa desprázase ata o seu bordo esquerdo. Con oito tabs, desde 112: 112 / 100 cara arriba é 2, a terceira tab, e a franxa vai ao 100, onde empeza a segunda, que cortaba. Desde 100, a segunda xa está enteira, e a franxa vai ao 0. Desde 0 non oculta nada: `saturating_sub(1)` deixa a tab 0, e `step` di que non se moveu.
- **Cara á dereita**, (`scroll` + `viewport`) / `width`, a división redondeada cara abaixo, é a primeira tab que non está enteira na franxa: a que corta á dereita ou, se non corta ningunha, a primeira que oculta. A franxa desprázase ata que o bordo dereito desa tab chega ao seu, (`hidden` + 1) × `width` − `viewport`. Desde 0: 688 / 100 é 6, a sétima tab, de 600 a 700, e a franxa vai ao 700 − 688 = 12, coa sétima enteira, en (616, 4, 716, 32). Desde 12: 700 / 100 é 7, a oitava, e a franxa vai ao 800 − 688 = 112, o final. Desde 112, xa non pode, e `scroll_to` deixa o 112.
- Cada clic deixa enteira unha tab máis do seu lado, e a do outro lado pode quedar cortada, porque a franxa, de 688, non mide un número enteiro de tabs. O botón do Windows Terminal move 50 píxeles, media tab, e deixa tabs cortadas nos dous lados; o da barra nova de Chrome, o ancho da franxa menos 32 DIP, o mínimo dunha tab que non é a activa ([ADR 0025 › Alternativas consideradas](../adr/0025-moitas-tabs-e-os-indicadores.md#alternativas-consideradas)).
- Con `tab_width` 0 non despraza nada, en vez de dividir entre 0. Desde a 2.8 ese caso non chega: `fit` non deixa ningunha tab máis estreita ca `min_tab`, que é polo menos 1.

```rust
pub fn reveal(&mut self, index: usize) -> bool {
    if !self.overflow || index >= self.tabs {
        return false;
    }
    let width = u64::from(self.tab_width);
    let left = (index as u64).saturating_mul(width);
    let right = left.saturating_add(width);
    let scroll = u64::from(self.scroll);
    let viewport = u64::from(self.viewport());
    let target = if left < scroll {
        left
    } else if right > scroll + viewport {
        right.saturating_sub(viewport).min(left)
    } else {
        return false;
    };
    self.scroll_to(target)
}
```

**`reveal`** despraza a franxa o mínimo para que a tab en `index` quede enteira nela: a tab que se mostra está sempre á vista ([ADR 0025](../adr/0025-moitas-tabs-e-os-indicadores.md), decisión 5).

1. Mentres as tabs caben, vense todas, e non fai nada; tampouco cun índice sen tab.
2. Se a tab empeza antes da franxa, a franxa desprázase ata o seu bordo esquerdo.
3. Se remata despois, ata que o seu bordo dereito chega ao da franxa, `right − viewport`; pero non máis alá do seu bordo esquerdo (`min(left)`): nunha franxa máis estreita ca unha tab, vese o seu principio, onde vai o título.
4. Se xa está enteira, non se move.

Así a franxa móvese o mínimo, e as outras tabs quedan onde se vían. Con oito tabs, desde 112, a primeira leva a franxa ao 0; desde 0, a sétima, ao 12; a sexta, de 500 a 600, xa está enteira, e non a move; e a oitava, ao 112. A barra do programa chámaa antes do fotograma no que a tab que se mostra ten que ir á vista, salvo mentres as tabs manteñen o seu ancho ([`keep_widths`](#as-tabs-no-seu-sitio-ao-pechar-co-rato-keep_widths)), e antes de abrir co teclado o menú dunha tab, que vai debaixo dela ([Código › meiga › `tabbar.rs`](meiga.md#cratesmeigasrctabbarrs)).

```rust
fn scroll_to(&mut self, target: u64) -> bool {
    let scroll = u32::try_from(target.min(self.max_scroll().into())).unwrap_or(0);
    mem::replace(&mut self.scroll, scroll) != scroll
}
```

- **`scroll_to`**, privada: despraza a franxa a `target`, ou ata onde pode: como moito ata `max_scroll`, e, porque `target` é un `u64`, polo menos ata 0. Os tres métodos acaban nela. `mem::replace` garda o desprazamento novo e dá o vello: se son distintos, a franxa moveuse. O mínimo cun `u32` sempre cabe nun, así que `try_from` non falla, e `unwrap_or(0)` só evita un `unwrap`.
- `use std::mem;`, ao principio do arquivo desde a 2.8, trae o módulo `mem` da biblioteca estándar, para `mem::replace`.

### As tabs no seu sitio ao pechar co rato: `keep_widths`

Desde a subfase 2.8 ([ADR 0025](../adr/0025-moitas-tabs-e-os-indicadores.md), decisión 6). Cando o × ou o botón do medio pecha unha tab co punteiro na barra, as outras quedan co seu ancho, e a franxa co seu desprazamento: a tab da dereita ocupa o sitio da que pechou, co seu × debaixo do punteiro, e pódense pechar varias seguidas sen mover o rato, coma no Windows Terminal.

```rust
pub fn keep_widths(&mut self, tabs: usize) {
    self.tabs = tabs;
}
```

- **`keep_widths`**: a barra ten agora `tabs` tabs, menos, porque se pecharon co punteiro nela. Só cambia o número de tabs: sen `fit`, `tab_width`, `overflow` e `scroll` quedan coma estaban, e o que as tabs deixan baleiro á dereita queda así ata que `resize` lles dá outra vez o seu ancho.
- **Mentres caben**, o + segue a última tab: con cinco tabs de 147 en 983 píxeles, e unha pechada, as catro quedan de 147, e o + vai en (596, 4, 624, 32), onde ía a quinta; `resize` dálles despois 184.
- **Mentres non caben**, a franxa queda co seu desprazamento, aínda que coas tabs que quedan non puidese chegar tan lonxe: con dez tabs e a franxa ao final, 312, e unha pechada, a franxa queda no 312, aínda que o máximo para nove é 212, e xa non se pode desprazar cara á dereita. O sitio da última queda baleiro: no 715, `part_at` dá `Empty`, e no 615, `Tab(8)`, a nova última, que quedou onde estaba. `resize` leva despois a franxa ao 212.
- A barra do programa chámao cando se pecha unha tab que non era a última, co punteiro na barra, se a pechou o × ou o botón do medio ou se as tabs xa mantiñan o seu ancho. Chama a `resize`, e as tabs recuperan o seu ancho, cando o punteiro sae da barra ou da xanela, cando un menú colle o rato, ao desprazar a franxa, ao cambiar o tamaño ou a escala da xanela, ao abrir tabs e ao pechar a última; e mentres tanto, a tab que se mostra non vai á vista ([Código › meiga › `tabbar.rs`](meiga.md#cratesmeigasrctabbarrs)).
- O seu comentario di que as tabs son menos, pero non o comproba: con máis, pasarían do sitio da barra. A barra do programa só o chama cando se pecha unha.

### O que hai debaixo dun punto: `part_at`

```rust
pub fn part_at(&self, at: Point) -> Option<Part> {
    if !self.bar().contains_point(at) {
        return None;
    }
    let across = |rect: Rect| (rect.left..rect.right).contains(&at.x);
    for button in CaptionButton::ALL {
        if across(self.caption(button)) {
            return Some(Part::Caption(button));
        }
    }
    if at.y < coord(self.band.into()) {
        return Some(Part::Empty);
    }
    if across(self.new_tab()) {
        return Some(Part::New);
    }
    if across(self.profiles()) {
        return Some(Part::Profiles);
    }
    for side in [Side::Left, Side::Right] {
        if self.scroll_button(side).is_some_and(across) {
            return Some(Part::Scroll(side));
        }
    }
    let from_start = i64::from(at.x) - self.start();
    if self.tab_width == 0 || !across(self.strip()) || from_start < 0 {
        return Some(Part::Empty);
    }
    // `from_start` is not negative.
    let index =
        usize::try_from(from_start as u64 / u64::from(self.tab_width)).unwrap_or(usize::MAX);
    if index >= self.tabs {
        return Some(Part::Empty);
    }
    let on_close = self
        .close(index)
        .is_some_and(|close| close.contains_point(at));
    Some(if on_close {
        Part::Close(index)
    } else {
        Part::Tab(index)
    })
}
```

**`part_at`** dá a parte da barra no píxel `at`, en coordenadas da área cliente, ou `None` fóra da barra. Desde a 2.9, os botóns da xanela ocupan todo o alto da barra, tamén a banda, e as tabs e os outros botóns, todo o alto da fila das tabs, debaixo da banda; ata a 2.8, as tabs e os botóns ocupaban todo o alto da barra, que era a fila:

1. Fóra da barra, `None`.
2. Desde a 2.9, nas columnas dun botón da xanela, `Caption` co botón, a calquera altura da barra, tamén na banda.
3. Desde a 2.9, na banda, fóra deles, `Empty`: tamén por riba das tabs e dos outros botóns, que non chegan a ela.
4. Nas columnas do +, `New`; desde a 2.5, nas do ▾, `Profiles`; e, desde a 2.8, nas dos botóns de desprazamento, se os hai, `Scroll` co seu lado. A *closure* `across` mira só a columna, cun rango de enteiros que inclúe `left` e non `right`, coma os rectángulos: a columna 756, a primeira do ▾, xa non é do +. `is_some_and(across)` só a mira se o botón existe.
5. Fóra da franxa, ou á esquerda da primeira tab, `Empty`. Mentres as tabs caben, a franxa é a barra enteira; mentres non, quedan fóra a separación á esquerda do ‹, a que vai entre o › e o +, e, á dereita do ▾, a separación e, desde a 2.9, o sitio para arrastrar a xanela. Con `tab_width` 0, tamén `Empty`, pero desde a 2.8 ningunha tab mide 0 ([`fit`](#a-barra-e-o-ancho-das-tabs-bar-room-viewport-e-fit)).
6. A tab sae dunha división: a distancia desde onde empeza a primeira tab ([`start`](#a-franxa-e-as-tabs-strip-start-tab-e-tab_area)), entre o ancho dunha. Mentres a franxa está desprazada, a primeira empeza á esquerda dela, e a este paso só chegan os puntos dentro da franxa: das tabs cortadas, só conta a súa parte nela. Despois da última, entre ela e o + ou á dereita do ▾, `Empty`; e, mentres as tabs manteñen o seu ancho despois dun peche co rato, tamén no sitio que quedou baleiro.
7. Na tab, `Close` se o punto está no seu × (`contains_point`); se non, `Tab`, tamén por riba e por debaixo do × e na separación de enriba.

- **Unha división, e non unha busca:** as tabs son todas iguais, así que a tab dun punto sae dunha conta, sexan dez ou cen.
- O comentario lembra por que `from_start` non é negativo: o paso 5 xa descartou os puntos á esquerda da primeira tab, así que o `as u64` non perde nada. Ata a 2.7, a distancia era `at.x - gap`. O cociente cabe nun `usize` nos sistemas de 64 bits; se non coubese, `usize::MAX` daría `Empty`.
- **Os botóns da xanela, antes ca a banda:** chegan ata o bordo de arriba da barra, e alí gañan á banda ([ADR 0027](../adr/0027-a-barra-de-tabs-coma-barra-de-titulo.md), decisión 2). Se a banda se mirase antes, as filas de arriba dos botóns serían `Empty`, e para Windows o bordo de arriba, que redimensiona a xanela. Os outros botóns míranse despois da banda, só pola súa columna: debaixo dela, toda a fila das tabs é súa.
- Os botóns de desprazamento míranse antes ca a franxa: as tabs cortadas seguen por debaixo deles, pero alí o punteiro está no botón.
- A barra do programa converte a resposta no que resalta o punteiro, coa tab desa posición ([`Part`](#part-e-side)); fóra da barra, `None` non resalta nada. Desde a 2.9, os botóns da xanela só se resaltan se a xanela ten a barra de título de meiga: sen ela, o seu sitio queda baleiro. Se cada evento do rato vai á barra ou ao terminal, decídeo antes o programa ([ADR 0020](../adr/0020-a-barra-de-tabs.md), decisión 7).
- Desde a 2.9, [`TitleBar::hit`](#o-que-é-cada-punto-hit) dálle a Windows a mesma resposta noutros termos: un botón da xanela é ese botón; `Empty`, na banda, o bordo de arriba, e fóra dela, a barra de título, que arrastra a xanela; e o resto, a área cliente.

No exemplo, `the_part_under_the_pointer` comproba:

| Punto | Parte | Por que |
|---|---|---|
| (4, 20) | `Tab(0)` | O primeiro píxel da primeira tab. |
| (100, 1) | `Tab(0)` | Na separación de enriba, por riba da tab que se debuxa: para o rato, a tab ocupa todo o alto da fila. |
| (220, 10) | `Close(0)` | No ×. |
| (220, 5) e (220, 30) | `Tab(0)` | Por riba e por debaixo do ×, que ocupa as filas de 6 a 29. |
| (243, 20) e (244, 20) | `Tab(0)` e `Tab(1)` | O último píxel da primeira tab e o primeiro da segunda. |
| (723, 20) e (725, 20) | `Tab(2)` e `Empty` | O último píxel da terceira, e a separación entre ela e o +. |
| (728, 0) e (755, 31) | `New` | As esquinas do +, tamén por riba do cadrado que se debuxa. |
| (756, 20) e (779, 0) | `Profiles` | O primeiro píxel do ▾, xusto despois do último do +, e a súa esquina de arriba á dereita, tamén por riba do que se debuxa. |
| (780, 20) | `Empty` | Á dereita do ▾. |
| (844, 0) | `Empty` | Desde a 2.9: o último píxel do sitio para arrastrar a xanela, na fila de arriba. |
| (845, 0) e (890, 31) | `Caption(CaptionButton::Minimize)` | Desde a 2.9: as esquinas do botón de minimizar, de arriba á esquerda e de abaixo á dereita. |
| (891, 20) e (936, 20) | `Caption(CaptionButton::Maximize)` | O primeiro e o último píxel do de maximizar. |
| (937, 20) e (982, 31) | `Caption(CaptionButton::Close)` | O primeiro píxel do de pechar, e o último da barra, abaixo á dereita. |
| (2, 20) | `Empty` | Á esquerda da primeira tab. |
| (100, 32), (−1, 10) e (983, 10) | `None` | Debaixo da barra, á esquerda e á dereita dela. Ata a 2.8, á dereita era (800, 10). |

E, desde a 2.8, con oito tabs e a franxa desprazada 50 píxeles, `the_part_under_the_pointer_while_the_tabs_scroll` comproba:

| Punto | Parte | Por que |
|---|---|---|
| (3, 20) | `Empty` | A separación á esquerda do ‹. |
| (4, 20) e (27, 0) | `Scroll(Side::Left)` | O primeiro píxel do ‹ e a súa esquina de arriba á dereita, tamén por riba do que se debuxa. |
| (28, 31) e (77, 31) | `Tab(0)` | A primeira tab vai de 28 − 50 = −22 a 78: só a súa parte na franxa, desde o 28, está debaixo do punteiro. |
| (78, 31) | `Tab(1)` | O primeiro píxel da segunda. |
| (50, 20) | `Close(0)` | O × da primeira, desde 78 − 4 − 24 = 50. |
| (715, 31) | `Tab(7)` | O último píxel da franxa, na oitava tab, que vai de 678 a 778, e que a franxa corta. |
| (716, 20) e (739, 31) | `Scroll(Side::Right)` | O primeiro e o último píxel do ›. |
| (740, 20) | `Empty` | A separación entre o › e o +. |
| (744, 20) | `New` | O primeiro píxel do +, despois do ›. |
| (795, 20) e (796, 20) | `Profiles` e `Empty` | O último píxel do ▾, e a separación á súa dereita. |
| (844, 31) | `Empty` | Desde a 2.9: o último píxel do sitio para arrastrar a xanela, que vai de 800 a 845, xusto os 45 do mínimo. |
| (845, 20) | `Caption(CaptionButton::Minimize)` | Desde a 2.9: o primeiro píxel dos botóns da xanela, que non se moven coa franxa. |

E, desde a 2.9, coa banda de 8 e tres tabs, `a_band_above_the_tabs_moves_them_down_and_the_buttons_of_the_window_cover_it` comproba:

| Punto | Parte | Por que |
|---|---|---|
| (100, 7) | `Empty` | A última fila da banda, por riba da primeira tab: a banda non é da tab. |
| (100, 8) | `Tab(0)` | A primeira fila debaixo da banda, na separación de enriba da tab, que xa é dela. |
| (730, 7) e (730, 8) | `Empty` e `New` | O mesmo no +: a última fila da banda, e a primeira debaixo dela. |
| (960, 0) | `Caption(CaptionButton::Close)` | O botón de pechar a xanela, na fila de arriba da banda: gaña a ela. |
| (845, 7) | `Caption(CaptionButton::Minimize)` | O primeiro píxel do de minimizar, na última fila da banda. |
| (100, 40) | `None` | Debaixo da barra, que coa banda mide 40. |

### Os axudantes: `scale`, `coord`, `signed`, `clamped`, `centered` e `centered_size`

```rust
use super::{coord, scale};
```

```rust
fn signed(pixels: u64) -> i64 {
    i64::try_from(pixels).unwrap_or(i64::MAX)
}

fn clamped(value: i64) -> i32 {
    i32::try_from(value).unwrap_or(if value < 0 { i32::MIN } else { i32::MAX })
}

fn centered(rect: Rect, side: u32) -> Rect {
    centered_size(rect, side, side)
}

fn centered_size(rect: Rect, width: u32, height: u32) -> Rect {
    let (width, height) = (coord(width.into()), coord(height.into()));
    let left = rect.left + (coord(rect.width().into()) - width) / 2;
    let top = rect.top + (coord(rect.height().into()) - height) / 2;
    Rect::new(left, top, left + width, top + height)
}
```

- **`scale`** e **`coord`** chegan do módulo pai, `ui`, con `use super::{coord, scale};`: ata a 2.3 estaban neste arquivo, e desde a 2.4 están en `ui/mod.rs`, porque os usan tamén os menús ([Os axudantes](#os-axudantes-base_dpi-scale-e-coord)). `scale` dá as medidas da barra ao DPI da xanela, en [`new`](#crear-e-cambiar-new-row_at-min_width-set_band-e-resize) e en `row_at`, ata a 2.8 `height_at`; e `coord` converte en coordenadas os números de píxeles das súas contas, que así non desbordan con ningún índice de tab, agás as sumas e as restas de [`close`](#o-botón-de-pechar-close-close_highlight-e-cross), que non saturan (ver `clamped`).
- **`signed`**, desde a 2.8: unha distancia na fila das tabs, en `u64`, coma un `i64`, ou `i64::MAX` se non cabe, para sumala a onde empeza a primeira tab, que pode ser negativo ([`tab`](#a-franxa-e-as-tabs-strip-start-tab-e-tab_area)).
- **`clamped`**, desde a 2.8: unha posición na fila das tabs, en `i64`, coma unha coordenada da xanela, un `i32`, ou, se non cabe, `i32::MIN` ou `i32::MAX`, segundo o seu signo: unha tab oculta moi lonxe da franxa queda moi lonxe, polo seu lado. `coord`, que só coñece os positivos, levaría á dereita unha tab oculta á esquerda. Ningunha tab chega tan lonxe: faría falta unha fila de máis de 2 147 483 647 píxeles, uns 21 millóns de tabs de 100. Cunha tab así, ou cun índice así, as sumas e as restas de [`close`](#o-botón-de-pechar-close-close_highlight-e-cross) (`tab.left + gap` e `tab.right - gap`), que non saturan, desbordarían, coma xa pasaba ata a 2.7 cun índice tan grande que `coord` deixase a tab en `i32::MAX`. Desde a 2.9, converte tamén os bordos dos botóns da xanela, que [`caption`](#os-botóns-da-xanela-captionbutton-caption-captions-glyph-e-glyph_offset) calcula en `i64` desde o ancho da barra, e que nunha xanela máis estreita ca eles son negativos.
- **`centered_size`**, desde a 2.5: o rectángulo de `width` por `height` píxeles no medio de `rect`. Se o que sobra é impar, o píxel de máis queda á dereita e debaixo: o signo + de 11 no + de 28 deixa 8 píxeles á esquerda e por riba, e 9 á dereita e por debaixo. Úsano `profiles_highlight` e `chevron` e, desde a 2.8, `scroll_highlight` e `scroll_chevron`, que non son cadrados.
- **`centered`**: o cadrado de `side` píxeles no medio de `rect`, coma un `centered_size` cos dous lados iguais; ata a 2.4 facía as contas el. Úsano `close_highlight`, `cross`, `new_highlight` e `plus` e, desde a 2.9, `glyph`: o glifo de 10 no botón de 46 × 32 deixa 18 píxeles a cada lado, 11 por riba e 11 por debaixo.

**Probas.** Trece, con coordenadas exactas, catro delas desde a 2.8 e catro desde a 2.9. `three` fai a barra do exemplo: tres tabs en 983 píxeles a 96 DPI, sen banda, coma nunha xanela maximizada; ata a 2.8, en 800 píxeles. Os 183 píxeles de máis son os do sitio para arrastrar a xanela e os dos seus botóns, así que nas probas que viñan de antes as tabs e os botóns da barra quedan onde estaban, e só cambian os anchos das xanelas. Desde a 2.8, varias usan a mesma xanela con máis tabs: cinco ou sete, que caben, e oito ou dez, que non; e, desde a 2.9, unha usa a mesma barra coa banda de 8.

| Proba | Que comproba |
|---|---|
| `tabs_are_as_wide_as_in_windows_terminal_and_the_buttons_follow_them` | A barra do exemplo, de 32 de alto, sen banda, coa fila de 32 tamén con `row_at(96)` (ata a 2.8, `height_at(96)`), en (0, 0, 983, 32) (ata a 2.8, (0, 0, 800, 32)); as tabs de 240, a primeira en (4, 4, 244, 32) e a terceira en (484, 4, 724, 32); o + en (728, 4, 756, 32), co resalte en (730, 6, 754, 30) e o signo de 11 píxeles en (736, 12, 747, 23), para que os dous brazos dunha liña de 1 teñan 5; e, desde a 2.5, unido a el, o ▾ en (756, 4, 780, 32), co resalte en (758, 6, 778, 30) e a frecha en (764, 16, 772, 20). Ata a 2.4 chamábase `tabs_are_as_wide_as_in_windows_terminal_and_the_button_follows_them`. |
| `the_buttons_of_the_window_are_on_its_right_as_high_as_the_bar` | Desde a 2.9. Na barra do exemplo, os botóns da xanela, de 46 e do alto da barra: minimizar en (845, 0, 891, 32), maximizar en (891, 0, 937, 32) e pechar en (937, 0, 983, 32), os tres en (845, 0, 983, 32); os glifos, de 10, no seu medio, o de pechar en (955, 11, 965, 21) e o de minimizar en (863, 11, 873, 21), cun desprazamento de 2 para o de restaurar e liñas de 1; e, entre o ▾ e o botón de minimizar, 65 píxeles: a separación de 4 e 61 para arrastrar a xanela. A 150 % (144 DPI), nunha xanela de 2195 píxeles con dúas tabs, os botóns miden 69 × 48: pechar en (2126, 0, 2195, 48) e minimizar en (1988, 0, 2057, 48); o glifo de pechar, de 15, en (2153, 16, 2168, 31); e o desprazamento do de restaurar, 3. |
| `a_tab_has_its_title_and_then_its_close_button` | O × da primeira tab en (216, 6, 240, 30), o seu resalte en (218, 8, 238, 28) e a cruz en (224, 14, 232, 22); o título, (12, 4, 216, 32), ata o ×; o separador da segunda tab, na súa primeira columna, (244, 10, 245, 26), e ningún na primeira nin despois da última; e o que cambia co aspecto da primeira, (4, 0, 245, 32). |
| `a_band_above_the_tabs_moves_them_down_and_the_buttons_of_the_window_cover_it` | Desde a 2.9. A mesma barra coa banda de 8, coma unha xanela restaurada a 96 DPI: `band()` dá 8, e `height()`, 40; a barra en (0, 0, 983, 40), coas tabs de 240. O que vai debaixo da banda baixa 8, coma no [exemplo restaurado](#cratesmeiga-coresrcuitabbarrs): a primeira tab en (4, 12, 244, 40), o + en (728, 12, 756, 40), o ▾ en (756, 12, 780, 40), o × da primeira en (216, 14, 240, 38) e o separador da segunda en (244, 18, 245, 34); o que cambia co aspecto da primeira, (4, 8, 245, 40), non colle a banda; e o botón de pechar a xanela chega ata arriba, (937, 0, 983, 40), co glifo en (955, 15, 965, 25). Os puntos da terceira táboa de [`part_at`](#o-que-hai-debaixo-dun-punto-part_at). Con oito tabs, o ‹ baixa a (4, 12, 28, 40), a franxa, (28, 0, 716, 40), cobre a banda, e o que cambia co aspecto da sétima é (628, 8, 716, 40). E `set_band(0)` deixa a disposición igual ca a de `three`, sen banda. |
| `there_is_always_room_to_drag_the_window_between_the_buttons` | Desde a 2.9. A 96, 120, 144, 168 e 192 DPI, en anchos desde o da barra máis estreita (`min_width`) ata 600 píxeles máis, de 7 en 7, e en 1920, 2560, 3840 e 7680, con 1 a 40 tabs, sen banda: dezaoito mil disposicións. En todas, entre a separación da dereita do ▾ e o botón de minimizar quedan polo menos os 45 píxeles de `DRAG`, escalados ao DPI ([ADR 0027](../adr/0027-a-barra-de-tabs-coma-barra-de-titulo.md), decisión 5). E na xanela do exemplo, sete tabs, que caben, deixan 46, un píxel máis, o que sobra do seu ancho; e oito, que non caben, xusto 45. |
| `the_narrowest_bar_shows_a_whole_tab_between_the_buttons_that_scroll` | Desde a 2.9. A barra máis estreita, `min_width`, mide 395 a 96 DPI, 3 × 4 + 28 + 24 + 45 + 3 × 46 + 2 × 24 + 100; 495 a 120; e 593 a 144. Nela, dúas tabs non caben, e a franxa mide xusto 100, unha tab enteira das máis estreitas; nunha píxel máis estreita, menos. E unha soa tab cabe, sen franxa, de 148. |
| `tabs_that_do_not_fit_shrink_alike_down_to_100_and_keep_their_close_button` | Na mesma barra, sete tabs miden 105 (983 − 3 × 4 − 28 − 24 − 45 − 3 × 46 = 736, entre 7; ata a 2.8, 800 − 3 × 4 − 28 − 24) e caben: a franxa é a barra enteira, sen botóns de desprazamento, e non se despraza; o + vai en (743, 4, 771, 32), e o ▾, en (771, 4, 795, 32); e a última tab ten o seu ×, en (711, 6, 735, 30), co título en (642, 4, 711, 32). E o límite: nunha xanela de 947 píxeles, as sete miden 100, 700 entre 7, e caben; nunha de 946, xa non. Ata a 2.8, 764 e 763. Ata a 2.7 chamábase `tabs_that_do_not_fit_shrink_alike_and_narrow_ones_have_no_close_button`, e comprobaba dez tabs de 73, co + en (738, 4, 766, 32), trinta de 24, sen ×, e tabs de 0 nunha xanela de 20 píxeles. |
| `the_narrowest_tabs_that_do_not_fit_go_in_a_strip_between_two_buttons` | Desde a 2.8. Na mesma barra, oito tabs non caben: miden 100, e van na franxa, de 736 − 2 × 24 = 688, en (28, 0, 716, 32), entre o ‹, en (4, 4, 28, 32), e o ›, en (716, 4, 740, 32); o resalte do ‹ en (6, 6, 26, 30), e as frechas, de 4 × 8, no medio dos botóns, en (14, 14, 18, 22) e (726, 14, 730, 22). O + e o ▾ van despois do ›, en (744, 4, 772, 32) e (772, 4, 796, 32): na 2.8, ao final da barra. A primeira tab vai en (28, 4, 128, 32), e a oitava en (728, 4, 828, 32); o que cambia co aspecto da sétima é (628, 0, 716, 32), cortado pola franxa, e co da oitava, nada. A franxa pode desprazarse ata 112, 800 − 688: sen desprazar, só cara á dereita. E nunha xanela de 20 píxeles, máis estreita ca os botóns, a franxa está baleira, e o máximo é a fila enteira, 200. |
| `the_strip_scrolls_by_tabs_by_pixels_and_to_show_a_tab` | Desde a 2.8, con oito tabs en 983 píxeles (ata a 2.8, en 800). Cara á dereita, unha tab leva a franxa ao 12, coa sétima, que cortaba, enteira en (616, 4, 716, 32), e outra ao 112, o final, onde xa non pode; cara á esquerda, ao 100, coa segunda, que cortaba, e ao 0, onde xa non pode. En píxeles, +50 deixa a primeira tab en (−22, 4, 78, 32); −80, a franxa en 0; −1 xa non a move; e +1000 lévaa ao 112. Para mostrar unha tab, o mínimo: a primeira, ao 0; a sétima, ao 12; a sexta, que xa está enteira, nada; e a oitava, ao 112. Unha xanela de 883 píxeles deixa o desprazamento no 112, e unha de 1033, no 62, o seu máximo: 1033 − 247 − 48 = 738, dos 800 da fila. E nunha de 1183, as oito caben, de 117, (1183 − 247) / 8, e a franxa non se despraza, nin para mostrar a oitava. Nunha de 333, a franxa mide 333 − 247 − 48 = 38, menos ca unha tab: para mostrar a segunda, vai ao 100, co lado esquerdo da tab, e non ao 162, co dereito. Ata a 2.8, as xanelas medían 183 píxeles menos, 700, 850, 1000 e 150, e o que non é das tabs, 64. |
| `tabs_keep_their_width_and_the_strip_its_scroll_when_some_close_in_place` | Desde a 2.8. Cinco tabs, de 147 en 983 píxeles (ata a 2.8, en 800): con `keep_widths(4)`, as catro seguen de 147, e o + vai despois da última, en (596, 4, 624, 32); con `resize`, miden 184. Dez tabs, coa franxa ao final, no 312: con `keep_widths(9)`, seguen sen caber, a franxa queda no 312, e non se pode desprazar cara á dereita; no 715, onde estaba a última tab, `Empty`, e no 615, na novena, `Tab(8)`. Con `resize`, a franxa vai ao 212. |
| `the_sizes_follow_the_scale` | A 144 DPI (150 %), nunha xanela de 2195 píxeles con dúas tabs (ata a 2.8, de 1920): a barra de 48 de alto, as tabs de 360, a segunda en (366, 6, 726, 48), o seu × en (684, 9, 720, 45), o + en (732, 6, 774, 48), o ▾ en (774, 6, 810, 48), co resalte en (777, 9, 807, 45) e a frecha en (786, 24, 798, 30), o trazo de 2 e o signo + de 16: 15, e un máis para brazos de 7 aos dous lados dunha liña de 2. A 120 DPI (125 %), o signo de 13, a frecha de 10 × 5, o dobre de ancha ca alta coma a calquera escala, e a fila das tabs de 40, con `row_at(120)`. Desde a 2.8, a 150 %, a tab máis estreita mide 150 e os botóns de desprazamento, 36: o sitio das tabs é 2195 − 3 × 6 − 42 − 36 − 68 − 3 × 69 = 1824 píxeles, desde a 2.9 co sitio para arrastrar a xanela, de 68, e os seus botóns, de 69 (ata a 2.8, 1920 − 3 × 6 − 42 − 36); caben doce, de 152, e trece non: miden 150, e van na franxa, de 1824 − 2 × 36, en (42, 0, 1794, 48), co ‹ en (6, 6, 42, 48). |
| `the_part_under_the_pointer_while_the_tabs_scroll` | Desde a 2.8: os puntos da segunda táboa de [`part_at`](#o-que-hai-debaixo-dun-punto-part_at), con oito tabs e a franxa desprazada 50 píxeles; desde a 2.9, tamén o sitio para arrastrar a xanela e o botón de minimizar. |
| `the_part_under_the_pointer` | Os puntos da primeira táboa de [`part_at`](#o-que-hai-debaixo-dun-punto-part_at); desde a 2.9, tamén os dos botóns da xanela. |

Desde a 2.9, catro copias do código cun erro feito a propósito cada unha fallan estas probas, cada unha coma se esperaba:

| Copia | O que estraga | Onde falla |
|---|---|---|
| `u-captions-below-band` | `part_at` mira a banda antes ca os botóns da xanela. | En `a_band_above_…`: no botón de pechar, en (960, 0), dentro da banda, dá `Empty`. |
| `u-caption-row-only` | Os botóns da xanela só ocupan a fila das tabs, debaixo da banda. | En `a_band_above_…`: o botón de pechar empeza no 8, e non no 0. |
| `u-min-width-no-scroll` | `min_width` non conta os botóns de desprazamento. | En `the_narrowest_bar_…`: 347, e non 395. |
| `u-drag-narrower` | O sitio para arrastrar a xanela mide 44 a 100 %. | En varias, porque o sitio entra en todas as contas do ancho das tabs: entre elas `a_band_above_…`, coa franxa un píxel máis ancha, (28, 0, 717, 40); `tabs_keep_their_width_…`, co desprazamento máximo no 311; `there_is_always_room_…`, con 44 píxeles para arrastrar con oito tabs; e `the_narrowest_bar_…`, con 394. |

## `crates/meiga-core/src/ui/titlebar.rs`

Desde a subfase 2.9: a barra de tabs coma barra de título da xanela ([ADR 0027](../adr/0027-a-barra-de-tabs-coma-barra-de-titulo.md), [Arquitectura › As tabs na barra de título](../arquitectura.md#as-tabs-na-barra-de-título), [Tecnoloxías › O marco da xanela e a barra de título](../tecnoloxias.md#62-o-marco-da-xanela-e-a-barra-de-título)). Windows pregúntalle á xanela que hai en cada punto dela (`WM_NCHITTEST`), e fai el o que fai esa parte. Este módulo dá a resposta para a barra de meiga, coa disposición da barra ([`ui/tabbar.rs`](#cratesmeiga-coresrcuitabbarrs)) e sabendo se hai un menú de meiga aberto; di onde se abre o menú da xanela con Alt+Espazo; e canto pode encoller a xanela. Coma o resto de `ui`, non usa Windows: só fai contas, e próbase con coordenadas exactas.

**Quen o usa:**

- **O programa**, ao crear as tabs e despois de cada evento da xanela, fai un `TitleBar` coa disposición da súa barra e se hai un menú aberto, e dállo á xanela, se esta ten a barra de título de meiga (`Tabs::title_bar`; [Código › meiga › `tabs.rs`](meiga.md#cratesmeigasrctabsrs)): calquera evento pode mover as partes da barra, ou abrir ou pechar un menú.
- **meiga-win** gárdao (`Window::set_title_bar`), e responde con el `WM_NCHITTEST` (`hit_test`) sen chamar ao programa, que pode estar ocupado con outro evento: primeiro pregúntalle a `DefWindowProcW`, que di onde está o marco de Windows, nos lados e abaixo, e só onde este di que é área cliente pregunta a `hit`, e converte o `Hit` na constante de Windows. En `WM_GETMINMAXINFO` pídelle a `min_client` o tamaño mínimo da área cliente ([ADR 0027](../adr/0027-a-barra-de-tabs-coma-barra-de-titulo.md), decisións 9 e 12). E, cando Windows lle pide á xanela o seu menú con Alt+Espazo, pídelle a `menu_corner` onde abrilo (decisión 6; [Código › meiga-win › `window.rs`](meiga-win.md#cratesmeiga-winsrcwindowrs)).
- Sen a barra de título de meiga, nos modos `--bench` ou se Windows non deixa poñela, a xanela non atende `WM_NCHITTEST` nin `WM_GETMINMAXINFO`, deixa a Windows o menú de Alt+Espazo, e o programa non lle dá ningún `TitleBar`.

```rust
//! The bar of tabs as the title bar of the window (ADR 0027): what each
//! point of it is for Windows, which drags the window from it, resizes the
//! window from its top edge, and knows where its buttons are; how small the
//! window can be; and where Alt+Space opens the menu of the window.
//!
//! Windows asks a window what is at a point with `WM_NCHITTEST`, and does
//! what that part does by itself: the title bar drags the window and, with a
//! double click, maximizes or restores it; the top edge resizes it; and the
//! button to maximize opens the snap layouts of Windows 11 under the
//! pointer. meiga answers for its bar with a [`Hit`]; it draws the bar and
//! its buttons itself, and the clicks on them go to the bar, and not to
//! Windows.

use super::tabbar::{CaptionButton, Layout, Part};
use crate::geometry::{Point, Size};
```

- O comentario do módulo (`//!`) di para que é, coma o parágrafo de enriba, e o que fai Windows con cada resposta: a barra de título arrastra a xanela, e un dobre clic maximízaa ou restáuraa; o bordo de arriba redimensiona a xanela; e, co punteiro parado no botón de maximizar, Windows 11 mostra os seus *snap layouts*. Os botóns da xanela son para Windows os seus botóns, pero, coma di o final do comentario, os seus clics non os fai Windows: meiga-win pásallos á barra do programa, que os debuxa e os fai actuar ao soltar o botón esquerdo enriba do mesmo ([ADR 0027](../adr/0027-a-barra-de-tabs-coma-barra-de-titulo.md), decisión 4).
- `use` trae de `tabbar` a disposición, as partes da barra e os botóns da xanela, e de `geometry`, `Point` e `Size`. Non trae `scale` nin `coord`: o módulo non ten medidas propias.

### `Hit`

```rust
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Hit {
    Client,
    Caption,
    Top,
    TopLeft,
    Minimize,
    Maximize,
    Close,
}
```

O que é un punto da xanela para Windows: a resposta de `WM_NCHITTEST` para el, sen os números de Windows. meiga-core non usa as APIs de Windows: `hit_test`, en meiga-win, dálle a cada variante a súa constante, cun `match` que o compilador obriga a cubrir enteiro.

| Variante | Que é | Para Windows | Que fai Windows |
|---|---|---|---|
| `Client` | Unha parte de meiga: as tabs, os seus ×, o +, o ▾, os botóns de desprazamento e o terminal; e, cun menú de meiga aberto, tamén as partes libres da barra. | `HTCLIENT` | Nada: as mensaxes do rato van á xanela, coma sempre. |
| `Caption` | As partes libres da barra, sen un menú aberto: os ocos e o sitio para arrastrar a xanela. | `HTCAPTION` | Arrastra a xanela, levala a un bordo da pantalla colócaa alí, e un dobre clic maximízaa ou restáuraa (decisión 5). O botón dereito non abre nada: o menú da xanela ábreo meiga-win (decisión 6). |
| `Top` | A banda enriba das tabs, coa xanela restaurada. | `HTTOP` | Redimensiona a xanela desde o seu bordo de arriba (decisión 2). |
| `TopLeft` | O principio da banda, xunto ao lado esquerdo da xanela. | `HTTOPLEFT` | Redimensiona a xanela en diagonal, desde a esquina de arriba á esquerda. |
| `Minimize`, `Maximize` e `Close` | Os botóns da xanela. | `HTMINBUTTON`, `HTMAXBUTTON` e `HTCLOSE` | Co punteiro parado no de maximizar, mostra os *snap layouts* de Windows 11. Os clics neles non os fai Windows: van á barra do programa (decisión 4). |

- Non hai ningunha variante para o marco dos lados e de abaixo: dío `DefWindowProcW`, antes de preguntar a `hit`. Tampouco unha esquina de arriba á dereita: alí van os botóns da xanela, que chegan ata o bordo de arriba e gañan á banda.
- É `Copy`, coma `Part`: un byte, que vai por valor.

### `TitleBar` e `new`

```rust
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TitleBar {
    layout: Layout,
    menu: bool,
}
```

| Campo | Que é |
|---|---|
| `layout` | A disposición da barra de tabs ([`Layout`](#layout-os-campos)) despois do último evento: onde van as tabs, os botóns da barra, a franxa e os botóns da xanela, e a banda, se a xanela está restaurada. |
| `menu` | Se hai un menú de meiga aberto, o dunha tab ou o dos perfís: entón un clic en calquera sitio só o pecha, así que a barra non arrastra a xanela ([ADR 0027](../adr/0027-a-barra-de-tabs-coma-barra-de-titulo.md), decisión 10, que concreta a decisión 8 do [ADR 0021](../adr/0021-os-menus-e-o-menu-da-tab.md)). |

```rust
pub const fn new(layout: Layout, menu: bool) -> Self {
    Self { layout, menu }
}
```

- **`new`**: a barra de título da barra de tabs `layout`, cun menú de meiga aberto ou non. Só garda os dous valores: as contas fanse en `hit`, cando Windows pregunta. É `const fn`.
- **Unha copia, e non unha referencia.** `TitleBar` é `Copy`: a disposición, que xa o é, e un `bool`. A xanela gárdao nunha `Cell<Option<TitleBar>>`, que o programa cambia cunha referencia compartida á xanela, e cópiao cada vez que Windows pregunta.
- **Por que despois de cada evento.** Windows pregunta con cada movemento do punteiro, tamén mentres o programa atende outro evento, e entón a xanela non pode preguntarlle ao programa. Pero a barra só cambia cando o programa atende un evento, así que, ao acabar cada un, a xanela xa ten a de agora ([ADR 0027](../adr/0027-a-barra-de-tabs-coma-barra-de-titulo.md), decisión 12).

### O que é cada punto: `hit`

```rust
pub fn hit(&self, at: Point, corner: i32) -> Hit {
    match self.layout.part_at(at) {
        Some(Part::Caption(button)) => match button {
            CaptionButton::Minimize => Hit::Minimize,
            CaptionButton::Maximize => Hit::Maximize,
            CaptionButton::Close => Hit::Close,
        },
        Some(Part::Empty) if at.y < i32::try_from(self.layout.band()).unwrap_or(i32::MAX) => {
            if at.x < corner {
                Hit::TopLeft
            } else {
                Hit::Top
            }
        }
        Some(Part::Empty) if !self.menu => Hit::Caption,
        _ => Hit::Client,
    }
}
```

**`hit`** dá o que é para Windows o punto `at`, en coordenadas da área cliente, sendo os primeiros `corner` píxeles desde o seu lado esquerdo a esquina do bordo de arriba, que redimensiona a xanela tamén cara á esquerda. Pregúntalle á disposición que parte da barra hai alí ([`part_at`](#o-que-hai-debaixo-dun-punto-part_at)), e o `match` mira os casos nesta orde:

1. **Un botón da xanela** (`Part::Caption`): ese botón, a calquera altura da barra, tamén na banda, e tamén cun menú aberto.
2. **A banda** (`Part::Empty` por riba de `band`): `TopLeft` nos primeiros `corner` píxeles, e `Top` no resto, tamén cun menú aberto. `part_at` dá `Empty` en toda a banda fóra dos botóns da xanela, tamén por riba das tabs e dos outros botóns. A comparación é `<`: a fila `band`, a primeira debaixo da banda, xa non é dela. Coa xanela maximizada, a banda mide 0, e ningún punto está nela.
3. **As partes libres da barra** (o resto de `Part::Empty`), sen un menú aberto: `Caption`, a barra de título. Son os ocos, á esquerda da primeira tab ou do ‹, entre a última e o + ou entre o › e o +, e á dereita do ▾, onde empeza o sitio para arrastrar a xanela; e o sitio que deixan as tabs pechadas co rato mentres as outras manteñen o seu ancho ([`keep_widths`](#as-tabs-no-seu-sitio-ao-pechar-co-rato-keep_widths)).
4. **O resto**, `Client`: as tabs, os seus ×, o +, o ▾ e os botóns de desprazamento; as partes libres, cun menú aberto; e, fóra da barra (`None`), o terminal.

**Por que esta orde.** Cada garda (`if`) do `match` só se mira se os casos de antes non casaron, e cada caso vai antes ca o que lle podería quitar os seus puntos:

- **Os botóns da xanela, primeiro**, porque son sempre os seus botóns: chegan ata o bordo de arriba e gañan á banda ([ADR 0027](../adr/0027-a-barra-de-tabs-coma-barra-de-titulo.md), decisión 2), e cun menú aberto seguen sendo botóns, cos *snap layouts* (decisión 10). Coa xanela maximizada, a fila de arriba da pantalla é tamén a dos botóns, e o último píxel da dereita é o de pechar: o punteiro levado contra a esquina da pantalla cae nel, coma di a lei de Fitts, coma no Windows 11.
- **A banda, antes ca o menú**, porque redimensiona a xanela tamén cun menú aberto, e empezar a redimensionar pecha o menú ([ADR 0021](../adr/0021-os-menus-e-o-menu-da-tab.md), decisión 9).
- **As partes libres, despois do menú**: cun menú aberto, un clic nelas é un clic fóra del, que só o pecha, e non empeza a arrastrar nin maximiza a xanela.

- **A esquina** non escala: son 19 píxeles desde o lado esquerdo da xanela a calquera DPI, coma co marco de Windows (decisión 2). Pero `hit` mide desde o lado esquerdo da área cliente, e entre os dous está o marco da esquerda, así que meiga-win pásalle 19 menos o grosor dese marco: a 96 DPI, cun marco de 8, 11. As probas usan 10.
- `i32::try_from(…).unwrap_or(i32::MAX)` converte a banda nunha coordenada, coma faría [`coord`](#os-axudantes-base_dpi-scale-e-coord), que este módulo non trae: ningunha banda chega a `i32::MAX`.

Coa barra do [exemplo de `tabbar.rs`](#cratesmeiga-coresrcuitabbarrs), tres tabs en 983 píxeles a 96 DPI, e a esquina en 10, `maximized_the_bar_drags_the_window_and_its_top_row_belongs_to_what_is_below` comproba, sen banda:

| Punto | Para Windows | Por que |
|---|---|---|
| (845, 0) | `Minimize` | O primeiro píxel do botón de minimizar, na fila de arriba. |
| (891, 31) | `Maximize` | A esquina de abaixo á esquerda do de maximizar. |
| (982, 0) | `Close` | O último píxel da dereita, na fila de arriba: coa xanela maximizada, a esquina da pantalla. |
| (800, 10) e (844, 0) | `Caption` | O sitio para arrastrar a xanela, tamén na fila de arriba, ata o botón de minimizar. |
| (2, 20) | `Caption` | A separación á esquerda da primeira tab. |
| (5, 0), (100, 0) e (220, 10) | `Client` | A primeira tab, tamén na fila de arriba, e o seu ×. |
| (730, 0) e (760, 20) | `Client` | O +, tamén na fila de arriba, e o ▾. |
| (100, 32) e (983, 10) | `Client` | Debaixo da barra, o terminal, e á súa dereita, fóra dela. |

Coa banda de 8, coma coa xanela restaurada a 96 DPI, `restored_the_band_resizes_the_window_but_not_under_its_buttons` comproba:

| Punto | Para Windows | Por que |
|---|---|---|
| (0, 0) e (9, 7) | `TopLeft` | A esquina da banda: os seus primeiros 10 píxeles, ata a súa última fila. |
| (10, 0), (100, 7) e (800, 0) | `Top` | O resto da banda, tamén por riba da primeira tab e do sitio para arrastrar a xanela. |
| (845, 0) e (982, 0) | `Minimize` e `Close` | Os botóns da xanela, na fila de arriba, dentro da banda. |
| (100, 8) | `Client` | A primeira tab, na primeira fila debaixo da banda. |
| (800, 8) e (2, 8) | `Caption` | As partes libres, na primeira fila debaixo da banda. |
| (100, 40) | `Client` | O terminal, debaixo da barra, que coa banda mide 40. |

E coa banda e un menú de meiga aberto, `with_a_menu_open_the_bar_is_for_meiga_but_the_edge_and_the_buttons_are_not` comproba:

| Punto | Para Windows | Por que |
|---|---|---|
| (800, 10) e (2, 20) | `Client` | As partes libres: un clic nelas só pecha o menú. |
| (100, 7) e (0, 0) | `Top` e `TopLeft` | A banda e a súa esquina seguen redimensionando a xanela. |
| (960, 0) | `Close` | O botón de pechar segue sendo o seu botón. |
| (100, 20) | `Client` | A primeira tab, coma sempre. |

### Onde se abre o menú da xanela co teclado: `menu_corner`

```rust
pub fn menu_corner(&self) -> Point {
    Point::new(0, i32::try_from(self.layout.height()).unwrap_or(i32::MAX))
}
```

**`menu_corner`** dá onde vai a esquina de arriba á esquerda do menú da xanela cando o abre o teclado, con Alt+Espazo, en coordenadas da área cliente: debaixo da barra enteira, a banda máis a fila das tabs ([`height`](#layout-os-campos)), e no lado esquerdo da área cliente, onde empeza o terminal ([ADR 0027](../adr/0027-a-barra-de-tabs-coma-barra-de-titulo.md), decisión 6). É onde se abren os menús de meiga co teclado, debaixo do que os abre ([ADR 0021](../adr/0021-os-menus-e-o-menu-da-tab.md), decisión 4).

- **Quen o usa:** meiga-win, cando Windows lle pide á xanela o seu menú con Alt+Espazo (`WM_SYSCOMMAND`, con `SC_KEYMENU` e o espazo). Pasa o punto a coordenadas da pantalla (`ClientToScreen`), e abre alí o menú de Windows, sen ningún elemento resaltado (`keyboard_menu`; [Código › meiga-win › `window.rs`](meiga-win.md#cratesmeiga-winsrcwindowrs)).
- **Debaixo da barra enteira, coa banda.** Coa xanela restaurada, a banda vai enriba das tabs, así que un menú que empezase debaixo só da fila das tabs, á altura da fila, taparía a parte de abaixo delas. Maximizada, a banda mide 0, e o menú vai debaixo da fila.
- **E non onde o abriría Windows,** debaixo de onde estaría a súa barra de título, enriba das tabs, e no seu modo menú, no que a tecla seguinte a Esc non lle chega ao programa da tab; nin na esquina de arriba á esquerda da xanela, coma o Windows Terminal ([ADR 0027 › Contexto](../adr/0027-a-barra-de-tabs-coma-barra-de-titulo.md#contexto)).
- **O lado esquerdo da área cliente**, coma no resto do módulo: o marco da esquerda queda fóra, e o menú empeza na primeira columna do terminal.
- `i32::try_from(…).unwrap_or(i32::MAX)` converte o alto nunha coordenada, coma a banda en [`hit`](#o-que-é-cada-punto-hit): ningunha barra chega a `i32::MAX`.

### O tamaño mínimo: `min_client`

```rust
pub fn min_client(dpi: u32, band: u32) -> Size {
    Size::new(
        Layout::min_width(dpi),
        band.saturating_add(Layout::row_at(dpi)),
    )
}
```

**`min_client`** dá o tamaño mínimo da área cliente, a parte da xanela que debuxa meiga, a `dpi` e cunha banda de `band` píxeles enriba das tabs ([ADR 0027](../adr/0027-a-barra-de-tabs-coma-barra-de-titulo.md), decisión 9):

- **De ancho, a barra máis estreita** ([`min_width`](#crear-e-cambiar-new-row_at-min_width-set_band-e-resize)): os botóns da xanela, o sitio para arrastrala, o + e o ▾, as tres separacións, os dous botóns de desprazamento e, entre eles, unha tab enteira das máis estreitas: 395 píxeles a 96 DPI, 495 a 120 e 593 a 144. Nunha xanela máis estreita, con tabs que non caben, a franxa non mostraría enteira ningunha. É menos ca os 460 do Windows Terminal e ca os 500 que pide como moito Microsoft Learn para os *snap layouts* ([ADR 0027 › Contexto](../adr/0027-a-barra-de-tabs-coma-barra-de-titulo.md#contexto)).
- **De alto, só a barra**, sen o terminal, coma no Windows Terminal: a banda máis a fila das tabs ([`row_at`](#crear-e-cambiar-new-row_at-min_width-set_band-e-resize)). Nunha xanela tan baixa, o terminal non ten ningunha fila.
- `saturating_add` non desborda con ningunha banda. O tamaño é un [`Size`](#cratesmeiga-coresrcgeometryrs), en píxeles á escala da xanela.
- **Quen o usa:** `WM_GETMINMAXINFO`, en meiga-win, pídeo co DPI da xanela e cunha banda do grosor do seu marco, e súmalle o marco de Windows: dúas veces de ancho, polos dous lados, e unha de alto, polo de abaixo; arriba, a banda xa vai dentro da área cliente. Con ese tamaño, Windows non deixa a xanela máis pequena, nin co rato nin con `SetWindowPos` ([Código › meiga-win › `window.rs`](meiga-win.md#cratesmeiga-winsrcwindowrs)).

**Probas.** Cinco, con coordenadas exactas. `bar` fai a barra de título das probas coa barra do [exemplo de `tabbar.rs`](#cratesmeiga-coresrcuitabbarrs), tres tabs en 983 píxeles a 96 DPI, coa banda que se lle pida, e cun menú aberto ou non. As tres de `hit` usan unha esquina de 10 píxeles.

| Proba | Que comproba |
|---|---|
| `maximized_the_bar_drags_the_window_and_its_top_row_belongs_to_what_is_below` | Sen banda, coma coa xanela maximizada, a primeira táboa de [`hit`](#o-que-é-cada-punto-hit): os botóns da xanela, tamén na fila de arriba, ata o último píxel da dereita; as partes libres, a barra de título, tamén na fila de arriba; e as tabs, os seus botóns, o terminal e o que queda fóra da barra, a área cliente. Ningún punto é o bordo de arriba: a fila de arriba da pantalla é do que hai debaixo dela (decisión 2). |
| `restored_the_band_resizes_the_window_but_not_under_its_buttons` | Coa banda de 8, coma coa xanela restaurada a 96 DPI, a segunda táboa: a banda é o bordo de arriba, e os seus primeiros 10 píxeles, a súa esquina, ata a súa última fila; os botóns da xanela, tamén na banda; e, desde a fila 8, a tab é a área cliente, e as partes libres, a barra de título. |
| `with_a_menu_open_the_bar_is_for_meiga_but_the_edge_and_the_buttons_are_not` | Coa banda e un menú aberto, a terceira táboa: as partes libres son a área cliente, pero a banda segue sendo o bordo de arriba, coa súa esquina, e o botón de pechar, o seu botón. |
| `the_keyboard_opens_the_menu_of_the_window_below_the_bar` | `menu_corner`: (0, 40) coa banda de 8, debaixo da barra enteira, e non (0, 32), debaixo só da fila; (0, 32) sen banda, coma maximizada; e (0, 59) a 144 DPI, cunha tab en 1475 píxeles, coa banda de 11 e a fila de 48. |
| `the_window_is_at_least_as_big_as_the_narrowest_bar` | `min_client`: 395 × 40 a 96 DPI, coa banda de 8; 495 × 40 a 120 DPI, sen banda, coa fila de 40; e 593 × 59 a 144 DPI, coa banda de 11 e a fila de 48. |

Oito copias do código cun erro feito a propósito cada unha fallan estas probas, cada unha coma se esperaba:

| Copia | O que estraga | Onde falla |
|---|---|---|
| `u-no-top-left` | A esquina da banda é `Top`. | En `restored_…` e en `with_a_menu_open_…`, no punto (0, 0). |
| `u-empty-is-client` | As partes libres da barra son `Client`, tamén sen menú. | En `maximized_…`, en (800, 10), e en `restored_…`, en (800, 8). |
| `u-menu-ignored` | Un menú aberto non cambia nada: as partes libres seguen sendo `Caption`. | En `with_a_menu_open_…`, en (800, 10). |
| `u-menu-takes-band` | Cun menú aberto, tamén a banda é `Client`. | En `with_a_menu_open_…`, en (100, 7). |
| `u-band-inclusive` | A banda colle tamén a fila de debaixo dela (`<=`). | En `maximized_…`, en (844, 0): sen banda, a copia fai da fila de arriba o bordo de arriba; e en `restored_…`, en (800, 8). |
| `u-maximize-hit` | O botón de maximizar é `Minimize`, e Windows non mostraría os *snap layouts*. | En `maximized_…`, en (891, 31). |
| `u-min-client-no-band` | `min_client` non suma a banda. | En `the_window_is_at_least_as_big_as_the_narrowest_bar`: 395 × 32, e non 395 × 40. |
| `u-menu-corner-row-only` | `menu_corner` dá o alto da fila das tabs, sen a banda. | En `the_keyboard_opens_the_menu_of_the_window_below_the_bar`: (0, 32), e non (0, 40). |

Que meiga-win lle dá a Windows a constante de cada `Hit`, coa esquina medida desde o lado da xanela, e o tamaño mínimo co marco, compróbao `the_title_bar_of_meiga_tells_windows_what_each_of_its_parts_is`, en [Código › meiga-win › `window.rs`](meiga-win.md#cratesmeiga-winsrcwindowrs); e, coa xanela de meiga na pantalla, restaurada, maximizada e cun menú aberto, `check-titlebar.ps1` ([Ferramentas › `check-titlebar.ps1`](ferramentas.md#toolscheck-titlebarps1)).

## `crates/meiga-core/src/ui/menu.rs`

Desde a subfase 2.4: un menú sobre a xanela, sen Windows ([ADR 0021](../adr/0021-os-menus-e-o-menu-da-tab.md), [Arquitectura › Os menús](../arquitectura.md#os-menús), [Tecnoloxías › Os menús: Windows, WinUI e Chrome](../tecnoloxias.md#56-os-menús-windows-winui-e-chrome)). Ten dúas partes:

- **`Menu`**: as entradas, elementos e liñas que os agrupan, e o resalte, que moven o punteiro e o teclado;
- **`Layout`**: onde van o menú e cada entrada, para un DPI, un tamaño das celas do texto e un sitio onde abrilo, dentro da xanela; e que entrada hai debaixo dun punto.

Vale para calquera menú, cunha acción en cada elemento, do tipo que escolla quen o usa ([ADR 0021](../adr/0021-os-menus-e-o-menu-da-tab.md), decisión 1): o primeiro foi o menú dunha tab; o segundo, desde a subfase 2.5, o dos perfís ([ADR 0022](../adr/0022-os-perfis-e-a-configuracion.md), decisión 6); e despois virá o explorador (5.2). Coma a barra, só fai contas, e próbase só, con coordenadas exactas. O menú que se ve e se usa, `MenuView`, está no programa: debúxao con estes rectángulos, leva a el o teclado e o rato, e marca as zonas que cambian ([Código › meiga › `menu.rs`](meiga.md#cratesmeigasrcmenurs)); e o menú da tab e o dos perfís, cos seus elementos e as súas accións, ábreos [`tabs.rs`](meiga.md#cratesmeigasrctabsrs).

**A forma do menú**, coas medidas dos menús de WinUI (`MenuFlyout`), coma o da tab no Windows Terminal, co rato e co teclado, en píxeles a escala 100 % ([ADR 0021](../adr/0021-os-menus-e-o-menu-da-tab.md), decisións 1, 3 e 4):

- un bordo de 1 píxel e, dentro, 2 máis por riba do primeiro elemento e por debaixo do último;
- cada elemento, 32 de alto: o seu resalte, de 28, a 4 píxeles dos lados e a 2 de arriba e de abaixo; e o seu texto, a 11 píxeles dos lados do resalte;
- cada liña, de 1 píxel, de lado a lado dentro do bordo, con 1 por riba e outro por debaixo: 3 de alto;
- tan ancho coma o seu elemento máis longo, co texto contado en celas, e nunca máis ca a xanela; e tan alto coma as súas entradas, aínda que non caiba;
- ábrese nun punto, coa súa esquina de arriba á esquerda nel, ou debaixo dunha parte da xanela, desde o lado esquerdo dela. Se non cabe por ese lado, vai ao outro, e non sae da xanela: un máis alto ca ela córtase por abaixo, e o punteiro non resalta nin escolle o que queda cortado;
- e todas as medidas, escaladas co DPI da xanela, redondeadas e de polo menos un píxel, coma as da barra.

**Un exemplo**, o das probas: o menú dunha tab coma era ata a subfase 2.6, sen *Restart tab*, con *Close other tabs* desactivado, aberto co punteiro en (100, 50), con celas de 7 × 16 píxeles, a 96 DPI (escala 100 %), nunha xanela de 800 × 600:

| Parte | Método | Rectángulo |
|---|---|---|
| O menú | `menu()` | (100, 50, 293, 187) |
| O sitio de *Duplicate tab* | `entry(0)` | (101, 53, 292, 85) |
| O seu resalte | `highlight(0)` | (105, 55, 288, 83) |
| O sitio do seu texto | `text(0)` | (116, 55, 277, 83) |
| O sitio da liña | `entry(1)` | (101, 85, 292, 88) |
| A liña | `line(1)` | (101, 86, 292, 87) |
| O sitio de *Close tab* | `entry(2)` | (101, 88, 292, 120) |
| O de *Close other tabs* | `entry(3)` | (101, 120, 292, 152) |
| O de *Close tabs to the right* | `entry(4)` | (101, 152, 292, 184) |

Os rectángulos van coma en [`Rect`](#cratesmeiga-coresrcgeometryrs), (`left`, `top`, `right`, `bottom`), sen incluír os dous últimos. O menú mide 193 × 137: de ancho, as 23 celas de 7 píxeles de *Close tabs to the right*, o texto máis longo, e 11 + 4 + 1 a cada lado, 23 × 7 + 2 × (1 + 4 + 11); e de alto, catro elementos de 32 e unha liña de 3, e 2 + 1 por riba e por debaixo, 4 × 32 + 3 + 2 × (1 + 2). A 150 % (144 DPI), con celas de 11 píxeles de ancho, o mesmo menú mide 303 × 208: 23 × 11 + 2 × (2 + 6 + 17) de ancho, e 4 × (42 + 2 × 3) + (2 + 2 × 2) + 2 × (2 + 3) de alto.

### As medidas do menú

```rust
const BORDER: u32 = 1;
const PADDING: u32 = 2;
const ITEM: u32 = 28;
const ITEM_SIDE: u32 = 4;
const ITEM_GAP: u32 = 2;
const TEXT_PADDING: u32 = 11;
const LINE: u32 = 1;
const LINE_GAP: u32 = 1;
```

As medidas, en píxeles a escala 100 %, as dos menús de WinUI ([ADR 0021](../adr/0021-os-menus-e-o-menu-da-tab.md), decisión 3). [`Layout::new`](#facer-o-menú-layoutnew) escálaas ao DPI da xanela con [`scale`](#os-axudantes-base_dpi-scale-e-coord), coma as da barra:

| Constante | A 100 % (96 DPI) | A 150 % (144 DPI) | Que é |
|---|---|---|---|
| `BORDER` | 1 | 2 | O grosor do bordo. |
| `PADDING` | 2 | 3 | O espazo entre o bordo e a primeira entrada, e entre a última e o bordo. |
| `ITEM` | 28 | 42 | O alto do resalte dun elemento. |
| `ITEM_SIDE` | 4 | 6 | O espazo entre o bordo e cada lado do resalte, que o resalte non colle. |
| `ITEM_GAP` | 2 | 3 | O espazo por riba e por debaixo do resalte, que tamén é do elemento: o sitio dun elemento mide 28 + 2 × 2 = 32, ou 42 + 2 × 3 = 48. |
| `TEXT_PADDING` | 11 | 17 | O espazo entre os lados do resalte e o texto. |
| `LINE` | 1 | 2 | O grosor da liña dun separador. |
| `LINE_GAP` | 1 | 2 | O espazo por riba e por debaixo da liña: o sitio dunha liña mide 1 + 2 × 1 = 3, ou 2 + 2 × 2 = 6. |

- A 150 %, cada medida multiplícase por 1,5 e redondéase ao máis próximo, cunha metade cara arriba: o bordo, 1,5, sobe a 2, e o espazo do texto, 16,5, a 17.
- Os elementos e as liñas non teñen ancho propio: van de lado a lado dentro do bordo, e o ancho do menú sae do texto máis longo ([`new`](#facer-o-menú-layoutnew)). O texto tampouco ten alto propio: o programa céntrao no alto do resalte.
- Son privadas: fóra do módulo só se ven escaladas, a través de `Layout`.

### `Entry`: os elementos e as liñas

```rust
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Entry<A> {
    Item {
        label: Cow<'static, str>,
        action: A,
        enabled: bool,
        bold: bool,
    },
    Separator,
}
```

Unha entrada dun menú:

| Variante | Que é |
|---|---|
| `Item` | Un elemento: o seu texto (`label`), a acción que pide cando se escolle (`action`), se se pode escoller (`enabled`) e, desde a subfase 2.5, se o seu texto vai en negra (`bold`): a escolla por defecto, coma o perfil que abre o +. Un elemento desactivado vese, co texto máis apagado, pero non se resalta nin se escolle ([ADR 0021](../adr/0021-os-menus-e-o-menu-da-tab.md), decisión 2). |
| `Separator` | Unha liña entre dous grupos de elementos. |

- **A acción** é do tipo `A`, o que escolla quen usa o menú: o menú dunha tab leva en cada elemento unha `TabAction` coa súa tab, o dos perfís, unha coa posición do seu perfil, e as probas, números. O menú non sabe que fai cada elemento: só di cal se escolleu ([Escoller](#escoller-action-e-chosen)).
- **O texto** é un `Cow<'static, str>`, de `std::borrow` (*clone-on-write*), que garda ou un texto fixo do programa, un `&'static str`, coma os do menú da tab, sen copialo, ou un `String`, feito ao abrir o menú, coma os nomes dos perfís. Lese igual nos dous casos, coma un `&str`.
- Deriva `Clone`, `Debug`, `PartialEq` e `Eq`, se a acción os ten; non `Copy`, porque o texto pode ser un `String`.

```rust
pub fn item(label: impl Into<Cow<'static, str>>, action: A, enabled: bool) -> Self {
    Self::Item {
        label: label.into(),
        action,
        enabled,
        bold: false,
    }
}

#[must_use]
pub fn bold(self) -> Self {
    match self {
        Self::Item {
            label,
            action,
            enabled,
            ..
        } => Self::Item {
            label,
            action,
            enabled,
            bold: true,
        },
        Self::Separator => Self::Separator,
    }
}

pub fn label(&self) -> Option<&str> {
    match self {
        Self::Item { label, .. } => Some(label),
        Self::Separator => None,
    }
}

pub const fn is_enabled(&self) -> bool {
    matches!(self, Self::Item { enabled: true, .. })
}
```

- **`item`** fai un elemento, sen negra. `impl Into<Cow<'static, str>>` acepta un `&'static str` ou un `String`, e `into` convérteo nun `Cow`: o menú dunha tab fai así os seus, coma `Entry::item("Close tab", TabAction::CloseTab(id), true)`, e o dos perfís, cos nomes, coma `Entry::item(profile.name.clone(), TabAction::OpenProfile(index), true)`.
- **`bold`**, desde a 2.5: a mesma entrada, co texto en negra se é un elemento; unha liña queda igual. Recibe a entrada, e devolve outra co mesmo texto e a mesma acción, movidos sen copialos; `..` descarta o `bold` de antes. Así se encadea: o menú dos perfís pono no perfil por defecto, `item.bold()`, coma o Windows Terminal ([ADR 0022](../adr/0022-os-perfis-e-a-configuracion.md), decisión 6). `#[must_use]`: o compilador avisa se se chama sen usar a entrada que devolve, que levaría a negra.
- **`label`**: o texto dun elemento, coma un `&str`, ou `None` nunha liña. A disposición mide os textos con el, e o menú do programa debúxaos.
- **`is_enabled`**: se é un elemento que se pode escoller; unha liña, nunca. `matches!` compara a entrada co patrón, un `Item` con `enabled` en `true`, sen movela. É unha `const fn`. A negra non ten un método coma este: o menú do programa le `bold` no patrón co que colle o texto e `enabled` de cada elemento.
- **A negra non cambia a disposición.** O menú do programa debuxa cada carácter na súa cela, coa mesma fonte en negra, así que [`Layout::new`](#facer-o-menú-layoutnew) mide igual un texto en negra e un normal ([Código › meiga › `menu.rs`](meiga.md#cratesmeigasrcmenurs)).

**O menú dunha tab** ten cinco entradas, en inglés, coma o resto da interface: *Duplicate tab*, unha liña, *Close tab*, *Close other tabs*, que só se pode escoller con máis dunha tab, e *Close tabs to the right*, que non se pode escoller na última ([ADR 0021](../adr/0021-os-menus-e-o-menu-da-tab.md), decisión 2). Cada elemento leva unha `TabAction` coa tab do menú, coma `TabAction::Duplicate(id)` ([Código › meiga › `tabs.rs`](meiga.md#cratesmeigasrctabsrs)).

**O menú dos perfís**, desde a 2.5, ten un elemento por perfil visíbel, co seu nome, na orde da lista, todos activados e sen liñas; o perfil por defecto vai en negra ([`Profiles`](#os-perfís-do-menú-profiles-e-merge); [ADR 0022](../adr/0022-os-perfis-e-a-configuracion.md), decisión 6). Cada elemento leva unha `TabAction::OpenProfile` coa posición do seu perfil, e ábrese debaixo dos dous botóns, o + e o ▾ ([`Anchor::Below`](#onde-se-abre-anchor-e-place)).

### `Menu`: as entradas e o resalte

```rust
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Menu<A> {
    entries: Vec<Entry<A>>,
    highlighted: Option<usize>,
}
```

- **`entries`**: as entradas, na orde na que se ven, de arriba abaixo. Non cambian: `Menu` non ten ningún método que as cambie.
- **`highlighted`**: a posición do elemento resaltado, ou `None`. Sempre é un elemento que se pode escoller: só o cambian `hover` e `step`, que nunca resaltan unha liña nin un elemento desactivado, e os campos son privados.
- Os seus métodos piden `A: Copy` (`impl<A: Copy> Menu<A>`), por [`action`](#escoller-action-e-chosen). Deriva o mesmo ca `Entry`.

```rust
pub const fn new(entries: Vec<Entry<A>>) -> Self {
    Self {
        entries,
        highlighted: None,
    }
}

pub fn entries(&self) -> &[Entry<A>] {
    &self.entries
}

pub const fn highlighted(&self) -> Option<usize> {
    self.highlighted
}

pub fn hover(&mut self, index: Option<usize>) -> bool {
    let index = index.filter(|&index| self.can_choose(index));
    self.highlight(index)
}
```

- **`new`**: un menú sen nada resaltado, coma cando o abre o rato: ata que o punteiro pasa por un elemento, non se resalta ningún ([ADR 0021](../adr/0021-os-menus-e-o-menu-da-tab.md), decisión 6). Cando o abre o teclado, o menú do programa resalta despois o primeiro elemento, con [`step(Step::First)`](#o-teclado-step-e-step), para que Intro o escolla e as frechas se movan desde el.
- **`entries`** e **`highlighted`** lense para debuxar o menú: cada entrada, e o resalte no elemento resaltado.
- **`hover`**: o punteiro está sobre a entrada en `index`, ou sobre ningunha (`None`). Se é un elemento que se pode escoller, resáltase; se non, non se resalta nada: o punteiro sobre unha liña ou sobre un elemento desactivado quita o resalte, coma fóra das entradas ou fóra da xanela (decisión 8). `filter` deixa o índice só se se pode escoller. Devolve se o resalte cambiou.

```rust
fn can_choose(&self, index: usize) -> bool {
    self.entries.get(index).is_some_and(Entry::is_enabled)
}

fn highlight(&mut self, index: Option<usize>) -> bool {
    let changed = self.highlighted != index;
    self.highlighted = index;
    changed
}
```

- **`can_choose`**, privada: se a entrada en `index` é un elemento que se pode escoller. Despois da última entrada, `get` dá `None`, e `is_some_and`, `false`: unha posición de máis non resalta nada, e non fai fallar nada.
- **`highlight`**, privada: resalta `index` e di se cambiou. `hover` e `step` devolven esa resposta: o menú do programa só debuxa outra vez os dous elementos, o que perde o resalte e o que o gaña, cando cambia ([ADR 0021](../adr/0021-os-menus-e-o-menu-da-tab.md), decisión 12).

### O teclado: `Step` e `step`

```rust
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Step {
    Next,
    Previous,
    First,
    Last,
}
```

```rust
pub fn step(&mut self, step: Step) -> bool {
    let count = self.entries.len();
    let choosable = |index: &usize| self.can_choose(*index);
    let target = match (step, self.highlighted) {
        (Step::First, _) | (Step::Next, None) => (0..count).find(choosable),
        (Step::Last, _) | (Step::Previous, None) => (0..count).rev().find(choosable),
        (Step::Next, Some(current)) => (current + 1..count).chain(0..=current).find(choosable),
        (Step::Previous, Some(current)) => (0..current)
            .rev()
            .chain((current..count).rev())
            .find(choosable),
    };
    target.is_some() && self.highlight(target)
}
```

**`step`** move o resalte coma pide o teclado, sen parar nas liñas nin nos elementos desactivados, e dando a volta nos extremos ([ADR 0021](../adr/0021-os-menus-e-o-menu-da-tab.md), decisión 7):

| Paso | Tecla | Sen resalte | Co elemento `current` resaltado |
|---|---|---|---|
| `Next` | ↓ | O primeiro elemento que se pode escoller. | O seguinte ou, despois do último, o primeiro. |
| `Previous` | ↑ | O último. | O anterior ou, antes do primeiro, o último. |
| `First` | Inicio | O primeiro. | O primeiro. |
| `Last` | Fin | O último. | O último. |

- Cada caso percorre as posicións nunha orde, e `find` colle a primeira que se pode escoller (`choosable`):
  - `First`, e `Next` sen resalte: desde a primeira entrada (`0..count`);
  - `Last`, e `Previous` sen resalte: desde a última, cara atrás (`rev`);
  - `Next`: desde a seguinte á resaltada ata o final e, despois, desde o principio ata a resaltada (`chain`);
  - `Previous`: desde a anterior ata a primeira e, despois, desde a última ata a resaltada.
- Os percorridos de `Next` e `Previous` acaban na resaltada: se é o único elemento que se pode escoller, atópana a ela, e nada cambia. Son rangos de enteiros encadeados, sen reservar memoria. `find` pásalle a cada posición unha referencia, e por iso a *closure* recibe un `&usize`.
- Devolve se o resalte cambiou, coma `hover`. Sen ningún elemento que se poida escoller, `target` é `None`, e o menú segue sen nada resaltado.
- As teclas convérteas en pasos o menú do programa ([Código › meiga › `menu.rs`](meiga.md#cratesmeigasrcmenurs)). `Step` é `Copy`: un byte, que vai por valor.

### Escoller: `action` e `chosen`

```rust
pub fn action(&self, index: usize) -> Option<A> {
    match self.entries.get(index) {
        Some(Entry::Item {
            action,
            enabled: true,
            ..
        }) => Some(*action),
        _ => None,
    }
}

pub fn chosen(&self) -> Option<A> {
    self.highlighted.and_then(|index| self.action(index))
}
```

- **`action`**: a acción do elemento en `index`, se se pode escoller. O patrón só casa cun `Item` con `enabled` en `true`: unha liña, un elemento desactivado ou unha posición despois da última dan `None`. O menú do programa pídea para o elemento onde sobe o botón: un clic nun desactivado non escolle nada, e o menú segue aberto ([ADR 0021](../adr/0021-os-menus-e-o-menu-da-tab.md), decisión 8).
- **`chosen`**: a acción do elemento resaltado, a que escollen Intro e Espazo (decisión 7); sen resalte, `None`.
- **`*action` copia a acción**, e por iso os métodos de `Menu` piden `A: Copy`: o menú non a perde nin a presta, e quen o usa pode pechalo antes de facela. `TabAction`, a do menú da tab e, desde a 2.5, a do dos perfís, é `Copy`: uns poucos bytes.

### Onde se abre: `Anchor` e `place`

```rust
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Anchor {
    At(Point),
    Below(Rect),
}
```

| Variante | Onde se abre o menú |
|---|---|
| `At(punto)` | Coa súa esquina de arriba á esquerda no punto: onde está o punteiro cando o botón dereito sobe nunha tab ([ADR 0021](../adr/0021-os-menus-e-o-menu-da-tab.md), decisión 5). |
| `Below(rectángulo)` | Debaixo desa parte da xanela, desde o seu lado esquerdo: a tab que se mostra, coma a debuxa a barra ([`tab`](#a-franxa-e-as-tabs-strip-start-tab-e-tab_area)), cando o abren a tecla de menú ou Maiús+F10 (decisión 6); desde a subfase 2.8, a súa parte na franxa, que antes se despraza ata ela ([ADR 0025](../adr/0025-moitas-tabs-e-os-indicadores.md), decisión 5). O menú empeza entón no bordo de abaixo da barra. Desde a subfase 2.5, tamén o menú dos perfís, que abren o ▾ e Ctrl+Maiús+Espazo: debaixo dos dous botóns, o + e o ▾, xuntos, desde o lado esquerdo do + ou, se non cabe, rematando no dereito do ▾ ([ADR 0022](../adr/0022-os-perfis-e-a-configuracion.md), decisión 6). Na barra do [exemplo de `tabbar.rs`](#cratesmeiga-coresrcuitabbarrs), os dous ocupan (728, 4, 780, 32). |

```rust
fn place(anchor: Anchor, size: Size, bounds: Rect) -> Point {
    let (width, height) = (coord(size.width.into()), coord(size.height.into()));
    let (x, y, other_x, other_y) = match anchor {
        Anchor::At(at) => (
            at.x,
            at.y,
            at.x.saturating_sub(width),
            at.y.saturating_sub(height),
        ),
        Anchor::Below(part) => (
            part.left,
            part.bottom,
            part.right.saturating_sub(width),
            part.top.saturating_sub(height),
        ),
    };
    let x = if x.saturating_add(width) > bounds.right {
        other_x
    } else {
        x
    };
    let y = if y.saturating_add(height) > bounds.bottom {
        other_y
    } else {
        y
    };
    Point::new(
        x.min(bounds.right.saturating_sub(width)).max(bounds.left),
        y.min(bounds.bottom.saturating_sub(height)).max(bounds.top),
    )
}
```

**`place`**, privada: a esquina de arriba á esquerda dun menú de `size` onde di `anchor`, dentro de `bounds`, a área cliente da xanela ([ADR 0021](../adr/0021-os-menus-e-o-menu-da-tab.md), decisión 4). Cada eixe decídese á parte:

1. Cada ancla dá dous sitios, o seu e o do outro lado:

   | Ancla | O seu sitio | O do outro lado |
   |---|---|---|
   | `At(at)` | (`at.x`, `at.y`): á dereita do punto, e por debaixo. | (`at.x` − ancho, `at.y` − alto): á esquerda e por riba, rematando no punto. |
   | `Below(part)` | (`part.left`, `part.bottom`): debaixo da parte, desde o seu lado esquerdo. | (`part.right` − ancho, `part.top` − alto): rematando no seu lado dereito, ou por riba dela. |

2. Se, desde o seu sitio, o menú pasa do lado dereito de `bounds`, vai ao outro lado; e o mesmo co bordo de abaixo. O seu sitio sempre vai cara á dereita e cara abaixo, así que só se miran eses dous lados.
3. Despois, móvese o xusto para quedar dentro: `min` lévao cara á esquerda ou cara arriba ata que remate dentro, e `max`, cara á dereita ou cara abaixo ata que empece dentro. Se tampouco cabía no outro lado, queda o máis preto del que pode.
4. `max` vai despois, e gaña: un menú máis alto ca a xanela empeza no seu bordo de arriba, e córtase por abaixo (decisión 1). Máis ancho ca ela non pode ser: [`new`](#facer-o-menú-layoutnew) déixao do seu ancho.

- `coord` converte o tamaño en coordenadas, e `saturating_sub` e `saturating_add` quedan nos extremos dun `i32` en vez de desbordar.
- Os casos das probas, co menú do exemplo, de 193 × 137:

| Ancla | Xanela | Menú | Por que |
|---|---|---|---|
| `At((100, 50))` | 800 × 600 | (100, 50, 293, 187) | Cabe á dereita do punteiro e por debaixo. |
| `At((700, 500))` | 800 × 600 | (507, 363, 700, 500) | Non cabe nin á dereita nin por debaixo: vai á esquerda e por riba, coa esquina de abaixo á dereita no punteiro. |
| `At((120, 100))` | 250 × 150 | (0, 0, 193, 137) | Non cabe en ningún lado: desde o outro, (−73, −37), entra na xanela pola súa esquina. |
| `At((40, 10))` | 150 × 600 | (0, 10, 150, 147) | A xanela é máis estreita ca o menú, que mide coma ela, 150, e vai desde o lado esquerdo da xanela. |
| `At((40, 10))` | 800 × 100 | (40, 0, 233, 137) | É máis alto ca a xanela: empeza arriba, e córtase por abaixo. |
| `Below((244, 4, 484, 32))` | 800 × 600 | (244, 32, 437, 169) | Debaixo da segunda tab da barra do [exemplo de `tabbar.rs`](#cratesmeiga-coresrcuitabbarrs), desde o seu lado esquerdo. |
| `Below((700, 4, 790, 32))` | 800 × 600 | (597, 32, 790, 169) | Unha tab preto do lado dereito da xanela: o menú remata no lado dereito da tab. |

### O sitio de cada parte: `Layout`

```rust
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Layout {
    border: u32,
    item_side: u32,
    item_gap: u32,
    text_padding: u32,
    line: u32,
    line_gap: u32,
    menu: Rect,
    bounds: Rect,
    entries: Vec<Rect>,
}
```

Onde van as partes dun menú, na xanela:

| Campo | Que é |
|---|---|
| De `border` a `line_gap` | As medidas que fan falta despois de facer o menú, xa escaladas ao DPI da xanela. `PADDING` e `ITEM` só se usan ao facelo, e non se gardan. |
| `menu` | O menú enteiro, co seu bordo. |
| `bounds` | A área cliente da xanela, que corta o menú se é máis alto ca ela. Desde a revisión do código da 2.4, [`entry_at`](#a-entrada-debaixo-dun-punto-entry_at) non dá ningunha entrada fóra dela. |
| `entries` | O sitio de cada entrada, na súa orde: de lado a lado dentro do bordo, e do alto dun elemento ou dunha liña. |

- **Todo se calcula unha vez**, ao abrir o menú, e as consultas só len rectángulos. O menú do programa garda a disposición mentres está aberto: se a xanela cambia de tamaño ou de escala, o menú péchase ([ADR 0021](../adr/0021-os-menus-e-o-menu-da-tab.md), decisión 9), e a disposición vai con el.
- Os campos son privados: os sitios das entradas sempre corresponden ao menú.
- Deriva `Clone`, `Debug`, `PartialEq` e `Eq`, pero non `Copy`, ao contrario ca a da barra, porque ten un vector.

### Facer o menú: `Layout::new`

```rust
pub fn new<A>(
    entries: &[Entry<A>],
    dpi: u32,
    cell: Size,
    anchor: Anchor,
    bounds: Rect,
) -> Self {
    let size = |pixels| scale(pixels, dpi);
    let (border, padding, item_side, item_gap) =
        (size(BORDER), size(PADDING), size(ITEM_SIDE), size(ITEM_GAP));
    let (text_padding, line, line_gap) = (size(TEXT_PADDING), size(LINE), size(LINE_GAP));
    let item = size(ITEM) + 2 * item_gap;
    let separator = line + 2 * line_gap;

    let columns = entries
        .iter()
        .filter_map(Entry::label)
        .map(text::width)
        .max()
        .unwrap_or(0);
    let text = u64::from(cell.width).saturating_mul(columns as u64);
    let chrome = 2 * u64::from(border + item_side + text_padding);
    let width = u32::try_from(text.saturating_add(chrome))
        .unwrap_or(u32::MAX)
        .min(bounds.width());
    let height = entries
        .iter()
        .fold(2 * (border + padding), |height, entry| {
            height.saturating_add(match entry {
                Entry::Item { .. } => item,
                Entry::Separator => separator,
            })
        });
    let menu = Rect::at(
        place(anchor, Size::new(width, height), bounds),
        Size::new(width, height),
    );

    let (left, right) = (
        menu.left + coord(border.into()),
        menu.right - coord(border.into()),
    );
    let mut top = menu.top + coord((border + padding).into());
    let entries = entries
        .iter()
        .map(|entry| {
            let height = coord(
                match entry {
                    Entry::Item { .. } => item,
                    Entry::Separator => separator,
                }
                .into(),
            );
            let room = Rect::new(left, top, right, top.saturating_add(height));
            top = room.bottom;
            room
        })
        .collect();
    Self {
        border,
        item_side,
        item_gap,
        text_padding,
        line,
        line_gap,
        menu,
        bounds,
        entries,
    }
}
```

**`new`** fai o menú de `entries` a `dpi`, co texto en celas de `cell` píxeles, onde di `anchor`, dentro de `bounds`, a área cliente da xanela. O menú dunha tab e, desde a 2.5, o dos perfís danlle as celas da fonte da barra, Cascadia Mono a 9 puntos, e o DPI da xanela ([ADR 0021](../adr/0021-os-menus-e-o-menu-da-tab.md), decisión 3). Paso a paso:

1. **As medidas.** Escala cada unha con [`scale`](#os-axudantes-base_dpi-scale-e-coord), a través da *closure* `size`, coma a barra. O sitio dun elemento, `item`, é o seu resalte co espazo de enriba e o de abaixo: 32 a 100 %, e 48 a 150 %; e o dunha liña, `separator`, a liña cos seus dous espazos: 3, e 6.
2. **O ancho.** `columns` son as celas do texto máis longo: `filter_map` salta as liñas, que non teñen texto, `text::width` conta as celas de cada un ([`text.rs`](#cratesmeiga-coresrctextrs)), e `max` colle o maior; sen elementos, 0. O texto ocupa `columns` celas de `cell.width` píxeles, e arredor del, a cada lado, van o espazo ata o resalte, o espazo ata o bordo e o bordo: `chrome`, 32 a 100 % e 50 a 150 %. Despois, `min` déixao como moito tan ancho coma a xanela: os textos que non caben córtaos con «…» o menú do programa (decisión 1).
3. **O alto.** `fold` suma ao bordo e ao espazo de enriba e de abaixo, 6 a 100 %, o sitio de cada entrada. O alto non se recorta á xanela: un menú máis alto córtase por abaixo ([`place`](#onde-se-abre-anchor-e-place)), e o que queda cortado non o ten o punteiro ([`entry_at`](#a-entrada-debaixo-dun-punto-entry_at)).
4. **O menú.** [`place`](#onde-se-abre-anchor-e-place) dá a súa esquina de arriba á esquerda, e `Rect::at` fai o rectángulo con ela e co tamaño.
5. **As entradas**, unha debaixo doutra, desde o bordo e o espazo de enriba: cada sitio vai de lado a lado dentro do bordo, co alto dun elemento ou dunha liña. `top` é onde empeza a seguinte: a *closure* de `map` móveo despois de cada unha, e `collect` garda os sitios nun vector.

- **Sen desbordar.** O ancho do texto calcúlase en `u64`, saturando, así que non desborda con ningún número de celas; se o ancho non cabe nun `u32`, `try_from` dá `u32::MAX`, e `min` déixao no da xanela. O alto satura en `u32`, e `coord` converte as medidas en coordenadas ([Os axudantes](#os-axudantes-base_dpi-scale-e-coord)).
- **O texto, en celas.** O texto vai coa fonte da barra, que ten todos os caracteres do mesmo ancho: medilo é contar celas, sen DirectWrite, coma os títulos das tabs ([ADR 0020](../adr/0020-a-barra-de-tabs.md), decisión 4).
- `new` é xenérica na acción, `A`, pero non a usa: só le os textos, e cales entradas son elementos e cales liñas. Por iso non pide `A: Copy`.

### As partes: `menu`, `border`, `entry`, `highlight`, `text` e `line`

```rust
pub const fn menu(&self) -> Rect {
    self.menu
}

pub const fn border(&self) -> u32 {
    self.border
}

pub fn entry(&self, index: usize) -> Rect {
    self.entries.get(index).copied().unwrap_or(Rect::EMPTY)
}

pub fn highlight(&self, index: usize) -> Rect {
    inset(self.entry(index), self.item_side, self.item_gap)
}

pub fn text(&self, index: usize) -> Rect {
    inset(self.highlight(index), self.text_padding, 0)
}

pub fn line(&self, index: usize) -> Rect {
    let room = self.entry(index);
    let top = room.top + coord(self.line_gap.into());
    Rect::new(room.left, top, room.right, top + coord(self.line.into()))
}
```

```rust
fn inset(rect: Rect, x: u32, y: u32) -> Rect {
    let (x, y) = (coord(x.into()), coord(y.into()));
    let inner = Rect::new(rect.left + x, rect.top + y, rect.right - x, rect.bottom - y);
    if inner.is_empty() { Rect::EMPTY } else { inner }
}
```

| Método | Que dá | No exemplo |
|---|---|---|
| `menu()` | O menú enteiro. O programa debúxao enteiro ao abrilo, e debuxa outra vez todo o que tapaba ao pechalo ([ADR 0021](../adr/0021-os-menus-e-o-menu-da-tab.md), decisión 12). | (100, 50, 293, 187) |
| `border()` | O grosor do bordo, que o programa debuxa por dentro do menú. | 1 |
| `entry(i)` | O sitio da entrada `i`: de lado a lado dentro do bordo. Despois da última, `Rect::EMPTY`. | `entry(0)`: (101, 53, 292, 85) |
| `highlight(i)` | O resalte do elemento `i`: o seu sitio sen `item_side` píxeles aos lados nin `item_gap` por riba e por debaixo. É tamén o que o programa debuxa outra vez cando o elemento gaña ou perde o resalte. | `highlight(0)`: (105, 55, 288, 83) |
| `text(i)` | O sitio do texto do elemento `i`: o resalte sen `text_padding` píxeles aos lados, co seu alto. O programa corta o texto ás columnas que caben nel, e debúxao centrado no seu alto. | `text(0)`: (116, 55, 277, 83) |
| `line(i)` | A liña do separador `i`: `line_gap` píxeles por debaixo do comezo do seu sitio, de lado a lado, e `line` de alto. | `line(1)`: (101, 86, 292, 87) |

- **`inset`**, privada: `rect` sen `x` píxeles aos lados nin `y` por riba e por debaixo; se non queda nada, `Rect::EMPTY`. Así, o resalte dunha liña, que só mide 3, ou o texto dun menú máis estreito ca as súas marxes, dan sempre o mesmo rectángulo baleiro, e non uns lados do revés.
- `highlight` e `text` só teñen sentido nun elemento, e `line`, nunha liña: o programa chama uns ou outro segundo a entrada.
- `entry` dá unha copia do rectángulo (`copied`), coma `menu`: os rectángulos son `Copy`.

### A entrada debaixo dun punto: `entry_at`

```rust
pub fn entry_at(&self, at: Point) -> Option<usize> {
    if !self.bounds.contains_point(at) {
        return None;
    }
    self.entries.iter().position(|room| room.contains_point(at))
}
```

**`entry_at`** dá a entrada que ten o píxel `at`, en coordenadas da área cliente, ou `None`: no bordo, no espazo por riba da primeira entrada e por debaixo da última, fóra do menú, e fóra da xanela.

- **Fóra da xanela**, desde a revisión do código da 2.4: nun menú máis alto ca ela, o que queda por debaixo do seu bordo de abaixo non se ve, e o punteiro non o resalta nin o escolle. Co rato capturado, cun botón premido no menú, o punteiro pode chegar alí, fóra da xanela ([Código › meiga › `menu_mouse`](meiga.md#o-rato-co-menú-aberto-menu_mouse)); antes, soltar o botón enriba dun elemento que non se vía escollíao. Os lados e o bordo de arriba nunca cortan o menú: o seu ancho non pasa do da xanela, e [`place`](#onde-se-abre-anchor-e-place) non o deixa saír por eles.
- **Todo o sitio dun elemento é del**, tamén aos lados do resalte e por riba e por debaixo del: entre dous elementos non hai ningún oco onde o punteiro perda o resalte. Os sitios van seguidos e non se solapan, así que un punto está, como moito, nun ([`contains_point`](#cratesmeiga-coresrcgeometryrs)).
- Unha liña tamén é unha entrada: o punteiro sobre ela dá a súa posición, e [`hover`](#menu-as-entradas-e-o-resalte) non resalta nada.
- `position` percorre os sitios en orde, e dá o primeiro que contén o punto.
- O programa usa a resposta co punteiro, para resaltar, e co botón que sobe, para escoller ([`action`](#escoller-action-e-chosen)).

No exemplo, `the_pointer_is_on_an_item_anywhere_in_its_room` comproba:

| Punto | Entrada | Por que |
|---|---|---|
| (101, 53) | 0 | O primeiro píxel do sitio de *Duplicate tab*. |
| (102, 84) | 0 | Á esquerda do seu resalte e por debaixo del. |
| (200, 86) | 1 | Na liña. |
| (291, 183) | 4 | O último píxel do sitio de *Close tabs to the right*. |
| (100, 60) | `None` | No bordo. |
| (200, 52) e (200, 185) | `None` | No espazo por riba da primeira entrada e por debaixo da última. |
| (99, 60) | `None` | Fóra do menú. |

**Probas.** Oito, con coordenadas exactas. `entries` fai as entradas do exemplo, as do menú dunha tab, con *Close other tabs* desactivado e os números do 1 ao 4 coma accións; e `layout`, o menú do exemplo onde diga a ancla: a 96 DPI, con celas de 7 × 16 píxeles, nunha xanela de 800 × 600.

| Proba | Que comproba |
|---|---|
| `the_menu_is_as_wide_as_its_longest_item_and_as_high_as_its_entries` | O menú do exemplo, en (100, 50, 293, 187), co bordo de 1; os sitios de *Duplicate tab*, da liña, de *Close tab* e de *Close tabs to the right*, e ningún despois do último (`Rect::EMPTY`); e o resalte e o texto de *Duplicate tab*, e a liña, coma na táboa do exemplo. |
| `the_pointer_is_on_an_item_anywhere_in_its_room` | Os puntos da táboa de [`entry_at`](#a-entrada-debaixo-dun-punto-entry_at). |
| `a_menu_that_does_not_fit_goes_to_the_other_side_and_stays_in_the_window` | Os casos de [`place`](#onde-se-abre-anchor-e-place) que non caben por ese lado: o punteiro en (700, 500); unha xanela de 250 × 150; unha de 150 de ancho, onde o texto de *Duplicate tab* queda en (16, 15, 134, 43); e unha de 100 de alto, onde o menú, en (40, 0, 233, 137), queda cortado: en (100, 99), por riba do bordo de abaixo da xanela, o punteiro está en *Close other tabs*, que a xanela corta, e en (100, 100), no seu sitio pero xa fóra da xanela, en ningunha entrada, nin en (100, 120), no sitio de *Close tabs to the right*, que non se ve (desde a revisión do código da 2.4). |
| `below_a_part_of_the_window_from_its_left_side` | Os dous casos de `Below` de [`place`](#onde-se-abre-anchor-e-place). |
| `the_sizes_follow_the_scale` | A 144 DPI (150 %), con celas de 11 × 25 nunha xanela de 1920 × 1080: o menú en (0, 0, 303, 208), o bordo de 2, o sitio de *Duplicate tab* en (2, 5, 301, 53), o seu resalte en (8, 8, 295, 50), e a liña en (2, 55, 301, 57). |
| `the_keyboard_skips_what_cannot_be_chosen_and_goes_round` | Sen resalte, `Next` vai a *Duplicate tab*; despois salta a liña e o elemento desactivado, e do último volta ao primeiro; `Previous` volta do primeiro ao último, e despois vai a *Close tab*; `First` vai ao primeiro e, outra vez, non cambia nada; `Last` vai ao último, e `chosen` dá a súa acción; e, sen resalte, `Previous` vai ao último. |
| `the_pointer_only_highlights_what_can_be_chosen` | O punteiro sobre *Close tab* resáltao, e `chosen` dá a súa acción; outra vez, non cambia nada; sobre a liña, quita o resalte; e sobre o elemento desactivado, e despois da última entrada, non resalta nada. `action` non dá a acción do desactivado, e si a do último. |
| `a_menu_without_items_to_choose_highlights_nothing` | Cun elemento desactivado e unha liña, `Next` e `Last` non resaltan nada; e `label` dá o texto do elemento, e nada na liña. |

Os elementos en negra, desde a 2.5, non teñen proba eiquí, porque a negra non cambia ningún rectángulo: próbaa o menú do programa, que debuxa o mesmo texto en normal e en negra, e comproba que o da negra se aparta máis do fondo, polo menos un 10 % (`the_text_of_a_bold_item_is_bold`, en [Código › meiga › `menu.rs`](meiga.md#cratesmeigasrcmenurs)).

## `crates/meiga-core/src/ui/field/mod.rs`

Desde a subfase 3.2: un campo de texto, unha liña que o usuario escribe e edita, coma o nome dunha tab (a subfase 3.3), o dun grupo (a 6.3) ou o filtro do explorador (a 5.3) ([ADR 0030](../adr/0030-o-campo-de-texto.md), [Arquitectura › O campo de texto](../arquitectura.md#o-campo-de-texto), [Tecnoloxías › Os grafemas e as palabras dun campo de texto](../tecnoloxias.md#66-os-grafemas-e-as-palabras-dun-campo-de-texto)). Coma a barra de tabs e os menús, vai en dúas metades. Esta é a que non depende de Windows, e próbase sen xanela: o texto, o cursor e a selección, o que fan con eles as teclas e o rato, e a parte do texto que se ve nas celas do campo. A outra, a vista, debúxao, pásalle o teclado e o rato, e le e escribe o portapapeis por el; chega na 3.3, co primeiro uso do campo, na tab (ADR 0030, decisión 1). Ata entón, un exemplo de consola próbao a man, coas teclas de verdade e o portapapeis de Windows ([Código › meiga-win › `examples/field.rs`](meiga-win.md#cratesmeiga-winexamplesfieldrs)). O campo non ten fíos nin temporizadores, e non traballa nos fotogramas: só cando chega unha tecla ou o rato.

**Coma Chrome en Windows.** O campo edita coma os campos de texto de Chrome 145 en Windows (`views::Textfield`), que o ADR compara coa caixa que renomea as tabs no Windows Terminal, o control `EDIT` de Win32, WezTerm, Kitty e Alacritty: o cursor para entre os grafemas, Retroceso borra un punto de código e Supr un grafema, Ctrl e as frechas van ao principio das palabras, e desfai e refai coma el. Tres cousas son distintas: as palabras son máis simples, o pegado vai nunha liña, e o texto ten un límite (decisións 3, 5 e 6).

O módulo vai no cartafol `src/ui/field/`, con catro submódulos:

| Arquivo | Que ten |
|---|---|
| `mod.rs` | `Field`, cos seus datos e os seus métodos, menos as teclas: o texto, a selección, o pegado, o rato, o que mostra e os cambios; e as funcións do pegado e do límite. |
| [`segment.rs`](#cratesmeiga-coresrcuifieldsegmentrs) | Onde para o cursor, entre os grafemas, e onde empezan as palabras. |
| [`history.rs`](#cratesmeiga-coresrcuifieldhistoryrs) | Os pasos para desfacer e refacer. |
| [`keys.rs`](#cratesmeiga-coresrcuifieldkeysrs) | O que fai cada tecla: `Field::key`. |
| [`tests.rs`](#cratesmeiga-coresrcuifieldtestsrs) | As probas do campo. |

Os estados do campo escríbense coma nas probas: `|` é o cursor, e `[` e `]` os extremos da selección. `a[bc|]d` é o texto `abcd`, con `bc` seleccionado e o cursor despois do `c`.

```rust
//! A text field: a line of text that the user edits, such as the name of a
//! tab (subphase 3.3), the name of a group (6.3) or the filter of the
//! explorer (5.3) (ADR 0030). This is the field without a window: its text,
//! its caret and its selection, what the keys and the mouse do with them,
//! and the part of the text that it shows in its cells. The window draws
//! it, and reads and writes the clipboard for it.
//!
//! It edits as the text fields of Chrome on Windows:
//! - the caret stops between the grapheme clusters of Unicode (UAX #29), and
//!   moves by words with Ctrl ([`segment`]); Backspace deletes the last code
//!   point before the caret, so that `é` written as `e` and an accent loses
//!   only the accent, and Delete the cluster after it;
//! - with a selection, typing replaces it, and Backspace and Delete delete
//!   it; the arrows without Shift go to its edge;
//! - undoing goes back one step at a time ([`history`]);
//! - a paste becomes a single line: the whitespace at its ends goes away;
//!   each line break and each tab, a space; and the other control
//!   characters, away. A paste of only whitespace is a space.
//!
//! The field holds at most a number of UTF-16 code units, as Windows counts
//! them, that its user chooses: a character that does not fit is not
//! typed, and a paste puts what fits, without splitting a cluster. Its text
//! never has control characters.
//!
//! It shows its text in a row of cells of the grid, as the rest of the
//! text that meiga draws ([`crate::text::cells`]): if the text does not
//! fit, it shows the part where the caret is, and moves it just enough to
//! keep the caret in sight.

mod history;
mod keys;
mod segment;

use std::mem;
use std::ops::Range;

use unicode_segmentation::UnicodeSegmentation;

use self::history::{Edit, History, Kind};
use self::segment::{is_stop, next_cluster, previous_cluster, word_at};
use crate::term::Side;
use crate::text::{self, CellChar};
```

- O comentario do módulo (`//!`) di o mesmo ca os parágrafos de enriba, e resume as regras: o cursor para entre os grafemas de UAX #29, e vai por palabras con Ctrl ([`segment.rs`](#cratesmeiga-coresrcuifieldsegmentrs)); Retroceso borra o último punto de código, e Supr o grafema seguinte; cunha selección, escribir cámbiaa, Retroceso e Supr bórrana e as frechas van ao seu bordo; desfacer volta paso a paso ([`history.rs`](#cratesmeiga-coresrcuifieldhistoryrs)); o pegado vai nunha liña; o texto ten un límite en unidades de UTF-16, e nunca caracteres de control; e móstrase en celas, coma o resto do texto que debuxa meiga ([`text::cells`](#cratesmeiga-coresrctextrs)), coa parte onde está o cursor.
- **Os submódulos son privados** (`mod`, sen `pub`): fóra de `field` non se ve nada deles. `keys.rs` engádelle métodos a `Field` noutro bloque `impl`, entre eles o público `key`, que fóra se usa coma calquera outro método. A privacidade de Rust vai por módulos: un método privado de `keys.rs`, coma `command`, só se ve en `field::keys`; e os privados de `mod.rs`, coma `select`, `replace` ou `track`, vense tamén nos submódulos, porque un módulo ve o que é privado nos seus antepasados, coma `scale` en [`ui`](#cratesmeiga-coresrcuimodrs).
- `use std::mem;` trae `mem::take`, para os cambios, e `Range`, os rangos da selección e das palabras.
- `use unicode_segmentation::UnicodeSegmentation;` trae o *trait* que lle dá a `str` o método `grapheme_indices`, cos grafemas dun texto e o principio de cada un: con el percorren o texto `stop_at`, `word_under` e `fitting`. O resto dos grafemas está en [`segment.rs`](#cratesmeiga-coresrcuifieldsegmentrs).
- `use self::history::…` e `use self::segment::…` traen o que usa este arquivo dos seus submódulos (`self` é este módulo). `Side` é o lado dunha cela, o mesmo tipo ca o da selección do terminal ([`term::Side`](#word_delimiters-selectionkind-e-side)). E de `text` veñen `text::cells` e `text::width`, as celas dun texto e cantas son, e `CellChar`, o que mostra cada cela ([`text.rs`](#cratesmeiga-coresrctextrs)).

### `Reply`: o que fixo unha tecla

```rust
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Reply {
    Ignored,
    Taken,
    Copy(String),
    Paste,
}
```

[`key`](#key-a-entrada-das-teclas) responde a cada tecla cun `Reply`:

| Variante | Que fixo o campo |
|---|---|
| `Ignored` | Nada: a tecla non é do campo, e decide quen o usa (ADR 0030, decisión 7). Son Intro, Esc, Tab, ↑ e ↓ sen Maiús, as teclas sen carácter, coma F2 ou Maiús soa, as que soben, os caracteres de control e os de Alt sen Ctrl. Esc desbotará a edición enteira (3.3), e o filtro do explorador moverase pola lista con ↑ e ↓. |
| `Taken` | Colleuna, aínda que non cambiase nada: unha frecha no extremo do texto, Retroceso ao principio, un carácter que non cabe ou Ctrl+C sen selección. A tecla non lle chega a ninguén máis. |
| `Copy(texto)` | Colleuna, e quen o usa pon o texto no portapapeis: a selección, con Ctrl+C ou Ctrl+Insert, que copian, ou con Ctrl+X ou Maiús+Supr, que cortan, e que xa a quitaron do campo. |
| `Paste` | Colleuna, e quen o usa lle dá a [`paste`](#cambiar-o-texto-set_text-select_all-e-paste) o texto do portapapeis: Ctrl+V e Maiús+Insert. |

- **O campo non toca o portapapeis.** Para escribir nel, Windows quere unha xanela que o abra (`OpenClipboard`), e o campo non ten ningunha: o texto vai e vén coma un `String`, e quen o usa leo e escríbeo coas funcións de `meiga-win` ([Código › meiga-win › `clipboard.rs`](meiga-win.md#cratesmeiga-winsrcclipboardrs)), coma o exemplo, que ten unha xanela oculta só para iso.
- O comentario de `Copy` chámalle `text` ao texto, que a variante leva sen nome. Ata `b96fa25` nomeaba só Ctrl+C e Ctrl+X; desde entón, nomea as catro teclas ([`command`](#as-ordes-command-e-repeat)).
- Deriva `Clone`, `Debug`, `PartialEq` e `Eq`, e non `Copy`, porque leva un `String`. As probas comparan as respostas con `assert_eq!`.

### `Changes` e `Shown`: o que cambiou e o que se mostra

```rust
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Changes {
    pub shown: bool,
    pub text: bool,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Shown {
    pub caret: usize,
    pub selection: Range<usize>,
}
```

**`Changes`**: o que cambiou no campo desde que se colleron os seus cambios por última vez, con [`take_changes`](#os-cambios-e-as-celas-take_changes-e-set_columns):

| Campo | Que cambiou |
|---|---|
| `shown` | O que mostra: o texto, o cursor, a selección, as súas celas ou a parte do texto nelas. Quen o usa debúxao outra vez. |
| `text` | O texto, se é outro. O filtro do explorador filtrará outra vez a lista. |

- Un cambio do texto é sempre tamén un cambio do que mostra: [`track`](#seguir-o-cursor-track-e-follow_caret) márcao.
- Só se marcan cando algo é outro: unha frecha no extremo do texto non é un cambio, nin o mesmo texto outra vez con `set_text`, nin o mesmo número de celas. Tampouco unha edición que deixa o mesmo texto, coma escribir un `a` enriba dun `a` seleccionado, ou desfacela: é un cambio do que mostra, porque a selección se vai ou volta, pero non do texto ([`edited`](#editar-replace-e-edited)).
- `Copy` e `Default`: dous `bool`, e `Changes::default()` é «nada cambiou».

**`Shown`**: o que mostra o campo nas súas celas, ademais do texto, que dá [`show`](#o-que-mostra-show):

| Campo | Que é |
|---|---|
| `caret` | A cela do campo en cuxo lado esquerdo vai o cursor, contando desde a primeira: sempre unha das celas do campo, se ten algunha, tamén ao final dun texto que non cabe, porque o campo deixa entón unha cela libre para o cursor ([`follow_caret`](#seguir-o-cursor-track-e-follow_caret)). Sen celas, 0. |
| `selection` | As celas seleccionadas, das que mostra o campo. Baleiro sen selección, ou se a selección queda fóra da parte que se ve. |

- Non é `Copy`, porque un `Range` de Rust non o é: un rango é tamén un iterador.

### `Drag`: o que selecciona o botón abaixo

```rust
#[derive(Clone, Debug, Default, PartialEq, Eq)]
enum Drag {
    #[default]
    None,
    Clusters,
    Words(Range<usize>),
    All,
}
```

O que selecciona un botón premido sobre o campo cando se move o punteiro ([`drag`](#o-rato-press-e-drag)):

| Variante | Que selecciona |
|---|---|
| `None` | Nada: non baixou ningún botón, ou houbo despois unha edición, aínda que deixase o mesmo texto ([`edited`](#editar-replace-e-edited)). É o valor por defecto (`#[default]`), o dun campo novo. |
| `Clusters` | Despois dun clic: desde onde empezou a selección ata o grafema debaixo do punteiro. |
| `Words(rango)` | Despois dun dobre clic: palabras enteiras, desde a do dobre clic, o rango que garda, ata a que está debaixo do punteiro. O comentario chámalle `.0`, o primeiro campo da variante. |
| `All` | Despois dun triplo clic: todo, e mover o punteiro non cambia nada. |

- É privado: fóra só se ven `press` e `drag`.
- **O campo non sabe cando sobe o botón.** Quen o usa só chama a `drag` mentres está abaixo, coma di o seu comentario, e o seguinte clic cambia o `Drag`. Unha edición remátao antes (ADR 0030, decisión 7).
- Deriva `Clone`, e non `Copy`, polo rango: `drag` cópiao antes de miralo, porque os seus casos chaman a `select`, que cambia o campo.

### `Field`

```rust
#[derive(Clone, Debug)]
pub struct Field {
    text: String,
    anchor: usize,
    caret: usize,
    limit: usize,
    history: History,
    high_surrogate: Option<u16>,
    drag: Drag,
    columns: usize,
    scroll: usize,
    changes: Changes,
}
```

| Campo | Que é |
|---|---|
| `text` | O texto, nunha liña: sen caracteres de control, e como moito con `limit` unidades de UTF-16. |
| `anchor` | Onde empezou a selección: o extremo que non moven Maiús nin o arrastre. |
| `caret` | O cursor, o outro extremo, que pode ir antes ou despois de `anchor`. Sen selección, os dous son o mesmo. |
| `limit` | As unidades de UTF-16 que pode ter o texto, como moito: as que diga quen o usa, ao facelo. Non cambia. |
| `history` | Os pasos para desfacer e refacer ([`History`](#edit-e-history)). |
| `high_surrogate` | A primeira metade dun carácter de fóra do plano básico de Unicode, coma a maioría dos emoji, mentres espera pola segunda, que chega coa tecla seguinte ([`typed`](#os-caracteres-typed-e-type_character)). |
| `drag` | O que selecciona o botón abaixo ([`Drag`](#drag-o-que-selecciona-o-botón-abaixo)). |
| `columns` | Cantas celas mostra o campo: 0 ata que llo diga quen o usa ([`set_columns`](#os-cambios-e-as-celas-take_changes-e-set_columns)). |
| `scroll` | A primeira cela do texto que mostra: o desprazamento horizontal, en celas. |
| `changes` | O que cambiou desde que se colleron os cambios por última vez. |

- **As posicións son bytes.** `anchor` e `caret` son desprazamentos en bytes do texto, que un `String` de Rust garda en UTF-8: `&self.text[..self.caret]` é o texto antes do cursor, sen copialo. E sempre están nun sitio onde pode parar o cursor: entre dous grafemas, ou nun extremo do texto ([`segment.rs`](#cratesmeiga-coresrcuifieldsegmentrs)). Nunca entre unha letra e o seu acento, nin no medio dun carácter, onde cortar o texto sería un *panic*. Todo o que os move mantén esta regra: as frechas, Inicio, Fin e o rato só van a eses sitios; cada edición leva o cursor a un deles ([`replace`](#editar-replace-e-edited)); e desfacer volta pór os que había. A proba ao chou compróbao despois de cada acción ([`tests.rs`](#a-proba-ao-chou-random_edits_keep_the_field_whole_and_undo_to_the_start)).
- **As celas saen do texto.** A cela dunha posición son as celas do texto antes dela, `text::width`: dúas posicións poden estar na mesma cela, coma as de antes e de despois dun grafema sen ancho.
- Os campos son privados: só os cambian os métodos, que manteñen as regras. Deriva `Clone` e `Debug`; `PartialEq` non, que as probas non o necesitan: comparan o texto, o cursor e a selección.

### Facer o campo e lelo: `new`, `text`, `limit`, `caret`, `selection` e `selected_text`

```rust
pub fn new(limit: usize) -> Self {
    Self {
        text: String::new(),
        anchor: 0,
        caret: 0,
        limit,
        history: History::default(),
        high_surrogate: None,
        drag: Drag::None,
        columns: 0,
        scroll: 0,
        changes: Changes::default(),
    }
}

pub fn text(&self) -> &str {
    &self.text
}

pub const fn limit(&self) -> usize {
    self.limit
}

pub const fn caret(&self) -> usize {
    self.caret
}

pub fn selection(&self) -> Range<usize> {
    self.anchor.min(self.caret)..self.anchor.max(self.caret)
}

pub fn selected_text(&self) -> &str {
    &self.text[self.selection()]
}
```

- **`new`**: un campo baleiro que garda como moito `limit` unidades de UTF-16, sen selección, sen pasos e sen celas: non mostra nada ata que sabe cantas ten. A 3.3 porá 1000, coma os títulos ([ADR 0029](../adr/0029-o-titulo-da-tab-e-os-titulos-gardados.md), decisión 6); o exemplo, 1000 ou o que diga `--limit`.
- **`text`**: o texto, prestado.
- **`limit`** e **`caret`**: o límite, e o cursor, coma un desprazamento en bytes do texto. Son `const fn`.
- **`selection`**: a selección, coma un rango de bytes, sempre de esquerda a dereita, estea o cursor no extremo que estea: `min` e `max` ordenan `anchor` e `caret`. Sen selección, un rango baleiro onde está o cursor.
- **`selected_text`**: o texto seleccionado, prestado. Copiar e cortar dan unha copia del.

### Cambiar o texto: `set_text`, `select_all` e `paste`

```rust
pub fn set_text(&mut self, text: &str) {
    let line = one_line(text);
    let line = fitting(&line, self.limit);
    self.track(|field| {
        if field.text != line {
            field.text = line.to_owned();
            field.changes.text = true;
        }
        let end = field.text.len();
        (field.anchor, field.caret) = (end, end);
        field.history.clear();
        field.high_surrogate = None;
        field.drag = Drag::None;
    });
}

pub fn select_all(&mut self) {
    let end = self.text.len();
    self.select(0, end);
}

pub fn paste(&mut self, clipboard: &str) {
    let line = one_line(clipboard);
    if line.is_empty() {
        return;
    }
    let trimmed = line.trim();
    let text = if trimmed.is_empty() { " " } else { trimmed };
    let text = fitting(text, self.room());
    if text.is_empty() {
        return;
    }
    self.track(|field| field.replace(field.selection(), text, Kind::Other));
}
```

**`set_text`** pon `text` no campo, en vez do que tiña, co cursor ao final:

1. [`one_line`](#pegar-nunha-liña-one_line-fitting-e-units) déixao nunha liña, e [`fitting`](#pegar-nunha-liña-one_line-fitting-e-units) córtao ao límite, en grafemas enteiros.
2. Dentro de [`track`](#seguir-o-cursor-track-e-follow_caret), que move a parte que se mostra e anota os cambios: se o texto é outro, cámbiao e anota o cambio; pon o cursor ao final, sen selección; e esquece os pasos, a metade dun carácter que esperaba e o arrastre.

- **Non se desfai.** Desfacer non trae o que había: o campo empeza de novo, coma un campo novo con ese texto. O exemplo pono así ao principio, e despois selecciona todo; e, con Intro, baleira o campo con `set_text("")`.
- **Ao contrario ca `paste`,** non quita os espazos dos extremos: o texto queda coma é, menos os caracteres de control e o que non cabe.
- O mesmo texto outra vez, co cursor xa ao final, non é un cambio.
- Que esquece a metade dun carácter e o arrastre compróbao, desde `50d2379`, `set_text_forgets_half_a_character_and_what_a_button_held_down_selects`; ata entón, ningunha proba o vía.

**`select_all`** selecciona todo, co cursor ao final: así, nun texto longo, o campo mostra o final. Úsana Ctrl+A e o triplo clic. Coma calquera cambio da selección, pecha o paso que se estaba a escribir ([`select`](#seleccionar-select)).

**`paste`** pega `clipboard`, o texto do portapapeis, en vez da selección ([ADR 0030](../adr/0030-o-campo-de-texto.md), decisións 5 e 6). Quen usa o campo chámao cando unha tecla responde `Paste`, co texto que leu do portapapeis; o exemplo, tamén con F5. Paso a paso:

1. [`one_line`](#pegar-nunha-liña-one_line-fitting-e-units) déixao nunha liña: cada salto de liña e cada tabulador, un espazo, e os outros caracteres de control, fóra. Se non queda nada, porque o portapapeis non tiña texto ou só tiña caracteres de control, non pega nada.
2. `trim` quita os espazos dos extremos, coma Chrome: tamén os que eran saltos de liña ou tabuladores, e os de Unicode, coma o que non parte (U+00A0) ou o ideográfico (U+3000). Se só había espazos, pega un.
3. [`fitting`](#pegar-nunha-liña-one_line-fitting-e-units) colle o que cabe en vez da selección ([`room`](#o-límite-room)), en grafemas enteiros. Se non cabe nada, non cambia nada: nin sequera quita a selección.
4. [`replace`](#editar-replace-e-edited) pono en vez da selección, co cursor despois, nun paso propio (`Kind::Other`), dentro de `track`.

| No portapapeis | Pega |
|---|---|
| `"plain"` | `plain` |
| `"  a\r\nb\tc\u{7}d\u{85}e\u{2028}f\u{2029}g\u{b}h\u{c}i\r\n"` | `a b cd e f g h i`: cada salto de liña e o tabulador, un espazo; o BEL (`\u{7}`), fóra; e os espazos e os saltos de liña dos extremos, tamén. |
| `"un\ndous\rtres\r\ncatro"` | `un dous tres catro` |
| `"a\n\rb"` | `a  b`: un LF e despois un CR son dous saltos de liña. |
| `"a   b"` | `a   b`: os espazos do medio quedan. |
| `"\u{a0}x\u{3000}"` | `x` |
| `"\u{1b}[31mred"` | `[31mred`: o ESC vaise, e o resto da secuencia queda coma texto. |
| `" \r\n\t "` | Un espazo. |
| `""`, ou `"\u{7}\u{1b}\u{0}"` | Nada. |

### Os cambios e as celas: `take_changes` e `set_columns`

```rust
pub fn take_changes(&mut self) -> Changes {
    mem::take(&mut self.changes)
}

pub fn set_columns(&mut self, columns: usize) {
    self.track(|field| field.columns = columns);
}
```

- **`take_changes`**: os cambios desde a última vez que se colleron, que esquece: `mem::take` devolve o valor e deixa `Changes::default()`, coma [`take_changed`](#o-cambio-take_changed-e-replace) cos títulos. Despois de pasarlle unha tecla ou o rato, quen usa o campo pregúntao, e só o debuxa outra vez se cambiou o que mostra.
- **`set_columns`**: o campo mostra `columns` celas. Dentro de [`track`](#seguir-o-cursor-track-e-follow_caret): con outro número de celas, a parte que se mostra móvese o xusto para que se vexa o cursor. Outro número de celas é un cambio de `shown`, porque o campo mostra máis ou menos texto; o mesmo, non. Con 0 celas, o campo non mostra nada.
- **Desde `b96fa25`.** Ata entón, `track` non miraba as celas, e `set_columns` só era un cambio se se movía a parte do texto que se ve: un campo que pasaba de 0 celas a 2, e de non mostrar nada a mostrar `ab`, dicía que nada cambiara, ao contrario do que di o comentario de `shown`. Atopouno o axente da documentación de `meiga-win`, ao documentar o exemplo ([Código › meiga-win › `examples/field.rs`](meiga-win.md#cratesmeiga-winexamplesfieldrs)).

### O que mostra: `show`

```rust
pub fn show(&self, cells: &mut Vec<CellChar>) -> Shown {
    cells.clear();
    cells.extend(text::cells(&self.text).skip(self.scroll).take(self.columns));
    // Only a wide character that the right side cuts ends in its left
    // half.
    if let Some(last @ CellChar::WideLeft(_)) = cells.last_mut() {
        *last = CellChar::Blank;
    }
    let shown = self.scroll..self.scroll + self.columns;
    let cell =
        |at: usize| text::width(&self.text[..at]).clamp(shown.start, shown.end) - shown.start;
    let selection = self.selection();
    Shown {
        caret: cell(self.caret),
        selection: cell(selection.start)..cell(selection.end),
    }
}
```

**`show`** pon en `cells` as celas do texto que mostra o campo, en vez do que tiña, e devolve onde van o cursor e a selección ([`Shown`](#changes-e-shown-o-que-cambiou-e-o-que-se-mostra)). Paso a paso:

1. `clear` baleira `cells` sen soltar a súa memoria, e `extend` pon as celas do texto desde a `scroll`, como moito `columns`: [`text::cells`](#cratesmeiga-coresrctextrs) dá as celas unha a unha, `skip` salta as de antes, e `take` para despois da última que cabe, sen percorrer o resto do texto. Se quen usa o campo garda o vector entre un debuxo e outro, coma a barra cos títulos ([`fit`](#cratesmeiga-coresrctextrs)), mostrar o campo non reserva memoria.
2. **Á dereita, nunca media letra ancha.** Se a última cela é a metade esquerda dun carácter ancho, a súa metade dereita quedou fóra: coma di o comentario, só pode acabar así un carácter ancho que corta o lado dereito do campo. Esa cela vai en branco (ADR 0030, decisión 7). `if let Some(last @ CellChar::WideLeft(_))` colle a referencia á última cela, se é desa clase, para cambiala no seu sitio. Á esquerda, o campo nunca empeza na metade dereita dun: dío [`follow_caret`](#seguir-o-cursor-track-e-follow_caret).
3. `shown` son as celas do texto que se ven, e `cell` dá a cela do campo dunha posición do texto: as celas do texto antes dela, dentro das que se ven (`clamp`), contadas desde a primeira. Unha posición antes da parte que se ve dá 0, e unha despois, `columns`.
4. O cursor e os dous extremos da selección, en celas do campo.

- `&self`: mostrar non cambia nada. A parte que se ve decidiuna `track`, cando cambiou algo.
- **Desde `50d2379`.** Ata entón, `show` miraba tamén que as celas enchesen o campo antes de deixar en branco a última, cousa que sempre pasa cando acaba nunha metade esquerda. Atopouno o axente da documentación.
- **Por exemplo**, coas probas: `a中中` co cursor ao final, en catro celas, mostra só `中` e a cela libre do cursor, porque unha cela antes mostraría a metade dereita do primeiro `中`; e `ab中` co cursor ao principio, en tres celas, mostra `ab` e unha cela en branco.
- Cada chamada mide o texto ata o cursor e ata os extremos da selección (`text::width`): nunha liña de como moito 1000 unidades, pouco.

### O rato: `press` e `drag`

```rust
pub fn press(&mut self, column: usize, side: Side, clicks: u8, shift: bool) {
    let column = isize::try_from(column).unwrap_or(isize::MAX);
    match clicks {
        2 => {
            let word = self.word_under(column);
            self.select(word.start, word.end);
            self.drag = Drag::Words(word);
        }
        3 => {
            self.select_all();
            self.drag = Drag::All;
        }
        _ => {
            let at = self.stop_at(column, side);
            if shift {
                self.select(self.anchor, at);
            } else {
                self.select(at, at);
            }
            self.drag = Drag::Clusters;
        }
    }
}

pub fn drag(&mut self, column: isize, side: Side) {
    match self.drag.clone() {
        Drag::None | Drag::All => {}
        Drag::Clusters => {
            let at = self.stop_at(column, side);
            self.select(self.anchor, at);
        }
        Drag::Words(word) => {
            let under = self.word_under(column);
            if under.start >= word.start {
                self.select(word.start, under.end.max(word.end));
            } else {
                self.select(word.end, under.start);
            }
        }
    }
}
```

O rato chega en celas: quen usa o campo converte o punto nunha cela e un lado, e conta os clics seguidos ([`Clicks`](#os-clics-seguidos-clicksettings-e-clicks), que despois do terceiro dá outra vez 1). O lado é o mesmo tipo ca o da selección do terminal, que dá [`cell_at`](#a-cela-debaixo-do-punteiro-cell_at) para a súa grella (ADR 0030, decisión 7).

**`press`**: o botón baixou na cela `column` do campo, contando desde a primeira, no lado `side`; é o clic número `clicks` seguido, con Maiús (`shift`) ou sen ela:

| Clics | Que fai | E despois, ao arrastrar |
|---|---|---|
| 1 | Leva o cursor ao sitio máis preto ([`stop_at`](#do-rato-ao-texto-stop_at-word_under-e-cell_of)), sen selección; con Maiús, move ata alí o final da selección, desde onde empezou. | `Clusters` |
| 2 | Selecciona a palabra, o grupo de signos ou o de espazos debaixo do punteiro ([`word_under`](#do-rato-ao-texto-stop_at-word_under-e-cell_of)), co cursor ao final; despois do final do texto, a última palabra. Sen texto con ancho, nada. | `Words(palabra)` |
| 3 | Selecciona todo. | `All` |

- `column` é un `usize`: o botón baixa dentro do campo. `isize::try_from` convérteo no tipo de `drag`, que pode ser negativo; un número que non cabe, imposíbel nunha xanela, queda en `isize::MAX`.
- Maiús só conta co clic simple. Calquera outro número de clics, que `Clicks` non dá, é coma un.
- Cada `select` que cambia algo pecha o paso que se estaba a escribir; un clic onde xa está o cursor non cambia nada, e o escrito segue no mesmo paso (`a_click_somewhere_else_ends_the_step_of_the_typing`).

**`drag`**: o punteiro moveuse á cela `column`, no lado `side`, co botón abaixo desde `press`. A cela pode estar fóra do campo, antes da primeira, cun número negativo, ou despois da última:

- **`Clusters`**: a selección vai desde onde empezou ata o sitio máis preto do punteiro, tamén cara atrás.
- **`Words(word)`**: palabras enteiras, coma Chrome. Se o punteiro está na palabra do dobre clic ou despois dela, a selección vai desde o principio desta ata o final da palabra debaixo do punteiro; se está antes, desde o final da do dobre clic ata o principio da do punteiro, co cursor á esquerda. A palabra do dobre clic queda sempre enteira.
- **`None`** e **`All`**: nada.
- **Fóra do campo, mostra máis.** `drag` non despraza nada el mesmo: leva o cursor a un sitio fóra da parte que se ve, e `track` móvea o xusto para velo ([`follow_caret`](#seguir-o-cursor-track-e-follow_caret)). Como a cela se conta desde a primeira que se mostra, que acaba de moverse, cada chamada fóra do campo vai un pouco máis alá. O campo non ten temporizador: Chrome move o texto mentres o punteiro está quedo fóra da caixa; o campo, só cando quen o usa chama a `drag`.
- `self.drag.clone()`: o `Drag` cópiase antes, porque os casos chaman a `select`, que cambia o campo.

### Seleccionar: `select`

```rust
fn select(&mut self, anchor: usize, caret: usize) -> bool {
    if (anchor, caret) == (self.anchor, self.caret) {
        return false;
    }
    self.track(|field| {
        (field.anchor, field.caret) = (anchor, caret);
        field.history.close();
    });
    true
}
```

**`select`** move o principio da selección a `anchor` e o cursor a `caret`. Se xa estaban alí, non fai nada e devolve `false`. Se non, dentro de [`track`](#seguir-o-cursor-track-e-follow_caret), móveos e pecha o paso de desfacer: calquera movemento do cursor ou da selección fai que a seguinte edición empece outro paso ([ADR 0030](../adr/0030-o-campo-de-texto.md), decisión 4). Devolve `true`.

- Pasan por ela todos os movementos: `select_all`, o rato (`press` e `drag`) e as teclas ([`move_caret`](#mover-o-cursor-step-go_to_end-e-move_caret)). As teclas devólvenlle a súa resposta a [`repeat`](#as-ordes-command-e-repeat), que para cando xa non cambia nada.
- Non mira se as posicións son sitios onde para o cursor: quen a chama sempre as dá así.

### Do rato ao texto: `stop_at`, `word_under` e `cell_of`

```rust
fn stop_at(&self, column: isize, side: Side) -> usize {
    let Some(cell) = self.cell_of(column) else {
        return 0;
    };
    // In halves of a cell.
    let half = 2 * cell + usize::from(side == Side::Right);
    let mut start = 0;
    for (at, cluster) in self.text.grapheme_indices(true) {
        let width = text::width(cluster);
        if half < 2 * start + width {
            return at;
        }
        start += width;
    }
    self.text.len()
}
```

**`stop_at`** dá o sitio onde para o cursor máis preto do lado `side` da cela `column` do campo, que pode estar fóra del:

- [`cell_of`](#do-rato-ao-texto-stop_at-word_under-e-cell_of) dá a cela do texto. Antes do principio do texto, o principio.
- **En metades de cela.** Cada cela ten dúas metades, a esquerda, `2 × cela`, e a dereita, `2 × cela + 1`: `half` é a do punteiro. Un grafema que empeza na cela `start` e ocupa `width` celas vai da metade `2 × start` á `2 × (start + width)`, e o seu medio está en `2 × start + width`. Se o punteiro está antes dese medio, o cursor vai antes do grafema (`at`); se non, despois, e a busca segue co seguinte.
- Despois do último grafema, o final do texto.

Así, o cursor vai antes ou despois de cada grafema segundo a metade da cela onde está o punteiro:

| Grafema | Antes del | Despois del |
|---|---|---|
| Dunha cela, coma `a` | A metade esquerda. | A dereita. |
| Dun carácter ancho, coma `中`, de dúas celas | As dúas metades da primeira cela. | As dúas da segunda. |
| De varias celas, coma unha familia de emoji, que ata a 8.1 ocupa as celas de cada emoji | As metades da primeira metade das súas celas. | As da segunda. |
| Sen ancho, coma o espazo de ancho cero (U+200B) | Ningunha. | Todas: un clic vai despois del. |

Na proba `a_click_puts_the_caret_on_the_nearest_side_of_the_cell`, co texto `a中b`, que ocupa catro celas:

| Cela e lado | Cursor | Por que |
|---|---|---|
| 0, esquerdo | 0 | Na metade esquerda do `a`: antes del. |
| 0, dereito | 1 | Na dereita: despois. |
| 1, esquerdo ou dereito | 1 | Na primeira cela de `中`: antes del. |
| 2, esquerdo ou dereito | 4 | Na segunda: despois. |
| 3, esquerdo | 4 | Na metade esquerda do `b`. |
| 3, dereito | 5 | Na dereita: o final. |
| 4 ou 9 | 5 | Despois do texto. |

- `grapheme_indices(true)` dá os grafemas estendidos, os de UAX #29, cada un co byte onde empeza; e [`text::width`](#cratesmeiga-coresrctextrs), as celas de cada un, a suma das dos seus caracteres.

```rust
fn word_under(&self, column: isize) -> Range<usize> {
    let cell = self.cell_of(column).unwrap_or(0);
    let mut start = 0;
    let mut last = None;
    for (at, cluster) in self.text.grapheme_indices(true) {
        let width = text::width(cluster);
        if width > 0 {
            last = Some(at);
            if cell < start + width {
                break;
            }
        }
        start += width;
    }
    last.map_or(self.text.len()..self.text.len(), |at| {
        word_at(&self.text, at)
    })
}

fn cell_of(&self, column: isize) -> Option<usize> {
    self.scroll.checked_add_signed(column)
}
```

- **`word_under`**: o grupo de grafemas da mesma clase ca o da cela `column`, o que selecciona un dobre clic: unha palabra, un grupo de signos ou un de espazos ([`word_at`](#as-palabras-next_word-previous_word-e-word_at)). Percorre os grafemas con ancho, os que poden estar debaixo do punteiro: `last` é o último visto, e o percorrido para no que ten a cela. Antes do principio do texto, a cela é a 0, e o grafema, o primeiro; despois do final, `last` queda no último. Sen ningún grafema con ancho, un rango baleiro ao final: nada.
- **`cell_of`**: a cela do texto que está na cela `column` do campo, `scroll` máis `column`. `checked_add_signed` súmalle a un `usize` un número con signo, e dá `None` se o resultado sería negativo: antes do principio do texto.

### Editar: `replace` e `edited`

```rust
fn replace(&mut self, range: Range<usize>, with: &str, kind: Kind) {
    let removed = self.text[range.clone()].to_owned();
    let before = (self.anchor, self.caret);
    self.text.replace_range(range.clone(), with);
    self.edited(removed != with);
    // What it inserted can join the text around it in another cluster:
    // the caret goes to its end, or, after deleting, to its start.
    let end = range.start + with.len();
    let caret = if is_stop(&self.text, end) {
        end
    } else if with.is_empty() {
        previous_cluster(&self.text, end)
    } else {
        next_cluster(&self.text, end)
    };
    (self.anchor, self.caret) = (caret, caret);
    self.history.record(Edit {
        at: range.start,
        removed,
        inserted: with.to_owned(),
        before,
        after: caret,
        kind,
    });
}

fn edited(&mut self, changed: bool) {
    self.changes.text |= changed;
    self.drag = Drag::None;
}
```

**`replace`** cambia o rango `range` do texto por `with`, e garda a edición para desfacela. Por ela pasan todas as edicións, menos desfacer e refacer: escribir, borrar e pegar. Quen a chama xa mirou que `with` cabe, e envólvea en [`track`](#seguir-o-cursor-track-e-follow_caret). Paso a paso:

1. Garda unha copia do que quita, `removed`, e a selección de antes, `before`.
2. `replace_range` cambia o texto no seu sitio.
3. [`edited`](#editar-replace-e-edited) anota se o texto é outro, se o que quitou e o que puxo non son iguais, e remata o arrastre.
4. **O cursor**, ao final do que puxo: `range.start + with.len()`, que ao borrar, con `with` baleiro, é o principio do rango. Pero o que puxo, ou o oco que deixou, pode xuntar o texto de arredor noutro grafema, e entón ese sitio xa non é un onde pare o cursor ([`is_stop`](#onde-para-o-cursor-next_cluster-previous_cluster-e-is_stop)). Ao poñer texto, o cursor vai ao final dese grafema ([`next_cluster`](#onde-para-o-cursor-next_cluster-previous_cluster-e-is_stop)); e ao borrar, ao seu principio ([`previous_cluster`](#onde-para-o-cursor-next_cluster-previous_cluster-e-is_stop)).
5. Sen selección: o principio da selección vai co cursor.
6. [`record`](#gardar-unha-edición-record-e-close) garda a edición: onde, o que quitou e o que puxo, a selección de antes, o cursor de despois e o seu tipo, `kind`, que di se a seguinte se pode xuntar con ela.

| Antes | Tecla | Despois | Por que |
|---|---|---|---|
| `e\|x` | Un acento combinado (U+0301), escrito | `e\u{301}\|x` | O acento xúntase co `e` nun grafema, `é`, e despois del o cursor pode parar. |
| `\|🇪🇸` | 🇫, escrito | `🇫🇪\|🇸` | O 🇫 fai unha bandeira co 🇪, e o 🇸 queda só: o cursor vai despois da bandeira nova. |
| `🇪🇸\|🇫🇷` | Retroceso | `\|🇪🇫🇷` | Sen o 🇸, o 🇪 e o 🇫 fan unha bandeira: o cursor vai antes dela. |
| `🇪\|x🇸` | Supr | `\|🇪🇸` | Sen o `x`, os dous indicadores fan unha bandeira. |

- **Cando o cursor salta, o paso pecha.** Se o cursor saltou ao final ou ao principio dun grafema, a seguinte edición xa non empeza onde acabou esta, e [`join`](#xuntar-dúas-edicións-join) non as xunta.
- **«`range` ou `with` non está baleiro»**, coma di o comentario: toda edición quita ou pon algo. `delete` mira que o rango non estea baleiro, `paste` que o texto que pega tampouco, e escribir sempre pon un carácter. Ata `67295da` había unha garda para unha edición que non fixese nin unha cousa nin a outra; a cobertura das probas mostrou que ningunha chamada chegaba a ela, e quitouse.

**`edited`**: o texto editouse, e é outro se `changed`. Escribir un carácter enriba do mesmo, seleccionado, déixao coma estaba, e entón non é un cambio do texto, coma co mesmo texto en `set_text`; `|=` marca o cambio, e nunca desmarca outro de antes que aínda non se colleu. E remata o que selecciona o botón abaixo: os sitios que colleu, coma a palabra dun dobre clic, poden non ser xa os mesmos, coma di o seu comentario. Chámana `replace`, `undo` e `redo`; `set_text` fai o mesmo el só.

- **Desde `a99de8c`.** Ata entón, calquera edición era un cambio do texto, tamén a que o deixaba igual, ao contrario de `set_text`. Atopouno a revisión independente do código, e compróbao o final de `the_changes_say_what_changed`, que falla sen o arranxo.
- **O arrastre.** A proba ao chou atopou este fallo mentres se escribía o campo: despois dun dobre clic, se o texto cambiaba co botón abaixo, o arrastre seguía seleccionando cos extremos da primeira palabra nun texto que xa non era o mesmo, onde podían quedar no medio dun grafema ou despois do final. Agora compróbao `a_change_of_the_text_ends_what_the_button_held_down_selects`.

### Desfacer e refacer: `undo` e `redo`

```rust
fn undo(&mut self) -> bool {
    let Some(edit) = self.history.undo() else {
        return false;
    };
    let Edit {
        at,
        ref removed,
        ref inserted,
        before,
        ..
    } = *edit;
    let changed = removed != inserted;
    self.text.replace_range(at..at + inserted.len(), removed);
    (self.anchor, self.caret) = before;
    self.edited(changed);
    true
}

fn redo(&mut self) -> bool {
    let Some(edit) = self.history.redo() else {
        return false;
    };
    let Edit {
        at,
        ref removed,
        ref inserted,
        after,
        ..
    } = *edit;
    let changed = removed != inserted;
    self.text.replace_range(at..at + removed.len(), inserted);
    (self.anchor, self.caret) = (after, after);
    self.edited(changed);
    true
}
```

- **`undo`** desfai o último paso: [`History::undo`](#os-pasos-desfeitos-undo-redo-e-clear) dállo, e pásao aos que se poden refacer. Pon outra vez o que quitou, `removed`, en vez do que puxo, `inserted`, e a selección de antes do paso, `before`: desfacer o escrito enriba dunha selección volta seleccionala ([ADR 0030](../adr/0030-o-campo-de-texto.md), decisión 4).
- **`redo`** refai o último paso desfeito: pon outra vez `inserted` en vez de `removed`, e o cursor onde quedou despois do paso, `after`, sen selección.
- **`changed`**: o paso cambia o texto se o que quitou e o que puxo non son iguais. Desfacer ou refacer o escrito enriba do mesmo texto non é un cambio do texto, desde `a99de8c` ([`edited`](#editar-replace-e-edited)).
- Os dous devolven se había un paso: [`undo_or_redo`](#o-portapapeis-e-desfacer-copy-cut-e-undo_or_redo), que os envolve en `track`, e as repeticións das teclas paran cando xa non hai máis.
- `let Edit { at, ref removed, ref inserted, before, .. } = *edit;` desfai nos seus campos a edición prestada: `at` e `before` cópianse, e `ref` presta os textos sen copialos. Rust deixa ler a edición, que é de `self.history`, mentres cambia `self.text`, porque son campos distintos.
- Os sitios que volta pór son os que había antes ou despois do paso, no mesmo texto, así que seguen sendo sitios onde para o cursor.

### O límite: `room`

```rust
fn room(&self) -> usize {
    let selected = units(self.selected_text());
    self.limit.saturating_sub(units(&self.text) - selected)
}
```

**`room`**: as unidades de UTF-16 que caben en vez da selección: o límite menos as do texto que queda, todo menos o seleccionado ([ADR 0030](../adr/0030-o-campo-de-texto.md), decisión 6). As unidades cóntanse coma as conta Windows, e coma as conta [`MAX_TITLE`](#os-límites-saved_titles-e-max_title) cos títulos: `x`, `ñ` ou `中`, unha; un emoji de fóra do plano básico, coma `😀`, dúas. Pregúntano escribir ([`type_character`](#os-caracteres-typed-e-type_character)) e pegar; `set_text` corta ao límite enteiro. `saturating_sub` daría 0 se o texto pasase do límite, que nunca pasa.

### Seguir o cursor: `track` e `follow_caret`

```rust
fn track<R>(&mut self, change: impl FnOnce(&mut Self) -> R) -> R {
    let before = (self.anchor, self.caret, self.scroll, self.columns);
    let result = change(self);
    self.follow_caret();
    if self.changes.text || (self.anchor, self.caret, self.scroll, self.columns) != before {
        self.changes.shown = true;
    }
    result
}
```

**`track`** fai un cambio, `change`, e despois pon ao día a parte do texto que se mostra ([`follow_caret`](#seguir-o-cursor-track-e-follow_caret)) e anota se cambiou o que se mostra: o texto, o principio da selección, o cursor, a primeira cela que se ve ou, desde `b96fa25`, o número de celas ([`set_columns`](#os-cambios-e-as-celas-take_changes-e-set_columns)). Todos os cambios do campo pasan por ela: `set_text`, `set_columns`, `select`, as edicións das teclas e de `paste`, e desfacer e refacer. É xenérica no resultado do cambio, `R`, que devolve: así `undo_or_redo` sabe se había un paso.

```rust
fn follow_caret(&mut self) {
    if self.columns == 0 {
        return;
    }
    let caret = text::width(&self.text[..self.caret]);
    let total = text::width(&self.text);
    let mut scroll = self
        .scroll
        .clamp((caret + 1).saturating_sub(self.columns), caret)
        .min((total + 1).saturating_sub(self.columns));
    if matches!(
        text::cells(&self.text).nth(scroll),
        Some(CellChar::WideRight(_))
    ) {
        scroll += 1;
    }
    self.scroll = scroll;
}
```

**`follow_caret`** move a parte do texto que se mostra o xusto para que se vexa o cursor ([ADR 0030](../adr/0030-o-campo-de-texto.md), decisión 7). Todo en celas: `caret`, as do texto antes do cursor, e `total`, as do texto. O campo mostra `columns` celas desde `scroll`, e o cursor vai no lado esquerdo da súa cela, así que se ve se `scroll ≤ caret ≤ scroll + columns − 1`. Paso a paso:

1. **O xusto para ver o cursor.** `clamp` deixa `scroll` entre `caret + 1 − columns` e `caret`: se o cursor quedou á esquerda da parte que se ve, esta empeza nel; se quedou á dereita, avanza ata que o cursor está na última cela; e se se ve, non se move. Ao final do texto, esa última cela é unha cela libre despois do texto, para o cursor. `saturating_sub` dá 0 ao principio.
2. **Nunca máis alá do final.** `min` non deixa que empece despois de `total + 1 − columns`: as celas do texto, a cela libre, e nada máis. Cando o texto se fai máis curto, con Retroceso ao final ou borrando unha selección, a parte que se ve volta cara atrás, e non quedan celas baleiras á dereita mentres hai texto á esquerda.
3. **Á esquerda, nunca media letra ancha.** Se a primeira cela que se ve é a metade dereita dun carácter ancho, empeza na seguinte: o campo mostra unha cela menos de texto. O cursor nunca está entre as dúas metades, así que segue á vista.

- **Sen celas,** o campo non mostra nada, e a parte do texto queda onde estaba ata que volta ter celas, coma di o seu comentario: entón móvese desde alí. A volta antes de facer as contas segue facendo falta: con 0 celas, o mínimo do `clamp`, `caret + 1`, pasaría do máximo, `caret`, e entón `clamp` falla (*panic*). Ata `50d2379`, sen celas, a parte que se ve voltaba ao principio, algo que nada podía ver; atopouno o axente da documentación.
- **Un `clamp`, desde `cc18533`.** Antes eran dúas comparacións, e un mutante, `<` por `<=` en `caret < scroll`, sobrevivía ás probas: co cursor na primeira cela que se ve, `scroll = caret` non cambiaba nada, así que era equivalente. O `clamp` di o mesmo nun paso, sen esa comparación ([Tecnoloxías › A cobertura e os mutantes](../tecnoloxias.md#64-a-cobertura-e-os-mutantes)).
- Cada chamada mide o texto enteiro (`text::width`) e busca a primeira cela que se ve (`nth`): nunha liña de como moito 1000 unidades, pouco.

Por exemplo, nun campo de cinco celas, escribindo `abcdefgh` (`a_text_that_does_not_fit_shows_the_part_with_the_caret`):

| Texto e cursor | `scroll` | O campo mostra |
|---|---|---|
| `abcd\|` | 0 | `abcd`, e o cursor na cela libre. |
| `abcde\|` | 1 | `bcde`, e o cursor na cela libre. |
| `abcdefgh\|` | 4 | `efgh`, e o cursor na cela libre. |
| Inicio: `\|abcdefgh` | 0 | `abcde`, co cursor na primeira cela. |
| Fin e catro veces ←: `abcd\|efgh` | 4 | `efgh`, co cursor na primeira cela. |
| Outra ←: `abc\|defgh` | 3 | `defgh`, co cursor na primeira cela. |

### Pegar nunha liña: `one_line`, `fitting` e `units`

```rust
fn one_line(text: &str) -> String {
    let mut line = String::with_capacity(text.len());
    let mut after_cr = false;
    for c in text.chars() {
        match c {
            '\n' if after_cr => {}
            '\r' | '\n' | '\t' | '\u{b}' | '\u{c}' | '\u{85}' | '\u{2028}' | '\u{2029}' => {
                line.push(' ');
            }
            c if c.is_control() => {}
            c => line.push(c),
        }
        after_cr = c == '\r';
    }
    line
}
```

**`one_line`**: `text` nunha liña ([ADR 0030](../adr/0030-o-campo-de-texto.md), decisión 5). Úsana `set_text` e `paste`:

| Carácter | Queda |
|---|---|
| CR LF | Un espazo. |
| CR, LF, VT, FF, NEL (U+0085), o separador de liñas (LS, U+2028) e o de parágrafos (PS, U+2029) | Un espazo cada un. |
| O tabulador | Un espazo. |
| Os outros caracteres de control (`char::is_control`: de U+0000 a U+001F e de U+007F a U+009F), coma ESC ou BEL | Nada. |
| O resto | Tal cal. |

- `after_cr` lembra se o carácter anterior era un CR: o LF que o segue é o mesmo salto de liña, o CR LF de Windows, e non deixa outro espazo. Un LF e despois un CR son dous.
- Os caracteres de formato, coma U+200B ou o ZWJ, non son de control: quedan. O ZWJ xunta os emoji, coma nunha familia.
- `with_capacity(text.len())`: a liña nunca é máis longa ca o texto, porque cada salto de liña ocupa polo menos un byte e queda nun. Non reserva memoria máis dunha vez.
- Percorre o texto enteiro, aínda que despois `fitting` só colla o que cabe.

```rust
fn fitting(text: &str, room: usize) -> &str {
    let mut units_so_far = 0;
    let mut end = 0;
    for (at, cluster) in text.grapheme_indices(true) {
        units_so_far += units(cluster);
        if units_so_far > room {
            break;
        }
        end = at + cluster.len();
    }
    &text[..end]
}

fn units(text: &str) -> usize {
    text.chars().map(char::len_utf16).sum()
}
```

- **`fitting`**: o principio máis longo de `text`, en grafemas enteiros, que ten como moito `room` unidades de UTF-16. Suma as unidades de cada grafema, e para no primeiro que xa non cabe, sen percorrer o resto. Por exemplo, con sitio para tres, de `cd👍🏽e` colle `cd`: o 👍🏽, o polgar co ton de pel, son catro unidades, e non cabe; e o `e`, aínda que cabería, vén despois. Úsana `set_text`, co límite, e `paste`, co sitio que queda.
- **`units`**: as unidades de UTF-16 de `text`, sen codificalo: `len_utf16` dá 1 ou 2 por carácter.

**Probas.** As do campo están en [`tests.rs`](#cratesmeiga-coresrcuifieldtestsrs) (`#[cfg(test)] mod tests;`, ao final do arquivo); as dos grafemas e as palabras, en [`segment.rs`](#cratesmeiga-coresrcuifieldsegmentrs), e as dos pasos, en [`history.rs`](#cratesmeiga-coresrcuifieldhistoryrs).

## `crates/meiga-core/src/ui/field/segment.rs`

Desde a subfase 3.2: onde para o cursor dun campo de texto, e onde empezan as súas palabras ([ADR 0030](../adr/0030-o-campo-de-texto.md), decisións 2 e 3; [Tecnoloxías › Os grafemas e as palabras dun campo de texto](../tecnoloxias.md#66-os-grafemas-e-as-palabras-dun-campo-de-texto)). Son funcións sen estado, que reciben un texto e unha posición en bytes. Os grafemas dáos `unicode-segmentation`, coas regras de UAX #29 de Unicode 17; as palabras teñen regras propias, máis simples ca as de UAX #29, para os nomes e os filtros de arquivos: `main.rs` son tres palabras, e Ctrl+Retroceso borra `rs`, e non o nome enteiro.

```rust
//! Where the caret of a text field stops, and where its words start.
//!
//! The caret stops between the grapheme clusters of Unicode (UAX #29), the
//! extended ones, as in Chrome: never between a letter and its accent, nor
//! inside an emoji with a skin tone, a flag or a sequence joined with a
//! zero-width joiner (ZWJ).
//!
//! A word is a run of clusters of the same class: letters, digits and `_`
//! make words; each run of other signs is another word; and the spaces
//! separate them. So `main.rs` is three words, and `C:\Users\x` five. Each
//! cluster has the class of its first character.

use std::ops::Range;

use unicode_segmentation::GraphemeCursor;
use unicode_width::UnicodeWidthChar;
```

- O comentario do módulo resume as dúas cousas: o cursor para entre os grafemas estendidos de UAX #29, coma en Chrome, e nunca entre unha letra e o seu acento, nin dentro dun emoji cun ton de pel, dunha bandeira ou dunha secuencia unida cun ZWJ; e unha palabra é un grupo de grafemas da mesma clase, a do seu primeiro carácter. `C:\Users\x` son cinco: `C`, `:\`, `Users`, `\` e `x`.
- `GraphemeCursor`, de `unicode-segmentation`, busca os límites dos grafemas; e `UnicodeWidthChar`, de `unicode-width`, dá o ancho dun carácter, para a súa clase.
- O que usan os outros arquivos é `pub(super)`: só o ven `field` e os seus submódulos. `class` é privada.

### Onde para o cursor: `next_cluster`, `previous_cluster` e `is_stop`

```rust
pub(super) fn next_cluster(text: &str, at: usize) -> usize {
    // With the whole text, the cursor never asks for more of it.
    GraphemeCursor::new(at, text.len(), true)
        .next_boundary(text, 0)
        .ok()
        .flatten()
        .unwrap_or(text.len())
}

pub(super) fn previous_cluster(text: &str, at: usize) -> usize {
    GraphemeCursor::new(at, text.len(), true)
        .prev_boundary(text, 0)
        .ok()
        .flatten()
        .unwrap_or(0)
}

pub(super) fn is_stop(text: &str, at: usize) -> bool {
    GraphemeCursor::new(at, text.len(), true)
        .is_boundary(text, 0)
        .unwrap_or(true)
}
```

| Función | Que dá | Quen a usa |
|---|---|---|
| `next_cluster` | O final do grafema que empeza en `at`, ou que ten `at` dentro: o seguinte sitio onde para o cursor, ou o final do texto. | → e Maiús+→, Supr, as palabras, e o cursor despois de poñer texto ([`replace`](#editar-replace-e-edited)). |
| `previous_cluster` | O principio do grafema que acaba en `at`, ou que ten `at` dentro: o sitio anterior onde para o cursor, ou 0. | ← e Maiús+←, as palabras, e o cursor despois de borrar. |
| `is_stop` | Se o cursor pode parar en `at`: entre dous grafemas, ou nun extremo do texto. | `replace`, e as probas. |

- **`GraphemeCursor`** é un cursor sobre un texto de `text.len()` bytes, en `at`. `true` pide os grafemas estendidos, os que usan os editores de hoxe: cos antigos (*legacy*), o signo vogal de `हि` quedaría fóra da súa consoante. `next_boundary` dá o seguinte límite, `prev_boundary` o anterior, e `is_boundary` di se en `at` hai un.
- **O texto enteiro, nun anaco.** O cursor recibe o texto a anacos, e pide outro, cun erro (`GraphemeIncomplete`), se o necesita para decidir: por exemplo, cantos indicadores rexionais hai antes. O campo dálle o texto enteiro, desde 0, así que nunca pide máis, coma di o comentario. `ok().flatten()` converte o `Result<Option<usize>>` nun `Option<usize>`, que é `None` nos extremos do texto: entón, o final ou 0. E `is_stop` daría `true` cun erro que non pode chegar.
- **Desde dentro dun grafema.** `replace` chama a `next_cluster` cun sitio onde o cursor non para, e recibe o final do grafema que o ten dentro. Os comentarios das dúas din os dous casos; o de `next_cluster`, desde `50d2379`, cando o atopou o axente da documentación.
- Cada chamada fai un cursor novo, que mira os caracteres arredor de `at`, sen reservar memoria. `at` ten que ser o límite dun carácter, e sempre o é.

### As clases: `Class`, `class`, `class_after` e `class_before`

```rust
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Class {
    Space,
    Word,
    Sign,
}

fn class(cluster: &str) -> Class {
    match cluster.chars().next() {
        Some(c) if c.is_whitespace() => Class::Space,
        Some(c) if c.is_alphanumeric() || c == '_' || c.width() == Some(0) => Class::Word,
        _ => Class::Sign,
    }
}

pub(super) fn class_after(text: &str, at: usize) -> Option<Class> {
    (at < text.len()).then(|| class(&text[at..next_cluster(text, at)]))
}

pub(super) fn class_before(text: &str, at: usize) -> Option<Class> {
    (at > 0).then(|| class(&text[previous_cluster(text, at)..at]))
}
```

Cada grafema ten unha clase, a do seu primeiro carácter ([ADR 0030](../adr/0030-o-campo-de-texto.md), decisión 3):

| Clase | Os grafemas que empezan por | Por exemplo |
|---|---|---|
| `Space` | Un espazo de Unicode (`char::is_whitespace`). | O espazo, o tabulador, o espazo que non parte (U+00A0) e o ideográfico (U+3000). |
| `Word` | Unha letra ou un número de calquera alfabeto (`char::is_alphanumeric`), `_`, ou un carácter sen ancho (`width() == Some(0)`). | `a`, `Ñ`, `7`, `_`, `中`, `ά`, e un acento combinado só. |
| `Sign` | Calquera outro. | `.`, `\`, `-`, `😀` e `€`. |

- **A clase do primeiro carácter.** Un `é` escrito coma `e` e un acento é `Word`, coma o `e`; un 👍🏽 é `Sign`, coma o 👍; e un espazo cun acento detrás, que UAX #29 xunta nun grafema, é `Space`.
- **Os caracteres sen ancho.** Coma di o comentario de `Word`, `class` pon en `Word` calquera carácter sen ancho ao principio dun grafema. Un acento combinado case nunca empeza un: só ao principio do texto, ou despois dun carácter que fai un grafema propio; en calquera outro sitio, xúntase co carácter anterior. O espazo de ancho cero (U+200B), en cambio, fai sempre un grafema propio, así que `a\u{200b}b` é unha palabra. Ata `50d2379`, o comentario só falaba dos acentos sen letra antes; atopouno o axente da documentación.
- **`class_after`**: a clase do grafema que empeza en `at`, ou `None` ao final do texto. **`class_before`**: a do que acaba en `at`, ou `None` ao principio. `bool::then` só calcula a clase se a condición se cumpre.

### As palabras: `next_word`, `previous_word` e `word_at`

```rust
pub(super) fn next_word(text: &str, at: usize) -> usize {
    let mut at = at;
    if let Some(class) = class_after(text, at).filter(|&class| class != Class::Space) {
        while class_after(text, at) == Some(class) {
            at = next_cluster(text, at);
        }
    }
    while class_after(text, at) == Some(Class::Space) {
        at = next_cluster(text, at);
    }
    at
}

pub(super) fn previous_word(text: &str, at: usize) -> usize {
    let mut at = at;
    while class_before(text, at) == Some(Class::Space) {
        at = previous_cluster(text, at);
    }
    if let Some(class) = class_before(text, at) {
        while class_before(text, at) == Some(class) {
            at = previous_cluster(text, at);
        }
    }
    at
}

pub(super) fn word_at(text: &str, at: usize) -> Range<usize> {
    let Some(class) = class_after(text, at) else {
        return at..at;
    };
    let mut start = at;
    while class_before(text, start) == Some(class) {
        start = previous_cluster(text, start);
    }
    let mut end = at;
    while class_after(text, end) == Some(class) {
        end = next_cluster(text, end);
    }
    start..end
}
```

- **`next_word`**, para Ctrl+→ e Ctrl+Supr: se `at` está ao principio ou no medio dunha palabra ou dun grupo de signos, pasa o que queda del; despois, os espazos. Queda no principio da palabra seguinte, coma en Windows e en Chrome en Windows, ou no final do texto. Desde uns espazos, só os pasa. `filter` colle a clase só se non é `Space`.
- **`previous_word`**, para Ctrl+← e Ctrl+Retroceso: volta por riba dos espazos de antes, e despois do grupo que hai antes deles, ata o seu principio; ou ata o principio do texto.
- **`word_at`**, para o dobre clic: o grupo da mesma clase ca o grafema que empeza en `at`, cara atrás e cara a diante. Ao final do texto, nada.
- Cada paso pregunta a clase dun grafema e despois busca o seu final, así que busca dúas veces o mesmo límite: nunha liña, abonda.

Onde paran Ctrl+→, desde o principio, e Ctrl+←, desde o final, que son os mesmos sitios, marcados con `|` (`ctrl_and_the_arrows_stop_at_the_start_of_each_word`):

```text
|foo |bar|
|  |foo   |bar  |
|main|.|rs|
|C|:\|Users|\|x|
|foo_bar|-|baz|
|...!|
|olá|, |mundo|
|a |😀😀 |b|
```

O dobre clic do terminal ten outras palabras: alí unha ruta ou un enderezo enteiros son unha, para copialos enteiros ([`word_at`](#o-dobre-clic-word_at), de `term/selection.rs`; [ADR 0030 › Alternativas consideradas](../adr/0030-o-campo-de-texto.md#alternativas-consideradas)).

**Probas.** Seis, coas funcións soas, sen campo. Tres axudantes: `stops` dá os sitios onde para o cursor desde o principio ata o final, un grafema cada vez, con `next_cluster`; `stops_back`, os mesmos desde o final, con `previous_cluster`; e `words`, onde para Ctrl+→ desde o principio, e onde para Ctrl+← desde o final.

| Proba | Que comproba |
|---|---|
| `the_caret_stops_between_clusters_in_both_directions` | Os sitios onde para o cursor, cara a diante e cara atrás, e `is_stop` en cada límite de carácter, en once textos: un baleiro; `ab`; `olá`, coa `á` xa composta, un só carácter; `e` e `n` con acentos combinados, dous grafemas; `中文`; 👍 cun ton de pel, e `!`; unha familia (home, ZWJ, muller, ZWJ e nena) e `x`; dúas bandeiras, 🇪🇸 e 🇬🇧, que van de dous en dous indicadores rexionais; ❤ co selector de variación dos emoji, e `a`; o conxunto devanágari `क्ष` (KA, VIRAMA e SSA), un só grafema; e tres *jamo* coreanos que fan unha sílaba. |
| `inside_a_cluster_the_caret_goes_to_its_ends` | Dentro dunha familia de emoji, despois do home, `previous_cluster` dá o principio da familia, e `next_cluster`, o seu final; e entre un `e` e o seu acento, 0 e 3. |
| `letters_digits_and_underscores_make_words_and_other_signs_another` | As clases, por diante e por detrás: o espazo, o tabulador, o espazo que non parte e o ideográfico, `Space`; `a`, `Ñ`, `7`, `_`, `中`, `ά` e un acento combinado só, `Word`; e `.`, `\`, `-`, `😀` e `€`, `Sign`. E ningunha despois do final nin antes do principio. |
| `ctrl_and_the_arrows_stop_at_the_start_of_each_word` | Os sitios de Ctrl+→ desde o principio, e os de Ctrl+← desde o final, que son os mesmos, en nove textos: os oito de enriba, e un baleiro. |
| `ctrl_and_the_arrows_go_from_the_middle_of_a_word_to_its_ends` | Desde o medio de `foo`, Ctrl+→ vai a `bar`; desde o medio de `bar`, Ctrl+← vai ao seu principio; e desde os espazos entre `foo` e `bar`, Ctrl+→ vai a `bar`, e Ctrl+← ao principio de `foo`. |
| `a_double_click_takes_the_run_of_its_class` | En `foo  bar.rs`, `word_at` en cada sitio: `foo` desde o seu principio e desde o seu medio, os dous espazos desde cada un, `bar`, `.`, `rs`, e nada ao final. Un grafema nunca se parte: en `x éé y`, cos acentos combinados, os dous `é` enteiros. E nada nun texto baleiro. |

## `crates/meiga-core/src/ui/field/history.rs`

Desde a subfase 3.2: os pasos que percorren desfacer e refacer nun campo de texto, coma os garda Chrome no seu `TextfieldModel` ([ADR 0030](../adr/0030-o-campo-de-texto.md), decisión 4). O campo garda cada edición con `record`, pecha o paso con `close` cando se move o cursor ou a selección, e colle os pasos con `undo` e `redo` para desfacelos e refacelos no texto ([`undo` e `redo`](#desfacer-e-refacer-undo-e-redo)). O historial non coñece o campo: só garda edicións, cos seus sitios e os seus textos, e próbase só.

```rust
//! The steps that undoing and redoing go through in a text field, as Chrome
//! keeps them (`TextfieldModel`): what each edit removed and inserted, and
//! the selection before it.
//!
//! An edit joins the one before it while nothing else happens between them:
//! the characters typed in a row are one step; so are the Backspaces in a
//! row, and the Deletes in a row. A paste, a cut, and deleting a selection
//! or a word are a step each. Moving the caret, selecting and undoing close
//! the last step: the next edit starts another.

use std::collections::VecDeque;
```

- O comentario do módulo di as regras: unha edición xúntase coa anterior mentres non pasa nada entre elas. Os caracteres escritos seguidos son un paso; os Retrocesos seguidos, outro; e os Supr seguidos, outro. Pegar, cortar e borrar unha selección ou unha palabra son un paso cada un. Mover o cursor, seleccionar e desfacer pechan o paso: a seguinte edición empeza outro. Refacer tamén o pecha.
- `VecDeque`, unha cola dobre, para os pasos feitos: os novos entran por un extremo, e o máis vello sae polo outro, sen mover os demais, coma os títulos gardados ([`Titles`](#titles)).

### `STEPS` e `Kind`

```rust
pub(super) const STEPS: usize = 100;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Kind {
    Typing,
    Backspace,
    Delete,
    Other,
}
```

- **`STEPS`**: os pasos que se poden desfacer, como moito 100 (decisión 4): un máis quita o máis vello. Cada paso garda o texto que quitou e o que puxo: cun límite de 1000 unidades de UTF-16, cada un deles ocupa como moito 3000 bytes, así que os 100 pasos dun campo, no peor caso, pegando unha e outra vez un texto enteiro en vez doutro, ocupan uns 600 KB; escribindo, moito menos.
- **`Kind`**, o tipo dunha edición, que di que se pode xuntar con ela:

| Tipo | Que edición | Que se xunta con ela |
|---|---|---|
| `Typing` | Escribir, tamén enriba dunha selección. | O seguinte carácter escrito onde acabou. |
| `Backspace` | Retroceso sen selección. | O seguinte Retroceso, xusto antes. |
| `Delete` | Supr sen selección. | O seguinte Supr, no mesmo sitio. |
| `Other` | Calquera outra: pegar, cortar, e borrar unha selección ou unha palabra. | Nada. |

### `Edit` e `History`

```rust
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct Edit {
    pub at: usize,
    pub removed: String,
    pub inserted: String,
    pub before: (usize, usize),
    pub after: usize,
    pub kind: Kind,
}

#[derive(Clone, Debug, Default)]
pub(super) struct History {
    done: VecDeque<Edit>,
    undone: Vec<Edit>,
    open: bool,
}
```

**`Edit`**: unha edición. En `at`, o texto tiña `removed`, e agora ten `inserted`:

| Campo | Que é |
|---|---|
| `at` | Onde empeza a edición: un desprazamento en bytes do texto. |
| `removed` | O texto que quitou; baleiro se só puxo. |
| `inserted` | O texto que puxo; baleiro se só quitou. |
| `before` | O principio da selección e o cursor antes da edición, que volta pór desfacela. |
| `after` | O cursor despois da edición, que volta pór refacela. |
| `kind` | O seu tipo. |

- Os campos son `pub` nun tipo `pub(super)`: lense desde `field`, que desfai e refai coas edicións.

**`History`**: os pasos.

| Campo | Que é |
|---|---|
| `done` | Os pasos feitos, que se poden desfacer: o máis vello primeiro. |
| `undone` | Os desfeitos, que se poden refacer: o último desfeito, ao final. |
| `open` | Se o último paso feito pode coller a seguinte edición. |

- Cada paso é unha `Edit`: as edicións que se xuntan fan unha soa, máis longa.
- `Default`: un historial baleiro, o dun campo novo. `Clone` e `Debug`, porque `Field` os deriva.

### Gardar unha edición: `record` e `close`

```rust
pub fn record(&mut self, edit: Edit) {
    self.undone.clear();
    let open = edit.kind != Kind::Other;
    if self.open
        && let Some(last) = self.done.back_mut()
        && join(last, &edit)
    {
        return;
    }
    if self.done.len() == STEPS {
        self.done.pop_front();
    }
    self.done.push_back(edit);
    self.open = open;
}

pub fn close(&mut self) {
    self.open = false;
}
```

**`record`** garda `edit`, xa feita no texto:

1. Os pasos desfeitos xa non se poden refacer: unha edición nova esquéceos (`a_new_edit_forgets_the_steps_undone`).
2. Se o último paso está aberto, e [`join`](#xuntar-dúas-edicións-join) pode xuntarlle `edit`, xa está: o paso medra.
3. Se non, `edit` é un paso novo, ao final; se xa había 100, antes sae o máis vello (`pop_front`). O paso novo queda aberto se se lle pode xuntar algo: se non é `Other`.

- `if self.open && let Some(last) = … && join(last, &edit)` encadea tres condicións (*let chains*, da edición 2024): `back_mut` dá o último paso para cambialo, e `join` só se chama se está aberto.

**`close`**: a seguinte edición empeza outro paso. Chámaa [`select`](#seleccionar-select), con cada movemento do cursor ou da selección.

### Os pasos desfeitos: `undo`, `redo` e `clear`

```rust
pub fn undo(&mut self) -> Option<&Edit> {
    self.open = false;
    let edit = self.done.pop_back()?;
    self.undone.push(edit);
    self.undone.last()
}

pub fn redo(&mut self) -> Option<&Edit> {
    self.open = false;
    let edit = self.undone.pop()?;
    self.done.push_back(edit);
    self.done.back()
}

pub fn clear(&mut self) {
    *self = Self::default();
}
```

- **`undo`** pecha o paso, colle o último feito, e pásao aos desfeitos; devolve unha referencia a el, para que o campo o desfaga no texto. Sen ningún, `None`: `?` volta antes.
- **`redo`** pecha o paso, colle o último desfeito, e pásao outra vez aos feitos.
- Os dous pechan o paso tamén se non había nada: escribir, Ctrl+Y sen nada que refacer, e escribir outra vez, son dous pasos (`undoing_and_redoing_close_the_last_step`). `redo` non mira o límite de 100: só volta pór o que `undo` quitou.
- **`clear`** esquece todo: `Self::default()`, un historial novo, que solta a memoria dos pasos. Chámaa [`set_text`](#cambiar-o-texto-set_text-select_all-e-paste).

### Xuntar dúas edicións: `join`

```rust
fn join(last: &mut Edit, next: &Edit) -> bool {
    match (last.kind, next.kind) {
        (Kind::Typing, Kind::Typing)
            if next.removed.is_empty() && last.at + last.inserted.len() == next.at =>
        {
            last.inserted.push_str(&next.inserted);
            last.after = next.after;
        }
        (Kind::Backspace, Kind::Backspace) if next.at + next.removed.len() == last.at => {
            last.removed.insert_str(0, &next.removed);
            last.at = next.at;
            last.after = next.after;
        }
        (Kind::Delete, Kind::Delete) if next.at == last.at => {
            last.removed.push_str(&next.removed);
            last.after = next.after;
        }
        _ => return false,
    }
    true
}
```

**`join`** xunta `next` ao paso `last`, se o continúa, e di se o fixo:

| O paso | A edición seguinte | Xúntanse se | O paso queda |
|---|---|---|---|
| `Typing` | `Typing`, sen quitar nada | Empeza onde acaba o que puxo o paso: `last.at + last.inserted.len() == next.at`. | Co texto novo despois do que puxo. |
| `Backspace` | `Backspace` | O que quita acaba onde empeza o paso: `next.at + next.removed.len() == last.at`. | Co texto que quita diante do que quitou, e empezando onde empeza a nova. |
| `Delete` | `Delete` | No mesmo sitio: `next.at == last.at`. | Co texto que quita despois do que quitou. |
| Calquera outro par | | Nunca. | |

- O paso colle sempre o cursor de despois da nova (`after`), e mantén a selección de antes da primeira (`before`): desfacer un paso volta pór a selección de antes de todo el.
- **Cando o cursor saltou** a outro grafema ([`replace`](#editar-replace-e-edited)), a edición seguinte xa non empeza onde tocaría, e non se xunta: empeza outro paso.
- **Cada tipo, só co seu:** escribir e despois Retroceso son dous pasos, e Retroceso e despois Supr, tamén.
- **A garda de `Typing`,** `next.removed.is_empty()`: escribir enriba dunha selección empeza un paso, e o que se escribe despois xúntase con el (`typing_over_a_selection_takes_the_next_characters`); pero un escrito que quita algo nunca continúa un paso, senón que empeza o seu. Desde o campo non chega ningún así, porque calquera selección pecha antes o paso; pero `History` ten as súas propias probas, que pasan pola garda, e esta mantén o que promete o historial por si só.

**Probas.** Once, co historial só, sen campo. `edit(tipo, at, quitado, posto)` fai unha edición coa selección de antes en `at` e o cursor de despois ao final do que puxo; `steps` desfai todos os pasos e dáos, o último primeiro, coma (`at`, quitado, posto); e `step` escribe un.

| Proba | Que comproba |
|---|---|
| `typing_in_a_row_is_one_step` | `o`, `l` e `á`, escritos seguidos, son un paso, `olá`. |
| `typing_over_a_selection_takes_the_next_characters` | `x`, escrito enriba de `abc`, e `y` despois son un paso: quita `abc`, pon `xy`, volta pór a selección de antes, (0, 3), e deixa o cursor despois do `y`. |
| `typing_somewhere_else_or_after_a_move_is_another_step` | Despois de `a`, `b` no mesmo sitio, e non onde acabou o `a`, é outro paso, e `c` xúntase con el; e `d`, despois de `close`, outro. |
| `backspaces_in_a_row_are_one_step_and_deletes_another` | Tres Retrocesos seguidos, que quitan `f`, `e` e `d`, son un paso, `def`. Un Supr despois deles, outro; un Supr noutro sitio, outro, e un máis no mesmo sitio xúntase con el, `bx`; e un Retroceso despois dos Supr, outro. |
| `a_backspace_that_does_not_end_where_the_last_one_started_is_another_step` | Dous Retrocesos que non se tocan, en 5 e en 2, son dous pasos. |
| `other_edits_are_one_step_each_and_nothing_joins_them` | `a` escrito, `pasted` pegado (`Other`), `b` escrito onde acabou o pegado, e dous `Other` máis, borrar todo e poñer `x`, son cinco pasos: nada se xunta cun `Other`. |
| `different_kinds_do_not_join` | Escribir e Retroceso, Retroceso e escribir, Supr e escribir, e escribir e Supr, no mesmo sitio, son dous pasos. |
| `undone_steps_can_be_redone_until_another_edit` | Dous pasos desfanse do último ao primeiro, e despois non hai máis; refacer trae o primeiro; e unha edición nova esquece o segundo, que xa non se pode refacer. |
| `undoing_and_redoing_close_the_last_step` | Despois de desfacer e refacer `ab`, `c` é outro paso; e despois dun `redo` sen nada que refacer, tamén. |
| `only_the_last_100_steps_can_be_undone` | De 101 pasos, desfanse 100, e o primeiro xa non está. |
| `clearing_forgets_everything` | Despois de `clear`, nin desfacer nin refacer dan nada, e o seguinte escrito non se xunta con ningún paso. |

## `crates/meiga-core/src/ui/field/keys.rs`

Desde a subfase 3.2: o que fan as teclas nun campo de texto, coma nos campos de texto de Chrome en Windows ([ADR 0030](../adr/0030-o-campo-de-texto.md), decisións 2, 3, 4 e 7). O campo recibe os eventos enteiros da xanela, as teclas que baixan e as que soben ([`KeyEvent`](#keyevent)): a tecla virtual, o carácter que escribe, en UTF-16, as teclas modificadoras e cantas veces se repetiu ([ADR 0014](../adr/0014-o-teclado.md); [Código › meiga-win › `keyboard.rs`](meiga-win.md#cratesmeiga-winsrckeyboardrs)). Así ve o mesmo ca o terminal: as teclas mortas, AltGr, os códigos de Alt, e o texto dun IME ou do panel de emoji, sen tecla. E responde cun [`Reply`](#reply-o-que-fixo-unha-tecla).

| Teclas | O que fan |
|---|---|
| ← → | Moven o cursor un grafema; sen Maiús, unha selección vaise, e o cursor queda no seu bordo dese lado. |
| Ctrl+← Ctrl+→ | Móveno ao principio da palabra anterior ou da seguinte. |
| Inicio Fin | Ao principio ou ao final do texto. |
| Maiús coas de enriba, Maiús+↑ Maiús+↓ | Seleccionan ata alí. |
| Retroceso Supr | Borran a selección, ou o último punto de código antes do cursor, ou o grafema despois del. |
| Ctrl+Retroceso Ctrl+Supr | Borran a selección, ou ata o principio da palabra anterior ou da seguinte. |
| Ctrl+A | Selecciona todo. |
| Ctrl+C Ctrl+Insert | Copian a selección. |
| Ctrl+X Maiús+Supr | Cortan a selección. |
| Ctrl+V Maiús+Insert | Pegan. |
| Ctrl+Z Alt+Retroceso | Desfán. |
| Ctrl+Y Ctrl+Maiús+Z Alt+Maiús+Retroceso | Refán. |

O arquivo engádelle métodos a `Field`, nun bloque `impl` propio, e só `key` é público. Os privados, coma `command` ou `typed`, defínense en `field::keys`, e só os ve este módulo; eles, en cambio, usan os privados de `mod.rs`, coma `select`, `replace` ou `track`.

```rust
//! What the keys do in a text field, as in the text fields of Chrome on
//! Windows (ADR 0030).
//!
//! | Keys | What they do |
//! |---|---|
//! | ← → | move the caret a cluster; without Shift, a selection goes away from its edge on that side |
//! | Ctrl+← Ctrl+→ | move it to the start of the previous or the next word |
//! | Home End | move it to the start or the end of the text |
//! | Shift and the keys above, Shift+↑ Shift+↓ | select up to there |
//! | Backspace Delete | delete the selection, or the last code point before the caret, or the cluster after it |
//! | Ctrl+Backspace Ctrl+Delete | delete the selection, or up to the start of the previous or the next word |
//! | Ctrl+A | select all |
//! | Ctrl+C Ctrl+Insert | copy the selection |
//! | Ctrl+X Shift+Delete | cut the selection |
//! | Ctrl+V Shift+Insert | paste |
//! | Ctrl+Z Alt+Backspace | undo |
//! | Ctrl+Y Ctrl+Shift+Z Alt+Shift+Backspace | redo |
//!
//! The other keys write their characters, also with AltGr, or with Ctrl+Alt,
//! which Windows takes as AltGr; with Alt alone, they write nothing, but Alt
//! writes the character of the code typed with it on the numeric keypad,
//! when it goes up. Control characters never go in. A key that the keyboard
//! repeats faster than the window takes it comes once with the count of its
//! repeats, and does its work that many times; but copying, cutting and
//! pasting, once, because its user reads the clipboard once for each key.
//! Enter, Esc, Tab, ↑, ↓ and the keys without a character do nothing: they
//! are for the user of the field.

use std::ops::Range;

use super::history::Kind;
use super::segment::{next_cluster, next_word, previous_cluster, previous_word};
use super::{Field, Reply};
use crate::input::{KeyEvent, own_character};
```

- O comentario do módulo ten a táboa de enriba e o resto: as outras teclas escriben os seus caracteres, tamén con AltGr, ou con Ctrl+Alt, que Windows toma coma AltGr; con Alt só non escriben nada, pero Alt escribe, ao subir, o carácter do código que se escribiu con el no teclado numérico; os caracteres de control nunca entran; unha tecla que o teclado repite máis rápido do que a colle a xanela chega unha vez coa conta das súas repeticións, e fai o seu traballo tantas veces, menos copiar, cortar e pegar, que se fan unha vez, porque quen usa o campo le o portapapeis unha vez por tecla; e Intro, Esc, Tab, ↑, ↓ e as teclas sen carácter non fan nada, que son de quen usa o campo.
- `use crate::input::{KeyEvent, own_character};`: o evento, e a regra que di se o carácter dunha tecla é seu ([`own_character`](#os-axudantes-das-secuencias)), a mesma ca a do terminal, que desde a 3.2 é `pub(crate)` para o campo.

### Os códigos das teclas

```rust
const VK_BACK: u16 = 0x08;
const VK_MENU: u16 = 0x12;
const VK_END: u16 = 0x23;
const VK_HOME: u16 = 0x24;
const VK_LEFT: u16 = 0x25;
const VK_UP: u16 = 0x26;
const VK_RIGHT: u16 = 0x27;
const VK_DOWN: u16 = 0x28;
const VK_INSERT: u16 = 0x2d;
const VK_DELETE: u16 = 0x2e;
const VK_A: u16 = 0x41;
const VK_C: u16 = 0x43;
const VK_V: u16 = 0x56;
const VK_X: u16 = 0x58;
const VK_Y: u16 = 0x59;
const VK_Z: u16 = 0x5a;
```

Os códigos de tecla virtual de Windows (`VK_…`) das teclas que usa o campo: Retroceso, Alt (`VK_MENU`), Fin, Inicio, as frechas, Insert, Supr e as letras das ordes, cuxos códigos son os das súas maiúsculas en ASCII. `meiga-core` non depende de `windows-sys`: coma `input.rs`, que ten os seus nun módulo privado, `vk`, o campo define os seus. O comentario do primeiro vale para todos.

### `key`: a entrada das teclas

```rust
pub fn key(&mut self, event: &KeyEvent) -> Reply {
    if !event.down {
        // Of the keys that go up, only Alt writes a character: the one
        // of the code typed with it on the numeric keypad.
        if event.key == VK_MENU && event.character != 0 {
            return self.typed(event.character, 1);
        }
        return Reply::Ignored;
    }
    // A character that is not of its key goes in as a character: the
    // one of a dead key that does not join with Backspace, Enter, Tab or
    // Esc, which Windows writes with them. The key comes next, with its
    // own character.
    if event.character != 0 && !own_character(event.key, event.character) {
        return self.typed(event.character, event.repeat);
    }
    let controls = event.controls;
    // AltGr, which is Ctrl+Alt for Windows, writes characters, and has
    // no commands.
    if !(controls.ctrl() && controls.alt())
        && let Some(reply) = self.command(event)
    {
        self.high_surrogate = None;
        return reply;
    }
    // The characters with Alt alone are for the menus of the window;
    // with Ctrl too, Windows takes Alt as AltGr.
    if event.character == 0 || (controls.alt() && !controls.ctrl()) {
        return Reply::Ignored;
    }
    self.typed(event.character, event.repeat)
}
```

**`key`** atende unha tecla que baixou ou subiu mentres o campo ten o teclado. Paso a paso:

1. **As teclas que soben** non fan nada, menos Alt cun carácter: o do código que se escribiu con el no teclado numérico, coma Alt+0241, que escribe `ñ` ao soltar Alt. Windows dálle ese carácter a Alt cando sobe, e as teclas do teclado numérico non escriben nada ([Código › meiga-win › `keyboard.rs`](meiga-win.md#cratesmeiga-winsrckeyboardrs)); co Alt abaixo, o campo non as colle (paso 4). O campo escribe o carácter que Windows lle dá a Alt ao subir tamén co Bloq Num desactivado, coma Chrome e o Bloc de notas; o terminal en VT, en cambio, só o envía co Bloq Num activado, coma o Windows Terminal ([`encode`](#keyencoder)). Na proba, Alt sobe sen o Bloq Num, e as teclas do código non fan nada nin co Bloq Num activado, coma o 0 e o 2 do teclado numérico, nin desactivado, cando chegan coma Insert, ↓, ← e Fin.
2. **Un carácter que non é da súa tecla** entra coma un carácter ([`own_character`](#os-axudantes-das-secuencias)). Pasa cunha tecla morta que non se xunta con Retroceso, Intro, Tab ou Esc: Windows escribe o acento con esa tecla, e despois a tecla chega outra vez co seu carácter. Así, ´ e Retroceso escriben ´ e despois bórrano, e ´ e Intro escriben ´ e deixan Intro para quen usa o campo. E pasa cun carácter do teclado numérico que chega só, coa tecla do seu código de exploración, coma o 7 coa tecla Inicio. Vai antes das ordes: se non, Retroceso borraría en vez de escribir o acento.
3. **As ordes, sen Ctrl+Alt.** Se a tecla é unha orde ([`command`](#as-ordes-command-e-repeat)), faina, e perde a primeira metade dun carácter que esperaba. Ctrl e Alt xuntos, de calquera lado, son AltGr para Windows, que escribe con eles os caracteres de AltGr, e non teñen ordes: AltGr+A escribe `æ` nalgunhas distribucións, e con Ctrl só seleccionaría todo. Windows dá AltGr coma o Ctrl esquerdo e o Alt dereito.
4. **Sen carácter, ou con Alt sen Ctrl,** non fai nada: as teclas sen carácter, coma ↑ e ↓ sen Maiús, F2 ou Maiús soa, e os caracteres con Alt só, que son dos menús da xanela, coma Alt+A. Con Ctrl e Alt, en cambio, escríbense, coma no terminal en VT ([`push_vt_character`](#keyencoder)): Ctrl+Alt+E, co Ctrl e o Alt esquerdos, escribe `€` na distribución española, igual ca AltGr+E.
5. **O resto escribe o seu carácter** ([`typed`](#os-caracteres-typed-e-type_character)), tantas veces coma se repetiu a tecla.

- **Desde `a99de8c`.** Ata entón, o campo só tomaba por AltGr o Ctrl esquerdo co Alt dereito, as teclas exactas de AltGr ([`alt_gr`](#controlkeys)), e tiraba os caracteres dos outros Ctrl+Alt, coma se fosen de Alt só: Ctrl+Alt+E cos esquerdos, ou cos dereitos, non escribía nada. Atopouno a revisión independente do código, cunha proba que falla sen o arranxo, `ctrl_and_alt_together_write_characters_as_altgr`. O campo xa non usa `alt_gr`.

| Evento | Resposta | O campo |
|---|---|---|
| A, con `a` | `Taken` | Escribe `a`. |
| Ctrl+A, con `0x01` | `Taken` | Selecciona todo. Windows escribe con Ctrl e unha letra un carácter de control, que é da tecla. |
| AltGr+E, con `€` | `Taken` | Escribe `€`. |
| Ctrl+Alt+E, cos esquerdos, con `€` | `Taken` | Escribe `€`, coma AltGr. |
| AltGr+←, sen carácter | `Ignored` | Nada: Ctrl+Alt non ten ordes. |
| Alt+A, con `a` | `Ignored` | Nada: é dos menús da xanela. |
| Intro, con CR | `Ignored` | Nada: o CR é un carácter de control. |
| Retroceso con `´`, e despois Retroceso con BS | `Taken` e `Taken` | Escribe `´`, e despois bórrao. |
| Intro con `´`, e despois Intro con CR | `Taken` e `Ignored` | Escribe `´`, e Intro queda para quen usa o campo. |
| Inicio, con `7` | `Taken` | Escribe `7`. |
| Alt, ao subir, con `ñ` | `Taken` | Escribe `ñ`. |
| Sen tecla, coas dúas metades de `😀` | `Taken` e `Taken` | A primeira espera, e a segunda escribe `😀`. |

### As ordes: `command` e `repeat`

```rust
fn command(&mut self, event: &KeyEvent) -> Option<Reply> {
    let controls = event.controls;
    let (ctrl, alt, shift) = (controls.ctrl(), controls.alt(), controls.shift());
    let times = event.repeat;
    let reply = match event.key {
        VK_LEFT | VK_RIGHT if !alt => {
            let right = event.key == VK_RIGHT;
            self.repeat(times, |field| field.step(right, ctrl, shift))
        }
        VK_HOME | VK_END if !alt => {
            self.go_to_end(event.key == VK_END, shift);
            Reply::Taken
        }
        VK_UP | VK_DOWN if shift && !alt => {
            self.go_to_end(event.key == VK_DOWN, true);
            Reply::Taken
        }
        VK_BACK if alt && !ctrl => self.repeat(times, |field| field.undo_or_redo(shift)),
        VK_BACK if !alt => self.repeat(times, |field| {
            if ctrl {
                field.delete_word(false)
            } else {
                field.backspace()
            }
        }),
        VK_DELETE if !alt && ctrl => self.repeat(times, |field| field.delete_word(true)),
        VK_DELETE if !alt && shift => self.cut(),
        VK_DELETE if !alt => self.repeat(times, Self::delete_forward),
        VK_INSERT if !alt && ctrl && !shift => self.copy(),
        VK_INSERT if !alt && shift && !ctrl => Reply::Paste,
        VK_A if ctrl && !alt => {
            self.select_all();
            Reply::Taken
        }
        VK_C if ctrl && !alt => self.copy(),
        VK_X if ctrl && !alt => self.cut(),
        VK_V if ctrl && !alt => Reply::Paste,
        VK_Z if ctrl && !alt => self.repeat(times, |field| field.undo_or_redo(shift)),
        VK_Y if ctrl && !alt => self.repeat(times, |field| field.undo_or_redo(true)),
        _ => return None,
    };
    Some(reply)
}

fn repeat(&mut self, times: u16, mut action: impl FnMut(&mut Self) -> bool) -> Reply {
    for _ in 0..times.max(1) {
        if !action(self) {
            break;
        }
    }
    Reply::Taken
}
```

**`command`**: o que fai `event`, unha tecla que baixa sen Ctrl+Alt, se é unha orde; se non, `None`. `ctrl`, `alt` e `shift` valen co Ctrl e o Alt de calquera lado:

| Tecla | Con | Fai | Repítese |
|---|---|---|---|
| ← → | Sen Alt; con Ctrl, por palabras; con Maiús, seleccionando | [`step`](#mover-o-cursor-step-go_to_end-e-move_caret) | Si |
| Inicio Fin | Sen Alt; con Maiús, seleccionando; Ctrl dá igual | [`go_to_end`](#mover-o-cursor-step-go_to_end-e-move_caret) | — |
| ↑ ↓ | Maiús, sen Alt | `go_to_end`, seleccionando | — |
| Retroceso | Alt, sen Ctrl; con Maiús, refai | [`undo_or_redo`](#o-portapapeis-e-desfacer-copy-cut-e-undo_or_redo) | Si |
| Retroceso | Sen Alt; con Ctrl, por palabras; Maiús dá igual | [`delete_word`](#borrar-backspace-delete_forward-delete_word-e-delete) ou [`backspace`](#borrar-backspace-delete_forward-delete_word-e-delete) | Si |
| Supr | Ctrl, sen Alt; Maiús dá igual | `delete_word`, cara a diante | Si |
| Supr | Maiús, sen Ctrl nin Alt | [`cut`](#o-portapapeis-e-desfacer-copy-cut-e-undo_or_redo) | — |
| Supr | Sen Ctrl, Maiús nin Alt | `delete_forward` | Si |
| Insert | Ctrl, sen Maiús nin Alt | [`copy`](#o-portapapeis-e-desfacer-copy-cut-e-undo_or_redo) | — |
| Insert | Maiús, sen Ctrl nin Alt | `Reply::Paste` | — |
| A | Ctrl, sen Alt | [`select_all`](#cambiar-o-texto-set_text-select_all-e-paste) | — |
| C | Ctrl, sen Alt | `copy` | — |
| X | Ctrl, sen Alt | `cut` | — |
| V | Ctrl, sen Alt | `Reply::Paste` | — |
| Z | Ctrl, sen Alt; con Maiús, refai | `undo_or_redo` | Si |
| Y | Ctrl, sen Alt | `undo_or_redo`, refacendo | Si |

- **A orde dos casos conta.** Supr mira Ctrl antes ca Maiús: Ctrl+Maiús+Supr borra unha palabra, coma Ctrl+Supr. E non son ordes Ctrl+Maiús+Insert, Insert só, ↑ e ↓ sen Maiús, nin ningunha tecla con Alt que non sexa Retroceso.
- **As letras, só con Ctrl.** Windows escribe con Ctrl e unha letra un carácter de control (Ctrl+A, `0x01`; Ctrl+C, `0x03`…), que é da tecla: a orde vai igual (`the_commands_work_with_the_characters_that_windows_gives_their_keys`). Sen Ctrl, ou só con Maiús, as letras escríbense (`the_letters_of_the_commands_write_themselves_without_ctrl`, desde `88dd960`).
- `self.repeat(times, Self::delete_forward)` pasa o método coma unha función, `fn(&mut Self) -> bool`.

**`repeat`** fai `action` `times` veces, ou unha se a conta é 0, mentres cambie algo: unha frecha no extremo, Retroceso ao principio ou desfacer sen máis pasos paran. Unha tecla que o teclado repite máis rápido do que a colle a xanela chega unha vez coa conta das súas repeticións, e fai así o seu traballo tantas veces coma se repetiu ([ADR 0030](../adr/0030-o-campo-de-texto.md), decisión 7). Devolve sempre `Taken`. Copiar, cortar e pegar fanse unha vez, tamén cunha conta: quen usa o campo le o portapapeis unha vez por tecla, e `Reply::Paste` non leva conta. As outras ordes que non se repiten farían o mesmo unha ou moitas veces. Ata `50d2379`, o comentario do módulo non dicía que copiar, cortar e pegar se fan unha vez; atopouno o axente da documentación.

### Os caracteres: `typed` e `type_character`

```rust
fn typed(&mut self, unit: u16, times: u16) -> Reply {
    let character = match unit {
        0xd800..=0xdbff => {
            self.high_surrogate = Some(unit);
            return Reply::Taken;
        }
        0xdc00..=0xdfff => match self.high_surrogate.take() {
            Some(high) => char::decode_utf16([high, unit]).next().and_then(Result::ok),
            None => return Reply::Taken,
        },
        _ => {
            self.high_surrogate = None;
            char::from_u32(u32::from(unit))
        }
    };
    match character {
        Some(character) if !character.is_control() => {
            self.repeat(times, |field| field.type_character(character))
        }
        _ => Reply::Ignored,
    }
}

fn type_character(&mut self, character: char) -> bool {
    if character.len_utf16() > self.room() {
        return false;
    }
    let mut buffer = [0; 4];
    let text = &*character.encode_utf8(&mut buffer);
    self.track(|field| field.replace(field.selection(), text, Kind::Typing));
    true
}
```

**`typed`**: unha unidade de UTF-16 dun carácter escrito `times` veces:

| Unidade | Que fai |
|---|---|
| A primeira metade dun carácter de fóra do plano básico (de U+D800 a U+DBFF) | Gárdaa, e espera pola segunda. Unha primeira metade que xa esperaba pérdese. |
| A segunda metade (de U+DC00 a U+DFFF) | Coa primeira gardada, `char::decode_utf16` xúntaas no carácter enteiro. Sen ela, pérdese. |
| Outra | Unha primeira metade que esperaba pérdese, e `char::from_u32` dá o carácter. |

- Un carácter de control, coma o CR de Intro, o HT de Tab, o ESC de Esc ou o `0x02` de Ctrl+B, non entra: `Ignored`, para quen usa o campo. Os outros escríbense con [`type_character`](#os-caracteres-typed-e-type_character), repetidos.
- **As metades** chegan en dous eventos, coma os emoji do panel de emoji ou dun IME ([`KeyEvent`](#keyevent)). As dúas responden `Taken`, e só a segunda cambia o campo. Coma di o comentario desde `50d2379`, outra primeira metade, outro carácter ou unha orde (en `key`) ocupan o sitio da que espera, e unha segunda metade sen primeira pérdese. Unha tecla que o campo non colle e que non ten carácter, coma ↑ ou Maiús, déixaa esperando; `set_text`, en cambio, esquécea.

**`type_character`** escribe `character` en vez da selección, se cabe ([`room`](#o-límite-room)): se non, non fai nada, e devolve `false`, e as repeticións paran. `encode_utf8` escribe o carácter nun *array* de 4 bytes, sen reservar memoria, e [`replace`](#editar-replace-e-edited) pono no texto, dentro de `track`, coma `Typing`: o escrito seguido, tamén enriba dunha selección, é un paso.

### Mover o cursor: `step`, `go_to_end` e `move_caret`

```rust
fn step(&mut self, right: bool, word: bool, extend: bool) -> bool {
    let selection = self.selection();
    // Without Shift, a selection goes away, from its edge on that side.
    let from = match (extend || selection.is_empty(), right) {
        (true, _) => self.caret,
        (false, true) => selection.end,
        (false, false) => selection.start,
    };
    let text = &self.text;
    let at = match (word, right) {
        (true, true) => next_word(text, from),
        (true, false) => previous_word(text, from),
        _ if !extend && !selection.is_empty() => from,
        (false, true) => next_cluster(text, from),
        (false, false) => previous_cluster(text, from),
    };
    self.move_caret(at, extend)
}

fn go_to_end(&mut self, end: bool, extend: bool) {
    let at = if end { self.text.len() } else { 0 };
    self.move_caret(at, extend);
}

fn move_caret(&mut self, at: usize, extend: bool) -> bool {
    let anchor = if extend { self.anchor } else { at };
    self.select(anchor, at)
}
```

**`step`**: ← ou →, por palabras con Ctrl (`word`), seleccionando con Maiús (`extend`):

1. **Desde onde.** Con Maiús, ou sen selección, desde o cursor. Sen Maiús e cunha selección, desde o seu bordo do lado da frecha: o principio con ←, e o final con →.
2. **Ata onde.** Con Ctrl, ao principio da palabra seguinte ou da anterior, desde alí ([`next_word` e `previous_word`](#as-palabras-next_word-previous_word-e-word_at)). Sen Ctrl, sen Maiús e cunha selección, ao mesmo bordo, e a selección vaise. E se non, ao grafema seguinte ou ao anterior ([`next_cluster` e `previous_cluster`](#onde-para-o-cursor-next_cluster-previous_cluster-e-is_stop)).
3. [`move_caret`](#mover-o-cursor-step-go_to_end-e-move_caret) leva alí o cursor.

| Antes | Tecla | Despois |
|---|---|---|
| `ab\|cd` | Maiús+→ dúas veces | `ab[cd\|]` |
| `ab\|cd` | Maiús+→ e Maiús+← | `ab\|cd`: onde empezou, sen selección |
| `a[bc\|]d` ou `a[\|bc]d` | ← | `a\|bcd` |
| `a[bc\|]d` ou `a[\|bc]d` | → | `abc\|d` |
| `[fo\|]o bar` | Ctrl+→ | `foo \|bar`: desde o final da selección |
| `foo b[\|ar]` | Ctrl+← | `foo \|bar`: desde o principio |
| `\|foo bar` | Ctrl+Maiús+→ | `[foo \|]bar` |

**`go_to_end`**: Inicio ou Fin, seleccionando con Maiús. **`move_caret`** leva o cursor a `at`, e con el o principio da selección, se non se selecciona; se se selecciona, o principio queda onde estaba. [`select`](#seleccionar-select) pecha o paso se cambia algo, e di se cambiou.

### Borrar: `backspace`, `delete_forward`, `delete_word` e `delete`

```rust
fn backspace(&mut self) -> bool {
    let selection = self.selection();
    if !selection.is_empty() {
        return self.delete(selection, Kind::Other);
    }
    match self.text[..self.caret].chars().next_back() {
        Some(last) => self.delete(self.caret - last.len_utf8()..self.caret, Kind::Backspace),
        None => false,
    }
}

fn delete_forward(&mut self) -> bool {
    let selection = self.selection();
    if !selection.is_empty() {
        return self.delete(selection, Kind::Other);
    }
    let end = next_cluster(&self.text, self.caret);
    self.delete(self.caret..end, Kind::Delete)
}

fn delete_word(&mut self, forward: bool) -> bool {
    let selection = self.selection();
    let range = if !selection.is_empty() {
        selection
    } else if forward {
        self.caret..next_word(&self.text, self.caret)
    } else {
        previous_word(&self.text, self.caret)..self.caret
    };
    self.delete(range, Kind::Other)
}

fn delete(&mut self, range: Range<usize>, kind: Kind) -> bool {
    if range.is_empty() {
        return false;
    }
    self.track(|field| field.replace(range, "", kind));
    true
}
```

| Método | Tecla | Cunha selección | Sen selección | Tipo, sen selección |
|---|---|---|---|---|
| `backspace` | Retroceso | Bórraa. | O último punto de código antes do cursor. | `Backspace` |
| `delete_forward` | Supr | Bórraa. | O grafema despois do cursor. | `Delete` |
| `delete_word` | Ctrl+Retroceso e Ctrl+Supr | Bórraa. | Ata o principio da palabra anterior, ou da seguinte. | `Other` |

- **Retroceso borra un punto de código, e Supr un grafema,** coma Chrome en Windows ([ADR 0030](../adr/0030-o-campo-de-texto.md), decisión 2). `chars().next_back()` dá o último carácter antes do cursor, e `len_utf8`, os seus bytes. Así, un `é` escrito coma `e` e un acento perde só o acento, para corrixilo; un emoji, que é un punto de código, aínda que en UTF-16 vai en dúas metades, vaise enteiro; e unha familia de emoji perde o seu último membro, e despois o ZWJ que o unía.
- **Cunha selección,** os tres bórrana nun paso propio (`Other`).
- **O cursor despois de borrar** vai ao principio do grafema onde quedou ([`replace`](#editar-replace-e-edited)).
- **Os pasos:** os Retrocesos seguidos son un paso, e os Supr seguidos, outro; borrar unha selección ou unha palabra, un paso cada vez (decisión 4).
- **`delete`** borra `range` coma `kind`, e di se borrou algo: un rango baleiro, coma Retroceso ao principio ou Supr ao final, non fai nada, nin pecha o paso. Se non, `replace` co texto baleiro, dentro de `track`.

### O portapapeis e desfacer: `copy`, `cut` e `undo_or_redo`

```rust
fn copy(&self) -> Reply {
    match self.selected_text() {
        "" => Reply::Taken,
        selected => Reply::Copy(selected.to_owned()),
    }
}

fn cut(&mut self) -> Reply {
    let reply = self.copy();
    self.delete(self.selection(), Kind::Other);
    reply
}

fn undo_or_redo(&mut self, redo: bool) -> bool {
    if redo {
        self.track(Self::redo)
    } else {
        self.track(Self::undo)
    }
}
```

- **`copy`**, Ctrl+C e Ctrl+Insert: a selección vai ao portapapeis, coma `Reply::Copy`, que quen usa o campo escribe nel. Sen selección, `Taken`: non fai nada, e a tecla non lle chega a ninguén máis (decisión 7). `&self`: non cambia nada.
- **`cut`**, Ctrl+X e Maiús+Supr: copia, e despois borra a selección, nun paso propio. Desfacelo trae o texto outra vez, seleccionado. Sen selección, nada.
- **`undo_or_redo`**: desfai o último paso, ou, con `redo`, refai o último desfeito, dentro de `track`, e di se había un. `Self::undo` e `Self::redo` pásanse coma funcións.

## `crates/meiga-core/src/ui/field/tests.rs`

Desde a subfase 3.2: as cincuenta e cinco probas do campo, nun arquivo aparte para que `mod.rs` sexa máis curto. É un submódulo de `field` (`#[cfg(test)] mod tests;`, ao final de `mod.rs`), así que ve os seus campos e métodos privados, coma `anchor`, `scroll`, `undo` ou `redo`; os privados de `keys.rs`, non: as teclas próbanse a través de `key`, coma as recibirá o campo. As outras dezasete do campo están cos seus módulos: seis en [`segment.rs`](#cratesmeiga-coresrcuifieldsegmentrs) e once en [`history.rs`](#cratesmeiga-coresrcuifieldhistoryrs).

```rust
//! The tests of the text field, with its state written as text: `|` is the
//! caret, and `[` and `]` the ends of the selection, as in `a[bc|]d`.

use super::*;
use crate::input::{ControlKeys, KeyEvent};
```

- **O estado coma texto.** Coma di o comentario do módulo, as probas escriben o estado do campo coma un texto: `|` é o cursor, e `[` e `]` os extremos da selección. `a[bc|]d` é `abcd` con `bc` seleccionado e o cursor despois do `c`; e `a[|bc]d`, o mesmo co cursor antes do `b`.
- `use super::*;` trae todo o de `mod.rs`, tamén o privado e o que este importa, coma `CellChar`, `Side` ou `segment`; e o outro `use`, `ControlKeys` e `KeyEvent`, para facer as teclas.

### As teclas das probas

Os códigos de tecla virtual que usan as probas son os de `keys.rs` e outros: Tab, Intro, Maiús, Esc, RePáx (`VK_PRIOR`), B, D, E, Q, o 0, o 1, o 2 e o 4 do teclado numérico, F2 e `VK_PACKET` (`0xE7`), a tecla dos caracteres que outros programas escriben coma teclas, coma o panel de emoji. E as teclas modificadoras: `none`, ningunha; `ctrl`, o Ctrl esquerdo; `shift`; `alt`, o Alt esquerdo; `ctrl_shift` e `alt_shift`; e `alt_gr`, o Ctrl esquerdo e o Alt dereito, coma dá Windows AltGr.

```rust
fn event(key: u16, character: u16, controls: ControlKeys) -> KeyEvent {
    KeyEvent {
        key,
        scan: 0,
        character,
        down: true,
        controls,
        repeat: 1,
    }
}

fn unit(c: char) -> u16 {
    let mut buffer = [0; 2];
    c.encode_utf16(&mut buffer)[0]
}

fn halves(c: char) -> [u16; 2] {
    let mut buffer = [0; 2];
    c.encode_utf16(&mut buffer);
    buffer
}
```

- **`event`**: unha tecla que baixa, unha vez, sen código de exploración, que o campo non mira.
- **`unit`**: a unidade de UTF-16 dun carácter do plano básico; e **`halves`**, as dúas metades dun de fóra.

### O estado coma texto: `make_with`, `make`, `state` e `shown`

```rust
fn make_with(state: &str, limit: usize) -> Field {
    let mut text = String::new();
    let (mut caret, mut start, mut end) = (None, None, None);
    for c in state.chars() {
        match c {
            '|' => caret = Some(text.len()),
            '[' => start = Some(text.len()),
            ']' => end = Some(text.len()),
            c => text.push(c),
        }
    }
    let mut field = Field::new(limit);
    field.set_text(&text);
    assert_eq!(field.text(), text, "{state:?} does not fit");
    let caret = caret.unwrap_or(text.len());
    field.anchor = match (start, end) {
        (Some(start), Some(end)) if caret == start => end,
        (Some(start), Some(_)) => start,
        _ => caret,
    };
    field.caret = caret;
    field.take_changes();
    field
}

fn make(state: &str) -> Field {
    make_with(state, 1000)
}

fn state(field: &Field) -> String {
    let text = field.text();
    let selection = field.selection();
    let mut out = String::new();
    let chars = text.char_indices().map(|(at, c)| (at, Some(c)));
    for (at, c) in chars.chain([(text.len(), None)]) {
        if !selection.is_empty() && at == selection.start {
            out.push('[');
        }
        if at == field.caret() {
            out.push('|');
        }
        if !selection.is_empty() && at == selection.end {
            out.push(']');
        }
        out.extend(c);
    }
    out
}

fn shown(field: &Field) -> String {
    let mut cells = Vec::new();
    let Shown { caret, selection } = field.show(&mut cells);
    let mut out = String::new();
    for index in 0..=cells.len() {
        if !selection.is_empty() && index == selection.start {
            out.push('[');
        }
        if index == caret {
            out.push('|');
        }
        if !selection.is_empty() && index == selection.end {
            out.push(']');
        }
        match cells.get(index) {
            Some(CellChar::Blank) => out.push(' '),
            Some(CellChar::Narrow(c) | CellChar::WideLeft(c)) => out.push(*c),
            Some(CellChar::WideRight(_)) | None => {}
        }
    }
    out
}
```

- **`make_with(estado, límite)`**: un campo cun límite, co texto, o cursor e a selección de `estado`. Quita as marcas, pon o texto con `set_text` e comproba que cabe; despois pon o cursor onde está o `|`, ou ao final, e o principio da selección no outro extremo, se hai `[` e `]`. Cambia os campos privados directamente, sen pasar por `select`, e colle os cambios: o campo empeza sen cambios, sen pasos e sen celas. **`make`**: o mesmo cun límite de 1000.
- **`state`**: o estado do campo escrito igual, coma o len os seus métodos públicos: o texto, con `[` e `]` nos extremos da selección e `|` no cursor. Percorre o texto carácter a carácter, e despois o final.
- **`shown`**: o que mostra o campo, escrito igual: as súas celas, unha en branco coma un espazo e unha ancha co seu carácter, unha vez; `|` antes da cela do cursor; e as celas seleccionadas entre `[` e `]`. A cela libre despois do texto non ten nada, así que `efgh|` é un campo que mostra `efgh` co cursor na cela seguinte.

### As accións: `tap`, `type_text` e `after`

```rust
fn tap(field: &mut Field, key: u16, controls: ControlKeys) -> Reply {
    field.key(&event(key, 0, controls))
}

fn type_text(field: &mut Field, text: &str) {
    for unit in text.encode_utf16() {
        field.key(&event(0, unit, none()));
    }
}

fn after(start: &str, keys: &[(u16, ControlKeys)]) -> String {
    let mut field = make(start);
    for &(key, controls) in keys {
        tap(&mut field, key, controls);
    }
    state(&field)
}
```

- **`tap`**: preme `key` coas teclas modificadoras, sen carácter: as frechas, Supr ou Ctrl+Z, sen o carácter de control que lles daría Windows, que o campo non necesita.
- **`type_text`**: escribe un texto coma un IME: cada unidade de UTF-16 nun evento sen tecla virtual, e as dúas metades dun emoji en dous.
- **`after`**: o estado dun campo despois dunhas teclas, para comparalo nunha liña, coma `after("ab|cd", &[(VK_HOME, shift())])`, que dá `[|ab]cd`.

### As probas, unha a unha

| Proba | Que comproba |
|---|---|
| `a_new_field_is_empty_and_shows_nothing_until_it_has_cells` | Un campo novo, cun límite de 10: sen texto, co cursor en 0, sen selección, sen celas e sen cambios. |
| `typing_writes_at_the_caret_and_replaces_the_selection` | A tecla X con `x` escribe no cursor e responde `Taken`; e `xy`, escrito cunha selección de `bc` cara a diante ou cara atrás, cámbiaa: `axy\|d`. |
| `the_letters_of_the_commands_write_themselves_without_ctrl` | Desde `88dd960`. A, C, V, X, Y e Z, soas e con Maiús, escriben as súas letras, `aAcCvVxXyYzZ`, e responden `Taken`: só son ordes con Ctrl. Colle sete mutantes que antes sobrevivían. |
| `altgr_writes_characters_and_alt_alone_does_not` | AltGr+E escribe `€`, e AltGr+A, `æ`, en vez de seleccionar todo; Alt+A é `Ignored`; e AltGr+A e AltGr+← sen carácter non fan nada, nin unha orde. |
| `ctrl_and_alt_together_write_characters_as_altgr` | Desde `a99de8c`. Ctrl+Alt+E, co Ctrl e o Alt esquerdos, escribe `€`, coma na distribución española, e Ctrl+Alt+Q, cos dereitos, `@`, coma na alemá; e sen carácter, A e ← con eles non fan nada, nin unha orde. Co código de antes, que tiraba eses caracteres, falla. |
| `alt_writes_the_character_of_its_code_when_it_goes_up` | Alt+0241: as teclas do código, o 0, o 2, o 4 e o 1 do teclado numérico e, co Bloq Num desactivado, Insert, ↓, ← e Fin, todas con Alt, son `Ignored`; Alt, ao subir con `ñ`, escribe `ñ`. Outra tecla que sobe co seu carácter, e Alt ao subir sen ningún, non escriben nada. |
| `control_characters_never_go_in` | Intro (CR), Ctrl+Intro (LF), Esc, Tab, Ctrl+B (`0x02`), DEL, NEL e o CSI de C1 (`0x9B`) son `Ignored`, e non cambian nada. |
| `characters_outside_the_basic_multilingual_plane_come_in_two_halves` | `😀` en dúas metades: a primeira non cambia nada, e a segunda escríbeo. Unha segunda metade soa pérdese; e unha primeira seguida doutro carácter, ou dunha orde, coma ←, tamén. |
| `characters_without_a_key_go_in_as_the_ones_of_an_ime_or_the_emoji_panel` | `中文`, sen tecla, coma dun IME, e `👍` en dúas metades coa tecla `VK_PACKET`, coma do panel de emoji, escríbense. |
| `a_dead_key_that_does_not_join_writes_its_accent_before_the_key` | Retroceso con `´` escribe `´`, e despois Retroceso co seu BS bórrao; Intro, Esc e Tab con `´` escriben `´`, e despois, co seu carácter, son `Ignored`. E un `7` do teclado numérico que chega coa tecla Inicio escríbese. |
| `a_key_that_repeats_does_its_work_once_per_repeat` | A tecla A repetida tres veces escribe `aaa`; ← dúas veces move o cursor dous grafemas; Retroceso cinco veces borra só o `a` que había antes do cursor; Ctrl+Z dúas veces desfai os dous pasos; Ctrl+Y dúas veces refaios; e Supr cunha conta de 0, que Windows nunca dá, borra unha vez. |
| `an_accent_typed_after_a_letter_joins_it` | Un acento combinado escrito despois dun `e` xúntase con el: o cursor queda despois do `é`, e ← vai antes del. |
| `what_joins_the_text_after_it_leaves_the_caret_after_the_whole_cluster` | 🇫 escrito antes de 🇪🇸 fai unha bandeira co 🇪: o cursor queda despois dela, `🇫🇪\|🇸`. |
| `a_character_that_does_not_fit_is_not_typed` | Cun límite de 3, `d` despois de `abc` non se escribe, sen cambios, pero a tecla responde `Taken`; en vez dunha selección, cabe; e despois de `ab`, `😀`, de dúas unidades, non cabe, e `ñ`, dunha, si. |
| `set_text_puts_a_single_line_cut_to_the_limit` | `set_text` cun tabulador, un CR LF e un BEL deixa `a b c`, cos dous cambios; os espazos dos extremos quedan; cun límite de 3, `ab` e un `é` co acento combinado, e `ab👍`, quedan en `ab`, en grafemas enteiros; e o mesmo texto outra vez, co cursor xa ao final, non é un cambio. |
| `set_text_forgets_the_steps_to_undo` | Despois de escribir `abc` e de `set_text("xyz")`, Ctrl+Z non fai nada. |
| `set_text_forgets_half_a_character_and_what_a_button_held_down_selects` | Desde `50d2379`. A primeira metade dun `😀`, `set_text("cd")` e a segunda metade: a segunda non se xunta coa primeira, e o texto queda `cd`. E un clic, `set_text("efgh")` co botón aínda abaixo, e un arrastre: o arrastre non selecciona nada. |
| `the_arrows_move_the_caret_one_cluster` | En `a`, unha familia de emoji, `é` cun acento combinado e `中`, → para nos bytes 1, 19, 22 e 25, e queda alí; e ← en 22, 19, 1 e 0. |
| `ctrl_and_the_arrows_move_by_words_and_home_and_end_to_the_ends` | En `foo bar.rs`, Ctrl+→ para antes de `bar` e de `.`; Ctrl+← antes de `rs` e, despois de tres veces, de `bar`; e Inicio e Fin, con e sen Ctrl, van aos extremos. |
| `shift_selects_from_where_the_selection_started` | Maiús coas frechas, Inicio, Fin, ↑ e ↓, e Ctrl+Maiús con Inicio e coas frechas, seleccionan desde onde empezou a selección; e voltar alí deixa o campo sen selección. |
| `without_shift_the_arrows_end_a_selection_at_its_edge` | Cunha selección cara a diante ou cara atrás, ← e → levan o cursor ao seu bordo dese lado, e Inicio e Fin, aos extremos; e Ctrl+→ e Ctrl+← saltan desde o bordo. |
| `the_keys_of_the_user_of_the_field_change_nothing` | ↑, ↓, RePáx, F2, Maiús soa, Alt+←, Alt+→, Alt+Inicio, Alt+Maiús+↑, Insert, Ctrl+Maiús+Insert, Alt+Supr, Ctrl+Alt+Retroceso e Alt+Z son `Ignored`, sen cambios. |
| `backspace_deletes_the_last_code_point_and_delete_the_next_cluster` | Retroceso despois dun `é` escrito coma `e` e un acento borra só o acento; despois de `b`, o `b`; ao principio, nada; despois de `😀`, o emoji enteiro; e despois dunha familia de dous, o segundo membro, e queda o ZWJ. Supr antes do `é` borra o grafema enteiro; antes da familia, toda; e ao final, nada. |
| `after_deleting_the_caret_goes_to_the_start_of_the_cluster_that_it_is_in` | Retroceso en `🇪🇸\|🇫🇷` deixa `\|🇪🇫🇷`, e Supr en `🇪\|x🇸`, `\|🇪🇸`: o cursor vai ao principio da bandeira nova. |
| `backspace_and_delete_delete_the_selection_also_with_ctrl` | Retroceso e Supr, con e sen Ctrl, borran unha selección cara a diante ou cara atrás. |
| `ctrl_backspace_and_ctrl_delete_delete_up_to_the_start_of_a_word` | Ctrl+Retroceso en `foo bar\|` deixa `foo \|`, e outra vez, nada; Ctrl+Supr en `\|foo bar` deixa `\|bar`; Ctrl+Retroceso en `main.r\|s`, `main.\|s`, e dous en `C:\Users\x\|`, `C:\Users\|`; nun campo baleiro, nada; e con Maiús, igual. |
| `ctrl_c_and_ctrl_insert_copy_the_selection` | Ctrl+C, Ctrl+Maiús+C e Ctrl+Insert dan `Copy("bc")`, sen cambiar nada; e sen selección, `Taken`. |
| `ctrl_x_and_shift_delete_cut_the_selection` | Ctrl+X, Ctrl+Maiús+X e Maiús+Supr dan `Copy("bc")` e bórrano; Ctrl+Z tráeo outra vez, seleccionado; e sen selección, `Taken`, sen cambios. |
| `ctrl_v_and_shift_insert_ask_for_the_clipboard` | Ctrl+V, Ctrl+Maiús+V e Maiús+Insert dan `Paste`, sen cambiar nada. |
| `the_commands_work_with_the_characters_that_windows_gives_their_keys` | As ordes cos caracteres que lles dá Windows: Ctrl+Retroceso con DEL, Retroceso con BS, e Ctrl+A, Ctrl+C, Ctrl+X, Ctrl+V, Ctrl+Z e Ctrl+Y cos seus caracteres de control. |
| `undoing_goes_back_a_step_and_redoing_forward` | Despois de escribir `foo`, ← e `x`: Ctrl+Z desfai o `x`, Alt+Retroceso o `foo`, e outro Ctrl+Z non fai nada, pero responde `Taken`; Ctrl+Y refai `foo`, Ctrl+Maiús+Z o `x`, e Alt+Maiús+Retroceso, xa sen nada que refacer, nada; e despois dun Ctrl+Z, Alt+Maiús+Retroceso refai o `x`. |
| `undoing_typing_over_a_selection_brings_the_selection_back` | `xy` escrito enriba de `bc`: Ctrl+Z trae `bc` seleccionado, e Ctrl+Y, `xy` outra vez. |
| `backspaces_in_a_row_and_deletes_in_a_row_are_a_step_each` | Dous Retrocesos e dous Supr seguidos son dous pasos; e cada palabra borrada con Ctrl+Retroceso, un. |
| `a_paste_is_a_step_of_its_own` | `c` escrito, `de` pegado e `f` escrito son tres pasos. |
| `a_new_edit_forgets_the_steps_undone` | Despois de desfacer `a`, escribir `b` esquéceo: Ctrl+Y xa non o trae. |
| `a_paste_becomes_a_single_line` | Os sete primeiros textos da táboa de [`paste`](#cambiar-o-texto-set_text-select_all-e-paste), pegados entre `<` e `>`. |
| `a_paste_of_only_whitespace_is_a_space_and_of_nothing_nothing` | Uns espazos, un CR LF e un tabulador pegan un espazo en vez da selección; e un texto baleiro, ou só con BEL, ESC e NUL, non pega nada nin cambia nada. |
| `a_paste_puts_what_fits_in_whole_clusters` | Cun límite de 5 e `ab`, de `cd👍🏽e` só entra `cd`: o 👍🏽, de catro unidades, non cabe. En vez dunha selección de `ab`, cun límite de 6, entra `cd👍🏽`. E se non cabe nada, nin un emoji nin un espazo, nada cambia: tampouco a selección. |
| `a_click_puts_the_caret_on_the_nearest_side_of_the_cell` | Os clics da táboa de [`stop_at`](#do-rato-ao-texto-stop_at-word_under-e-cell_of), en `a中b`, sen selección. |
| `a_click_with_shift_moves_the_end_of_the_selection` | Maiús e un clic moven o cursor e deixan o principio da selección onde estaba, cara a diante e cara atrás. |
| `dragging_selects_from_where_the_button_went_down` | Un clic e un arrastre seleccionan desde onde baixou o botón, cara a diante e cara atrás; e un arrastre sen botón, nada. |
| `a_double_click_selects_a_word_and_dragging_whole_words` | En `foo  bar.rs baz`, o dobre clic selecciona `foo`, os dous espazos, o `.` e, despois do final, `baz`. E arrastrar despois dun dobre clic en `bar` selecciona palabras enteiras: ata `rs`; cara atrás, ata `foo`, co `bar` enteiro; e outra vez só `bar`. |
| `a_triple_click_selects_everything_and_dragging_keeps_it` | O triplo clic selecciona todo, e arrastrar non o cambia. |
| `a_double_click_without_text_selects_nothing` | Nun campo baleiro, ou só cun espazo de ancho cero, o dobre clic e o arrastre non seleccionan nada. |
| `a_change_of_the_text_ends_what_the_button_held_down_selects` | Despois dun dobre clic en `bar`, co botón abaixo, escribir `x` remata o arrastre: arrastrar xa non selecciona. E igual desfacendo e refacendo. É a proba do fallo que atopou a proba ao chou ([`edited`](#editar-replace-e-edited)). |
| `a_click_somewhere_else_ends_the_step_of_the_typing` | Un clic onde está o cursor non pecha o paso, e `c` xúntase con `ab`; dous clics que van a outro sitio e voltan pechan o paso, e `d` é outro. |
| `a_text_that_does_not_fit_shows_the_part_with_the_caret` | A táboa de [`follow_caret`](#seguir-o-cursor-track-e-follow_caret): escribir `abcdefgh` nun campo de cinco celas, e despois Inicio, Fin e ←. |
| `a_shorter_text_brings_back_the_part_that_the_field_shows` | En cinco celas, `abcdefgh` co cursor ao final mostra `efgh`; despois de Retroceso, `defg`; despois de borrar dúas letras seleccionadas, `bcde`; e despois de borralo todo, nada. |
| `the_field_never_shows_half_a_wide_character` | `a中中` co cursor ao final, en catro celas, mostra só `中`; e `ab中` co cursor ao principio, en tres, `ab` e unha cela en branco. |
| `the_selection_shows_in_the_cells_that_the_field_shows` | En tres celas, Maiús+Inicio desde o final de `abcdef` mostra `abc` seleccionado; e Ctrl+A, `ef` seleccionado e o cursor despois. |
| `clusters_without_width_take_no_cell` | Un espazo de ancho cero non ocupa cela: o campo mostra o cursor xusto antes do `x`. |
| `a_field_without_cells_shows_nothing` | Despois de pasar a 0 celas, `show` baleira o vector e dá un `Shown` por defecto. |
| `dragging_out_of_the_field_shows_more_on_that_side` | En cinco celas, arrastrar desde o principio á cela 7 mostra `defgh`, con `defg` seleccionado, e á 9, `efgh` seleccionado e o cursor na cela libre; e cara atrás, desde o final, ás celas −2 e −5, `cdefg`, con `cdef` seleccionado, e `abcde`, todo seleccionado. |
| `the_changes_say_what_changed` | Escribir cambia o texto e o que se mostra; coller os cambios esquéceos; ← só cambia o que se mostra; unha frecha no extremo do texto, nada; desde `b96fa25`, dúas celas nun campo que non tiña ningunha cambian o que se mostra, e as mesmas outra vez, nada; e, desde `a99de8c`, escribir `a` enriba dun `a` seleccionado, e desfacelo, cambian o que se mostra, pero non o texto. Ata `b96fa25`, comprobaba o contrario coas celas: que esas dúas non eran un cambio, porque a primeira cela que se vía seguía sendo a mesma. |
| `random_edits_keep_the_field_whole_and_undo_to_the_start` | 300 series de 45 accións ao chou, e o que o campo sempre mantén despois de cada unha ([máis abaixo](#a-proba-ao-chou-random_edits_keep_the_field_whole_and_undo_to_the_start)). |

### A proba ao chou: `random_edits_keep_the_field_whole_and_undo_to_the_start`

```rust
struct Random(u64);

impl Random {
    fn below(&mut self, n: usize) -> usize {
        let mut x = self.0;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.0 = x;
        (x % n as u64) as usize
    }
}

const PIECES: [&str; 18] = [
    "a", "Z", "7", "_", " ", ".", "\\", "é", "e\u{301}", "\u{301}", "中", "😀", "👍🏽", "🇪", "🇸",
    "\u{200d}", "\u{200b}", "ñ",
];

const KEYS: [u16; 16] = [
    VK_LEFT, VK_RIGHT, VK_HOME, VK_END, VK_UP, VK_DOWN, VK_BACK, VK_DELETE, VK_INSERT, VK_A, VK_C,
    VK_V, VK_X, VK_Y, VK_Z, VK_RETURN,
];
```

- **`Random`**: un xerador de números pseudoaleatorios *xorshift64*, coma nas probas do modelo, de `damage.rs` e do terminal: cada semente dá sempre a mesma secuencia, así que, se a proba falla, falla sempre igual. `below(n)` dá un número de 0 a `n − 1`.
- **`PIECES`**: o que escriben as edicións ao chou: dúas letras, un número, `_`, un espazo, dous signos, `é` composto e escrito coma `e` e un acento, un acento só, un ideograma, dous emoji, un deles cun ton de pel, dous indicadores rexionais, o ZWJ, o espazo de ancho cero e `ñ`. Así se xuntan e se parten grafemas en todos os sitios.
- **`KEYS`**: as teclas que preme: as frechas, Inicio, Fin, Retroceso, Supr, Insert, as letras das ordes e Intro.

```rust
fn check(field: &mut Field, before: &(String, usize, usize, usize, usize), context: &str) {
    let text = field.text().to_owned();
    for at in [field.anchor, field.caret] {
        assert!(
            text.is_char_boundary(at) && segment::is_stop(&text, at),
            "{context}: {at} in {text:?}"
        );
    }
    assert!(units(&text) <= field.limit(), "{context}: {text:?}");
    assert!(!text.chars().any(char::is_control), "{context}: {text:?}");
    let mut cells = Vec::new();
    let shown = field.show(&mut cells);
    assert!(cells.len() <= field.columns, "{context}: {cells:?}");
    assert!(shown.caret <= cells.len(), "{context}: {shown:?} {cells:?}");
    if field.columns > 0 {
        // The caret is in sight: `show` puts it in the cells anyway.
        let caret = text::width(&text[..field.caret]);
        assert!(
            (field.scroll..field.scroll + field.columns).contains(&caret),
            "{context}: the caret on the cell {caret}, and the field from {}",
            field.scroll
        );
        assert!(shown.caret < field.columns, "{context}: {shown:?}");
        assert!(
            !matches!(cells.first(), Some(CellChar::WideRight(_))),
            "{context}: {cells:?}"
        );
    }
    let selection = &shown.selection;
    assert!(
        selection.start <= selection.end && selection.end <= cells.len(),
        "{context}: {shown:?} {cells:?}"
    );
    let changes = field.take_changes();
    // An action can make more than one edit, and the last one can give back
    // the text that there was: only what changed in the end must be told.
    if text != before.0 {
        assert!(changes.text, "{context}: {changes:?}");
    }
    if (text, field.anchor, field.caret, field.scroll, field.columns) != *before {
        assert!(changes.shown, "{context}: {changes:?}");
    }
}
```

**`check`** comproba, despois de cada acción, o que o campo sempre mantén:

1. O cursor e o principio da selección están no límite dun carácter, e nun sitio onde para o cursor.
2. O texto ten como moito o límite de unidades de UTF-16, e ningún carácter de control.
3. `show` dá como moito tantas celas coma ten o campo, e o cursor nunha delas ou xusto despois da última. Con celas, o cursor vese: a súa cela no texto, `text::width(&text[..field.caret])`, está entre a primeira que se ve, `scroll`, e a última, `scroll + columns − 1`; a cela que dá `show` é unha das do campo; e a primeira nunca é a metade dereita dun carácter ancho. A selección está dentro das celas.
4. Se o texto é outro ao final, `Changes::text`; e se o texto, o principio da selección, o cursor, a primeira cela que se ve ou, desde `b96fa25`, o número de celas son outros, `Changes::shown`.

- **O cursor á vista, desde `50d2379`.** Ata entón, a proba só miraba a cela do cursor que dá `show`, que, coma di o comentario, mete o cursor nas celas que se ven igual: un cursor á esquerda da parte que se ve daba a cela 0, e pasaba. Agora mide a súa cela no texto. Atopouno o axente da documentación.
- **Non comproba o contrario,** que un cambio anotado sexa un cambio ao final, coma di o seu comentario: unha acción pode facer máis dunha edición, e a última pode devolver o texto que había. Escribir `👍🏽` enriba dun `👍🏽` seleccionado son dúas edicións: o `👍` cambia o texto, e o ton de pel volta deixalo coma estaba. O texto cambiou polo medio, e `Changes::text` dío, aínda que ao final sexa o mesmo.

```rust
#[test]
fn random_edits_keep_the_field_whole_and_undo_to_the_start() {
    let controls = [none(), shift(), ctrl(), ctrl_shift(), alt(), alt_shift()];
    let sides = [Side::Left, Side::Right];
    for seed in 1..=300_u64 {
        let mut random = Random(seed.wrapping_mul(0x9e37_79b9_7f4a_7c15));
        let mut field = Field::new(4 + random.below(12));
        let mut clipboard = String::new();
        // At most two edits each, which are fewer than the steps that can be
        // undone.
        for step in 0..45 {
            let before = (
                field.text.clone(),
                field.anchor,
                field.caret,
                field.scroll,
                field.columns,
            );
            let action = random.below(10);
            match action {
                0..=3 => type_text(&mut field, PIECES[random.below(PIECES.len())]),
                4..=6 => {
                    let key = KEYS[random.below(KEYS.len())];
                    match tap(&mut field, key, controls[random.below(controls.len())]) {
                        Reply::Copy(text) => clipboard = text,
                        Reply::Paste => field.paste(&clipboard),
                        Reply::Ignored | Reply::Taken => {}
                    }
                }
                7 => {
                    let clicks = 1 + random.below(3) as u8;
                    let side = sides[random.below(2)];
                    field.press(random.below(14), side, clicks, random.below(2) == 1);
                }
                8 => field.drag(random.below(20) as isize - 4, sides[random.below(2)]),
                _ => field.set_columns(random.below(12)),
            }
            check(
                &mut field,
                &before,
                &format!("seed {seed}, step {step}, action {action}"),
            );
        }
        // Redoing what was undone, and then undoing and redoing everything,
        // goes from the last text to the empty one and back.
        while field.redo() {}
        let last = field.text.clone();
        while field.undo() {}
        assert_eq!(field.text(), "", "seed {seed}");
        while field.redo() {}
        assert_eq!(field.text, last, "seed {seed}");
    }
}
```

**A proba** fai 300 series de 45 accións, cada unha cun campo novo cun límite ao chou, de 4 a 15 unidades, para que o límite se alcance a miúdo. A semente multiplícase por `0x9e37_79b9_7f4a_7c15`, a razón áurea en 64 bits, para que as 300 secuencias empecen ben distintas, e ningunha é 0, coa que *xorshift* só daría ceros.

| Acción | Cantas veces | Que fai |
|---|---|---|
| Escribir | 4 de 10 | Unha peza de `PIECES`. |
| Unha tecla | 3 de 10 | Unha de `KEYS`, soa, con Maiús, Ctrl, Ctrl+Maiús, Alt ou Alt+Maiús. Se copia ou corta, o texto vai a un portapapeis da proba; e se pide pegar, pega o que ten. |
| Un clic | 1 de 10 | Simple, dobre ou triplo, con Maiús ou sen ela, nun lado dunha das celas da 0 á 13, que poden estar fóra do campo. |
| Arrastrar | 1 de 10 | Ata unha cela da −4 á 15, tamén fóra do campo, polos dous lados. |
| Outras celas | 1 de 10 | De 0 a 11. |

- Despois de cada acción, `check`, cun contexto que di a semente, o paso e a acción, para repetir un fallo.
- **Ao final, desfacer todo volta ao principio.** Refai o que quedou desfeito, e garda o texto; desfai todos os pasos, e o texto ten que ser o baleiro, o do campo novo; e refaios todos, e ten que ser outra vez o último. Cada acción fai como moito dúas edicións, coma di o comentario, así que as 45 fan como moito 90, menos ca os 100 pasos: ningún se perde. Así comproba que os sitios e os textos de todos os pasos, xuntos ou non, encaixan uns cos outros.
- **Atopou un fallo** mentres se escribía o campo: o arrastre por palabras despois dun dobre clic seguía coa palabra do texto de antes ([`edited`](#editar-replace-e-edited)).

**Como se comproba.** Só coas probas, a cobertura das liñas de `history.rs`, `keys.rs`, `segment.rs` e `mod.rs` é do 100 %: a primeira medida atopou en `replace` unha garda á que non chegaba ningunha chamada, e quitouse (`67295da`) ([Tecnoloxías › A cobertura e os mutantes](../tecnoloxias.md#64-a-cobertura-e-os-mutantes)). `cargo mutants --in-diff`, co *diff* da 3.2, fixo 323 mutantes: 252, collidos polas probas; 22, que fan que as probas non rematen, cun bucle sen fin ao camiñar polos grafemas ou polas palabras (*timeout*); 41, que non compilan (*unviable*); e 8, que sobrevivían (*missed*). Un mutante que sobrevivía, `<` por `<=` en `follow_caret`, era equivalente, e `follow_caret` usa agora un `clamp`, sen esa comparación (`cc18533`). Outros sete sobrevivían no mapa das teclas: coas gardas de C, V e Y sempre certas, ou con `||` en vez de `&&` nas de C, V, Y e Z, as letras soas copiaban, pegaban, desfacían ou refacían, e ningunha proba o vía, porque ningunha escribía esas letras coas súas teclas; `the_letters_of_the_commands_write_themselves_without_ctrl` (`88dd960`) cólleos os sete. E o exemplo de consola probouse enteiro a través dun ConPTY, con teclas enteiras ([Código › meiga-win › `examples/field.rs`](meiga-win.md#cratesmeiga-winexamplesfieldrs)); documentándoo, atopouse que outro número de celas non era un cambio ([`set_columns`](#os-cambios-e-as-celas-take_changes-e-set_columns), `b96fa25`). Despois, a revisión independente do código atopou dous fallos máis, arranxados en `a99de8c`, cadanseu cunha proba que falla sen o arranxo: o campo tiraba os caracteres de Ctrl+Alt que non eran do AltGr exacto ([`key`](#key-a-entrada-das-teclas)), e unha edición que deixaba o mesmo texto era un cambio do texto ([`edited`](#editar-replace-e-edited)). E, documentando o código, o axente da documentación atopou o resto do que arranxou `50d2379`: a proba ao chou non vía un cursor fóra da vista ([`check`](#a-proba-ao-chou-random_edits_keep_the_field_whole_and_undo_to_the_start)); ningunha proba comprobaba que `set_text` esquece a metade dun carácter e o arrastre; `show` miraba unha condición que sempre se cumpría, e `follow_caret` movía sen celas unha parte do texto que nada podía ver; e algúns comentarios dicían menos ca o código. Despois de cada arranxo, outra vez os mutantes das funcións que cambiaron: os 74 de `command` e `follow_caret`, collidos todos; os 26 de `key`, `edited`, `replace`, `undo`, `redo` e `track`, 25 collidos e un que non compila; e os 12 de `show`, `follow_caret` e `set_text`, collidos todos.

## `crates/meiga-core/tests/recordings.rs`

Probas de integración: usan o crate desde fóra, como os outros crates, con gravacións reais do ConPTY que leva meiga, feitas co exemplo `conpty` de `meiga-win` e a súa opción `--record` ([Código › meiga-win](meiga-win.md#cratesmeiga-winexamplesconptyrs)). As gravacións están en `tests/data/` e entran no binario das probas con `include_bytes!`, así que as probas non len ningún arquivo:

| Gravación | Que é | Tamaño |
|---|---|---|
| `cmd-echo.mrec` | `cmd /c echo ola`. | 140 bytes |
| `spinner.mrec` | O escenario `tools\perf\scenarios\spinner.ps1`, durante 3 s. | 2 243 bytes |
| `ai-stream.mrec` | O escenario `tools\perf\scenarios\ai-stream.ps1`, durante 12 s: texto con cores, un emoji, `❯` e unha táboa con liñas de caixa. | 45 838 bytes |
| `shell-marks.mrec` | Desde a subfase 4.1: `pwsh -NoLogo -NoProfile -NoExit -WorkingDirectory C:\Windows`, cun *prompt* que escribe OSC 133 e OSC 9;9, e Intro, que escribe OSC 133;C, sen as predicións de PSReadLine; despois, `cd System32`, `cmd /c exit 3` e `exit`. | 4 718 bytes |

- As catro son dunha pseudoconsola de 120 × 30, a do exemplo, e só levan a saída destes comandos, sen datos de ningunha máquina. Están marcadas como binarias no `.gitattributes` ([Código › workspace](workspace.md#gitattributes-e-gitignore)).
- **A de pwsh.** O seu *prompt* escribe D co código de saída da última orde, A, o cartafol entre comiñas con OSC 9;9, o texto do *prompt*, coma `PS C:\Windows>`, e B; o primeiro, sen D. Intro escribe C antes da saída da orde. Despois de `exit` non vén ningún *prompt*.
- `feed` dálle uns anacos a un terminal novo, e devolve o que mostra (`Shown`): o texto e as celas de cada fila, o cursor, o título, os modos, as respostas e, desde a 4.1, o que dixo a shell, o cartafol, a última marca e o código de saída.

| Proba | Que comproba |
|---|---|
| `cmd_writes_ola` | A primeira fila é `ola`, e o resto están baleiras; o cursor, na seguinte. O win32-input-mode está activo mentres corre o programa, e ao final o ConPTY desactívao. A única resposta é a de DA1, que o ConPTY pide ao arrancar. |
| `the_spinner_only_changes_what_it_draws_again` | O primeiro fotograma do *spinner* cambia a liña enteira (16 celas); os seguintes, co cursor oculto, só o símbolo, e o número dos segundos cando cambia: 1 ou 13 celas. Ao final, o símbolo nunha cor do cubo da paleta, `Thinking` en negra e os segundos atenuados, e o cursor visíbel outra vez, na fila seguinte: unha barra que pestanexa, porque o programa non escolleu outro cursor. |
| `the_ai_answer_keeps_its_text_and_colours` | O emoji 💡 nas súas dúas metades, `❯` en cian e en negra (`Indexed(6)`), un título na cor 141 da paleta, a táboa con liñas de caixa e o cursor ao final. |
| `the_shell_says_where_it_is_and_how_its_commands_end` | Desde a subfase 4.1, coa gravación de pwsh. Bloque a bloque, cada vez que o terminal avisa dun cambio do que di a shell: o cartafol pasa de `C:\Windows` a `C:\Windows\System32`, e o código de saída, de ningún a 0, despois de `cd`, e a 3, despois de `cmd /c exit 3`; ao final, a marca é C, porque `exit` corre e despois non vén ningún *prompt*. As catro primeiras filas son as tres ordes, cos seus *prompts* e sen nada dos OSC, e unha baleira; e `Shown` leva o que dixo a shell: `C:\Windows\System32`, C e 3. |
| `how_the_output_is_split_does_not_change_what_is_shown` | Cada gravación nun só anaco, byte a byte (cada secuencia e cada carácter de varios bytes partidos) e en 20 particións aleatorias mostra exactamente o mesmo, desde a 4.1 tamén o que dixo a shell. |
| `copying_only_what_changed_gives_the_same_screen` | A mesma invariante ca en `tests.rs`, coa saída real: despois de cada bloque, a copia só do dano é igual á copia enteira. Desde a 4.1, tamén coa gravación de pwsh. |

## `crates/meiga-core/benches/scanner.rs`

Desde a subfase 4.1: o *benchmark* do escáner, que mide o que custa fronte ao tempo que leva o terminal en ler a mesma saída, que o inclúe ([ADR 0032](../adr/0032-o-escaner-osc.md), decisións 1 e 6). O plan pide que custe menos do 1 % ([`scanner.rs`](#cratesmeiga-coresrctermscannerrs)).

```text
cargo bench -p meiga-core --bench scanner -- [--rounds <n>] [<recording>...]
```

- **Sen o arnés das probas** (`harness = false`, no [`Cargo.toml`](#cratesmeiga-corecargotoml)): os *benchmarks* do arnés, con `#[bench]`, só compilan con Rust *nightly*. Este é un programa co seu `main`, que mide el mesmo con `Instant`. Tampouco ten dependencias: `criterion` ou `divan` traerían moitas máis, para dúas medidas emparelladas que caben nun arquivo (ADR 0032, alternativas). `cargo bench` compílao co perfil `bench`, que herda o de *release*.
- **As saídas.** Cinco que fai el, de 4 MiB cada unha, en bloques de 64 KiB, coma os que le meiga do ConPTY, para un terminal de 120 × 30 (`Output::made`):

  | Saída | Que é |
  |---|---|
  | `colours` | Un rexistro cunha cor *truecolor* para cada carácter: a saída con máis secuencias, unhas 5 ESC cada 100 bytes. |
  | `brackets` | Listas dentro de listas: un `]` cada 4 bytes, e ningún ESC. |
  | `text` | Palabras, sen secuencias. |
  | `marks` | Unha shell coas súas marcas e o seu cartafol, OSC 133 e 9;9, en cada *prompt*, cada 20 filas: moito máis a miúdo do que ninguén escribe ordes. |
  | `links` | Arquivos con ligazóns, dous OSC 8 en cada fila, coma os escribe `ls --hyperlink`. |

  Despois, as catro gravacións das probas de `meiga-core`, que entran no programa con `include_bytes!`, e as que se lle dean, cos seus bloques e o seu tamaño (`Output::recorded`).
- **Cada rolda** le cada saída cun escáner novo e despois cun terminal novo, do tamaño da saída: o escáner, bloque a bloque, contando o que atopa; e o terminal, con `feed`, que inclúe o escáner, e con `end_sync` se queda aberta unha actualización sincronizada (`round`). `black_box` impide que o compilador quite un traballo cuxo resultado non se usa, ou que adiante algo porque coñece os bytes. Primeiro fai unha rolda que non conta, para ter todo na memoria; despois, 11, salvo que `--rounds` diga outra cousa.
- **O que mostra**, unha fila por saída: os bytes, os bloques, o que atopou o escáner, a mediana dos tempos do escáner e do terminal, e o tempo do escáner fronte ao do terminal, en %: a mediana das roldas, e a máis baixa e a máis alta. Cada porcentaxe compara as dúas medidas dunha rolda, coa mesma saída. `median` ordena os valores e dá o do medio; despois, as porcentaxes quedan ordenadas, e a primeira e a última son a máis baixa e a máis alta. `max(1e-9)` evita dividir por cero.
- **Unha ruta relativa** dunha gravación tómase desde o cartafol do *workspace*, onde se executa cargo (`from_workspace`): `cargo bench` executa o *benchmark* no cartafol de `meiga-core`, e unha ruta coma `.perf\bigfile.mrec` non se atoparía (`e7be009`). `env!("CARGO_MANIFEST_DIR")` é o cartafol de `meiga-core` ao compilar, e os dous `..` levan ao do *workspace*.
- `--bench`, o argumento que `cargo bench` lle dá a todos os *benchmarks*, ignórase. Unha opción que non existe, ou `--rounds` sen un número maior ca 0, mostra o uso e remata co código 2; unha gravación que non se pode ler, co 1.

Por exemplo, coas gravacións das probas e unha de `bigfile`, feita co exemplo `conpty` e a súa opción `--record` coma na [subfase 1.5](../subfases/1.5-fachada.md):

```text
cargo bench -p meiga-core --bench scanner -- .perf\bigfile.mrec
```

```text
Output                   Bytes  Blocks  Found   Scanner ms  Terminal ms  Scanner/terminal, in % (lowest–highest)
colours                4194304      64      0            …            …  … (…–…)
…
shell-marks.mrec          2292     201     14            …            …  … (…–…)
…
```

**O que mediu na 4.1**, o escáner fronte ao terminal:

| Saída | Do tempo do terminal |
|---|---|
| `text` | 0,5 % |
| `brackets` | 0,6–0,7 % |
| `colours` | 1,1 % |
| `marks` | Arredor do 2,8 % |
| `links` | Arredor do 1,7 % |
| `ai-stream.mrec` | 0,45 % |
| `bigfile` | 0,75 % |

Co escáner de antes, que se paraba en cada ESC, `bigfile` levaba o 3,3 %; e buscando só a parella `ESC ]` con `memmem`, `colours`, o 1,3 % ([ADR 0032](../adr/0032-o-escaner-osc.md)). Os OSC moi seguidos custan máis, pero a ritmos que ninguén escribe.

**Na CI**, o paso *Check the scanner measurement* do job `windows` execútao con `--locked`, despois da medida dos fotogramas, e mostra os números sen xulgalos, coma as medidas do arranque e dos fotogramas ([Código › GitHub › Job `windows`](github.md#job-windows)).

## `crates/meiga-core/examples/replay.rs`

Un exemplo de consola que lle dá unha gravación a un terminal de meiga tan rápido como pode, e mostra a que velocidade a leu: o que fai o fío lector coa saída do programa, sen a pseudoconsola e sen debuxar.

```text
cargo run --release -p meiga-core --example replay -- <recording> [--rounds <n>] [--screen]
```

- Cada rolda dálle todos os bloques, en orde e tal como chegaron, a un terminal novo do tamaño da gravación, e mide o tempo, en MB de saída por segundo (1 MB = 1 000 000 bytes). Hai 5 roldas, salvo que `--rounds` diga outra cousa, e ao final mostra a mediana, a mínima e a máxima. A velocidade só di algo cunha gravación grande: as pequenas len en microsegundos.
- **As actualizacións sincronizadas** que o programa non remata móstranse cando vence o seu tempo, contado cos momentos da gravación, como fai meiga desde a subfase 1.6 co temporizador dos fotogramas: `sync_end` é o momento da gravación en que vence a que está pendente, e o primeiro bloque que chega despois remátaa con `end_sync`.
- As respostas recóllense despois de cada bloque, como fai o fío lector, que llas envía ao programa.
- **`--screen`** mostra ao final o que hai na pantalla: as filas con texto, numeradas desde 1 como nas secuencias de control; o cursor; o título; desde a subfase 4.1, o que dixo a shell, o seu cartafol, de Windows ou dunha URL co seu equipo, a súa última marca, coa súa letra, e o código de saída da última orde; os modos activos; e as respostas, cos caracteres de control á vista (␛ é ESC, ␍ é CR…), como o exemplo `conpty`. Se queda unha actualización sincronizada pendente, móstraa antes.
- Os erros de uso (sen gravación, unha opción que non existe, `--rounds` sen un número maior ca 0 ou dúas gravacións) mostran o uso e rematan co código 2; unha gravación que non se pode ler, co 1.

Por exemplo, coa gravación de `cmd /c echo ola`:

```text
crates\meiga-core\tests\data\cmd-echo.mrec: 120 × 30, 6 blocks, 54 bytes of output in 0.1 s.
Round 1: 0.0001 s, … MB/s.
…
Median: … MB/s (from … to …).
Screen of 120 × 30, the rows with text:
   1 │ola
Cursor: row 2, column 1, Beam, blinking.
Title: (none)
Folder: (none)
Mark: (none)
Exit code: (none)
Modes: alternate_scroll, show_cursor
Answers: ␛[?6c
```

Coa gravación de pwsh, `shell-marks.mrec`, as liñas do que dixo a shell son:

```text
Folder: C:\Windows\System32
Mark: C, the output
Exit code: 3
```

## `crates/meiga-core/examples/tabs.rs`

Desde a subfase 2.1, un exemplo de consola para probar o modelo das tabs a man, sen xanela e sen programas nas tabs: le ordes, unha por liña, e despois de cada unha mostra a barra en texto.

```text
cargo run -p meiga-core --example tabs
```

```text
Commands: new, close TAB, move TAB POSITION, activate TAB, next, previous, help, quit.
TAB is the name of a tab, and POSITION counts the tabs from 1.
[a]
> new
a [b]
```

- Cada tab chámase cunha letra, `a`, `b`, `c`…, na orde na que se abriu, e despois de `z`, `aa`, `ab`… (`Names::fresh`). O contido de cada tab é o seu nome: a xanela é unha `Window<String>`.
- A barra mostra a tab activa entre corchetes (`bar`).
- As ordes van en inglés, coma as outras mensaxes de meiga:
  - `new` abre unha tab ao final e actívaa, coma o botón + desde a subfase 2.3;
  - `close TAB` pecha unha tab; se é a única, a xanela pecharíase, e o exemplo remata;
  - `move TAB POSITION` move unha tab a unha posición, contada desde 1, coma na barra;
  - `activate TAB`, `next` e `previous` activan unha tab;
  - `help` mostra as ordes, e `quit` remata.
- Un nome que non existe, unha posición que non o é e unha orde que non entende mostran unha mensaxe, e a xanela queda igual.
- `run` interpreta cada orde cun `match` sobre as palabras da liña (`["move", name, position]`…) e responde que mostrar (`Outcome`).
- Se as ordes chegan por unha tubaxe, e non desde o teclado, non mostra o indicador `> ` antes de ler, senón despois, xunto con cada orde, para que a saída se lea coma unha sesión. `IsTerminal` di se a entrada é unha consola. Así o usan os pasos de comprobación da subfase.
