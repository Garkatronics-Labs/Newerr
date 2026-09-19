# Newerr

CLI tool for software error management and automation.

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
