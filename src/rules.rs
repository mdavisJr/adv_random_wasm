use crate::convert::{handle_from_js, is_class_of, number_sets_from_js, numbers_from_js, pool_from_js, to_usize};
use adv_random::prelude as adv;
use wasm_bindgen::convert::TryFromJsValue;
use wasm_bindgen::prelude::*;

//A rule that was taken out of its JavaScript object.  Every rule can be used with rule and with exclude.
#[doc(hidden)]
#[wasm_bindgen(js_name = "__RuleHandle", skip_typescript)]
pub struct RuleHandle {
    rule: Box<dyn adv::RuleTrait>,
    exclude_rule: Box<dyn adv::ExcludeRuleTrait>,
}

impl RuleHandle {
    fn new(rule: impl adv::RuleTrait + adv::ExcludeRuleTrait + Clone + 'static) -> RuleHandle {
        return RuleHandle { rule: Box::new(rule.clone()), exclude_rule: Box::new(rule) };
    }

    pub fn into_rule(self) -> Box<dyn adv::RuleTrait> {
        return self.rule;
    }

    pub fn into_exclude_rule(self) -> Box<dyn adv::ExcludeRuleTrait> {
        return self.exclude_rule;
    }
}

pub fn rule_from_js(value: &JsValue) -> Result<RuleHandle, JsError> {
    //NoDuplicate has nothing to set, so the class can be passed in the same way as in Rust
    if is_class_of(value, JsValue::from(NoDuplicate::new())) {
        return Ok(RuleHandle::new(adv::NoDuplicate));
    }
    return handle_from_js(value)
        .and_then(|x| RuleHandle::try_from_js_value(x).ok())
        .ok_or_else(|| JsError::new("rule must be one of the rules, like NumberRange.all(1, 20)"));
}

//The methods that every rule has
macro_rules! rule_class {
    ($name:ident) => {
        #[wasm_bindgen]
        impl $name {
            #[doc(hidden)]
            #[wasm_bindgen(js_name = "__handle", skip_typescript)]
            pub fn handle(&self) -> RuleHandle {
                return RuleHandle::new(self.0.clone());
            }

            #[wasm_bindgen(js_name = "toString")]
            pub fn to_string_js(&self) -> String {
                return self.0.to_string();
            }
        }
    };
}

/// The numbers are from a min to a max.  min and max are inclusive.
#[wasm_bindgen]
pub struct NumberRange(adv::NumberRange);
rule_class!(NumberRange);

#[wasm_bindgen]
impl NumberRange {
    /// No number has a range yet.  Give the numbers at some indexes a range with at.
    #[wasm_bindgen(constructor)]
    pub fn constructor() -> NumberRange {
        return NumberRange::new();
    }

    /// No number has a range yet.  Give the numbers at some indexes a range with at.
    pub fn new() -> NumberRange {
        return NumberRange(adv::NumberRange::new());
    }

    /// Every number is from min to max
    pub fn all(min: f64, max: f64) -> Result<NumberRange, JsError> {
        return Ok(NumberRange(adv::NumberRange::all(to_usize(min, "min")?, to_usize(max, "max")?)));
    }

    /// The numbers at these indexes are from min to max.  The first number is index 0.
    pub fn at(&self, #[wasm_bindgen(unchecked_param_type = "Indexes")] indexes: &JsValue, min: f64, max: f64) -> Result<NumberRange, JsError> {
        return Ok(NumberRange(self.0.clone().at(numbers_from_js(indexes, "indexes")?, to_usize(min, "min")?, to_usize(max, "max")?)));
    }
}

/// How many of something there has to be.  It is used by NumberSpace.run and NumberPool.item.
#[wasm_bindgen]
#[derive(Clone, Copy)]
pub struct OperatorType(adv::OperatorType);

#[wasm_bindgen]
impl OperatorType {
    /// Less than
    #[wasm_bindgen(js_name = "Lt")]
    pub fn lt(value: f64) -> Result<OperatorType, JsError> {
        return Ok(OperatorType(adv::OperatorType::Lt(to_usize(value, "value")?)));
    }

