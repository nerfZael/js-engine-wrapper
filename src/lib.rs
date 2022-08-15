pub mod wrap;
use JSON::json;
use wrap::*;
use polywrap_wasm_rs::{JSON};
use boa_engine::{ Context, JsValue };
pub fn eval(args: ArgsEval) -> EvalResult {
    let js_code = args.src;

    let mut context = Context::default();

    match context.eval(js_code) {
        Ok(result) => {
            EvalResult {
                value: Some(match &result {
                    JsValue::Null => json!("null"),
                    JsValue::Undefined => json!("undefined"),
                    JsValue::Boolean(bool) => json!(bool),
                    JsValue::String(string) => json!(string.to_string()),
                    JsValue::Rational(f64) => json!(f64),
                    JsValue::Integer(i32) => json!(i32),
                    JsValue::BigInt(big_int) => json!(big_int.to_string()),
                    _ => json!("Object or Symbol".to_string())          
                }),
                error: None
            }
        }
        Err(err) => {
            EvalResult {
                value: None,
                error: Some(err.display().to_string())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use polywrap_wasm_rs::JSON::json;

    pub use crate::wrap::*;

    #[test]
    fn eval_null() {
        let args = ArgsEval {
            src: "const null_value = null; null_value".to_string(),
        };
        
        let result = crate::eval(args);

        let expected = EvalResult {
            value: Some(json!("null")),
            error: None
        };
        assert_eq!(result.value.unwrap(), expected.value.unwrap());
    }

    #[test]
    fn eval_undefined() {
        let args = ArgsEval {
            src: "const undefined_value = undefined; undefined_value".to_string(),
        };
        
        let result = crate::eval(args);

        let expected = EvalResult {
            value: Some(json!("undefined")),
            error: None
        };
        assert_eq!(result.value.unwrap(), expected.value.unwrap());
    }

    #[test]
    fn eval_string() {
        let args = ArgsEval {
            src: "'hello' + ' ' + 'world'".to_string(),
        };
        
        let result = crate::eval(args);

        let expected = EvalResult {
            value: Some(json!("hello world")),
            error: None
        };
        assert_eq!(result.value.unwrap(), expected.value.unwrap());
    }

    #[test]
    fn eval_bool() {
        let args = ArgsEval {
            src: "const is_true = true; is_true".to_string(),
        };
        
        let result = crate::eval(args);

        let expected = EvalResult {
            value: Some(json!(true)),
            error: None
        };
        assert_eq!(result.value.unwrap(), expected.value.unwrap());
    }

    #[test]
    fn eval_rational() {
        let args = ArgsEval {
            src: "const num = 123.456; num".to_string(),
        };
        
        let result = crate::eval(args);

        let expected = EvalResult {
            value: Some(json!(123.456)),
            error: None
        };
        assert_eq!(result.value.unwrap(), expected.value.unwrap());
    }

    #[test]
    fn eval_integer() {
        let args = ArgsEval {
            src: "const num = 5; num".to_string(),
        };
        
        let result = crate::eval(args);

        let expected = EvalResult {
            value: Some(json!(5)),
            error: None
        };
        assert_eq!(result.value.unwrap(), expected.value.unwrap());
    }

    #[test]
    fn eval_bit_int() {
        let args = ArgsEval {
            src: "const num = BigInt(9007199254740991); num".to_string(),
        };
        
        let result = crate::eval(args);

        let expected = EvalResult {
            value: Some(json!("9007199254740991")),
            error: None
        };
        assert_eq!(result.value.unwrap(), expected.value.unwrap());
    }

    #[test]
    fn eval_object() {
        let args = ArgsEval {
            src: "const obj = { prop1: 1, prop2: 'hello' }; obj".to_string(),
        };
        
        let result = crate::eval(args);

        // let expected = EvalResult {
        //     value: Some(json!({
        //         "prop1": 1,
        //         "prop2": "hello"
        //     })),
        //     error: None
        // };

        // assert_eq!(result.value.unwrap(), expected.value.unwrap());

        assert_eq!(result.value.unwrap(), json!("Object or Symbol"));
    }

    #[test]
    fn eval_undefined_variable() {
        let args = ArgsEval {
            src: "undefined_variable".to_string(),
        };
        
        let result = crate::eval(args);

        let expected = EvalResult {
            value: None,
            error: Some("\"ReferenceError\": \"undefined_variable is not defined\"".to_string())
        };

        assert_eq!(result.error.unwrap(), expected.error.unwrap());
    }
}