# Documentación de meiga

Esta documentación explica en detalle **como está feito meiga**: a arquitectura, as tecnoloxías que usa e o código, arquivo por arquivo. Actualízase en cada PR, e ningún cambio se integra sen a súa documentación.

> O código, os comentarios, as mensaxes dos commits, o README e as descricións dos PR están en inglés. Esta documentación está en galego.

## Por onde empezar

1. [Arquitectura](arquitectura.md): as pezas do programa, como se relacionan e por que están separadas así.
2. [Tecnoloxías](tecnoloxias.md): que é cada tecnoloxía e ferramenta, por que a usamos, como está configurada e como preparar o contorno de desenvolvemento.
3. [Código](codigo/README.md): explicación detallada de cada arquivo.
4. [Subfases](subfases/README.md): o diario do proxecto. Que se fixo en cada subfase e en cada mantemento, por que e como probalo.
5. [ADR](adr/README.md): as decisións de arquitectura, unha por documento, co seu contexto, as alternativas descartadas e as consecuencias.

## Convencións desta documentación

- Os termos técnicos sen tradución habitual mantéñense en inglés e en cursiva a primeira vez que aparecen (*crate*, *workspace*, *lint*…).
- Cada documento indica a subfase do estado que describe.
- As partes marcadas como *previsto* describen deseño acordado que aínda non está implementado.
- Os diagramas están feitos con [Mermaid](https://mermaid.js.org/), unha linguaxe de texto que GitHub debuxa directamente.
- Non se inclúen datos de ningunha máquina concreta (medicións, rutas persoais…).
