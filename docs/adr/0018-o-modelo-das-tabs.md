# ADR 0018 · O modelo das tabs

- **Estado:** Aceptado
- **Data:** 2026-11-14
- **Relacionados:** [ADR 0005](0005-fios-e-fotogramas.md), [ADR 0011](0011-fin-e-peche-das-sesions.md)

## Contexto

- A fase 2 trae as tabs. Todo o que vén despois necesita as mesmas regras: que tabs ten unha xanela, en que orde, cal está activa, e que pasa ao abrir, pechar, mover ou activar unha. A subfase 2.1 escribe só esas regras, en `meiga-core` e sen interface, para probalas soas antes de que as use ningunha outra peza.
- O que virá despois, e o que lle pide ao modelo:
  - **2.2, varias sesións:** cada tab terá a súa sesión, cos seus dous fíos, e a súa vista. O fío lector dunha tab avisará á xanela de que esa tab cambiou; ao cambiar de tab haberá un único redebuxado completo, e as tabs que non se ven só procesarán a saída dos seus programas ([ADR 0005](0005-fios-e-fotogramas.md)). A xanela ten que saber que tab é cada unha aínda que se movan ou se pechen outras, e cando cambia a activa.
  - **2.3 a 2.7:** a barra debuxa as tabs na súa orde e marca a activa; os atallos activan a seguinte e a anterior; e pechar a última tab pecha a xanela.
  - **6.1 e 6.4:** os grupos, coas súas tabs sempre seguidas, e arrastrar as tabs.
  - **7.1 e 7.2:** gardar a orde e a tab activa, e restaurar as tabs sen lanzar as súas shells ata que se activan.
  - **F1, a vista dividida:** ver 2 ou 4 tabs á vez, unha delas co teclado.
  - **F6, varias xanelas:** mover unha tab a outra xanela.
- **Chrome,** ao pechar a tab activa, activa a seguinte, a da súa dereita, ou a anterior se era a última, salvo que prefira outra polas ligazóns desde as que se abriron as tabs ou polo seu grupo. Ctrl+T abre a tab nova ao final, e Ctrl+Tab percorre as tabs en orde e dá a volta.
- **O Windows Terminal,** ao pechar a tab enfocada, volta á última que se usou; abre as tabs novas ao final (`newTabPosition`, que por defecto é `afterLastTab`); e a seguinte e a anterior tamén dan a volta.
- Unha xanela terá decenas de tabs, non miles: a subfase 2.8 proba con 30.

## Decisión