    /// Less than or equal to
    #[wasm_bindgen(js_name = "Lte")]
    pub fn lte(value: f64) -> Result<OperatorType, JsError> {
        return Ok(OperatorType(adv::OperatorType::Lte(to_usize(value, "value")?)));
    }

    /// Equal to
    #[wasm_bindgen(js_name = "Eq")]
    pub fn eq(value: f64) -> Result<OperatorType, JsError> {
        return Ok(OperatorType(adv::OperatorType::Eq(to_usize(value, "value")?)));
    }

    /// Greater than or equal to
    #[wasm_bindgen(js_name = "Gte")]
    pub fn gte(value: f64) -> Result<OperatorType, JsError> {
        return Ok(OperatorType(adv::OperatorType::Gte(to_usize(value, "value")?)));
    }

    /// Greater than
    #[wasm_bindgen(js_name = "Gt")]
    pub fn gt(value: f64) -> Result<OperatorType, JsError> {
        return Ok(OperatorType(adv::OperatorType::Gt(to_usize(value, "value")?)));
    }

    /// From lower to upper.  lower and upper are inclusive.
    #[wasm_bindgen(js_name = "Between")]
    pub fn between(lower: f64, upper: f64) -> Result<OperatorType, JsError> {
        return Ok(OperatorType(adv::OperatorType::Between(to_usize(lower, "lower")?, to_usize(upper, "upper")?)));
    }

    #[wasm_bindgen(js_name = "toString")]
    pub fn to_string_js(&self) -> String {
        return self.0.to_string();
    }
}

/// How many of the numbers come from a pool of numbers.
/// A pool is a list of numbers like [1, 4], a range like range(1, 10), or the characters of a string like "ABC".
#[wasm_bindgen]
pub struct NumberPool(adv::NumberPool);
rule_class!(NumberPool);

#[wasm_bindgen]
impl NumberPool {
    /// No numbers have to come from a pool yet.  Say what is needed with exactly, atLeast, atMost, between and noneOf.
    #[wasm_bindgen(constructor)]
    pub fn constructor() -> NumberPool {
        return NumberPool::new();
    }

    /// No numbers have to come from a pool yet.  Say what is needed with exactly, atLeast, atMost, between and noneOf.
    pub fn new() -> NumberPool {
        return NumberPool(adv::NumberPool::new());
    }

    /// Letters and 1 to 3 digits.  count is how many characters there are.
    pub fn alphanumeric(count: f64) -> Result<NumberPool, JsError> {
        return Ok(NumberPool(adv::NumberPool::alphanumeric(to_usize(count, "count")?)));
    }

    /// Exactly this many of the characters are digits
    pub fn digits(&self, count: f64) -> Result<NumberPool, JsError> {
        return Ok(NumberPool(self.0.clone().digits(to_usize(count, "count")?)));
    }

    /// Adds 1 or 2 special characters
    #[wasm_bindgen(js_name = "withSpecialCharacters")]
    pub fn with_special_characters(&self) -> NumberPool {
        return NumberPool(self.0.clone().with_special_characters());
    }

    /// Exactly this many of the characters are special characters
    #[wasm_bindgen(js_name = "specialCharacters")]
    pub fn special_characters(&self, count: f64) -> Result<NumberPool, JsError> {
        return Ok(NumberPool(self.0.clone().special_characters(to_usize(count, "count")?)));
    }

    /// This many of the numbers come from the pool.  A number that is picked twice counts twice.
    pub fn exactly(&self, count: f64, #[wasm_bindgen(unchecked_param_type = "Pool")] pool: &JsValue) -> Result<NumberPool, JsError> {
        return Ok(NumberPool(self.0.clone().exactly(to_usize(count, "count")?, pool_from_js(pool)?)));
    }

