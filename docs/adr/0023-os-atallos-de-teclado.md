# ADR 0023 · Os atallos de teclado

- **Estado:** Aceptado
- **Data:** 2026-11-16
- **Relacionados:** [ADR 0014](0014-o-teclado.md), [ADR 0015](0015-o-rato-a-seleccion-e-o-portapapeis.md), [ADR 0016](0016-o-historial-o-tamano-e-o-zoom.md), [ADR 0019](0019-varias-sesions-e-tabs-en-segundo-plano.md), [ADR 0021](0021-os-menus-e-o-menu-da-tab.md), [ADR 0022](0022-os-perfis-e-a-configuracion.md)

## Contexto

- A subfase 2.6 trae os atallos de teclado: manexar as tabs sen rato e sen roubarlle teclas á shell. O plan pídelle un sistema de atallos configurábel, con Ctrl+Maiús+T e Ctrl+Maiús+W, Ctrl+Tab e Ctrl+Maiús+Tab, Ctrl+1…9 e Ctrl+Maiús+1…9, o perfil N. Feito cando Ctrl+T e Ctrl+W seguen chegando á shell.
- O que xa había: as teclas que colle meiga estaban fixas no código, cada unha coas súas condicións. Copiar e pegar desde a 1.8 ([ADR 0015](0015-o-rato-a-seleccion-e-o-portapapeis.md), decisións 4 e 5), o historial e o zoom desde a 1.9 ([ADR 0016](0016-o-historial-o-tamano-e-o-zoom.md), decisións 8 e 12), catro atallos provisionais das tabs desde a 2.2, «ata que a 2.6 faga os atallos configurábeis» ([ADR 0019](0019-varias-sesions-e-tabs-en-segundo-plano.md), decisión 11), o menú da tab desde a 2.4 ([ADR 0021](0021-os-menus-e-o-menu-da-tab.md), decisión 6) e o dos perfís desde a 2.5 ([ADR 0022](0022-os-perfis-e-a-configuracion.md), decisión 6). Ningunha tiña Alt, así que AltGr, que Windows dá coma o Ctrl esquerdo e o Alt dereito, nunca coincidía con elas. As teclas dun atallo non lle chegan ao programa, nin cando soben nin mentres repiten, e só algunhas repiten a súa acción. Desde a 2.5, meiga le un `config.toml`, e os erros móstranse nunha caixa, coa súa liña e a súa columna ([ADR 0022](0022-os-perfis-e-a-configuracion.md), decisións 1, 2 e 8).
- Comprobouse antes de escribilo:
  - **O Windows Terminal** v1.25.3131.0 ten Ctrl+Maiús+1…9 para abrir o perfil N do seu menú, Ctrl+Maiús+D para duplicar a tab, Ctrl+Tab e Ctrl+Maiús+Tab, e Ctrl+Alt+1…8 para mostrar a tab N e Ctrl+Alt+9 a última. Escribe os atallos coma `ctrl+shift+t`: os modificadores `ctrl`, `alt`, `shift` e `win`, e unha tecla, unidos con `+`; as letras e os díxitos, os nomes coma `enter`, `pgup` ou `numpad_plus`, `vk(N)`, `sc(N)` ou un carácter, que busca na distribución do teclado de agora. Distingue maiúsculas nos nomes das teclas, salvo nas letras, e non admite espazos. `ctrl+a+b` é un erro. Non mira os atallos con AltGr, e só cando a tecla baixa. Cando a tab N non existe, mostra a última; e cando non existe o perfil N, non abre nada, e a tecla vai ao programa.
  - **Chrome** 158 mostra a tab N con Ctrl+1…8, e a última con Ctrl+9, tamén cos díxitos do teclado numérico; se a tab N non existe, non fai nada. Mostra a anterior e a seguinte con Ctrl+RePáx e Ctrl+AvPáx, ademais de Ctrl+Tab. O seu código di que non se use Ctrl+Alt nos atallos, porque é o AltGr dos teclados doutros idiomas. **O Warp** ten os mesmos en Windows: Ctrl+1…8, Ctrl+9, Ctrl+RePáx e Ctrl+AvPáx.
  - **A consola** dálles aos programas que len VT, coma os de WSL, un carácter de control con Ctrl e algúns díxitos: Ctrl+2 é NUL, o Ctrl+@ de readline, que pon a marca; Ctrl+3, ESC; Ctrl+4 e Ctrl+5, FS e GS; Ctrl+6, RS, o Ctrl+^ de vim, que volta ao arquivo anterior; Ctrl+7, US, o Ctrl+_ de readline, que desfai; e Ctrl+8, DEL. Ctrl+1 e Ctrl+9 dan o propio díxito.
  - **PSReadLine** 2.4.5, no modo de Windows, non usa Ctrl+1…9; Alt+0…9 dan un argumento numérico, coma en bash, e Ctrl+RePáx e Ctrl+AvPáx desprazan a xanela da consola unha liña. **vim** usa Ctrl+RePáx e Ctrl+AvPáx para cambiar de páxina de tabs.

