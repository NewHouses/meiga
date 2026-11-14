# Mantemento M1 · ConPTY, Rust e Dependabot

> **Entrega:** [PR #19](https://github.com/NewHouses/meiga/pull/19), despois da subfase 1.10: o ConPTY 1.25.261109001, Rust 1.100.0, unha regra de Dependabot para `windows-core` e a documentación. O código de meiga non cambia. As medicións fixéronse en local, e os seus números non se publican, porque describen unha máquina concreta: eiquí só se comparan.

## Obxectivo

Poñer ao día, antes de empezar coas tabs, as dependencias que avisaron mentres se revisaba a 1.10. O job *ConPTY version* da CI atopou unha versión nova do ConPTY, a 1.25.261109001. E Dependabot propuxo dous cambios: Rust 1.100.0, no PR [#16](https://github.com/NewHouses/meiga/pull/16), sen actualizar a documentación, que cita a versión; e `windows-core` 0.100.0, no PR [#17](https://github.com/NewHouses/meiga/pull/17), que non compila.

- **Feito cando:** meiga compila e pasa todas as comprobacións co ConPTY e o Rust novos, sen gastar máis, e Dependabot xa non propón un `windows-core` que non compila.

## Que se fixo

| Arquivo | Para que | Explicación |
|---|---|---|
| `tools/fetch-conpty.ps1` | O ConPTY 1.25.261109001, o do Windows Terminal 1.25.3131.0: a versión e o SHA-256 do paquete. | [Código › ferramentas](../codigo/ferramentas.md#toolsfetch-conptyps1) |
| `rust-toolchain.toml` | Rust 1.100.0. | [Código › workspace](../codigo/workspace.md#rust-toolchaintoml) |
| `.github/dependabot.yml` | Unha regra `ignore`: Dependabot non propón as versións *minor* nin *major* de `windows-core`, só os seus parches. | [Código › .github](../codigo/github.md#githubdependabotyml) |
| `docs/tecnoloxias.md` | As versións; a resposta DA1 e os desprazamentos no ConPTY novo; por que `windows-core` sobe con `windows`; e a versión 0.100 de windows-rs. | [Tecnoloxías › ConPTY](../tecnoloxias.md#31-conpty-a-pseudoconsola-de-windows) |
| `docs/codigo/` | As versións, e a regra de `windows-core`. | [Código](../codigo/README.md) |
| `docs/subfases/README.md` e `docs/README.md` | A sección de mantemento, con esta fila. | [Subfases › Mantemento](README.md#mantemento) |

## O que trae o ConPTY novo

A 1.25.261109001 é a do Windows Terminal 1.25.3131.0, publicada o 11 de novembro ([notas da versión](https://github.com/microsoft/terminal/releases/tag/v1.25.3131.0)), e a anterior, a 1.25.260930003, era a do 1.25.2733.0. Entre as dúas hai 17 cambios ([a comparación](https://github.com/microsoft/terminal/compare/v1.25.2733.0...v1.25.3131.0)). Cinco están na consola, que vai no `OpenConsole.exe` do paquete:

- **Os desprazamentos de `ScrollConsoleScreenBuffer`.** Cando un programa da consola move unha rexión da pantalla, coma vi ao desprazar o texto ou PowerShell con `$Host.UI.RawUI.ScrollBufferContents`, o ConPTY ten que contarllo ao terminal. meiga non fai operacións rectangulares (á pregunta DA1 responde coma un VT102), así que o ConPTY le a rexión e escríbea outra vez no seu sitio. Ata agora, ese camiño escribía as filas desaliñadas se o destino quedaba en parte fóra da rexión permitida: ao subir unha rexión unha fila, as filas quedaban onde estaban e a última perdíase, na consola e na pantalla de meiga. A versión nova arránxao ([microsoft/terminal#20773](https://github.com/microsoft/terminal/issues/20773)), usa a resposta a DA1, que antes ignoraba ([microsoft/terminal#20772](https://github.com/microsoft/terminal/issues/20772)), e, cando o programa despraza a pantalla enteira, envía unhas poucas secuencias para borrar ou inserir liñas, en vez de escribila enteira ([Tecnoloxías › ConPTY](../tecnoloxias.md#31-conpty-a-pseudoconsola-de-windows)). Compróbase no paso 7.
- **Un bloqueo.** Un programa que escribía sen secuencias VT e co axuste de liña desactivado (sen `ENABLE_VIRTUAL_TERMINAL_PROCESSING` nin `ENABLE_WRAP_AT_EOL_OUTPUT`) un carácter de dúas columnas na última columna quedaba bloqueado para sempre en `WriteConsoleW`; a consola ocupaba un núcleo enteiro, e os outros programas da mesma consola tamén quedaban bloqueados ([microsoft/terminal#20693](https://github.com/microsoft/terminal/issues/20693), arranxado en [#20862](https://github.com/microsoft/terminal/pull/20862)). Non está nas notas da versión.
- **O modo LNM** (`ESC [ 20 h`), no que cada salto de liña volta tamén ao principio da liña: se un programa o desactivaba (`ESC [ 20 l`) xusto despois de usalo, o ConPTY podía seguir engadindo un retorno de carro aos saltos de liña seguintes, que voltaban ao principio da liña sen que o programa o pedise ([microsoft/terminal#20863](https://github.com/microsoft/terminal/pull/20863)).
- **A xanela emerxente de cmd que pide o número dun comando do historial** (F9): se o que se escribía nela non era un número, a consola non remataba ben a lectura da liña, e cmd podía comportarse de forma inesperada ([microsoft/terminal#20779](https://github.com/microsoft/terminal/pull/20779)).
- **O borrado da pantalla** (`cls` en cmd e `Clear-Host` en PowerShell): agora a consola tamén o fai cunha secuencia de borrado cando non hai ConPTY, máis rápido e sen reservar a memoria de todo o seu búfer ([microsoft/terminal#19957](https://github.com/microsoft/terminal/pull/19957)). Co ConPTY xa o facía así, e meiga non nota diferenza.

Outros tres están na biblioteca das secuencias de control que a consola comparte co Windows Terminal. Os títulos de máis de 1 000 caracteres xa non cambian o título da consola, coma no xterm ([microsoft/terminal#20839](https://github.com/microsoft/terminal/pull/20839)), pero o ConPTY pásalle igual a secuencia ao terminal, e meiga recibe o título enteiro. E hai dous arranxos do protocolo de teclado de Kitty, para os programas que o piden ([#20768](https://github.com/microsoft/terminal/pull/20768) e [#20769](https://github.com/microsoft/terminal/pull/20769)). Os outros nove son do Windows Terminal (a súa interface, a súa configuración e o seu teclado) e da súa compilación.

## Como se mediu

Na máquina da liña base (subfase 0.8), coa compilación da 1.10 (Rust 1.99.1 e o ConPTY 1.25.260930003) fronte á deste mantemento (Rust 1.100.0 e o ConPTY 1.25.261109001), alternando e maximizadas:

- **Os escenarios:** o repouso, só co M1, comparado co da porta da 1.10; a saída masiva, catro veces (1.10, M1, M1, 1.10); e o *spinner*, dúas (M1 e 1.10), con `run-scenario.ps1 -Start -Maximized`. E o arranque, con `measure-startup.ps1`, catro veces (1.10, M1, 1.10, M1).
- **Os desprazamentos:** un script de PowerShell que move unha rexión con `ScrollBufferContents`, no exemplo `conpty` de `meiga-win`, cos dous ConPTY: o exemplo grava o que escribe o ConPTY, e o exemplo `replay` de `meiga-core` dálle a gravación ao terminal de meiga e mostra a súa pantalla. O script garda tamén o que ten a súa consola. Despois, o mesmo script en meiga, coas dúas compilacións, cunha captura da xanela.
- **O título longo:** co mesmo exemplo e os dous ConPTY, un programa que pon un título de 1 001 caracteres e despois outro de 1 000: o que lle chega ao terminal, e o título que ten a consola despois de cada un.

## Decisións e alternativas descartadas

| Decisión | Alternativa descartada | Motivo |
|---|---|---|
| Un PR de mantemento cos tres cambios, despois da 1.10 e antes da 2.1 (decidido contigo) | Integrar os PR de Dependabot, e actualizar o ConPTY noutro | Os tres cambian versións que cita a documentación, e cada un pasaría polas mesmas comprobacións. |
| Actualizar o ConPTY agora (decidido contigo) | Esperar á seguinte versión | Arranxa os desprazamentos, que se ven mal en meiga. |
| Rust 1.100.0 eiquí, e pechar o PR #16 (decidido contigo) | Integrar o PR de Dependabot | O PR #16 deixaba a documentación coa versión anterior. |
| Pechar o PR #17, e unha regra en `dependabot.yml` para `windows-core` (decidido contigo) | Pechar cada PR coma este cando chegue | A macro `#[implement]` necesita o `windows-core` que usa `windows`: unha versión nova del só non compila, e Dependabot propoñeríaa outra vez. |
| A regra ignora as versións *minor* e *major* de `windows-core` | Ignorar todas as súas versións | Os parches si valen: Cargo usa o mesmo para `windows` e para `meiga-render`. Dependabot conta a 0.100.0 coma unha *minor* da 0.62.2, aínda que en Cargo as dúas son incompatíbeis ([Código › .github](../codigo/github.md#githubdependabotyml)). |
| Seguir con `windows` 0.62.2 e `windows-sys` 0.61.2 | Pasar xa á 0.100 | Aínda non saíron. Cando saian, cambiarán moito, e adaptar meiga será un PR propio ([Tecnoloxías › windows-sys](../tecnoloxias.md#21-windows-sys-as-apis-de-windows-desde-rust)). |
| Manter `rust-version = "1.99"`, a versión mínima de Rust | Subila á 1.100 | O código segue compilando coa 1.99.1, coa que se fixo a compilación da 1.10 desta comparación, e o resolvedor 3 escolle as dependencias compatíbeis con ela ([Código › workspace](../codigo/workspace.md#cargotoml-raíz)). |
| Non cambiar o ADR 0004 nin o documento da 1.4, que citan a versión anterior do ConPTY | Actualizalos | Contan o que se decidiu e o que se fixo entón: un ADR aceptado non se reescribe, e as subfases son o diario. A versión de agora está no script e na documentación do código. |

## Como comprobar que todo está ben

> **Importante: usa PowerShell 7.** Todos os comandos son para PowerShell 7 (`pwsh`), no cartafol do repositorio. Se abres *Windows PowerShell* (a versión 5.1, que trae Windows), os scripts non se executan: só mostran o comando para facelo con `pwsh`. Para saber cal tes aberto, executa `$PSVersionTable.PSVersion`: o primeiro número ten que ser 7.

### No navegador

1. **A CI pasa.** No [PR #19](https://github.com/NewHouses/meiga/pull/19), os tres *checks* en verde: **Build and test (Windows)**, **Load scenarios (bash)** e **Dependency policy (cargo-deny)**.

   **Resultado esperado**, en **Build and test (Windows)**:

   - *Install the Rust toolchain from rust-toolchain.toml*: `rustc 1.100.0 (0ef1bc386 2026-11-09)` e `cargo 1.100.0 (3acc79c0c 2026-11-05)`.
   - *Download the ConPTY for the tests and the package*:

     ```text
     Microsoft.Windows.Console.ConPTY 1.25.261109001: downloaded from nuget.org, 1763943 bytes, with the expected SHA-256.
     conpty.dll: version 1.25.2611.09001, 119608 bytes, signed by Microsoft Corporation.
     OpenConsole.exe: version 1.25.2611.09001, 1089336 bytes, signed by Microsoft Corporation.
     conpty.dll and OpenConsole.exe are in D:\a\meiga\meiga\target\conpty
     ```

   - *Test*: as mesmas seis liñas `test result: ok.` con probas ca na 1.10: `23 passed`, `112 passed`, `5 passed`, `35 passed`, `23 passed` e `9 passed`.

### Na túa máquina

2. **Descarga a rama do PR:**

   ```powershell
   git fetch origin
   git switch m1-mantemento
   ```

3. **Rust 1.100.0:**

   ```powershell
   rustc --version
   ```

   **Resultado esperado:** `rustc 1.100.0 (0ef1bc386 2026-11-09)`. Se non a tes, rustup descárgaa primeiro ([Tecnoloxías › rustup](../tecnoloxias.md#2-rustup-e-rust-toolchaintoml)).

4. **O ConPTY novo:**

   ```powershell
   .\tools\fetch-conpty.ps1
   ```

   **Resultado esperado:** a primeira vez, descarga o paquete e comproba o seu SHA-256, coma na CI (paso 1). As seguintes, só comproba o que xa ten:

   ```text
   Microsoft.Windows.Console.ConPTY 1.25.261109001: the package is already in …\target\conpty.
   conpty.dll: version 1.25.2611.09001, 119608 bytes, signed by Microsoft Corporation.
   OpenConsole.exe: version 1.25.2611.09001, 1089336 bytes, signed by Microsoft Corporation.
   conpty.dll and OpenConsole.exe are in …\target\conpty
   ```

5. **Pasa as comprobacións e compila:**

   ```powershell
   cargo fmt --all --check
   cargo clippy --workspace --all-targets -- -D warnings
   cargo test --workspace
   cargo deny check
   .\tools\package.ps1
   .\tools\check-imports.ps1
   .\tools\check-docs.ps1
   .\tools\check-powershell.ps1
   (Get-Item .\target\release\meiga.exe).Length
   ```

   **Resultado esperado:** ningún erro, e ningún aviso novo de Clippy co Rust novo.

   - `cargo test` mostra as mesmas nove liñas `test result: ok.` ca na 1.10: 207 probas.
   - `check-imports.ps1`, as mesmas DLL ca na 1.10, e `OK: no dependencies on the Visual C++ runtime.`
   - `check-docs.ps1`: `OK: 2149 relative links checked in 51 files.`, con máis ligazóns ca na 1.10, as deste documento.
   - `check-powershell.ps1`: `OK: the 22 scripts ask for PowerShell 7, and Windows PowerShell 5.1 shows how to run them.`
   - O tamaño de `meiga.exe`, `566272` (na 1.10, `557568`): o compilador novo.

6. **As comprobacións da xanela, os fotogramas, o teclado, o rato e o tamaño, co ConPTY novo.** Mentres corren, non premas o rato nin o teclado:

   ```powershell
   .\tools\check-window.ps1
   .\tools\check-present.ps1
   .\tools\check-keyboard.ps1
   .\tools\check-mouse.ps1
   .\tools\check-resize.ps1
   ```

   **Resultado esperado:** as mesmas liñas `OK:` ca na 1.10, sen ningún erro.

7. **Os desprazamentos.** Abre meiga:

   ```powershell
   (Start-Process .\target\release\meiga.exe -Wait -PassThru).ExitCode
   ```

   Na xanela, pega esta liña e preme Intro. Borra a pantalla, escribe dez filas e sobe unha fila as filas 3 a 7, coma vi ao desprazar unha rexión:

   ```powershell
   Clear-Host; 0..9 | % { 'row{0} abcdefghij' -f $_ }; $r = [System.Management.Automation.Host.Rectangle]::new(0, 3, 30, 7); $Host.UI.RawUI.ScrollBufferContents($r, [System.Management.Automation.Host.Coordinates]::new(0, 2), $r, [System.Management.Automation.Host.BufferCell]::new('.', 'Gray', 'Black', 'Complete'))
   ```

   **Resultado esperado:** `row3` desapareceu, e as filas 4 a 7 subiron unha; debaixo de `row7`, unha fila de 31 puntos, o recheo:

   ```text
   row0 abcdefghij
   row1 abcdefghij
   row2 abcdefghij
   row4 abcdefghij
   row5 abcdefghij
   row6 abcdefghij
   row7 abcdefghij
   ...............................
   row8 abcdefghij
   row9 abcdefghij
   ```

   Coa versión anterior do ConPTY, a da 1.10, as filas 3 a 6 quedan onde estaban, e `row7` desaparece baixo os puntos. Pecha a xanela con Alt+F4: PowerShell mostra `0`.

8. **(Opcional) Mide meiga.** Cada escenario leva uns 3 minutos: non toques nada mentres corren, e pecha os programas que gasten CPU.

   ```powershell
   .\tools\perf\run-scenario.ps1 idle -Start .\target\release\meiga.exe -Maximized
   .\tools\perf\run-scenario.ps1 spinner -Start .\target\release\meiga.exe -Maximized
   .\tools\perf\run-scenario.ps1 bigfile -Start .\target\release\meiga.exe -Maximized
   .\tools\perf\measure-startup.ps1
   ```

   **Resultado esperado:** os mesmos orzamentos ca na 1.10. Na fila `terminal`, en repouso, `cpu_pct` **0,00** e `ctx_per_s` **0 ou moi preto**, e na fila `console`, o OpenConsole, tamén **0,00**; co *spinner*, `cpu_pct` **por debaixo de 1**; coa saída masiva, `Printing took … s on average` **por debaixo de 1,13 s** e `cpu_s` **por debaixo de 2,0**; e `measure-startup.ps1`, a mediana por debaixo de 150 ms. Se algunha medición sae en amarelo, `Not stable`, a máquina estaba ocupada: repítea.

9. **Os resultados non chegan a Git:**

   ```powershell
   git status --short
   ```

   **Resultado esperado:** ningunha liña. Os CSV das medicións están en `.perf\`, que Git ignora.

10. **(Despois de integralo) Dependabot.** Abre https://github.com/NewHouses/meiga/network/updates e, no ecosistema `cargo`, a última revisión.

    **Resultado esperado:** sen erros na configuración, e sen ningún PR novo para `windows-core`.

## Resultados

| Comprobación | Resultado |
|---|---|
| fmt, clippy, test e deny | Pasan con Rust 1.100.0, sen avisos novos: as mesmas 207 probas. |
| `meiga.exe` | 566 272 bytes (na 1.10, 557 568): co mesmo código, o compilador novo xera un 1,75 % máis de código (`.text`, 7 456 bytes) e algo máis de datos (`.rdata`, 1 040). Importa as mesmas DLL. |
| O ConPTY | `conpty.dll` ten o mesmo tamaño, e `OpenConsole.exe`, 40 bytes menos. Ao arrancar envía o mesmo ca a versión anterior. As probas de integración de `meiga-win` e as comprobacións da xanela, os fotogramas, o teclado, o rato e o tamaño pasan igual. |
| Os desprazamentos | Co ConPTY anterior, ao subir unha fila as filas 3 a 7, coa rexión permitida nas mesmas filas, as filas quedaron onde estaban e a última perdeuse, na súa consola e na pantalla de meiga; co novo, subiron. |
| O título longo | Cos dous ConPTY, os dous títulos chegan enteiros ao terminal. O novo xa non lle pon á súa consola o de 1 001 caracteres: segue co título de antes. |
| O repouso | Coma na 1.10: 0 % de CPU, e nin meiga nin o seu OpenConsole espertan máis ca antes. |
| A saída masiva | Igual: o tempo de imprimir e a CPU de meiga e do seu OpenConsole, cada un dentro dun 4 % entre as dúas versións. A máquina estaba ocupada, e tres das catro medicións saíron «non estábeis». |
| O *spinner* | Igual: meiga e o seu OpenConsole espertan as mesmas veces, e os seus ciclos por segundo cambian menos dun 4 %. |
| O arranque | Non empeora: nas dúas roldas, a mediana do M1 saíu un 8–10 % máis baixa ca a da 1.10. Non se investigou se vén do compilador ou do ConPTY. |
| Revisión do código | Unha revisión independente comprobou a regra de Dependabot, o SHA-256 e o tamaño do paquete, as versións, os tamaños, o número de probas, os pasos da CI e os desprazamentos do paso 7, cos dous ConPTY. Atopou dous erros na sección «O que trae o ConPTY novo», xa corrixidos: o arranxo do borrado da pantalla só cambia a consola cando non hai ConPTY, e entre os outros arranxos hai tres máis na consola, que a primeira versión desta sección atribuía á interface do Windows Terminal. |
| CI | PENDENTE |

## Pendente

- **`windows` e `windows-sys` 0.100:** cando saian, un PR propio para adaptar meiga a eles, ou xerar as funcións que usa con `windows-bindgen` ([Tecnoloxías › windows-sys](../tecnoloxias.md#21-windows-sys-as-apis-de-windows-desde-rust)).
- **Os PR #16 e #17 de Dependabot:** péchanse ao integrar este.

## Seguinte paso

Fase 2 · Tabs e perfís. Subfase 2.1 · Modelo de tabs: a lóxica das tabs en `meiga-core`, sen interface. A xanela, as tabs en orde e a activa; abrir, pechar, mover e activar; e o foco separado da tab activa, para a futura vista dividida.
