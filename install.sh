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
  for cmd in npm cargo; do
    if ! command -v "$cmd" >/dev/null 2>&1; then
      echo "Falta '$cmd'. Instala Node.js y Rust (ver README) y vuelve a ejecutar ./install.sh" >&2
      exit 1
    fi
  done
  if [ ! -x ./node_modules/.bin/tauri ]; then
    echo "==> Instalando dependencias de npm (npm ci)..."
    npm ci
  fi
  ./node_modules/.bin/tauri build
fi

echo "==> Copiando $APP_NAME a $TARGET_DIR..."
rm -rf "${TARGET_DIR:?}/$APP_NAME"
cp -R "$APP_SRC" "$TARGET_DIR/"

echo ""
echo "✅ ¡Class Transcriber se instaló correctamente en /Applications!"
echo "👉 Puedes abrirla desde tus Aplicaciones, Launchpad o Spotlight."
echo ""
