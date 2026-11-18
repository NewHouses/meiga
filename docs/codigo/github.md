# Código › `.github/` (integración continua e Dependabot)

> Estado: mantemento M2, despois da subfase 2.8. No M2, o *workflow* non cambia: `check-exit.ps1` comproba tamén que meiga agarda antes de saír por unha sesión que se bloqueou ao pecharse a xanela; `check-profiles.ps1` e `check-tabs.ps1` agardan a que se calmen as shells das tabs novas, e os programas que arrancan, e `check-profiles.ps1` dálle a meiga máis tempo para saír; e `measure-startup.ps1` agarda 10 s máis a que remate meiga. Na 2.8 chegou ao *workflow* o paso *Check many tabs*, con `check-manytabs.ps1`; na 2.7, o paso *Check the end of the programs*, con `check-exit.ps1`; na 2.6, o paso *Check the keyboard shortcuts*, con `check-shortcuts.ps1`; na 2.5, o paso *Check the profiles and the configuration*, con `check-profiles.ps1`; na 2.4, o paso *Check the menu of a tab*, con `check-menu.ps1`; na 2.3, o paso *Check the tab bar with the mouse*, con `check-tabbar.ps1`; na 2.2, o paso *Check the tabs*, con `check-tabs.ps1`; e no mantemento M1, a regra de `dependabot.yml` para `windows-core`.