    /// This many or more of the numbers come from the pool
    #[wasm_bindgen(js_name = "atLeast")]
    pub fn at_least(&self, count: f64, #[wasm_bindgen(unchecked_param_type = "Pool")] pool: &JsValue) -> Result<NumberPool, JsError> {
        return Ok(NumberPool(self.0.clone().at_least(to_usize(count, "count")?, pool_from_js(pool)?)));
    }

    /// This many or fewer of the numbers come from the pool
    #[wasm_bindgen(js_name = "atMost")]
    pub fn at_most(&self, count: f64, #[wasm_bindgen(unchecked_param_type = "Pool")] pool: &JsValue) -> Result<NumberPool, JsError> {
        return Ok(NumberPool(self.0.clone().at_most(to_usize(count, "count")?, pool_from_js(pool)?)));
    }

    /// From min to max of the numbers come from the pool
    pub fn between(&self, min: f64, max: f64, #[wasm_bindgen(unchecked_param_type = "Pool")] pool: &JsValue) -> Result<NumberPool, JsError> {
        return Ok(NumberPool(self.0.clone().between(to_usize(min, "min")?, to_usize(max, "max")?, pool_from_js(pool)?)));
    }

    /// None of the numbers come from the pool
    #[wasm_bindgen(js_name = "noneOf")]
    pub fn none_of(&self, #[wasm_bindgen(unchecked_param_type = "Pool")] pool: &JsValue) -> Result<NumberPool, JsError> {
        return Ok(NumberPool(self.0.clone().none_of(pool_from_js(pool)?)));
    }

    /// Adds a pool with a key, and any type for how many numbers come from it
    pub fn item(&self, key: &str, #[wasm_bindgen(unchecked_param_type = "Pool")] pool: &JsValue, needs: &OperatorType) -> Result<NumberPool, JsError> {
        return Ok(NumberPool(self.0.clone().item(key, pool_from_js(pool)?, needs.0)));
    }
}

/// The numbers at some indexes come from a pool of numbers.
#[wasm_bindgen]
pub struct NumberPoolByIndex(adv::NumberPoolByIndex);
rule_class!(NumberPoolByIndex);

#[wasm_bindgen]
impl NumberPoolByIndex {
    /// No numbers have to come from a pool yet.  Say which ones with at.
    #[wasm_bindgen(constructor)]
    pub fn constructor() -> NumberPoolByIndex {
        return NumberPoolByIndex::new();
    }

    /// No numbers have to come from a pool yet.  Say which ones with at.
    pub fn new() -> NumberPoolByIndex {
        return NumberPoolByIndex(adv::NumberPoolByIndex::new());
    }

    /// The numbers at these indexes come from this pool.  The first number is index 0.
    pub fn at(
        &self,
        #[wasm_bindgen(unchecked_param_type = "Indexes")] indexes: &JsValue,
        #[wasm_bindgen(unchecked_param_type = "Pool")] pool: &JsValue
    ) -> Result<NumberPoolByIndex, JsError> {
        return Ok(NumberPoolByIndex(self.0.clone().at(numbers_from_js(indexes, "indexes")?, pool_from_js(pool)?)));
    }
}

/// No number is there more than once.  With exclude, at least one number is there more than once.
#[wasm_bindgen]
pub struct NoDuplicate(adv::NoDuplicate);
rule_class!(NoDuplicate);

#[wasm_bindgen]
impl NoDuplicate {
    #[wasm_bindgen(constructor)]
    pub fn constructor() -> NoDuplicate {
        return NoDuplicate::new();
    }

    pub fn new() -> NoDuplicate {
        return NoDuplicate(adv::NoDuplicate);
    }
}

/// How many of the numbers are odd and how many are even.
#[wasm_bindgen]
pub struct OddEven(adv::OddEven);
rule_class!(OddEven);

#[wasm_bindgen]
impl OddEven {
    /// No odd and no even numbers yet.  Say how many with odd and even.
    #[wasm_bindgen(constructor)]
    pub fn constructor() -> OddEven {
        return OddEven::new();
    }

