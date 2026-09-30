// Packs the npm package, unpacks it into a clean directory, and imports it the way a
// consumer would. The unpacked copy sits under node_modules/.cache so its dependencies
// resolve from the repository's node_modules without a network install.
import { execFileSync } from 'node:child_process';
import { mkdirSync, rmSync, writeFileSync } from 'node:fs';
import { join, resolve } from 'node:path';

const root = resolve(import.meta.dirname, '..');
const dir = join(root, 'node_modules/.cache/flatgeobuf-pack-test');
const pkg = join(dir, 'node_modules/flatgeobuf');
rmSync(dir, { recursive: true, force: true });
mkdirSync(pkg, { recursive: true });

const tarball = execFileSync('npm', ['pack', '--silent', '--pack-destination', dir], {
    cwd: root,
    encoding: 'utf8',
}).trim();
execFileSync('tar', ['-xzf', join(dir, tarball), '-C', pkg, '--strip-components=1']);

writeFileSync(
    join(dir, 'check.mjs'),
    `import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';

let logged = 0;
for (const level of ['debug', 'log', 'info']) console[level] = () => logged++;

const root = await import('flatgeobuf');
assert.ok(root.geojson && root.generic && root.ol, 'root exports geojson, generic and ol');
const { deserialize } = await import('flatgeobuf/geojson');
assert.equal(typeof (await import('flatgeobuf/generic')).deserialize, 'function');
assert.equal(typeof (await import('flatgeobuf/ol')).deserialize, 'function');
assert.equal(typeof (await import('flatgeobuf/lib/mjs/geojson.js')).deserialize, 'function', 'deep imports still resolve');

const bytes = new Uint8Array(readFileSync(${JSON.stringify(join(root, 'test/data/UScounties.fgb'))}));
const rect = { minX: -106.88, minY: 36.75, maxX: -101.11, maxY: 41.24 };
let features = 0;
for await (const _ of deserialize(bytes, { rect })) features++;
assert.equal(features, 86);
assert.equal(logged, 0, 'the packaged build does not log');
`,
);
execFileSync('node', ['check.mjs'], { cwd: dir, stdio: 'inherit' });
rmSync(dir, { recursive: true, force: true });
console.log(`${tarball}: imports resolve and a bbox query runs without logging`);
