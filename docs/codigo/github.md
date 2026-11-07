# Código › `.github/` (integración continua e Dependabot)

> Estado: subfase 0.6.

O cartafol `.github/` contén a configuración que usa GitHub: o *workflow* de integración continua e a configuración de Dependabot. Os conceptos (*workflow*, *job*, *runner*…) explícanse en [Tecnoloxías › GitHub Actions](../tecnoloxias.md#15-github-actions).

## `.github/workflows/ci.yml`

### Nome e eventos

```yaml
name: CI

on:
  pull_request:
    branches: [main]
  push:
    branches: [main]
  schedule:
    # Every Monday, so that new security advisories are reported even when the
    # dependencies did not change. Only the cargo-deny job runs on schedule.
    - cron: "17 5 * * 1"
  workflow_dispatch:
```

- `name: CI`: o nome que aparece na lapela *Actions* de GitHub e na insignia do README.
- `on`: os eventos que lanzan o workflow.
  - `pull_request` cara a `main`: valida cada PR antes de integralo. GitHub non proba a rama do PR tal cal, senón o resultado de mesturala con `main` (a referencia `refs/pull/N/merge`). É dicir, proba o que quedaría en `main`.
  - `push` a `main`: valida o resultado final de cada integración e garda a caché que reutilizarán os PR.
  - `schedule`: unha execución semanal, para detectar vulnerabilidades novas en dependencias que non cambiaron. O formato `cron` ten cinco campos: minuto, hora, día do mes, mes e día da semana. `17 5 * * 1` significa os luns ás 05:17 (UTC). O minuto 17 é intencionado: GitHub recomenda evitar as horas en punto, cando hai máis carga e as execucións programadas poden atrasarse.
  - `workflow_dispatch`: permite lanzalo a man desde a web, co botón *Run workflow*.

### Permisos

```yaml
permissions:
  contents: read
```

Cada execución recibe un *token* temporal (`GITHUB_TOKEN`) para operar co repositorio, que pode ter permisos de escritura. Eiquí limítase a ler o código. Aínda que un paso estivese comprometido, non podería modificar o repositorio, crear versións nin escribir nos PR. É o principio de **mínimo privilexio**.

### Concorrencia

```yaml
concurrency:
  group: ${{ github.workflow }}-${{ github.ref }}
  cancel-in-progress: ${{ github.event_name == 'pull_request' }}
```

- `${{ … }}` é unha **expresión** de GitHub Actions: GitHub avalíaa antes de executar o workflow.
- Dúas execucións co mesmo `group` nunca corren á vez. O grupo combina o nome do workflow e a referencia, que é distinta para cada PR e para `main`.
- `cancel-in-progress`: nun PR, se chega un commit novo mentres a CI aínda valida o anterior, a execución vella cancélase, porque o seu resultado xa non interesa. En `main` non se cancela nada: cada integración valídase enteira.

### Variábeis de contorno

```yaml
env:
  CARGO_TERM_COLOR: always
  # Do not set RUSTFLAGS: it would replace the rustflags in .cargo/config.toml,
  # including +crt-static.
```

- `CARGO_TERM_COLOR: always`: Cargo usa cores aínda que a súa saída non vaia a un terminal, e os rexistros lense mellor na web.
- O comentario avisa dunha trampa real. Cargo ten varias fontes de opcións para o compilador e usa só a primeira que atopa: a variábel `RUSTFLAGS` vai antes que os `rustflags` de `.cargo/config.toml`. Se alguén a define (por exemplo, con `-D warnings`, un costume habitual nas CI de Rust), Cargo ignora `.cargo/config.toml` e perdemos o `+crt-static`. Comprobouse nesta subfase: con `RUSTFLAGS='-C opt-level=3'`, `meiga.exe` voltou a depender de `VCRUNTIME140.dll`. O script [`check-imports.ps1`](ferramentas.md#toolscheck-importsps1) detecta este caso.

### Job `windows`

```yaml
  windows:
    name: Build and test (Windows)
    if: github.event_name != 'schedule'
    runs-on: windows-2025
    timeout-minutes: 30
    defaults:
      run:
        shell: pwsh
```

- `windows` é o identificador interno do job, e `name` o nome visíbel. **As regras da rama `main` esixen o check por este nome**, así que, se cambia, hai que actualizar as regras.
- `if`: o job non corre nas execucións programadas, que só buscan vulnerabilidades novas.
- `runs-on: windows-2025`: o *runner*, unha máquina virtual de GitHub con Windows Server 2025. Indícase a versión exacta en vez de `windows-latest` (ver [Tecnoloxías › Imaxes dos runners](../tecnoloxias.md#imaxes-dos-runners)).
- `timeout-minutes: 30`: se algo se bloquea, o job cancélase aos 30 minutos, en vez de agardar as 6 horas do límite por defecto.
- `defaults.run.shell: pwsh`: os pasos `run` execútanse con PowerShell 7, como os scripts de `tools/`.

#### Descargar o código

```yaml
      - uses: actions/checkout@3d3c42e5aac5ba805825da76410c181273ba90b1 # v7.0.1
        with:
          persist-credentials: false
```

- `uses` executa unha *action*: un compoñente reutilizábel publicado noutro repositorio. `actions/checkout` descarga o código do repositorio no runner.
- A versión fíxase co **SHA completo do commit**, e a etiqueta (`v7.0.1`) queda nun comentario. O motivo explícase en [Tecnoloxías › Seguridade do workflow](../tecnoloxias.md#seguridade-do-workflow).
- `persist-credentials: false`: por defecto, `checkout` deixa o *token* gardado na configuración de Git do runner, para que os pasos seguintes poidan facer `git push`. Non o necesitamos, e así ningún paso posterior pode reutilizalo.

#### Comprobar a documentación

```yaml
      - name: Check documentation links
        run: ./tools/check-docs.ps1
```

Comproba que todas as ligazóns internas da documentación apuntan a arquivos e títulos que existen (ver [Código › ferramentas](ferramentas.md#toolscheck-docsps1)). Vai antes de instalar Rust porque non o necesita: se a documentación ten unha ligazón rota, o job falla en segundos.

#### Comprobar o arnés de medición

```yaml
      - name: Check the measurement harness
        run: ./tools/perf/test-measure.ps1 -Duration 5
```

Executa a proba de `tools/perf/measure.ps1` con procesos de consumo coñecido (ver [Código › medición](perf.md#toolsperftest-measureps1)). Tampouco necesita Rust e leva uns 15 s. Non mide o rendemento de meiga: as máquinas da CI son compartidas e darían cifras con moito ruído. Só comproba que o arnés segue medindo ben, para que un cambio nos scripts non o estrague sen que se note.

#### Instalar Rust

```yaml
      - name: Install the Rust toolchain from rust-toolchain.toml
        run: |
          rustup toolchain install
          rustc --version
          cargo --version
```

- `run` executa comandos, e `|` permite escribir varias liñas.
- Os runners de GitHub xa traen rustup. `rustup toolchain install`, sen argumentos, le `rust-toolchain.toml` e instala esa versión exacta cos seus compoñentes. Non fai falta ningunha action de terceiros: unha dependencia menos.
- Os dous últimos comandos escriben as versións no rexistro, para diagnosticar problemas.

#### Instalar cargo-about

```yaml
      - uses: taiki-e/install-action@3a3daafd3b7f23b8f65d9f5ba2db203fb59b79e2 # v2.90.2
        with:
          tool: cargo-about@0.9.3
```

- `cargo-about` é necesario para empaquetar (xera os avisos de terceiros).
- `taiki-e/install-action` descarga o binario xa compilado e **comproba o seu SHA-256** cunha lista que vén na propia action, que tamén está fixada por SHA. Compilalo con `cargo install` levaría varios minutos en cada execución.
- A versión é a mesma que en local: 0.9.3.

#### Caché

```yaml
      - uses: Swatinem/rust-cache@d1527c8f67ef9af357290475d4af4b0fab8f12e0 # v2.9.3
        with:
          cache-bin: "false"
          save-if: ${{ github.ref == 'refs/heads/main' }}
```

- Garda entre execucións as dependencias descargadas e o cartafol `target/`, para non compilar todo desde cero cada vez. A clave da caché inclúe a versión de Rust e o `Cargo.lock`: se cambian, empézase cunha caché nova. Por iso este paso vai despois de instalar Rust.
- `cache-bin: "false"`: non garda os programas de `~/.cargo/bin`. As ferramentas instálanse verificadas en cada execución, e a caché nunca as substitúe.
- `save-if`: a caché só se garda desde `main`. Os PR poden usar a de `main`, pero unha caché creada nun PR só lle serviría a ese PR e ocuparía espazo: GitHub limita a caché a 10 GB por repositorio.

#### Comprobacións

```yaml
      - name: Check formatting
        run: cargo fmt --all --check

      - name: Clippy
        run: cargo clippy --workspace --all-targets --locked -- -D warnings

      - name: Test
        run: cargo test --workspace --locked
```

As mesmas comprobacións que en local (ver [Tecnoloxías](../tecnoloxias.md#7-lints-rustc-e-clippy)). Con `--locked`, se o `Cargo.lock` non está ao día, a CI falla en vez de actualizalo pola súa conta.

#### Empaquetar e comprobar o executábel

```yaml
      - name: Build and package
        run: ./tools/package.ps1

      - name: Check that meiga.exe does not need the Visual C++ runtime
        run: ./tools/check-imports.ps1
```

Compila en modo *release*, crea o `.zip` en `dist/` e comproba as DLL das que depende `meiga.exe`. Os dous scripts explícanse en [Código › ferramentas](ferramentas.md).

#### Subir o artefacto

```yaml
      - uses: actions/upload-artifact@cf430e030ddbb5b0abf93d22962f4752f3646cd9 # v7.0.2
        with:
          path: dist/*.zip
          archive: false
          if-no-files-found: error
          retention-days: 30
```

- Sobe o `.zip` como **artefacto** da execución, que se pode descargar desde a web.
- `archive: false`: sóbeo tal cal. Por defecto, a action metería os arquivos noutro zip, e descargariamos un zip dentro doutro. Con esta opción só se pode subir un arquivo, e o artefacto leva o seu nome.
- `if-no-files-found: error`: se non hai ningún zip, o paso falla, en vez de mostrar só un aviso.
- `retention-days: 30`: os artefactos bórranse aos 30 días.

### Job `deny`

```yaml
  deny:
    name: Dependency policy (cargo-deny)
    runs-on: ubuntu-24.04
    timeout-minutes: 15
    steps:
      - uses: actions/checkout@3d3c42e5aac5ba805825da76410c181273ba90b1 # v7.0.1
        with:
          persist-credentials: false

      - name: Install the Rust toolchain from rust-toolchain.toml
        run: rustup toolchain install

      - uses: taiki-e/install-action@3a3daafd3b7f23b8f65d9f5ba2db203fb59b79e2 # v2.90.2
        with:
          tool: cargo-deny@0.20.2

      - name: cargo deny
        run: cargo deny --locked check
```

- Corre en paralelo co job de Windows.
- Usa Linux porque o resultado non depende do sistema (`deny.toml` xa indica que se analicen as dependencias de Windows) e as máquinas Linux arrancan antes.
- Necesita Rust porque cargo-deny le as dependencias a través de `cargo metadata`.
- `cargo deny --locked check`: `--locked` vai antes de `check` porque é unha opción xeral de cargo-deny, non do subcomando `check`.
- É o único job das execucións programadas. Cada vez descarga a base de datos de vulnerabilidades actualizada.

## `.github/dependabot.yml`

```yaml
version: 2
updates:
  - package-ecosystem: cargo
    directory: /
    schedule:
      interval: weekly
    # Wait before proposing a new release, so that a compromised or broken
    # version has time to be detected and yanked.
    cooldown:
      default-days: 7
    groups:
      cargo:
        patterns: ["*"]
```

- `version: 2`: a versión do formato do arquivo, que sempre é 2.
- `updates`: unha entrada por cada tipo de dependencia (*ecosistema*).
- `package-ecosystem: cargo`: os crates de Rust (`Cargo.toml` e `Cargo.lock`), buscados desde a raíz (`directory: /`).
- `schedule.interval: weekly`: busca versións novas unha vez por semana.
- `cooldown.default-days: 7`: só propón versións publicadas hai polo menos 7 días. Cando alguén publica unha versión maliciosa ou rota, adoita detectarse e retirarse nos primeiros días; así non chega a este repositorio.
- `groups`: todas as actualizacións do ecosistema (`patterns: ["*"]`) van nun único PR semanal, en vez dun PR por dependencia.

```yaml
  - package-ecosystem: github-actions
    directory: /
    schedule:
      interval: weekly
    cooldown:
      default-days: 7
    groups:
      github-actions:
        patterns: ["*"]
```

O mesmo para as actions dos workflows. Dependabot entende as actions fixadas por SHA: actualiza o SHA e tamén o comentario coa versión.

```yaml
  # The Rust version pinned in rust-toolchain.toml.
  - package-ecosystem: rust-toolchain
    directory: /
    schedule:
      interval: weekly
```

Vixía `rust-toolchain.toml` e abre un PR cando sae unha versión nova de Rust, que é cada seis semanas. Así a versión fixada non queda vella, e cada cambio de versión pasa pola CI antes de integrarse.
