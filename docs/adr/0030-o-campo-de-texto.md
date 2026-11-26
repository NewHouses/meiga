# ADR 0030 · O campo de texto

- **Estado:** Aceptado
- **Data:** 2026-11-26
- **Relacionados:** [ADR 0002](0002-windows-e-renderizado-cpu.md), [ADR 0006](0006-licenzas-e-dependencias.md), [ADR 0010](0010-texto-con-directwrite.md), [ADR 0014](0014-o-teclado.md), [ADR 0015](0015-o-rato-a-seleccion-e-o-portapapeis.md), [ADR 0020](0020-a-barra-de-tabs.md), [ADR 0021](0021-os-menus-e-o-menu-da-tab.md), [ADR 0023](0023-os-atallos-de-teclado.md), [ADR 0029](0029-o-titulo-da-tab-e-os-titulos-gardados.md)

## Contexto

- **Quen o vai usar.** Renomear unha tab (a subfase 3.3), renomear un grupo (a 6.3) e o filtro do explorador (a 5.3) necesitan o mesmo: unha liña de texto que o usuario escribe e edita. meiga non ten XAML nin controis de Windows na súa interface: debuxa ela mesma a barra de tabs e os menús, na CPU ([ADR 0020](0020-a-barra-de-tabs.md) e [ADR 0021](0021-os-menus-e-o-menu-da-tab.md)), e cada parte vai en dúas metades: un modelo sen Windows en `meiga-core::ui`, que se proba sen xanela, e a vista no programa, que o debuxa e lle pasa o teclado e o rato.
- **O texto da interface son celas.** A barra e os menús debuxan o texto en celas da grella, coas mesmas fontes ca o terminal ([ADR 0010](0010-texto-con-directwrite.md)): cada carácter ocupa unha cela, ou dúas se é ancho, segundo `unicode-width`, e os de ancho cero, coma un acento combinado, ningunha. Xuntar os grafemas ao debuxar, coma unha familia de emoji ou unha bandeira, chega na 8.1.
- **As teclas chegan enteiras** ([ADR 0014](0014-o-teclado.md)): a tecla, o carácter que escribe, en UTF-16 (os de fóra do plano básico, coma a maioría dos emoji, en dous eventos, unha metade en cada un) e as teclas modificadoras. Unha tecla morta que non se xunta coa seguinte chega coma un carácter máis con Retroceso, Intro, Tab ou Esc; o carácter dun código de Alt e o teclado numérico chega ao soltar Alt; e o texto dun IME ou do panel de emoji, coma caracteres sen tecla.
- **As referencias,** polo seu código e pola súa documentación:

| | O cursor | Ctrl+→ | Desfacer | Pegar varias liñas | Lonxitude máxima |
|---|---|---|---|---|---|
| Chrome 145 en Windows (`views::Textfield`) | Por grafemas; Retroceso borra un punto de código, e Supr un grafema | Ao principio da palabra seguinte (ICU) | Ctrl+Z, Ctrl+Y, Ctrl+Maiús+Z e Alt+Retroceso; o escrito seguido, nun paso | Recorta os espazos dos extremos e deixa os saltos de liña do medio | Ningunha |
| Windows Terminal 1.25 (renomear a tab, un `TextBox` de XAML) | — | — | — | — | 1024, que ao pegar non conta |
| Control `EDIT` de Win32, dunha liña | — | Ata os espazos | Un nivel | Sen documentar | 32 767 |
| WezTerm, Kitty e o buscador de Alacritty | WezTerm, por grafemas (os antigos, *legacy*); Kitty, por celas; Alacritty, por caracteres | WezTerm, con Alt; Kitty, ao final da palabra; Alacritty, ningún | Sen atallo | WezTerm, tal cal; Kitty, en varias liñas; Alacritty, sen os saltos | — |

  O Windows Terminal abre a caixa de renomear co título enteiro seleccionado; Intro confirma, Esc cancela, e perder o foco confirma. Chrome selecciona unha palabra co dobre clic e todo co triplo; e, se se arrastra fóra da caixa, móvea cun temporizador.
