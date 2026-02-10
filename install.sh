#!/bin/sh
# shellcheck disable=SC2039
# Taskbook-rs installer script
# Usage: curl -fsSL https://raw.githubusercontent.com/alexanderdavidsen/taskbook-rs/master/install.sh | sh
set -eu

REPO="alexanderdavidsen/taskbook-rs"
BINARY_NAME="tb"
INSTALL_DIR="${TB_INSTALL_DIR:-${HOME}/.local/bin}"

info() {
    printf "\033[1;34m[info]\033[0m %s\n" "$1"
}

error() {
    printf "\033[1;31m[error]\033[0m %s\n" "$1" >&2
    exit 1
}

warn() {
    printf "\033[1;33m[warn]\033[0m %s\n" "$1"
}

success() {
    printf "\033[1;32m[ok]\033[0m %s\n" "$1"
}

check_command() {
    command -v "$1" >/dev/null 2>&1
}

detect_os() {
    case "$(uname -s)" in
        Linux*)  echo "linux" ;;
        Darwin*) echo "macos" ;;
        *)       error "Unsupported operating system: $(uname -s). Only Linux and macOS are supported." ;;
    esac
}

detect_arch() {
    case "$(uname -m)" in
        x86_64|amd64)  echo "x86_64" ;;
        aarch64|arm64) echo "aarch64" ;;
        *)             error "Unsupported architecture: $(uname -m). Only x86_64 and aarch64 are supported." ;;
    esac
}

get_latest_version() {
    if check_command curl; then
        curl -fsSL "https://api.github.com/repos/${REPO}/releases/latest" | sed -n 's/.*"tag_name": *"\([^"]*\)".*/\1/p'
    elif check_command wget; then
        wget -qO- "https://api.github.com/repos/${REPO}/releases/latest" | sed -n 's/.*"tag_name": *"\([^"]*\)".*/\1/p'
    else
        error "Either curl or wget is required to download taskbook-rs."
    fi
}

download() {
    url="$1"
    dest="$2"
    if check_command curl; then
        curl -fsSL "$url" -o "$dest"
    elif check_command wget; then
        wget -qO "$dest" "$url"
    fi
}

main() {
    printf "\n"
    printf "  \033[1mtaskbook-rs installer\033[0m\n"
    printf "  Task & note management from the command line\n"
    printf "\n"

    # Check dependencies
    if ! check_command curl && ! check_command wget; then
        error "Either curl or wget is required to download taskbook-rs."
    fi

    if ! check_command tar; then
        error "tar is required to extract the release archive."
    fi

    # Detect platform
    os="$(detect_os)"
    arch="$(detect_arch)"
    info "Detected platform: ${os}/${arch}"

    # Get version
    version="${TB_VERSION:-}"
    if [ -z "$version" ]; then
        info "Fetching latest release version..."
        version="$(get_latest_version)"
    fi

    if [ -z "$version" ]; then
        error "Failed to determine the latest release version. Set TB_VERSION to install a specific version."
    fi

    info "Installing taskbook-rs ${version}"

    # Build download URL
    archive="tb-${os}-${arch}.tar.gz"
    url="https://github.com/${REPO}/releases/download/${version}/${archive}"

    # Download to temp directory
    tmpdir="$(mktemp -d)"
    trap 'rm -rf "$tmpdir"' EXIT

    info "Downloading ${url}..."
    download "$url" "${tmpdir}/${archive}" || error "Download failed. Check that release ${version} has an asset for ${os}/${arch}."

    # Extract
    info "Extracting..."
    tar xzf "${tmpdir}/${archive}" -C "$tmpdir" || error "Failed to extract archive."

    # Install
    mkdir -p "$INSTALL_DIR"
    mv "${tmpdir}/${BINARY_NAME}" "${INSTALL_DIR}/${BINARY_NAME}"
    chmod +x "${INSTALL_DIR}/${BINARY_NAME}"
    success "Installed ${BINARY_NAME} to ${INSTALL_DIR}/${BINARY_NAME}"

    # Check if install dir is in PATH
    case ":${PATH}:" in
        *":${INSTALL_DIR}:"*) ;;
        *)
            warn "${INSTALL_DIR} is not in your PATH."
            printf "\n"
            printf "  Add it to your shell configuration:\n"
            printf "\n"
            if check_command bash; then
                printf "    \033[1mbash\033[0m:  echo 'export PATH=\"%s:\$PATH\"' >> ~/.bashrc\n" "$INSTALL_DIR"
            fi
            if check_command zsh; then
                printf "    \033[1mzsh\033[0m:   echo 'export PATH=\"%s:\$PATH\"' >> ~/.zshrc\n" "$INSTALL_DIR"
            fi
            if check_command fish; then
                printf "    \033[1mfish\033[0m:  set -Ux fish_user_paths %s \$fish_user_paths\n" "$INSTALL_DIR"
            fi
            printf "\n"
            ;;
    esac

    printf "\n"
    printf "  Run \033[1mtb --help\033[0m to get started.\n"
    printf "\n"
}

main
