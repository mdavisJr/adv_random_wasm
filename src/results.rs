use crate::random::JsRandom;
use adv_random::prelude as adv;
use js_sys::Array;
use std::sync::Arc;
use wasm_bindgen::prelude::*;

/// The random numbers from Settings.generate.
#[wasm_bindgen]
pub struct RandomNumbers {
    random_numbers: adv::RandomNumbers,
    js_random: Option<Arc<JsRandom>>,
}

impl RandomNumbers {
    pub fn new(random_numbers: adv::RandomNumbers, js_random: Option<Arc<JsRandom>>) -> RandomNumbers {
        return RandomNumbers { random_numbers, js_random };
    }
}

#[wasm_bindgen]
impl RandomNumbers {
    /// The numbers, in the order they were created
    #[wasm_bindgen(unchecked_return_type = "number[]")]
    pub fn numbers(&self) -> Array {
        return self.random_numbers.numbers().iter().map(|x| JsValue::from(*x as u32)).collect();
    }

    /// The numbers as characters, in the order they were created
    pub fn string(&self) -> Result<String, RandomError> {
        return self.random_numbers.string().map_err(RandomError);
    }

    /// The numbers as characters, in a random order
    #[wasm_bindgen(js_name = "shuffledString")]
    pub fn shuffled_string(&self) -> Result<String, JsValue> {
        let string = self.random_numbers.shuffled_string();
        if let Some(js_random) = &self.js_random {
            js_random.check()?;
        }
        return string.map_err(|x| RandomError(x).into());
    }

    /// How many attempts it took
    pub fn attempts(&self) -> usize {
        return self.random_numbers.attempts();
    }

    /// The numbers, the attempts and everything that happened in every attempt, for debugging
    pub fn report(&self) -> String {
        return self.random_numbers.report();
    }

    #[wasm_bindgen(js_name = "toString")]
    pub fn to_string_js(&self) -> String {
        return format!("{:?}", self.random_numbers);
    }
}

/// What went wrong.  It is what RandomError.errorType returns.
#[wasm_bindgen]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RandomErrorType {
    /// The settings can not work.  The messages say why.
    BadRequest,
    /// No numbers that match the rules were found in the attempts that are allowed
    Failed,
    /// A number is not a valid character, so the numbers can not be a string
    InvalidCharacter,
}

/// Why Settings.generate, or a string from its numbers, did not work.  This is what they throw.
#[wasm_bindgen]
pub struct RandomError(adv::RandomError);

impl RandomError {
    pub fn new(random_error: adv::RandomError) -> RandomError {
        return RandomError(random_error);
    }
}

#[wasm_bindgen]
impl RandomError {
    #[wasm_bindgen(js_name = "errorType")]
    pub fn error_type(&self) -> RandomErrorType {
        return match self.0.error_type() {
            adv::RandomErrorType::BadRequest => RandomErrorType::BadRequest,
            adv::RandomErrorType::InvalidCharacter => RandomErrorType::InvalidCharacter,
            _ => RandomErrorType::Failed
        };
    }

    pub fn messages(&self) -> Vec<String> {
        return self.0.messages().clone();
    }

    pub fn attempts(&self) -> usize {
        return self.0.attempts();
    }

    /// The settings, the attempts and everything that happened in every attempt, for debugging
    pub fn report(&self) -> String {
        return self.0.report();
    }

    /// Always "RandomError", like the name of an Error
    #[wasm_bindgen(getter)]
    pub fn name(&self) -> String {
        return String::from("RandomError");
    }

    /// The short version, such as "BadRequest: NumberRange min: 20 is greater than max: 1"
    #[wasm_bindgen(getter)]
    pub fn message(&self) -> String {
        return self.0.to_string();
    }

    /// The short version, such as "BadRequest: NumberRange min: 20 is greater than max: 1"
    #[wasm_bindgen(js_name = "toString")]
    pub fn to_string_js(&self) -> String {
        return self.0.to_string();
    }
}
