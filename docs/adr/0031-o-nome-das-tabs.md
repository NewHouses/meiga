# ADR 0031 · O nome das tabs

- **Estado:** Aceptado
- **Data:** 2026-11-26
- **Relacionados:** [ADR 0014](0014-o-teclado.md), [ADR 0015](0015-o-rato-a-seleccion-e-o-portapapeis.md), [ADR 0020](0020-a-barra-de-tabs.md), [ADR 0021](0021-os-menus-e-o-menu-da-tab.md), [ADR 0023](0023-os-atallos-de-teclado.md), [ADR 0024](0024-o-fin-do-programa-e-o-reinicio.md), [ADR 0027](0027-a-barra-de-tabs-coma-barra-de-titulo.md), [ADR 0029](0029-o-titulo-da-tab-e-os-titulos-gardados.md), [ADR 0030](0030-o-campo-de-texto.md)

## Contexto

- **O que pide o plan.** A subfase 3.3 deixa pór a cada tab o nome que se queira: a edición ábrese cun dobre clic na tab, desde o seu menú ou con Ctrl+Maiús+R; Intro confirma e Esc cancela; un nome baleiro volta ao título dinámico, o do programa ou o nome do perfil; e o nome manda sobre os títulos do programa. É o primeiro uso do campo de texto da 3.2 ([ADR 0030](0030-o-campo-de-texto.md)), que quedou sen vista: o cursor que pestanexa, o rato en píxeles e o portapapeis da xanela chegan agora, cun límite de 1000 unidades de UTF-16, coma os títulos ([ADR 0029](0029-o-titulo-da-tab-e-os-titulos-gardados.md), decisión 6).
- **O que había.** Unha tab mostra o título que lle dá o seu programa ou, sen el, o nome do seu perfil ([ADR 0020](0020-a-barra-de-tabs.md)), e a xanela leva o título da tab que se ve. O botón esquerdo mostra unha tab ao baixar, sen contar os clics. O dobre clic nas partes libres da barra é de Windows, que maximiza a xanela ([ADR 0027](0027-a-barra-de-tabs-coma-barra-de-titulo.md)); pero as tabs son área cliente, e o seu dobre clic estaba libre. Co menú aberto, as teclas que baixan son del, e unha que xa estaba abaixo non fai nada mentres se repite ([ADR 0021](0021-os-menus-e-o-menu-da-tab.md)); os atallos de meiga collen as súas teclas antes ca o programa ([ADR 0023](0023-os-atallos-de-teclado.md)), e Ctrl+Maiús+R estaba libre. O cursor do terminal pestanexa coma o de Windows: co seu tempo, e só ata 5 s despois da última tecla.
- **As referencias,** polo seu código e pola súa documentación:

| | Como se abre | Intro sen cambiar o texto | Un nome só de espazos | A xanela perde o foco | O título da xanela | Duplicar a tab |
|---|---|---|---|---|---|---|
| Windows Terminal 1.25 | Dobre clic, ou *Rename tab* no menú da tab; as accións `renameTab` e `openTabRenamer` non teñen atallo | Garda o título que mostraba coma nome | É un nome | — | O nome | Copia o nome |
| VS Code 1.148 | *Rename* no menú, ou F2 na lista dos terminais | — | Segundo a súa mensaxe, quita o nome; no código, é un nome | Segue editando | — | — |
| Edición das etiquetas de Windows (`ListView` e `TreeView`) | F2, ou un clic no elemento escollido | Coma Esc: non cambia nada | Decide cada programa | — | — | — |

  O Windows Terminal e VS Code abren a caixa co texto enteiro seleccionado; Intro garda, Esc cancela, e un clic fóra da caixa garda. Un nome baleiro volta ao título do programa no Windows Terminal, en VS Code, en Kitty 0.51 e en WezTerm, e en todos eles un nome manda sobre os títulos que dá o programa. O Intro que escolle *Rename tab* no menú do Windows Terminal non pecha a caixa ao subir: só atende un Intro que sobe se baixou nela. E Chrome, ao arrastrar fóra dun campo, segue movendo o texto cun temporizador.

## Decisión

