# ADR 0017 · Os fotogramas por Escritorio remoto

- **Estado:** Aceptado
- **Data:** 2026-11-14
- **Relacionados:** [ADR 0002](0002-windows-e-renderizado-cpu.md), [ADR 0005](0005-fios-e-fotogramas.md), [ADR 0008](0008-temporizador-dos-fotogramas.md)

## Contexto

- O [ADR 0005](0005-fios-e-fotogramas.md) decide que hai como moito un fotograma por intervalo (`max_fps`), e propón para `max_fps = "auto"` 30 fotogramas por segundo nas sesións de Escritorio remoto e 60 en local. A cifra era unha hipótese, que se decide coas medicións da porta do MVP, a [subfase 1.10](../subfases/1.10-porta.md).
- **O límite do Escritorio remoto.** Por defecto, o Escritorio remoto non envía ao cliente máis de 30 fotogramas por segundo. O valor `DWMFRAMEINTERVAL` do rexistro do equipo remoto sobe ese límite a 60 ([Microsoft Learn](https://learn.microsoft.com/en-us/troubleshoot/windows-server/remote/frame-rate-limited-to-30-fps)). Na máquina na que se desenvolve meiga está posto, así que alí os 60 fotogramas por segundo chegarían ao cliente.
- **Onde se paga un fotograma.** Nunha máquina sen GPU usada por Escritorio remoto, cada fotograma custa en tres sitios: en meiga, que o compón e o presenta na CPU ([ADR 0002](0002-windows-e-renderizado-cpu.md)); no DWM, que compón o escritorio, tamén na CPU; e no servizo do Escritorio remoto, que codifica e envía os píxeles que cambiaron.
- **Cando importa o intervalo.** meiga debuxa no momento o primeiro cambio despois dunha pausa ([ADR 0005](0005-fios-e-fotogramas.md)): o intervalo só limita os cambios que chegan máis rápido ca el.
- **As medicións** (subfase 1.10). meiga a 30 e a 60 fotogramas por segundo por Escritorio remoto, maximizado e alternando as dúas versións. A de 60 foi unha compilación temporal, fóra do repositorio:
  - **O *spinner* e a IA:** os mesmos fotogramas a 30 e a 60. Os dous escenarios cambian a pantalla 10 veces por segundo, e o rexistro dos fotogramas (`--trace-frames`) mostra os mesmos 10 por segundo, todos debuxados no momento. Ao escribir pasa o mesmo: as teclas adoitan chegar máis amodo ca o intervalo, e cada unha debúxase no momento.
  - **A saída masiva** (`bigfile`, 50 MiB): imprímese no mesmo tempo, e a 60 meiga gasta arredor dun 20 % máis de CPU: o que máis lle custa é ler a saída, e os fotogramas, en menos dun segundo, son poucos. No DWM e no servizo do Escritorio remoto, a diferenza non se distingue do ruído.
  - **A xanela enteira cambiando en cada fotograma, sen parar** (`measure-frames.ps1`), coma cando o texto se despraza durante moito tempo: a 60, meiga gasta arredor do dobre; o DWM traballa entre un 20 % e un 55 % máis ca a 30; e o servizo do Escritorio remoto, co texto, case o dobre, e cunha soa cor, o mesmo.
  - **Un cadrado pequeno que se move sen parar:** a 60 custa pouco máis.

## Decisión

- `max_fps = "auto"` segue sendo **30 fotogramas por segundo nas sesións de Escritorio remoto** (33,3 ms) **e 60 en local** (16,7 ms). O código non cambia: `auto_interval` en `meiga-core` xa o fai.
- meiga non segue o límite da sesión: aínda que o Escritorio remoto permita 60, como na máquina de desenvolvemento, usa 30.
- Cando chegue a configuración completa (subfase 8.4), unha opción `max_fps` (`auto`, `30` ou `60`) permitirá escoller 60 a quen prefira un desprazamento máis suave.
- En local, 60, coma propuña o ADR 0005. Non se mediu nesta subfase, porque a máquina de desenvolvemento só se usa por Escritorio remoto; na CI, que corre nunha sesión local, os modos de medición chegan a uns 59 fotogramas por segundo.

## Alternativas consideradas

| Alternativa | Por que se descarta |
|---|---|
| 60 fotogramas por segundo por Escritorio remoto | Cando a xanela enteira cambia en cada fotograma, coma cando o texto se despraza durante moito tempo, meiga gasta arredor do dobre, o DWM entre un 20 % e un 55 % máis, e o servizo do Escritorio remoto, case o dobre; coa saída masiva, meiga gasta un 20 % máis. Escribir, os *spinners* e as ferramentas de IA non ganan nada. E por defecto o Escritorio remoto non envía máis de 30: os fotogramas de máis non chegarían ao cliente. |
| Seguir o límite da sesión: 60 se o Escritorio remoto o permite | Ese custo, ata o dobre ao desprazar o texto, pagaríase precisamente nas máquinas que o permiten, como a de desenvolvemento, para gañar só suavidade ao desprazar. |
| Adaptativo: 60 cos fotogramas pequenos e 30 cos grandes | Os fotogramas pequenos xa veñen de cambios máis lentos ca o intervalo, como escribir ou un *spinner*, e debúxanse no momento. Os que gañarían suavidade, os do desprazamento, son os grandes. Máis código para case nada. |
| Menos de 30, por exemplo 20 | Aforraría máis ao desprazar o texto, pero o desprazamento e os arrastres irían máis a saltos ca o resto do escritorio, que o Escritorio remoto envía a 30. |

## Consecuencias

**Positivas:**

- Cando a xanela enteira cambia en cada fotograma, coma cando o texto se despraza durante moito tempo, meiga gasta arredor da metade ca a 60, e o DWM e o servizo do Escritorio remoto, menos. Coa saída masiva, a diferenza é menor: a 60, meiga gastaba un 20 % máis.
- Escribir, os *spinners* e as ferramentas de IA vense igual ca a 60: cada cambio debúxase no momento.
- Non hai nada que configurar nin que detectar: só se mira se a sesión é remota, coma ata agora.

**Negativas e riscos:**

- Nas máquinas que permiten 60 fotogramas por segundo, o desprazamento e a saída continua vense menos suaves ca noutros programas. Mitigación: a opción `max_fps` da 8.4.
- Mesmo a 30, cando o texto se despraza, cada fotograma presenta a xanela enteira: na porta, coa saída masiva, o DWM traballou máis con meiga ca co Windows Terminal, aínda que o servizo do Escritorio remoto traballou menos. Mitigación: a subfase 9.2, que despraza o *framebuffer* e só debuxa as filas novas.
- As medicións son dunha soa máquina. Mitigación: as fases seguintes miden outra vez, e se os custos cambian, un ADR novo.

## Referencias

- Microsoft Learn: [Frame rate is limited to 30 FPS in remote sessions](https://learn.microsoft.com/en-us/troubleshoot/windows-server/remote/frame-rate-limited-to-30-fps).
- [Subfase 1.10 · Porta do MVP](../subfases/1.10-porta.md): como se mediu.
- [Código › meiga-core › `pacing.rs`](../codigo/meiga-core.md#cratesmeiga-coresrcpacingrs): `auto_interval`.
- [Código › medición › `measure-frames.ps1`](../codigo/perf.md#toolsperfmeasure-framesps1) e [Código › escenarios de carga › `run-scenario.ps1`](../codigo/escenarios.md#run-scenariops1): as ferramentas.
