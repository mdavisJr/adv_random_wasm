use crate::convert::handle_from_js;
use adv_random::prelude as adv;
use js_sys::{Function, Reflect};
use std::fmt;
use std::sync::{Arc, Mutex};
use wasm_bindgen::convert::TryFromJsValue;
use wasm_bindgen::prelude::*;

//The biggest whole number that a JavaScript number can hold without losing any of it
const MAX_SAFE_INTEGER: f64 = 9007199254740991.0;

//A random number generator that the settings and the JavaScript object it came from both use
#[derive(Clone, Debug)]
pub struct SharedRandom(Arc<dyn adv::RandomTrait + Send + Sync>);

impl adv::RandomTrait for SharedRandom {
    fn get_number(&self, min: usize, max: usize) -> usize {
        return self.0.get_number(min, max);
    }

    fn get_bool(&self) -> bool {
        return self.0.get_bool();
    }
}

//A random number generator that was taken out of its JavaScript object
#[doc(hidden)]
#[wasm_bindgen(js_name = "__RandomHandle", skip_typescript)]
pub struct RandomHandle(SharedRandom);

/// The random number generator that is used when the settings do not have one.
#[wasm_bindgen]
pub struct DefaultRandom(SharedRandom);

#[wasm_bindgen]
impl DefaultRandom {
    #[wasm_bindgen(constructor)]
    pub fn constructor() -> DefaultRandom {
        return DefaultRandom::new();
    }

    pub fn new() -> DefaultRandom {
        return DefaultRandom(SharedRandom(Arc::new(adv::DefaultRandom::new())));
    }

    #[doc(hidden)]
    #[wasm_bindgen(js_name = "__handle", skip_typescript)]
    pub fn handle(&self) -> RandomHandle {
        return RandomHandle(self.0.clone());
    }
}

/// Gives the same numbers every time for the same seed.
#[wasm_bindgen]
pub struct SeededRandom(SharedRandom);

#[wasm_bindgen]
impl SeededRandom {
    /// seed is a whole number that is 0 or more.  A bigint can be used for a seed that is too big for a number.
    #[wasm_bindgen(constructor)]
    pub fn constructor(#[wasm_bindgen(unchecked_param_type = "number | bigint")] seed: &JsValue) -> Result<SeededRandom, JsError> {
        return SeededRandom::new(seed);
    }

    /// seed is a whole number that is 0 or more.  A bigint can be used for a seed that is too big for a number.
    pub fn new(#[wasm_bindgen(unchecked_param_type = "number | bigint")] seed: &JsValue) -> Result<SeededRandom, JsError> {
        return Ok(SeededRandom(SharedRandom(Arc::new(adv::SeededRandom::new(seed_from_js(seed)?)))));
    }

    #[doc(hidden)]
    #[wasm_bindgen(js_name = "__handle", skip_typescript)]
    pub fn handle(&self) -> RandomHandle {
        return RandomHandle(self.0.clone());
    }
}

fn seed_from_js(seed: &JsValue) -> Result<u64, JsError> {
    if let Some(v) = seed.as_f64() {
        if (0.0..=MAX_SAFE_INTEGER).contains(&v) && v.fract() == 0.0 {
            return Ok(v as u64);
        }
    }
    if seed.is_bigint() {
        if let Ok(v) = u64::try_from(seed.clone().unchecked_into::<js_sys::BigInt>()) {
            return Ok(v);
        }
    }
    return Err(JsError::new("seed must be a whole number from 0 to Number.MAX_SAFE_INTEGER, or a bigint from 0 to 18446744073709551615n"));
}

//A random number generator that is written in JavaScript: an object with getNumber(min, max) and getBool().
//RandomTrait can not return an error, so the first error is kept here and thrown when the settings are done.
pub struct JsRandom {
    target: JsValue,
    get_number: Function,
    get_bool: Function,
    error: Mutex<Option<JsValue>>,
}

//JsValue can only be used on the thread it was created on.  The npm package is built without threads, so there is
//only one thread that can use it.
unsafe impl Send for JsRandom {}
unsafe impl Sync for JsRandom {}

impl fmt::Debug for JsRandom {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "JsRandom")
    }
}

impl JsRandom {
    fn set_error(&self, error: JsValue) {
        let mut current_error = self.error.lock().unwrap();
        if current_error.is_none() {
            *current_error = Some(error);
        }
    }

    fn has_error(&self) -> bool {
        return self.error.lock().unwrap().is_some();
    }

    //Returns the error from the generator, if there was one, so it can be thrown
    pub fn check(&self) -> Result<(), JsValue> {
        return match self.error.lock().unwrap().take() {
            Some(error) => Err(error),
            None => Ok(())
        };
    }
}

impl adv::RandomTrait for JsRandom {
    fn get_number(&self, min: usize, max: usize) -> usize {
        if self.has_error() {
            return min;
        }
        match self.get_number.call2(&self.target, &JsValue::from(min as u32), &JsValue::from(max as u32)) {
            Ok(value) => {
                match value.as_f64() {
                    Some(v) if v >= min as f64 && v <= max as f64 && v.fract() == 0.0 => return v as usize,
                    _ => self.set_error(JsError::new(&format!("getNumber({}, {}) returned {:?}.  It must return a whole number from min to max", min, max, value)).into())
                }
            },
            Err(error) => self.set_error(error)
        }
        return min;
    }

    fn get_bool(&self) -> bool {
        if self.has_error() {
            return false;
        }
        match self.get_bool.call0(&self.target) {
            Ok(value) => {
                match value.as_bool() {
                    Some(v) => return v,
                    None => self.set_error(JsError::new(&format!("getBool() returned {:?}.  It must return true or false", value)).into())
                }
            },
            Err(error) => self.set_error(error)
        }
        return false;
    }
}

fn function(value: &JsValue, name: &str) -> Option<Function> {
    if !value.is_object() {
        return None;
    }
    return Reflect::get(value, &JsValue::from_str(name)).ok()?.dyn_into().ok();
}

//Returns the generator for the settings.  The JsRandom is also returned when the generator is written in JavaScript,
//to check it for errors.
pub fn random_from_js(value: &JsValue) -> Result<(SharedRandom, Option<Arc<JsRandom>>), JsError> {
    if let Some(random_handle) = handle_from_js(value).and_then(|x| RandomHandle::try_from_js_value(x).ok()) {
        return Ok((random_handle.0, None));
    }
    return match (function(value, "getNumber"), function(value, "getBool")) {
        (Some(get_number), Some(get_bool)) => {
            let js_random = Arc::new(JsRandom { target: value.clone(), get_number, get_bool, error: Mutex::new(None) });
            Ok((SharedRandom(js_random.clone()), Some(js_random)))
        },
        _ => Err(JsError::new("random must be SeededRandom.new(seed), DefaultRandom.new(), or an object with getNumber(min, max) and getBool()"))
    };
}
