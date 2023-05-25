pub mod wrap;
use JSON::json;
use wrap::*;
use polywrap_wasm_rs::{JSON};
use boa_engine::{ Context, JsValue, JsString };
use boa_engine::{
    native_function::NativeFunction, prelude::JsObject, property::Attribute, Context, JsResult,
    JsValue, Source,
};
use std::fs::read_to_string;

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


pub fn invoke(_: &JsValue, args: &[JsValue], ctx: &mut Context) -> JsResult<JsValue> {
    let arg = args.get(0).unwrap();

    println!("This URI: {}", arg.as_string().unwrap());

    Ok(JsValue::String(JsString::from("Hello, world!")))
}

#[cfg(test)]
mod tests {
    use boa_engine::Context;
    use polywrap_wasm_rs::JSON::json;

    use crate::invoke;
    pub use crate::wrap::*;
    use boa_engine::{
        native_function::NativeFunction, prelude::JsObject, property::Attribute, Context, JsResult,
        JsValue, Source,
    };

    #[test]
    fn import() {
        let args = ArgsEval {
            src: "const null_value = null; null_value".to_string(),
        };

        // Creating the execution context
        let mut ctx = Context::default();

        // Adding custom implementation that mimics 'require'
        ctx.register_global_function("invoke", 0, NativeFunction::from_fn_ptr(invoke))
            .unwrap();

        // Adding custom object that mimics 'module.exports'
        let moduleobj = JsObject::default();
        moduleobj
            .set("exports", JsValue::from(" "), false, &mut ctx)
            .unwrap();
        ctx.register_global_property("module", JsValue::from(moduleobj), Attribute::default())
            .unwrap();

        // Instantiating the engine with the execution context
        // Loading, parsing and executing the JS code from the source file
        ctx.eval(Source::from_bytes(&buffer.unwrap())).unwrap();
        
        let result = crate::eval(args);

        let expected = EvalResult {
            value: Some(json!("null")),
            error: None
        };
        assert_eq!(result.value.unwrap(), expected.value.unwrap());
    }

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
            src: "const obj = { prop1: 1, prop2: 'hello' }; JSON.stringify(obj)".to_string(),
        };
        
        let result = crate::eval(args);

        let serialized_obj = json!({
            "prop1": 1,
            "prop2": "hello"
        });
    
        let expected = EvalResult {
            value: Some(json!(serialized_obj.to_string())),
            error: None
        };

        assert_eq!(result.value.unwrap(), expected.value.unwrap());
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