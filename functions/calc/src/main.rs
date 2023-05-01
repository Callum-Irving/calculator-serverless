use aws_lambda_events::http::header::ACCESS_CONTROL_ALLOW_ORIGIN;
use aws_lambda_events::{
    encodings::Body,
    event::apigw::{ApiGatewayProxyRequest, ApiGatewayProxyResponse},
    http::HeaderMap,
};
use lambda_runtime::{service_fn, Error as LambdaError, LambdaEvent};
use log::{info, LevelFilter};
use precise_calc::context::Context as CalcContext;
use precise_calc::eval::eval_stmt;
use precise_calc::parser::parse_stmt_list;
use precise_calc::CalcError;
use serde::Deserialize;
use serde_json::{json, Value};
use simple_logger::SimpleLogger;

#[tokio::main]
async fn main() -> Result<(), LambdaError> {
    SimpleLogger::new()
        .with_level(LevelFilter::Info)
        .init()
        .expect("failed to initialize simple logger");

    let func = service_fn(handler);
    lambda_runtime::run(func).await?;
    Ok(())
}

#[derive(Deserialize)]
struct CalcEvent {
    stmts: String,
}

async fn handler(
    event: LambdaEvent<ApiGatewayProxyRequest>,
) -> Result<ApiGatewayProxyResponse, LambdaError> {
    let request = event.payload;
    info!("Method: {}", request.http_method);
    info!("Headers: {:?}", request.headers);
    info!("Body: {:?}", request.body);

    let body = request.body.expect("request had no body");
    let calc_event: CalcEvent =
        serde_json::from_str(&body).expect("couldn't deserialize body into CalcEvent");

    // Parse statements
    let (rest, stmts) = match parse_stmt_list(&calc_event.stmts) {
        Ok((rest, stmts)) => (rest, stmts),
        Err(_) => return Err(Box::new(CalcError::ParseError)),
    };

    // Make sure we parsed everything
    if !rest.is_empty() {
        return Err(Box::new(CalcError::ParseError));
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

    let mut headers = HeaderMap::new();
    headers.insert(ACCESS_CONTROL_ALLOW_ORIGIN, "*".parse().unwrap());

    // Create response
    let resp = ApiGatewayProxyResponse {
        status_code: 200,
        headers,
        multi_value_headers: HeaderMap::new(),
        body: Some(Body::Text(json!({ "results": results }).to_string())),
        is_base64_encoded: Some(false),
    };

    //Ok(json!({ "results": results }))
    Ok(resp)
}
