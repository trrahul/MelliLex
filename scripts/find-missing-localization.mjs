/**
 * Find hardcoded strings in JSX that should be localized via t().
 *
 * Usage: node scripts/find-missing-localization.mjs
 *
 * Scans .tsx files for:
 *   - Text content in JSX elements (between > and <)
 *   - String props like title="...", placeholder="...", aria-label="..."
 *
 * Excludes:
 *   - CSS class names, data attributes, technical identifiers
 *   - Test files
 *   - Very short strings (< 2 chars) and purely numeric strings
 *   - Strings that are already locale keys or interpolated expressions
 */
import { readFileSync, readdirSync, statSync } from 'fs';
import { join, dirname, extname, relative } from 'path';
import { fileURLToPath } from 'url';

const __dirname = dirname(fileURLToPath(import.meta.url));
const srcDir = join(__dirname, '..', 'src');

// ── Collect source files ────────────────────────────────────────────────
function getSourceFiles(dir) {
  const files = [];
  for (const entry of readdirSync(dir)) {
    const fullPath = join(dir, entry);
    const stat = statSync(fullPath);
    if (stat.isDirectory()) {
      if (entry === 'node_modules' || entry === 'locales' || entry === '__tests__' || entry === 'test') continue;
      files.push(...getSourceFiles(fullPath));
    } else if (extname(entry) === '.tsx' && !entry.includes('.test.')) {
      files.push(fullPath);
    }
  }
  return files;
}

// Props that typically contain user-visible text
const TEXT_PROPS = ['title', 'placeholder', 'aria-label', 'aria-description', 'alt'];

// Strings to ignore (technical, CSS, or well-known constants)
const IGNORE_PATTERNS = [
  /^[a-z-]+$/, // CSS class fragments like "flex", "mb-4"
  /^#/, // hex colors
  /^\d+(\.\d+)?(%|px|rem|em|vh|vw)?$/, // numbers / dimensions
  /^(true|false|null|undefined)$/,
  /^(sm|md|lg|xl|2xl|xs)$/, // size variants
  /^(default|outline|ghost|secondary|destructive|link)$/, // button variants
  /^(button|submit|text|password|email|number|checkbox|radio)$/, // input types
  /^(div|span|p|h[1-6]|ul|ol|li|a|img|input|form|label|button|section)$/, // HTML tags
  /^[A-Z][a-zA-Z]+$/, // Component names like "Card", "Button"
  /^(GET|POST|PUT|DELETE|PATCH)$/, // HTTP methods
  /^https?:\/\//, // URLs
  /^[\s\W]+$/, // whitespace / punctuation only
  /^(MelliLex|N\/A)$/, // Brand names / constants
  /^\{/, // Already an expression
  /^\./, // Dot-prefixed (file extension etc.)
  /^void/, // TypeScript type annotations
  /Promise/, // TypeScript type annotations
];

function shouldIgnore(str) {
  const trimmed = str.trim();
  if (trimmed.length < 2) return true;
  if (IGNORE_PATTERNS.some(p => p.test(trimmed))) return true;
  return false;
}

const findings = [];

for (const filePath of getSourceFiles(srcDir)) {
  const content = readFileSync(filePath, 'utf8');
  const lines = content.split('\n');
  const relPath = relative(join(__dirname, '..'), filePath).replace(/\\/g, '/');

  for (let i = 0; i < lines.length; i++) {
    const line = lines[i];
    const lineNum = i + 1;

    // Skip import/comment lines
    if (line.trimStart().startsWith('import ') || line.trimStart().startsWith('//') || line.trimStart().startsWith('*')) continue;

    // 1. Check for hardcoded text props: title="...", placeholder="...", aria-label="..."
    for (const prop of TEXT_PROPS) {
      const propRegex = new RegExp(`${prop}="([^"]+)"`, 'g');
      let match;
      while ((match = propRegex.exec(line)) !== null) {
        const value = match[1];
        if (!shouldIgnore(value) && !value.includes('{') && !/^[a-z][-a-z0-9]*$/.test(value)) {
          findings.push({ file: relPath, line: lineNum, type: `prop:${prop}`, value });
        }
      }
    }

    // 2. Check for hardcoded JSX text content
    // Match text between > and < that isn't inside an expression
    const jsxTextRegex = />([^<>{]+)</g;
    let match;
    while ((match = jsxTextRegex.exec(line)) !== null) {
      const text = match[1].trim();
      if (text && !shouldIgnore(text) && !text.startsWith('{')) {
        // Filter out className-only content, single words that look like components, etc.
        if (/[a-zA-Z]{2,}/.test(text) && !/^[a-z][-a-z0-9\s]+$/.test(text)) {
          findings.push({ file: relPath, line: lineNum, type: 'jsx-text', value: text });
        }
      }
    }
  }
}

// ── Output ──────────────────────────────────────────────────────────────
console.log(`\n📊 Missing Localization Report`);
console.log(`══════════════════════════════`);
console.log(`Files scanned: ${getSourceFiles(srcDir).length}`);
console.log(`Potential hardcoded strings: ${findings.length}`);
console.log();

if (findings.length > 0) {
  // Group by file
  const grouped = {};
  for (const f of findings) {
    if (!grouped[f.file]) grouped[f.file] = [];
    grouped[f.file].push(f);
  }

  for (const [file, items] of Object.entries(grouped).sort()) {
    console.log(`📄 ${file}`);
    for (const item of items) {
      console.log(`   L${item.line} [${item.type}] "${item.value}"`);
    }
    console.log();
  }
} else {
  console.log(`✅ No hardcoded strings found!`);
}

console.log(`ℹ️  Note: This tool may report false positives. Review each finding manually.`);
console.log(`   Strings in test files, locales, and __tests__ directories are excluded.`);
