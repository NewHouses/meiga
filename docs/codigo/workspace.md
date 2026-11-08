# Código › configuración do workspace

> Estado: subfase 1.3.

Os arquivos da raíz do repositorio configuran como se compila e se revisa todo o proxecto. Neste documento explícanse un a un.

## `Cargo.toml` (raíz)

É o **manifesto do workspace**. Non define ningún crate: só agrupa os de `crates/` e fixa a configuración común.

```toml
[workspace]
resolver = "3"
members = ["crates/*"]
```

- `[workspace]` sen sección `[package]`: é un **manifesto virtual**, xa que a raíz non é un crate.
- `members = ["crates/*"]`: calquera cartafol de `crates/` cun `Cargo.toml` forma parte do workspace. Para engadir un crate novo abonda con crear o cartafol.
- `resolver = "3"`: a versión do algoritmo que escolle as versións das dependencias. A 3 ten en conta a versión mínima de Rust do proxecto. Nun manifesto virtual hai que indicala a man, porque non hai ningunha edición da que deducila.

```toml
[workspace.package]
version = "0.1.0"
edition = "2024"
rust-version = "1.99"
license = "LicenseRef-Proprietary"
repository = "https://github.com/NewHouses/meiga"
publish = false
```

Valores comúns que cada crate herda con `campo.workspace = true`, para non repetilos en catro sitios:

