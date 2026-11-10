//The checks that are run in the browser.  They are the same for every way of loading the package.
//Returns the name of every check with 'ok', or with what went wrong.

export function runChecks(advRandom) {
    const {
        ExcludeNumberSets, NoDuplicate, NumberPool, NumberPoolByIndex, NumberRange, NumberSpace, OperatorType,
        RandomError, RandomErrorType, SeededRandom, Settings, range
    } = advRandom;
    const results = {};

    function check(name, fn) {
        try {
            results[name] = fn() === true ? 'ok' : 'failed';
        } catch (e) {
            results[name] = `threw ${e?.message ?? e}`;
        }
    }

    check('range', () => {
        const numbers = Settings.builder(10).range(1, 20).build().generate().numbers();
        return Array.isArray(numbers) && numbers.length === 10 && numbers.every(x => x >= 1 && x <= 20);
    });

    //the default generator gets its seed from crypto.getRandomValues of the browser
    check('default random number generator', () => {
        const runs = Array.from({ length: 20 }, () => Settings.builder(10).range(1, 1000000).build().generate().numbers().join());
        return new Set(runs).size === 20;
    });

    check('no duplicates', () => {
        return new Set(Settings.builder(10).range(1, 20).noDuplicates().build().generate().numbers()).size === 10;
    });

    //NoDuplicate is found by its class, which has a different name after a bundler makes the code smaller
    check('exclude NoDuplicate', () => {
        return new Set(Settings.builder(10).range(1, 20).exclude(NoDuplicate).build().generate().numbers()).size < 10;
    });

    check('number pool', () => {
        const numbers = Settings.builder(10)
            .range(1, 100)
            .noDuplicates()
            .rule(NumberPool.new().noneOf([23]).exactly(2, [1, 4]).atLeast(3, [17, 18, 19, 20, 21]))
            .exclude(NumberRange.all(22, 29))
            .build()
            .generate()
            .numbers();
        return numbers.includes(1) && numbers.includes(4) && numbers.filter(x => x >= 17 && x <= 21).length >= 3;
    });

    check('seeded random number generator', () => {
        const numbers = Settings.builder(10).range(1, 20).random(SeededRandom.new(42)).build().generate().numbers();
        return numbers.join() === '5,13,20,9,4,3,17,11,15,11';
    });

    check('alphanumeric string', () => {
        return /^[a-zA-Z0-9]{20}$/.test(Settings.builder(20).rule(NumberPool.alphanumeric(20)).build().generate().shuffledString());
    });

    check('license number', () => {
        const string = Settings.builder(9)
            .rule(ExcludeNumberSets.new().string('AB1234567'))
            .rule(NumberPoolByIndex.new().at([0, 1], 'ABCDEFGHIJKLMNOPQRSTUVWXYZ').at(range(2, 8), '1234567890'))
            .build()
            .generate()
            .string();
        return /^[A-Z]{2}[0-9]{7}$/.test(string);
    });

    check('number space', () => {
        const numbers = Settings.builder(10).range(1, 100).rule(NumberSpace.new().run(OperatorType.Eq(3), 9)).build().generate().numbers();
        numbers.sort((a, b) => a - b);
        return numbers.every((x, i) => i === 0 || x - numbers[i - 1] === 3);
    });

    check('random number generator written in JavaScript', () => {
        let next = 0;
        const countingRandom = {
            getNumber: (min, max) => min + (next++) % (max - min + 1),
            getBool: () => true
        };
        return Settings.builder(5).range(1, 20).random(countingRandom).build().generate().numbers().join() === '3,6,9,12,15';
    });

    check('RandomError', () => {
        try {
            Settings.builder(5).range(20, 1).build().generate();
            return false;
        } catch (e) {
            return e instanceof RandomError && e.errorType() === RandomErrorType.BadRequest && e.message.startsWith('BadRequest');
        }
    });

    check('value that is not valid', () => {
        try {
            Settings.builder(-1);
            return false;
        } catch (e) {
            return e instanceof Error;
        }
    });

    return results;
}
