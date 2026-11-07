# ADR 0008 · Temporizador dos fotogramas

- **Estado:** Aceptado
- **Data:** 2026-11-08
- **Relacionados:** [ADR 0005](0005-fios-e-fotogramas.md)

## Contexto

- O [ADR 0005](0005-fios-e-fotogramas.md) decide que hai como moito un fotograma por intervalo: se o intervalo desde o último fotograma xa pasou, o cambio debúxase no momento; se non, un temporizador dunha soa vez debúxao ao final do intervalo. Con `max_fps = "auto"`, o intervalo é de 33,3 ms (30 fps) nas sesións remotas e de 16,7 ms (60 fps) en local.
- O temporizador habitual dunha xanela, `SetTimer`, avisa cunha mensaxe `WM_TIMER`, pero só nos tics do reloxo do sistema: por defecto, 64 por segundo, un cada 15,6 ms. Un temporizador de 33,3 ms dispara no terceiro tic, aos 46,9 ms, así que daría 21 fotogramas por segundo en vez de 30, e uns 32 en vez de 60. Comprobouse cun prototipo antes de escribir o código.
- Pódese subir a resolución do reloxo con `timeBeginPeriod(1)`, pero Microsoft advirte de que o planificador cambia entón de tarefa máis a miúdo e de que a CPU pode non entrar nos modos de aforro de enerxía. Ata Windows 10 2004, ademais, o cambio afectaba a todo o sistema.
- Desde Windows 10 1803 hai **temporizadores de alta resolución**: un *waitable timer* creado con `CREATE_WAITABLE_TIMER_HIGH_RESOLUTION` dispara no momento pedido sen cambiar a resolución do reloxo. É un obxecto do núcleo, polo que un fío pode esperar por el.
- O fío da interface ten que esperar, ao mesmo tempo, as mensaxes da xanela. `MsgWaitForMultipleObjectsEx` fai as dúas cousas: durme ata que chega unha mensaxe ou ata que se sinala un obxecto, como o temporizador.
- Cando o usuario move ou redimensiona a xanela, ou abre o seu menú, Windows executa o seu propio bucle de mensaxes (un **bucle modal**) ata que remata. Ese bucle só espera mensaxes: non sabe nada do temporizador de meiga.

## Decisión

- O fío da interface espera en `MsgWaitForMultipleObjectsEx`, polas mensaxes e polo temporizador dos fotogramas. Antes de esperar, baleira a cola con `PeekMessageW`. Isto concreta a táboa de fíos do [ADR 0005](0005-fios-e-fotogramas.md), que di que o fío da interface espera en `GetMessageW`: sen fotogramas pendentes, as dúas esperas son equivalentes.
- O temporizador dos fotogramas é un *waitable timer* de alta resolución, e só está activo mentres hai un fotograma pendente. Sen cambios, o fío só espera polas mensaxes.
- Durante os bucles modais de Windows, desde `WM_ENTERSIZEMOVE` ata `WM_EXITSIZEMOVE` e desde `WM_ENTERMENULOOP` ata `WM_EXITMENULOOP`, o fotograma pendente prográmase tamén con `SetTimer`, que chega como unha mensaxe máis. O primeiro dos dous avisos debuxa o fotograma, e o outro descártase.

## Alternativas consideradas

| Alternativa | Por que se descarta |
|---|---|
| Só `SetTimer` | 21 fps en vez de 30, e uns 32 en vez de 60: os tics do reloxo atrasan cada fotograma ata 15,6 ms. |
| `SetTimer` con `timeBeginPeriod(1)` | Máis espertares e menos aforro de enerxía en todo o proceso, só para corrixir un temporizador. |
| Un temporizador do grupo de fíos de Windows (`CreateThreadpoolTimer`) que publica unha mensaxe | É preciso e funcionaría tamén nos bucles modais, pero esperta outro fío en cada fotograma. Queda como opción se os bucles modais o piden. |
| Un temporizador multimedia (`timeSetEvent`) | Obsoleto, e avisa desde outro fío. |
| Esperar só o temporizador, ou `Sleep`, no fío da interface | Mentres espera, a xanela non responde ás mensaxes. |

## Consecuencias

**Positivas:**

- Os fotogramas chegan cando toca, cun atraso de menos dun milisegundo: a 30 fps, un cada 33,3 ms.
- Nin se cambia a resolución do reloxo, nin hai fíos novos.
- En repouso, todo segue igual: 0 espertares, porque sen cambios non hai ningún temporizador activo.

**Negativas e riscos:**

- Nos bucles modais, os fotogramas van ao ritmo dos tics do reloxo: un cada 47 ms aproximadamente, uns 21 por segundo en vez de 30. Mitigación: só pasa mentres o usuario move ou redimensiona a xanela; se chega a notarse, o temporizador do grupo de fíos da táboa resólveo.
- Dous temporizadores para un mesmo fotograma, e `CancelWaitableTimer` non borra un aviso que xa chegou. Mitigación: a xanela garda cando vence o fotograma pendente, e só o primeiro aviso o entrega; os demais ignóranse.

## Referencias

- Microsoft Learn: [`CreateWaitableTimerExW`](https://learn.microsoft.com/en-us/windows/win32/api/synchapi/nf-synchapi-createwaitabletimerexw) (`CREATE_WAITABLE_TIMER_HIGH_RESOLUTION`, desde Windows 10 1803), [`MsgWaitForMultipleObjectsEx`](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-msgwaitformultipleobjectsex), [About Timers](https://learn.microsoft.com/en-us/windows/win32/winmsg/about-timers) («a timer's accuracy depends on the system clock rate») e [`timeBeginPeriod`](https://learn.microsoft.com/en-us/windows/win32/api/timeapi/nf-timeapi-timebeginperiod).
- Microsoft Learn: [`WM_ENTERSIZEMOVE`](https://learn.microsoft.com/en-us/windows/win32/winmsg/wm-entersizemove) e [`WM_ENTERMENULOOP`](https://learn.microsoft.com/en-us/windows/win32/menurc/wm-entermenuloop).
- [Tecnoloxías › Temporizadores](../tecnoloxias.md#26-temporizadores-e-a-espera-do-fío-da-interface) e [Código › meiga-win](../codigo/meiga-win.md#cratesmeiga-winsrcwindowrs).