    /// No odd and no even numbers yet.  Say how many with odd and even.
    pub fn new() -> OddEven {
        return OddEven(adv::OddEven::new());
    }

    /// How many of the numbers are odd
    pub fn odd(&self, count: f64) -> Result<OddEven, JsError> {
        return Ok(OddEven(self.0.odd(to_usize(count, "count")?)));
    }

    /// How many of the numbers are even
    pub fn even(&self, count: f64) -> Result<OddEven, JsError> {
        return Ok(OddEven(self.0.even(to_usize(count, "count")?)));
    }
}

/// The numbers at some indexes are odd or even.
#[wasm_bindgen]
pub struct OddEvenByIndex(adv::OddEvenByIndex);
rule_class!(OddEvenByIndex);

#[wasm_bindgen]
impl OddEvenByIndex {
    /// No numbers have to be odd or even yet.  Say which ones with odd and even.
    #[wasm_bindgen(constructor)]
    pub fn constructor() -> OddEvenByIndex {
        return OddEvenByIndex::new();
    }

    /// No numbers have to be odd or even yet.  Say which ones with odd and even.
    pub fn new() -> OddEvenByIndex {
        return OddEvenByIndex(adv::OddEvenByIndex::new());
    }

    /// The numbers at these indexes are odd.  The first number is index 0.
    pub fn odd(&self, #[wasm_bindgen(unchecked_param_type = "Indexes")] indexes: &JsValue) -> Result<OddEvenByIndex, JsError> {
        return Ok(OddEvenByIndex(self.0.clone().odd(numbers_from_js(indexes, "indexes")?)));
    }

    /// The numbers at these indexes are even.  The first number is index 0.
    pub fn even(&self, #[wasm_bindgen(unchecked_param_type = "Indexes")] indexes: &JsValue) -> Result<OddEvenByIndex, JsError> {
        return Ok(OddEvenByIndex(self.0.clone().even(numbers_from_js(indexes, "indexes")?)));
    }
}

/// How many numbers are in a row, like 4, 5, 6, and how many are not next to another number.
#[wasm_bindgen]
pub struct Sequential(adv::Sequential);
rule_class!(Sequential);

#[wasm_bindgen]
impl Sequential {
    /// No numbers yet.  Say what is needed with singles and inARow.
    #[wasm_bindgen(constructor)]
    pub fn constructor() -> Sequential {
        return Sequential::new();
    }

    /// No numbers yet.  Say what is needed with singles and inARow.
    pub fn new() -> Sequential {
        return Sequential(adv::Sequential::new());
    }

    /// How many numbers are not next to another number
    pub fn singles(&self, count: f64) -> Result<Sequential, JsError> {
        return Ok(Sequential(self.0.clone().singles(to_usize(count, "count")?)));
    }

    /// Adds one sequence of this many numbers in a row.  4, 5, 6 is 3 in a row.
    #[wasm_bindgen(js_name = "inARow")]
    pub fn in_a_row(&self, count: f64) -> Result<Sequential, JsError> {
        return Ok(Sequential(self.0.clone().in_a_row(to_usize(count, "count")?)));
    }

    /// Adds a sequence for every count.  The same as calling inARow for each one.
    #[wasm_bindgen(js_name = "inARowEach")]
    pub fn in_a_row_each(&self, #[wasm_bindgen(unchecked_param_type = "ArrayLike<number> | Iterable<number>")] counts: &JsValue) -> Result<Sequential, JsError> {
        return Ok(Sequential(self.0.clone().in_a_row_each(&numbers_from_js(counts, "counts")?)));
    }
}

/// Says if the runs of every type of a NumberSpace are created in the same request, or one type per request.
#[wasm_bindgen]
#[derive(Clone, Copy)]
pub enum ProcessNumberSpaceItems {
    AllPerRequest,
    OnePerRequest,
}

/// The space between the numbers after they are sorted.  The space between 4 and 7 is 3.
#[wasm_bindgen]
pub struct NumberSpace(adv::NumberSpace);
rule_class!(NumberSpace);

