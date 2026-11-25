# Mantemento M3 · As sesións fóra do fío da xanela

> **Entrega:** [PR #32](https://github.com/NewHouses/meiga/pull/32), despois da subfase 2.10: o fío escritor de cada sesión crea a pseudoconsola e o programa, e a xanela mostra a tab nova no momento, sen agardar por Windows; un programa que non arranca dío un intre despois, cunha liña máis se o seu cartafol non está; e, desde agora, cada subfase leva probas de todo o que cambia, mide a súa cobertura e próbaas con mutantes, e a CI mide a cobertura das probas. As medicións fixéronse en local, e os seus números non se publican, porque describen unha máquina concreta: eiquí só se comparan.

## Obxectivo

Que a xanela non agarde a que Windows cree os procesos dunha sesión. Na porta 2.10, abrir unha tab levou de mediana arredor de tres cuartos do seu orzamento, 30 ms, pero o p95 non pasou con Windows PowerShell nin con Ubuntu: algunhas aperturas atrancáronse mentres Windows creaba o programa, que meiga lanzaba no fío da xanela ([Subfases › 2.10](2.10-porta.md)). Decidiuse contigo arranxalo nun mantemento, xusto despois da porta e antes da 3.1. E pediches tamén asegurar que o código ten as probas necesarias, e que cada subfase engade probas exhaustivas que aseguren que non hai erros: desde o M3, é unha regra do proxecto ([Tecnoloxías › A cobertura e os mutantes](../tecnoloxias.md#64-a-cobertura-e-os-mutantes)).

- **Feito cando:** abrir unha tab pasa o orzamento con todos os perfís, coa mediana e o p95 por debaixo de 30 ms, e o arranque segue por debaixo de 150 ms.
- **Medición:** abrir unha tab, con cada perfil, e o arranque, fronte á 2.10 compilada igual; e o repouso con 5 tabs, que non debe cambiar: o mantemento non engade ningún fío nin temporizador.

## O que pasaba ao abrir unha tab

- **A xanela creaba a sesión.** Ata a 2.10, `Conpty::spawn` facía no fío que o chamaba, o da xanela, todo o que necesita unha sesión: os eventos, o *job*, as tubaxes, a pseudoconsola (`ConptyCreatePseudoConsole`, que lanza `OpenConsole.exe`), o programa nela (`CreateProcessW`) e os seus dous fíos ([ADR 0011](../adr/0011-fin-e-peche-das-sesions.md)). Ao abrir, duplicar ou reiniciar unha tab, a xanela non debuxaba a tab nova ata que Windows creaba os dous procesos; e ao arrancar, a primeira sesión ía antes do primeiro fotograma ([ADR 0013](../adr/0013-da-saida-a-pantalla.md), decisión 9).
- **Crear un proceso atráncase ás veces.** Na porta 2.10, 5 das 100 aperturas levaron entre 5,9 e 7,5 veces o orzamento, sempre dentro de meiga, antes do fotograma; e 3 dos 40 arranques, case o dobre do seu. Unha sonda atopou a causa fóra de meiga: crear un proceso atráncase ás veces, case sempre o mesmo tempo, tamén sen o ConPTY. Seguramente é unha función que Windows chama cada vez que se crea un proceso, coma a dun antivirus, sen comprobar (9.1). Iso non se pode arranxar en meiga; o que si se pode é que a xanela non agarde por el ([Tecnoloxías › Crear os procesos fóra do fío da interface](../tecnoloxias.md#63-crear-os-procesos-fóra-do-fío-da-interface)).
- **A carga do ConPTY.** `Conpty::load` miraba en cada sesión se `conpty.dll` e `OpenConsole.exe` estaban ao lado de `meiga.exe`, e cargaba a biblioteca, que a primeira vez nun proceso leva algúns milisegundos: tamén antes do primeiro fotograma.
- **O Windows Terminal**, polo seu código, crea a pseudoconsola e o programa no fío da interface, coma meiga ata a 2.10; e Chromium lanza os seus procesos nun fío dedicado, e quen os pide non agarda.

## Pechar mentres o programa se conecta

Atopouse ao medir o arranque co M3: `measure-startup.ps1` pecha meiga en canto pinta, e case todos os peches tardaban os 10 s despois dos que o escritor remata unha sesión bloqueada, mentres meiga agardaba por ela antes de saír ([ADR 0026](../adr/0026-o-peche-das-sesions-ao-sair.md)). A medición levou case quince veces máis ca coa 2.10.

- **O que fai o ConPTY.** Polo seu código, o ConPTY non empeza a falar co terminal ata que se lle conecta o primeiro programa: mentres atende a súa conexión, antes de responderlle, escribe `ESC [ c`, `ESC [ ? 1004 h` e `ESC [ ? 9001 h`. Pechar a pseudoconsola, ou romper a tubaxe da entrada, envíalles `CTRL_CLOSE_EVENT` aos programas da consola, e só unha vez; se chega mentres o primeiro se conecta, ese programa non o recibe, coma di un comentario do propio código. O programa segue nunha consola que ninguén le, e a sesión bloquéase ata que o escritor a remata, 10 s despois ([ADR 0011](../adr/0011-fin-e-peche-das-sesions.md)).
- **Por que agora.** Coa 2.10, a xanela non pintaba ata crear o programa, e un peche despois case nunca chegaba nese intre. Desde que o escritor crea o programa, a xanela pinta antes, e un peche en canto pinta, ou o dunha tab que se pecha ao abrirse, pode chegar xusto mentres o programa se conecta.
- **Unha sonda** pechou sesións de PowerShell 7 co ConPTY empaquetado entre que Windows creaba o programa e que este se conectaba: bloqueouse o 74 % co código da 2.10 e o 31 % co do M3, e ningunha agardando a que se vise `ESC [ ? 9001 h`. Con cmd, Windows PowerShell, Ubuntu e Git Bash, e co ConPTY de Windows, ningunha. Despois dun tempo, a sonda deixou de bloquearse, sen saber por que; pero meiga pechado en canto pinta, co perfil por defecto, seguiu bloqueándose sempre, coa sesión desconectada: doce de doce veces co M3, ningunha de oito coa 2.10, e ningunha de doce agardando a secuencia. Ao final, coa sesión conectada, unha copia do M3 que pecha no momento, coma antes do arranxo, pechada en canto pinta, bloqueouse a metade das veces coa máquina ocupada, e ningunha das doce coa máquina tranquila: o peche só se perde nun intre curto, e cando chega depende da máquina e da sesión.
- **Coa máquina ocupada, agardar a secuencia non abondaba.** A proba nova do programa auxiliar que pecha unha sesión en canto existe o seu programa fallou unha vez de cada cinco: a sesión bloqueábase igual. O ConPTY escribe a secuencia antes de responderlle ao programa, e un peche xusto despois perdíase. Cunha segunda sonda, pechando así sesións de `cmd`: coa máquina tranquila, ningunha bloqueada; con todos os procesadores ocupados por outros fíos, o 90 % co ConPTY empaquetado e o 73 % co de Windows; agardando ademais 10 ms despois da secuencia, o 20 %, e 50 ms, ningunha de 20; e rematando os programas cando se conectan, ningunha de 45.
- **O arranxo** (decidido contigo: agardar a conexión, como moito 1 s, e despois rematar os programas): ao pecharse a sesión, apúntase se o lector xa viu `ESC [ ? 9001 h` na saída. Se a viu, o escritor pecha coma sempre. Se non, non pecha a pseudoconsola, nin solta a tubaxe da entrada, ata que o programa se conecta, ou remata, ou remata o ConPTY, como moito 1 s; e se se conectou, remata os programas da sesión, que normalmente aínda non executaron nada seu, e despois pecha a pseudoconsola ([ADR 0028](../adr/0028-as-sesions-fora-do-fio-da-xanela.md), decisión 8). Un programa que non se conecta nese segundo, coma un con xanela propia, recibe o peche coma sempre, e segue.
- **Se o lector se atrasa** (atopouno a terceira revisión do código; decidido contigo: aceptalo e dicilo): o ConPTY respóndelle ao programa sen agardar polo lector, e o escritor remata os programas cando o lector ve a conexión. Se o lector a ve tarde, coa máquina moi ocupada, co *lock* do terminal ocupado ou nunha tab oculta, coa súa prioridade máis baixa, o programa pode ter empezado a executar o seu código, e remata igual, co código 1, sen recibir o peche. Só pasa ao pechar unha tab, ou meiga, mentres arranca o seu programa, e sen o arranxo ese programa quedaría nunha consola que ninguén le, ata que o escritor rematase a sesión 10 s despois.

## Que se fixo

| Arquivo | Para que | Explicación |
|---|---|---|
| `crates/meiga-win/src/conpty.rs` | Unha función nova, `spawn`, lanza unha sesión co ConPTY que está ao lado do executábel ou, se non está, co de Windows, que carga unha vez o primeiro escritor que o necesita; `Conpty::spawn` fai o mesmo cun ConPTY concreto. As dúas fan no fío que as chama só o que non agarda polos procesos: comproban a orde, o cartafol, o bloque das variábeis e o tamaño, crean os eventos, o *job* e as tubaxes, e lanzan os dous fíos; e voltan coa sesión. O escritor crea a pseudoconsola, co último tamaño que recibiu a sesión, e o programa nela, se a sesión non se pechou antes, e despois envíalle a entrada que agardaba. Se non pode, apunta por que, e o lector dío cando se pecha a saída, cun método novo do `Output`, `failed`. Os fíos chámanse `conpty writer` e `conpty reader` ata que arranca o programa, e despois levan o seu número. O número do programa (`Pty::process_id`) é opcional. Cando se pecha a sesión antes de que o programa se conecte á pseudoconsola, o escritor agarda a que se conecte, como moito 1 s: ata que o lector ve `ESC [ ? 9001 h` nos primeiros 1024 bytes da saída, ou remata o programa ou o ConPTY; e se se conecta, remata os programas da sesión en vez de pechar a pseudoconsola. Vinte probas unitarias novas, a maioría cun ConPTY falso que agarda nunha porta mentres crea a pseudoconsola. | [Código › meiga-win](../codigo/meiga-win.md#cratesmeiga-winsrcconptyrs) |
| `crates/meiga-win/tests/conpty.rs` | Oito probas novas de integración: un programa que non existe; a entrada, o tamaño e a prioridade de antes de que arranque o programa; o número do seu proceso; o ConPTY que está ao lado do executábel; os erros dun ConPTY empaquetado sen os seus arquivos ou sen as súas funcións; e sesións que se pechan en canto empezan. O programa auxiliar das probas do M2 pecha antes dúas sesións, unha cuxo programa acaba de arrancar e outra que aínda non creou nada, agarda por elas e mira que o programa da primeira rematou. | [Código › meiga-win](../codigo/meiga-win.md#cratesmeiga-wintestsconptyrs) |
| `crates/meiga-win/examples/conpty.rs` | O exemplo mostra por que non arrancou o programa, e sae co código 2. | [Código › meiga-win](../codigo/meiga-win.md#cratesmeiga-winexamplesconptyrs) |
| `crates/meiga/src/session.rs` | `start` lanza a sesión con `conpty::spawn`, e volta en canto ten os seus fíos. Cando o programa non arranca, o lector dío (`failed`), e o terminal móstrao coma ata agora, nunha fila súa e co aviso do que se pode facer; e, cando o cartafol non está (`ERROR_DIRECTORY`), cunha liña máis, coma o Windows Terminal: `Could not access starting directory "…"`. Cinco probas novas, tres delas co lector de verdade nunha sesión de verdade. | [Código › meiga](../codigo/meiga.md#cratesmeigasrcsessionrs) |
| `crates/meiga/src/tabs.rs` | Só mostra no momento os erros dunha sesión que nin sequera puido empezar, coma os de Windows ao preparala (`not_started`). | [Código › meiga](../codigo/meiga.md#cratesmeigasrctabsrs) |
| `crates/meiga/src/main.rs` | O comentario da espera polas sesións pechadas, que inclúe as que se estaban a crear. | [Código › meiga](../codigo/meiga.md#cratesmeigasrcmainrs) |
| `tools/check-exit.ps1` | O paso 11 espera a liña do cartafol entre o erro e o aviso. O paso 9 le as prioridades percorrendo só os fíos de meiga, moitas veces por milisegundo, e falla se o lector do programa que arranca *Restart tab* nunha tab oculta segue coa prioridade normal 3 ms despois de levar o número do programa, ou se non o leva en `TimeoutSeconds`. E o paso 14 abre unha terceira tab, e pecha a xanela en canto aparece o escritor da súa sesión, mentres este crea a pseudoconsola e o programa. | [Código › ferramentas](../codigo/ferramentas.md#toolscheck-exitps1) |
| `tools/check-window.ps1` | Un paso novo: abre meiga co seu perfil por defecto e pecha a xanela en canto pinta, mentres arranca a shell da tab; meiga ten que saír co código 0 antes de 5 s, sen deixar nada. E, se falla un paso, remata meiga coa súa árbore de procesos, a shell e a consola da tab. | [Código › ferramentas](../codigo/ferramentas.md#toolscheck-windowps1) |
| `.github/workflows/ci.yml` | Un job novo, *Coverage (Windows)*, que non é obrigatorio: corre as probas con `cargo-llvm-cov`, escribe o resumo da cobertura na execución e garda o informe. | [Código › GitHub](../codigo/github.md#job-coverage) |
| `docs/adr/0028-as-sesions-fora-do-fio-da-xanela.md` | A decisión nova: o escritor crea a sesión, que mentres tanto xa garda a entrada e o tamaño; o que pasa ao pechala, tamén mentres o programa se conecta, ou se non se pode crear; e o ConPTY, unha vez. Os ADR 0005, 0011, 0013, 0024 e 0026 pasan a estar concretados por el. | [ADR 0028](../adr/0028-as-sesions-fora-do-fio-da-xanela.md) |
| `docs/tecnoloxias.md` | Dúas seccións novas: crear os procesos fóra do fío da interface, e a cobertura e os mutantes. | [Tecnoloxías](../tecnoloxias.md#63-crear-os-procesos-fóra-do-fío-da-interface) |
| `docs/arquitectura.md` | As sesións, os fíos e o estado. | [Arquitectura](../arquitectura.md#sesións-a-shell-o-conpty-e-os-dous-fíos) |
| `docs/codigo/` | A explicación de todo o código que cambia, o job novo e o que mide agora `measure-startup.ps1`. | [Código](../codigo/README.md) |
| `docs/subfases/README.md` | A fila do M3. | [Subfases › Mantemento](README.md#mantemento) |
| `README.md` | O estado, que a tab nova se mostra no momento, a liña do cartafol, o peche dunha sesión cuxo programa aínda non se conectou e o job da cobertura. | — |

Sen dependencias novas. A CI instala `cargo-llvm-cov` só no seu job.

## As probas

Pediches asegurar que o código ten as probas necesarias, e que cada subfase engade probas exhaustivas. Decidiuse contigo:

- **Desde o M3, cada subfase** leva probas de todo o que engade ou cambia, tamén dos casos raros e dos erros; e cada erro que se atopa, unha proba que falla sen o arranxo.
- **A cobertura** das liñas que cambian mídese coas probas e coas comprobacións con xanela, cunha compilación instrumentada en *release*, coma a que se distribúe ([Tecnoloxías › A cobertura e os mutantes](../tecnoloxias.md#64-a-cobertura-e-os-mutantes)).
- **Os mutantes:** `cargo mutants --in-diff` co *diff* da subfase, e copias estragadas a man para o que só ven as comprobacións.
- **O código que xa había** revísase pouco a pouco: cada subfase, os arquivos que toca.
- **Na CI,** un job non obrigatorio corre as probas coa cobertura e garda o informe.

O que deu no M3:

- **As probas:** 385 (na 2.10, 352). En `meiga-win`, vinte unitarias novas en `conpty.rs` e oito de integración; e no programa, cinco en `session.rs`, tres delas cun fío lector de verdade. As unitarias de `conpty.rs` usan un ConPTY falso que agarda nunha porta mentres crea a pseudoconsola, para facer cousas nese intre; e oito delas son as do peche antes da conexión: sete chegaron cos seus dous arranxos, e a oitava, coa terceira revisión do código. E tres pasos das comprobacións con xanela: o 11 e o 14 de `check-exit.ps1`, e un novo en `check-window.ps1`, que pecha meiga en canto pinta.
- **A cobertura:** só coas probas, as liñas de `conpty.rs` cubertas pasaron do 89,7 % na 2.10 ao 96,8 %, e as de `session.rs`, do 87,6 % ao 98,7 %; todas as do código novo. As que quedan en `conpty.rs` son erros de Windows que non se poden provocar nunha proba, coma os de crear un fío, un evento ou un *job*, ou escribir nunha tubaxe que xa pechou o ConPTY; `has_ended`, que só se usa cunha sesión de máis de 62 programas; e liñas das probas que só corren se algo falla, coma as dúas que engade a proba nova para unha máquina moi ocupada. Coas comprobacións con xanela, nunha compilación instrumentada en *release*, o 95,3 % das liñas de todo o código (só coas probas, o 79,8 %): `session.rs`, o 99,3 %; `conpty.rs`, o 96,8 %, coas mesmas liñas sen cubrir ca só coas probas; e `tabs.rs`, `app.rs` e `main.rs`, que só corren cunha xanela, o 94,8 %, o 76,3 % e o 57,6 %. Nestes dous quedan os erros que se mostran nunha caixa de mensaxe, os modos `--bench`, que mide `measure-frames.ps1`, e mover a xanela ou cambiar a súa escala, que non fai ningunha comprobación.
- **Os mutantes:** `cargo mutants --in-diff`, co *diff* do M3, fai 67 mutantes: as probas collen 56; 8 non compilan, porque devolverían un `Pty`, un `Conpty` ou un `Process` por defecto, que non existe; e 3 non os colle ningunha proba, porque só corren cunha xanela: `main`, o `wake` do `Waker` e `Tab::run`. Cóllenos as comprobacións con xanela. Ademais, trinta e dúas copias de `conpty.rs` estragadas a man, dezanove delas do peche antes da conexión: trinta e unha fallan cada unha as probas que teñen que fallar, e a copia sen cambios pásaas todas; a outra, a que mira a conexión cando o escritor chega ao peche, e non ao pechar a sesión, pásaas, e acéptase: só se diferencia se o lector ve a conexión entre os dous, e as probas non o poden provocar; e nove copias de meiga para as comprobacións con xanela: sete fallan cada unha no seu paso, entre elas a que deixa coa prioridade normal o lector dunha tab oculta que reinicia, que pasaba `check-exit.ps1` desde a 2.8; a que pecha no momento, sen agardar a que se conecte o programa, só falla `check-window.ps1` cando o peche chega mentres o programa se conecta, a metade das veces coa máquina ocupada, e coa sesión tranquila pasa; e a que agarda o segundo enteiro porque o lector non avisa da conexión pasa sempre. Esas dúas cóllenas sempre as probas unitarias do peche.

## Como se mediu

Na máquina da liña base (subfase 0.8), sen GPU e por Escritorio remoto, coa sesión desconectada, coma na porta 2.10: as medicións só corren coa sesión desconectada ou bloqueada, cada unha empeza cando a sesión leva 60 s así, e se alguén a conecta outra vez, a medición para, e repítese despois. meiga, maximizado e cun `APPDATA` propio, fronte á 2.10 compilada igual (`main` co PR #29, que só cambia a CI), con `tools\package.ps1`; as dúas, alternando cal vai primeiro:

- **Abrir unha tab:** cunha primeira tab de PowerShell 7, 20 veces con cada perfil, de Ctrl+Maiús+N, co número do perfil, ata o primeiro fotograma completo, e ata o primeiro fotograma despois del, o da primeira saída do seu programa; e despois Ctrl+Maiús+W, que pecha a tab, ata que remata o seu programa. Pasa se a mediana e o p95, a 19.ª de 20, están por debaixo de 30 ms.
- **O arranque:** `measure-startup.ps1`, en dúas roldas de 20 de cada compilación, a primeira coa 2.10 primeiro e a segunda co M3 primeiro. Pasa se a mediana está por debaixo de 150 ms.
- **O repouso con 5 tabs:** coma o repouso da liña base, con 5 tabs de PowerShell 7: a CPU, os espertares, a memoria privada e os fíos.

## Decisións e alternativas descartadas

O detalle está no [ADR 0028](../adr/0028-as-sesions-fora-do-fio-da-xanela.md).

| Decisión | Alternativa descartada | Motivo |
|---|---|---|
| O fío escritor de cada sesión crea a pseudoconsola e o programa (decidido contigo) | Un fío curto por cada sesión que se crea, que lle pase o `Pty` á xanela; un fío lanzador permanente, coma o de Chromium; ou seguir creándoas no fío da xanela, coma o Windows Terminal | O mesmo para quen usa meiga, pero sen fíos novos, e con menos código: a sesión xa recibía a entrada e o tamaño desde calquera fío, e o escritor xa pechaba a pseudoconsola e aseguraba o fin. Cun lanzador permanente, as sesións crearíanse unha detrás doutra; e no fío da xanela, cada atranco de Windows sería un atranco da xanela. |
| Mentres se crea, a sesión xa está: garda o que se escribe e o tamaño, a prioridade do lector vale, e a tab móstrase no momento, sen indicador (decidido contigo) | Descartar o que se escribe antes de que arranque o programa, coma o Windows Terminal; ou un indicador mentres se crea | Ata agora, o que se escribía agardaba na cola da xanela, e non se perdía. Un indicador sería unha animación, que o principio 1 non permite, ou un que case sempre se vería só un intre. |
| Pechar mentres se crea: se o escritor aínda non arrancou o programa, xa non o arranca; se o estaba a arrancar, ou xa o arrancou pero o programa aínda non se conectou á pseudoconsola, agarda a que se conecte, como moito 1 s, e remata os programas da sesión (decidido contigo) | Pechar no momento, coma antes, e deixar que o escritor remate a sesión se se bloquea; agardar sen límite; ver a conexión na primeira saída do ConPTY; pechar coma sempre en canto se conecta; agardar 50 ms máis; ou, cando o lector se atrasa, apuntar a conexión antes de darlle a saída ao terminal, ou levar o lector coa prioridade normal ata a conexión | O ConPTY perde un peche que chega mentres se lle conecta o primeiro programa, tamén xusto despois de escribir `ESC [ ? 9001 h`, e a sesión bloquearíase 10 s, tamén ao saír meiga: coa máquina ocupada, case sempre. Normalmente, o programa aínda non executou nada seu, e rematalo non perde nada; se o lector ve tarde a conexión, pode ter empezado, e remata igual (decidido contigo: aceptalo). Apuntala antes só quitaría o tempo do terminal, e non o do lector, e a proba que detén o lector antes de que a apunte non se podería facer; e a prioridade normal cambiaría a regra das tabs ocultas nos primeiros milisegundos de cada sesión. Sen límite, un programa que nunca se conecta, coma un con xanela propia, deixaría a sesión aberta ata que rematase el. O ConPTY empaquetado escribe `ESC [ 1 t` un intre antes de que se conecte o programa. E un tempo fixo quizais non abondase cunha máquina máis ocupada. |
| Os erros de crear a pseudoconsola ou o programa dío o lector, coma o fin dun programa, e cando o cartafol non está, cunha liña máis, coma o Windows Terminal (decidido contigo) | Que `spawn` agarde polo programa para devolver o erro; as tres liñas do Windows Terminal | Agardar volvería facer agardar a xanela por Windows. As outras dúas liñas do Windows Terminal repiten o texto de Windows, que meiga xa mostra. |
| O ConPTY que está ao lado do executábel cárgao unha vez o primeiro escritor que o necesita (decidido contigo) | Cargalo no fío da xanela, unha vez, ou en cada tab coma ata agora | A carga era o único que ía antes do primeiro fotograma; e mirar os arquivos en cada tab non serve de nada. |
| Os fíos levan o número do programa en canto arranca: o escritor dállo aos dous, e o lector póñeo el mesmo ao arrancar, se xa o hai | Un número de sesión | As comprobacións e os perfís de Windows buscan o lector polo proceso do seu programa, e o número dunha sesión non o coñece ninguén fóra de meiga. |
| Desde o M3, cada subfase leva probas de todo o que cambia, a súa cobertura, tamén coas comprobacións con xanela, e os seus mutantes; e a CI mide a cobertura das probas nun job que non é obrigatorio (decidido contigo) | Revisar agora todo o código que xa había, nun mantemento propio; ou só as probas, sen medir a cobertura nin os mutantes | O código que xa había revísase pouco a pouco, cos arquivos que toca cada subfase. A cobertura di que código non se executa, e os mutantes, que erros non collen as probas. Na CI, as comprobacións con xanela necesitarían outra compilación, e a cobertura completa queda para a medición en local. |

## Como comprobar que todo está ben

> **Importante: usa PowerShell 7.** Todos os comandos son para PowerShell 7 (`pwsh`), no cartafol do repositorio. Se abres *Windows PowerShell* (a versión 5.1, que trae Windows), os scripts non se executan: só mostran o comando para facelo con `pwsh`. Para saber cal tes aberto, executa `$PSVersionTable.PSVersion`: o primeiro número ten que ser 7.

### No navegador

1. **A CI pasa.** No [PR #32](https://github.com/NewHouses/meiga/pull/32), os tres *checks* obrigatorios teñen que estar en verde: **Build and test (Windows)**, **Load scenarios (bash)** e **Dependency policy (cargo-deny)**; e o novo, **Coverage (Windows)**, que non é obrigatorio, tamén.

   **Resultado esperado**, en **Build and test (Windows)**:

   - *Test*: sete liñas `test result: ok.` con probas: `72 passed` (o programa: na 2.10 eran 67), `197 passed` e `5 passed` (`meiga-core` e as súas gravacións), `36 passed` (`meiga-render`), `51 passed` (`meiga-win`: na 2.10 eran 31), `22 passed; 0 failed; 1 ignored` (as probas de integración do ConPTY: na 2.10 eran 14) e `2 passed` (as dos perfís).
   - *Check the end of the programs*: a liña da grella e trece liñas `OK:`, coma na 2.10. Cambian dúas: a undécima di agora que a tab de Later conta tamén que o seu cartafol non está; e a última, que meiga agarda tamén pola sesión dunha terceira tab, que estaba a arrancar cando se pechou a xanela.
   - *Check that meiga.exe opens its window, runs its program and closes cleanly*: as seis liñas `OK:` da 2.10 e unha nova, a sétima: `OK: closed as soon as it is painted, while the shell of its tab starts, it ends with exit code 0 … s later, and leaves nothing of its tab.`
   - ⏳ *Check the start-up measurement*.
   - Os outros pasos, coma na 2.10: as mesmas DLL e as mesmas liñas `OK:`.

   E en **Coverage (Windows)**: o resumo da cobertura de cada arquivo, no resumo da execución (*Summary*), e na sección *Artifacts* do final, o informe en HTML, `coverage`.

2. **(Opcional) A decisión.** Abre o [ADR 0028](../adr/0028-as-sesions-fora-do-fio-da-xanela.md).

   **Resultado esperado:** o contexto (como se creaba unha sesión, a porta 2.10, o que xa permitía a sesión, o Windows Terminal, Chromium, os nomes dos fíos, a carga do ConPTY e pechar mentres o programa se conecta), oito decisións, cinco delas marcadas «decidido contigo», as alternativas e as consecuencias. A primeira di que o escritor crea a sesión; a segunda, que mentres se crea a sesión xa está; a cuarta, os erros; a quinta, o ConPTY, unha vez; e a oitava, que un peche antes da conexión agarda a que se conecte o programa, como moito 1 s, e remata os programas da sesión: normalmente antes de que executen código seu, pero, se o lector se atrasa, poden ter empezado. Nas referencias, o código do ConPTY que o explica. No [índice dos ADR](../adr/README.md) hai vinte e oito: o 0005, o 0011, o 0013, o 0024 e o 0026 están concretados polo 0028, e no gráfico o 0028 vén do 0004, do 0005, do 0011, do 0013, do 0019, do 0024 e do 0026.

### Na túa máquina

3. **Descarga a rama do PR:**

   ```powershell
   git fetch origin
   git switch m3-sesions
   ```

   **Resultado esperado:** `Switched to a new branch 'm3-sesions'` e `branch 'm3-sesions' set up to track 'origin/m3-sesions'.`

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

   - `cargo test` mostra dez liñas `test result: ok.`, as sete do paso 1 e tres con `0 passed`, as da documentación.
   - `cargo deny check` mostra en amarelo o aviso de sempre, `warning[duplicate]: found 2 duplicate entries for crate 'windows-sys'`, e remata con `advisories ok, bans ok, licenses ok, sources ok`.
   - `package.ps1` remata con `Wrote …\dist\meiga-0.1.0-x86_64-pc-windows-msvc.zip`.
   - `check-imports.ps1`, as mesmas DLL ca na 2.10, e `OK: no dependencies on the Visual C++ runtime.`
   - `check-docs.ps1`: `OK: 6172 relative links checked in 74 files.` (na 2.10, 5851 en 72): as do ADR 0028 e do documento do mantemento, e as que suma a documentación que cambia.
   - `check-powershell.ps1`: `OK: the 30 scripts ask for PowerShell 7, and Windows PowerShell 5.1 shows how to run them.`, coma na 2.10.
   - O tamaño, `871936` (na 2.10, `864256`): a sesión que se crea no escritor, os seus erros, os nomes dos seus fíos e o peche antes da conexión.

5. **As probas novas:**

   ```powershell
   cargo test -p meiga-win --lib -- conpty::tests
   cargo test -p meiga-win --test conpty
   cargo test -p meiga --bin meiga -- session::tests
   ```

   **Resultado esperado:** todas `ok`, en calquera orde:

   - vinte e catro probas `conpty::tests::…`, con `test result: ok. 24 passed; 0 failed; 0 ignored; 0 measured; 27 filtered out`: as catro de antes e as vinte do M3, entre elas as oito do peche antes da conexión, coma `a_session_closed_before_its_program_connects_ends_it_once_it_has` e `a_session_whose_connection_is_not_seen_in_a_second_closes_as_any_other`;
   - despois duns segundos, as de integración do ConPTY, con `test result: ok. 22 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out`: as catorce de antes e as oito do M3, coma `a_program_that_is_not_there_does_not_start`, e a ignorada, `helper`, o programa que usan as do M2;
   - e dezaoito probas `session::tests::…`, con `test result: ok. 18 passed; 0 failed; 0 ignored; 0 measured; 54 filtered out`, entre elas as cinco do M3, coma `the_reader_gives_the_output_to_the_terminal_and_tells_the_end`.

6. **(Opcional) A cobertura das probas.** Se tes `cargo-llvm-cov` (`cargo install cargo-llvm-cov --version 0.9.1 --locked`, e `rustup component add llvm-tools`):

   ```powershell
   cargo llvm-cov --workspace --locked --summary-only
   ```

   **Resultado esperado:** despois de compilar e pasar as probas, a táboa da cobertura de cada arquivo e, ao final, `TOTAL`. Na columna *Lines* de `meiga-win\src\conpty.rs`, arredor do 96,8 % (na 2.10, arredor do 90 %), e na de `meiga\src\session.rs`, arredor do 98,7 % (na 2.10, do 88 %). As de `tabs.rs`, `app.rs` e `main.rs` case non se cobren: só corren cunha xanela, e próbanas as comprobacións (paso 11).

7. **A tab nova, no momento.** Abre meiga:

   ```powershell
   (Start-Process .\target\release\meiga.exe -Wait -PassThru).ExitCode
   ```

   Na xanela, preme Ctrl+Maiús+2 e, sen agardar, escribe `echo ola` e preme Intro.

   **Resultado esperado:** a tab nova, «Windows PowerShell», móstrase no momento, co terminal baleiro; uns instantes despois aparecen a cabeceira de Windows PowerShell e o seu *prompt*, que executa o que escribiches: `ola`. O que se escribe mentres arranca o programa agarda por el, coma ata agora agardaba na xanela. Despois, preme Ctrl+Maiús+T catro veces seguidas, rápido: as catro tabs aparecen unha tras outra, sen agardar, e os seus programas arrancan á vez. Pecha a xanela co botón de pechar: PowerShell mostra `0`.

8. **Pechar en canto pinta.** Abre meiga e pecha a súa xanela en canto pinta, mentres arranca a shell da súa tab, e mide canto tarda en saír:

   ```powershell
   $meiga = Start-Process .\target\release\meiga.exe -PassThru
   $null = $meiga.WaitForInputIdle(10000)
   $clock = [Diagnostics.Stopwatch]::StartNew(); $null = $meiga.CloseMainWindow(); $meiga.WaitForExit(); $clock.Elapsed.TotalSeconds; $meiga.ExitCode
   ```

   **Resultado esperado:** os segundos, moito menos dun, e `0`. A xanela aparece e péchase no momento. Sen o arranxo, PowerShell 7 non recibía o peche, e meiga tardaba algo máis de 10 s en saír.

9. **Un cartafol que non está.** Crea un `config.toml` de proba cun perfil que empeza nun cartafol que non existe, e abre meiga:

   ```powershell
   $config = "$env:APPDATA\meiga\config.toml"
   New-Item -ItemType Directory -Force (Split-Path $config) | Out-Null
   @'
   [[profiles]]
   name = "Sen cartafol"
   command = '%SystemRoot%\System32\cmd.exe'
   directory = 'C:\meiga\non existe'
   '@ | Set-Content $config
   (Start-Process .\target\release\meiga.exe -Wait -PassThru).ExitCode
   ```

   Fai clic no ▾ e escolle *Sen cartafol*.

   **Resultado esperado:** a tab «Sen cartafol» móstrase no momento, e un intre despois di por que non arrancou o seu programa, en dúas liñas, e debaixo o que se pode facer:

   ```text
   [could not start `C:\WINDOWS\System32\cmd.exe` in `C:\meiga\non existe`: CreateProcessW failed: The directory name is invalid. (os error 267)]
   Could not access starting directory "C:\meiga\non existe"
   You can now close this tab with Ctrl+D, or press Enter to restart.
   ```

   A primeira liña pode ocupar dúas filas, e o erro de Windows vai na súa lingua. Antes do título da tab, a × vermella do fin. Preme Ctrl+D: a tab péchase. Pecha a xanela co botón de pechar: PowerShell mostra `0`. Despois, borra o arquivo de proba:

   ```powershell
   Remove-Item $config
   Remove-Item (Split-Path $config)
   Test-Path (Split-Path $config)
   ```

   **Resultado esperado:** `False`.

10. **Comproba o fin dos programas automaticamente.** Mentres corre, uns 45 s, non premas o rato nin o teclado:

    ```powershell
    .\tools\check-exit.ps1
    ```

    **Resultado esperado:** a liña da grella, `The grid has … x … cells of … x … pixels.`, e trece liñas `OK:`, coma na 2.10. A undécima di agora que a tab de Later conta tamén que o seu cartafol non está:

    ```text
    OK: Ctrl+Shift+2 opens Later, whose folder is not there, and its tab tells why its program could not start, that its folder is not there, and what can be done then; Enter tries again, with one full frame, and once the folder is there, starts the program in it.
    ```

    E a última, que meiga agarda tamén pola sesión dunha terceira tab, que se abre xusto antes de pecharse a xanela: remata algo máis de 10 s despois, polos 10 s que agarda a sesión da primeira tab.

    ```text
    OK: when its window closes, right after a third tab opens, and the program of the first tab, as its console closes, starts another one in it, the session of that tab does not end; meiga waits for it, also once the sessions of the second tab and of the third one, which was starting, have ended: 10 s after the program of the first tab ended, its session ends the programs left in it, and meiga ends with exit code 0, … s after its window closed, and leaves nothing of its tabs.
    ```

11. **As comprobacións anteriores seguen pasando.** Mentres corren, uns 5 minutos, non premas o rato nin o teclado:

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
    .\tools\check-titlebar.ps1
    ```

    **Resultado esperado:** as mesmas liñas ca na 2.10, e unha máis en `check-window.ps1`, a de pecharse en canto pinta. Liñas `OK:`, por orde: sete, dez, dúas (despois da que di a distribución do teclado), dezaoito (despois da que di a grella), nove (despois da que di as celas), oito, once (despois da que di o alto da barra), dez, oito, dez, trece (despois da que di o alto da barra) e catorce (despois da que di o alto da barra).

12. **Mide o arranque.** Durante uns 15 s, meiga ábrese e péchase 22 veces: non toques nada.

    ```powershell
    .\tools\perf\measure-startup.ps1
    ```

    **Resultado esperado:** a táboa cos tempos de 20 arranques, coa mediana (`median_ms`) por debaixo de 150, o orzamento: nas medicións do mantemento, arredor dun terzo del, e coa 2.10, máis da metade. E todo leva uns 15 s, coma coa 2.10: o script pecha meiga en canto pinta, e co M3 sen o arranxo do peche, cada arranque levaba uns 10 s máis.

13. **Mide o repouso con 5 tabs: a porta de rendemento.** Durante uns 4 minutos, sen tocar nada despois de abrir as tabs:

    ```powershell
    .\tools\perf\run-scenario.ps1 idle -Start .\target\release\meiga.exe -Delay 30
    ```

    Cando apareza `Measuring starts in 30 s.`, preme catro veces Ctrl+Maiús+T na xanela de meiga, e despois aparta o punteiro da xanela e non toques nada.

    **Resultado esperado:** tres liñas `Window N of 3: …` e, na fila `terminal` da táboa:

    - `cpu_pct` **0,00**;
    - `ctx_per_s`, os espertares por segundo, **0** ou moi preto. O mantemento non engade ningún fío nin temporizador: os fíos de meiga non espertan;
    - `private_mb_max` por debaixo de **40**, o orzamento;
    - `threads_max`, once ou algúns máis: un fío para a xanela e dous por cada tab, máis os que crea Windows ao arrancar.

    Ao final, o script pecha meiga, que remata despois das súas sesións.

14. **Os resultados non chegan a Git:**

    ```powershell
    git status --short
    ```

    **Resultado esperado:** ningunha liña. Os CSV das medicións están en `.perf\`, que Git ignora.

## Resultados

| Comprobación | Resultado |
|---|---|
| fmt, clippy, test e deny | Pasan. 385 probas (na 2.10, 352): as vinte unitarias novas de `conpty.rs`, as oito de integración e as cinco de `session.rs`. E a ignorada de sempre, `helper`. |
| «Feito cando» | Cúmprese. Con todos os perfís, abrir unha tab leva unha mediana do 12–13 % do orzamento e un p95 do 13–14 %, e a máis lenta das 100 aperturas, o 15 %; e o arranque, unha mediana do 36–37 % do seu. |
| Abrir unha tab | Coa 2.10, a mediana foi do 76–79 % do orzamento con cada perfil, e tres das 100 aperturas pasaron del, unha delas case oito veces. Co M3, a mediana é unhas seis veces menor con cada perfil, e a máis lenta das 100 leva só un 15 % máis ca a mediana: a xanela xa non agarda por Windows, e os atrancos ao crear un proceso non chegan a ela. |
| A primeira saída do programa | Coma coa 2.10: a mediana de cada perfil, dentro dun 5 % da súa, salvo Command Prompt, un 14 % menor. O programa arranca no mesmo tempo e, algunha vez, coma coa 2.10, uns 200 ms máis tarde: agora iso só atrasa a súa primeira saída. |
| O arranque | Coa 2.10, a mediana foi do 55 % e do 53 % do orzamento, en dúas roldas, e o máis lento dos 40 arranques levou case o dobre do orzamento. Co M3, a mediana é do 36–37 %, un terzo menos, e o máis lento, o 62 %: a xanela aparece antes, porque xa non agarda polo programa. Esa medición pecha meiga en canto pinta, e co M3, antes do arranxo do peche, levou case quince veces máis. Co arranxo, leva o mesmo ca coa 2.10, e as medianas seguen igual: do 37–38 % do orzamento co M3, e do 54–55 % coa 2.10. |
| Pechar en canto pinta | Co M3 sen o arranxo, coa sesión desconectada, meiga pechado en canto pintaba, co perfil por defecto, tardaba sempre os 10 s dunha sesión bloqueada en saír, en doce de doce veces; co arranxo, en ningunha. O paso novo de `check-window.ps1` pasa. Unha copia de meiga que pecha no momento, coma antes do arranxo, fállao cando o peche chega mentres a shell se conecta: a metade das veces coa máquina ocupada, e ningunha das doce coa sesión conectada e tranquila. Cóllea sempre a proba unitaria do peche. |
| Repouso con 5 tabs | Igual ca coa 2.10: sen CPU nin espertares, coa mesma memoria privada, menos dun 1 % de diferenza, e os mesmos fíos. |
| `meiga.exe` | 871 936 bytes (na 2.10, 864 256): a sesión que se crea no escritor, os seus erros, os nomes dos seus fíos e o peche antes da conexión. |
| A cobertura | Só coas probas, o 79,8 % das liñas (na 2.10, o 78,1 %): `conpty.rs`, o 96,8 % (89,7 %), e `session.rs`, o 98,7 % (87,6 %), con todas as liñas do código novo. Coas comprobacións con xanela, nunha compilación instrumentada en *release*, o 95,3 %: `session.rs`, o 99,3 %; `conpty.rs`, o 96,8 %; `tabs.rs`, o 94,8 %; `app.rs`, o 76,3 %; e `main.rs`, o 57,6 %. O que queda sen cubrir son erros de Windows que non se poden provocar, os erros que se mostran nunha caixa de mensaxe, os modos `--bench`, e mover a xanela ou cambiar a súa escala. |
| Os mutantes | `cargo mutants --in-diff`: dos 67 mutantes do *diff*, as probas collen 56, 8 non compilan, e os 3 que só corren cunha xanela cóllenos as comprobacións. Trinta e unha das trinta e dúas copias de `conpty.rs` estragadas a man fallan as súas probas, e a outra, que mira a conexión cando chega o escritor, acéptase; e sete das nove copias de meiga, a súa comprobación; as outras dúas, a que pecha no momento, que `check-window.ps1` só colle coa máquina ocupada, e a que agarda o segundo enteiro, cóllenas as probas unitarias do peche. |
| Revisión do código | Ao documentar o código atopáronse un problema, ocos nas probas, nas comprobacións e na CI, e comentarios que dicían outra cousa, xa corrixidos. **O programa que non arranca:** a sesión apuntaba o erro sen marcarse como rematada, e unha tecla que chegase entón quedaba no estado, sen ninguén que a enviase; agora márcase, e unha proba compróbao. **As probas e as comprobacións:** unha proba de integración podía mirar o número do programa antes de que arrancase, e outra deixaba un cartafol temporal; o paso 14 de `check-exit.ps1` podía soltar Ctrl+Maiús antes de que meiga lese a T; e o job da cobertura seguía aínda que fallase o informe en HTML. **Despois, tres revisións independentes.** A primeira, a do primeiro arranxo do peche antes da conexión, non atopou problemas no código, que comprobou co do ConPTY; atopou que, se fallaba o paso novo de `check-window.ps1`, o seu `finally` só remataba meiga, e deixaba vivos a shell e a súa consola: agora remata os programas da tab tamén. A segunda, a da rama enteira, tampouco atopou problemas no código, pero si dous ocos nas comprobacións: o paso 9 de `check-exit.ps1` deixaba o lector dunha tab oculta reiniciada 50 ms coa prioridade normal, e unha meiga que non a baixase pasaba, porque a primeira saída do programa esperta a xanela, que a baixa; e nin o paso 14 nin a proba do programa auxiliar podían fallar cunha sesión pechada mentres arranca, porque a sesión bloqueada da primeira tab o tapaba. Agora o paso 9 percorre só os fíos de meiga, moitas veces por milisegundo, e falla se o lector aínda ten a prioridade normal 3 ms despois de recibir o número do programa; o paso 14 agarda a que arranque a terceira tab antes de pechar a xanela; e a proba pecha primeiro, soas, dúas sesións que arrancan. Esa proba nova fallou unha vez de cada cinco: coa máquina ocupada, o peche perdíase tamén agardando a conexión, porque o ConPTY a escribe antes de responderlle ao programa; decidiches rematar os programas cando o programa se conecta despois do peche (decisión 8 do [ADR 0028](../adr/0028-as-sesions-fora-do-fio-da-xanela.md)). A proba supuña ademais que `TerminateJobObject` agarda polos programas, e non agarda: agora agarda ela. A terceira, a deste arranxo, non atopou carreiras; atopou un oco nas probas, xa corrixido: ningunha collía o caso no que o escritor non ve conectarse o programa nun segundo, e unha copia que remata os programas de toda sesión pechada antes da conexión pasaba todas; agora unha proba nova fállaa. E sinalou que o escritor non sempre remata o programa antes de que execute código seu: o ConPTY respóndelle sen agardar polo lector, e se o lector se atrasa, o programa pode ter empezado. Decidiches aceptalo e dicilo nos comentarios e nos documentos. Ao documentar a comprobación, atopouse tamén que o paso 9 de `check-exit.ps1` agardaba para sempre se o lector nunca recibía o número do programa: agora falla aos `TimeoutSeconds`. **Acéptanse:** a copia que mira a conexión cando o escritor chega ao peche, que só se diferencia se o lector ve a conexión entre os dous, e as probas non o poden provocar; e o peche que chega xusto despois de que o lector vexa a conexión, que aínda se pode perder (ver *Pendente*). |
| CI | ⏳ |

## Pendente

- **O atranco ao crear un proceso** segue: agora pasa no escritor da sesión, e atrasa a primeira saída do programa, pero non a xanela. De onde vén, cun perfil de WPR, na 9.1.
- **O ConPTY de Windows 10:** o peche agarda a que o ConPTY escriba `ESC [ ? 9001 h` ao conectarse o programa. O empaquetado e o de Windows 11 escríbeno; se o de Windows 10 non o escribise, cada peche cun meiga sen o seu ConPTY ao lado agardaría 1 s. Comprobalo na 8.6, coa barra de título en Windows 10.
- **Por que a sonda deixou de bloquearse,** mentres meiga pechado en canto pinta seguía bloqueándose sempre coa sesión desconectada, e por que, coa sesión conectada e tranquila, unha copia sen o arranxo pechada igual non se bloqueou nunca, quedou sen explicar: o peche só se perde nun intre curto, e cando chega depende da máquina e da sesión. O arranxo non depende diso: en calquera caso, agarda a que o programa se conecte e remata os programas da sesión. Por iso, o paso novo de `check-window.ps1` só colle unha meiga sen o arranxo cando o peche chega nese intre, sobre todo coa máquina ocupada; as probas unitarias do peche cóllena sempre.
- **Un peche xusto despois da conexión** aínda se pode perder: o ConPTY escribe `ESC [ ? 9001 h` antes de que o programa remate de conectarse, e a súa saída non di cando remata ([ADR 0028](../adr/0028-as-sesions-fora-do-fio-da-xanela.md), consecuencias). Pode pasar sobre todo coa máquina ocupada, e entón, coma antes do M3, o escritor remata a sesión 10 s despois. A proba de integración que pecha sesións mentres arrancan, `the_sessions_that_were_closed_end_before_the_process_exits`, acéptao.
- **Un lector que se atrasa:** o escritor remata os programas dunha sesión pechada antes da conexión cando o lector ve a conexión, e o ConPTY respóndelle ao programa sen agardar polo lector. Se o lector se atrasa, coa máquina moi ocupada, co *lock* do terminal ocupado ou nunha tab oculta, coa súa prioridade máis baixa, o programa pode ter empezado a executar o seu código, e remata co código 1, sen o peche. Acéptase (decidido contigo): apuntar a conexión antes de darlle a saída ao terminal só quitaría o tempo do terminal, e levar o lector coa prioridade normal ata a conexión cambiaría a regra das tabs ocultas ([ADR 0028](../adr/0028-as-sesions-fora-do-fio-da-xanela.md), consecuencias).

## Seguinte paso

Subfase 3.1 · Títulos dinámicos: que a tab mostre o que di a shell ou a app. O título que pon o programa, en directo, e o recorte con «…» chegaron na 2.3, e o nome do perfil coma título por defecto, na 2.5; falta decidir se un reinicio da pantalla (`ESC c`) volta ao nome do perfil, porque desde a 1.5 a tab mantén o último título.
