# ADR 0011 · Fin e peche das sesións de ConPTY

- **Estado:** Aceptado
- **Data:** 2026-11-08
- **Relacionados:** [ADR 0004](0004-conpty-empaquetado.md), [ADR 0005](0005-fios-e-fotogramas.md)

## Contexto

- O [ADR 0005](0005-fios-e-fotogramas.md) decide dous fíos por tab coa shell en marcha: o lector, en `ReadFile` da saída do ConPTY en bloques de 64 KiB, e o escritor, que espera nunha canle a entrada que hai que enviar. O fin do proceso detéctase con `RegisterWaitForSingleObject`, e `ClosePseudoConsole` chámase fóra do fío da interface. O [ADR 0004](0004-conpty-empaquetado.md) decide usar o ConPTY de Windows Terminal empaquetado, co flag `PSEUDOCONSOLE_GLYPH_WIDTH_WCSWIDTH`, e deixa para a subfase 1.4 estudar se Dependabot pode vixiar a súa versión.
- Na subfase 1.4, antes de escribir o código, comprobouse cun prototipo, co ConPTY 1.25.260930003 e co de Windows 11, como rematan e como se pechan as sesións:
  - **Pechar.** `ClosePseudoConsole` envíalle `CTRL_CLOSE_EVENT` a cada programa conectado á consola, que remata. A saída péchase cando remata o ConPTY, despois dos programas: `ReadFile` falla entón con `ERROR_BROKEN_PIPE`. `cmd` remata co código `0xC000013A` (`STATUS_CONTROL_C_EXIT`).
  - **Esperar ao pechar.** `ConptyClosePseudoConsole` do ConPTY 1.25 só pecha os seus *handles* e volta de inmediato. O `ClosePseudoConsole` de Windows tamén, desde Windows 11 24H2; nas versións anteriores, espera ata que remata o ConPTY, e o ConPTY non pode rematar se ninguén le a súa saída.
  - **Pechar mentres o programa arranca.** Se a consola se pecha nos primeiros milisegundos despois de `CreateProcessW`, mentres o programa aínda se está a conectar a ela, o ConPTY e o programa poden quedar esperando un polo outro para sempre: pasou cos dous ConPTY. Outras veces, o programa falla só, co código `0xC0000142` (`STATUS_DLL_INIT_FAILED`); Microsoft documenta que pode mostrar unha caixa de erro. Uns milisegundos máis tarde, isto xa non pasa. Rematar o programa (`TerminateProcess`) desbloquéao todo: o ConPTY remata e a saída péchase.
  - **O flag WCSWIDTH.** O ConPTY de Windows ignórao: o seu `conhost.exe` non recibe `--textMeasurement`, e a pseudoconsola créase igual.
  - **Os dous arquivos.** Se `OpenConsole.exe` non está ao lado de `conpty.dll`, a DLL arranca sen avisar o `conhost.exe` de Windows.
