//Tests the npm package in pkg.  Run them with:  npm run build && npm test
//The package is imported by its name, so the exports in its package.json are tested too.

import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { createRequire } from 'node:module';
import { test } from 'node:test';
import {
    DefaultRandom, ExcludeNumberSets, NoDuplicate, NumberPool, NumberPoolByIndex, NumberRange, NumberSpace, OddEven,
    OddEvenByIndex, OperatorType, ProcessNumberSpaceItems, RandomError, RandomErrorType, RangeInclusive, SeededRandom,
    Sequential, Settings, range
} from 'adv-random';

const UPPER_CASE = 'ABCDEFGHIJKLMNOPQRSTUVWXYZ';
const SPECIAL_CHARACTERS = '!@#$%^&*()-+=';

function sorted(numbers) {
    return [...numbers].sort((a, b) => a - b);
}

function spaces(numbers) {
    const sortedNumbers = sorted(numbers);
    return sortedNumbers.slice(1).map((x, i) => x - sortedNumbers[i]);
}

function count(string, characters) {
    return [...string].filter(x => characters.includes(x)).length;
}

function assertInRange(numbers, min, max) {
    assert.ok(numbers.every(x => Number.isInteger(x) && x >= min && x <= max), `${numbers} are not all from ${min} to ${max}`);
}

//Runs the settings many times, because one run of random numbers can pass by luck
function generateMany(settings, check) {
    for (let i = 0; i < 50; i++) {
        check(settings.generate());
    }
}

function throwsRandomError(fn, errorType) {
    assert.throws(fn, e => {
        assert.ok(e instanceof RandomError);
        assert.equal(e.errorType(), errorType);
        return true;
    });
}

test('10 random numbers between 1 and 20', () => {
    generateMany(Settings.builder(10).range(1, 20).build(), randomNumbers => {
        const numbers = randomNumbers.numbers();
        assert.ok(Array.isArray(numbers));
        assert.equal(numbers.length, 10);
        assertInRange(numbers, 1, 20);
        assert.ok(randomNumbers.attempts() >= 1);
    });
});

test('no duplicates', () => {
    generateMany(Settings.builder(10).range(1, 20).noDuplicates().build(), randomNumbers => {
        assert.equal(new Set(randomNumbers.numbers()).size, 10);
    });
});

test('NoDuplicate can be the class or an object', () => {
    for (const rule of [NoDuplicate, NoDuplicate.new(), new NoDuplicate()]) {
        generateMany(Settings.builder(10).range(1, 20).rule(rule).build(), randomNumbers => {
            assert.equal(new Set(randomNumbers.numbers()).size, 10);
        });
        generateMany(Settings.builder(10).range(1, 20).exclude(rule).build(), randomNumbers => {
            assert.ok(new Set(randomNumbers.numbers()).size < 10);
        });
    }
});

test('phone number with a range for some indexes', () => {
    const settings = Settings.builder(3).rule(NumberRange.new().at([0, 1], 100, 999).at([2], 1000, 9999)).build();
    generateMany(settings, randomNumbers => {
        const numbers = randomNumbers.numbers();
        assertInRange(numbers.slice(0, 2), 100, 999);
        assertInRange(numbers.slice(2), 1000, 9999);
    });
});

test('excluded ranges', () => {
    const settings = Settings.builder(10)
        .range(1, 100)
        .noDuplicates()
        .exclude(NumberRange.all(20, 29))
        .exclude(NumberRange.all(51, 75))
        .build();
    generateMany(settings, randomNumbers => {
        assert.ok(randomNumbers.numbers().every(x => !(x >= 20 && x <= 29) && !(x >= 51 && x <= 75)));
    });
});

test('number pool with noneOf, exactly and atLeast', () => {
    const settings = Settings.builder(10)
        .range(1, 100)
        .noDuplicates()
        .rule(NumberPool.new()
            .noneOf([23])
            .exactly(2, [1, 4])
            .atLeast(3, [17, 18, 19, 20, 21]))
        .build();
    generateMany(settings, randomNumbers => {
        const numbers = randomNumbers.numbers();
        assert.ok(!numbers.includes(23));
        assert.ok(numbers.includes(1) && numbers.includes(4));
        assert.ok(numbers.filter(x => x >= 17 && x <= 21).length >= 3);
    });
});

