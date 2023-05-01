use lambda_runtime::{service_fn, Error as LambdaError, LambdaEvent};
use precise_calc::context::Context as CalcContext;
use precise_calc::eval::eval_stmt;
use precise_calc::parser::parse_stmt_list;
use precise_calc::CalcError;
use serde::Deserialize;
use serde_json::{json, Value};

#[tokio::main]
async fn main() -> Result<(), LambdaError> {
    let func = service_fn(handler);
    lambda_runtime::run(func).await?;
    Ok(())
}

#[derive(Deserialize)]
struct CalcEvent {
    stmts: String,
}

async fn handler(event: LambdaEvent<CalcEvent>) -> Result<Value, LambdaError> {
    // Parse statements
    let (rest, stmts) = match parse_stmt_list(&event.payload.stmts) {
        Ok((rest, stmts)) => (rest, stmts),
        Err(_) => return Err(Box::new(CalcError::ParseError)),
    };

    // Make sure we parsed everything
    if !rest.is_empty() {
        return Ok(json!("ERROR: failed to parse all input"));
    }

    // Create evaluation context
    let mut ctx = CalcContext::new();

    // Evaluate statements
    let mut results = vec![];
    for stmt in stmts {
        let res = match eval_stmt(&stmt, &mut ctx) {
            Ok(val) => val.to_string(),
            Err(e) => e.to_string(),
        };
        results.push(res);
    }

    Ok(json!({ "results": results }))
}