1. **Tres xeitos de abrila** (presentado contigo): o dobre clic nunha tab; *Rename tab*, o primeiro elemento do seu menú; e Ctrl+Maiús+R, a acción nova `rename_tab`, que o `config.toml` pode cambiar ou liberar. O dobre clic conta coma Windows: o segundo clic, dentro do tempo e do rectángulo do dobre clic, que a barra le outra vez cada vez que a xanela recupera o foco; despois dun dobre clic, o seguinte clic volta ser o primeiro; e só se os dous son na mesma tab, sen outro botón entre eles en ningún sitio: na barra, no terminal, no campo, no menú ou no marco da xanela. O primeiro clic xa mostra a tab; e *Rename tab* nunha tab oculta móstraa antes. Se a tab non se ve enteira na barra, desprázase ata que se vexa.
2. **O campo vai no sitio do título** (presentado contigo): despois do indicador e antes do botón de pechar, coa letra da barra e as cores da tab activa; a selección, na cor da selección do terminal. Se a barra corta a tab, o campo vai na parte do título que se ve, onde van os seus clics. O cursor é unha barra fina da cor do texto, tan grosa coma o cursor de Windows, que pestanexa coma o do terminal, e coma el le outra vez a configuración de Windows cada vez que a xanela recupera o foco; mentres tanto, o do terminal vese coma sen o foco, e non pestanexa. O punteiro enriba do campo é a barra de texto, agás mentres segue abaixo un botón que baixou no terminal ou na barra: entón o rato é deles, tamén un clic no campo. Empeza co texto que mostra a tab, todo seleccionado.
3. **Un nome só de espazos é ningún** (decidido contigo): os espazos dos extremos van fóra, coma nos títulos, e un nome baleiro volta ao título do programa, ou ao nome do perfil.
4. **Intro sen cambiar o texto non fai nada** (decidido contigo), coma a edición das etiquetas de Windows: unha tab que mostraba o título do programa segue mostrando os seus títulos.
5. **Un clic fóra do campo garda o nome** (decidido contigo), e despois fai o seu: noutra tab, móstraa; no terminal, selecciona; nun botón da barra, o do botón. Tamén onde manexa o rato Windows (decidido contigo despois da revisión): na parte libre da barra, na banda de enriba das tabs e nos bordos, un botón que baixa garda o nome, e despois Windows fai o seu: move a xanela, maximízaa ou restáuraa co dobre clic, cambia o seu tamaño, ou a xanela abre o seu menú co botón dereito. Para iso, `meiga-win` avisa con `Event::FrameMouseDown` antes de que Windows actúe, tamén cando a xanela leva a barra de título de Windows.
6. **Perder o foco da xanela non remata a edición** (decidido contigo), coma en VS Code: sen o foco, o cursor do campo non se ve; co foco outra vez, segue.
7. **Mentres se edita, todas as teclas son do campo** (decidido contigo), coma co menú aberto: os atallos de meiga non fan nada, e o programa non recibe ningunha tecla, nin ao baixar nin ao subir. As que estaban abaixo ao abrilo, coma o Intro que escolleu *Rename tab* ou as de Ctrl+Maiús+R, non fan nada mentres se repiten; ao subir, chéganlle ao programa se baixaran para el. Tab, ↑ e ↓ non fan nada, a roda enriba da barra non despraza as tabs, e os botóns dereito e central no campo tampouco fan nada.
8. **A xanela leva o nome** (decidido contigo), coma o Windows Terminal: o título da xanela, e o da barra de tarefas, é o nome da tab que se ve, se o ten.
9. **Duplicar copia o nome** (decidido contigo), coma o Windows Terminal; e reiniciar o programa da tab ([ADR 0024](0024-o-fin-do-programa-e-o-reinicio.md)) mantén o nome. Os títulos do programa seguen chegando mentres a tab ten nome: se o nome se borra, vese o último.
10. **Arrastrar fóra do campo segue mostrando máis texto** (decidido contigo), coma Chrome: cada 100 ms, mentres o botón esquerdo segue abaixo, co punteiro fóra por ese lado, aínda que non se mova.
11. **O resto** (presentado contigo): o botón de pechar da tab péchaa; e se a tab se pecha mentres se edita o seu nome, a edición remata con ela.