test('number pool with atMost, between and item', () => {
    const settings = Settings.builder(10)
        .range(1, 100)
        .noDuplicates()
        .rule(NumberPool.new()
            .atMost(1, range(1, 10))
            .between(2, 3, range(11, 20))
            .item('thirties', new Set([30, 31, 32, 33, 34]), OperatorType.Gte(2)))
        .build();
    generateMany(settings, randomNumbers => {
        const numbers = randomNumbers.numbers();
        assert.ok(numbers.filter(x => x <= 10).length <= 1);
        const teens = numbers.filter(x => x >= 11 && x <= 20).length;
        assert.ok(teens >= 2 && teens <= 3);
        assert.ok(numbers.filter(x => x >= 30 && x <= 34).length >= 2);
    });
});

test('a pool can be a list, a typed array, a set, a range, an object with min and max, or a string', () => {
    for (const pool of [[65, 66, 67], new Uint32Array([65, 66, 67]), new Set([65, 66, 67]), range(65, 67), new RangeInclusive(65, 67), { min: 65, max: 67 }, 'ABC']) {
        const numbers = Settings.builder(6).rule(NumberPool.new().exactly(6, pool)).build().generate().numbers();
        assertInRange(numbers, 65, 67);
    }
});

test('odd and even by index', () => {
    const settings = Settings.builder(5).range(1, 10).noDuplicates().rule(OddEvenByIndex.new().odd([0, 2, 4]).even([1, 3])).build();
    generateMany(settings, randomNumbers => {
        assert.deepEqual(randomNumbers.numbers().map(x => x % 2), [1, 0, 1, 0, 1]);
    });
});

test('odd and even counts', () => {
    generateMany(Settings.builder(10).range(1, 20).noDuplicates().oddEven(5, 5).build(), randomNumbers => {
        assert.equal(randomNumbers.numbers().filter(x => x % 2 === 1).length, 5);
    });
    generateMany(Settings.builder(10).range(1, 20).rule(OddEven.new().odd(0).even(10)).build(), randomNumbers => {
        assert.ok(randomNumbers.numbers().every(x => x % 2 === 0));
    });
});

test('sequential', () => {
    generateMany(Settings.builder(5).range(1, 10).noDuplicates().rule(Sequential.new().singles(5)).build(), randomNumbers => {
        assert.ok(spaces(randomNumbers.numbers()).every(x => x > 1));
    });
    generateMany(Settings.builder(5).range(1, 10).noDuplicates().rule(Sequential.new().singles(3).inARow(2)).build(), randomNumbers => {
        assert.equal(spaces(randomNumbers.numbers()).filter(x => x === 1).length, 1);
    });
    generateMany(Settings.builder(5).range(1, 10).noDuplicates().rule(Sequential.new().singles(1).inARowEach([2, 2])).build(), randomNumbers => {
        assert.equal(spaces(randomNumbers.numbers()).filter(x => x === 1).length, 2);
    });
});

test('alphanumeric strings', () => {
    generateMany(Settings.builder(20).rule(NumberPool.alphanumeric(20)).build(), randomNumbers => {
        const string = randomNumbers.shuffledString();
        assert.match(string, /^[a-zA-Z0-9]{20}$/);
        const digits = count(string, '0123456789');
        assert.ok(digits >= 1 && digits <= 3);
        //the same characters in a different order
        assert.equal([...randomNumbers.string()].sort().join(''), [...string].sort().join(''));
    });
    generateMany(Settings.builder(20).rule(NumberPool.alphanumeric(20).withSpecialCharacters()).build(), randomNumbers => {
        const specialCharacters = count(randomNumbers.shuffledString(), SPECIAL_CHARACTERS);
        assert.ok(specialCharacters >= 1 && specialCharacters <= 2);
    });
    generateMany(Settings.builder(25).noDuplicates().rule(NumberPool.alphanumeric(25).digits(10).specialCharacters(5)).build(), randomNumbers => {
        const string = randomNumbers.shuffledString();
        assert.equal(string.length, 25);
        assert.equal(new Set(string).size, 25);
        assert.equal(count(string, '0123456789'), 10);
        assert.equal(count(string, SPECIAL_CHARACTERS), 5);
    });
});