1. **Un modelo puro, en `meiga-core`:** o módulo `model`, cunha `Window<T>` que garda as súas tabs nun vector, na orde da barra, e a posición da activa. Sen APIs de Windows, sen fíos e sen `unsafe`, coma o resto do crate.
2. **Cada tab leva o seu contido** (`Tab<T>`): o que o programa garde para ela, desde a 2.2 a súa sesión e a súa vista. O tipo escólleo o programa, e as probas usan textos e números. Así, a orde das tabs vive nun só sitio, e o programa non ten que manter ao lado outra lista que poida discordar dela.
3. **Identificadores que nunca se repiten:** cada tab recibe un `TabId`, e cada xanela un `WindowId`, números de 64 bits que saen dun contador atómico para todo o programa, coma os `ThreadId` dos fíos de Rust. Unha tab pechada non devolve o seu, e dúas xanelas nunca dan o mesmo. Así, un aviso que chegue tarde desde o fío lector dunha tab pechada non se pode confundir con outra tab, e unha tab podería pasar a outra xanela co seu identificador (F6). O contido dunha tab pode facerse sabendo xa o seu identificador (`try_new_with` e `try_open_with`): desde a 2.2, a sesión, cuxo fío lector o enviará cos seus avisos; se facelo falla, a xanela queda como estaba. E un identificador pode voltar facerse co seu número (`TabId::from_raw`), o que chegue nunha mensaxe: só o modelo dá números novos e pon tabs na xanela, así que un número inventado non pode crear unha tab nin facer que dúas compartan un, e se non é dunha tab da xanela non atopa nada.
4. **Unha xanela sempre ten unha tab activa,** e nunca queda sen tabs. Pechar a única tab non a quita: a resposta, `Close::LastTab`, dille ao programa que peche a xanela (2.7). Pechar calquera outra devolve o seu contido, para que o programa peche a súa sesión. `Close` leva `#[must_use]`: o compilador avisa se o programa ignora a resposta, tamén despois dun `?`.
5. **Ao pechar a tab activa, actívase a da dereita** ou, se era a última, a da esquerda, coma en Chrome (decidido contigo). Pechar outra tab non cambia a activa.
6. **As tabs novas ábrense ao final** (decidido contigo), coma en Chrome con Ctrl+T e no Windows Terminal, e **abrir non activa:** o botón + activará a súa tab, pero a restauración da sesión (7.2) abre as tabs sen activalas.
7. **Mover unha tab** a unha posición corre un posto as que quedan entre a súa posición de antes e a nova, e a tab activa segue sendo a mesma.
8. **Activar** unha tab, ou a seguinte ou a anterior, que dan a volta nos extremos, coma Ctrl+Tab en Chrome e no Windows Terminal. A resposta di de que tab a que tab cambiou (`Switch`), ou nada se xa estaba activa: o programa saberá cando redebuxar todo e a que fíos lectores cambiarlles a prioridade (2.2).
9. **A tab activa, a enfocada e as visíbeis son tres preguntas:** a activa é a que marca a barra; a enfocada, a que recibe o teclado; e as visíbeis, as que se debuxan. Hoxe son a mesma tab, e o modelo só garda a activa: as outras dúas respóstanse con ela. O resto do programa preguntará cada unha polo seu nome, así que coa vista dividida esas preguntas seguirán valendo: o modelo gardará entón o *layout* e o foco, e o programa debuxará cada tab visíbel no seu rectángulo, dará a cada ConPTY o seu tamaño e levará o rato á tab que estea debaixo.
10. **Sen pánicos:** o que non se pode facer, cunha tab que non está na xanela ou cunha posición despois da última, devolve un erro (`model::Error`), e a xanela queda como estaba.
11. **A activa gárdase pola súa posición:** o programa consultaraa en cada tecla e en cada fotograma, e así é un acceso directo. Abrir, pechar e mover axustan esa posición. Buscar unha tab polo seu identificador percorre o vector: con decenas de tabs, uns nanosegundos, e sen ningún mapa.

## Alternativas consideradas

| Alternativa | Por que se descarta |
|---|---|
| Un modelo só cos identificadores, e as sesións nun mapa á parte | Dúas estruturas que hai que manter iguais, e unha busca no mapa en cada acceso á tab activa. |
| Identificar cada tab pola súa posición | A posición cambia ao abrir, pechar e mover: o aviso dun fío lector chegaría a outra tab. |
| Un contador de tabs en cada xanela | Dúas xanelas darían os mesmos números, e mover unha tab a outra xanela (F6) obrigaría a cambiarllo. |
| Reservar un identificador e abrir despois a tab con el | Dúas tabs poderían acabar co mesmo identificador. Con `try_open_with`, só o modelo dá os identificadores e pon as tabs. |
| Abrir a tab cun contido provisional, e poñer a sesión despois | O contido tería que admitir un estado provisional, e a sesión necesita o identificador ao nacer. |
| Un `HashMap` coas tabs, ademais da orde | Con decenas de tabs, percorrer un vector contiguo é igual de rápido ou máis, e serían dúas estruturas que coordinar. |
| Gardar a activa polo seu identificador | Cada consulta da activa, en cada tecla e en cada fotograma, percorrería o vector, e habería que tratar o caso imposíbel de que non estea. |
| Gardar xa o foco á parte da activa | Hoxe sempre sería igual ca a activa: unha regra máis que manter, sen ningún comportamento que probar. Chega coa vista dividida, que lle dá sentido. |
| Permitir unha xanela sen tabs | Cada consulta da activa tería que contemplar que non hai ningunha. |
| Ao pechar a activa, voltar á última usada, coma o Windows Terminal, ou activar a da esquerda | Decidido contigo: coma en Chrome. |
| Abrir as tabs novas á dereita da activa | Decidido contigo: ao final. Duplicar unha tab (2.4) abrirá a copia á súa dereita. |
| Que abrir active sempre a tab nova | A restauración da sesión (7.2) abre as tabs sen activalas. |
| `proptest` para as probas ao chou | Unha dependencia de desenvolvemento, con moitas outras detrás, para o que xa fai unha secuencia fixa *xorshift*, coma noutras probas de meiga. |

