#!/usr/bin/env bash
set -e

DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" >/dev/null 2>&1 && pwd )"
APP_NAME="Class Transcriber.app"
APP_SRC="$DIR/src-tauri/target/release/bundle/macos/$APP_NAME"
DMG_NAME="Class-Transcriber.dmg"
TEMP_DIR="/tmp/ct_dmg_build"

echo "==> Generando instalador DMG para Class Transcriber..."

if [ ! -d "$APP_SRC" ]; then
  echo "Compilando la aplicación primero..."
  cd "$DIR"
  if [ ! -x ./node_modules/.bin/tauri ]; then
    echo "==> Instalando dependencias de npm (npm ci)..."
    npm ci
  fi
  ./node_modules/.bin/tauri build
fi

rm -rf "$TEMP_DIR" "$DIR/$DMG_NAME"
mkdir -p "$TEMP_DIR"
cp -R "$APP_SRC" "$TEMP_DIR/"
ln -s /Applications "$TEMP_DIR/Applications"

hdiutil create -volname "Class Transcriber" -srcfolder "$TEMP_DIR" -ov -format UDZO "$DIR/$DMG_NAME"
rm -rf "$TEMP_DIR"

echo ""
echo "✅ Instalador creado exitosamente: $DIR/$DMG_NAME"
echo ""