- A revisión do código da subfase atopou que o bloqueo ao pechar mentres un programa arranca non lle pasa só ao programa da sesión: tamén a un programa que este acaba de arrancar e que se está a conectar á mesma consola, como a shell detrás dun lanzador (`cmd /c cmd /k`) ou un programa lanzado con `start /b`. Entón o programa da sesión remata, pero o ConPTY segue esperando polo outro, e a saída non se pecha nunca. Comprobouse: pasa ás veces, e moito máis coa máquina ocupada, por exemplo con varias sesións arrancando á vez.
- Windows dálle a un programa de consola o tempo de `SPI_GETHUNGAPPTIMEOUT`, 5 s, para atender `CTRL_CLOSE_EVENT`, e despois remátao el. Comprobouse cos dous ConPTY: un programa que tarda 30 s en atendelo remata 5 s despois de pechar a pseudoconsola, co código `0xC000013A`. Ademais, Windows envíalles o evento aos programas da consola dun en dun, do máis novo ao máis vello, e espera por cada un: con dous programas así, a sesión remata aos 10 s, e con tres, aos 15 s. Atopouno a segunda revisión do código.
- Un *job* é un obxecto de Windows que agrupa procesos. Un proceso pode crearse xa nun *job* (`PROC_THREAD_ATTRIBUTE_JOB_LIST`, desde Windows 10), e os procesos que crea entran nel tamén, salvo que pidan saír (`CREATE_BREAKAWAY_FROM_JOB`) e o *job* o permita (`JOB_OBJECT_LIMIT_BREAKAWAY_OK`). `TerminateJobObject` remata todos os seus procesos. Desde Windows 8 os *jobs* poden aniñarse, así que funciona aínda que meiga xa corra noutro *job*. Comprobouse tamén cun *job* con restricións de interface, que Microsoft exclúe do aniñamento: os programas arrancan e o *job* da sesión remátaos igual.
- O Windows Terminal 1.25 chama a `ConptyReleasePseudoConsole` despois de arrancar a shell. Desde entón, o ConPTY remata só cando se desconecta o último programa da consola, e a tab pecha cando se pecha a saída. Esa función só está en `kernel32` desde Windows 11 24H2, e `windows-sys` 0.61 non a ten.
- Dependabot só coñece os paquetes NuGet a través de proxectos de .NET (`packages.config`, `.csproj`…), e non sabería actualizar o SHA-256 que comproba `tools/fetch-conpty.ps1`.

## Decisión

1. **Dous fíos por sesión**, como di o [ADR 0005](0005-fios-e-fotogramas.md):
   - o **lector** fai `ReadFile` de 64 KiB nun búfer reutilizado e pasa cada bloque ao código que usa a sesión;
   - o **escritor** espera con `WaitForMultipleObjects`, á vez, un evento (hai entrada que enviar, ou a sesión se pechou) e o proceso do programa.
2. **Sen `RegisterWaitForSingleObject`.** O escritor xa está a durmir: esperar tamén polo proceso non custa nada, e non fai falta ningunha función que Windows chame desde o seu grupo de fíos.
3. **A entrada** vai nun `Vec` protexido por un `Mutex`, en vez da canle: `write` engade os bytes e esperta o escritor só se estaba baleiro; o escritor intercambia ese `Vec` co seu, sen reservar memoria nova.
4. **O escritor pecha a pseudoconsola** cando o programa remata ou cando se pecha a sesión. Nunca o fío da interface: pechar a sesión non espera nada. Tampouco o lector, que ten que seguir lendo para que o `ClosePseudoConsole` de Windows anterior a 24H2 poida rematar. Se non se pode crear o escritor, `spawn` remata primeiro os programas da sesión e pecha a saída, e despois a pseudoconsola: nunca a pecha mentres un programa se conecta a ela.
5. **A sesión remata cando remata o programa**, non cando se desconecta o último proceso da consola. Os procesos que sigan conectados reciben `CTRL_CLOSE_EVENT` cando o escritor pecha a pseudoconsola.
6. **Unha sesión pechada sempre remata.** Cada sesión ten un *job*, e o programa créase nel (`PROC_THREAD_ATTRIBUTE_JOB_LIST`): os programas que arranque entran tamén. Despois de pechar a pseudoconsola, o escritor espera a que se peche a saída e remate o programa. Mentres tanto, os programas que tiña a sesión ao pechala teñen que ir rematando, porque Windows lles pide que rematen dun en dun e dálle 5 s a cada un: o escritor apúntaos nese momento, coa lista de procesos do *job* (`JobObjectBasicProcessIdList`) e un *handle* de cada un. Se pasan 10 s, o dobre, sen que remate ningún deles, a sesión está bloqueada, e o escritor remata todos os programas do *job* (`TerminateJobObject`), co código 1; o ConPTY remata entón, e a saída péchase. Os programas que arrancan despois non contan: un programa de limpeza que lanza outro ao pecharse non pode agochar que rematan os demais, nin un programa que arranca outros sen parar pode adiar o final.
7. **O *job* non obriga a nada máis.** Os programas poden saír del se o piden (`JOB_OBJECT_LIMIT_BREAKAWAY_OK`): sen iso, os que o piden non arrancarían. E pechar o *job* non remata os seus programas (sen `JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE`): un programa que non depende da consola, como un editor gráfico aberto desde a tab, segue vivo despois de pechala.
8. **O lector avisa do fin despois do último byte**, co código de saída do programa, e avisa ao escritor de que se pechou a saída. Se, cando se pecha a saída, o programa aínda non rematou, espérao ata 11 s: se a sesión se pechou, o escritor remátao cando pasan 10 s sen que remate ningún dos programas que tiña.
9. **Tubaxes anónimas e síncronas** (`CreatePipe`), a da saída cun búfer de 64 KiB. meiga pecha as súas copias dos extremos do ConPTY en canto crea a pseudoconsola. O programa arranca sen herdar ningún *handle* e sen *handles* estándar propios (`STARTF_USESTDHANDLES` con valores nulos), así que usa os da pseudoconsola.
10. **O ConPTY empaquetado** só se usa se están os dous arquivos ao lado de `meiga.exe`. `conpty.dll` cárgase coa ruta completa, buscando as súas dependencias só no seu cartafol e en System32. Se falta algún dos dous, úsase o de Windows. O flag WCSWIDTH só se lle pasa ao empaquetado.
11. **Vixilancia da versión:** un *job* da CI, cada luns e cando se lanza a man, que falla se nuget.org ten unha versión estábel do ConPTY máis nova ca a fixada (`tools/fetch-conpty.ps1 -CheckLatest`). Non é obrigatorio, así que nunca bloquea un PR. Isto resolve o que deixou aberto o [ADR 0004](0004-conpty-empaquetado.md).

