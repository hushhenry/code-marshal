#!/usr/bin/env bash
set -euo pipefail

REPO="hushhenry/code-marshal"
BIN_NAME="code-marshal"

usage() {
  cat <<'EOF'
Install code-marshal from the latest GitHub Release.

Usage:
  curl -fsSL https://raw.githubusercontent.com/hushhenry/code-marshal/master/scripts/install.sh | bash

Options:
  --repo <owner/name>     Override GitHub repo (default: hushhenry/code-marshal)
  --version <tag>         Install a specific version tag (default: latest)
  --prefix <dir>          Install prefix (default: ~/.local/bin)
  --force                 Overwrite existing binary
EOF
}

VERSION="latest"
PREFIX="${HOME}/.local/bin"
FORCE=0

while [[ $# -gt 0 ]]; do
  case "$1" in
    -h|--help) usage; exit 0 ;;
    --repo) REPO="$2"; shift 2 ;;
    --version) VERSION="$2"; shift 2 ;;
    --prefix) PREFIX="$2"; shift 2 ;;
    --force) FORCE=1; shift ;;
    *) echo "Unknown arg: $1"; usage; exit 1 ;;
  esac
done

need() {
  command -v "$1" >/dev/null 2>&1 || { echo "Missing dependency: $1" >&2; exit 1; }
}

need uname
need mktemp
need tar
need grep
need sed

if command -v curl >/dev/null 2>&1; then
  HTTP_GET="curl -fsSL"
elif command -v wget >/dev/null 2>&1; then
  HTTP_GET="wget -qO-"
else
  echo "Need curl or wget" >&2
  exit 1
fi

OS="$(uname -s | tr '[:upper:]' '[:lower:]')"
ARCH="$(uname -m)"

case "$OS" in
  linux) TARGET_OS="unknown-linux-gnu" ;;
  darwin) TARGET_OS="apple-darwin" ;;
  msys*|mingw*|cygwin*)
    echo "Windows is not supported by this installer yet." >&2
    exit 1
    ;;
  *) echo "Unsupported OS: $OS" >&2; exit 1 ;;
esac

case "$ARCH" in
  x86_64|amd64) TARGET_ARCH="x86_64" ;;
  arm64|aarch64) TARGET_ARCH="aarch64" ;;
  *) echo "Unsupported arch: $ARCH" >&2; exit 1 ;;
esac

TARGET="${TARGET_ARCH}-${TARGET_OS}"

API_BASE="https://api.github.com/repos/${REPO}/releases"
if [[ "$VERSION" == "latest" ]]; then
  API_URL="${API_BASE}/latest"
else
  API_URL="${API_BASE}/tags/${VERSION}"
fi

JSON="$(${HTTP_GET} "${API_URL}")"
TAG="$(echo "$JSON" | sed -n 's/.*"tag_name"[ ]*:[ ]*"\([^"]*\)".*/\1/p' | head -n1)"
if [[ -z "$TAG" ]]; then
  echo "Failed to resolve release tag from GitHub API." >&2
  exit 1
fi

ASSET="code-marshal-${TAG}-${TARGET}.tar.gz"
DL_URL="$(echo "$JSON" | sed -n 's/.*"browser_download_url"[ ]*:[ ]*"\([^"]*\)".*/\1/p' | grep -F "${ASSET}" | head -n1)"

if [[ -z "$DL_URL" ]]; then
  echo "No matching asset found for ${TARGET}." >&2
  echo "Expected asset name: ${ASSET}" >&2
  exit 1
fi

mkdir -p "$PREFIX"
OUT_BIN="${PREFIX}/${BIN_NAME}"

if [[ -f "$OUT_BIN" && "$FORCE" -ne 1 ]]; then
  echo "${OUT_BIN} already exists. Re-run with --force to overwrite." >&2
  exit 1
fi

TMPDIR="$(mktemp -d)"
trap 'rm -rf "$TMPDIR"' EXIT

ARCHIVE_PATH="${TMPDIR}/${ASSET}"

if [[ "$HTTP_GET" == curl* ]]; then
  curl -fL --retry 3 --retry-delay 1 -o "$ARCHIVE_PATH" "$DL_URL"
else
  wget -qO "$ARCHIVE_PATH" "$DL_URL"
fi

tar -xzf "$ARCHIVE_PATH" -C "$TMPDIR"

if [[ ! -f "${TMPDIR}/${BIN_NAME}" ]]; then
  echo "Archive did not contain expected binary: ${BIN_NAME}" >&2
  exit 1
fi

install -m 0755 "${TMPDIR}/${BIN_NAME}" "$OUT_BIN"

echo "Installed ${BIN_NAME} to ${OUT_BIN}"
echo "Run: ${BIN_NAME} --help"
