#!/usr/bin/env bash
set -e

DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" >/dev/null 2>&1 && pwd )"
APP_NAME="Class Transcriber.app"
APP_SRC="$DIR/src-tauri/target/release/bundle/macos/$APP_NAME"
TARGET_DIR="/Applications"

echo "==> Instalando Class Transcriber en tu Mac..."

if [ ! -d "$APP_SRC" ]; then
  echo "El ejecutable no se encontró en $APP_SRC. Compilando..."
  cd "$DIR"
  ./node_modules/.bin/tauri build
fi

echo "==> Copiando $APP_NAME a $TARGET_DIR..."
rm -rf "$TARGET_DIR/$APP_NAME"
cp -R "$APP_SRC" "$TARGET_DIR/"

# Retirar cuarentena de macOS si corresponde
xattr -dr com.apple.quarantine "$TARGET_DIR/$APP_NAME" 2>/dev/null || true

echo ""
echo "✅ ¡Class Transcriber se instaló correctamente en /Applications!"
echo "👉 Puedes abrirla desde tus Aplicaciones, Launchpad o Spotlight."
echo ""
