# adv-random
Create random numbers, passwords, or strings based on rules.  This is the JavaScript version of the Rust crate [adv_random](https://crates.io/crates/adv_random).  It is the same Rust code, built to WebAssembly, and it is used with almost the same syntax.  It comes with TypeScript types.

Please see examples of using rules below.  The package can handle more combinations of rules than what is shown below.

The version says which adv_random it is built from.  adv-random 3.0.x is built from adv_random 3.0.

# Getting Started
Add adv-random to your project.
```text
npm install adv-random
```
Say how many numbers you want, add the rules, and call `generate`.
```js
import { Settings } from 'adv-random';

//10 random numbers between 1 and 20 with no duplicates
const randomNumbers = Settings.builder(10)
    .range(1, 20)
    .noDuplicates()
    .build()
    .generate();
console.log(randomNumbers.numbers());
```
#### Output: [9, 1, 2, 20, 10, 7, 4, 14, 19, 17]
It works in Node 19 or later with `import` and with `require`, and in browsers with or without a bundler.

# Browsers
With a bundler, import `adv-random` the same way as above.  webpack needs `experiments.asyncWebAssembly` turned on, and Vite needs the `vite-plugin-wasm` plugin.

Without a bundler, import `adv-random/web` and wait for the default export before using anything else.  It loads the WebAssembly file, which has to be served from the same folder as the JavaScript file.
```html
<script type="module">
    import init, { Settings } from './node_modules/adv-random/web/adv_random.js';

    await init();
    const randomNumbers = Settings.builder(10).range(1, 20).build().generate();
    console.log(randomNumbers.numbers());
</script>
```

Every example in this file is run by `npm test`, so they all work with the current version.

# Differences From Rust
| Rust | JavaScript |
|---|---|
| `Settings::builder(10)` | `Settings.builder(10)` |
| `.no_duplicates()`, `.at_least(...)`, `.in_a_row(...)` | `.noDuplicates()`, `.atLeast(...)`, `.inARow(...)` |
| `NumberPool::new()` | `NumberPool.new()` or `new NumberPool()` |
| `NoDuplicate` | `NoDuplicate`, `NoDuplicate.new()` or `new NoDuplicate()` |
| `1..=10` | `range(1, 10)` |
| `OperatorType::Lt(3)` | `OperatorType.Lt(3)` |
| `generate()` returns a `Result` | `generate()` returns the numbers and throws a `RandomError` |
| Numbers are `usize` | Numbers are whole numbers from 0 to 4294967295 |

A method never changes the object it is called on.  It returns a new object, so use what it returns.  A rule, a builder or a random number generator can be used as many times as you want.

A number that is not a whole number from 0 to 4294967295, or a value of the wrong type, throws an `Error` right away.

Custom rules can not be written in JavaScript yet.

# Settings
`Settings.builder(count)` starts the settings for `count` random numbers.

| Method | What it does |
|---|---|
| `.rule(rule)` | Adds a rule that the numbers have to match |
| `.exclude(rule)` | Adds a rule that the numbers can not match |
| `.range(min, max)` | Every number is from min to max.  The same as `.rule(NumberRange.all(min, max))` |
| `.noDuplicates()` | No number is there more than once.  The same as `.rule(NoDuplicate)` |
| `.oddEven(odd, even)` | This many odd numbers and this many even numbers.  The same as `.rule(OddEven.new().odd(odd).even(even))` |
| `.random(generator)` | Uses your random number generator |
| `.build()` | Returns the `Settings` |

`Settings` can be used as many times as you want.  Every call to `generate` creates new random numbers.

# Results
`settings.generate()` returns `RandomNumbers`, which has the numbers.

| Method | What it returns |
|---|---|
| `numbers()` | The numbers as an array, in the order they were created |
| `string()` | The numbers as characters, in the order they were created |
| `shuffledString()` | The numbers as characters, in a random order |
| `attempts()` | How many attempts it took |
| `report()` | The numbers, the attempts and everything that happened in every attempt, for debugging |

`generate`, `string` and `shuffledString` throw a `RandomError` that says what went wrong.  `message` and `toString()` are the short version.  `messages()` is every problem as an array.  `report()` returns the settings, the attempts and everything that happened in every attempt, for debugging.

| `errorType()` | What it means |
|---|---|
| `RandomErrorType.BadRequest` | The settings can not work.  `messages()` says why |
| `RandomErrorType.Failed` | No numbers that match the rules were found in the attempts that are allowed |
| `RandomErrorType.InvalidCharacter` | A number is not a valid character, so the numbers can not be a string |

```js
import { RandomError, RandomErrorType, Settings } from 'adv-random';

try {
    const randomNumbers = Settings.builder(10).range(1, 20).oddEven(5, 4).build().generate();
    console.log(randomNumbers.numbers());
} catch (e) {
    if (e instanceof RandomError && e.errorType() === RandomErrorType.BadRequest) {
        console.log(e.message);
    } else {
        throw e;
    }
}
```
#### Output: BadRequest: Odd count: 5 and Even count: 4 must add up to count: 10
`RandomError` is not a JavaScript `Error`, so check for it with `instanceof RandomError`.

# Invalid Settings
Every rule checks what it is given, and `build` checks the rules against the count and against each other.  Examples are odd and even counts that do not add up to the count, a range with a min greater than its max, an index that is past the last number, and a pool that is empty.  `generate` throws an error with the type `RandomErrorType.BadRequest`, and the same problems are in `settings.errors()` before you generate anything.
```js
import { Settings } from 'adv-random';

const settings = Settings.builder(5).range(20, 1).oddEven(1, 1).build();
for (const error of settings.errors()) {
    console.log(error);
}
```
#### Output:
```text
NumberRange min: 20 is greater than max: 1
Odd count: 1 and Even count: 1 must add up to count: 5
```

# Random Number Generator
Every `Settings` uses `DefaultRandom`, which gets its seed from `crypto.getRandomValues`.  Call `.random(...)` on the builder to use a different one.

`SeededRandom` gives the same numbers every time for the same seed.  The seed is a whole number, or a `bigint` when it is too big for a number.
```js
import { SeededRandom, Settings } from 'adv-random';

const randomNumbers = Settings.builder(10)
    .range(1, 20)
    .random(SeededRandom.new(42))
    .build()
    .generate();
console.log(randomNumbers.numbers());
```
#### Output: [5, 13, 20, 9, 4, 3, 17, 11, 15, 11]
Your own generator is any object that has `getNumber(min, max)` and `getBool()`.  This one counts up instead of being random, to keep the example short.
```js
import { Settings } from 'adv-random';

let next = 0;
const countingRandom = {
    //a whole number from min to max.  min and max are inclusive.
    getNumber(min, max) {
        return min + (next++) % (max - min + 1);
    },

    getBool() {
        return this.getNumber(0, 1) === 1;
    }
};

const randomNumbers = Settings.builder(5)
    .range(1, 20)
    .random(countingRandom)
    .build()
    .generate();
console.log(randomNumbers.numbers());
```
#### Output: [3, 6, 9, 12, 15]
An error that your generator throws is thrown by `build`, `generate` or `shuffledString`.

# Examples
### Create 10 random numbers between 1 and 20 (min and max numbers are inclusive)
```js
import { Settings } from 'adv-random';

const randomNumbers = Settings.builder(10)
    .range(1, 20)
    .build()
    .generate();
console.log(randomNumbers.numbers());
```
#### Output: [5, 7, 14, 2, 7, 7, 20, 1, 1, 5]
---
### Create 10 random numbers between 1 and 20 with no duplicates
```js
import { Settings } from 'adv-random';

const randomNumbers = Settings.builder(10)
    .range(1, 20)
    .noDuplicates()
    .build()
    .generate();
console.log(randomNumbers.numbers());
```
#### Output: [7, 11, 4, 19, 18, 8, 10, 5, 1, 14]
---
### Create 10 random numbers between 1 and 20 with duplicates numbers
```js
import { NoDuplicate, Settings } from 'adv-random';

const randomNumbers = Settings.builder(10)
    .range(1, 20)
    .exclude(NoDuplicate)
    .build()
    .generate();
console.log(randomNumbers.numbers());
```
#### Output: [8, 11, 19, 19, 14, 17, 18, 1, 4, 14]
---
### Create random phone number
```js
import { NumberRange, Settings } from 'adv-random';

const randomNumbers = Settings.builder(3)
    .rule(NumberRange.new().at([0, 1], 100, 999).at([2], 1000, 9999))
    .build()
    .generate();
const numbers = randomNumbers.numbers();
console.log(`${numbers[0]}-${numbers[1]}-${numbers[2]}`);
```
#### Output: 381-984-7431
---
### Create 10 numbers between 1 and 100, excluding numbers 20 - 29 and 51 - 75, and with no duplicates
```js
import { NumberRange, Settings } from 'adv-random';

const randomNumbers = Settings.builder(10)
    .range(1, 100)
    .noDuplicates()
    .exclude(NumberRange.all(20, 29))
    .exclude(NumberRange.all(51, 75))
    .build()
    .generate();
console.log(randomNumbers.numbers());
```
#### Output: [98, 76, 99, 45, 6, 97, 36, 39, 16, 11]
---
### Create 10 numbers between 1 and 100 with no duplicates that doesn't contain the number 23, contains the numbers 1 and 4, and at least 3 numbers from 17, 18, 19, 20, 21
```js
import { NumberPool, Settings } from 'adv-random';

const randomNumbers = Settings.builder(10)
    .range(1, 100)
    .noDuplicates()
    .rule(NumberPool.new()
        .noneOf([23])
        .exactly(2, [1, 4])
        .atLeast(3, [17, 18, 19, 20, 21]))
    .build()
    .generate();
console.log(randomNumbers.numbers());
```
#### Output: [1, 4, 20, 21, 47, 90, 95, 62, 29, 19]
A pool is a list of numbers like `[1, 4]`, a range like `range(1, 10)`, or the characters of a string like `"ABC"`.  A number that is picked twice counts twice, so `exactly(2, [1, 4])` means both 1 and 4 because there are no duplicates.  `NumberPool` also has `atMost` and `between`.

---
### Create 5 numbers between 1 and 10 with no duplicates that are Odd, Even, Odd, Even, and Odd
```js
import { OddEvenByIndex, Settings } from 'adv-random';

const randomNumbers = Settings.builder(5)
    .range(1, 10)
    .noDuplicates()
    .rule(OddEvenByIndex.new().odd([0, 2, 4]).even([1, 3]))
    .build()
    .generate();
console.log(randomNumbers.numbers());
```
#### Output: [9, 8, 7, 6, 3]
---
### Create 5 numbers between 1 and 10 with no duplicates where first 3 numbers are Even and the last 2 numbers are Odd
```js
import { OddEvenByIndex, Settings } from 'adv-random';

const randomNumbers = Settings.builder(5)
    .range(1, 10)
    .noDuplicates()
    .rule(OddEvenByIndex.new().odd([3, 4]).even([0, 1, 2]))
    .build()
    .generate();
console.log(randomNumbers.numbers());
```
#### Output: [2, 8, 4, 9, 3]
---
### Create 5 numbers between 1 and 10 with no duplicates and the 5th number is odd
```js
import { OddEvenByIndex, Settings } from 'adv-random';

const randomNumbers = Settings.builder(5)
    .range(1, 10)
    .noDuplicates()
    .rule(OddEvenByIndex.new().odd([4]))
    .build()
    .generate();
console.log(randomNumbers.numbers());
```
#### Output: [4, 3, 1, 10, 5]
---
### Create 10 numbers between 1 and 20 with no duplicates that has 5 odd and 5 even numbers
```js
import { Settings } from 'adv-random';

const randomNumbers = Settings.builder(10)
    .range(1, 20)
    .noDuplicates()
    .oddEven(5, 5)
    .build()
    .generate();
console.log(randomNumbers.numbers());
```
#### Output: [3, 6, 17, 19, 18, 5, 9, 16, 8, 12]
---
### Create 10 numbers between 1 and 20 that has all even numbers
```js
import { Settings } from 'adv-random';

const randomNumbers = Settings.builder(10)
    .range(1, 20)
    .oddEven(0, 10)
    .build()
    .generate();
console.log(randomNumbers.numbers());
```
#### Output: [10, 2, 20, 20, 14, 20, 10, 12, 12, 6]
---
### Create 5 numbers between 1 and 10 with no duplicates that are all nonsequential
`singles` is how many numbers are not next to another number.  `inARow` adds one sequence, so `inARow(3)` is 3 numbers in a row like 4, 5, 6.
```js
import { Sequential, Settings } from 'adv-random';

const randomNumbers = Settings.builder(5)
    .range(1, 10)
    .noDuplicates()
    .rule(Sequential.new().singles(5))
    .build()
    .generate();
console.log(randomNumbers.numbers());
```
#### Output: [10, 8, 5, 1, 3]
---
### Create 5 numbers between 1 and 10 with no duplicates that has 3 nonsequential and has 1 sequential set of 2 numbers
```js
import { Sequential, Settings } from 'adv-random';

const randomNumbers = Settings.builder(5)
    .range(1, 10)
    .noDuplicates()
    .rule(Sequential.new().singles(3).inARow(2))
    .build()
    .generate();
console.log(randomNumbers.numbers());
```
#### Output: [1, 2, 9, 6, 4]
---
### Create 5 numbers between 1 and 10 with no duplicates that has 1 nonsequential number and has 2 sequential sets of 2 numbers a piece
```js
import { Sequential, Settings } from 'adv-random';

const randomNumbers = Settings.builder(5)
    .range(1, 10)
    .noDuplicates()
    .rule(Sequential.new().singles(1).inARow(2).inARow(2))
    .build()
    .generate();
console.log(randomNumbers.numbers());
```
#### Output: [9, 8, 6, 1, 2]
---
### Create a random 20 character string with no special characters
`NumberPool.alphanumeric(20)` is letters and 1 to 3 digits.  `withSpecialCharacters` adds 1 or 2 special characters.  `digits` and `specialCharacters` say exactly how many there are, and the rest are letters.
```js
import { NumberPool, Settings } from 'adv-random';

const randomNumbers = Settings.builder(20)
    .rule(NumberPool.alphanumeric(20))
    .build()
    .generate();
console.log(randomNumbers.shuffledString());
```
#### Output: btkokcf4kzkfPnOr9L9e
---
### Create a random 20 character string with special characters
```js
import { NumberPool, Settings } from 'adv-random';

const randomNumbers = Settings.builder(20)
    .rule(NumberPool.alphanumeric(20).withSpecialCharacters())
    .build()
    .generate();
console.log(randomNumbers.shuffledString());
```
#### Output: GbNK8UT!DJqhZ%uvzf08
---
### Create a random 25 character string with 10 letters, 10 numbers, and 5 special characters and no duplicates
```js
import { NumberPool, Settings } from 'adv-random';

const randomNumbers = Settings.builder(25)
    .noDuplicates()
    .rule(NumberPool.alphanumeric(25).digits(10).specialCharacters(5))
    .build()
    .generate();
console.log(randomNumbers.shuffledString());
```
#### Output: FqDl7heG#dMt=25649%380)*1
---
### Create random 15 character string from upper case characters, from numbers 3 6 5, and from special characters # *
```js
import { NumberPool, Settings } from 'adv-random';

const randomNumbers = Settings.builder(15)
    .rule(NumberPool.new()
        .exactly(11, 'ABCDEFGHIJKLMNOPQRSTUVWXYZ')
        .exactly(2, '365')
        .exactly(2, '#*'))
    .build()
    .generate();
console.log(randomNumbers.shuffledString());
```
#### Output: 3GLA3M#R*MLBJRC
---
### Create random license number that starts with 2 apha-characters followed by 7 numeric characters. Also excludes "AB1234567" and "CB1234567" because they are already assigned to someone.
```js
import { ExcludeNumberSets, NumberPoolByIndex, Settings, range } from 'adv-random';

const randomNumbers = Settings.builder(9)
    .rule(ExcludeNumberSets.new().string('AB1234567').string('CB1234567'))
    .rule(NumberPoolByIndex.new()
        .at([0, 1], 'ABCDEFGHIJKLMNOPQRSTUVWXYZ')
        .at(range(2, 8), '1234567890'))
    .build()
    .generate();
console.log(randomNumbers.string());
```
#### Output: LG1299382
---
### Create 10 random numbers between 1 and 100 where the space between numbers is less than 3
`run(OperatorType.Lt(3), 9)` is 9 spaces in a row that are less than 3.  10 numbers have 9 spaces between them.
```js
import { NumberSpace, OperatorType, Settings } from 'adv-random';

const randomNumbers = Settings.builder(10)
    .range(1, 100)
    .rule(NumberSpace.new().run(OperatorType.Lt(3), 9))
    .build()
    .generate();
const sortedNumbers = randomNumbers.numbers().sort((a, b) => a - b);
console.log(sortedNumbers);
```
#### Output: [46, 47, 48, 50, 52, 54, 55, 57, 58, 59]
---
### Create 10 random numbers between 1 and 100 where the space between numbers is less than or equal to 3
```js
import { NumberSpace, OperatorType, Settings } from 'adv-random';

const randomNumbers = Settings.builder(10)
    .range(1, 100)
    .rule(NumberSpace.new().run(OperatorType.Lte(3), 9))
    .build()
    .generate();
const sortedNumbers = randomNumbers.numbers().sort((a, b) => a - b);
console.log(sortedNumbers);
```
#### Output: [19, 20, 23, 24, 27, 30, 32, 33, 36, 38]
---
### Create 10 random numbers between 1 and 100 where the space between numbers is 3
```js
import { NumberSpace, OperatorType, Settings } from 'adv-random';

const randomNumbers = Settings.builder(10)
    .range(1, 100)
    .rule(NumberSpace.new().run(OperatorType.Eq(3), 9))
    .build()
    .generate();
const sortedNumbers = randomNumbers.numbers().sort((a, b) => a - b);
console.log(sortedNumbers);
```
#### Output: [31, 34, 37, 40, 43, 46, 49, 52, 55, 58]
---
### Create 10 random numbers between 1 and 100 where the space between numbers is greater than or equal to 3
```js
import { NumberSpace, OperatorType, Settings } from 'adv-random';

const randomNumbers = Settings.builder(10)
    .range(1, 100)
    .rule(NumberSpace.new().run(OperatorType.Gte(3), 9))
    .build()
    .generate();
const sortedNumbers = randomNumbers.numbers().sort((a, b) => a - b);
console.log(sortedNumbers);
```
#### Output: [5, 13, 49, 53, 59, 75, 84, 87, 95, 98]
---
### Create 10 random numbers between 1 and 100 where the space between numbers is greater than 3
```js
import { NumberSpace, OperatorType, Settings } from 'adv-random';

const randomNumbers = Settings.builder(10)
    .range(1, 100)
    .rule(NumberSpace.new().run(OperatorType.Gt(3), 9))
    .build()
    .generate();
const sortedNumbers = randomNumbers.numbers().sort((a, b) => a - b);
console.log(sortedNumbers);
```
#### Output: [3, 13, 19, 24, 28, 50, 60, 75, 94, 98]
---
### Create 10 random numbers between 1 and 100 where the space between numbers is between 3 and 6
```js
import { NumberSpace, OperatorType, Settings } from 'adv-random';

const randomNumbers = Settings.builder(10)
    .range(1, 100)
    .rule(NumberSpace.new().run(OperatorType.Between(3, 6), 9))
    .build()
    .generate();
const sortedNumbers = randomNumbers.numbers().sort((a, b) => a - b);
console.log(sortedNumbers);
```
#### Output: [20, 26, 32, 37, 42, 48, 52, 56, 60, 63]
---

# Building This Package
The package is built from the Rust code in `src` with [wasm-pack](https://github.com/rustwasm/wasm-pack).  It needs Rust with the `wasm32-unknown-unknown` target, wasm-pack and Node.
```text
npm run build
npm install
npm test
```
`npm run build` creates the npm package in `pkg`, with one build for Node, one for bundlers and one for browsers without a bundler.  `npm install` links `pkg` into `node_modules` so the tests can import it by its name.  It only has to be run one time.  `pkg` is the folder to publish.

# License
MIT OR Apache-2.0
