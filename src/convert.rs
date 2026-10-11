use adv_random::prelude as adv;
use js_sys::{Array, Function, Reflect};
use wasm_bindgen::prelude::*;

//usize is 32 bits in wasm
const MAX_NUMBER: f64 = u32::MAX as f64;

//JavaScript numbers are floats, so every number is checked before it is used
pub fn to_usize(value: f64, name: &str) -> Result<usize, JsError> {
    if (0.0..=MAX_NUMBER).contains(&value) && value.fract() == 0.0 {
        return Ok(value as usize);
    }
    return Err(JsError::new(&format!("{}: {} must be a whole number from 0 to {}", name, value, MAX_NUMBER)));
}

pub fn js_to_usize(value: &JsValue, name: &str) -> Result<usize, JsError> {
    return match value.as_f64() {
        Some(v) => to_usize(v, name),
        None => Err(JsError::new(&format!("{}: {:?} is not a number", name, value)))
    };
}

/// Every number from min to max.  This is what 1..=10 is in Rust.  Create one with range(min, max).
#[wasm_bindgen]
#[derive(Clone, Copy, Debug)]
pub struct RangeInclusive {
    min: usize,
    max: usize,
}

#[wasm_bindgen]
impl RangeInclusive {
    #[wasm_bindgen(constructor)]
    pub fn constructor(min: f64, max: f64) -> Result<RangeInclusive, JsError> {
        return range(min, max);
    }

    #[wasm_bindgen(getter)]
    pub fn min(&self) -> usize {
        return self.min;
    }

    #[wasm_bindgen(getter)]
    pub fn max(&self) -> usize {
        return self.max;
    }

    #[wasm_bindgen(js_name = "toString")]
    pub fn to_string_js(&self) -> String {
        return format!("{}..={}", self.min, self.max);
    }
}

/// Every number from min to max.  min and max are inclusive.  Use it where Rust uses a range like 1..=10.
#[wasm_bindgen]
pub fn range(min: f64, max: f64) -> Result<RangeInclusive, JsError> {
    return Ok(RangeInclusive { min: to_usize(min, "min")?, max: to_usize(max, "max")? });
}

fn property(value: &JsValue, key: &str) -> Option<JsValue> {
    if !value.is_object() {
        return None;
    }
    return Reflect::get(value, &JsValue::from_str(key)).ok().filter(|x| !x.is_undefined());
}

//The min and max of range(min, max), or of any object that has a min and a max
fn min_max(value: &JsValue) -> Option<Result<(usize, usize), JsError>> {
    let min = property(value, "min")?;
    let max = property(value, "max")?;
    return Some(js_to_usize(&min, "min").and_then(|min| js_to_usize(&max, "max").map(|max| (min, max))));
}

fn array(value: &JsValue) -> Option<Array> {
    if Array::is_array(value) {
        return Some(Array::from(value));
    }
    //typed arrays, sets and anything else that can be iterated or has a length
    if value.is_object() && (property(value, "length").is_some() || Reflect::has(value, &js_sys::Symbol::iterator()).unwrap_or(false)) {
        return Some(Array::from(value));
    }
    return None;
}

fn array_to_numbers(array: &Array, name: &str) -> Result<Vec<usize>, JsError> {
    let mut numbers: Vec<usize> = Vec::with_capacity(array.length() as usize);
    for item in array.iter() {
        numbers.push(js_to_usize(&item, name)?);
    }
    return Ok(numbers);
}

//A list of numbers, or a range
pub fn numbers_from_js(value: &JsValue, name: &str) -> Result<Vec<usize>, JsError> {
    if let Some(min_max) = min_max(value) {
        let (min, max) = min_max?;
        return Ok((min..=max).collect());
    }
    return match array(value) {
        Some(array) => array_to_numbers(&array, name),
        None => Err(JsError::new(&format!("{} must be a list of numbers like [0, 1] or a range like range(0, 5)", name)))
    };
}

//A list of lists of numbers
pub fn number_sets_from_js(value: &JsValue, name: &str) -> Result<Vec<Vec<usize>>, JsError> {
    let array = array(value).ok_or_else(|| JsError::new(&format!("{} must be a list of lists of numbers", name)))?;
    let mut number_sets: Vec<Vec<usize>> = Vec::with_capacity(array.length() as usize);
    for item in array.iter() {
        number_sets.push(numbers_from_js(&item, name)?);
    }
    return Ok(number_sets);
}

//A pool is a list of numbers, a range, or the characters of a string
pub fn pool_from_js(value: &JsValue) -> Result<adv::PoolType, JsError> {
    if let Some(characters) = value.as_string() {
        return Ok(adv::PoolType::from(characters.as_str()));
    }
    if let Some(min_max) = min_max(value) {
        let (min, max) = min_max?;
        return Ok(adv::PoolType::MinMax(min, max));
    }
    return match array(value) {
        Some(array) => Ok(adv::PoolType::set(array_to_numbers(&array, "pool")?)),
        None => Err(JsError::new("pool must be a list of numbers like [1, 4], a range like range(1, 10), or a string like \"ABC\""))
    };
}

//Calls the hidden method that the rules and the random number generators have.  It returns a new handle, so the
//object that was passed in is not used up and can be passed in again.
pub fn handle_from_js(value: &JsValue) -> Option<JsValue> {
    let handle: Function = property(value, "__handle")?.dyn_into().ok()?;
    return handle.call0(value).ok();
}

//True when the value is the class itself, like NoDuplicate, and not an object of the class
pub fn is_class_of(value: &JsValue, object: JsValue) -> bool {
    return value.is_function() && property(&object, "constructor").is_some_and(|x| x == *value);
}
