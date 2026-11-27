# Código

> Estado: subfase 3.3. Na 3.3, renomear as tabs, en `meiga`, `meiga-core` e `meiga-win`, que avisa dos botóns que baixan onde manexa o rato Windows, coas súas probas, e a súa comprobación, `check-rename.ps1`, que tamén leva a CI; e `check-menu.ps1`, `check-exit.ps1` e `check-profiles.ps1` van un elemento máis abaixo no menú dunha tab, que empeza agora por *Rename tab*, e `check-menu.ps1` comproba tamén que un botón na banda de enriba das tabs pecha o menú; na 3.2, o campo de texto, en `meiga-core`, coas súas probas, e un exemplo para probalo a man en `meiga-win`; na 3.1, o terminal de `meiga-core` leva el mesmo o título e os títulos gardados, con probas en `meiga-core`, `meiga-win` e `meiga`, e `check-tabbar.ps1` comproba o título despois dun reinicio da pantalla; no M3, as sesións créanse no seu fío escritor, en `meiga-win` e `meiga`, con máis probas, `check-exit.ps1` comproba a liña do cartafol que non existe, a prioridade do lector reiniciado desde que arranca o programa e unha tab que se abre mentres meiga sae, `check-window.ps1` pecha meiga en canto pinta, e a CI mide a cobertura das probas; na 2.10 cambiaron o *workflow* da CI e dúas comprobacións dos escenarios; na 2.9 chegaron as tabs na barra de título, en `meiga-core`, `meiga-win` e `meiga`, e a súa comprobación; no M2, o peche das sesións ao saír, en `meiga-win` e `meiga`, e a súa comprobación; na 2.8, moitas tabs e os indicadores, en `meiga-core` e `meiga`, e a súa comprobación; na 2.7, o fin do programa e o reinicio, en `meiga-core` e `meiga`, e a súa comprobación; na 2.6, os atallos de teclado, en `meiga-core` e `meiga`, e a súa comprobación; na 2.5, os perfís e a configuración, en `meiga-core`, `meiga-win` e `meiga`, e a súa comprobación.

Explicación detallada do código, arquivo por arquivo. Para o *porqué* das decisións xerais, ver [Arquitectura](../arquitectura.md) e [Tecnoloxías](../tecnoloxias.md).

## Mapa do repositorio

```text
meiga/
├─ .cargo/config.toml      → workspace.md
├─ .github/
│  ├─ workflows/ci.yml     → github.md
│  └─ dependabot.yml       → github.md
├─ .gitattributes          → workspace.md
├─ .gitignore              → workspace.md
├─ Cargo.toml              → workspace.md
├─ Cargo.lock              → workspace.md
├─ about.toml              → workspace.md
├─ clippy.toml             → workspace.md
├─ deny.toml               → workspace.md
├─ rust-toolchain.toml     → workspace.md
├─ rustfmt.toml            → workspace.md
├─ README.md, LICENSE, SECURITY.md → workspace.md
├─ crates/
│  ├─ meiga-core/          → meiga-core.md
│  ├─ meiga-render/        → meiga-render.md
│  ├─ meiga-win/           → meiga-win.md
│  └─ meiga/               → meiga.md
├─ tools/                  → ferramentas.md
│  └─ perf/                → perf.md
│     ├─ scenarios/        → escenarios.md
│     └─ gen-bigfile.ps1, run-scenario.ps1, test-scenarios.ps1 → escenarios.md
└─ docs/                   (esta documentación)
```