Isto concreta o [ADR 0005](0005-fios-e-fotogramas.md) (como espera o escritor e quen detecta o fin do proceso) e o [ADR 0004](0004-conpty-empaquetado.md) (o flag só no ConPTY empaquetado e a vixilancia da versión).

## Alternativas consideradas

| Alternativa | Por que se descarta |
|---|---|
| `RegisterWaitForSingleObject`, como dicía o [ADR 0005](0005-fios-e-fotogramas.md) | Unha función que Windows chama desde o seu grupo de fíos, e que tería que avisar ao escritor, para algo que o escritor pode esperar sen custo. |
| Rematar cando se desconecta o último proceso, como o Windows Terminal 1.25 | Un programa en segundo plano mantería a tab aberta despois de saír da shell. Ademais, sen o ConPTY empaquetado e antes de Windows 11 24H2 faría falta outra vía. |
| Unha soa tubaxe con nome, dúplex e solapada, como o Windows Terminal | Máis código (`OVERLAPPED`, eventos e cancelacións) para aforrar unha tubaxe. Cos fíos síncronos, que xa dormen mentres non hai nada, non se gaña nada. Microsoft recomenda tubaxes síncronas para o ConPTY. |
| Pechar a pseudoconsola no fío da interface | Antes de Windows 11 24H2 pode esperar ata que remata o ConPTY, coa xanela bloqueada. |
| Pechar a pseudoconsola no lector | Antes de Windows 11 24H2, bloqueo: `ClosePseudoConsole` espera polo ConPTY, e o ConPTY por alguén que lea a súa saída. |
| Non pechar a pseudoconsola ata uns milisegundos despois de arrancar o programa | Non garante nada se a máquina está ocupada, nin serve para os programas que arranca a shell. O *job* cobre todos os casos. |
| Rematar só o programa da sesión (`TerminateProcess`) se non rematou 5 s despois de pechar, como na primeira versión | A revisión do código atopou que non abonda: se o que se está a conectar é un programa que arrancou o da sesión, o da sesión remata, e o ConPTY segue esperando polo outro. Comprobouse cunha proba. |
| Cancelar a lectura (`CancelSynchronousIo`) para que o lector avise do fin | O ConPTY e o programa bloqueado seguirían vivos para sempre. |
| Rematar o anfitrión do ConPTY (`OpenConsole.exe` ou `conhost.exe`) | O ConPTY non dá o seu proceso, e habería que buscalo entre os fillos de meiga. O programa bloqueado tamén seguiría vivo. |
| Un *job* con `JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE` | Pechar a tab remataría tamén os programas que deben seguir, como un editor gráfico aberto desde ela. Como pecha meiga as súas sesións ao saír decídese na subfase 1.6. |
| Un tempo fixo para todo o peche, 10 s, como na segunda versión | Windows pecha os programas dun en dun, 5 s cada un: con dous programas lentos, un peche que non está bloqueado leva máis de 10 s, e o escritor remataría a shell e todos os programas do *job*, tamén os gráficos. Atopouno a segunda revisión do código, e comprobouse. Un tempo moito máis longo tampouco o garantiría. |
| Contar os programas vivos do *job* (`ActiveProcesses`), e esperar mentres baixen | Atopouno a terceira revisión do código, e comprobouse: os programas que arrancan durante o peche, como un programa de limpeza que lanza outro ao pecharse, ou o `conhost.exe` dun programa coa súa propia consola, compensan os que rematan, e un peche que avanza parece bloqueado. |
| Contar todos os programas do *job* que rematan | Un programa que arranca outros sen parar, como un editor que lanza `git`, faría que sempre rematase algún, e adiaría o final para sempre. |
| Esperar só 5 s por cada programa | Windows remata cada un aos 5 s, e un pouco despois pasa ao seguinte: co dobre, un peche que avanza nunca parece bloqueado. |
| Dependabot | Precisa un proxecto de .NET falso no repositorio, e os seus PR fallarían, porque non actualizaría o SHA-256. |

