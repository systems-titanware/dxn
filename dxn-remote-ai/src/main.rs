// Remote AI Function Server Example
// This is a standalone server that provides AI functions via HTTP

use actix_web::{web, App, HttpServer, HttpResponse, Responder};
use dxn_shared::{FunctionRequest, FunctionResponse};
use serde_json::json;

async fn generate_text(req: web::Json<FunctionRequest>) -> impl Responder {
    // Extract prompt from params
    let prompt = req.params.get("prompt")
        .and_then(|v| v.as_str())
        .unwrap_or("");

    // Mock AI generation (in real implementation, call actual AI model)
    let result = format!("Generated text for prompt: '{}'", prompt);
    
    HttpResponse::Ok().json(FunctionResponse::success(json!(result)))
}

async fn analyze_sentiment(req: web::Json<FunctionRequest>) -> impl Responder {
    let text = req.params.get("text")
        .and_then(|v| v.as_str())
        .unwrap_or("");

    // Mock sentiment analysis
    let sentiment = if text.contains("happy") || text.contains("great") {
        "positive"
    } else if text.contains("sad") || text.contains("bad") {
        "negative"
    } else {
        "neutral"
    };
    
    HttpResponse::Ok().json(FunctionResponse::success(json!({
        "sentiment": sentiment,
        "confidence": 0.85
    })))
}

async fn health() -> impl Responder {
    HttpResponse::Ok().json(json!({
        "status": "healthy",
        "service": "dxn-remote-ai"
    }))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    println!("Starting DXN Remote AI Server on http://127.0.0.1:8081");
    
    HttpServer::new(|| {
        App::new()
            .route("/api/functions/generate_text", web::post().to(generate_text))
            .route("/api/functions/analyze_sentiment", web::post().to(analyze_sentiment))
            .route("/api/health", web::get().to(health))
    })
    .bind("127.0.0.1:8081")?
    .run()
    .await
}