| Documento | Contido |
|---|---|
| [workspace.md](workspace.md) | Configuración da raíz: Cargo, coas dependencias, como o parser de TOML, versión de Rust, *lints*, formato, licenzas e Git. |
| [github.md](github.md) | Integración continua (GitHub Actions), coas comprobacións de meiga, tamén a dos perfís e da configuración, a do nome das tabs, a dos atallos de teclado, a do fin dos programas, a das moitas tabs e a da barra de título, e Dependabot. |
| [meiga-core.md](meiga-core.md) | Lóxica independente de Windows: o modelo das xanelas e as súas tabs, que pecha tamén varias dunha vez, a disposición da barra de tabs, que é tamén a barra de título, coa banda que cambia o tamaño da xanela, o sitio para arrastrala e os seus botóns, co ancho mínimo das tabs e a franxa que as despraza cando non caben, os menús, cos seus elementos, o resalte e o sitio de cada parte, as cores e a paleta, os rectángulos, o tamaño da grella, as zonas que cambian, cando debuxar o seguinte fotograma, o ancho dos caracteres e cortar un texto con «…», as comiñas da liña de comandos e o nome do programa, as teclas e o que le o programa para cada unha, o rato e os seus informes, o texto que se pega, o arquivo de configuración, coa súa táboa de atallos, os atallos de teclado, coas súas combinacións, as súas accións, tamén a que renomea a tab, e os atallos por defecto, os perfís e o perfil por defecto, o contorno dos programas, unha ordenación por inserción, o campo de texto, que edita unha liña por grafemas e palabras, con desfacer, o límite, o pegado e o rato, e o terminal: a fachada sobre `alacritty_terminal`, co historial, o cambio de tamaño, a selección, o reinicio, o escáner e as gravacións, coas súas probas e dous exemplos: un que mide a velocidade do terminal cunha gravación, e outro que proba o modelo das tabs a man. |
| [meiga-render.md](meiga-render.md) | Renderizador por CPU: o lenzo, cos seus anacos con coordenadas propias, o *framebuffer* e a súa presentación na xanela, as fontes de DirectWrite, coa negra, a cursiva e as liñas do subliñado e do riscado, a caché de glifos e o debuxo do texto. |
| [meiga-win.md](meiga-win.md) | Capa de plataforma de Windows: a xanela, co seu marco propio, para que a barra de tabs sexa a barra de título, o que é para Windows cada parte dela, o seu tamaño mínimo, o sitio que deixa maximizada a unha barra de tarefas que se oculta e o seu menú, a súa escala e os seus cambios, os seus eventos, tamén o de cando xa está na pantalla, o teclado, o rato, tamén onde o manexa Windows, e cando sae da xanela, o título, os avisos desde outros fíos, co seu número, o bucle de mensaxes, o temporizador dos fotogramas, os axustes do cursor e do rato de Windows, o portapapeis e as caixas de mensaxe, a que pregunta e a que avisa; os perfís, que atopa no rexistro e nos arquivos, o contorno de meiga e o cartafol do sistema; as sesións de ConPTY, o seu tamaño, o cartafol e o contorno do seu programa, a prioridade dos seus lectores e a espera, ao saír, polas que se pecharon, coas súas probas, un exemplo que tamén as grava, e outro que proba o campo de texto a man na consola. |
| [meiga.md](meiga.md) | O programa executábel: as opcións e o programa da liña de comandos, os perfís, a configuración e os seus atallos (`shells.rs`), as tabs, cada unha coa súa sesión, o fin do seu programa e o reinicio, e o seu nome, que se edita na propia tab cun cursor que pestanexa coma o do terminal, a barra de tabs, que é tamén a barra de título, co rato, os títulos, os indicadores e os botóns, co + dividido e os tres da xanela, e a franxa que despraza as tabs que non caben, coa roda e os seus dous botóns, os menús, co menú dunha tab, que a renomea, a duplica, a reinicia e pecha varias tabs dunha vez, e o dos perfís, que se abre tamén con Ctrl+Maiús+Espazo, a vista que mostra a activa, a debuxa, busca os atallos de teclado e lle envía ao programa as outras teclas, o seu tamaño, o historial, a barra de desprazamento e o zoom, o rato da vista, copiar e pegar, o fluxo de fotogramas, os modos de medición, o rexistro dos fotogramas e, ao saír, a espera polas sesións que se pecharon; o seu manifesto e o `build.rs` que o incrusta. |
| [ferramentas.md](ferramentas.md) | Scripts e modelos de `tools/`: empaquetado, descarga do ConPTY, comprobación das DLL, da xanela, dos fotogramas, do teclado, do rato e do portapapeis, do tamaño, do historial e do zoom, das tabs, da barra de tabs, do menú dunha tab, do nome das tabs, dos perfís e da configuración, dos atallos de teclado, do fin dos programas e do peche das sesións ao saír, das moitas tabs, da barra de título, das ligazóns da documentación e de que os scripts piden PowerShell 7, os avisos de terceiros e unha mostra de cores e estilos para comprobar un terminal a ollo. As comprobacións arrancan meiga cun `APPDATA` propio, baleiro ou co `config.toml` que comproban. |
| [perf.md](perf.md) | Medición do rendemento (`tools/perf/`): o script que mide os terminais, o código C# que le os procesos, a súa proba e as medicións do arranque, cun `APPDATA` baleiro, e dos fotogramas, tamén os de texto. |
| [escenarios.md](escenarios.md) | Escenarios de carga (`tools/perf/scenarios/`): o spinner, a IA e o arquivo grande, en PowerShell e en bash, o script que mide un terminal mentres corren, que tamén arranca meiga, cun `APPDATA` baleiro, e as súas probas. |
