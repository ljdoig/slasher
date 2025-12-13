#!/bin/zsh
set -e

if [[ "$#" -lt 1 ]]; then
    echo "Usage: $0 <project name>" >&2
    exit 1
fi

name=$1
echo "Deploying to gh pages project: $name"

# Check webpage uses right js file
if ! grep -q "${name}.js" "web/index.html"; then
  echo "Error: The file 'web/index.html' does not contain the required string: '${name}.js'."
  exit 1 
fi

cargo build --release --target wasm32-unknown-unknown

rm -f web/${name}_bg.wasm
rm -f web/${name}.js
wasm-bindgen \
    --no-typescript \
    --out-dir web \
    --target web \
    target/wasm32-unknown-unknown/release/${name}.wasm
echo "Optimizing wasm file..."
wasm-opt -Oz -o web/${name}_bg_opt.wasm web/${name}_bg.wasm
mv web/${name}_bg_opt.wasm web/${name}_bg.wasm
# cat build/append.txt >> web/${name}.js

rm -rf web/assets
cp -r assets web/   

TMPDIR=$(mktemp -d)
cp -r web $TMPDIR/web
sed -i '' s/web/$name/g $TMPDIR/web/index.html

git switch web
cp -r $TMPDIR/web/* .
rm -rf $TMPDIR

git status
printf "Do you want to commit these changes? [y/N] "
read -r response
if [[ "$response" =~ ^([yY][eE][sS]|[yY])$ ]]
then
  git add . -v
  git commit -m "Deploy ${name} to gh-pages"
  git push origin web
  git switch master
else
  echo "Aborting."
  git reset --hard HEAD
  git clean -fd
  git switch master
  exit 1
fi