## Consecuencias

**Positivas:**

- As regras das tabs próbanse soas e en milisegundos, sen xanelas nin procesos: unha proba compara 10 000 cambios ao chou cunha lista simple, e vinte máis comproban cada regra.
- `meiga.exe` non cambia: segue cunha sesión ata a 2.2, e o ligador deixa fóra o modelo, que aínda non usa.
- A vista dividida (F1) e as varias xanelas (F6) teñen o sitio preparado no modelo, sen pagalo hoxe.

**Negativas e riscos:**

- Os grupos (6.1) engadirán unha regra que pechar e mover terán que respectar: as tabs dun grupo van sempre seguidas.
- Ao pechar unha tab, o programa recibe o seu contido. Se o descarta sen máis, Rust destrúeo igual, e desde a 1.4 destruír unha sesión pecha a súa pseudoconsola ([ADR 0011](0011-fin-e-peche-das-sesions.md)).
- Buscar unha tab polo seu identificador custa máis canto máis tabs hai: con centos, aínda serían microsegundos.
- Os identificadores só valen mentres meiga corre: `session.toml` (7.1) gardará a orde das tabs, e ao restaurala as tabs recibirán identificadores novos.

## Referencias

- Chromium: [`TabStripModel::DetermineNewSelectedIndex`](https://chromium.googlesource.com/chromium/src/+/main/chrome/browser/ui/tabs/tab_strip_model.cc), a tab que se activa ao pechar a activa («Otherwise, prefer picking the tab after the last tab in the block»), en `tab_strip_model.cc`.
- Windows Terminal v1.25.3131.0: [a última tab usada, ao pechar a enfocada](https://github.com/microsoft/terminal/blob/v1.25.3131.0/src/cascadia/TerminalApp/TabManagement.cpp#L478-L488), e [a seguinte e a anterior, que dan a volta](https://github.com/microsoft/terminal/blob/v1.25.3131.0/src/cascadia/TerminalApp/TabManagement.cpp#L508-L514), en `TabManagement.cpp`; e [`newTabPosition`](https://github.com/microsoft/terminal/blob/v1.25.3131.0/doc/cascadia/profiles.schema.json#L2709-L2717), no esquema da súa configuración.
- Rust: [`std::thread::ThreadId`](https://doc.rust-lang.org/std/thread/struct.ThreadId.html), [`AtomicU64::fetch_add`](https://doc.rust-lang.org/std/sync/atomic/struct.AtomicU64.html#method.fetch_add), [`Ordering::Relaxed`](https://doc.rust-lang.org/std/sync/atomic/enum.Ordering.html#variant.Relaxed) e [`slice::rotate_left`](https://doc.rust-lang.org/std/primitive.slice.html#method.rotate_left).
- [Tecnoloxías › Xenéricos e contadores atómicos](../tecnoloxias.md#53-xenéricos-e-contadores-atómicos).
- [Código › meiga-core › `model/mod.rs`](../codigo/meiga-core.md#cratesmeiga-coresrcmodelmodrs) e [› `model/window.rs`](../codigo/meiga-core.md#cratesmeiga-coresrcmodelwindowrs).