test('string from pools of characters', () => {
    const settings = Settings.builder(15)
        .rule(NumberPool.new()
            .exactly(11, UPPER_CASE)
            .exactly(2, '365')
            .exactly(2, '#*'))
        .build();
    generateMany(settings, randomNumbers => {
        const string = randomNumbers.shuffledString();
        assert.equal(count(string, UPPER_CASE), 11);
        assert.equal(count(string, '365'), 2);
        assert.equal(count(string, '#*'), 2);
    });
});

test('license number with pools by index and excluded strings', () => {
    const settings = Settings.builder(9)
        .rule(ExcludeNumberSets.new().string('AB1234567').string('CB1234567'))
        .rule(NumberPoolByIndex.new()
            .at([0, 1], UPPER_CASE)
            .at(range(2, 8), '1234567890'))
        .build();
    generateMany(settings, randomNumbers => {
        assert.match(randomNumbers.string(), /^[A-Z]{2}[0-9]{7}$/);
    });
});

test('excluded sets of numbers', () => {
    //1, 2 and 3 can be in 6 orders, and 5 of them are excluded
    const excludeNumberSets = ExcludeNumberSets.new()
        .numbers([1, 2, 3])
        .numbersEach([[1, 3, 2], [2, 1, 3], [2, 3, 1]])
        .stringEach(['\u0003\u0001\u0002']);
    generateMany(Settings.builder(3).range(1, 3).noDuplicates().rule(excludeNumberSets).build(), randomNumbers => {
        assert.deepEqual(randomNumbers.numbers(), [3, 2, 1]);
    });
});

test('number space', () => {
    const operatorTypes = [
        [OperatorType.Lt(3), x => x < 3],
        [OperatorType.Lte(3), x => x <= 3],
        [OperatorType.Eq(3), x => x === 3],
        [OperatorType.Gte(3), x => x >= 3],
        [OperatorType.Gt(3), x => x > 3],
        [OperatorType.Between(3, 6), x => x >= 3 && x <= 6]
    ];
    for (const [operatorType, isMatch] of operatorTypes) {
        const settings = Settings.builder(10).range(1, 100).rule(NumberSpace.new().run(operatorType, 9)).build();
        generateMany(settings, randomNumbers => {
            const numberSpaces = spaces(randomNumbers.numbers());
            assert.ok(numberSpaces.every(isMatch), `${operatorType}: ${numberSpaces}`);
        });
    }
    const settings = Settings.builder(10)
        .range(1, 100)
        .rule(NumberSpace.new().runs(OperatorType.Eq(3), [9]).process(ProcessNumberSpaceItems.OnePerRequest))
        .build();
    assert.ok(spaces(settings.generate().numbers()).every(x => x === 3));
});

test('the same seed gives the same numbers', () => {
    const settings = seed => Settings.builder(10).range(1, 20).random(SeededRandom.new(seed)).build();
    //these are the numbers that adv_random 3.0.0 gives in Rust for the same settings and seed
    assert.deepEqual(settings(42).generate().numbers(), [5, 13, 20, 9, 4, 3, 17, 11, 15, 11]);
    assert.deepEqual(settings(42n).generate().numbers(), [5, 13, 20, 9, 4, 3, 17, 11, 15, 11]);
    assert.deepEqual(new SeededRandom(42) instanceof SeededRandom, true);
    assert.notDeepEqual(settings(43).generate().numbers(), settings(42).generate().numbers());

    const withRules = Settings.builder(5)
        .range(1, 39)
        .noDuplicates()
        .oddEven(3, 2)
        .rule(Sequential.new().singles(3).inARow(2))
        .random(SeededRandom.new(2026))
        .build();
    assert.deepEqual(withRules.generate().numbers(), [25, 2, 26, 29, 17]);

    const string = seed => Settings.builder(20).rule(NumberPool.alphanumeric(20).withSpecialCharacters()).random(SeededRandom.new(seed)).build().generate().shuffledString();
    assert.equal(string(7), string(7));
});

