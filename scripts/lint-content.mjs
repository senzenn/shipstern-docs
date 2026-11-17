#!/usr/bin/env node
// Content linter for content/docs. Enforces the rules in AGENTS.md that a
// build cannot catch: frontmatter, banned words and second-person wording,
// em dashes, callout count, internal links and anchors, and links to the docs
// repository instead of Shipstern.
//
// Usage: node scripts/lint-content.mjs   (exit code 1 on any problem)

import { readFileSync, readdirSync, statSync } from 'node:fs';
import { join, relative, sep } from 'node:path';
import GithubSlugger from 'github-slugger';

const ROOT = 'content/docs';
const REQUIRED = ['title', 'description', 'llmsDescription'];
const SECOND_PERSON = ['you', 'your', 'yours', 'yourself'];
const BANNED = [
  'simply', 'just', 'easy', 'easily', 'please', 'leverage', 'utilize', 'utilise',
  'seamless', 'seamlessly', 'supercharge', 'unlock', 'in order to', 'powerful',
  'robust', 'effortless', 'blazing', 'cutting-edge', 'game-changer',
  ...SECOND_PERSON,
];
const MAX_CALLOUTS = 2;

function walk(dir) {
  return readdirSync(dir).flatMap((name) => {
    const path = join(dir, name);
    return statSync(path).isDirectory() ? walk(path) : path.endsWith('.mdx') ? [path] : [];
  });
}

/** content/docs/(docs)/runtime/index.mdx -> /docs/runtime */
function urlFor(file) {
  const parts = relative(ROOT, file)
    .replace(/\.mdx$/, '')
    .split(sep)
    .filter((p) => !/^\(.*\)$/.test(p));
  if (parts.at(-1) === 'index') parts.pop();
  return '/' + ['docs', ...parts].join('/');
}

/** Strip fenced code, inline code, includes, JSX attributes and URLs so only prose is checked. */
function prose(body) {
  return body
    .replace(/```[\s\S]*?```/g, '')
    .replace(/`[^`\n]*`/g, '')
    .replace(/<include[\s\S]*?<\/include>/g, '')
    .replace(/\{\/\*[\s\S]*?\*\/\}/g, '')
    .replace(/\(https?:\/\/[^)]*\)/g, '')
    .replace(/<[^>]+>/g, '');
}

/**
 * Anchor IDs a page renders: headings (slugged the way Fumadocs does) and the
 * `<Update version="v0.11.0">` entries on release pages (`v0-11-0`).
 */
function anchorsFor(text) {
  const slugger = new GithubSlugger();
  const ids = new Set();
  const noCode = text.replace(/```[\s\S]*?```/g, '');
  for (const [, heading] of noCode.matchAll(/^#{2,6} (.+)$/gm)) {
    ids.add(slugger.slug(heading.replace(/`/g, '').replace(/\[([^\]]*)\]\([^)]*\)/g, '$1')));
  }
  for (const [, version] of noCode.matchAll(/<Update[^>]*version="([^"]+)"/g)) ids.add(version.replaceAll('.', '-'));
  return ids;
}

const files = walk(ROOT);
const anchors = new Map(files.map((f) => [urlFor(f), anchorsFor(readFileSync(f, 'utf8'))]));
const urls = new Set(anchors.keys());

/** Report a /docs link whose page or #anchor does not exist. */
function checkLink(href, report) {
  const [rawPath, hash] = href.split('#');
  const path = rawPath.replace(/\/$/, '');
  if (!urls.has(path)) return report(`broken internal link ${href}`);
  if (hash && !anchors.get(path).has(decodeURIComponent(hash))) report(`link to missing anchor ${href}`);
}
const problems = [];

for (const file of files) {
  const text = readFileSync(file, 'utf8');
  const fm = /^---\n([\s\S]*?)\n---\n/.exec(text);
  const body = fm ? text.slice(fm[0].length) : text;
  const report = (msg) => problems.push(`${file}: ${msg}`);

  if (!fm) report('missing frontmatter');
  else {
    for (const key of REQUIRED) if (!new RegExp(`^${key}:`, 'm').test(fm[1])) report(`frontmatter is missing "${key}"`);
    for (const line of fm[1].split('\n')) {
      if (/^\w+: [^"'].*: /.test(line)) report(`quote this frontmatter value, it contains ": ": ${line.slice(0, 50)}`);
    }
  }

  const plain = prose(body);
  const lines = plain.split('\n');
  lines.forEach((line, i) => {
    if (line.includes('—')) report(`em dash in prose: "${line.trim().slice(0, 60)}"`);
    for (const word of BANNED) {
      if (new RegExp(`\\b${word}\\b`, 'i').test(line)) report(`avoid "${word}": "${line.trim().slice(0, 60)}"`);
    }
  });
  if (fm && fm[1].includes('—')) report('em dash in frontmatter');
  if (fm && new RegExp(`\\b(${SECOND_PERSON.join('|')})\\b`, 'i').test(fm[1])) report('second-person wording in frontmatter');

  const callouts = (body.match(/<(Note|Tip|Warning|Info|Callout)[\s>]/g) ?? []).length;
  if (callouts > MAX_CALLOUTS) report(`${callouts} callouts (max ${MAX_CALLOUTS})`);

  for (const [, href] of body.matchAll(/\]\((\/docs[^)\s]*)\)/g)) checkLink(href, report);
  for (const [, href] of body.matchAll(/href="(\/docs[^"]*)"/g)) checkLink(href, report);

  // GitHub links go to the Shipstern repository, never to this docs repo.
  for (const [match] of body.matchAll(/github\.com\/senzenn\/shipstern-docs[^\s)"]*/g)) {
    report(`link to the docs repo, use https://github.com/solana-rpc/shipstern instead: ${match}`);
  }
}

if (problems.length) {
  console.error(problems.join('\n'));
  console.error(`\n${problems.length} problem(s) in ${files.length} pages.`);
  process.exit(1);
}
console.log(`content ok: ${files.length} pages`);
