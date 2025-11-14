#!/bin/sh
set -e

REPO="nixval/declarch"
BIN_NAME="declarch"

get_latest_asset_url() {
  curl --silent "https://api.github.com/repos/$REPO/releases/latest" |
    grep '"browser_download_url"' |
    sed -E 's/.*"browser_download_url": "([^"]+)".*/\1/' |
    grep "$BIN_NAME" |
    head -n 1
}

echo "Fetching latest release info for $REPO ..."
ASSET_URL=$(get_latest_asset_url)

if [ -z "$ASSET_URL" ]; then
  echo "Error: no release asset found containing '$BIN_NAME'."
  echo "Check the Releases page on GitHub."
  exit 1
fi

FILE_NAME=$(basename "$ASSET_URL")

echo "Downloading: $FILE_NAME"
curl -L -o "$FILE_NAME" "$ASSET_URL"

case "$FILE_NAME" in
  *.tar.gz|*.tgz)
    echo "Extracting archive..."
    tar -xzf "$FILE_NAME"
    rm "$FILE_NAME"
    BIN_PATH=$(find . -maxdepth 1 -type f -name "$BIN_NAME*" | head -n 1)
    ;;
  *.zip)
    echo "Extracting zip..."
    unzip -o "$FILE_NAME"
    rm "$FILE_NAME"
    BIN_PATH=$(find . -maxdepth 1 -type f -name "$BIN_NAME*" | head -n 1)
    ;;
  *)
    BIN_PATH="$FILE_NAME"
    ;;
esac

if [ ! -f "$BIN_PATH" ]; then
  echo "Error: extracted binary not found."
  exit 1
fi

chmod +x "$BIN_PATH"

echo "Installing to /usr/local/bin (sudo required)..."
sudo mv "$BIN_PATH" /usr/local/bin/"$BIN_NAME"

echo ""
echo "Installed successfully as '$BIN_NAME'."
echo "Try: $BIN_NAME --help"
