//Tests the npm package in pkg in real browsers.  Run it with:  npm run test:browser
//The browsers have to be installed one time with:  npx playwright install chromium webkit firefox
//
//The package is loaded the three ways a browser can load it: without a bundler, with Vite and with webpack.
//Every page runs the checks in checks.js, and this file reads the results.
//To use only some of the browsers, name them:  npm run test:browser -- chromium webkit

import { readFile, writeFile } from 'node:fs/promises';
import { createServer } from 'node:http';
import { dirname, extname, join, normalize } from 'node:path';
import { fileURLToPath } from 'node:url';
import { chromium, firefox, webkit } from 'playwright';
import { build } from 'vite';
import wasm from 'vite-plugin-wasm';
import webpack from 'webpack';

const browserDir = dirname(fileURLToPath(import.meta.url));
const root = join(browserDir, '..', '..');
const dist = join(browserDir, 'dist');

const allBrowsers = [
    ['chromium', 'Chromium', chromium],
    ['webkit', 'WebKit, which Safari uses', webkit],
    ['firefox', 'Firefox', firefox]
];
const names = process.argv.slice(2);
const unknownNames = names.filter(x => !allBrowsers.some(y => y[0] === x));
if (unknownNames.length > 0) {
    console.error(`Unknown browser: ${unknownNames.join(', ')}.  The browsers are ${allBrowsers.map(x => x[0]).join(', ')}.`);
    process.exit(1);
}
const browsers = allBrowsers.filter(x => names.length === 0 || names.includes(x[0]));

async function buildWithVite() {
    await build({
        root: browserDir,
        configFile: false,
        logLevel: 'error',
        plugins: [wasm()],
        build: {
            //the wasm is loaded with a top level await
            target: 'esnext',
            outDir: join(dist, 'vite'),
            emptyOutDir: true,
            rollupOptions: { input: join(browserDir, 'bundler.html') }
        }
    });
}

async function buildWithWebpack() {
    const outDir = join(dist, 'webpack');
    const stats = await new Promise((resolve, reject) => {
        webpack({
            mode: 'production',
            context: browserDir,
            entry: './main.js',
            output: { path: outDir, filename: 'main.js', clean: true },
            experiments: { asyncWebAssembly: true },
            performance: { hints: false }
        }, (err, stats) => err ? reject(err) : resolve(stats));
    });
    if (stats.hasErrors()) {
        throw new Error(stats.toString({ all: false, errors: true }));
    }
    await writeFile(join(outDir, 'index.html'), '<!doctype html>\n<meta charset="utf-8">\n<title>adv-random with webpack</title>\n<script src="main.js"></script>\n');
}

const contentTypes = { '.html': 'text/html', '.js': 'text/javascript', '.wasm': 'application/wasm' };

//Serves the files of a folder.  A page that is opened from a file can not load wasm, so the pages need a server.
function serve(folder) {
    return new Promise(resolve => {
        const server = createServer(async (request, response) => {
            const path = normalize(decodeURIComponent(request.url.split('?')[0]));
            try {
                const body = await readFile(join(folder, path));
                response.writeHead(200, { 'content-type': contentTypes[extname(path)] ?? 'application/octet-stream' });
                response.end(body);
            } catch {
                response.writeHead(404);
                response.end('Not found');
            }
        });
        server.listen(0, '127.0.0.1', () => resolve(server));
    });
}

//Opens the page and returns what is wrong with it.  Nothing is wrong when it returns an empty list.
async function testPage(browser, folder, path) {
    const server = await serve(folder);
    const page = await browser.newPage();
    const problems = [];
    page.on('pageerror', e => problems.push(`error: ${e.message}`));
    page.on('console', message => {
        if (message.type() === 'error' || message.type() === 'warning') {
            problems.push(`console ${message.type()}: ${message.text()}`);
        }
    });
    try {
        await page.goto(`http://127.0.0.1:${server.address().port}${path}`);
        await page.waitForFunction(() => window.__results, null, { timeout: 15000 });
        const results = await page.evaluate(() => window.__results);
        for (const [name, result] of Object.entries(results)) {
            if (result !== 'ok') {
                problems.push(`${name}: ${result}`);
            }
        }
        return { checks: Object.keys(results).length, problems };
    } catch (e) {
        problems.push(`the page did not finish: ${e.message.split('\n')[0]}`);
        return { checks: 0, problems };
    } finally {
        await page.close();
        server.close();
    }
}

await buildWithVite();
await buildWithWebpack();
const pages = [
    ['without a bundler', root, '/tests/browser/plain.html'],
    ['Vite', join(dist, 'vite'), '/bundler.html'],
    ['webpack', join(dist, 'webpack'), '/index.html']
];

let failed = 0;
for (const [, title, browserType] of browsers) {
    let browser;
    try {
        browser = await browserType.launch();
    } catch (e) {
        console.log(`\n${title}\n  FAIL  could not start: ${e.message.split('\n')[0]}\n        Install the browsers with:  npx playwright install chromium webkit firefox`);
        failed++;
        continue;
    }
    console.log(`\n${title} ${browser.version()}`);
    for (const [name, folder, path] of pages) {
        const { checks, problems } = await testPage(browser, folder, path);
        if (problems.length === 0) {
            console.log(`  pass  ${name} (${checks} checks)`);
        } else {
            failed++;
            console.log(`  FAIL  ${name}`);
            for (const problem of problems) {
                console.log(`        ${problem}`);
            }
        }
    }
    await browser.close();
}

console.log(failed === 0 ? '\nEvery page passed' : `\n${failed} failed`);
process.exit(failed === 0 ? 0 : 1);