- **Os grafemas en Rust.** Os grafemas de Unicode (UAX #29) xuntan unha letra cos seus acentos, un emoji co seu ton de pel, as secuencias de emoji con ZWJ, as parellas de indicadores rexionais (as bandeiras) e os conxuntos dos alfabetos da India. O crate `unicode-segmentation` 1.13.3, dos mesmos autores ca `unicode-width` (MIT ou Apache-2.0, Unicode 17), dáos; medido en `meiga.exe`, as súas táboas engaden uns 29,7 KB cos grafemas, e uns 53,8 KB cos grafemas e as palabras de UAX #29, cando o programa usa o campo.

## Decisión

1. **O campo enteiro, sen xanela, en `meiga-core::ui::field`** (decidido contigo): o texto, o cursor e a selección, os grafemas e as palabras, as teclas, desfacer, o límite, o pegado, o rato en celas e o desprazamento horizontal, con probas exhaustivas; e un exemplo para probalo a man na consola, `cargo run -p meiga-win --example field`, coas teclas de verdade e o portapapeis de Windows. A 3.3 debúxao na tab: o cursor que pestanexa, o rato en píxeles e o portapapeis da xanela.
2. **O cursor para entre os grafemas de UAX #29, con `unicode-segmentation`** (decidido contigo), coma en Chrome: nunca entre unha letra e o seu acento, nin dentro dunha bandeira ou dun emoji con ton de pel. Retroceso borra o último punto de código antes del, e Supr o grafema seguinte, coma Chrome en Windows: un `é` escrito coma `e` e un acento perde só o acento. A dependencia nova cumpre a política de licenzas ([ADR 0006](0006-licenzas-e-dependencias.md)).
3. **As palabras, con regras propias** (decidido contigo): as letras, os números e `_` fan palabras; cada grupo doutros signos, coma `.`, `\`, `-` ou un emoji, é outra palabra; e os espazos sepáranas. Ctrl+→ vai ao principio da palabra seguinte, e Ctrl+← ao da anterior, coma en Windows e en Chrome en Windows; Ctrl+Retroceso e Ctrl+Supr borran ata alí, ou a selección, se a hai. O dobre clic selecciona a palabra, o grupo de signos ou o de espazos. Así, `main.rs` son tres palabras.
4. **Desfacer e refacer, coma Chrome** (decidido contigo): Ctrl+Z e Alt+Retroceso desfán; Ctrl+Y, Ctrl+Maiús+Z e Alt+Maiús+Retroceso refán. O escrito seguido é un paso, tamén enriba dunha selección; os Retrocesos seguidos, outro; e os Supr seguidos, outro. Pegar, cortar e borrar unha selección ou unha palabra son un paso cada un. Calquera movemento do cursor ou da selección pecha o paso. Desfacer volta pór a selección que había. Gárdanse ata 100 pasos.
5. **Pegar nunha liña** (decidido contigo): fóra os espazos e os saltos de liña dos extremos, coma en Chrome; cada salto de liña do medio (CR LF, CR, LF, VT, FF, NEL, LS ou PS) e cada tabulador, un espazo; e os outros caracteres de control, fóra. Se só había espazos, pega un espazo; e un portapapeis sen texto, ou só con caracteres de control, non pega nada.
6. **Un límite, en unidades de UTF-16** (decidido contigo), coma as conta Windows, que pon cada uso do campo: o que non cabe non se escribe, e ao pegar entra o que colla, sen partir un grafema. A 3.3 porá 1000, coma os títulos ([ADR 0029](0029-o-titulo-da-tab-e-os-titulos-gardados.md), decisión 6).
7. **O resto, coma Chrome en Windows** (presentado contigo):
   - ← e → moven o cursor un grafema; Inicio e Fin, ao principio e ao final; con Maiús, seleccionan; e Maiús+↑ e Maiús+↓, ata os extremos. Cunha selección, ← e → sen Maiús van ao seu bordo, e Ctrl+← e Ctrl+→ saltan desde el.
   - Ctrl+A selecciona todo; Ctrl+C e Ctrl+Insert copian; Ctrl+X e Maiús+Supr cortan; e Ctrl+V e Maiús+Insert pegan. Sen selección, copiar e cortar non fan nada, e a tecla non chega a ninguén.
   - Os caracteres con AltGr escríbense, tamén con calquera Ctrl e calquera Alt, que Windows toma coma AltGr; os de Alt só, non, que son dos menús da xanela; e os de control, nunca. O código de Alt e o teclado numérico escríbese ao soltar Alt; unha tecla morta que non se xunta escribe o seu acento; e as dúas metades dun carácter de fóra do plano básico xúntanse. Unha tecla que se repite máis rápido do que a colle a xanela fai o seu traballo tantas veces coma se repetiu; copiar, cortar e pegar, unha vez, porque quen usa o campo le o portapapeis unha vez por tecla.
   - Intro, Esc, Tab, ↑, ↓ e as teclas sen carácter non son do campo: decide quen o usa (o filtro do explorador moverase pola lista con ↑ e ↓).
   - O rato, en celas: un clic leva o cursor ao lado máis preto (nun carácter ancho, a primeira cela vai antes del, e a segunda despois); Maiús+clic move o final da selección; arrastrar selecciona, e fóra do campo mostra máis texto por ese lado; o dobre clic selecciona unha palabra, e arrastrar despois, palabras enteiras; e o triplo, todo. Un cambio do texto mentres o botón está abaixo remata o que selecciona o arrastre.
   - Se o texto non cabe, o campo mostra a parte onde está o cursor, e móvea o xusto para velo: ao final dun texto longo, o cursor ten unha cela libre. Nunca mostra media letra ancha á esquerda, e unha ancha que corta o lado dereito non se mostra.

## Alternativas consideradas

| Alternativa | Por que se descarta |
|---|---|
| Na 3.2, tamén a vista: o debuxo, o cursor que pestanexa e o rato en píxeles | Decidido contigo: na 3.3, co seu primeiro uso. Sen ninguén que a use, sería código sen usar no binario, e ningunha comprobación con xanela a vería. |
| Un exemplo con ordes escritas, coma o das tabs da 2.1, ou ningún | Decidido contigo: un interactivo, coas teclas de verdade (as teclas mortas, AltGr, os códigos de Alt, o panel de emoji) e o portapapeis de Windows. |
| Un control `EDIT` de Win32 ou un `TextBox` de XAML | meiga debuxa ela mesma a súa interface, sen XAML ([ADR 0002](0002-windows-e-renderizado-cpu.md)); un control de Windows sería unha xanela fillo, con outras fontes e o seu propio debuxo, fóra dos danos e dos fotogramas de meiga. |
| Grafemas con regras propias: os caracteres de ancho cero pegados ao anterior, e xuntos o ZWJ, as bandeiras e os tons de pel | Decidido contigo: UAX #29, con `unicode-segmentation`. As propias son aproximadas nos alfabetos da India e no coreano, e a 8.1 necesitará os grafemas exactos para debuxalos. |
| Mover o cursor por puntos de código, coma o buscador de Alacritty | Decidido contigo: por grafemas. O cursor podería quedar entre unha letra e o seu acento, ou dentro dunha bandeira. |
| As palabras de UAX #29, coma Chrome | Decidido contigo: regras propias. Custan uns 24 KB máis, e `main.rs`, `can't`, `3.14` ou `example.com` serían unha soa palabra: nun filtro, Ctrl+Retroceso borraría o nome enteiro dun arquivo. |
| As palabras do dobre clic do terminal ([ADR 0015](0015-o-rato-a-seleccion-e-o-portapapeis.md)) | Decidido contigo: regras propias. Alí unha ruta ou un enderezo enteiros son unha palabra, que no terminal se quere copiar enteira; nun nome ou nun filtro, móvese por partes. |
| Un só nivel de desfacer, coma o control `EDIT`, ou ningún | Decidido contigo: coma Chrome. Esc desbotará a edición enteira (3.3), pero o filtro do explorador non ten Esc que desfaga un cambio. |
| Pegar só a primeira liña, ou coma Chrome tal cal | Decidido contigo: nunha liña, con espazos. Pegar só a primeira perdería o resto sen avisar; e os saltos de liña do medio, que meiga non debuxa, quedarían invisíbeis. |
| Sen límite, coma Chrome | Decidido contigo: un límite. Un nome non necesita máis ca un título, e un portapapeis enorme non debe encher a memoria dunha tab. |

## Consecuencias

**Positivas:**

- Un só campo para os tres usos, que se proba enteiro sen xanela: cada tecla, o rato, o desprazamento, os grafemas, as palabras, o desfacer, o pegado e o límite; e unha proba con secuencias aleatorias que comproba despois de cada acción o que o campo sempre mantén.
- As teclas son as de Chrome en Windows, e chegan coma as recibe meiga: as teclas mortas, AltGr, os códigos de Alt, o texto dun IME e o panel de emoji.
- O campo non traballa nos fotogramas: só cando chega unha tecla ou o rato, e as súas mudanzas (`Changes`) dinlle á vista se ten que debuxalo outra vez.
- As táboas de `unicode-segmentation` só chegan a `meiga.exe` cando o programa usa o campo, na 3.3.

**Negativas e riscos:**

- Desde a 3.3, `meiga.exe` medirá uns 29,7 KB máis, e hai unha dependencia máis que manter ao día.
- Ata a 8.1, o debuxo non xunta os grafemas: unha familia de emoji vese coma varios emoji, e o cursor salta por riba de todos á vez; e un acento combinado non ocupa cela.
- O campo non mostra a composición dun IME, coma o terminal: o IME móstraa na súa propia xanela, e o campo recibe o resultado.
- Sen texto de dereita a esquerda: o campo debuxa os caracteres na orde en que están.
- Na consola, o terminal colle algunhas teclas do campo para si, coma Ctrl+V: por iso o exemplo tamén pega con F5.
- `meiga.exe` cambia, aínda que non usa o campo: co código novo, a optimización de todo o programa (LTO) decide doutra maneira en tres funcións, que quedan 512 bytes máis curtas. O comportamento é o mesmo, e as probas e as comprobacións con xanela verifícano.

## Referencias

- Chromium 145.0.7632.45:
  - [`RenderTextHarfBuzz::AdjacentWordSelectionModel`](https://github.com/chromium/chromium/blob/145.0.7632.45/ui/gfx/render_text_harfbuzz.cc#L1636-L1670), as palabras (en Windows, ao principio dunha en ambas direccións); e [`PlatformStyle::RangeToDeleteBackwards`](https://github.com/chromium/chromium/blob/145.0.7632.45/ui/views/style/platform_style.cc#L49-L55), o punto de código de Retroceso;
  - `TextfieldModel`: [`Delete`](https://github.com/chromium/chromium/blob/145.0.7632.45/ui/views/controls/textfield/textfield_model.cc#L449-L478), [`Backspace`](https://github.com/chromium/chromium/blob/145.0.7632.45/ui/views/controls/textfield/textfield_model.cc#L480-L508), [`Paste`](https://github.com/chromium/chromium/blob/145.0.7632.45/ui/views/controls/textfield/textfield_model.cc#L665-L695), e como se xuntan os pasos, [escribindo](https://github.com/chromium/chromium/blob/145.0.7632.45/ui/views/controls/textfield/textfield_model.cc#L227-L238) e [borrando](https://github.com/chromium/chromium/blob/145.0.7632.45/ui/views/controls/textfield/textfield_model.cc#L309-L341);
  - `Textfield`: [as teclas](https://github.com/chromium/chromium/blob/145.0.7632.45/ui/views/controls/textfield/textfield.cc#L2483-L2623), [borrar palabras](https://github.com/chromium/chromium/blob/145.0.7632.45/ui/views/controls/textfield/textfield.cc#L2212-L2265) e [os caracteres que escribe](https://github.com/chromium/chromium/blob/145.0.7632.45/ui/views/controls/textfield/textfield.cc#L1708-L1738);
  - [`SelectionController`](https://github.com/chromium/chromium/blob/145.0.7632.45/ui/views/selection_controller.cc#L44-L129), os clics e o arrastre.
- Windows Terminal v1.25.3131.0: [`TabHeaderControl::BeginRename`](https://github.com/microsoft/terminal/blob/v1.25.3131.0/src/cascadia/TerminalApp/TabHeaderControl.cpp#L73-L83), [Intro e Esc](https://github.com/microsoft/terminal/blob/v1.25.3131.0/src/cascadia/TerminalApp/TabHeaderControl.cpp#L39-L52), [perder o foco](https://github.com/microsoft/terminal/blob/v1.25.3131.0/src/cascadia/TerminalApp/TabHeaderControl.cpp#L97-L123) e [`MaxLength`](https://github.com/microsoft/terminal/blob/v1.25.3131.0/src/cascadia/TerminalApp/TabHeaderControl.xaml#L62-L71).
- Microsoft: [`TextBox`](https://learn.microsoft.com/en-us/uwp/api/windows.ui.xaml.controls.textbox?view=winrt-28000) («MaxLength does not restrict the length of pasted text»), [`EM_SETLIMITTEXT`](https://learn.microsoft.com/en-us/windows/win32/controls/em-setlimittext), [`EM_UNDO`](https://learn.microsoft.com/en-us/windows/win32/controls/em-undo) e [os atallos de teclado de Windows](https://support.microsoft.com/en-us/accessibility/windows/keyboard-shortcuts-in-windows).
- [UAX #29, Unicode 17](https://www.unicode.org/reports/tr29/tr29-47.html) e [`unicode-segmentation` 1.13.3](https://docs.rs/unicode-segmentation/1.13.3/unicode_segmentation/).
- [Tecnoloxías › Os grafemas e as palabras dun campo de texto](../tecnoloxias.md#66-os-grafemas-e-as-palabras-dun-campo-de-texto).
- [Código › meiga-core › `ui/field/mod.rs`](../codigo/meiga-core.md#cratesmeiga-coresrcuifieldmodrs) e [Código › meiga-win › `examples/field.rs`](../codigo/meiga-win.md#cratesmeiga-winexamplesfieldrs).
- [Subfases › 3.2 · Campo de texto](../subfases/3.2-campo.md).
