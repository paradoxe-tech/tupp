#!/bin/bash

PROJECT_NAME="tupp"
INSTALL_DIR="$HOME/.local/bin"
ARCH=$(uname -m)
VERSION="$1"

if [ -z "$1" ]; then
    echo "ERROR: Missing version argument."
    echo "Usage: $0 <VERSION>"
    exit 1
fi

KERNEL_NAME=$(uname -s)
case "$KERNEL_NAME" in
    Linux*)  OS="linux";;
    Darwin*) OS="macos";;
    *)       echo "Operating System '$KERNEL_NAME' not recognized. Using 'linux' as default."; OS="linux";;
esac

if [[ ! "$VERSION" =~ ^v ]]; then
    VERSION="v${VERSION}"
fi

DOWNLOAD_FILENAME="${PROJECT_NAME}-${VERSION}-${OS}-${ARCH}"
RELEASES_URL="https://github.com/mtripnaux/${PROJECT_NAME}/releases"
DOWNLOAD_URL="${RELEASES_URL}/download/${VERSION}/${DOWNLOAD_FILENAME}"

echo "Updating ${PROJECT_NAME} to ${VERSION} for ${OS}-${ARCH}."

TEMP_DIR=$(mktemp -d)
DOWNLOAD_PATH="${TEMP_DIR}/${DOWNLOAD_FILENAME}"

echo "Downloading binary from GitHub Releases..."
wget -q -O "${DOWNLOAD_PATH}" "${DOWNLOAD_URL}"

if [ $? -ne 0 ]; then
    echo "Download failed. Check if binary '${DOWNLOAD_FILENAME}' exist on ${RELEASES_URL}. If not, feel free to ask for it using GitHub issues."
    rm -rf "${TEMP_DIR}"
    exit 1
fi

sudo mv "${DOWNLOAD_PATH}" "${INSTALL_DIR}/${PROJECT_NAME}"
sudo chmod +x "${INSTALL_DIR}/${PROJECT_NAME}"

rm -rf "${TEMP_DIR}"

if [ -x "${INSTALL_DIR}/${PROJECT_NAME}" ]; then
    echo "Success. Installed version: $(${PROJECT_NAME} --version 2>/dev/null)"
else
    echo "The binary was installed but it is not executable."
    exit 1
fi