O cartafol `.github/` contén a configuración que usa GitHub: o *workflow* de integración continua e a configuración de Dependabot. Os conceptos (*workflow*, *job*, *runner*…) explícanse en [Tecnoloxías › GitHub Actions](../tecnoloxias.md#15-github-actions).

## `.github/workflows/ci.yml`

### Nome e eventos

```yaml
name: CI

on:
  pull_request:
    branches: [main]
  push:
    branches: [main]
  schedule:
    # Every Monday, so that new security advisories and new versions of the
    # ConPTY are reported even when nothing changed. Only the cargo-deny job
    # and the ConPTY version check run on schedule.
    - cron: "17 5 * * 1"
  workflow_dispatch:
```

- `name: CI`: o nome que aparece na lapela *Actions* de GitHub e na insignia do README.
- `on`: os eventos que lanzan o workflow.
  - `pull_request` cara a `main`: valida cada PR antes de integralo. GitHub non proba a rama do PR tal cal, senón o resultado de mesturala con `main` (a referencia `refs/pull/N/merge`). É dicir, proba o que quedaría en `main`.
  - `push` a `main`: valida o resultado final de cada integración e garda a caché que reutilizarán os PR.
  - `schedule`: unha execución semanal, para detectar vulnerabilidades novas en dependencias que non cambiaron e versións novas do ConPTY. O formato `cron` ten cinco campos: minuto, hora, día do mes, mes e día da semana. `17 5 * * 1` significa os luns ás 05:17 (UTC). O minuto 17 é intencionado: GitHub recomenda evitar as horas en punto, cando hai máis carga e as execucións programadas poden atrasarse.
  - `workflow_dispatch`: permite lanzalo a man desde a web, co botón *Run workflow*.

### Permisos

```yaml
permissions:
  contents: read
```

Cada execución recibe un *token* temporal (`GITHUB_TOKEN`) para operar co repositorio, que pode ter permisos de escritura. Eiquí limítase a ler o código. Aínda que un paso estivese comprometido, non podería modificar o repositorio, crear versións nin escribir nos PR. É o principio de **mínimo privilexio**.

### Concorrencia

```yaml
concurrency:
  group: ${{ github.workflow }}-${{ github.ref }}
  cancel-in-progress: ${{ github.event_name == 'pull_request' }}
```

- `${{ … }}` é unha **expresión** de GitHub Actions: GitHub avalíaa antes de executar o workflow.
- Dúas execucións co mesmo `group` nunca corren á vez. O grupo combina o nome do workflow e a referencia, que é distinta para cada PR e para `main`.
- `cancel-in-progress`: nun PR, se chega un commit novo mentres a CI aínda valida o anterior, a execución vella cancélase, porque o seu resultado xa non interesa. En `main` non se cancela nada: cada integración valídase enteira.

### Variábeis de contorno

```yaml
env:
  CARGO_TERM_COLOR: always
  # Do not set RUSTFLAGS: it would replace the rustflags in .cargo/config.toml,
  # including +crt-static.
```

- `CARGO_TERM_COLOR: always`: Cargo usa cores aínda que a súa saída non vaia a un terminal, e os rexistros lense mellor na web.
- O comentario avisa dunha trampa real. Cargo ten varias fontes de opcións para o compilador e usa só a primeira que atopa: a variábel `RUSTFLAGS` vai antes que os `rustflags` de `.cargo/config.toml`. Se alguén a define (por exemplo, con `-D warnings`, un costume habitual nas CI de Rust), Cargo ignora `.cargo/config.toml` e perdemos o `+crt-static`. Comprobouse nesta subfase: con `RUSTFLAGS='-C opt-level=3'`, `meiga.exe` voltou a depender de `VCRUNTIME140.dll`. O script [`check-imports.ps1`](ferramentas.md#toolscheck-importsps1) detecta este caso.

### Job `windows`

```yaml
  windows:
    name: Build and test (Windows)
    if: github.event_name != 'schedule'
    runs-on: windows-2025
    timeout-minutes: 30
    defaults:
      run:
        shell: pwsh
```

- `windows` é o identificador interno do job, e `name` o nome visíbel. **As regras da rama `main` esixen o check por este nome**, así que, se cambia, hai que actualizar as regras.
- `if`: o job non corre nas execucións programadas, que só buscan vulnerabilidades novas.
- `runs-on: windows-2025`: o *runner*, unha máquina virtual de GitHub con Windows Server 2025. Indícase a versión exacta en vez de `windows-latest` (ver [Tecnoloxías › Imaxes dos runners](../tecnoloxias.md#imaxes-dos-runners)).
- `timeout-minutes: 30`: se algo se bloquea, o job cancélase aos 30 minutos, en vez de agardar as 6 horas do límite por defecto.
- `defaults.run.shell: pwsh`: os pasos `run` execútanse con PowerShell 7, como os scripts de `tools/`.

#### Descargar o código

```yaml
      - uses: actions/checkout@3d3c42e5aac5ba805825da76410c181273ba90b1 # v7.0.1
        with:
          persist-credentials: false
```

- `uses` executa unha *action*: un compoñente reutilizábel publicado noutro repositorio. `actions/checkout` descarga o código do repositorio no runner.
- A versión fíxase co **SHA completo do commit**, e a etiqueta (`v7.0.1`) queda nun comentario. O motivo explícase en [Tecnoloxías › Seguridade do workflow](../tecnoloxias.md#seguridade-do-workflow).
- `persist-credentials: false`: por defecto, `checkout` deixa o *token* gardado na configuración de Git do runner, para que os pasos seguintes poidan facer `git push`. Non o necesitamos, e así ningún paso posterior pode reutilizalo.

#### Comprobar a documentación

```yaml
      - name: Check documentation links
        run: ./tools/check-docs.ps1
```

Comproba que todas as ligazóns internas da documentación apuntan a arquivos e títulos que existen (ver [Código › ferramentas](ferramentas.md#toolscheck-docsps1)). Vai antes de instalar Rust porque non o necesita: se a documentación ten unha ligazón rota, o job falla en segundos.

#### Comprobar que os scripts piden PowerShell 7

```yaml
      - name: Check that the scripts ask for PowerShell 7
        run: ./tools/check-powershell.ps1
```

Comproba que todos os scripts de `tools/` explican como executalos con PowerShell 7 se se abren con Windows PowerShell 5.1: executa cada un en 5.1 e mira a mensaxe (ver [Código › ferramentas](ferramentas.md#toolscheck-powershellps1)). O runner de Windows trae as dúas versións de PowerShell. Leva uns segundos.

#### Comprobar o arnés de medición

```yaml
      - name: Check the measurement harness
        run: ./tools/perf/test-measure.ps1 -Duration 5
```

Executa a proba de `tools/perf/measure.ps1` con procesos de consumo coñecido (ver [Código › medición](perf.md#toolsperftest-measureps1)). Tampouco necesita Rust e leva uns 20 s. Non mide o rendemento de meiga: as máquinas da CI son compartidas e darían cifras con moito ruído. Só comproba que o arnés segue medindo ben, para que un cambio nos scripts non o estrague sen que se note.

#### Comprobar os escenarios de carga

```yaml
      - name: Check the load scenarios in PowerShell
        run: ./tools/perf/test-scenarios.ps1 -Shell pwsh
```

Executa os escenarios de PowerShell e comproba o que debuxan e como se coordinan con `run-scenario.ps1` (ver [Código › escenarios de carga](escenarios.md#test-scenariosps1)). Os de bash compróbanse no [job `scenarios`](#job-scenarios), en Linux, porque o runner de Windows non ten WSL.

#### Instalar Rust

```yaml
      - name: Install the Rust toolchain from rust-toolchain.toml
        run: |
          rustup toolchain install
          rustc --version
          cargo --version
```

- `run` executa comandos, e `|` permite escribir varias liñas.
- Os runners de GitHub xa traen rustup. `rustup toolchain install`, sen argumentos, le `rust-toolchain.toml` e instala esa versión exacta cos seus compoñentes. Non fai falta ningunha action de terceiros: unha dependencia menos.
- Os dous últimos comandos escriben as versións no rexistro, para diagnosticar problemas.

#### Instalar cargo-about

```yaml
      - uses: taiki-e/install-action@3a3daafd3b7f23b8f65d9f5ba2db203fb59b79e2 # v2.90.2
        with:
          tool: cargo-about@0.9.3
```

- `cargo-about` é necesario para empaquetar (xera os avisos de terceiros).
- `taiki-e/install-action` descarga o binario xa compilado e **comproba o seu SHA-256** cunha lista que vén na propia action, que tamén está fixada por SHA. Compilalo con `cargo install` levaría varios minutos en cada execución.
- A versión é a mesma que en local: 0.9.3.

#### Caché

```yaml
      - uses: Swatinem/rust-cache@d1527c8f67ef9af357290475d4af4b0fab8f12e0 # v2.9.3
        with:
          cache-bin: "false"
          save-if: ${{ github.ref == 'refs/heads/main' }}
```

- Garda entre execucións as dependencias descargadas e o cartafol `target/`, para non compilar todo desde cero cada vez. A clave da caché inclúe a versión de Rust e o `Cargo.lock`: se cambian, empézase cunha caché nova. Por iso este paso vai despois de instalar Rust.
- `cache-bin: "false"`: non garda os programas de `~/.cargo/bin`. As ferramentas instálanse verificadas en cada execución, e a caché nunca as substitúe.
- `save-if`: a caché só se garda desde `main`. Os PR poden usar a de `main`, pero unha caché creada nun PR só lle serviría a ese PR e ocuparía espazo: GitHub limita a caché a 10 GB por repositorio.

#### Comprobacións

```yaml
      - name: Check formatting
        run: cargo fmt --all --check

      - name: Clippy
        run: cargo clippy --workspace --all-targets --locked -- -D warnings

      - name: Download the ConPTY for the tests and the package
        run: ./tools/fetch-conpty.ps1

      - name: Test
        run: cargo test --workspace --locked
```

As mesmas comprobacións que en local (ver [Tecnoloxías](../tecnoloxias.md#7-lints-rustc-e-clippy)). Con `--locked`, se o `Cargo.lock` non está ao día, a CI falla en vez de actualizalo pola súa conta.

Desde a subfase 1.4, as probas de `meiga-win` lanzan programas no ConPTY que vai con meiga, así que antes hai que descargalo con [`fetch-conpty.ps1`](ferramentas.md#toolsfetch-conptyps1): sen el, esas probas fallan cunha mensaxe que o di. O script comproba o SHA-256 do paquete e a sinatura de Microsoft, e deixa os dous arquivos en `target\conpty\`, onde os buscan as probas e, máis adiante, `package.ps1`. A caché de Rust só garda o que compila Cargo, así que cada execución descarga o paquete, de 1,7 MB.

#### Empaquetar e comprobar o executábel

```yaml
      - name: Build and package
        run: ./tools/package.ps1

      - name: Check that meiga.exe does not need the Visual C++ runtime
        run: ./tools/check-imports.ps1

      - name: Check that meiga.exe opens its window, runs its program and closes cleanly
        run: ./tools/check-window.ps1

      - name: Check that meiga.exe presents only what changes
        run: ./tools/check-present.ps1

      - name: Check the keys with the Spanish and US keyboard layouts
        run: |
          ./tools/check-keyboard.ps1 -Layout Spanish
          ./tools/check-keyboard.ps1 -Layout US

      - name: Check the mouse, the selection and the clipboard
        run: ./tools/check-mouse.ps1

      - name: Check the resizing, the history and the zoom
        run: ./tools/check-resize.ps1

      - name: Check the tabs
        run: ./tools/check-tabs.ps1

      - name: Check the tab bar with the mouse
        run: ./tools/check-tabbar.ps1

      - name: Check the menu of a tab
        run: ./tools/check-menu.ps1

      - name: Check the profiles and the configuration
        run: ./tools/check-profiles.ps1

      - name: Check the keyboard shortcuts
        run: ./tools/check-shortcuts.ps1

      - name: Check the end of the programs
        run: ./tools/check-exit.ps1

      - name: Check many tabs
        run: ./tools/check-manytabs.ps1

      - name: Check the start-up measurement
        run: ./tools/perf/measure-startup.ps1 -Runs 3 -Warmup 1

      - name: Check the frame measurement
        run: ./tools/perf/measure-frames.ps1 -Duration 3 -Warmup 1
```

- Compila en modo *release*, crea o `.zip` en `dist/`, co ConPTY, e comproba as DLL das que depende `meiga.exe` (ver [Código › ferramentas](ferramentas.md)). Desde a subfase 1.6, `package.ps1` deixa tamén o ConPTY ao lado de `target\release\meiga.exe`, así que os pasos seguintes executan meiga co mesmo ConPTY ca o paquete, e non co de Windows (ver [`package.ps1`](ferramentas.md#toolspackageps1)).
- Desde a subfase 2.5, meiga le ao arrancar a súa configuración, `%APPDATA%\meiga\config.toml` ([ADR 0022](../adr/0022-os-perfis-e-a-configuracion.md), decisión 1), así que os pasos seguintes arráncano cun `APPDATA` propio: as comprobacións, de `check-window.ps1` a `check-menu.ps1`, e `measure-startup.ps1`, cun cartafol temporal baleiro, para que teña os seus valores por defecto; e `check-profiles.ps1` e, desde a 2.6, `check-shortcuts.ps1`, cada unha cun cartafol co `config.toml` que comproba; e, desde a 2.7, `check-exit.ps1` e, desde a 2.8, `check-manytabs.ps1`, cada unha cun cartafol cun `config.toml` cos seus perfís. No *runner* non hai ningún `config.toml`, pero así as comprobacións e as medicións tampouco dependen do de quen as executa no seu equipo. Os programas das tabs reciben o mesmo `APPDATA`, porque herdan o contorno de meiga. `measure-frames.ps1` non o necesita: nos modos de medición, meiga non le a configuración.
- `check-window.ps1` abre `meiga.exe` con `cmd` no seu terminal, e comproba a xanela, a escala por monitor, que o título pasa a ser o que pon cmd, que meiga remata co código 0 ao pechala e que, despois, rematan tamén cmd e o anfitrión da súa consola, `OpenConsole.exe` (ver [Código › ferramentas](ferramentas.md#toolscheck-windowps1)). Os *runners* de Windows de GitHub teñen un escritorio, así que a xanela ábrese de verdade, aínda que ninguén a vexa. Leva uns segundos: cmd espera un segundo antes de poñer o título.
- Ata a subfase 1.5, o paso de `check-window.ps1` chamábase *Check that meiga.exe opens its window and closes it cleanly*. O nome novo di o que comproba desde a 1.6: tamén o programa do terminal. Os nomes dos pasos, ao contrario ca o do job, non os esixen as regras da rama `main`, así que cambialo non obriga a cambiar nada máis.
- `check-present.ps1` lanza en meiga un script de PowerShell que escribe na pantalla paso a paso, e comproba no rexistro dos fotogramas que meiga presenta só o que cambia, agrupa os cambios, presenta a xanela enteira cando cambia a cor do fondo e non debuxa en repouso nin coa xanela minimizada, tamén cando o programa escribe mentres tanto; e que, coa xanela minimizada, meiga remata á súa hora límite unha actualización sincronizada que o programa deixa aberta: o título escrito nela chega á xanela (ver [Código › ferramentas](ferramentas.md#toolscheck-presentps1)). Ata a subfase 1.5 debuxaba na demo con eventos do rato. Leva uns segundos.
- `check-keyboard.ps1`, desde a subfase 1.7, teclea na xanela de meiga e comproba o que le PowerShell no terminal: o carácter, a tecla e as modificadoras de cada tecla (ver [Código › ferramentas](ferramentas.md#toolscheck-keyboardps1)). Corre dúas veces, co teclado español, coas teclas mortas e AltGr, e co estadounidense. O *runner* ten o estadounidense, así que na primeira a comprobación cambia ao español, cargándoo se fai falta, e ao rematar volta poñer o de antes. `run: |` executa as dúas liñas no mesmo paso, unha despois da outra: se a primeira falla, o paso falla sen executar a segunda.
- `check-mouse.ps1`, desde a subfase 1.8, selecciona, copia e pega na xanela de meiga, e comproba o que chega ao portapapeis e o que len tres scripts de PowerShell no terminal: un coma un programa que le o teclado, outro coma un programa da consola de Windows que pide o rato e outro coma un programa de WSL (ver [Código › ferramentas](ferramentas.md#toolscheck-mouseps1)). No *runner*, a xanela de meiga está en primeiro plano, así que comproba que un programa pode copiar ao portapapeis (OSC 52); nunha máquina onde non o está, comproba o contrario. Leva uns segundos.
- `check-resize.ps1`, desde a subfase 1.9, cambia o tamaño da xanela de meiga cun script de PowerShell no seu terminal, que le o búfer da súa consola despois de cada cambio, e comproba que a pantalla de meiga ten as mesmas filas ca o ConPTY, tamén despois da pantalla alternativa e dun cambio de tamaño coa caixa de mensaxe aberta; e despois, o historial coa roda, coas teclas e coa barra de desprazamento, a selección que despraza a vista soa e o zoom (ver [Código › ferramentas](ferramentas.md#toolscheck-resizeps1)). Usa o portapapeis, coma `check-mouse.ps1`, e move o punteiro real por riba da xanela. Leva uns segundos.
- `check-tabs.ps1`, desde a subfase 2.2, abre tabs na xanela de meiga, cambia entre elas e péchaas cos seus atallos, cun script de PowerShell na primeira tab que fai un paso cada vez (ver [Código › ferramentas](ferramentas.md#toolscheck-tabsps1)). Sabe que tab se mostra pola prioridade dos fíos lectores, e comproba no rexistro dos fotogramas que cambiar de tab debuxa un só fotograma completo. E comproba que a saída dunha tab oculta só debuxa a súa tab na barra, unha vez, co indicador de que o seu programa escribiu algo (desde a 2.8, [ADR 0025](../adr/0025-moitas-tabs-e-os-indicadores.md), decisión 7; ata a 2.7, non debuxaba nada); que o seu programa recibe as respostas ás súas preguntas, tamén dentro dunha actualización sincronizada que non remata; que a tab mantén a súa grella, e o seu título non chega á xanela, só á súa tab na barra, ata que se mostra; e que o texto que copia ao portapapeis non chega nunca, tampouco dentro dunha actualización sincronizada que o programa mantén aberta ata que se mostra a tab. No *runner*, a xanela de meiga está en primeiro plano, así que comproba tamén que o texto que copia unha tab á vista si chega, tamén dentro dunha actualización sincronizada que o programa deixa aberta e que a xanela remata á súa hora límite: así demostra que o da tab oculta non chega porque meiga o descarta, e non porque a xanela non poida copiar. Desde o mantemento M2, antes de que escriba a tab oculta, agarda a que a shell da tab nova, e os programas que arranca, coma o git dun *prompt*, leven un segundo sen usar o procesador, e sen que arranque nin remate ningún: entón xa escribiu o seu *prompt*, que, se chegase despois, se tomaría por saída da tab oculta. E agarda o mesmo pola shell da terceira tab antes de pechala: un programa que arranca mentres se pecha a súa consola non deixa rematar a sesión ata 10 s despois ([ADR 0011](../adr/0011-fin-e-peche-das-sesions.md)), e meiga agarda por ela antes de saír ([ADR 0026](../adr/0026-o-peche-das-sesions-ao-sair.md)). Usa o portapapeis, coma `check-mouse.ps1`, e cambia o tamaño da xanela. Leva uns 10 s.
- `check-tabbar.ps1`, desde a subfase 2.3, usa co rato a barra de tabs da xanela de meiga, cun script de PowerShell na primeira tab que fai un paso cada vez, coma `check-tabs.ps1` (ver [Código › ferramentas](ferramentas.md#toolscheck-tabbarps1)). Publica as mensaxes do rato nos sitios da barra, que calcula coma meiga, co fío da xanela suspendido, coma `check-mouse.ps1`, e teclea Ctrl+Tab coma `check-tabs.ps1`. Comproba que o botón + abre unha tab ao soltalo enriba del; que un clic mostra unha tab; que o botón do medio pecha a tab que ten debaixo, e o × da última, a xanela; que un × que se preme e se solta fóra non pecha nada; e que, se Ctrl+Tab mostra outra tab mentres o botón do medio está premido nunha, a barra segue co rato ata que o botón se solta. E comproba no rexistro dos fotogramas que cada cambio debuxa só o que cambia: o punteiro, a tab ou o botón que ten debaixo; un título novo, o seu rectángulo; e pechar unha tab oculta, a barra. Ademais, mide cantas veces esperta o fío da xanela mentres un programa cambia o título tan rápido como pode, na tab que se mostra e nunha oculta: como moito 1 000 por segundo, porque a xanela colle os títulos como moito unha vez por intervalo de fotograma ([ADR 0020](../adr/0020-a-barra-de-tabs.md), decisión 5); e o mesmo cando o programa pon o mesmo título unha e outra vez, que ademais non debuxa nada. Ao contrario ca os orzamentos, ese límite ten sentido nunha máquina compartida: separa unhas decenas de espertares por segundo, os de quen colle os títulos no fotograma, dos miles de quen collese cada un. Para iso necesita o ConPTY de meiga, que `package.ps1` deixou ao lado de `meiga.exe`: sen el, meiga usaría a consola de Windows, que xunta os títulos, e a comprobación avisa de que o límite non se comproba de verdade. Move o punteiro real fóra da xanela, se está enriba, e ao rematar devólveo a onde estaba. Leva uns 25 s.
- `check-menu.ps1`, desde a subfase 2.4, usa co rato e co teclado o menú dunha tab da xanela de meiga, cun script de PowerShell na primeira tab que fai un paso cada vez e le as teclas que lle chegan, coma `check-tabbar.ps1` (ver [Código › ferramentas](ferramentas.md#toolscheck-menups1)). Publica as mensaxes do rato e das teclas co fío da xanela suspendido, nos sitios da barra e do menú, que calcula coma meiga: só o ancho do menú, que depende da fonte, sae do fotograma que o abre. Comproba que a tecla de menú e Maiús+F10 abren o menú da tab que se mostra, debaixo dela, mentres F10 só vai ao programa, e que o botón dereito abre o da tab onde se solta, no punteiro, sen mostrala; que Intro e os clics escollen os elementos: *Duplicate tab* abre unha copia da tab xusto á súa dereita, *Close tabs to the right* e *Close other tabs* pechan as súas tabs, e *Close tab*, unha tab oculta, sen cambiar a que se mostra, e na última, a xanela; que un elemento desactivado, *Close tabs to the right* na última tab ou, desde a 2.7, *Restart tab* mentres corre o programa da tab, non se resalta nin se escolle, e que as frechas saltan a liña e os desactivados e dan a volta; e que un clic fóra pecha o menú sen facer nada máis, e que tamén se pecha cando a xanela cambia de tamaño e cando se minimiza, e que Esc, cun botón premido no menú, o pecha e solta o rato. E comproba no rexistro dos fotogramas que abrir o menú debuxa só o menú; mover o resalte, só os resaltes; e pechalo, o que tapaba; e que, mentres está aberto, non se debuxa nada, tampouco cando o programa escribe debaixo del, e si o que escribe ao seu lado. Polas teclas que le o script, comproba que o programa non recibe ningunha das que colle o menú, nin ao baixar nin ao subir, pero si a Maiús que baixara para el antes de abrirse, e a Ctrl que baixara antes e se repetiu co menú aberto. Move o punteiro real fóra da xanela, se está enriba, e ao rematar devólveo a onde estaba; cambia o tamaño da xanela, e minimízaa e restáuraa. Leva uns 30 s.
- `check-profiles.ps1`, desde a subfase 2.5, comproba os perfís e a configuración de meiga (ver [Código › ferramentas](ferramentas.md#toolscheck-profilesps1)). Arranca meiga cun `APPDATA` propio, cun `config.toml` que fai perfil por defecto a *Probe*, un script de PowerShell que corre noutro cartafol, cunha ñ no nome, oculta o Command Prompt e ten dous erros; e dálle a meiga as variábeis do Windows Terminal e outros valores para `TERM_PROGRAM`, `TERM_PROGRAM_VERSION`, `COLORTERM` e `WSLENV`, coma se o abrisen o Windows Terminal ou VS Code. Atopa as shells instaladas coma meiga, nos cartafoles dos seus instaladores e no rexistro, e publica as mensaxes do rato e das teclas co fío da xanela suspendido, nos sitios da barra e do menú, que calcula coma meiga, coma `check-menu.ps1`. Comproba que, coa xanela xa na pantalla e pintada, unha caixa de mensaxe diante dela di os dous erros, coa súa liña e a súa columna, mentres o programa da primeira tab xa corre, e que ao pecharse a xanela pinta todo outra vez; que a primeira tab corre *Probe*, coas variábeis `%NOME%` expandidas, no seu cartafol, e que o seu programa recibe `TERM_PROGRAM=meiga`, `TERM_PROGRAM_VERSION` e `COLORTERM=truecolor`, un `WSLENV` con eses nomes e sen os do Windows Terminal, e non `WT_SESSION` nin `WT_PROFILE_ID`; que Ctrl+Maiús+Espazo e o botón ▾ abren o menú dos perfís debaixo dos botóns + e ▾, sen o Command Prompt e con *Probe* ao final, que as repeticións do Espazo non abren nada, que un clic en *Probe* abre unha tab con el ao final, e que cada cambio debuxa só o que cambia; que, no menú, cada perfil que atopa meiga abre unha tab que corre a liña de comandos que espera, no cartafol persoal, e os de WSL, co `--cd "~"`, no de Linux, e que Ctrl+Maiús+W a pecha e remata o seu programa; que o + abre *Probe*, o perfil por defecto, e *Duplicate tab* cópiao, coa mesma liña de comandos e no mesmo cartafol; que meiga remata co código 0 ao pechar a xanela, e rematan tamén os programas das súas tabs e as súas consolas; e que, cun `config.toml` que non é TOML, a caixa di onde está o erro e que meiga ignorou o arquivo, e a primeira tab corre a shell por defecto, e non o perfil que di o arquivo. Le a liña de comandos de cada programa con WMI, e o seu cartafol, na memoria do seu proceso. Abrir cada perfil arranca de verdade a súa shell, que pode cargar os scripts de inicio do usuario, e WSL pode ter que arrancar: por iso agarda, como moito 10 s, a que cada unha leve un segundo sen usar o procesador antes de pechala; desde o M2, tamén os programas que arrancou, sen que arranque nin remate ningún durante ese segundo. O `bash.exe` de Git Bash é un lanzador, que agarda sen usar o procesador pola shell que arranca, mentres esta aínda arranca programas, coma o git do seu *prompt*; e un programa que arranca mentres se pecha a súa consola non deixa que se peche, e a sesión remata os seus programas 10 s despois ([ADR 0011](../adr/0011-fin-e-peche-das-sesions.md)). Pasou coa tab de Git Bash no primeiro intento da CI do *push* da 2.7 a `main`, cando a comprobación só miraba a shell ([ADR 0026](../adr/0026-o-peche-das-sesions-ao-sair.md)). E, desde o M2, ao pechar a xanela dálle a meiga para saír o mesmo tempo ca ás shells do usuario para rematar, 30 s por defecto: se unha delas aínda traballaba cando se pechou a súa tab, a súa sesión pode bloquearse, e meiga agarda por ela. Move o punteiro real fóra da xanela, se está enriba, e ao rematar devólveo a onde estaba.
- `check-shortcuts.ps1`, desde a subfase 2.6, comproba os atallos de teclado de meiga: os que trae, os que engade, cambia e libera o `config.toml`, e as teclas que lle deixa ao programa (ver [Código › ferramentas](ferramentas.md#toolscheck-shortcutsps1)). Arranca meiga cun `APPDATA` propio, cun `config.toml` con dous perfís que corren o mesmo script de PowerShell, *Probe*, o perfil por defecto, e *Probe 2*, e que oculta as shells que atopa meiga. A súa táboa `[shortcuts]` libera Ctrl+6, dálle a Ctrl+Alt+2 a segunda tab, a Ctrl+8 a última no canto da oitava, e a Alt+O e a Alt+R `close_other_tabs` e `close_tabs_to_the_right`, dúas accións sen atallo propio, e ten catro erros. Teclea os atallos coma `check-profiles.ps1`, co fío da xanela suspendido, e coma os envía un teclado: as modificadoras baixan, a tecla baixa e sobe, e as modificadoras soben. Comproba que unha caixa de mensaxe di os erros de `[shortcuts]`, coa súa liña e a súa columna; que Ctrl+T, Ctrl+W e Ctrl+6, que libera o arquivo, chegan ao programa; que Ctrl+Maiús+T abre unha soa tab, tamén mentres o T se repite; que AltGr+2, que Windows dá coma o Ctrl esquerdo e o Alt dereito, chega ao programa, e Ctrl+Alt+2, co Alt esquerdo, mostra a segunda tab; que Ctrl+Maiús+D abre unha copia da tab á súa dereita; que Ctrl+1, Ctrl+2 e Ctrl+3 mostran as tabs desas posicións, e Ctrl+9 a última, cada unha cun fotograma completo, e que Ctrl+4, con tres tabs, non debuxa nada nin lle chega ao programa; que Ctrl+AvPáx e Ctrl+RePáx mostran a seguinte e a anterior, dando a volta, e que RePáx segue mentres se repite; que AvPáx, que baixou para o programa antes ca Ctrl, lle segue chegando mentres se repite e ao subir, e non mostra ningunha tab; que Ctrl+Maiús+2 e Ctrl+Maiús+1 abren ao final os perfís desas posicións do menú, onde os mostran Ctrl+4 e Ctrl+5, e Ctrl+Maiús+3, con dous perfís, non abre nada nin lle chega ao programa; que Ctrl+8 mostra a última de cinco tabs, coma di o arquivo; que Alt+R pecha as tabs á dereita da que se mostra, e as da súa esquerda quedan, e Alt+O, coa do medio, as outras, aos dous lados, e o programa recibe o Alt, e non o R nin o O; e que Ctrl+Maiús+W pecha a última tab, e con ela a xanela. Despois das teclas cuxo efecto non se ve, teclea un Z e agarda a que lle chegue ao programa da tab que se mostra: entón a xanela xa colleu todas as de antes. Leva uns 30 s.
- `check-exit.ps1`, desde a subfase 2.7, comproba o que fai meiga cando remata o programa dunha tab ([ADR 0024](../adr/0024-o-fin-do-programa-e-o-reinicio.md); ver [Código › ferramentas](ferramentas.md#toolscheck-exitps1)). Arranca meiga cun `APPDATA` propio, cun `config.toml` con dous perfís que corren o mesmo script de PowerShell, *Probe*, o perfil por defecto, e *Later*, nun cartafol que ao principio non existe, e que oculta as shells que atopa meiga. O script de cada tab escribe nun arquivo o seu cartafol e o tamaño da súa consola, e noutro as teclas que lle chegan; escribe o texto que lle dá a comprobación, e remata cando ela llo di, cun código de saída, despois dun texto ou sen el. A comprobación le as liñas do terminal cun triplo clic, que as copia ao portapapeis, e teclea coma `check-shortcuts.ps1`; despois das teclas que non mostran nada, publica un movemento do rato enriba dun botón da barra: cando a xanela debuxa o botón, xa colleu as teclas. Sabe que tab se mostra pola prioridade dos fíos lectores, coma `check-tabs.ps1`, e atopa os programas das tabs, coa súa liña de comandos, con WMI. Comproba que, cando un programa remata co código 0, a súa tab pecha: unha oculta, e só se debuxa a barra, mentres o menú dos perfís, aberto despois do menú desa tab, ou o doutra tab segue aberto, e despois o menú dos perfís colle Esc; unha oculta co seu menú aberto, que se pecha con ela, e entón o programa da tab que se mostra recibe Esc; e a que se mostra, cun fotograma completo que mostra a da súa esquerda: mentres o menú dos perfís segue aberto, e despois o menú colle Esc, e co menú da súa tab aberto, que se pecha con ela, e entón o programa da tab da esquerda recibe Esc; que, cando o programa da última tab remata co código 3 despois de escribir `bye`, a tab queda, con `bye`, debaixo a liña do código nunha fila súa, e debaixo o aviso, e meiga segue; que entón as teclas que non son atallos non fan nada nin debuxan nada, e a selección segue, e Ctrl+Intro e AltGr+D nin reinician o programa nin pechan a tab; que Intro, que se repite unha vez, reinicia o programa na mesma tab, coa mesma liña de comandos e no mesmo cartafol, e móstrao cun fotograma completo, e o programa novo non recibe nin a repetición nin o Intro que sobe, e o que escribiu o anterior está no historial, onde o mostra Ctrl+Maiús+Inicio; que, mentres corre o programa dunha tab, *Restart tab* está desactivado no seu menú: Maiús+F10 ábreo con *Duplicate tab* resaltado, e a frecha abaixo leva a *Close tab*, que Intro escolle; que, cando o programa remata co código 1 ao principio dunha fila, a liña do código vai nesa fila; que, cando o programa dunha tab oculta escribe unha liña e dá un título, só se debuxa a súa tab na barra, e cando despois remata co código 4, a tab queda, e só se debuxa outra vez a súa tab, co indicador do fin (desde a 2.8; ata a 2.7, non se debuxaba nada); que no menú que abre nela o botón dereito, a frecha abaixo dúas veces resalta *Restart tab*, e Intro reinicia o programa na tab sen mostrala, co seu fío lector coa prioridade baixa das tabs ocultas desde o principio, que a comprobación mira en canto aparece o fío; que Ctrl+D pecha a tab que se mostra, cuxo programa rematou, e mostra a seguinte, cuxo programa non recibe nin o D nin o Ctrl que soben; que Ctrl+Maiús+2 abre *Later*, e a súa tab di por que non puido arrancar o programa, e debaixo o aviso, e Intro téntao outra vez, cun fotograma completo, e cando o cartafol xa existe, arráncao nel; que, coa caixa de mensaxe que pregunta antes de pegar aberta, rematan co código 0 os programas dunha tab oculta e da que se mostra, e cando a caixa se pecha, pechan as dúas tabs, e a última, a xanela: meiga remata co código 0, e tamén as consolas; que, cando remata co código 0 o programa da liña de comandos de meiga, `cmd /c exit 0`, meiga remata tamén co código 0; e, desde o M2, que, cando se pecha a xanela e o programa da primeira tab, ao pecharse a súa consola, arranca outro nela, que se conecta á consola mentres o ConPTY a pecha, a sesión desa tab queda bloqueada: meiga agarda por ela, tamén cando xa rematou a da segunda tab, que se pecha despois, e 10 s despois de que remate o programa da primeira tab, o fío escritor da sesión remata os programas que quedan nela ([ADR 0011](../adr/0011-fin-e-peche-das-sesions.md)); entón meiga remata co código 0, menos de 17 s despois de pecharse a xanela, e non deixa nada das súas tabs ([ADR 0026](../adr/0026-o-peche-das-sesions-ao-sair.md)). Para esta última comprobación, arranca meiga outra vez nun *job* da comprobación, que remata o que quede nel se a comprobación falla, co script de *Probe* como programa da súa liña de comandos, co parámetro `-Closer`: cando se pecha a súa consola, o script arranca nela un cmd que corre ping. Abre a segunda tab con Ctrl+Maiús+T, pecha a xanela con `WM_CLOSE` e, polas horas nas que remataron os procesos, comproba que meiga remata polo menos 9,9 s despois ca o programa da primeira tab; despois, que no *job* non queda ningún proceso. Usa o portapapeis, coma `check-mouse.ps1`, e ao rematar devólvelle o texto que tiña; e move o punteiro real fóra da xanela, se está enriba, e ao rematar devólveo a onde estaba.
- `check-manytabs.ps1`, desde a subfase 2.8, comproba que 30 tabs seguen sendo manexábeis ([ADR 0025](../adr/0025-moitas-tabs-e-os-indicadores.md); ver [Código › ferramentas](ferramentas.md#toolscheck-manytabsps1)). Arranca meiga cun `APPDATA` propio, cun `config.toml` con dous perfís, *Light*, o primeiro do menú, que abre Ctrl+Maiús+1 e corre cmd cun indicador de comandos que só oculta o cursor, para que as súas tabs custen pouco, e *Probe*, o perfil por defecto, un script de PowerShell que escribe o texto que lle dá a comprobación e remata cando ela llo di, cun código de saída; e que oculta as shells que atopa meiga. Dálle á xanela unha área cliente de 800 × 400 píxeles a escala 100 %, ou o mesmo escalado, onde na barra caben sete tabs, e oito xa non. Calcula a barra coma meiga ([Código › meiga-core › `ui/tabbar.rs`](meiga-core.md#cratesmeiga-coresrcuitabbarrs)) e publica os clics, a roda e as teclas co fío da xanela suspendido, coma `check-profiles.ps1`; sabe que tab se mostra pola prioridade dos fíos lectores, coma `check-tabs.ps1`, e le no rexistro dos fotogramas o que debuxa a xanela. Comproba que, con sete tabs, se encollen por igual e caben, co + despois delas, e o punteiro na última, no + e na primeira debuxa só esa parte; que a oitava xa non cabe: as tabs quedan co ancho mínimo, e a franxa, entre os botóns de desprazamento, mostra a nova, que se mostra, co + e o ▾ á dereita da barra; que, con 30 tabs, cada unha corre o seu programa, nada se solapa, e a franxa mostra a última, que remata antes do botón da dereita, que xa non pode desprazala: o punteiro enriba del, e un clic nel, non debuxan nada; que a roda na barra despraza unha tab por paso, cara á primeira adiante ou á esquerda, e cara á última atrás ou á dereita, e garda para o seguinte xiro o que sobra dun paso; que un clic nun botón de desprazamento despraza a franxa ata que a primeira tab que oculta por ese lado, aínda que sexa en parte, queda enteira á vista; que, mentres o botón esquerdo segue premido nun deles, co punteiro enriba, a franxa despraza outra tab aos 250 ms e despois cada 100 ms; que cada unha das 30 tabs se mostra cun clic no seu título, despois de que a roda a traia á vista; que, cando o × ou o botón do medio pecha unha tab, as outras quedan co seu ancho e no seu sitio mentres o punteiro segue na barra, co seguinte × debaixo del, e cando o punteiro sae da barra recuperan o seu ancho, e a franxa mostra a tab que se mostra; que unha tab que pecha o teclado, co punteiro na barra, deixa que as outras recuperen o seu ancho no momento, e despois unha que pecha o rato as fai quedar co seu; que a última tab, pechada co rato, deixa que as outras recuperen o seu ancho no momento, e o × da nova última queda debaixo do punteiro; que mostrar unha tab co teclado, tamén dando a volta á barra, despraza a franxa ata ela, que despois a roda pode deixala fóra da vista, onde queda, e que Maiús+F10 despraza a franxa ata ela e abre o seu menú debaixo; que, mentres unha tab está oculta, o seu programa escribe, e só se debuxa o título da súa tab, unha vez, e toca a campá, que o debuxa unha vez máis; que mostrar a tab quita os dous indicadores, e a campá mentres se mostra non debuxa nada; que o fin do seu programa co código 3, mentres está oculta, debuxa o seu título outra vez; e que meiga remata co código 0 ao pechar a xanela, e rematan tamén os programas das súas tabs e as súas consolas. Se non pode levar o punteiro do sistema ata o botón, coma co cliente do Escritorio remoto minimizado, comproba no seu lugar que o botón premido despraza a franxa unha vez, e non máis cando o punteiro sae del. Na CI si pode: na execución manual da rama da 2.8, a sexta comprobación contou 9 tabs nun segundo, unha ao premer, outra aos 250 ms e despois unha cada 100 ms. Move o punteiro real fóra da xanela, se está enriba, e enriba do botón da esquerda na sexta comprobación; e ao rematar lévao a onde estaba. Leva uns 50 s.
- `measure-startup.ps1` arranca meiga 4 veces e mide o tempo ata o primeiro fotograma, que desde a subfase 1.6 inclúe lanzar a shell por defecto e, desde a 2.5, buscar a configuración e atopar os perfís, para lanzar o perfil por defecto (ver [Código › medición](perf.md#toolsperfmeasure-startupps1)), e `measure-frames.ps1` mide 3 s de cada tipo de fotograma: parciais, completos e completos de texto (ver [Código › medición](perf.md#toolsperfmeasure-framesps1)). Só comproban que os scripts seguen funcionando: igual que co arnés de medición, os números dunha máquina compartida non se comparan con nada. Desde o M2, despois de pechar a xanela, `measure-startup.ps1` agarda ata 20 s, e non 10, a que remate meiga: a xanela péchase mentres arranca a shell da primeira tab, e meiga agarda pola súa sesión, que, se a shell aínda se está a conectar á súa consola, queda bloqueada ata que o seu fío escritor a remata, 10 s despois ([ADR 0026](../adr/0026-o-peche-das-sesions-ao-sair.md)).

#### Subir o artefacto

```yaml
      - uses: actions/upload-artifact@cf430e030ddbb5b0abf93d22962f4752f3646cd9 # v7.0.2
        with:
          path: dist/*.zip
          archive: false
          if-no-files-found: error
          retention-days: 30
```

- Sobe o `.zip` como **artefacto** da execución, que se pode descargar desde a web.
- `archive: false`: sóbeo tal cal. Por defecto, a action metería os arquivos noutro zip, e descargariamos un zip dentro doutro. Con esta opción só se pode subir un arquivo, e o artefacto leva o seu nome.
- `if-no-files-found: error`: se non hai ningún zip, o paso falla, en vez de mostrar só un aviso.
- `retention-days: 30`: os artefactos bórranse aos 30 días.

### Job `scenarios`

```yaml
  scenarios:
    name: Load scenarios (bash)
    if: github.event_name != 'schedule'
    runs-on: ubuntu-24.04
    timeout-minutes: 10
    steps:
      - uses: actions/checkout@3d3c42e5aac5ba805825da76410c181273ba90b1 # v7.0.1
        with:
          persist-credentials: false

      - name: Check the bash scripts with ShellCheck
        run: |
          shellcheck --version
          shellcheck -x -P SCRIPTDIR tools/perf/scenarios/*.sh

      - name: Check the load scenarios in bash
        shell: pwsh
        run: ./tools/perf/test-scenarios.ps1 -Shell bash
```

- Comproba as versións en bash dos escenarios de carga, as que se usan nas tabs de WSL. Corre en Linux porque o runner de Windows non ten WSL, e o Ubuntu do runner trae o mesmo bash 5 ca o Ubuntu de WSL.
- **ShellCheck** analiza os scripts de bash e avisa dos erros habituais: variábeis sen comiñas, comandos que fallan sen que se note, sintaxe que non funciona en todas as versións… (ver [Tecnoloxías › bash, WSL e ShellCheck](../tecnoloxias.md#20-bash-wsl-e-shellcheck)). Vén instalado no runner. `-x` segue os arquivos que se cargan con `.` (`common.sh`), e `-P SCRIPTDIR` búscaos no cartafol de cada script.
- O último paso usa PowerShell 7, que tamén vén no runner de Linux: a mesma proba que no job de Windows, pero coas versións de bash (ver [Código › escenarios de carga](escenarios.md#test-scenariosps1)).
- Coma o job de Windows, non corre nas execucións programadas.

### Job `deny`

```yaml
  deny:
    name: Dependency policy (cargo-deny)
    runs-on: ubuntu-24.04
    timeout-minutes: 15
    steps:
      - uses: actions/checkout@3d3c42e5aac5ba805825da76410c181273ba90b1 # v7.0.1
        with:
          persist-credentials: false

      - name: Install the Rust toolchain from rust-toolchain.toml
        run: rustup toolchain install

      - uses: taiki-e/install-action@3a3daafd3b7f23b8f65d9f5ba2db203fb59b79e2 # v2.90.2
        with:
          tool: cargo-deny@0.20.2

      - name: cargo deny
        run: cargo deny --locked check
```

- Corre en paralelo co job de Windows.
- Usa Linux porque o resultado non depende do sistema (`deny.toml` xa indica que se analicen as dependencias de Windows) e as máquinas Linux arrancan antes.
- Necesita Rust porque cargo-deny le as dependencias a través de `cargo metadata`.
- `cargo deny --locked check`: `--locked` vai antes de `check` porque é unha opción xeral de cargo-deny, non do subcomando `check`.
- Corre tamén nas execucións programadas. Cada vez descarga a base de datos de vulnerabilidades actualizada.

### Job `conpty`

```yaml
  conpty:
    name: ConPTY version
    # Not on pull requests: a new version of the ConPTY must not block them.
    if: github.event_name == 'schedule' || github.event_name == 'workflow_dispatch'
    runs-on: ubuntu-24.04
    timeout-minutes: 5
    steps:
      - uses: actions/checkout@3d3c42e5aac5ba805825da76410c181273ba90b1 # v7.0.1
        with:
          persist-credentials: false

      - name: Check that the pinned ConPTY is the latest stable version
        shell: pwsh
        run: ./tools/fetch-conpty.ps1 -CheckLatest
```

- Desde a subfase 1.4. Comproba en nuget.org que non hai unha versión estábel do ConPTY máis nova ca a fixada en [`fetch-conpty.ps1`](ferramentas.md#toolsfetch-conptyps1), e falla se a hai ([ADR 0011](../adr/0011-fin-e-peche-das-sesions.md)). Dependabot non pode facelo: só coñece os paquetes NuGet dos proxectos de .NET.
- `if`: só corre nas execucións programadas, os luns, e nas que se lanzan a man. Nos PR e en `main` non corre, así que unha versión nova nunca bloquea un PR, e tampouco é un dos *checks* obrigatorios da rama `main`.
- Non descarga o paquete nin necesita Rust: só le a lista de versións. Por iso corre en Linux, co PowerShell 7 que trae o *runner*.
- Cando falla, GitHub avisa por correo a quen ten o repositorio, e o erro di que versión hai e que dúas liñas do script hai que cambiar.

## `.github/dependabot.yml`

```yaml
version: 2
updates:
  - package-ecosystem: cargo
    directory: /
    schedule:
      interval: weekly
    # Wait before proposing a new release, so that a compromised or broken
    # version has time to be detected and yanked.
    cooldown:
      default-days: 7
    groups:
      cargo:
        patterns: ["*"]
    ignore:
      # The #[implement] of meiga-render needs the windows-core that windows
      # uses. A new minor or major version of windows-core alone does not
      # build: it is updated by hand, with windows. Patches still come.
      - dependency-name: windows-core
        update-types:
          - version-update:semver-minor
          - version-update:semver-major
```

- `version: 2`: a versión do formato do arquivo, que sempre é 2.
- `updates`: unha entrada por cada tipo de dependencia (*ecosistema*).
- `package-ecosystem: cargo`: os crates de Rust (`Cargo.toml` e `Cargo.lock`), buscados desde a raíz (`directory: /`).
- `schedule.interval: weekly`: busca versións novas unha vez por semana.
- `cooldown.default-days: 7`: só propón versións publicadas hai polo menos 7 días. Cando alguén publica unha versión maliciosa ou rota, adoita detectarse e retirarse nos primeiros días; así non chega a este repositorio.
- `groups`: todas as actualizacións do ecosistema (`patterns: ["*"]`) van nun único PR semanal, en vez dun PR por dependencia.
- `ignore`, desde o mantemento M1: Dependabot non propón as versións *minor* nin *major* de `windows-core`, só os seus parches. `meiga-render` declárao pola macro `#[implement]`, que ten que usar o mesmo `windows-core` ca o crate `windows` ([Tecnoloxías › COM e o crate `windows`](../tecnoloxias.md#28-com-e-o-crate-windows)), e unha versión nova del só, sen un `windows` que a use, non compila: pasou coa 0.100.0, no PR [#17](https://github.com/NewHouses/meiga/pull/17). Dependabot le as versións coma *major.minor.patch*, así que para el 0.62.2 → 0.100.0 é unha versión *minor*; para Cargo, nas versións 0.x, cambiar o segundo número xa é incompatíbel, e cambiar o terceiro, non. Cando saia un `windows` novo, o seu PR fallará ata subir a man `windows-core` á versión que use. As alertas de seguridade de `windows-core` seguen chegando: a regra só cambia os PR que abre Dependabot.

```yaml
  - package-ecosystem: github-actions
    directory: /
    schedule:
      interval: weekly
    cooldown:
      default-days: 7
    groups:
      github-actions:
        patterns: ["*"]
```

O mesmo para as actions dos workflows. Dependabot entende as actions fixadas por SHA: actualiza o SHA e tamén o comentario coa versión.

```yaml
  # The Rust version pinned in rust-toolchain.toml.
  - package-ecosystem: rust-toolchain
    directory: /
    schedule:
      interval: weekly
```

Vixía `rust-toolchain.toml` e abre un PR cando sae unha versión nova de Rust, que é cada seis semanas. Así a versión fixada non queda vella, e cada cambio de versión pasa pola CI antes de integrarse.
