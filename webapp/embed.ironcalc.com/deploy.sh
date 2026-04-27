#!/usr/bin/env bash

set -euo pipefail

echo "🧹 Cleaning dist..."
rm -rf dist/*

echo "📦 Building..."
npm run dist
mkdir -p dist/fonts

echo "🔤 Copying fonts..."
cp ../app.ironcalc.com/frontend/src/fonts/* dist/fonts/

echo "🗜️ Compressing assets..."
(
  cd dist/assets
  brotli wasm*
  brotli index-*
)

echo "🧨 Recreating remote ~/embed..."
ssh app.ironcalc.com 'rm -rf ~/embed && mkdir -p ~/embed'

echo "🚀 Uploading..."
scp -r dist/* app.ironcalc.com:~/embed/

echo "📂 Publishing to /var/www/embed..."
ssh -t app.ironcalc.com '
  set -euo pipefail
  sudo rm -rf /var/www/embed
  sudo mkdir -p /var/www/embed
  sudo cp -r ~/embed/. /var/www/embed/
  sudo chown -R caddy:caddy /var/www/embed
'

echo "✅ Done"

