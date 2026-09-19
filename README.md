# Newerr

CLI tool for software error management and automation.

## Usage

- `newerr init` – create the `.newerr/` project config
- `newerr err <name> --category <cat> --severity <sev> [--message <msg>] [--hint <hint>]` – add a new error
- `newerr cat <name>` – add a new category
- `newerr modify <id> <property> <value>` – modify an error property
- `newerr gen [--frontend <name>]` – generate the code file, written to `generated_errors_file_path` (builtin frontends: `markdown`, `odin`)
- `newerr doc [--template <name>]` – generate the documentation, written to `generated_docs_file_path` without touching the `gen` output (default template: `markdown`)
- `newerr completions <shell>` – print shell completions

Templates are looked up in `.newerr/templates/<name>.toml` first, then in the
builtin ones. Config lives in `.newerr/config.toml`:

- `generated_errors_file_path` – where `gen` writes
- `generated_docs_file_path` – where `doc` writes (optional; `doc` errors if unset)
- `id_generator` – id generation strategy
- `package` – optional package name used in templates

## Install

```sh
curl -fsSL https://raw.githubusercontent.com/Garkatronics-Labs/Newerr/main/install.sh | bash
```

The installer downloads the binary from the latest GitHub Release into
`~/.local/bin` and sets up shell completions for `bash`, `zsh` or `fish`
(it runs `newerr completions <shell>` and wires it into your shell config).

Options:

- `NEWERR_INSTALL_DIR` – installation directory (default: `~/.local/bin`)
- `NEWERR_VERSION` – specific release tag, e.g. `v1.0.0` (default: `latest`)
- `NEWERR_SKIP_COMPLETIONS=1` – skip the completions setup

## Releasing

Push to `main` (or to the `release` branch) a commit whose first line starts
with the version prefix:

```sh
git commit -m "release 1.2.3: description of the release"   # or "version 1.2.3: ..."
git push
```

The workflow will bump `Cargo.toml`, tag `v1.2.3`, build the binary and
publish a GitHub Release (asset `newerr-linux-amd64`), plus `cargo publish`
if the `CARGO_REGISTRY_TOKEN` secret is set.