## Consecuencias

**Positivas:**

- Unha sesión en repouso non esperta: os seus dous fíos están bloqueados.
- Pechar unha sesión nunca bloquea, en ningunha versión de Windows.
- Unha sesión pechada sempre remata, aínda que se peche no primeiro milisegundo, e aínda que o que se estea a conectar sexa un programa que arrancou a shell.
- O mesmo modelo serve para o ConPTY empaquetado e para o de Windows.
- Os cambios de versión do ConPTY avísanse sós.

**Negativas e riscos:**

- Antes de Windows 11 24H2, co ConPTY de Windows, unha sesión pechada nos seus primeiros milisegundos pode deixar o escritor bloqueado en `ClosePseudoConsole`, sen chegar a rematar o *job*: quedarían dous fíos e dous procesos esperando. Mitigación: só pasa sen o ConPTY empaquetado, que vai sempre no paquete de meiga, e nunha xanela de tempo moi curta.
- Unha sesión bloqueada tarda polo menos 10 s en rematar: ata entón seguen vivos os seus dous fíos, o ConPTY e os seus programas. Mitigación: só pasa se se pecha nos primeiros milisegundos dun programa, e a tab xa non se ve.
- Un programa que arranca durante o peche conectado á mesma consola mantén aberta a sesión, pero non conta entre os que rematan: se segue vivo 10 s despois de rematar o último dos programas que tiña a sesión, o escritor remátao co resto do *job*. Mitigación: tivo polo menos eses 10 s, e un programa que arranca na súa propia consola, mesmo sen xanela, non mantén aberta a sesión e segue vivo.
- Cando unha sesión se bloquea, o escritor remata tamén os programas do *job* que non dependen da consola, como un editor gráfico aberto desde a tab. Mitigación: só nese caso, que é raro; ao pechar unha sesión que non se bloquea, seguen vivos.
- Un programa que saia do *job* e se bloquee ao conectarse á consola, ou un ConPTY bloqueado por outro motivo, poderían deixar a sesión sen rematar. Mitigación: o primeiro ten que coincidir co peche nos primeiros milisegundos dese programa; o segundo non se viu nunca.
- Se meiga sae coas sesións abertas, os seus fíos rematan con el, sen a espera do escritor: unha sesión que estea a arrancar podería deixar o programa e o ConPTY esperando. Mitigación: decidir na subfase 1.6, cando meiga abra a súa primeira shell, como pecha as súas sesións ao saír.
- Se o ConPTY deixase de ler a entrada, `WriteFile` bloquearía o escritor, e con el o peche. Mitigación: o ConPTY le a entrada sempre; só pasaría cun ConPTY bloqueado.
- Unha tarefa máis na CI, cada semana. Mitigación: corre en Linux, nuns segundos, e non descarga o paquete.