## Decisión

1. **Os atallos, nunha táboa `[shortcuts]` do `config.toml`** (decidido contigo), cun atallo e unha acción en cada liña:

   ```toml
   [shortcuts]
   "ctrl+t" = "new_tab"          # outro atallo para unha acción
   "ctrl+6" = "none"             # Ctrl+6 chega outra vez ao programa
   "ctrl+shift+u" = "profile_4"
   ```

   meiga ten os seus atallos, os da decisión 9, e o arquivo engade outros, cámbialles a acción ou libéraos: `"none"` libera un atallo, e a tecla vai entón ao programa. Os que o arquivo non nomea quedan coma estaban, e unha acción que recibe un atallo novo mantén os seus. meiga le a táboa ao arrancar, co resto do arquivo; a recarga, na 8.4. Concreta a decisión 3 do [ADR 0022](0022-os-perfis-e-a-configuracion.md).
2. **A sintaxe do Windows Terminal**: os modificadores `ctrl`, `alt` e `shift`, en calquera orde, e unha tecla, unidos con `+`. Sen distinguir maiúsculas, tamén nos nomes das teclas, e sen contar os espazos arredor de cada parte; un modificador repetido é o mesmo. As teclas son as letras, os díxitos, `f1` a `f24`, `numpad0` a `numpad9` (ou `numpad_0`…) e os nomes do Windows Terminal, sen os das teclas do navegador: `enter`, `tab`, `space`, `backspace`, `menu` ou `app`, `insert`, `delete`, `home`, `end`, `pgup` ou `pageup`, `pgdn` ou `pagedown`, `esc` ou `escape`, `left`, `right`, `up`, `down`, `numpad_plus` ou `numpad_add`, `numpad_minus` ou `numpad_subtract`, `numpad_multiply`, `numpad_divide`, `numpad_period` ou `numpad_decimal`, e `plus`, `comma`, `minus` e `period`, as teclas que escriben eses caracteres en calquera distribución: a tecla `+` escríbese `plus`. Cada tecla vai pola súa tecla virtual, e non polo carácter que escribe: `ctrl+shift+2` é o mesmo atallo con calquera distribución, aínda que o 2 con Maiús escriba `@` nunhas e `"` noutras. As accións, coma na táboa da decisión 4, en minúsculas. Non hai `win`, porque as teclas Windows son de Windows, nin Alt+F4 nin Alt+Espazo, tamén con Maiús, que pechan a xanela e abren o seu menú; nin `vk()`, `sc()` ou un carácter calquera, que depende da distribución. Os atallos de meiga non teñen Alt, así que as combinacións con Alt seguen indo ao programa, salvo as que o arquivo faga atallos: concreta a decisión 5 do [ADR 0014](0014-o-teclado.md).
3. **Os erros, coma os outros do arquivo** ([ADR 0022](0022-os-perfis-e-a-configuracion.md), decisións 2 e 8): na caixa de mensaxe, coa súa liña e a súa columna, a do atallo se está mal o atallo, e a da acción se está mal a acción. Só se ignora a entrada mal. **O mesmo atallo dúas veces**, escrito igual ou doutra maneira, coma `"alt+o"` e `"Alt+O"`: vale o primeiro, e o seguinte é un erro, «repeats the shortcut of line N», coma un perfil co nome doutro. Escrito exactamente igual, as dúas chaves son a mesma para TOML, e o arquivo enteiro é un erro de sintaxe.
4. **As accións** son as que xa facía meiga, coa tab e o perfil N, e duplicar (decisión 8):

   | Acción | Atallos de meiga | Que fai |
   |---|---|---|
   | `new_tab` | Ctrl+Maiús+T | Abre unha tab co perfil por defecto, coma o +. |
   | `close_tab` | Ctrl+Maiús+W | Pecha a tab que se mostra; a última pecha a xanela. |
   | `next_tab` | Ctrl+Tab, Ctrl+AvPáx | Mostra a seguinte, ou a primeira despois da última. |
   | `previous_tab` | Ctrl+Maiús+Tab, Ctrl+RePáx | Mostra a anterior, ou a última antes da primeira. |
   | `tab_1`…`tab_N` | Ctrl+1…Ctrl+8 | Mostra a tab N da barra. |
   | `last_tab` | Ctrl+9 | Mostra a última tab. |
   | `profile_1`…`profile_N` | Ctrl+Maiús+1…Ctrl+Maiús+9 | Abre unha tab co perfil N do menú dos perfís. |
   | `profile_menu` | Ctrl+Maiús+Espazo | Abre o menú dos perfís. |
   | `tab_menu` | Menú, Maiús+Menú, Maiús+F10 | Abre o menú da tab que se mostra. |
   | `duplicate_tab` | Ctrl+Maiús+D | Duplica a tab que se mostra. |
   | `close_other_tabs` | — | Pecha as outras tabs. |
   | `close_tabs_to_the_right` | — | Pecha as tabs á dereita da que se mostra. |
   | `copy` | Ctrl+C, Ctrl+Maiús+C | Copia a selección, se a hai. |
   | `paste` | Ctrl+V, Ctrl+Maiús+V | Pega o portapapeis. |
   | `scroll_up`, `scroll_down` | Ctrl+Maiús+↑, Ctrl+Maiús+↓ | Move a vista do historial unha fila. |
   | `scroll_page_up`, `scroll_page_down` | Maiús+RePáx e Ctrl+Maiús+RePáx; Maiús+AvPáx e Ctrl+Maiús+AvPáx | Unha pantalla. |
   | `scroll_to_top`, `scroll_to_bottom` | Ctrl+Maiús+Inicio, Ctrl+Maiús+Fin | Ao principio do historial e á pantalla. |
   | `zoom_in`, `zoom_out`, `zoom_reset` | Ctrl e `+`, Ctrl e `-`, Ctrl+0, e eses tres no teclado numérico | Un punto máis, un menos, e o tamaño do principio. |

   `tab_N` e `profile_N` contan desde 1, coma a barra e o menú: `tab_0` é un erro. Copiar sen selección e as teclas do historial na pantalla alternativa, que non ten historial, deixan pasar a tecla ao programa, coma antes.