Isto concreta o [ADR 0020](0020-a-barra-de-tabs.md) (a tab mostra antes o seu nome, e o dobre clic nela edítao), o [ADR 0021](0021-os-menus-e-o-menu-da-tab.md) (o menú da tab ten un elemento máis, e un botón no marco da xanela tamén o pecha), o [ADR 0023](0023-os-atallos-de-teclado.md) (un atallo máis por defecto), o [ADR 0027](0027-a-barra-de-tabs-coma-barra-de-titulo.md) (meiga sabe dos botóns que baixan onde manexa o rato Windows antes ca el) e o [ADR 0030](0030-o-campo-de-texto.md) (a vista do campo).

## Alternativas consideradas

| Alternativa | Por que se descarta |
|---|---|
| Un nome só de espazos coma nome, coma o Windows Terminal | Decidido contigo: un nome que non se ve deixaría a tab baleira, e os títulos xa perden os espazos dos extremos. |
| Intro sen cambiar o texto garda o título coma nome, coma o Windows Terminal | Decidido contigo: non fai nada. Abrir e confirmar sen querer deixaría a tab co título dese momento para sempre; quen o queira coma nome pode escribilo. |
| Un clic fóra do campo cancela | Decidido contigo: garda, coma o Windows Terminal e VS Code, e o clic fai o seu. |
| Un clic na parte libre da barra só garda, coma co menú aberto, onde esa parte pasa a ser área cliente | Decidido contigo despois da revisión: o clic tamén fai o seu, e Windows move ou maximiza a xanela. A banda e os bordos seguirían cambiando o tamaño sen gardar. |
| Os clics onde manexa o rato Windows non gardan, e a edición segue, coma nun campo dunha xanela de Windows ao premer na súa barra de título | Decidido contigo despois da revisión: a decisión 5 vale para calquera botón que baixe fóra do campo. |
| Perder o foco da xanela garda o nome e remata | Decidido contigo: segue, coma en VS Code. Un Alt+Tab para copiar un nome noutro programa non debe rematalo a medias. |
| Os atallos de meiga antes ca o campo, coma o Windows Terminal, onde as teclas que non colle a caixa van ao mapa das accións | Decidido contigo: todas para o campo, coma no menú. O campo escribe os caracteres de Ctrl+Alt coma os de AltGr, e un atallo con Ctrl+Alt collería ese carácter. |
| A xanela co título do programa | Decidido contigo: co nome, coma o Windows Terminal. É o que se quere ver na barra de tarefas e en Alt+Tab. |
| Duplicar sen o nome | Decidido contigo: a copia dunha tab chamada «build» tamén se chama «build», coma no Windows Terminal. |
| Arrastrar fóra do campo só move o texto cando se move o punteiro | Decidido contigo: cun temporizador, coma Chrome; se non, para ver o final dun nome longo habería que abanear o rato. |
| Unha caixa de diálogo, coma Chrome co nome dunha xanela, ou unha liña aparte, coma Kitty | O plan pide editalo na propia tab, coma o Windows Terminal e VS Code. |
| Sen atallo por defecto, coma o Windows Terminal | O plan pide Ctrl+Maiús+R. |

## Consecuencias

**Positivas:**

- O nome vese onde se busca: na tab, no título da xanela e na barra de tarefas.
- O editor do nome (`rename.rs`) próbase sen xanela: cada tecla, como remata, os clics e o arrastre, o pestanexo, a configuración nova de Windows e o lugar do campo; e a barra, o dobre clic e o debuxo do campo. Na CI, `check-rename.ps1` comproba coa xanela o que debuxa e pestanexa, as teclas e os clics que non lle chegan ao programa, os clics onde manexa o rato Windows, e os nomes que levan a xanela e as copias.
- O cursor do terminal e o do campo pestanexan co mesmo código (`blink.rs`).
- Calquera parte de meiga pode reaccionar agora a un botón que baixa onde manexa o rato Windows, co evento novo de `meiga-win`.

**Negativas e riscos:**