- `version`: a versión de meiga. Todos os crates comparten versión porque se distribúen xuntos.
- `edition = "2024"`: a edición de Rust (ver [Tecnoloxías › Rust](../tecnoloxias.md#1-rust)).
- `rust-version = "1.99"`: a versión mínima de Rust (*MSRV*). Se alguén compila cunha versión máis antiga, Cargo dá un erro claro en vez de fallos estraños. O resolvedor 3 tamén a usa para escoller versións das dependencias.
- `license = "LicenseRef-Proprietary"`: o identificador SPDX dunha licenza propia (SPDX reserva o prefixo `LicenseRef-` para as licenzas que non están na súa lista). O texto real está no arquivo `LICENSE`.
- `repository`: a URL do código fonte, como metadato.
- `publish = false`: impide publicar por erro algún crate en crates.io.

```toml
[workspace.dependencies]
meiga-core = { path = "crates/meiga-core" }
meiga-render = { path = "crates/meiga-render" }
meiga-win = { path = "crates/meiga-win" }
unicode-width = { version = "0.2.3", default-features = false }
windows = "0.62.2"
windows-core = "0.62.2"
windows-sys = "0.61.2"
```

Declara unha soa vez onde está cada dependencia e que versión se usa:

- Os crates internos, polo seu cartafol (`path`). Cada crate que o necesite escribe só `meiga-core.workspace = true`.
- As dependencias externas, coa súa versión, para que todos os crates usen a mesma. Cada crate escolle as súas *features*: por exemplo, `meiga-win` usa `windows-sys = { workspace = true, features = [...] }` (ver [Código › meiga-win](meiga-win.md#cratesmeiga-wincargotoml)).
  - `windows-sys`, desde a subfase 1.1, para Win32 e GDI (ver [Tecnoloxías › windows-sys](../tecnoloxias.md#21-windows-sys-as-apis-de-windows-desde-rust)).
  - `windows` e `windows-core`, desde a 1.3, para DirectWrite (ver [Tecnoloxías › COM e o crate `windows`](../tecnoloxias.md#28-com-e-o-crate-windows)). As dúas teñen a mesma versión, porque `windows` depende de `windows-core`, e comparten `windows-link` con `windows-sys`.
  - `unicode-width`, desde a 1.3, para o ancho dos caracteres. `default-features = false` quita a *feature* `cjk`, as táboas para os contextos CJK, que meiga non usa (ver [Tecnoloxías › O ancho dos caracteres](../tecnoloxias.md#29-o-ancho-dos-caracteres-unicode-width)). Neste caso as opcións van no `Cargo.toml` raíz, porque un crate pode engadir *features* pero non quitar as que trae por defecto.
- `"0.61.2"` significa «0.61.2 ou unha versión compatíbel posterior»: nas versións 0.x, as 0.61.*. A versión exacta que se usa é a do `Cargo.lock`.

```toml
[workspace.lints.rust]
rust_2018_idioms = { level = "warn", priority = -1 }
unsafe_op_in_unsafe_fn = "deny"

[workspace.lints.clippy]
dbg_macro = "deny"
expect_used = "warn"
todo = "deny"
undocumented_unsafe_blocks = "deny"
unimplemented = "deny"
unwrap_used = "deny"
```

As regras de calidade para todo o código. O significado de cada unha está en [Tecnoloxías › *Lints*](../tecnoloxias.md#7-lints-rustc-e-clippy). Detalles da sintaxe:

- `priority = -1` aplica primeiro un grupo de regras (`rust_2018_idioms`), para que as regras individuais declaradas despois poidan cambiar o nivel dalgunha delas.
- Estas regras non se aplican soas: cada crate ten que declarar `[lints] workspace = true`.

```toml
[profile.release]
codegen-units = 1
lto = "fat"
panic = "abort"

# Release build with debug information, for profiling with WPR/WPA.
[profile.profiling]
inherits = "release"
debug = "limited"
```

- `[profile.release]`: as opcións da compilación final, explicadas en [Tecnoloxías › Perfís](../tecnoloxias.md#4-perfís-de-compilación). `opt-level = 3` non aparece porque xa é o valor por defecto en *release*.
- `[profile.profiling]`: un perfil propio. `inherits = "release"` copia todas as opcións de *release* e `debug = "limited"` engade os nomes das funcións e os números de liña para os perfiladores. Úsase con `cargo build --profile profiling`, e o resultado vai a `target\profiling\`.

## `Cargo.lock`

Xérao Cargo automaticamente e **non se edita a man**. Garda a versión exacta de cada crate do workspace e de cada dependencia, coa súa suma de comprobación (*checksum*). Agora contén os catro crates propios e 17 dependencias: `windows-sys` e `windows-link`, desde a subfase 1.1; e, desde a 1.3, `unicode-width` e o crate `windows` cos seus: `windows-core`, `windows-collections`, `windows-future`, `windows-implement`, `windows-interface`, `windows-numerics`, `windows-result`, `windows-strings` e `windows-threading`, máis catro que só se usan ao compilar as macros (`proc-macro2`, `quote`, `syn` e `unicode-ident`). Súbese ao repositorio para que as compilacións sexan reproducíbeis.

## `rust-toolchain.toml`

```toml
[toolchain]
channel = "1.99.1"
components = ["clippy", "rustfmt"]
profile = "minimal"
```

Fixa a versión exacta do compilador e os compoñentes necesarios. rustup lé este arquivo cada vez que se executa `cargo` dentro do repositorio. Explicación completa en [Tecnoloxías › rustup](../tecnoloxias.md#2-rustup-e-rust-toolchaintoml).

## `.cargo/config.toml`

```toml
# Link the C runtime statically so that meiga.exe does not depend on the
# Visual C++ redistributable.
[target.'cfg(all(windows, target_env = "msvc"))']
rustflags = ["-C", "target-feature=+crt-static"]
```

- `.cargo/config.toml` é a configuración de Cargo para este repositorio. Aplícase a todas as compilacións feitas dentro del.
- `[target.'cfg(…)']` aplica as opcións só cando o destino cumpre a condición: Windows co compilador de Microsoft (MSVC).
- `rustflags` pasa opcións directamente ao compilador. `+crt-static` liga o *runtime* de C dentro do `.exe` (ver [Tecnoloxías › CRT](../tecnoloxias.md#5-runtime-de-c-ligado-estaticamente-crt-static)).

## `rustfmt.toml`

```toml
use_field_init_shorthand = true
use_try_shorthand = true
```

Opcións do formateador, explicadas en [Tecnoloxías › rustfmt](../tecnoloxias.md#8-rustfmt). O resto de opcións usan os valores por defecto, que son o estilo estándar de Rust.

## `clippy.toml`

```toml
allow-expect-in-tests = true
allow-unwrap-in-tests = true
```

Fai que as regras `unwrap_used` e `expect_used` non se apliquen nas probas. Nunha proba, que `unwrap()` provoque un pánico é precisamente o que se quere: a proba falla e mostra onde.

## `deny.toml`

A política de dependencias que revisa `cargo deny check`. Cada sección explícase en [Tecnoloxías › cargo-deny](../tecnoloxias.md#9-cargo-deny). Resumo:

| Sección | Que fai |
|---|---|
| `[graph] targets` | Só considera as dependencias que se compilan para Windows x64. |
| `[advisories]` | Erro se unha dependencia ten unha vulnerabilidade coñecida ou se a súa versión foi retirada (`yanked = "deny"`). `ignore = []`: non se ignora ningún aviso. |
| `[licenses]` | Lista de licenzas permitidas (só permisivas), limiar de recoñecemento do 93 % e sen avisos polas licenzas que aínda non se usan. |
| `[licenses.private]` | Non revisa os crates propios, que non se publican. |
| `[bans]` | Aviso se hai dúas versións da mesma dependencia; erro se unha versión usa comodíns. |
| `[sources]` | Só se permiten dependencias de crates.io. |

## `about.toml`

Configuración de `cargo-about` para xerar os avisos de terceiros. Explicada en [Tecnoloxías › cargo-about](../tecnoloxias.md#10-cargo-about-e-handlebars). A lista `accepted` debe coincidir sempre coa de `deny.toml`. Por iso hai un comentario no propio arquivo que o lembra.

## `.gitattributes` e `.gitignore`

Explicados en [Tecnoloxías › Git e GitHub](../tecnoloxias.md#12-git-e-github).

## `README.md`, `LICENSE` e `SECURITY.md`

- `README.md` (en inglés): que é meiga, estado, obxectivos, requisitos (entre eles, PowerShell 7 para os scripts de `tools/`), como compilalo e empaquetalo, as comprobacións que debe pasar cada cambio e como medir un terminal co arnés e cos escenarios de carga. Leva unha insignia (*badge*) co estado da CI en `main` e unha ligazón a esta documentación.
- `LICENSE`: todos os dereitos reservados. O código vese en GitHub, pero non se concede permiso para usalo, copialo nin distribuílo. A única excepción son os dereitos que os termos de servizo de GitHub dan a calquera usuario: ver o repositorio e facer un *fork* dentro de GitHub. O último parágrafo lembra que os compoñentes de terceiros manteñen as súas propias licenzas, e que os seus avisos se distribúen cos binarios en `THIRD-PARTY-NOTICES.txt`.
- `SECURITY.md`: como informar dun problema de seguridade de forma privada.
