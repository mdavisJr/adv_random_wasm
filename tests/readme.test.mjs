//Runs every JavaScript example in README.md, so the examples always work with the current version.

import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { test } from 'node:test';
import * as advRandom from 'adv-random';

const readme = readFileSync(new URL('../README.md', import.meta.url), 'utf8');

//Returns the code of every ```js block
export function examples() {
    return [...readme.matchAll(/```js\n([\s\S]*?)```/g)].map(x => x[1]);
}

//Runs an example and returns what it printed.  The import of the package is changed into a variable, because code
//that is run from a string can not import.
export function run(example) {
    const code = example.replace(/^import (\{[^}]+\}) from 'adv-random';$/m, 'const $1 = advRandom;');
    assert.ok(!/^import /m.test(code), 'an example can only import from adv-random');
    const printed = [];
    const console = { log: (...values) => printed.push(values) };
    new Function('advRandom', 'console', code)(advRandom, console);
    return printed;
}

const allExamples = examples();

test('README has examples', () => {
    assert.ok(allExamples.length >= 30);
});

for (const [index, example] of allExamples.entries()) {
    const title = example.split('\n').find(x => x.startsWith('//')) ?? example.split('\n')[0];
    test(`README example ${index + 1}: ${title}`, () => {
        for (let i = 0; i < 20; i++) {
            assert.ok(run(example).length >= 1, 'the example did not print anything');
        }
    });
}