test('numbers up to 4294967295', () => {
    const numbers = Settings.builder(6).range(0, 4000000000).random(SeededRandom.new(1)).build().generate().numbers();
    assert.deepEqual(numbers, [3188318561, 865487534, 2071599301, 3989672609, 623384694, 1645920958]);
    assert.deepEqual(Settings.builder(2).range(4294967295, 4294967295).build().generate().numbers(), [4294967295, 4294967295]);
});

test('a random number generator written in JavaScript', () => {
    let next = 0;
    const countingRandom = {
        getNumber(min, max) {
            return min + (next++) % (max - min + 1);
        },
        getBool() {
            return this.getNumber(0, 1) === 1;
        }
    };
    const numbers = Settings.builder(5).range(1, 20).random(countingRandom).build().generate().numbers();
    //the same numbers that the CountingRandom example gives in Rust
    assert.deepEqual(numbers, [3, 6, 9, 12, 15]);

    const mathRandom = {
        getNumber: (min, max) => min + Math.floor(Math.random() * (max - min + 1)),
        getBool: () => Math.random() < 0.5
    };
    const string = Settings.builder(20).rule(NumberPool.alphanumeric(20)).random(mathRandom).build().generate().shuffledString();
    assert.match(string, /^[a-zA-Z0-9]{20}$/);

    assertInRange(Settings.builder(5).range(1, 20).random(DefaultRandom.new()).build().generate().numbers(), 1, 20);
});

test('errors from a random number generator written in JavaScript are thrown', () => {
    const throws = { getNumber() { throw new Error('boom'); }, getBool() { return true; } };
    assert.throws(() => Settings.builder(5).range(1, 20).random(throws).build().generate(), /boom/);
    const settings = Settings.builder(5).range(1, 20).random({ getNumber: () => 1.5, getBool: () => true }).build();
    assert.throws(() => settings.generate(), /must return a whole number from min to max/);
    //the error is not kept for the next time
    assert.throws(() => settings.generate(), /must return a whole number from min to max/);
    assert.throws(() => Settings.builder(1).random({}), /random must be/);
});

test('settings that can not work', () => {
    const settings = Settings.builder(5).range(20, 1).oddEven(1, 1).build();
    assert.equal(settings.errors().length, 2);
    assert.match(settings.errors()[0], /NumberRange min: 20 is greater than max: 1/);
    try {
        settings.generate();
        assert.fail('generate did not throw');
    } catch (e) {
        assert.ok(e instanceof RandomError);
        assert.equal(e.name, 'RandomError');
        assert.equal(e.errorType(), RandomErrorType.BadRequest);
        assert.equal(RandomErrorType[e.errorType()], 'BadRequest');
        assert.equal(e.messages().length, 2);
        assert.match(e.message, /^BadRequest: NumberRange min: 20 is greater than max: 1/);
        assert.equal(String(e), e.message);
        assert.match(e.report(), /BadRequest/);
    }
    assert.deepEqual(Settings.builder(5).range(1, 20).build().errors(), []);
});

test('no numbers found and invalid characters', () => {
    //3 numbers can not come from a pool of 1 number when there are no duplicates
    throwsRandomError(() => Settings.builder(3).range(1, 3).noDuplicates().rule(NumberPool.new().exactly(3, [2])).build().generate(), RandomErrorType.BadRequest);
    //every number from 1 to 2 is excluded
    throwsRandomError(() => Settings.builder(2).range(1, 2).noDuplicates().exclude(NumberRange.all(1, 2)).build().generate(), RandomErrorType.Failed);
    //0xD800 is half of a character
    throwsRandomError(() => Settings.builder(1).range(0xD800, 0xD800).build().generate().string(), RandomErrorType.InvalidCharacter);
});

