mod wrap;
use wrap::*;
use boa_engine::native_function::NativeFunctionPointer;
use polywrap_wasm_rs::subinvoke;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use wrap::{EvalResult, ArgsEval, ModuleTrait, Module};
use getrandom::register_custom_getrandom;
use boa_engine::module::ModuleLoader;
use boa_engine::{Context, Source, JsString, NativeFunction, JsResult, JsValue};

fn custom_getrandom(_: &mut [u8]) -> Result<(), getrandom::Error> {
    return Ok(());
}

register_custom_getrandom!(custom_getrandom);

impl ModuleTrait for Module {
    fn eval(args: ArgsEval) -> Result<EvalResult, String> {
        let result = eval_and_parse(&args.src, vec![
            GlobalFun {
                name: "subinvoke".to_string(),
                function: subinvoke,
            },
        ]);

        let result = result?.unwrap();

        let result = EvalResult {
            value: Some(result),
            error: None
        };

        Ok(result)
    }
}

#[derive(Debug)]
pub struct CustomModuleLoader {
}

impl CustomModuleLoader {
    pub fn new() -> JsResult<Self> {
        Ok(Self {
        })
    }
}

impl ModuleLoader for CustomModuleLoader {
    fn load_imported_module(
        &self,
        _referrer: boa_engine::module::Referrer,
        _specifier: JsString,
        _finish_load: Box<dyn FnOnce(JsResult<boa_engine::Module>, &mut Context<'_>)>,
        _context: &mut Context<'_>,
    ) {
    }
}

pub struct GlobalFun {
    pub name: String,
    pub function: NativeFunctionPointer,
}

/// Evaluate the given ECMAScript code.
pub fn eval_and_parse(src: &str, globals: Vec<GlobalFun>) -> Result<Option<Value>, String>  {
    // This can be overriden with any custom implementation of `ModuleLoader`.
    let loader = &CustomModuleLoader::new().unwrap();
    let dyn_loader: &dyn ModuleLoader = loader;

    // Just need to cast to a `ModuleLoader` before passing it to the builder.
    let mut ctx = &mut Context::builder().module_loader(dyn_loader).build().unwrap();

    for global in globals {
        ctx.register_global_callable(&global.name, 0, NativeFunction::from_fn_ptr(global.function))
            .unwrap();
    }

    let result = ctx.eval(Source::from_bytes(src.as_bytes()));

    let val = result.map_err(|err| err.to_string())?.to_json(&mut ctx).unwrap();
    Ok(Some(val))
}

fn subinvoke(_: &JsValue, args: &[JsValue], ctx: &mut Context<'_>) -> JsResult<JsValue> {
    let uri = args.get(0).unwrap();
    let uri: String = uri.as_string().unwrap().to_std_string().unwrap();
    
    let method = args.get(1).unwrap();
    let method = method.as_string().unwrap().to_std_string().unwrap();

    let args = args.get(2).unwrap();
    let args = args.to_json(ctx).unwrap().to_string();
    let args = json_to_msgpack(&args);

    let result: Result<Vec<u8>, String> = subinvoke::wrap_subinvoke(
        &uri,
        &method,
        args,
    );

    // let input = MockType {
    //     prop: uri + &method,
    // };
    // let result: Result<Vec<u8>, String> = Ok(rmp_serde::encode::to_vec_named(&input).unwrap());

    let result = match result {
        Ok(result) => msgpack_to_json(&result),
        Err(err) => {
            serde_json::to_string(&err).unwrap()
        }
    };

    let result = match serde_json::from_str(&result) {
        Ok(json) => JsValue::from_json(&json, ctx).unwrap(),
        Err(err) => {
            let json = serde_json::to_string(&err.to_string()).unwrap();
            let json = serde_json::from_str(&json).unwrap();
            JsValue::from_json(&json, ctx).unwrap()
        }
    };

    Ok(result)
}

pub fn mock_subinvoke(_: &JsValue, args: &[JsValue], ctx: &mut Context<'_>) -> JsResult<JsValue> {
    let uri = args.get(0).unwrap();
    let uri: String = uri.as_string().unwrap().to_std_string().unwrap();
    
    let method = args.get(1).unwrap();
    let method = method.as_string().unwrap().to_std_string().unwrap();

    let args = args.get(2).unwrap();
    let args = args.to_json(ctx).unwrap().to_string();
    let args = json_to_msgpack(&args);

    let input = MockType {
        prop: uri + &method + &args.len().to_string(),
    };
    let bytes = rmp_serde::encode::to_vec(&input).unwrap();

    let result = msgpack_to_json(&bytes);

    let json: serde_json::Value = serde_json::from_str(&result).unwrap();

    let result = JsValue::from_json(&json, ctx).unwrap();

    Ok(result)
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct MockType {
    pub prop: String,
}

pub fn msgpack_to_json(bytes: &[u8]) -> String {
    let value: rmpv::Value = rmp_serde::from_slice(&bytes).unwrap();
    serde_json::to_string(&value).unwrap()
}

pub fn json_to_msgpack(string: &str) -> Vec<u8> {
    let value: serde_json::Value = serde_json::from_str(string).unwrap();
    rmp_serde::encode::to_vec(&value).unwrap()
}

#[cfg(test)]
mod tests {
    use boa_engine::{JsValue, Context, JsResult};

    use crate::{msgpack_to_json, json_to_msgpack, MockType, GlobalFun, eval_and_parse};

    pub fn mock_subinvoke(_: &JsValue, args: &[JsValue], ctx: &mut Context<'_>) -> JsResult<JsValue> {
        let uri = args.get(0).unwrap();
        let uri: String = uri.as_string().unwrap().to_std_string().unwrap();
        
        let method = args.get(1).unwrap();
        let method = method.as_string().unwrap().to_std_string().unwrap();
    
        let args = args.get(2).unwrap();
        let args = args.to_json(ctx).unwrap().to_string();
        let args = json_to_msgpack(&args);
    
        let input = MockType {
            prop: uri + &method + &args.len().to_string(),
        };
        let bytes = rmp_serde::encode::to_vec(&input).unwrap();
    
        let result = msgpack_to_json(&bytes);
    
        let json: serde_json::Value = serde_json::from_str(&result).unwrap();
    
        let result = JsValue::from_json(&json, ctx).unwrap();
    
        Ok(result)
    }
    
    #[test]
    fn end_to_end_serialization() {
        let input = MockType {
            prop: "a".to_string(),
        };
        let bytes = rmp_serde::encode::to_vec(&input).unwrap();

        let json = msgpack_to_json(&bytes);
        let result = json_to_msgpack(&json);

        assert_eq!(result, bytes);
    }

    #[test]
    fn end_to_end() {
        let src: String = r#"function doStuff(args) {
            let a = subinvoke('Hello Turtle', 'doStuff', args);
            return a;
        }"#.to_string();

        let result = eval_and_parse(&src, vec![
            GlobalFun {
                name: "subinvoke".to_string(),
                function: mock_subinvoke,
            },
        ]);

        let result = result.unwrap().unwrap();
        let result: MockType = serde_json::from_value(result).unwrap();

        let expected = MockType {
            prop: "Hello World".to_string()
        };
        
        assert_eq!(result, expected);
    }

    #[test]
    fn closures() {
        let src = r#"
        function doStuff(args) {
            function alog(a) {
              return subinvoke("ens/logger.eth", "debug", { message: "as" });
            };
  
            alog("test");
            return {
              prop: "x"
            };
          }
        doStuff({ prop: "aa" });
        "#.to_string();

        let result = eval_and_parse(&src, vec![
            GlobalFun {
                name: "subinvoke".to_string(),
                function: mock_subinvoke,
            },
        ]);

        let result = result.unwrap().unwrap();
        let result: MockType = serde_json::from_str(&result.to_string()).unwrap();

        let expected = MockType {
            prop: "Hello TurtledoStuff9".to_string()
        };
        
        assert_eq!(result, expected);
    }

    // #[test]
    // fn import() {
    //     let args = ArgsEval {
    //         src: "const lol = invoke('Hello Turtle');lol".to_string(),
    //     };

    //     // Creating the execution context
    //     let mut ctx = Context::default();

    //     ctx.register_global_callable("invoke", 0, NativeFunction::from_fn_ptr(invoke))
    //         .unwrap();

    //     let result = ctx.eval(Source::from_bytes(args.src.as_bytes()));

    //     let expected = EvalResult {
    //         value: Some(json!("Hello, world!")),
    //         error: None
    //     };

    //     let a = parse_result(result);
    //     assert_eq!(a.unwrap().value.unwrap(), expected.value.unwrap());
    // }

    // #[test]
    // fn eval_null() {
    //     let args = ArgsEval {
    //         src: "const null_value = null; null_value".to_string(),
    //     };
        
    //     let result = crate::eval(args);

    //     let expected = EvalResult {
    //         value: Some(json!("null")),
    //         error: None
    //     };
    //     assert_eq!(result.unwrap().value.unwrap(), expected.value.unwrap());
    // }

    // #[test]
    // fn eval_undefined() {
    //     let args = ArgsEval {
    //         src: "const undefined_value = undefined; undefined_value".to_string(),
    //     };
        
    //     let result = crate::eval(args);

    //     let expected = EvalResult {
    //         value: Some(json!("undefined")),
    //         error: None
    //     };
    //     assert_eq!(result.unwrap().value.unwrap(), expected.value.unwrap());
    // }

    // #[test]
    // fn eval_string() {
    //     let args = ArgsEval {
    //         src: "'hello' + ' ' + 'world'".to_string(),
    //     };
        
    //     let result = crate::eval(args);

    //     let expected = EvalResult {
    //         value: Some(json!("hello world")),
    //         error: None
    //     };
    //     assert_eq!(result.unwrap().value.unwrap(), expected.value.unwrap());
    // }

    // #[test]
    // fn eval_bool() {
    //     let args = ArgsEval {
    //         src: "const is_true = true; is_true".to_string(),
    //     };
        
    //     let result = crate::eval(args);

    //     let expected = EvalResult {
    //         value: Some(json!(true)),
    //         error: None
    //     };
    //     assert_eq!(result.unwrap().value.unwrap(), expected.value.unwrap());
    // }

    // #[test]
    // fn eval_rational() {
    //     let args = ArgsEval {
    //         src: "const num = 123.456; num".to_string(),
    //     };
        
    //     let result = crate::eval(args);

    //     let expected = EvalResult {
    //         value: Some(json!(123.456)),
    //         error: None
    //     };
    //     assert_eq!(result.unwrap().value.unwrap(), expected.value.unwrap());
    // }

    // #[test]
    // fn eval_integer() {
    //     let args = ArgsEval {
    //         src: "const num = 5; num".to_string(),
    //     };
        
    //     let result = crate::eval(args);

    //     let expected = EvalResult {
    //         value: Some(json!(5)),
    //         error: None
    //     };
    //     assert_eq!(result.unwrap().value.unwrap(), expected.value.unwrap());
    // }

    // #[test]
    // fn eval_bit_int() {
    //     let args = ArgsEval {
    //         src: "const num = BigInt(9007199254740991); num".to_string(),
    //     };
        
    //     let result = crate::eval(args);

    //     let expected = EvalResult {
    //         value: Some(json!("9007199254740991")),
    //         error: None
    //     };
    //     assert_eq!(result.unwrap().value.unwrap(), expected.value.unwrap());
    // }

    // #[test]
    // fn eval_object() {
    //     let args = ArgsEval {
    //         src: "const obj = { prop1: 1, prop2: 'hello' }; JSON.stringify(obj)".to_string(),
    //     };
        
    //     let result = crate::eval(args);

    //     let serialized_obj = json!({
    //         "prop1": 1,
    //         "prop2": "hello"
    //     });
    
    //     let expected = EvalResult {
    //         value: Some(json!(serialized_obj.to_string())),
    //         error: None
    //     };

    //     assert_eq!(result.unwrap(), expected.value.unwrap());
    // }

    // #[test]
    // fn eval_undefined_variable() {
    //     let args = ArgsEval {
    //         src: "undefined_variable".to_string(),
    //     };
        
    //     let result = crate::eval(args);

    //     let expected = EvalResult {
    //         value: None,
    //         error: Some("\"ReferenceError\": \"undefined_variable is not defined\"".to_string())
    //     };

    //     assert_eq!(result.unwrap().error.unwrap(), expected.error.unwrap());
    // }
}
