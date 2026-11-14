# ADR 0005 · Modelo de fíos e fotogramas

- **Estado:** Aceptado; concretado polos [ADR 0008](0008-temporizador-dos-fotogramas.md), [ADR 0011](0011-fin-e-peche-das-sesions.md), [ADR 0013](0013-da-saida-a-pantalla.md) e [ADR 0017](0017-os-fotogramas-por-escritorio-remoto.md)
- **Data:** 2026-11-06
- **Relacionados:** [ADR 0002](0002-windows-e-renderizado-cpu.md), [ADR 0004](0004-conpty-empaquetado.md)

## Contexto

- O primeiro principio de meiga é **cero traballo en repouso**. Un temporizador periódico ou un *polling* fan espertar a CPU aínda que non pase nada, e o custo multiplícase polo número de tabs.
- As shells poden escribir moito e moi rápido: compilacións, `cat` dun arquivo grande, ferramentas de IA que escriben en *streaming*. Se cada escritura provocase un redebuxado, a CPU gastaríase en fotogramas que ninguén chega a ver.
- Pode haber moitas tabs abertas, pero só se ve unha. As demais deben custar o mínimo.
- Un fío bloqueado nunha lectura ou nunha espera non consome CPU: Windows non o planifica ata que ten algo que facer.

## Decisión

**Fíos:**

| Fío | Cantos | Onde espera |
|---|---|---|
| Interface | 1 | `GetMessageW`, ata que chega unha mensaxe. |
| Lector | 1 por tab coa shell en marcha | `ReadFile` da saída do ConPTY, en bloques de 64 KiB. |
| Escritor | 1 por tab coa shell en marcha | Unha canle coa entrada pendente de enviar á shell. |

- O lector analiza a saída e actualiza a grella baixo un *lock* curto. Despois avisa á interface con `PostMessage(WM_APP_WAKE)`, **só** se a tab se ve e non hai xa un aviso pendente.
- As tabs en segundo plano só analizan a saída. Á interface só lle avisan nas transicións: primeira actividade, cambio de título, campá ou fin do proceso. O seu lector baixa de prioridade.
- O fin do proceso detéctase con `RegisterWaitForSingleObject`, no grupo de fíos de Windows, sen fío propio.
- `ClosePseudoConsole` chámase fóra do fío da interface, porque pode bloquearse ata que se baleira a tubaxe.

**Fotogramas:**

- Como moito un fotograma por intervalo (`max_fps`).
- Se xa pasou o intervalo desde o último fotograma, o cambio debúxase no momento: a primeira tecla despois dunha pausa vese sen atraso. Se non, prográmase un temporizador dunha soa vez para o final do intervalo.
- Sen cambios non hai ningún temporizador activo.
- Un fotograma: *lock* breve, copia só das filas cambiadas, liberación do *lock*, e composición e presentación fóra del.
- `max_fps = "auto"`: a proposta inicial é 30 fps nas sesións remotas e 60 en local. O valor final decídese con medicións na subfase 1.10.

## Alternativas consideradas

| Alternativa | Por que se descarta |
|---|---|
| Un *runtime* asíncrono (por exemplo, tokio) | Engade un grupo de fíos e un planificador propios, máis memoria e máis complexidade, para un problema que os fíos bloqueados xa resolven. |
| Un único fío de E/S para todas as tabs (E/S solapada, IOCP) | Menos fíos, pero código bastante máis complexo, e un fío bloqueado só custa memoria. Pódese revisar se as medicións o xustifican. |
| Un temporizador fixo, por exemplo a 60 Hz, que debuxa se hai cambios | Esperta 60 veces por segundo aínda que non pase nada: incumpre o primeiro principio. |
| Debuxar en cada escritura da shell | O custo dependería do número de escrituras, non do que se ve. |
| Analizar a saída no fío da interface | Unha tab con moita saída bloquearía a interface de todas. |

## Consecuencias

**Positivas:**

- En repouso, 0 espertares por segundo.
- Custo limitado con saída masiva: como moito `max_fps` fotogramas por segundo, aínda que a shell escriba miles de veces.
- Latencia mínima ao escribir.
- As tabs en segundo plano non debuxan nada.

**Negativas e riscos:**

- O lector e a interface comparten a grella. Mitigación: seccións críticas curtas e copia só das filas cambiadas.
- Dous fíos por tab coa shell en marcha, coa memoria das súas pilas. Mitigación: as tabs restauradas non arrancan a shell ata que se activan (subfase 7.2).
- A cifra de 30 ou 60 fps é unha hipótese. Mitigación: medila na subfase 1.10 e rexistrar a decisión.

## Referencias

- [Arquitectura › Fluxo de datos](../arquitectura.md#3-fluxo-de-datos) e [Arquitectura › Fíos](../arquitectura.md#4-fíos).
- Microsoft Learn: [`GetMessageW`](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-getmessagew), [`PostMessageW`](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-postmessagew) e [`RegisterWaitForSingleObject`](https://learn.microsoft.com/en-us/windows/win32/api/winbase/nf-winbase-registerwaitforsingleobject).
