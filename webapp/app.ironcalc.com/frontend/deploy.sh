rm -rf dist/*
npm run build
cd dist/assets && brotli wasm* && brotli index-*
cd ..
scp -r * app.ironcalc.com:~/app/
