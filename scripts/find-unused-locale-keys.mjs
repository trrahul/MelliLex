/**
 * Find unused locale keys — keys defined in en.json but not referenced in source code.
 *
 * Usage: node scripts/find-unused-locale-keys.mjs
 *
 * Patterns detected:
 *   t('key')  t("key")  i18n.t('key')  i18nKey="key"
 *   Dynamic patterns like t(`prefix.${var}`) are flagged as partial matches.
 */
import { readFileSync, readdirSync, statSync } from 'fs';
import { join, dirname, extname } from 'path';
import { fileURLToPath } from 'url';

const __dirname = dirname(fileURLToPath(import.meta.url));
const srcDir = join(__dirname, '..', 'src');
const localesDir = join(srcDir, 'locales');

// ── Collect all keys from en.json ──────────────────────────────────────
function flattenKeys(obj, prefix = '') {
  const keys = [];
  for (const [key, value] of Object.entries(obj)) {
    const fullKey = prefix ? `${prefix}.${key}` : key;
    if (typeof value === 'object' && value !== null && !Array.isArray(value)) {
      keys.push(...flattenKeys(value, fullKey));
    } else {
      keys.push(fullKey);
    }
  }
  return keys;
}

const enData = JSON.parse(readFileSync(join(localesDir, 'en.json'), 'utf8'));
const allKeys = flattenKeys(enData);

// ── Collect source files ────────────────────────────────────────────────
function getSourceFiles(dir) {
  const files = [];
  for (const entry of readdirSync(dir)) {
    const fullPath = join(dir, entry);
    const stat = statSync(fullPath);
    if (stat.isDirectory()) {
      if (entry === 'node_modules' || entry === 'locales') continue;
      files.push(...getSourceFiles(fullPath));
    } else if (['.ts', '.tsx', '.js', '.jsx'].includes(extname(entry))) {
      files.push(fullPath);
    }
  }
  return files;
}

const sourceFiles = getSourceFiles(srcDir);
const allSource = sourceFiles.map(f => readFileSync(f, 'utf8')).join('\n');

// ── Extract statically referenced keys ─────────────────────────────────
// Matches: t('key'), t("key"), i18n.t('key'), i18nKey="key"
const staticKeyPattern = /\bt\(\s*['"]([^'"]+)['"]/gm;
const i18nKeyPattern = /i18nKey\s*=\s*["']([^"']+)["']/gm;
// Also match string literals that look like locale keys (e.g. assigned to variables)
// Pattern: 'namespace.key' or "namespace.key" where it has at least one dot
const stringLiteralKeyPattern = /['"]([a-z][a-zA-Z]*(?:\.[a-zA-Z][a-zA-Z0-9]*)+)['"]/gm;

const usedKeys = new Set();
let match;

while ((match = staticKeyPattern.exec(allSource)) !== null) {
  usedKeys.add(match[1]);
}
while ((match = i18nKeyPattern.exec(allSource)) !== null) {
  usedKeys.add(match[1]);
}
while ((match = stringLiteralKeyPattern.exec(allSource)) !== null) {
  // Only add if it looks like an actual locale key (matches a known top-level namespace)
  const candidate = match[1];
  const topLevel = candidate.split('.')[0];
  if (enData[topLevel]) usedKeys.add(candidate);
}

// ── Detect dynamic key prefixes (template literals) ─────────────────────
// e.g. t(`explore.${section}Card.title`) → prefix "explore."
const dynamicPattern = /\bt\(\s*`([^`]+)`/gm;
const dynamicPrefixes = [];

while ((match = dynamicPattern.exec(allSource)) !== null) {
  const template = match[1];
  const staticPrefix = template.split('${')[0];
  if (staticPrefix) dynamicPrefixes.push(staticPrefix);
}

// ── Determine unused keys ───────────────────────────────────────────────
function isUsed(key) {
  if (usedKeys.has(key)) return true;
  // Check if key matches a dynamic prefix
  for (const prefix of dynamicPrefixes) {
    if (key.startsWith(prefix)) return true;
  }
  // Check if any used key is a parent of this key (returnObjects usage)
  for (const used of usedKeys) {
    if (key.startsWith(used + '.')) return true;
  }
  return false;
}

const unusedKeys = allKeys.filter(key => !isUsed(key));

// ── i18next plural suffixes ─────────────────────────────────────────────
const pluralSuffixes = ['_one', '_other', '_zero', '_two', '_few', '_many'];
const trueUnused = [];
const pluralVariants = [];

for (const key of unusedKeys) {
  const isPluralVariant = pluralSuffixes.some(suffix => {
    if (key.endsWith(suffix)) {
      const baseKey = key.slice(0, -suffix.length);
      return isUsed(baseKey) || usedKeys.has(baseKey);
    }
    return false;
  });
  if (isPluralVariant) {
    pluralVariants.push(key);
  } else {
    trueUnused.push(key);
  }
}

// ── Output ──────────────────────────────────────────────────────────────
console.log(`\n📊 Locale Key Usage Report`);
console.log(`══════════════════════════`);
console.log(`Total keys in en.json: ${allKeys.length}`);
console.log(`Keys referenced in source: ${allKeys.length - unusedKeys.length}`);
console.log(`Unused keys: ${trueUnused.length}`);
if (pluralVariants.length > 0) {
  console.log(`Plural variants (OK): ${pluralVariants.length}`);
}
console.log();

if (trueUnused.length > 0) {
  console.log(`⚠️  Potentially unused keys:`);
  console.log(`─────────────────────────`);
  // Group by top-level namespace
  const grouped = {};
  for (const key of trueUnused) {
    const ns = key.split('.')[0];
    if (!grouped[ns]) grouped[ns] = [];
    grouped[ns].push(key);
  }
  for (const [ns, keys] of Object.entries(grouped).sort()) {
    console.log(`\n  [${ns}]`);
    for (const key of keys) {
      console.log(`    ${key}`);
    }
  }
  console.log();
}

if (dynamicPrefixes.length > 0) {
  console.log(`ℹ️  Dynamic key prefixes detected (keys under these may be used dynamically):`);
  for (const prefix of [...new Set(dynamicPrefixes)].sort()) {
    console.log(`    ${prefix}*`);
  }
  console.log();
}

console.log(`✅ Done. Review the unused keys above and remove them if no longer needed.`);