5. **Ctrl+1…8 mostran a tab N, e Ctrl+9 a última** (decidido contigo), coma Chrome e o Warp. Só os díxitos da fila de arriba, sen os do teclado numérico, e sen Alt: o Windows Terminal usa Ctrl+Alt+1…9, e Ctrl+Alt é o AltGr dos teclados que o usan para escribir, coma o español, onde AltGr+2 escribe `@`.
6. **Ctrl+Maiús+1…9 abren o perfil N do menú** (decidido contigo), coma no Windows Terminal: unha tab ao final, que se mostra, coma escollelo no menú. meiga busca as shells outra vez antes, coma ao abrir o menú, para que N sexa o perfil que mostraría o menú nese momento. Concreta a decisión 6 do [ADR 0022](0022-os-perfis-e-a-configuracion.md).
7. **Ctrl+RePáx e Ctrl+AvPáx mostran a tab anterior e a seguinte** (decidido contigo), coma en Chrome e no Warp, ademais de Ctrl+Maiús+Tab e Ctrl+Tab. A miña recomendación era deixalas para o programa, coma o Windows Terminal: vim e PSReadLine úsanas (Contexto). Quen as queira no programa libéraas con `"none"`.
8. **Ctrl+Maiús+D duplica a tab que se mostra** (decidido contigo), coma no Windows Terminal: o mesmo ca *Duplicate tab* no seu menú, coa copia xusto á súa dereita, que se mostra ([ADR 0021](0021-os-menus-e-o-menu-da-tab.md), decisión 10). Adiántase da 4.5, onde queda que a copia empece no cartafol actual da shell.
9. **Todas as teclas que xa colle meiga son atallos, e configurábeis** (decidido contigo): as da táboa da decisión 4, que son as de antes, coas novas desta subfase, Ctrl+1…9, Ctrl+Maiús+1…9, Ctrl+RePáx, Ctrl+AvPáx e Ctrl+Maiús+D. Ctrl+T e Ctrl+W seguen indo ao programa. Esc non é un atallo: cunha selección, quítaa antes de mirar os atallos, e sen ela, vai ao programa, coma desde a 1.8 ([ADR 0015](0015-o-rato-a-seleccion-e-o-portapapeis.md), decisión 4). Concreta a decisión 11 do [ADR 0019](0019-varias-sesions-e-tabs-en-segundo-plano.md), a 6 do [ADR 0021](0021-os-menus-e-o-menu-da-tab.md), a 6 do [ADR 0022](0022-os-perfis-e-a-configuracion.md), a 4 e a 5 do [ADR 0015](0015-o-rato-a-seleccion-e-o-portapapeis.md), e a 8 e a 12 do [ADR 0016](0016-o-historial-o-tamano-e-o-zoom.md): as súas teclas son agora os atallos de meiga, que o arquivo pode cambiar.
10. **AltGr nunca dispara un atallo**, coma no Windows Terminal: escribe caracteres, coma `@`, `#`, `€` ou os corchetes no teclado español. Para meiga, AltGr é o Ctrl esquerdo e o Alt dereito sen os outros dous, coma no teclado desde a 1.7 ([ADR 0014](0014-o-teclado.md)); calquera outra combinación de Ctrl e Alt é Ctrl+Alt, e dispara os atallos `ctrl+alt+…` que poña o arquivo. O Windows Terminal mira só que estean eses dous, e non os outros.
11. **Unha tab ou un perfil N que non existe non fai nada**, e a tecla tampouco lle chega ao programa: a tecla é dun atallo de meiga, teña ou non onde ir. Coa tab, coma en Chrome. O Windows Terminal fai outra cousa en cada caso: mostra a última tab, e deixa a tecla do perfil para o programa (Contexto).
12. **Repiten mentres se mantén a tecla** a tab seguinte e a anterior, o historial e o zoom, menos o tamaño do principio; as outras accións, non: unha tecla mantida abre unha soa tab. As teclas dun atallo non lle chegan ao programa mentres repiten nin cando soben, repitan ou non a acción, coma ata agora; e os modificadores que baixaron para o programa chegan a el tamén cando soben. Ao revés, unha tecla que baixou para o programa é súa ata que sobe: as súas repeticións non buscan atallos, tamén cos modificadores que se premen mentres tanto, e o programa recibe a súa soltura. Con AvPáx mantido en `less`, premer Ctrl non mostra a tab seguinte: se a mostrase, `less` non recibiría nunca AvPáx ao subir, e recibiríao o programa da outra tab. É a regra dos menús desde a 2.4 ([ADR 0021](0021-os-menus-e-o-menu-da-tab.md), decisión 7).
13. **Os atallos non se mostran nos menús** (decidido contigo): na 8.3, co tema e a aparencia.