## Referencias

- Microsoft Learn:
  - [`ClosePseudoConsole`](https://learn.microsoft.com/en-us/windows/console/closepseudoconsole): envía `CTRL_CLOSE_EVENT`; desde Windows 11 24H2 volta de inmediato, e antes espera indefinidamente se ninguén le a saída;
  - [Creating a Pseudoconsole session](https://learn.microsoft.com/en-us/windows/console/creating-a-pseudoconsole-session): tubaxes síncronas, as copias dos *handles* que hai que pechar, e o erro `0xc0000142` ao pechar mentres o programa arranca;
  - [`HandlerRoutine`](https://learn.microsoft.com/en-us/windows/console/handlerroutine), táboa *Timeouts*: `CTRL_CLOSE_EVENT`, `SPI_GETHUNGAPPTIMEOUT`, 5000 ms;
  - [Job Objects](https://learn.microsoft.com/en-us/windows/win32/procthread/job-objects): os procesos que crea un proceso dun *job* entran nel, `JOB_OBJECT_LIMIT_BREAKAWAY_OK` é un límite estendido, e `TerminateJobObject` remata todos;
  - [`JOBOBJECT_BASIC_PROCESS_ID_LIST`](https://learn.microsoft.com/en-us/windows/win32/api/winnt/ns-winnt-jobobject_basic_process_id_list): a lista dos procesos dun *job*;
  - [Nested Jobs](https://learn.microsoft.com/en-us/windows/win32/procthread/nested-jobs): os *jobs* aníñanse desde Windows 8;
  - [`UpdateProcThreadAttribute`](https://learn.microsoft.com/en-us/windows/win32/api/processthreadsapi/nf-processthreadsapi-updateprocthreadattribute): `PROC_THREAD_ATTRIBUTE_JOB_LIST`, desde Windows 10.
- Windows Terminal v1.25.2733.0:
  - [`src/winconpty/winconpty.cpp`, liñas 395–415](https://github.com/microsoft/terminal/blob/v1.25.2733.0/src/winconpty/winconpty.cpp#L395-L415): pechar só pecha os *handles* do ConPTY, sen esperar;
  - [`src/winconpty/winconpty.cpp`, liñas 558–577](https://github.com/microsoft/terminal/blob/v1.25.2733.0/src/winconpty/winconpty.cpp#L558-L577): `ConptyReleasePseudoConsole`;
  - [`src/cascadia/TerminalConnection/ConptyConnection.cpp`, liña 451](https://github.com/microsoft/terminal/blob/v1.25.2733.0/src/cascadia/TerminalConnection/ConptyConnection.cpp#L451) e [liñas 537–548](https://github.com/microsoft/terminal/blob/v1.25.2733.0/src/cascadia/TerminalConnection/ConptyConnection.cpp#L537-L548): o Windows Terminal libera a referencia e pecha a tab cando se desconecta o último programa.
- [Tecnoloxías › ConPTY](../tecnoloxias.md#31-conpty-a-pseudoconsola-de-windows), [Tecnoloxías › Procesos, tubaxes e eventos](../tecnoloxias.md#32-procesos-tubaxes-e-eventos) e [Código › meiga-win](../codigo/meiga-win.md#cratesmeiga-winsrcconptyrs).
