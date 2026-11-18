# Mantemento M2 · O peche das sesións ao saír

> **Entrega:** [PR #28](https://github.com/NewHouses/meiga/pull/28), despois da subfase 2.8: meiga agarda, antes de saír, a que rematen as sesións das súas tabs; o escritor de cada sesión ve rematar cada un dos seus programas, e co ConPTY de Windows pecha a pseudoconsola nun fío propio; e as comprobacións agardan a que se calmen as shells antes de pechar as súas tabs. As medicións fixéronse en local, e os seus números non se publican, porque describen unha máquina concreta: eiquí só se comparan.

## Obxectivo

Investigar e arranxar o que quedaba pendente desde a 1.4: que lles pasa ás sesións pechadas cando meiga sae. O [ADR 0011](../adr/0011-fin-e-peche-das-sesions.md) promete que unha sesión pechada sempre remata, e quen o asegura é o seu fío escritor; pero deixou para a 1.6 que pasa se meiga sae antes, e nin a 1.6 nin as subfases seguintes o decidiron. Viuse na CI ao integrar a 2.7: despois de pecharse a xanela, a consola dunha tab de `check-profiles.ps1` seguía viva, e ao final do job quedaron dous programas orfos. Decidiuse contigo investigalo despois da 2.8.

- **Feito cando:** ao pechar a xanela, meiga non deixa ningún programa nin ningún ConPTY das súas tabs, tamén cando unha sesión se bloquea, e a CI compróbao.
- **Medición:** o mantemento non ten orzamento propio, pero non pode romper os da fase. Non engade ningún fío, temporizador nin *polling* co ConPTY empaquetado: o escritor de cada sesión xa existía, e agora agarda máis obxectos na mesma chamada; co de Windows, só un fío curto en cada peche. Unha saída normal non se nota, e o repouso, a memoria e o arranque quedan coma na 2.8.

## O que pasaba ao saír

- **Os escritores morrían con meiga.** Ao pecharse a xanela caían as sesións das tabs, e cada unha pechábase sen agardar ([ADR 0011](../adr/0011-fin-e-peche-das-sesions.md), decisión 4). Despois voltaba `main`, e o proceso saía. Ao saír un proceso, Windows remata os seus outros fíos sen deixalos acabar: os escritores morrían no medio da súa espera, e ninguén podía agardar por eles, porque `Conpty::spawn` soltaba o `JoinHandle` do escritor.
- **As sesións que se bloquean.** Unha sesión bloqueada é a dun programa que se estaba a conectar á consola cando esta se pechou, coma un programa que a shell acaba de arrancar: o ConPTY e ese programa quedan esperando un polo outro ata que o escritor remata o *job* da sesión. Sen o escritor, quedaban así para sempre. Cunha sonda que abría sesións, as pechaba mentres arrancaban os seus programas e saía no momento, arredor da metade das sesións quedaron bloqueadas para sempre, co seu ConPTY e os seus programas; agardando uns segundos antes de saír, ningunha. Cun programa que, ao pecharse a súa consola, arranca outro nela, bloqueáronse todas. As sesións que se pechan normalmente non dependen do escritor: remataron todas, agardase ou non a sonda.
- **Na CI.** No primeiro intento da execución do *push* da 2.7 a `main`, `check-profiles.ps1` pechou a tab de Git Bash cando o seu programa, o lanzador `bash.exe`, levaba un segundo sen usar o procesador, mentres o bash de MSYS2 que lanzara aínda arrancaba programas, coma o `git.exe` do *prompt*. A sesión bloqueouse, e meiga saíu uns segundos despois, antes de que o escritor a rematase. O segundo intento pasou.
- **Cando o vía o escritor.** O escritor só miraba se rematara algún dos programas da sesión cando pasaban 10 s. Así, unha sesión bloqueada remataba 10 ou 20 s despois de pecharse, segundo rematase ou non algún dos seus programas nos primeiros 10 s; por iso a última proba de integración de `meiga-win` levaba na CI unhas veces 10 s e outras 20.
- **O Windows Terminal**, polo seu código, non remata ningún programa ao pechar unha sesión, nin usa *jobs*: unha sesión bloqueada quedaría así tamén nel. Non se comprobou.
- **O peche do ConPTY de Windows antes de Windows 11 24H2** agarda sen límite a que remate o ConPTY, e nunha sesión bloqueada o ConPTY non remata ata que o escritor remata os seus programas, despois de pechar: o escritor quedaría bloqueado no peche para sempre. O [ADR 0011](../adr/0011-fin-e-peche-das-sesions.md) xa o tiña nos seus riscos, e a revisión independente do código atopouno outra vez: agardando polos escritores, meiga non sairía nunca.

## Que se fixo

| Arquivo | Para que | Explicación |
|---|---|---|
| `crates/meiga-win/src/conpty.rs` | Cada `Pty` garda o `JoinHandle` do seu fío escritor e, cando se solta, pásao a unha lista, `CLOSING`, que só garda os escritores que non remataron (`keep_closing`). `wait_for_closed_sessions` agarda por todos os da lista. O escritor agarda á vez a saída, o programa e ata 62 dos programas que tiña a sesión ao pechala (`WATCHED`): cada un que remata dálle á sesión outros 10 s desde ese momento, e a sesión que pasa 10 s sen que remate ningún está bloqueada. Co ConPTY de Windows, o escritor pecha a pseudoconsola nun fío propio, que ninguén agarda (`close_console`), e unha proba unitaria compróbao. | [Código › meiga-win](../codigo/meiga-win.md#cratesmeiga-winsrcconptyrs) |
| `crates/meiga/src/main.rs` | Antes de devolver o código de saída, despois de `run` e tamén despois da caixa que mostra un erro, agarda polas sesións pechadas (`conpty::wait_for_closed_sessions`), xa sen xanela. | [Código › meiga](../codigo/meiga.md#cratesmeigasrcmainrs) |
| `crates/meiga-win/tests/conpty.rs` | Dúas probas novas: o escritor non remata unha sesión cuxos programas seguen rematando, un cada 4 s; e un proceso que pecha sesións, unha delas bloqueada, agarda por elas antes de saír, e non deixa nada. Os seus programas son o propio binario das probas, que fai o papel que lle di unha variábel de contorno (a proba `helper`, que só corre cando a lanza outra). | [Código › meiga-win](../codigo/meiga-win.md#cratesmeiga-wintestsconptyrs) |
| `crates/meiga-win/examples/conpty.rs` | O exemplo tamén agarda polas sesións pechadas antes de saír co código do programa. | [Código › meiga-win](../codigo/meiga-win.md#cratesmeiga-winexamplesconptyrs) |
| `tools/check-exit.ps1` | Un paso novo, o 14: ao pecharse a xanela, o programa da primeira tab arranca outro na súa consola, e a sesión bloquéase. meiga, que corre nun *job* da comprobación, ten que rematar 10 s despois dese programa, co código 0, e sen deixar nada das súas tabs. E, co ConPTY de Windows, o paso 6 le o historial unha fila máis abaixo: esa consola borra a pantalla ao arrancar, e `alacritty_terminal` mete entón unha fila en branco no historial de cada tab. Desde a 2.7, con ese ConPTY, a comprobación fallaba eiquí, e nunca chegaba ao paso 14. | [Código › ferramentas](../codigo/ferramentas.md#toolscheck-exitps1) |
| `tools/check-profiles.ps1` | Antes de pechar a tab de cada perfil, agarda a que a súa shell e os programas que arrancou leven un segundo sen usar o procesador, e sen que arranque nin remate ningún: o lanzador de Git Bash non fai nada mentres o seu bash arranca programas. E, nos pasos 7 e 8, dálle a meiga para saír o mesmo tempo ca ás shells do usuario para rematar, 30 s por defecto: se unha delas segue traballando cando se pecha a súa tab, a súa sesión pode bloquearse, e meiga agarda por ela. | [Código › ferramentas](../codigo/ferramentas.md#toolscheck-profilesps1) |
| `tools/check-tabs.ps1` | Agarda a que a shell dunha tab nova, e os programas que arranca, coma o `git` dun *prompt*, leven un segundo sen usar o procesador, e sen que arranque nin remate ningún, antes de mirar a saída da tab oculta, e antes de pechar a terceira tab: un *prompt* que chegaba tarde tomábase pola saída da tab oculta, e pechar unha tab mentres a súa shell arranca un programa pode bloquear a sesión. E, coas consolas de Windows, `conhost.exe`, o paso 5 acepta a resposta da consola, que responde ela a pregunta da tab oculta, e a comprobación avisa de que non comproba as respostas de meiga. Desde a 2.2, con ese ConPTY, a comprobación fallaba eiquí. | [Código › ferramentas](../codigo/ferramentas.md#toolscheck-tabsps1) |
| `tools/perf/measure-startup.ps1` | Agarda 10 s máis a que remate meiga despois de pedirlle que peche a xanela: nunha máquina ocupada, a xanela pode pecharse mentres a shell aínda se conecta á súa consola, e meiga agarda pola súa sesión. | [Código › perf](../codigo/perf.md#toolsperfmeasure-startupps1) |
| `tools/perf/run-scenario.ps1` | Agarda ata 15 s, e non 5, a que remate o terminal que lanzou, antes de rematalo. | [Código › escenarios](../codigo/escenarios.md#run-scenariops1) |
| `docs/adr/0026-o-peche-das-sesions-ao-sair.md` | A decisión nova: meiga agarda sen límite polas sesións que pechou antes de saír, o escritor ve rematar cada un dos seus programas, e co ConPTY de Windows pecha a pseudoconsola nun fío propio. O 0011 pasa a estar concretado por el. | [ADR 0026](../adr/0026-o-peche-das-sesions-ao-sair.md) |
| `docs/tecnoloxias.md` | Unha sección nova, o fin dun proceso e dos seus fíos; e, nas do ConPTY e dos procesos, o que pasa coas sesións ao saír e a espera do escritor, e que a consola de Windows responde ela a pregunta dun programa polos atributos do terminal. | [Tecnoloxías](../tecnoloxias.md#61-o-fin-dun-proceso-e-dos-seus-fíos) |
| `docs/arquitectura.md` | As sesións, os fíos e o estado. | [Arquitectura](../arquitectura.md#sesións-a-shell-o-conpty-e-os-dous-fíos) |
| `docs/codigo/` | A explicación de todo o código que cambia. | [Código](../codigo/README.md) |
| `docs/subfases/README.md` | A sección de mantemento, con esta fila. | [Subfases › Mantemento](README.md#mantemento) |
| `README.md` | O estado, o que fai meiga ao pechar a xanela e a comprobación nova. | — |

Sen dependencias novas.

## Como se mediu

Na máquina da liña base (subfase 0.8), coa xanela de meiga do tamaño por defecto, fronte á 2.8 compilada igual. Todas as meiga teñen un `APPDATA` propio, coma nas comprobacións, e cada compilación mídese cando ninguén usou o equipo durante 3 minutos:

- **A saída normal:** con 1 e con 5 tabs do perfil por defecto en repouso, o tempo desde que se pide pechar a xanela ata que remata o proceso, e o que queda despois, en tres tandas, dúas co M2 primeiro e unha coa 2.8 primeiro.
- **Unha sesión que se bloquea:** a primeira tab corre un programa que, ao pecharse a súa consola, arranca outro nela, coma no paso 14 de `check-exit.ps1`: canto tarda meiga en rematar, e o que queda despois, coas dúas compilacións.
- **As probas de integración do ConPTY,** dez veces seguidas con cada compilación: canto leva a que pecha sesións mentres arrancan os seus programas, que na 2.8 levaba 10 ou 20 s.
- **O repouso con 5 tabs:** `run-scenario.ps1 idle`, en dúas roldas que alternan cal vai primeiro.
- **A memoria e os *handles*:** con 1, 5 e 30 tabs en repouso, en dúas roldas que alternan cal vai primeiro.
- **O arranque:** `measure-startup.ps1`, 20 veces con cada compilación, en seis pares de roldas que alternan cal vai primeiro, os dous últimos cunha pausa de 30 s antes de cada compilación.

## Decisións e alternativas descartadas

O detalle está no [ADR 0026](../adr/0026-o-peche-das-sesions-ao-sair.md).

| Decisión | Alternativa descartada | Motivo |
|---|---|---|
| Antes de saír, meiga agarda a que rematen as sesións que pechou (decidido contigo) | Rematar os *jobs* das sesións ao saír; ou un *job* con `JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE`, que os remata cando se pecha o último *handle* del | Rematarían tamén os programas que deben seguir cando se pecha a súa tab, coma un editor gráfico aberto desde ela, e os que atenden `CTRL_CLOSE_EVENT`, coma unha shell que garda o seu historial, non chegarían a facelo. |
| Sen límite de tempo (decidido contigo) | Un límite, por exemplo 30 s | A espera acaba: Windows remata cada programa aos 5 s, e o escritor remata a sesión que non avanza. Só non acabaría se o escritor quedase bloqueado, nos dous casos que di o ADR (decisión 2), que non se viron nunca. Cortala deixaría outra vez sen rematar unha sesión que se bloquease despois. |
| O escritor ve rematar cada programa, e os 10 s contan desde o último que rematou (decidido contigo) | Seguir mirando só cando pasan 10 s, coma ata a 2.8; ou mirar cada pouco | Unha sesión bloqueada levaría ata 20 s, e meiga agardaría ese tempo sen xanela. Mirar cada pouco espertaría o escritor sen parar mentres se pecha a sesión. |
| Só os escritores | Agardar tamén polos lectores | Un lector agarda a que se peche a saída, e cun ConPTY que non a pecha nunca, coma o dun programa que saíu do *job*, meiga non sairía nunca. O escritor xa asegura o fin. |
| Unha lista dos escritores ao lado das sesións, que só garda os que non remataron | Agardar ao soltar cada `Pty`; ou que quen pecha unha sesión garde o seu escritor | A xanela quedaría bloqueada mentres remata cada sesión; e un `Drop` non pode devolver nada, e cada sitio que pecha sesións tería que gardalo. |
| Co ConPTY de Windows, a pseudoconsola péchase nun fío propio, que ninguén agarda (decidido contigo: arranxalo) | Só documentalo coma un risco; mirar a versión de Windows e usar o fío só antes de 24H2; ou agardar tamén por ese fío ao saír | Sen o ConPTY empaquetado, antes de Windows 11 24H2, cada sesión bloqueada deixaría meiga sen xanela para sempre. Mirar a versión sería máis código para aforrar un fío curto só cando falta o ConPTY empaquetado. E un programa que saíu do *job* pode manter vivo o ConPTY, e o peche non voltaría nunca. |
| As comprobacións agardan a que se calmen a shell dunha tab e os programas que arrancou antes de pechala | Pechar a tab cando a súa shell leva un segundo sen usar o procesador, coma ata a 2.8 | O lanzador de Git Bash non fai nada mentres o seu bash arranca programas: a sesión podía bloquearse, e agora meiga agardaría 10 s por ela. |
| `check-profiles.ps1` dálle a meiga para saír o tempo das shells do usuario, 30 s por defecto | Seguir con `TimeoutSeconds`, 10 s por defecto; ou agardar sen límite a que se calme cada shell | A comprobación só pode agardar un tempo por cada shell do usuario, e unha que segue traballando pode pecharse mentres arranca un programa: meiga agarda pola súa sesión ata 10 s. |
| `measure-startup.ps1` e `run-scenario.ps1` agardan máis a que remate meiga | Rematalo antes | Rematar meiga antes deixaría bloqueada para sempre a sesión pola que agarda. |
| Co ConPTY de Windows, o paso 6 de `check-exit.ps1` le o historial unha fila máis abaixo, e meiga deixará de meter esa fila na 8.6 (decidido contigo) | Arranxar meiga xa no M2, para que un `ESC [ 2 J` nunha pantalla baleira sen historial non meta ningunha fila; ou só corrixir a documentación, que dicía que a comprobación funciona tamén co ConPTY de Windows | A fila non ten que ver co peche das sesións, e só a mete o ConPTY de Windows, que meiga usa se non atopa o seu. E sen arranxar a comprobación, o paso 14 non correría nunca co ConPTY de Windows, o único co que o peche vai nun fío propio. |
| Coas consolas de Windows, o paso 5 de `check-tabs.ps1` acepta a resposta da consola, e a comprobación avisa de que non comproba as respostas de meiga (decidido contigo) | Só corrixir a documentación, que dicía que a comprobación funciona tamén co ConPTY de Windows, e arranxala na 8.6; ou deixala así, coma unha limitación do contorno | A consola de Windows responde ela a pregunta, e meiga non a ve: con ese ConPTY, o paso non pode comprobar meiga, pero o resto da comprobación si, coma `check-tabbar.ps1`, que avisa do seu límite e segue. Así, todas as comprobacións que din funcionar co ConPTY de Windows funcionan con el. |

## Como comprobar que todo está ben

> **Importante: usa PowerShell 7.** Todos os comandos son para PowerShell 7 (`pwsh`), no cartafol do repositorio. Se abres *Windows PowerShell* (a versión 5.1, que trae Windows), os scripts non se executan: só mostran o comando para facelo con `pwsh`. Para saber cal tes aberto, executa `$PSVersionTable.PSVersion`: o primeiro número ten que ser 7.

### No navegador

1. **A CI pasa.** No [PR #28](https://github.com/NewHouses/meiga/pull/28), os tres *checks* teñen que estar en verde: **Build and test (Windows)**, **Load scenarios (bash)** e **Dependency policy (cargo-deny)**.

   **Resultado esperado**, en **Build and test (Windows)**:

   - *Test*: sete liñas `test result: ok.` con probas: `60 passed` (o programa), `188 passed` e `5 passed` (`meiga-core` e as súas gravacións), `36 passed` (`meiga-render`), `27 passed` (`meiga-win`: na 2.8 eran 26), `14 passed; 0 failed; 1 ignored` (as probas de integración do ConPTY: na 2.8 eran 12, sen a ignorada) e `2 passed` (as dos perfís). A ignorada é `helper`, o programa que usan as dúas novas.
   - *Check the end of the programs*: a liña da grella e trece liñas `OK:`. A nova, a última, coa pantalla do *runner*:

     ```text
     OK: when its window closes, and the program of the first tab, as its console closes, starts another one in it, the session of that tab does not end; meiga waits for it, also once the session of the second tab has ended: 10 s after the program of the first tab ended, its session ends the programs left in it, and meiga ends with exit code 0, … s after its window closed, and leaves nothing of its tabs.
     ```

   - Os outros pasos, coma na 2.8: as mesmas DLL e as mesmas liñas `OK:`.

2. **(Opcional) A decisión.** Abre o [ADR 0026](../adr/0026-o-peche-das-sesions-ao-sair.md).

   **Resultado esperado:** o contexto (o que deixou pendente o ADR 0011, o que pasaba ao saír, as sesións nas que se notaba, a CI, cando o vía o escritor, o peche do ConPTY de Windows antes de Windows 11 24H2, o Windows Terminal e o que fan Windows e Rust), seis decisións, catro delas marcadas «decidido contigo», as alternativas e as consecuencias. A primeira di que meiga agarda polas sesións que pechou; a segunda, que sen límite de tempo; a terceira, que o escritor ve rematar cada programa; e a sexta, que co ConPTY de Windows a pseudoconsola se pecha nun fío propio. No [índice dos ADR](../adr/README.md) hai vinte e seis. O 0011 está concretado polos 0024 e 0026, e no gráfico o 0026 vén do 0005, do 0011, do 0019 e do 0024.

### Na túa máquina

3. **Descarga a rama do PR:**

   ```powershell
   git fetch origin
   git switch m2-peche
   ```

   **Resultado esperado:** `Switched to a new branch 'm2-peche'` e `branch 'm2-peche' set up to track 'origin/m2-peche'.`

4. **Pasa as comprobacións e compila.** Se borraches `target\`, executa antes `.\tools\fetch-conpty.ps1`: as probas de `meiga-win` e o paquete necesitan o ConPTY.

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

   **Resultado esperado:** ningún erro, e ningún aviso de Clippy.

   - `cargo test` mostra dez liñas `test result: ok.`, as sete do paso 1 e tres con `0 passed`, as da documentación. As probas de integración do ConPTY levan uns 13 s, e non uns 10 coma na 2.8: as dúas novas pechan sesións que tardan máis de 10 s en rematar.
   - `cargo deny check` mostra en amarelo o aviso de sempre, `warning[duplicate]: found 2 duplicate entries for crate 'windows-sys'`, e remata con `advisories ok, bans ok, licenses ok, sources ok`.
   - `package.ps1` remata con `Wrote …\dist\meiga-0.1.0-x86_64-pc-windows-msvc.zip`.
   - `check-imports.ps1`, as mesmas DLL ca na 2.8, e `OK: no dependencies on the Visual C++ runtime.`
   - `check-docs.ps1`: `OK: 5179 relative links checked in 69 files.`, con máis ligazóns ca na 2.8, as do ADR 0026, deste documento e da documentación do código que cambia.
   - `check-powershell.ps1`: `OK: the 29 scripts ask for PowerShell 7, and Windows PowerShell 5.1 shows how to run them.`, coma na 2.8.
   - O tamaño, `840704` (na 2.8, `836096`): a espera polas sesións pechadas, a espera do escritor por cada programa e o peche da pseudoconsola nun fío propio.

5. **As probas novas:**

   ```powershell
   cargo test -p meiga-win --test conpty -- keep_ending were_closed
   cargo test -p meiga-win --lib -- may_wait
   ```

   **Resultado esperado:** primeiro, as dúas probas novas de integración, en calquera orde, e `test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 13 filtered out`, despois duns 12 s: cada unha pecha unha sesión que tarda máis de 10 s en rematar.

   ```text
   test the_sessions_that_were_closed_end_before_the_process_exits ... ok
   test a_session_whose_programs_keep_ending_is_not_ended_by_the_writer ... ok
   ```

   Despois, a unitaria do peche nun fío propio, `test conpty::tests::a_pseudoconsole_that_may_wait_closes_on_a_thread_of_its_own ... ok`, con `test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 26 filtered out`.

6. **Unha saída normal non se nota.** Abre meiga:

   ```powershell
   $meiga = Start-Process .\target\release\meiga.exe -PassThru
   ```

   Na xanela, preme catro veces Ctrl+Maiús+T, e agarda a que as cinco tabs mostren o seu *prompt*. Despois, nesta consola, pecha a xanela e mide canto tarda meiga en rematar:

   ```powershell
   $clock = [Diagnostics.Stopwatch]::StartNew(); $null = $meiga.CloseMainWindow(); $meiga.WaitForExit(); $clock.ElapsedMilliseconds; $meiga.ExitCode
   ```

   **Resultado esperado:** PowerShell mostra os milisegundos, unha fracción de segundo, e `0`, o código de saída. meiga sae xusto despois de rematar as súas sesións, e non deixa nada: as consolas das tabs e as súas shells xa remataron. Coa 2.8 saía un pouco antes, pero as consolas e as shells seguían vivas un pouco despois del. Se pechas a xanela antes de que todas as tabs mostren o seu *prompt*, meiga pode tardar uns 10 s: unha shell que aínda se conecta á súa consola pode bloquear a sesión, e meiga agarda por ela.

7. **(Opcional) Unha sesión que se bloquea.** Crea un script que, cando se pecha a súa consola, arranca outro programa nela, coma unha shell que arranca o `git` do seu *prompt*, e abre meiga con el na primeira tab:

   ```powershell
   $closer = Join-Path $env:TEMP 'meiga-closer.ps1'
   @'
   Add-Type -TypeDefinition @"
   using System.Diagnostics;
   using System.Runtime.InteropServices;
   using System.Threading;

   public static class Closer
   {
       delegate bool Handler(uint signal);

       [DllImport("kernel32.dll")]
       static extern bool SetConsoleCtrlHandler(Handler handler, bool add);

       static readonly Handler handler = OnSignal;

       public static void Start() { SetConsoleCtrlHandler(handler, true); }

       static bool OnSignal(uint signal)
       {
           // CTRL_CLOSE_EVENT: start another program in the console that closes.
           if (signal != 2) return false;
           Process.Start(new ProcessStartInfo("cmd.exe", "/d /c ping -n 30 127.0.0.1 >nul") { UseShellExecute = false });
           Thread.Sleep(500);
           return true;
       }
   }
   "@
   [Closer]::Start()
   '@ | Set-Content $closer
   $meiga = Start-Process .\target\release\meiga.exe -ArgumentList 'pwsh', '-NoLogo', '-NoProfile', '-NoExit', '-File', $closer -PassThru
   ```

   Agarda a que a tab mostre o seu *prompt*. Despois, nesta consola, pecha a xanela e mide canto tarda meiga en rematar; mira tamén se queda algún `ping`, e borra o script:

   ```powershell
   $clock = [Diagnostics.Stopwatch]::StartNew(); $null = $meiga.CloseMainWindow(); $meiga.WaitForExit(); $clock.Elapsed.TotalSeconds; $meiga.ExitCode
   Get-Process PING -ErrorAction Ignore
   Remove-Item $closer
   ```

   **Resultado esperado:** PowerShell mostra os segundos, algo máis de 10, e `0`, o código de saída; `Get-Process PING` non mostra nada. PowerShell remata medio segundo despois de pecharse a súa consola, e o `ping` que arrancou queda na sesión, que se bloquea: o escritor agarda 10 s desde que remata PowerShell e, como non remata ningún outro programa, remata os que quedan, entre eles o `ping`. meiga agarda por esa sesión antes de saír. Coa 2.8, meiga remataba no momento, e o `ping`, o seu `cmd` e a consola da tab seguían vivos ata que o `ping` remataba só, uns 30 s despois.

8. **Comproba o fin dos programas automaticamente.** Mentres corre, non premas o rato nin o teclado:

   ```powershell
   .\tools\check-exit.ps1
   ```

   **Resultado esperado:** a liña da grella, `The grid has … x … cells of … x … pixels.`, e trece liñas `OK:`: as doce da 2.8 e a nova, a do paso 1. Na nova, meiga remata algo máis de 10 s despois de pecharse a súa xanela: os 10 s que agarda a sesión da primeira tab desde que remata o seu programa, antes de rematar o que quedou nela.

9. **As comprobacións anteriores seguen pasando.** Mentres corren, non premas o rato nin o teclado:

   ```powershell
   .\tools\check-window.ps1
   .\tools\check-present.ps1
   .\tools\check-keyboard.ps1
   .\tools\check-mouse.ps1
   .\tools\check-resize.ps1
   .\tools\check-tabs.ps1
   .\tools\check-tabbar.ps1
   .\tools\check-menu.ps1
   .\tools\check-profiles.ps1
   .\tools\check-shortcuts.ps1
   .\tools\check-manytabs.ps1
   ```

   **Resultado esperado:** as mesmas liñas ca na 2.8. Liñas `OK:`, por orde: seis, dez, dúas (despois da que di a distribución do teclado), dezaoito, nove, oito, once (despois da que di o alto da barra e o ancho das tabs), dez, oito, dez e trece (despois da que di o alto da barra e o ancho das tabs).

10. **Mide o repouso con 5 tabs: a porta de rendemento.** Durante uns 4 minutos, sen tocar nada despois de abrir as tabs:

    ```powershell
    .\tools\perf\run-scenario.ps1 idle -Start .\target\release\meiga.exe -Delay 30
    ```

    Cando apareza `Measuring starts in 30 s.`, preme catro veces Ctrl+Maiús+T na xanela de meiga, e despois aparta o punteiro da xanela e non toques nada.

    **Resultado esperado:** tres liñas `Window N of 3: …` e, na fila `terminal` da táboa:

    - `cpu_pct` **0,00**;
    - `ctx_per_s`, os espertares por segundo, **0** ou moi preto. O mantemento non engade ningún fío nin temporizador en repouso, e os fíos de meiga non espertan. Na primeira medición poden contar algúns fíos que Windows crea no proceso ao arrancar, e que rematan sós;
    - `private_mb_max` por debaixo de **40**, o orzamento;
    - `threads_max`, once ou algúns máis: un fío para a xanela e dous por cada tab, máis os que crea Windows ao arrancar.

    Ao final, o script pecha meiga, que remata despois das súas sesións.

11. **Os resultados non chegan a Git:**

    ```powershell
    git status --short
    ```

    **Resultado esperado:** ningunha liña. Os CSV das medicións están en `.perf\`, que Git ignora.

## Resultados

| Comprobación | Resultado |
|---|---|
| fmt, clippy, test e deny | Pasan. 332 probas (na 2.8, 329): as dúas novas de integración do ConPTY e a unitaria do peche nun fío propio. E unha ignorada, `helper`, o programa que usan as novas de integración, que só corre cando o lanza unha delas. |
| «Feito cando» | Cumprido. Ao pechar a xanela, meiga non deixa nada das súas tabs: nin cunha nin con cinco tabs en repouso, nin cunha sesión bloqueada, que remata 10 s despois do seu último programa. Na CI compróbao o paso 14 de `check-exit.ps1`, co ConPTY empaquetado, e en local, tamén co de Windows. |
| A saída normal | Con 1 e con 5 tabs en repouso, en tres tandas: o M2 sae xusto despois de rematar as súas sesións, e non deixa nada. A 2.8 saía un pouco antes, pero deixaba vivas as consolas e as shells das tabs, que remataban despois del: o M2 remata máis ou menos cando remataban elas coa 2.8, nunhas tandas un pouco antes e noutras un pouco despois. Non se nota. |
| Unha sesión que se bloquea | Coa primeira tab coma no paso 7, en dúas tandas que alternan cal vai primeiro: o M2 remata algo máis de 10 s despois de pecharse a xanela, os 10 s que agarda a sesión desde que remata a shell, co código 0, e non deixa nada. A 2.8 remataba no momento, pero a consola da tab, o `cmd` e o `ping` seguían vivos ata que o `ping` remataba só, uns 30 s despois; cun programa que non remata só, quedarían para sempre. Na CI, o paso 14 de `check-exit.ps1` dá o mesmo: meiga remata 10,6 s despois de pecharse a xanela. |
| As probas de integración do ConPTY | A que pecha sesións mentres arrancan os programas dos seus lanzadores, dez veces con cada compilación, alternando: arredor de 10 s coas dúas, e nunca 20. Nesta máquina, tampouco a 2.8 chegou aos 20 s, que na súa CI saíron en dúas das seis execucións da rama e do PR (20,25 e 20,10 s). Todas xuntas levan agora uns 13 s, porque as dúas novas pechan sesións que tardan máis de 10 s en rematar: na CI, 12,2 s. |
| Repouso con 5 tabs | Igual ca na 2.8, en dúas roldas que alternan cal vai primeiro: o terminal non usa o procesador, case non esperta, e ten a mesma memoria, por debaixo do orzamento, 40 MB, e os mesmos fíos: un da xanela e dous por tab, máis os que crea Windows ao arrancar. O mantemento non engade ningún fío nin temporizador en repouso. |
| A memoria e os *handles* | Con 1, 5 e 30 tabs en repouso, en dúas roldas: a mesma memoria e os mesmos fíos ca na 2.8, e un *handle* máis por tab, o do fío escritor da súa sesión: cada `Pty` garda agora o seu `JoinHandle`, para agardar por el ao saír, e a 2.8 soltábao, e pechaba o *handle* do fío. |
| O arranque | Igual ca na 2.8: nas roldas nas que a máquina estaba tranquila, as dúas arrancan no mesmo tempo. Noutras estaba ocupada, aínda que ninguén a usase, e as dúas variaron moito, unhas veces máis o M2 e outras máis a 2.8, tamén coa pausa antes de cada compilación. O mantemento non cambia nada do que fai meiga ao arrancar. |
| `meiga.exe` | 840 704 bytes (na 2.8, 836 096): uns 4,5 KB máis, a espera polas sesións pechadas, a espera do escritor por cada programa e o peche da pseudoconsola nun fío propio. As mesmas DLL ca na 2.8. |
| O ConPTY de Windows | Cunha copia de `meiga.exe` sen o ConPTY empaquetado ao lado, pasan as oito comprobacións que din funcionar con ese ConPTY: `check-window.ps1`, `check-tabs.ps1`, `check-tabbar.ps1` (co seu aviso), `check-menu.ps1`, `check-profiles.ps1`, `check-shortcuts.ps1`, `check-exit.ps1` e `check-manytabs.ps1`. Tamén o paso 14: a sesión bloquéase tamén con ese ConPTY, que se pecha nun fío propio, e meiga remata 10 s despois do programa da primeira tab, sen deixar nada. Dúas fallaban antes, non polo M2, e arranxáronse nel (decidido contigo): `check-exit.ps1`, no paso 6 desde a 2.7, pola fila en branco do principio do historial, e `check-tabs.ps1`, no paso 5 desde a 2.2, porque a consola de Windows responde ela a pregunta da tab oculta. A 2.8 falla igual nas dúas. |
| Revisión do código | Ao documentar o código atopáronse comentarios que dicían outra cousa e ocos nas comprobacións e nos scripts, xa corrixidos. **Os comentarios:** o da espera polas sesións pechadas dicía que o escritor ve rematar os programas da sesión: ve pecharse a saída e rematar o programa, ou remata os do *job* se a sesión se bloqueou, e os que non están na consola, coma unha xanela que abriu a shell, seguen se a sesión rematou soa; e o da proba das sesións pechadas non dicía que a proba conta os programas antes de rematar os que queden. **As comprobacións:** o paso 14 de `check-exit.ps1` di agora, se meiga remata en seguida e as consolas das tabs xa remataron, que con ese ConPTY a sesión non se bloqueou, e non que meiga non agardou, e solta os procesos das consolas tamén se falla; `check-profiles.ps1` dálle a meiga para saír o tempo das shells do usuario, e non o das súas propias esperas; e `check-tabs.ps1` agarda tamén polos programas que arranca a shell dunha tab nova, coma `check-profiles.ps1`. **Os scripts de medición:** `run-scenario.ps1` agarda tamén cando o terminal xa non ten xanela: meiga pecha a súa cando o programa da única tab remata co código 0, e despois agarda polas sesións. **Despois, unha revisión independente** non atopou problemas importantes. Comprobou a lista dos escritores e a súa espera, a espera do escritor por cada programa, os camiños de erro, as probas, o paso 14 de `check-exit.ps1` e os scripts. Sinalou un matiz: co ConPTY de Windows, antes de Windows 11 24H2, o peche agarda sen límite a que remate o ConPTY, e nunha sesión bloqueada o ConPTY non remata ata que o escritor remata os seus programas, despois de pechar: o escritor quedaría no peche para sempre, e meiga, agardando por el, non sairía nunca. Decidiches arranxalo: co ConPTY de Windows, o escritor pecha a pseudoconsola nun fío propio, que ninguén agarda. Antes comprobouse na documentación de Microsoft e no código do ConPTY: o da versión 1.18 tamén agardaba, e o da 1.25, o que empaqueta meiga, xa non. Unha proba unitaria nova, cun peche falso que agarda ata que a proba o deixa voltar, compróbao, e fállana dúas copias: a que pecha sempre no escritor e a que pecha sempre nun fío. E dúas copias de meiga que imitan co ConPTY empaquetado o peche de antes de 24H2: a que pecha no escritor falla o paso 14 de `check-exit.ps1`, porque meiga segue vivo pasado o límite da comprobación, e a que pecha nun fío pasa as trece comprobacións. **Despois, a revisión da corrección** non atopou problemas importantes. Comprobou cada sitio onde se pecha unha pseudoconsola, que se o fío do peche non se pode crear a pseudoconsola se pecha no escritor, o cambio de tamaño fronte ao peche, a concorrencia, a proba unitaria, os documentos e as fontes. Atopou dúas frases que dicían máis ca o código, xa corrixidas: que o peche nunca se chama no fío da xanela, cando `Conpty::spawn` pecha a pseudoconsola no fío que o chama se falla, sen ningún programa nin a saída que o fagan agardar; e que a espera sempre acaba, cando, se Windows non pode crear o fío do peche, a pseudoconsola se pecha no escritor, e co ConPTY de Windows anterior a 24H2 unha sesión bloqueada deixaríao agardando para sempre. |
| CI | PENDENTE |

## Pendente

- **Se se mata meiga,** coma desde o Xestor de tarefas, ou se falla cun *panic*, non agarda: as sesións bloqueadas quedan coma ata a 2.8. Un *job* con `JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE` cubriríao, pero remataría os programas que deben seguir cando se pecha a súa tab.
- **Co ConPTY de Windows,** se un programa que saíu do *job* mantivese vivo o ConPTY dunha sesión pechada, o fío que a pecha quedaría bloqueado ata que meiga saia, coma o lector, sen gastar CPU.
- **Máis de 62 programas:** nunha sesión que tiña máis ao pechala, os que pasan de 62 só se miran cando pasan 10 s sen que remate ningún dos outros, coma ata a 2.8.
- **Mentres meiga agarda,** o seu proceso segue vivo, sen xanela: quen agarda a que remate, coma un script, agarda tamén. Abrir outro meiga mentres tanto non ten ningún problema.
- **Co ConPTY de Windows, o historial de cada tab empeza cunha fila en branco:** esa consola borra a pantalla ao arrancar, e `alacritty_terminal`, ao borrar unha pantalla baleira sen historial, mete nel unha fila ([Código › meiga-core](../codigo/meiga-core.md#o-fin-do-programa-e-o-reinicio-restart-e-at_empty_row_start)). O polgar da barra de desprazamento vese desde o principio, e Ctrl+Maiús+Inicio leva a esa fila. Arránxase na 8.6.

## Seguinte paso

Subfase 2.9 · Tabs na barra de título: o aspecto de Chrome e do Warp. A barra de tabs pasa á barra de título da xanela, cun marco propio e os botóns de minimizar, maximizar e pechar, e a xanela segue comportándose coma calquera xanela de Windows 11: os *snap layouts*, arrastrala e un dobre clic para maximizala, tamén por Escritorio remoto.
