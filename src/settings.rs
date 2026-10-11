use crate::convert::to_usize;
use crate::random::{random_from_js, JsRandom};
use crate::results::{RandomError, RandomNumbers};
use crate::rules::rule_from_js;
use adv_random::prelude as adv;
use std::sync::Arc;
use wasm_bindgen::prelude::*;

/// The count and the rules for creating random numbers.  Create one with Settings.builder(count).
#[wasm_bindgen]
pub struct Settings {
    settings: adv::Settings,
    js_random: Option<Arc<JsRandom>>,
}

#[wasm_bindgen]
impl Settings {
    /// Starts the settings for count random numbers
    pub fn builder(count: f64) -> Result<SettingsBuilder, JsError> {
        return Ok(SettingsBuilder { builder: adv::Settings::builder(to_usize(count, "count")?), js_random: None });
    }

    /// Creates the random numbers.  Throws a RandomError when the settings can not work or no numbers were found.
    /// Every call creates new random numbers.
    pub fn generate(&self) -> Result<RandomNumbers, JsValue> {
        let result = self.settings.generate();
        if let Some(js_random) = &self.js_random {
            js_random.check()?;
        }
        return match result {
            Ok(random_numbers) => Ok(RandomNumbers::new(random_numbers, self.js_random.clone())),
            Err(e) => Err(RandomError::new(e).into())
        };
    }

    /// Problems found with the rules.  generate throws a BadRequest error when there are any.
    pub fn errors(&self) -> Vec<String> {
        return self.settings.errors().to_vec();
    }

    pub fn count(&self) -> usize {
        return self.settings.count();
    }

    #[wasm_bindgen(js_name = "toString")]
    pub fn to_string_js(&self) -> String {
        return format!("{:?}", self.settings);
    }
}

/// Builds Settings.  Create one with Settings.builder(count).
#[wasm_bindgen]
pub struct SettingsBuilder {
    builder: adv::SettingsBuilder,
    js_random: Option<Arc<JsRandom>>,
}

impl SettingsBuilder {
    fn with(&self, builder: adv::SettingsBuilder) -> SettingsBuilder {
        return SettingsBuilder { builder, js_random: self.js_random.clone() };
    }
}

#[wasm_bindgen]
impl SettingsBuilder {
    /// Adds a rule that the random numbers have to match
    pub fn rule(&self, #[wasm_bindgen(unchecked_param_type = "Rule")] rule: &JsValue) -> Result<SettingsBuilder, JsError> {
        return Ok(self.with(self.builder.clone().boxed_rules([rule_from_js(rule)?.into_rule()])));
    }

    /// Adds a rule that the random numbers can not match
    pub fn exclude(&self, #[wasm_bindgen(unchecked_param_type = "Rule")] rule: &JsValue) -> Result<SettingsBuilder, JsError> {
        return Ok(self.with(self.builder.clone().boxed_excludes([rule_from_js(rule)?.into_exclude_rule()])));
    }

    /// Every number is from min to max.  The same as rule(NumberRange.all(min, max)).
    pub fn range(&self, min: f64, max: f64) -> Result<SettingsBuilder, JsError> {
        return Ok(self.with(self.builder.clone().range(to_usize(min, "min")?, to_usize(max, "max")?)));
    }

    /// No number is there more than once.  The same as rule(NoDuplicate).
    #[wasm_bindgen(js_name = "noDuplicates")]
    pub fn no_duplicates(&self) -> SettingsBuilder {
        return self.with(self.builder.clone().no_duplicates());
    }

    /// This many odd numbers and this many even numbers.  The same as rule(OddEven.new().odd(odd).even(even)).
    #[wasm_bindgen(js_name = "oddEven")]
    pub fn odd_even(&self, odd: f64, even: f64) -> Result<SettingsBuilder, JsError> {
        return Ok(self.with(self.builder.clone().odd_even(to_usize(odd, "odd")?, to_usize(even, "even")?)));
    }

    /// Uses this random number generator instead of DefaultRandom
    pub fn random(&self, #[wasm_bindgen(unchecked_param_type = "Random")] random: &JsValue) -> Result<SettingsBuilder, JsError> {
        let (random, js_random) = random_from_js(random)?;
        return Ok(SettingsBuilder { builder: self.builder.clone().random(random), js_random });
    }

    /// Returns the Settings.  They can be used as many times as you want.
    pub fn build(&self) -> Result<Settings, JsValue> {
        //NumberPool.alphanumeric picks how many digits there are with the random number generator when it is built
        let settings = self.builder.clone().build();
        if let Some(js_random) = &self.js_random {
            js_random.check()?;
        }
        return Ok(Settings { settings, js_random: self.js_random.clone() });
    }
}
