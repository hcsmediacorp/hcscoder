#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DIST_DIR="${ROOT_DIR}/dist"
VERSION="$(grep -m1 '^version = ' "${ROOT_DIR}/Cargo.toml" | sed -E 's/version = "([^"]+)"/\1/')"
TARGET_TRIPLE="$(rustc -vV | awk '/host:/ {print $2}')"
PKG_BASE="hcscoder-v${VERSION}-${TARGET_TRIPLE}"

printf '\n==> Building release binaries for %s\n' "${TARGET_TRIPLE}"
cargo build --release --locked --manifest-path "${ROOT_DIR}/Cargo.toml"

mkdir -p "${DIST_DIR}/${PKG_BASE}"
cp "${ROOT_DIR}/target/release/hcscoder" "${DIST_DIR}/${PKG_BASE}/" 2>/dev/null || true
cp "${ROOT_DIR}/target/release/hcscoder.exe" "${DIST_DIR}/${PKG_BASE}/" 2>/dev/null || true
cp "${ROOT_DIR}/target/release/hcscoder-setup" "${DIST_DIR}/${PKG_BASE}/" 2>/dev/null || true
cp "${ROOT_DIR}/target/release/hcscoder-setup.exe" "${DIST_DIR}/${PKG_BASE}/" 2>/dev/null || true
cp "${ROOT_DIR}/README.md" "${DIST_DIR}/${PKG_BASE}/"
cp "${ROOT_DIR}/LICENSE" "${DIST_DIR}/${PKG_BASE}/"

if [[ "${TARGET_TRIPLE}" == *"windows"* ]]; then
  ARCHIVE="${DIST_DIR}/${PKG_BASE}.zip"
  rm -f "${ARCHIVE}"
  (cd "${DIST_DIR}" && zip -rq "${ARCHIVE}" "${PKG_BASE}")
else
  ARCHIVE="${DIST_DIR}/${PKG_BASE}.tar.gz"
  rm -f "${ARCHIVE}"
  tar -C "${DIST_DIR}" -czf "${ARCHIVE}" "${PKG_BASE}"
fi

printf '==> Package created: %s\n' "${ARCHIVE}"
printf '==> Contents:\n'
if command -v tar >/dev/null 2>&1 && [[ "${ARCHIVE}" == *.tar.gz ]]; then
  tar -tzf "${ARCHIVE}"
else
  unzip -l "${ARCHIVE}" | sed -n '1,20p'
fi