- `meiga.exe` mide 79 872 bytes máis (952 320): entra o campo enteiro, o seu código e as táboas dos grafemas de `unicode-segmentation`, das que o [ADR 0030](0030-o-campo-de-texto.md) mediu uns 29,7 KB, e o editor do nome.
- O nome pérdese ao pechar meiga: a 7.1 gardarao no `session.toml`.
- No menú da tab, *Duplicate tab* xa non é o primeiro: co teclado, precisa ↓ antes de Intro.
- O campo debuxa os grafemas coma o resto da interface: ata a 8.1, unha familia de emoji vese coma varios emoji ([ADR 0030](0030-o-campo-de-texto.md)).

## Referencias

- Windows Terminal v1.25.3131.0:
  - [abrir co dobre clic](https://github.com/microsoft/terminal/blob/v1.25.3131.0/src/cascadia/TerminalApp/Tab.cpp#L158-L163), [*Rename tab* no menú](https://github.com/microsoft/terminal/blob/v1.25.3131.0/src/cascadia/TerminalApp/Tab.cpp#L1673-L1682), e [as accións](https://github.com/microsoft/terminal/blob/v1.25.3131.0/src/cascadia/TerminalApp/AppActionHandlers.cpp#L721-L753), sen atallo [por defecto](https://github.com/microsoft/terminal/blob/v1.25.3131.0/src/cascadia/TerminalSettingsModel/defaults.json#L721-L803);
  - `TabHeaderControl`: [Intro, Esc e o Intro que sobe](https://github.com/microsoft/terminal/blob/v1.25.3131.0/src/cascadia/TerminalApp/TabHeaderControl.cpp#L18-L55), [`BeginRename`](https://github.com/microsoft/terminal/blob/v1.25.3131.0/src/cascadia/TerminalApp/TabHeaderControl.cpp#L70-L83) e [perder o foco](https://github.com/microsoft/terminal/blob/v1.25.3131.0/src/cascadia/TerminalApp/TabHeaderControl.cpp#L93-L135);
  - [o nome antes ca o título](https://github.com/microsoft/terminal/blob/v1.25.3131.0/src/cascadia/TerminalApp/Tab.cpp#L1008-L1028), [o título da xanela](https://github.com/microsoft/terminal/blob/v1.25.3131.0/src/cascadia/TerminalApp/TerminalPage.cpp#L2840-L2849) e [duplicar](https://github.com/microsoft/terminal/blob/v1.25.3131.0/src/cascadia/TerminalApp/TabManagement.cpp#L278-L301).
- VS Code 1.148.0: [a caixa na lista dos terminais](https://github.com/microsoft/vscode/blob/1.148.0/src/vs/workbench/browser/parts/views/inlineEditInputBox.ts#L81-L132) e [o nome baleiro](https://github.com/microsoft/vscode/blob/1.148.0/src/vs/workbench/contrib/terminal/browser/terminalActions.ts#L1742-L1757).
- Kitty v0.51.1, [o nome antes ca o título](https://github.com/kovidgoyal/kitty/blob/v0.51.1/kitty/tabs.py#L669-L688); e WezTerm 20240203-110809-5046fc22, [o mesmo](https://github.com/wezterm/wezterm/blob/20240203-110809-5046fc22/wezterm-gui/src/tabbar.rs#L106-L124).
- Microsoft: [`LVN_ENDLABELEDIT`](https://learn.microsoft.com/en-us/windows/win32/controls/lvn-endlabeledit), [`GetDoubleClickTime`](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-getdoubleclicktime), [as mensaxes do dobre clic](https://learn.microsoft.com/en-us/windows/win32/inputdev/about-mouse-input#double-click-messages), que veñen de catro en catro, [`WM_NCLBUTTONDBLCLK`](https://learn.microsoft.com/en-us/windows/win32/inputdev/wm-nclbuttondblclk) e [`GetCaretBlinkTime`](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-getcaretblinktime).
- [Tecnoloxías › Renomear unha tab noutros programas](../tecnoloxias.md#67-renomear-unha-tab-noutros-programas).
- [Código › meiga › `rename.rs`](../codigo/meiga.md#cratesmeigasrcrenamers), [`blink.rs`](../codigo/meiga.md#cratesmeigasrcblinkrs), [`tabs.rs`](../codigo/meiga.md#cratesmeigasrctabsrs) e [`tabbar.rs`](../codigo/meiga.md#cratesmeigasrctabbarrs).
- [Subfases › 3.3 · Renomear](../subfases/3.3-renomear.md).
