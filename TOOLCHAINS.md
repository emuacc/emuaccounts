# Toolchains

Which Rust and Java the project builds with, how they are pinned, and how to
tell them apart from the other installs that live on a developer Mac.

## Canonical versions

| Tool | Version | Pinned by | Used for |
|---|---|---|---|
| Rust | **1.98.0** — rustup toolchain `1.98.0-aarch64-apple-darwin` | `rust-toolchain.toml` (channel `1.98.0`, minimal profile + rustfmt, clippy, llvm-tools); `src-tauri/Cargo.toml` `rust-version = "1.98"`; `dtolnay/rust-toolchain@1.98.0` in `.github/workflows/ci.yml` and `build.yml` | Every Cargo build and test |
| Java | **OpenJDK 21** — Homebrew `openjdk@21` (21.0.12.1 at time of writing) | `.java-version` (`21`); `actions/setup-java` 21 in the CI `grammar-drift` job; `scripts/generate_ledger_parsers.sh` refuses anything older than 21 | Only `scripts/generate_ledger_parsers.sh` (ANTLR → Rust). Ordinary builds never invoke Java |
| ANTLR | antlr4rust runtime `=0.5.2` (a dev-dependency: test builds only); generator jar antlr4rust v0.5.0 (ANTLR 4.13.3), sha256-verified on download | `src-tauri/Cargo.toml`; `scripts/generate_ledger_parsers.sh` | Committed sources in `src-tauri/src/ledger_parser/generated/`, used only by the grammar parity tests |
| Node | 24 | `.nvmrc` (`.github/workflows/build.yml` currently installs Node 20) | Vite, Vitest, Cucumber |

`cargo` is the rustup proxy at `~/.cargo/bin/cargo`, the only `cargo` on the
machine. It reads `rust-toolchain.toml` and selects 1.98.0 regardless of the
rustup default.

## Other installs on the development Mac (not used)

| Path | Version | Origin | Notes |
|---|---|---|---|
| `/usr/bin/java` | Oracle Java 1.8.0_503 | macOS launcher stub → `/Library/Internet Plug-Ins/JavaAppletPlugin.plugin` | The only JVM `/usr/libexec/java_home` knows about. `openjdk@21` is keg-only and is **not** symlinked into `/Library/Java/JavaVirtualMachines`, so `java_home -v 21` fails; JDK 21 is reached only through `JAVA_HOME`/`PATH`. |

History: until 2026-08-27 a MacPorts prefix at `/opt/local` (installed only
as a dependency chain for the `halloy` IRC client) shadowed `cargo` with rustc
1.87.0 in non-interactive shells, producing
`error: rustc 1.87.0 is not supported by the following packages`. MacPorts was
removed entirely; if that error reappears, a stray `cargo` is ahead of
`~/.cargo/bin` on `PATH` again.

## How a shell picks the right one

Both toolchains are exported from `~/.zshenv`, which every zsh reads —
interactive terminals, `zsh -c`, git hooks, cron and agent tools alike:

```zsh
export JAVA_HOME="/opt/homebrew/opt/openjdk@21/libexec/openjdk.jdk/Contents/Home"
export PATH="$HOME/.cargo/bin:$JAVA_HOME/bin:$PATH"
```

Login shells need one more line: macOS `/etc/zprofile` runs `path_helper`
after `~/.zshenv` and moves `/usr/bin` (the Oracle Java 8 stub) back to the
front, so `~/.zprofile` re-asserts the same `PATH` prepend at its end.
Backups of the previous files: `~/.zshrc.pre-java-home-20260827`,
`~/.zshenv.pre-java-home-20260827`, `~/.zprofile.pre-java-home-20260827`.

- **Rust**: `~/.cargo/bin/cargo` is the only `cargo` on the machine; nothing
  else can shadow it.
- **Java**: correct in every zsh. A non-zsh parent that inherits no
  environment (a `sh -c` from an app launched by Finder) still sees
  `/usr/bin/java` = 1.8; prefix such a command with
  `PATH="/opt/homebrew/opt/openjdk@21/bin:$PATH"`. Only
  `scripts/generate_ledger_parsers.sh` cares.

## Verify

```sh
which cargo                   # ~/.cargo/bin/cargo
cargo --version               # cargo 1.98.0
rustup show active-toolchain  # 1.98.0-aarch64-apple-darwin (overridden by '.../rust-toolchain.toml')
java -version                 # openjdk version "21.0.x"
scripts/generate_ledger_parsers.sh --check   # Committed ledger parser sources are up to date.
```

## Changing a pin

- **Rust**: `rustup toolchain install <ver>`, then update `rust-toolchain.toml`,
  `rust-version` in `src-tauri/Cargo.toml`, and the `dtolnay/rust-toolchain@<ver>`
  step in both workflows. Optionally `rustup default <ver>`.
- **Java**: install the new Homebrew `openjdk@<N>`, then update `JAVA_HOME` in
  `~/.zshenv`, `.java-version`, `java-version` in `ci.yml`, and the minimum
  check in `scripts/generate_ledger_parsers.sh`. Regenerate and commit the
  parsers.
- **ANTLR generator**: update `GENERATOR_URL` and `GENERATOR_SHA256` in
  `scripts/generate_ledger_parsers.sh` together with the `antlr4rust` runtime
  version in `Cargo.toml`; regenerate and commit.