test('values that are not valid are thrown as an Error', () => {
    assert.throws(() => Settings.builder(-1), /count: -1 must be a whole number/);
    assert.throws(() => Settings.builder(1.5), /count: 1.5 must be a whole number/);
    assert.throws(() => Settings.builder(5).range(1, 4294967296), /max: 4294967296 must be a whole number/);
    assert.throws(() => Settings.builder(5).rule(42), /rule must be one of the rules/);
    assert.throws(() => Settings.builder(5).rule(SeededRandom.new(1)), /rule must be one of the rules/);
    assert.throws(() => Settings.builder(5).random(NoDuplicate.new()), /random must be/);
    assert.throws(() => NumberPool.new().exactly(1, 5), /pool must be/);
    assert.throws(() => NumberPool.new().exactly(1, [1, 'a']), /is not a number/);
    assert.throws(() => NumberRange.new().at('01', 1, 2), /indexes must be/);
    assert.throws(() => SeededRandom.new(-1), /seed must be/);
    assert.throws(() => SeededRandom.new('1'), /seed must be/);
});

test('objects are not changed or used up by the methods that are called on them', () => {
    const builder = Settings.builder(5).range(1, 10);
    const withNoDuplicates = builder.noDuplicates();
    const allEven = builder.oddEven(0, 5);
    generateMany(withNoDuplicates.build(), randomNumbers => assert.equal(new Set(randomNumbers.numbers()).size, 5));
    generateMany(allEven.build(), randomNumbers => assert.ok(randomNumbers.numbers().every(x => x % 2 === 0)));

    //one rule in more than one builder
    const rule = NumberPool.new().exactly(5, [2, 4, 6]);
    const moreRules = rule.noneOf([8]);
    assert.notEqual(String(rule), String(moreRules));
    for (const settings of [Settings.builder(5).rule(rule).build(), Settings.builder(5).rule(rule).rule(NumberRange.all(1, 10)).build()]) {
        assertInRange(settings.generate().numbers().map(x => x / 2), 1, 3);
    }

    //one seeded generator in more than one settings keeps going, the same as a clone of it in Rust
    const seededRandom = SeededRandom.new(42);
    const first = Settings.builder(10).range(1, 20).random(seededRandom).build().generate().numbers();
    const second = Settings.builder(10).range(1, 20).random(seededRandom).build().generate().numbers();
    assert.deepEqual(first, [5, 13, 20, 9, 4, 3, 17, 11, 15, 11]);
    assert.notDeepEqual(second, first);
});

test('toString', () => {
    assert.equal(String(range(1, 10)), '1..=10');
    assert.equal(String(OperatorType.Lt(3)), 'Lt:3');
    assert.ok(String(NumberRange.all(1, 20)).length > 0);
    assert.match(String(Settings.builder(3).range(1, 9).build().generate()), /^RandomNumbers/);
    assert.equal(Settings.builder(3).build().count(), 3);
});

test('the package can be used with require', () => {
    const require = createRequire(import.meta.url);
    const advRandom = require('adv-random');
    assertInRange(advRandom.Settings.builder(5).range(1, 20).build().generate().numbers(), 1, 20);
});

test('the build for browsers without a bundler', async () => {
    const web = await import('adv-random/web');
    //a browser gets the wasm file with fetch.  Node can not, so it is read from the file.
    const require = createRequire(import.meta.url);
    const wasm = readFileSync(require.resolve('adv-random').replace(/nodejs.adv_random\.js$/, 'web/adv_random_bg.wasm'));
    await web.default({ module_or_path: wasm });
    const numbers = web.Settings.builder(10).range(1, 20).random(web.SeededRandom.new(42)).build().generate().numbers();
    assert.deepEqual(numbers, [5, 13, 20, 9, 4, 3, 17, 11, 15, 11]);
});
