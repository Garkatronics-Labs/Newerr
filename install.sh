#!/usr/bin/env bash
# install.sh - Installer for newerr
#
# Usage:
#   curl -fsSL https://raw.githubusercontent.com/Garkatronics-Labs/Newerr/main/install.sh | bash
#
# Optional environment variables:
#   NEWERR_INSTALL_DIR        Installation directory (default: ~/.local/bin)
#   NEWERR_VERSION            Release tag to install (default: latest)
#   NEWERR_SKIP_COMPLETIONS   Set to 1 to skip shell completions setup

set -euo pipefail

REPO="Garkatronics-Labs/Newerr"
BIN_NAME="newerr"
INSTALL_DIR="${NEWERR_INSTALL_DIR:-$HOME/.local/bin}"
VERSION="${NEWERR_VERSION:-latest}"

info()  { printf '\033[1;34m==>\033[0m %s\n' "$1"; }
warn()  { printf '\033[1;33m==>\033[0m %s\n' "$1"; }
error() { printf '\033[1;31mError:\033[0m %s\n' "$1" >&2; exit 1; }

detect_platform() {
	os="$(uname -s)"
	arch="$(uname -m)"

	case "$os" in
		Linux)  os="linux" ;;
		Darwin) os="macos" ;;
		*) error "unsupported operating system: $os" ;;
	esac

	case "$arch" in
		x86_64|amd64) arch="amd64" ;;
		arm64|aarch64) arch="arm64" ;;
		*) error "unsupported architecture: $arch" ;;
	esac

	echo "${os}-${arch}"
}

build_url() {
	local platform="$1"
	if [ "$VERSION" = "latest" ]; then
		echo "https://github.com/${REPO}/releases/latest/download/${BIN_NAME}-${platform}"
	else
		echo "https://github.com/${REPO}/releases/download/${VERSION}/${BIN_NAME}-${platform}"
	fi
}

detect_shell() {
	basename "${SHELL:-/bin/sh}"
}

append_if_missing() {
	local file="$1" marker="$2"
	shift 2
	touch "$file"
	if ! grep -qF -- "$marker" "$file"; then
		{
			printf '\n'
			printf '%s\n' "$@"
		} >> "$file"
		return 0
	fi
	return 1
}

install_completions() {
	local shell_name="$(detect_shell)"

	# /bin/sh is commonly bash (or dash); the bash completion script is the
	# closest match either way.
	case "$shell_name" in
		sh) shell_name="bash" ;;
	esac

	case "$shell_name" in
		bash)
			local comp_dir="${XDG_DATA_HOME:-$HOME/.local/share}/bash-completion/completions"
			local comp_file="${comp_dir}/${BIN_NAME}"
			mkdir -p "$comp_dir"
			"${INSTALL_DIR}/${BIN_NAME}" completions bash > "$comp_file"
			if append_if_missing "$HOME/.bashrc" "# newerr completions" \
				"[ -f \"${comp_file}\" ] && source \"${comp_file}\"  # newerr completions"; then
				info "bash completions installed (they load on the next shell session)"
			else
				info "bash completions already configured"
			fi
			;;
		zsh)
			local comp_dir="${XDG_DATA_HOME:-$HOME/.local/share}/zsh/site-functions"
			local comp_file="${comp_dir}/_${BIN_NAME}"
			mkdir -p "$comp_dir"
			"${INSTALL_DIR}/${BIN_NAME}" completions zsh > "$comp_file"
			if append_if_missing "$HOME/.zshrc" "# newerr completions" \
				"fpath=(\"${comp_dir}\" \$fpath)  # newerr completions" \
				"autoload -U compinit && compinit"; then
				info "zsh completions installed (they load on the next shell session)"
			else
				info "zsh completions already configured"
			fi
			;;
		fish)
			local comp_dir="${XDG_CONFIG_HOME:-$HOME/.config}/fish/completions"
			mkdir -p "$comp_dir"
			"${INSTALL_DIR}/${BIN_NAME}" completions fish > "${comp_dir}/${BIN_NAME}.fish"
			info "fish completions installed (they load on the next shell session)"
			;;
		*)
			warn "no completions for shell '${shell_name}' (supported: bash, zsh, fish)"
			;;
	esac
}

main() {
	platform="$(detect_platform)"
	url="$(build_url "$platform")"
	tmp_file="$(mktemp)"

	info "downloading ${BIN_NAME} (${platform}, ${VERSION})"

	if ! curl -fsSL -o "$tmp_file" "$url"; then
		rm -f "$tmp_file"
		error "failed to download binary from: $url"
	fi

	if ! file "$tmp_file" | grep -qE 'ELF|Mach-O'; then
		rm -f "$tmp_file"
		error "downloaded file is not a valid binary (does asset '${BIN_NAME}-${platform}' exist?)"
	fi

	mkdir -p "$INSTALL_DIR"
	install -m 755 "$tmp_file" "${INSTALL_DIR}/${BIN_NAME}"
	rm -f "$tmp_file"

	info "installed to ${INSTALL_DIR}/${BIN_NAME}"

	if ! echo ":$PATH:" | grep -q ":${INSTALL_DIR}:"; then
		warn "${INSTALL_DIR} is not in your PATH"
		shell_rc=""
		case "$(detect_shell)" in
			bash|sh) shell_rc="$HOME/.bashrc" ;;
			zsh)     shell_rc="$HOME/.zshrc" ;;
			fish)    shell_rc="$HOME/.config/fish/config.fish" ;;
		esac
		if [ -n "$shell_rc" ]; then
			warn "add this line to ${shell_rc}:"
		else
			warn "add this line to your shell config:"
		fi
		echo
		echo "    export PATH=\"${INSTALL_DIR}:\$PATH\""
		echo
	fi

	if [ "${NEWERR_SKIP_COMPLETIONS:-0}" = "1" ]; then
		warn "skipping completions setup (NEWERR_SKIP_COMPLETIONS=1)"
	else
		install_completions
	fi

	info "verify the install with: ${BIN_NAME} --version"
}

main "$@"