#[wasm_bindgen]
impl NumberSpace {
    /// No spaces are needed yet.  Say what is needed with run.
    #[wasm_bindgen(constructor)]
    pub fn constructor() -> NumberSpace {
        return NumberSpace::new();
    }

    /// No spaces are needed yet.  Say what is needed with run.
    pub fn new() -> NumberSpace {
        return NumberSpace(adv::NumberSpace::new());
    }

    /// Adds one run of this many spaces in a row that match the type.  10 numbers have 9 spaces between them, so
    /// run(OperatorType.Lt(3), 9) is 10 numbers that are all less than 3 apart.
    pub fn run(&self, number_space_type: &OperatorType, count: f64) -> Result<NumberSpace, JsError> {
        return Ok(NumberSpace(self.0.clone().run(number_space_type.0, to_usize(count, "count")?)));
    }

    /// Adds a run for every count.  The same as calling run for each one.
    pub fn runs(
        &self,
        number_space_type: &OperatorType,
        #[wasm_bindgen(unchecked_param_type = "ArrayLike<number> | Iterable<number>")] counts: &JsValue
    ) -> Result<NumberSpace, JsError> {
        return Ok(NumberSpace(self.0.clone().runs(number_space_type.0, &numbers_from_js(counts, "counts")?)));
    }

    /// Says if the runs of every type are created in the same request, or one type per request
    pub fn process(&self, process_number_space_items: ProcessNumberSpaceItems) -> NumberSpace {
        let process_number_space_items = match process_number_space_items {
            ProcessNumberSpaceItems::AllPerRequest => adv::ProcessNumberSpaceItems::AllPerRequest,
            ProcessNumberSpaceItems::OnePerRequest => adv::ProcessNumberSpaceItems::OnePerRequest
        };
        return NumberSpace(self.0.clone().process(process_number_space_items));
    }
}

/// Sets of numbers, or strings, that the random numbers can not be.
#[wasm_bindgen]
pub struct ExcludeNumberSets(adv::ExcludeNumberSets);
rule_class!(ExcludeNumberSets);

#[wasm_bindgen]
impl ExcludeNumberSets {
    /// Nothing is excluded yet.  Say what is with numbers and string.
    #[wasm_bindgen(constructor)]
    pub fn constructor() -> ExcludeNumberSets {
        return ExcludeNumberSets::new();
    }

    /// Nothing is excluded yet.  Say what is with numbers and string.
    pub fn new() -> ExcludeNumberSets {
        return ExcludeNumberSets(adv::ExcludeNumberSets::new());
    }

    /// The random numbers can not be these numbers in this order
    pub fn numbers(&self, #[wasm_bindgen(unchecked_param_type = "ArrayLike<number> | Iterable<number>")] numbers: &JsValue) -> Result<ExcludeNumberSets, JsError> {
        return Ok(ExcludeNumberSets(self.0.clone().numbers(&numbers_from_js(numbers, "numbers")?)));
    }

    /// Excludes every list of numbers.  The same as calling numbers for each one.
    #[wasm_bindgen(js_name = "numbersEach")]
    pub fn numbers_each(&self, #[wasm_bindgen(unchecked_param_type = "ArrayLike<number>[]")] number_sets: &JsValue) -> Result<ExcludeNumberSets, JsError> {
        return Ok(ExcludeNumberSets(self.0.clone().numbers_each(number_sets_from_js(number_sets, "numberSets")?)));
    }

    /// The random string can not be this string
    pub fn string(&self, string: &str) -> ExcludeNumberSets {
        return ExcludeNumberSets(self.0.clone().string(string));
    }

    /// Excludes every string.  The same as calling string for each one.
    #[wasm_bindgen(js_name = "stringEach")]
    pub fn string_each(&self, strings: Vec<String>) -> ExcludeNumberSets {
        return ExcludeNumberSets(self.0.clone().string_each(strings));
    }
}
