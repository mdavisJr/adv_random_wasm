#![allow(clippy::needless_return)]
#![allow(clippy::new_without_default)]
//adv_random for JavaScript.  Every class here is a thin wrapper around the adv_random type with the same name.
//The methods do not change the object they are called on.  They return a new object, so they can be chained
//the same way as in Rust and an object can be used more than once.

mod convert;
mod random;
mod results;
mod rules;
mod settings;

pub use crate::convert::{range, RangeInclusive};
pub use crate::random::{DefaultRandom, SeededRandom};
pub use crate::results::{RandomError, RandomErrorType, RandomNumbers};
pub use crate::rules::*;
pub use crate::settings::{Settings, SettingsBuilder};

use wasm_bindgen::prelude::*;

#[wasm_bindgen(typescript_custom_section)]
const TYPESCRIPT_TYPES: &'static str = r#"
/** A pool of numbers: a list of numbers like [1, 4], a range like range(1, 10), or the characters of a string like "ABC". */
export type Pool = ArrayLike<number> | Iterable<number> | string | RangeInclusive | { min: number, max: number };

/** Indexes of the random numbers: a list like [0, 2, 4] or a range like range(2, 8).  The first number is index 0. */
export type Indexes = ArrayLike<number> | Iterable<number> | RangeInclusive | { min: number, max: number };

/** A rule for SettingsBuilder.rule and SettingsBuilder.exclude.  NoDuplicate can be passed without calling new. */
export type Rule = NumberRange | NumberPool | NumberPoolByIndex | NoDuplicate | typeof NoDuplicate | OddEven | OddEvenByIndex | Sequential | NumberSpace | ExcludeNumberSets;

/** Your own random number generator. */
export interface RandomGenerator {
    /** A whole number from min to max.  min and max are inclusive. */
    getNumber(min: number, max: number): number;
    getBool(): boolean;
}

/** A random number generator for SettingsBuilder.random. */
export type Random = DefaultRandom | SeededRandom | RandomGenerator;
"#;
