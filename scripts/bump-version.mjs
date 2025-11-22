/**
 * Version bump script for MelliLex.
 * Updates version in: package.json, tauri.conf.json, Cargo.toml
 *
 * Usage:
 *   node scripts/bump-version.mjs patch    # 0.1.0 → 0.1.1
 *   node scripts/bump-version.mjs minor    # 0.1.0 → 0.2.0
 *   node scripts/bump-version.mjs major    # 0.1.0 → 1.0.0
 *   node scripts/bump-version.mjs 1.2.3    # explicit version
 */

import { readFileSync, writeFileSync } from 'fs';
import { resolve, dirname } from 'path';
import { fileURLToPath } from 'url';

const __dirname = dirname(fileURLToPath(import.meta.url));
const root = resolve(__dirname, '..');

const files = {
  package: resolve(root, 'package.json'),
  tauri: resolve(root, 'src-tauri/tauri.conf.json'),
  cargo: resolve(root, 'src-tauri/Cargo.toml'),
};

function getCurrentVersion() {
  const pkg = JSON.parse(readFileSync(files.package, 'utf-8'));
  return pkg.version;
}

function bumpVersion(current, type) {
  const [major, minor, patch] = current.split('.').map(Number);
  switch (type) {
    case 'major': return `${major + 1}.0.0`;
    case 'minor': return `${major}.${minor + 1}.0`;
    case 'patch': return `${major}.${minor}.${patch + 1}`;
    default:
      if (/^\d+\.\d+\.\d+$/.test(type)) return type;
      console.error(`Invalid version type: "${type}". Use: patch, minor, major, or X.Y.Z`);
      process.exit(1);
  }
}

function updateJson(filePath, newVersion) {
  const content = JSON.parse(readFileSync(filePath, 'utf-8'));
  content.version = newVersion;
  writeFileSync(filePath, JSON.stringify(content, null, 2) + '\n', 'utf-8');
}

function updateCargo(filePath, newVersion) {
  let content = readFileSync(filePath, 'utf-8');
  // Only replace the version in the [package] section (first occurrence)
  content = content.replace(
    /^(version\s*=\s*")[\d.]+(")/m,
    `$1${newVersion}$2`
  );
  writeFileSync(filePath, content, 'utf-8');
}

const type = process.argv[2];
if (!type) {
  console.error('Usage: node scripts/bump-version.mjs <patch|minor|major|X.Y.Z>');
  process.exit(1);
}

const current = getCurrentVersion();
const next = bumpVersion(current, type);

console.log(`Bumping version: ${current} → ${next}\n`);

updateJson(files.package, next);
console.log(`  ✓ package.json`);

updateJson(files.tauri, next);
console.log(`  ✓ src-tauri/tauri.conf.json`);

updateCargo(files.cargo, next);
console.log(`  ✓ src-tauri/Cargo.toml`);

console.log(`\nDone! Version is now ${next}`);
console.log(`\nNext steps:`);
console.log(`  git add -A && git commit -m "chore: bump version to ${next}"`);
console.log(`  git tag v${next}`);
console.log(`  git push && git push --tags`);
