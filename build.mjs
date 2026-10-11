//Builds the npm package into pkg.  Run it with:  npm run build
//wasm-pack builds for one target at a time, so it is run once for Node, once for bundlers and once for browsers
//without a bundler, and the three builds are put into one package.

import { execFileSync } from 'node:child_process';
import { copyFileSync, readFileSync, readdirSync, rmSync, writeFileSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';

const root = dirname(fileURLToPath(import.meta.url));
const pkg = join(root, 'pkg');
const targets = ['nodejs', 'bundler', 'web'];
const outName = 'adv_random';

const cargoToml = readFileSync(join(root, 'Cargo.toml'), 'utf8');
const version = cargoToml.match(/^version\s*=\s*"([^"]+)"/m)[1];

rmSync(pkg, { recursive: true, force: true });
for (const target of targets) {
    const outDir = join(pkg, target);
    execFileSync('wasm-pack', ['build', '--release', '--target', target, '--out-dir', outDir, '--out-name', outName, '--no-pack'], { cwd: root, stdio: 'inherit' });
    //only the .js, .d.ts and .wasm files are kept
    for (const file of readdirSync(outDir)) {
        if (!/\.(js|ts|wasm)$/.test(file)) {
            rmSync(join(outDir, file), { recursive: true, force: true });
        }
    }
    //the Node build is CommonJS and the other builds are ES modules, so each folder says which one it is
    const type = target === 'nodejs' ? 'commonjs' : 'module';
    writeFileSync(join(outDir, 'package.json'), JSON.stringify({ type }, null, 2) + '\n');
}

const packageJson = {
    name: 'adv-random',
    version,
    description: 'Create random numbers, passwords, or strings based on rules',
    keywords: ['rand', 'random', 'rule', 'string', 'password', 'wasm'],
    author: 'Mark Davis <markeyd83@gmail.com>',
    license: 'MIT OR Apache-2.0',
    repository: { type: 'git', url: 'git+https://github.com/mdavisJr/adv_random_wasm.git' },
    main: `nodejs/${outName}.js`,
    module: `bundler/${outName}.js`,
    types: `bundler/${outName}.d.ts`,
    exports: {
        '.': {
            types: `./bundler/${outName}.d.ts`,
            node: `./nodejs/${outName}.js`,
            default: `./bundler/${outName}.js`
        },
        //for a browser without a bundler.  The wasm is loaded by calling the default export.
        './web': {
            types: `./web/${outName}.d.ts`,
            default: `./web/${outName}.js`
        }
    },
    files: ['nodejs', 'bundler', 'web', 'README.md', 'LICENSE-MIT', 'LICENSE-APACHE'],
    //the bundler build starts the wasm when it is imported
    sideEffects: [`./bundler/${outName}.js`]
};
writeFileSync(join(pkg, 'package.json'), JSON.stringify(packageJson, null, 2) + '\n');
for (const file of ['README.md', 'LICENSE-MIT', 'LICENSE-APACHE']) {
    copyFileSync(join(root, file), join(pkg, file));
}
console.log(`\nBuilt adv-random ${version} into ${pkg}`);