## Alternativas consideradas

| Alternativa | Por que se descarta |
|---|---|
| Unha lista de accións coma a do Windows Terminal, con `keys` e `command` | Máis longa de escribir a man. Na táboa, cada atallo é unha chave, e TOML xa non deixa escribir dúas veces o mesmo texto. |
| A acción coma chave, `new_tab = "ctrl+t"` | Para liberar unha tecla habería que saber de que acción é, e unha acción con varios atallos necesitaría unha lista. |
| Ctrl+Alt+1…9 para a tab N, coma o Windows Terminal | Ctrl+Alt é AltGr nos teclados que o usan para escribir (decisión 5), e Chrome non o usa por iso. |
| Alt+1…9, coma Chrome en Linux e outros terminais | No PSReadLine, Alt e un díxito dan un argumento numérico, e en bash tamén. |
| Ctrl cos díxitos do teclado numérico, coma Chrome | Decidido contigo, só a fila de arriba: o teclado numérico queda para o programa, e o arquivo pode engadilos. |
| Deixar Ctrl+RePáx e Ctrl+AvPáx ao programa, coma o Windows Terminal | Era a miña recomendación; decidiches telas coma en Chrome e no Warp (decisión 7). |
| `vk()`, `sc()` e calquera carácter, coma o Windows Terminal | Un carácter depende da distribución do teclado de cada momento, e `vk()` e `sc()` case nunca fan falta. Poden chegar despois sen cambiar o resto. |
| Que valla o último dun atallo repetido | Un perfil co nome doutro xa deixaba o primeiro ([ADR 0022](0022-os-perfis-e-a-configuracion.md), decisión 2): o mesmo para os atallos. |
| Cunha tab N que non existe, mostrar a última, coma o Windows Terminal | A última xa ten o seu atallo, Ctrl+9, e un díxito de máis mostraría unha tab que non se pediu. |
| Cunha tab ou un perfil N que non existe, deixar a tecla ao programa, coma o Windows Terminal cos perfís | O programa recibiría unha tecla ou non segundo cantas tabs houbese abertas, ou cantas shells atopase meiga en cada equipo. |
| Que todos os atallos repitan mentres se mantén a tecla | Unha tecla mantida un pouco de máis abriría ou pecharía varias tabs. |
| Que as repeticións dunha tecla que baixou para o programa busquen atallos, cos modificadores que se premen despois | O programa non recibiría nunca a soltura da tecla e, se o atallo mostrase outra tab, recibiríaa o programa desa tab (decisión 12). |
| Mostrar os atallos nos menús agora | Decidido contigo: na 8.3. |

