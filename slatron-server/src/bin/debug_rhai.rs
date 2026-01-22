use rhai::{Engine, Scope};

fn register_content_loader_functions(engine: &mut Engine) {
    engine.register_fn(
        "shell_execute",
        |cmd: String, _args: rhai::Array| -> rhai::Map {
            println!("MOCKED shell_execute: {}", cmd);
            let mut map = rhai::Map::new();
            map.insert("code".into(), 0_i64.into());
            map.insert("stdout".into(), "{}".into());
            map.insert("stderr".into(), "".into());
            map
        },
    );
    engine.register_fn("to_json", |_v: rhai::Dynamic| -> String {
        "{}".to_string()
    });
}

fn main() {
    let script =
        std::fs::read_to_string("temp_repro.rhai").expect("Failed to read temp_repro.rhai");

    println!("Script Loaded ({} bytes)", script.len());

    let mut engine = Engine::new();
    engine.on_print(|x| println!("[SCRIPT] {}", x));

    // Register functions
    register_content_loader_functions(&mut engine);

    // Override parse_json with serde_json
    engine.register_fn("parse_json", |json: String| -> rhai::Dynamic {
        match serde_json::from_str::<serde_json::Value>(&json) {
            Ok(v) => rhai::serde::to_dynamic(&v).unwrap_or(rhai::Dynamic::UNIT),
            Err(e) => {
                println!("JSON Parse Error: {}", e);
                rhai::Dynamic::UNIT
            }
        }
    });

    println!("\nCompiling...");
    let ast = match engine.compile(&script) {
        Ok(ast) => ast,
        Err(e) => {
            println!("Compilation Error: {}", e);
            return;
        }
    };

    let mut scope = Scope::new();

    println!("\nEvaluating AST (Top Level)...");
    match engine.eval_ast_with_scope::<rhai::Dynamic>(&mut scope, &ast) {
        Ok(_) => println!("Top Level Eval Complete"),
        Err(e) => println!("Top Level Eval Error: {}", e),
    }

    println!("\nCalling load_content...");
    match engine.call_fn::<rhai::Dynamic>(&mut scope, &ast, "load_content", ()) {
        Ok(_) => println!("Call load_content Complete"),
        Err(e) => {
            println!("Call load_content Error: {}", e);
            if let rhai::EvalAltResult::ErrorFunctionNotFound(sig, _) = *e {
                println!("Fn Not Found: {}", sig)
            }
        }
    }
}
