# Subfases

O diario do proxecto. meiga avanza en fases pequenas, e cada fase divídese en **subfases** centradas nun único punto. Cada subfase entrégase nun PR que inclúe o código, a súa documentación e un documento eiquí con:

- o obxectivo;
- que se fixo e por que;
- as alternativas descartadas;
- **como comprobar que todo está ben**, paso a paso e co resultado esperado de cada paso (desde a subfase 0.4);
- os resultados obtidos;
- o que queda pendente.

| Subfase | Título | Entrega | Documento |
|---|---|---|---|
| 0.1 | Ferramentas | Local, sen cambios no repositorio | As ferramentas descríbense en [Tecnoloxías › Preparar o contorno](../tecnoloxias.md#14-preparar-o-contorno-de-desenvolvemento). |
| 0.2 | Repositorio público | Commit inicial directo a `main` | [0.2-repositorio.md](0.2-repositorio.md) |
| 0.3 | Esqueleto do workspace | [PR #1](https://github.com/NewHouses/meiga/pull/1) | [0.3-workspace.md](0.3-workspace.md) |
| 0.4 | Integración continua | [PR #2](https://github.com/NewHouses/meiga/pull/2) | [0.4-ci.md](0.4-ci.md) |
| 0.5 | ADR | [PR #3](https://github.com/NewHouses/meiga/pull/3) | [0.5-adr.md](0.5-adr.md) |
| 0.6 | Arnés de medición | [PR #4](https://github.com/NewHouses/meiga/pull/4) | [0.6-medicion.md](0.6-medicion.md) |
| 0.7 | Escenarios de carga | [PR #5](https://github.com/NewHouses/meiga/pull/5) | [0.7-escenarios.md](0.7-escenarios.md) |
| 0.8 | Liña base | Local, sen cambios no repositorio | As medicións do Warp e do Windows Terminal e os orzamentos de rendemento quedan fóra do repositorio, porque describen unha máquina concreta. |
| 1.1 | Xanela baleira | [PR #7](https://github.com/NewHouses/meiga/pull/7) | [1.1-xanela.md](1.1-xanela.md) |
| 1.2 | *Framebuffer* e presentación | [PR #8](https://github.com/NewHouses/meiga/pull/8) | [1.2-framebuffer.md](1.2-framebuffer.md) |
| 1.3 | Texto: fontes e caché de glifos | [PR #9](https://github.com/NewHouses/meiga/pull/9) | [1.3-texto.md](1.3-texto.md) |
| 1.4 | ConPTY | [PR #10](https://github.com/NewHouses/meiga/pull/10) | [1.4-conpty.md](1.4-conpty.md) |
| 1.5 | Fachada do terminal | [PR #11](https://github.com/NewHouses/meiga/pull/11) | [1.5-fachada.md](1.5-fachada.md) |
| 1.6 | Da saída á pantalla | [PR #12](https://github.com/NewHouses/meiga/pull/12) | [1.6-saida.md](1.6-saida.md) |
| 1.7 | Teclado | [PR #13](https://github.com/NewHouses/meiga/pull/13) | [1.7-teclado.md](1.7-teclado.md) |
| 1.8 | Rato, selección e portapapeis | [PR #14](https://github.com/NewHouses/meiga/pull/14) | [1.8-rato.md](1.8-rato.md) |
| 1.9 | Scrollback, redimensionado e zoom | [PR #15](https://github.com/NewHouses/meiga/pull/15) | [1.9-historial.md](1.9-historial.md) |
| 1.10 | Porta do MVP | [PR #18](https://github.com/NewHouses/meiga/pull/18) | [1.10-porta.md](1.10-porta.md) |
| 2.1 | Modelo de tabs | [PR #20](https://github.com/NewHouses/meiga/pull/20) | [2.1-modelo.md](2.1-modelo.md) |
| 2.2 | Varias sesións e tabs en segundo plano | [PR #21](https://github.com/NewHouses/meiga/pull/21) | [2.2-sesions.md](2.2-sesions.md) |

## Mantemento

Entre subfases, algúns PR só actualizan as dependencias e as ferramentas, sen cambiar o que fai meiga: unha versión nova do ConPTY ou de Rust, ou unha regra de Dependabot. Cada un ten o seu documento, cos mesmos apartados ca unha subfase, e o seu identificador, M1, M2…

| Mantemento | Despois de | Título | Entrega | Documento |
|---|---|---|---|---|
| M1 | 1.10 | ConPTY 1.25.261109001, Rust 1.100.0 e Dependabot | [PR #19](https://github.com/NewHouses/meiga/pull/19) | [m1-mantemento.md](m1-mantemento.md) |
