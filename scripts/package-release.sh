#!/usr/bin/env bash
# Collects a universal build's release files into out/: the .dmg, a .zip of the app, the signed update
# (.app.tar.gz), latest.json for the in-app updater, and SHA256SUMS.txt.
# Run after: pnpm tauri build --target universal-apple-darwin --config '{"bundle":{"createUpdaterArtifacts":true}}'
# DOWNLOAD_BASE is where latest.json says the update can be downloaded; it defaults to this version's GitHub release.
set -euo pipefail
cd "$(dirname "$0")/.."

version=$(node -p "require('./src-tauri/tauri.conf.json').version")
repo=${GITHUB_REPOSITORY:-irfadrazick/gitsaurus}
base=${DOWNLOAD_BASE:-https://github.com/$repo/releases/download/v$version}
bundle=src-tauri/target/universal-apple-darwin/release/bundle
update="Gitsaurus_${version}_universal.app.tar.gz"

rm -rf out
mkdir out
cp "$bundle/dmg/Gitsaurus_${version}_universal.dmg" out/
ditto -c -k --keepParent "$bundle/macos/Gitsaurus.app" "out/Gitsaurus_${version}_universal.zip"
cp "$bundle/macos/Gitsaurus.app.tar.gz" "out/$update"

VERSION="$version" URL="$base/$update" NOTES="https://github.com/$repo/releases/tag/v$version" \
  SIGNATURE="$(cat "$bundle/macos/Gitsaurus.app.tar.gz.sig")" \
  node -e '
    const { VERSION, URL, NOTES, SIGNATURE } = process.env;
    const target = { signature: SIGNATURE, url: URL };
    const manifest = {
      version: VERSION,
      notes: `See ${NOTES}`,
      pub_date: new Date().toISOString(),
      platforms: { "darwin-aarch64": target, "darwin-x86_64": target },
    };
    require("fs").writeFileSync("out/latest.json", JSON.stringify(manifest, null, 2) + "\n");
  '

(cd out && shasum -a 256 * > SHA256SUMS.txt)
ls -l out