## Consecuencias

**Positivas:**

- As tabs manéxanse co teclado: mostrar unha polo seu sitio, a última, a anterior e a seguinte, abrir calquera perfil e duplicar a que se mostra.
- Calquera tecla de meiga pódese cambiar ou liberar, e engadir outras; Ctrl+T e Ctrl+W seguen chegando á shell.
- Un atallo mal só ignora a súa liña, e a caixa di onde está.
- AltGr segue escribindo, tamén cun atallo `ctrl+alt+…` no arquivo.
- As teclas de meiga son unha táboa, e non condicións espalladas polo código: atopar a acción dunha tecla é buscar nunha lista dunhas cincuenta entradas, só cando a tecla baixa. En repouso, nada.
- `check-shortcuts.ps1` comproba na CI a caixa dos erros, as teclas que chegan ao programa e as que colle meiga, AltGr, as repeticións, tamén as dunha tecla que baixou para o programa, e os atallos das tabs e dos perfís, cos do arquivo.

**Negativas e riscos:**

- `meiga.exe` medra uns 13 KB.
- Ctrl+RePáx, Ctrl+AvPáx e Ctrl+1…9 xa non chegan ao programa, salvo que o arquivo os libere: vim perde os seus atallos das páxinas de tabs e Ctrl+^ (Ctrl+6), e readline a marca de Ctrl+@ (Ctrl+2) e o desfacer de Ctrl+_ (Ctrl+7).
- Ata a 8.4, un cambio nos atallos necesita reiniciar meiga, e ata a 8.3 os menús non os mostran.
- Pechar as outras tabs ou as da dereita co teclado necesita un atallo no arquivo, ou o menú da tab.
- AltGr non se pode usar nos atallos.

## Referencias

