# ADR 0009 · Que se debuxa cando a xanela non se ve

- **Estado:** Aceptado
- **Data:** 2026-11-08
- **Relacionados:** [ADR 0002](0002-windows-e-renderizado-cpu.md), [ADR 0005](0005-fios-e-fotogramas.md)

## Contexto

- Un dos principios de meiga é non debuxar o que non se ve. O [ADR 0002](0002-windows-e-renderizado-cpu.md) di: «se a xanela está minimizada ou tapada, non se debuxa nada». Ao implementar a presentación, na subfase 1.2, estudouse que se pode saber en cada caso e canto custa sabelo.
- **Minimizada.** Windows avisa con `WM_SIZE` (`SIZE_MINIMIZED`), sen ningún custo. Nese estado, o contido da xanela non está na pantalla.
- **Tapada por outras xanelas.** Desde Windows 8, o compositor de xanelas (DWM) está sempre activo: cada xanela debuxa na súa propia superficie fóra da pantalla, e DWM xúntaas todas na imaxe que se ve. O contido dunha xanela tapada segue vivo (sae nas miniaturas da barra de tarefas, en Alt+Tab e na vista de tarefas), e Windows non lle di á aplicación se se ve ou non. Quen o detecta faino pola súa conta: Chromium ten un fío que recibe os cambios de todas as xanelas do sistema (`SetWinEventHook`), percórreas (`EnumWindows`) e calcula que zonas quedan á vista.
- **Outros casos nos que non se ve nada:** a xanela está noutro escritorio virtual, a sesión está bloqueada ou o Escritorio remoto está desconectado. Saber todo isto require preguntarlle a Windows ou subscribirse aos seus avisos: `WTSRegisterSessionNotification` (en `wtsapi32.dll`) para o bloqueo e a desconexión, e `IVirtualDesktopManager` (COM) ou o atributo `DWMWA_CLOAKED` de DWM (en `dwmapi.dll`) para os escritorios virtuais. Cargar cada unha desas DLL leva uns milisegundos ao arrancar, e meiga non necesita ningunha delas para o resto.
- **Os terminais de referencia.** A consola clásica de Windows (`conhost`) deixa de debuxar coa xanela minimizada, despois de comprobalo con `IsIconic`. O Windows Terminal segue debuxando minimizado: está pendente desde 2019, no *issue* #1989. Ningún dos dous detecta se a xanela está tapada.
- O custo de debuxar sen que se vexa xa está limitado: meiga só presenta o que cambia e, como moito, `max_fps` fotogramas por segundo ([ADR 0005](0005-fios-e-fotogramas.md)).

## Decisión

- **Minimizada:** non se debuxa nada. Nin se compón, nin se presenta, nin queda ningún temporizador activo. Os cambios acumúlanse e, ao restaurala, compónse o *framebuffer* enteiro e píntase a xanela unha vez.
- **Tapada, total ou parcialmente:** debúxase como sempre. meiga non o detecta.
- **Noutro escritorio virtual, coa sesión bloqueada ou co Escritorio remoto desconectado:** decídese na subfase 8.6 (robustez do contorno), medindo o custo dos avisos de Windows.
- Isto concreta o [ADR 0002](0002-windows-e-renderizado-cpu.md) no punto das xanelas tapadas. As tabs en segundo plano seguen como di o [ADR 0005](0005-fios-e-fotogramas.md): só analizan a saída da shell.

## Alternativas consideradas

| Alternativa | Por que se descarta |
|---|---|
| Detectar as xanelas tapadas, como Chromium | Un fío máis, que esperta cada vez que se move calquera xanela do sistema, para aforrar fotogramas que xa son pequenos e limitados. Revisarase na fase 9 se as medicións o xustifican. |
| Detectar xa os escritorios virtuais, o bloqueo e a desconexión | Máis DLL ao arrancar e máis código, nunha subfase centrada na presentación. Encaixa na 8.6, xunto coa reconexión do Escritorio remoto. |
| Non debuxar sen o foco | Unha xanela sen foco pode estar á vista, por exemplo ao lado doutra. |
| Seguir debuxando coa xanela minimizada, como o Windows Terminal | Sería traballo sen ningún resultado á vista, e saber que está minimizada non custa nada. |

## Consecuencias

**Positivas:**

- Coa xanela minimizada, meiga non debuxa nada, aínda que o contido siga cambiando.
- Ao restaurala, un único repintado mostra o estado actual.
- Ao arrancar non se carga ningunha DLL máis.

**Negativas e riscos:**

- Unha xanela tapada segue debuxando: compoñer, presentar e o traballo de DWM. Mitigación: os fotogramas só levan o que cambia, como moito `max_fps` por segundo, e a fase 9 medirao.
- Ata a subfase 8.6, meiga debuxa tamén noutro escritorio virtual, coa sesión bloqueada ou desconectada. Mitigación: o mesmo límite de fotogramas.

## Referencias

- Microsoft Learn: [`WM_SIZE`](https://learn.microsoft.com/en-us/windows/win32/winmsg/wm-size), [Desktop Window Manager](https://learn.microsoft.com/en-us/windows/win32/dwm/dwm-overview) («their drawing is redirected to off-screen surfaces»), [`WTSRegisterSessionNotification`](https://learn.microsoft.com/en-us/windows/win32/api/wtsapi32/nf-wtsapi32-wtsregistersessionnotification), [`IVirtualDesktopManager::IsWindowOnCurrentVirtualDesktop`](https://learn.microsoft.com/en-us/windows/win32/api/shobjidl_core/nf-shobjidl_core-ivirtualdesktopmanager-iswindowoncurrentvirtualdesktop) e [`DWMWINDOWATTRIBUTE`](https://learn.microsoft.com/en-us/windows/win32/api/dwmapi/ne-dwmapi-dwmwindowattribute) (`DWMWA_CLOAKED`).
- Windows Terminal v1.25.2733.0:
  - [`src/interactivity/win32/windowproc.cpp`, liñas 336–348](https://github.com/microsoft/terminal/blob/v1.25.2733.0/src/interactivity/win32/windowproc.cpp#L336-L348): `conhost` marca a xanela como minimizada (`CONSOLE_IS_ICONIC`) e deixa de debuxar;
  - [`src/cascadia/WindowsTerminal/IslandWindow.cpp`, liñas 738–754](https://github.com/microsoft/terminal/blob/v1.25.2733.0/src/cascadia/WindowsTerminal/IslandWindow.cpp#L738-L754): `TODO GH#1989 Stop rendering island content when the app is minimized`;
  - o *issue* [#1989](https://github.com/microsoft/terminal/issues/1989), aberto desde xullo de 2019.
- Chromium 159.0.8149.1, [`ui/aura/native_window_occlusion_tracker_win.h`](https://github.com/chromium/chromium/blob/159.0.8149.1/ui/aura/native_window_occlusion_tracker_win.h#L42-L44): o seguimento das xanelas que tapan as de Chromium, nun fío propio (liña 91), con `EnumWindows` (liña 150) e `SetWinEventHook` (liña 251).
- [Tecnoloxías › Pintar nunha xanela](../tecnoloxias.md#25-pintar-nunha-xanela-wm_paint-e-dwm).