- Windows Terminal v1.25.3131.0:
  - `defaults.json`: [as tabs e os perfís](https://github.com/microsoft/terminal/blob/v1.25.3131.0/src/cascadia/TerminalSettingsModel/defaults.json#L739-L761) e [o zoom](https://github.com/microsoft/terminal/blob/v1.25.3131.0/src/cascadia/TerminalSettingsModel/defaults.json#L797-L802);
  - `KeyChordSerialization.cpp`: [os nomes das teclas](https://github.com/microsoft/terminal/blob/v1.25.3131.0/src/cascadia/TerminalSettingsModel/KeyChordSerialization.cpp#L22-L88) e [como le un atallo](https://github.com/microsoft/terminal/blob/v1.25.3131.0/src/cascadia/TerminalSettingsModel/KeyChordSerialization.cpp#L124-L234);
  - [sen atallos con AltGr, e só ao baixar a tecla](https://github.com/microsoft/terminal/blob/v1.25.3131.0/src/cascadia/TerminalControl/TermControl.cpp#L1800-L1814), en `TermControl.cpp`, e [que é AltGr](https://github.com/microsoft/terminal/blob/v1.25.3131.0/src/cascadia/TerminalCore/ControlKeyStates.hpp#L124-L127), en `ControlKeyStates.hpp`;
  - en `AppActionHandlers.cpp`, [un perfil N que non existe deixa a tecla para o programa](https://github.com/microsoft/terminal/blob/v1.25.3131.0/src/cascadia/TerminalApp/AppActionHandlers.cpp#L459-L478) e [mostrar a tab N](https://github.com/microsoft/terminal/blob/v1.25.3131.0/src/cascadia/TerminalApp/AppActionHandlers.cpp#L480-L488); e, en `TabManagement.cpp`, [unha tab N que non existe pasa a ser a última](https://github.com/microsoft/terminal/blob/v1.25.3131.0/src/cascadia/TerminalApp/TabManagement.cpp#L539-L562);
  - [Ctrl cos díxitos para os programas que len VT](https://github.com/microsoft/terminal/blob/v1.25.3131.0/src/terminal/input/terminalInput.cpp#L497-L521), en `terminalInput.cpp`.
- Chromium 158.0.8127.37: en `accelerator_table.cc`, [sen Ctrl+Alt](https://github.com/chromium/chromium/blob/158.0.8127.37/chrome/browser/ui/accelerator_table.cc#L42-L43) e [os atallos das tabs](https://github.com/chromium/chromium/blob/158.0.8127.37/chrome/browser/ui/accelerator_table.cc#L76-L100); e, en `browser_commands.cc`, [a tab N e a última](https://github.com/chromium/chromium/blob/158.0.8127.37/chrome/browser/ui/browser_commands.cc#L1584-L1619).
- Raymond Chen, *The Old New Thing*: [*Why Ctrl+Alt shouldn't be used as a shortcut modifier*](https://devblogs.microsoft.com/oldnewthing/20040329-00/?p=40003).
- Warp, a documentación, no commit `9894bc0`: [os atallos das tabs en Windows](https://github.com/warpdotdev/docs/blob/9894bc06861026ac727b9ea6cb7fac26bc6abe51/src/content/docs/getting-started/keyboard-shortcuts.mdx#L337-L355).
- PSReadLine v2.4.5: [Ctrl+RePáx e Ctrl+AvPáx](https://github.com/PowerShell/PSReadLine/blob/v2.4.5/PSReadLine/KeyBindings.cs#L256-L261) e [Alt cos díxitos](https://github.com/PowerShell/PSReadLine/blob/v2.4.5/PSReadLine/KeyBindings.cs#L220-L230), en `KeyBindings.cs`.
- O manual de bash: [`digit-argument`](https://www.gnu.org/software/bash/manual/html_node/Numeric-Arguments.html), e [`undo` e `set-mark`](https://www.gnu.org/software/bash/manual/html_node/Miscellaneous-Commands.html).
- vim: [`CTRL-<PageDown>`](https://vimhelp.org/tabpage.txt.html#CTRL-%3CPageDown%3E) e [`CTRL-<PageUp>`](https://vimhelp.org/tabpage.txt.html#CTRL-%3CPageUp%3E), en `tabpage.txt`, e [`CTRL-^`](https://vimhelp.org/editing.txt.html#CTRL-%5E), en `editing.txt`.
- Microsoft Learn: [*Virtual-Key Codes*](https://learn.microsoft.com/en-us/windows/win32/inputdev/virtual-key-codes) e [*Keyboard Input Overview*](https://learn.microsoft.com/en-us/windows/win32/inputdev/about-keyboard-input).
- [Tecnoloxías › Os atallos de teclado](../tecnoloxias.md#58-os-atallos-de-teclado-as-teclas-virtuais-altgr-e-os-doutros-programas).
- [Arquitectura › Os atallos de teclado](../arquitectura.md#os-atallos-de-teclado).
- [Código › meiga-core › `shortcut.rs`](../codigo/meiga-core.md#cratesmeiga-coresrcshortcutrs) e [› `config.rs`](../codigo/meiga-core.md#cratesmeiga-coresrcconfigrs), e [Código › meiga › `view.rs`](../codigo/meiga.md#cratesmeigasrcviewrs) e [› `tabs.rs`](../codigo/meiga.md#cratesmeigasrctabsrs).
