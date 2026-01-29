use super::*;
use actix_web::{body, http::StatusCode, test, App};
use std::sync::RwLock;
use uuid::Uuid;

use crate::data::models::{SystemData, SystemDataModel};
use crate::functions::models::{SystemFunctionModel, SystemFunctions, FunctionType};
use crate::integrations::models::SystemIntegrations;
use crate::system::models::{AppState, System};
use crate::system::server::models::{SystemServer, SystemServerRoute};

fn build_app_state(
    functions: Vec<SystemFunctionModel>,
    data_models: Vec<SystemDataModel>,
    server_routes: Vec<SystemServerRoute>,
) -> AppState {
    AppState {
        app_name: "test-app".to_string(),
        counter: RwLock::new(0),
        db_name: "test-db".to_string(),
        system: System {
            data: SystemData {
                public: Some(data_models),
                private: None,
            },
            server: SystemServer {
                public: Some(server_routes),
                private: None,
            },
            integrations: SystemIntegrations {
                public: None,
                private: None,
            },
            functions: SystemFunctions {
                public: Some(functions),
                private: None,
            },
            service_mesh: None,
        },
        uuid: Uuid::now_v7(),
    }
}

#[actix_web::test]
async fn test_get_functions_pagination() {
    // Prepare 3 functions
    let functions = vec![
        SystemFunctionModel {
            name: "func1".to_string(),
            function_type: FunctionType::Wasm,
            path: None,
            function_name: None,
            library_path: None,
            symbol_name: None,
            service_name: None,
            endpoint: None,
            script_path: None,
            script_language: None,
            version: 1,
            parameters: None,
            return_type: None,
            params: None,
        },
        SystemFunctionModel {
            name: "func2".to_string(),
            function_type: FunctionType::Wasm,
            path: None,
            function_name: None,
            library_path: None,
            symbol_name: None,
            service_name: None,
            endpoint: None,
            script_path: None,
            script_language: None,
            version: 1,
            parameters: None,
            return_type: None,
            params: None,
        },
        SystemFunctionModel {
            name: "func3".to_string(),
            function_type: FunctionType::Wasm,
            path: None,
            function_name: None,
            library_path: None,
            symbol_name: None,
            service_name: None,
            endpoint: None,
            script_path: None,
            script_language: None,
            version: 1,
            parameters: None,
            return_type: None,
            params: None,
        },
    ];

    let data_models: Vec<SystemDataModel> = Vec::new();
    let server_routes: Vec<SystemServerRoute> = Vec::new();

    let app_state = build_app_state(functions, data_models, server_routes);
    let data = web::Data::new(app_state);

    let app = test::init_service(
        App::new()
            .app_data(data.clone())
            .route("/api/config/functions", web::get().to(get_functions)),
    )
    .await;

    // Request page 2 with page_size 1
    let req = test::TestRequest::get()
        .uri("/api/config/functions?page=2&page_size=1")
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);

    let body = body::to_bytes(resp.into_body()).await.unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

    // Expect 1 item on page 2
    assert_eq!(json["data"].as_array().unwrap().len(), 1);
    assert_eq!(json["pagination"]["page"].as_i64().unwrap(), 2);
    assert_eq!(json["pagination"]["page_size"].as_i64().unwrap(), 1);
    assert_eq!(json["pagination"]["total"].as_i64().unwrap(), 3);
    assert_eq!(json["pagination"]["total_pages"].as_i64().unwrap(), 3);
}

#[actix_web::test]
async fn test_get_data_models_pagination() {
    use crate::data::models::SystemDataModelField;

    let data_models = vec![
        SystemDataModel {
            name: "profile".to_string(),
            version: 1,
            fields: vec![SystemDataModelField {
                name: "email".to_string(),
                datatype: "text".to_string(),
                value: "{vault.profile.email}".to_string(),
                primary: None,
                secondary: None,
            }],
        },
        SystemDataModel {
            name: "wallet".to_string(),
            version: 1,
            fields: vec![SystemDataModelField {
                name: "address".to_string(),
                datatype: "text".to_string(),
                value: "{vault.profile.address}".to_string(),
                primary: None,
                secondary: None,
            }],
        },
    ];

    let functions: Vec<SystemFunctionModel> = Vec::new();
    let server_routes: Vec<SystemServerRoute> = Vec::new();

    let app_state = build_app_state(functions, data_models, server_routes);
    let data = web::Data::new(app_state);

    let app = test::init_service(
        App::new()
            .app_data(data.clone())
            .route("/api/config/data", web::get().to(get_data)),
    )
    .await;

    let req = test::TestRequest::get()
        .uri("/api/config/data?page=1&page_size=1")
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);

    let body = body::to_bytes(resp.into_body()).await.unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

    // Expect 1 item on page 1 out of 2 total
    assert_eq!(json["data"].as_array().unwrap().len(), 1);
    assert_eq!(json["pagination"]["page"].as_i64().unwrap(), 1);
    assert_eq!(json["pagination"]["page_size"].as_i64().unwrap(), 1);
    assert_eq!(json["pagination"]["total"].as_i64().unwrap(), 2);
    assert_eq!(json["pagination"]["total_pages"].as_i64().unwrap(), 2);
}

#[actix_web::test]
async fn test_get_server_routes_pagination_empty() {
    let functions: Vec<SystemFunctionModel> = Vec::new();
    let data_models: Vec<SystemDataModel> = Vec::new();
    let server_routes: Vec<SystemServerRoute> = Vec::new();

    let app_state = build_app_state(functions, data_models, server_routes);
    let data = web::Data::new(app_state);

    let app = test::init_service(
        App::new()
            .app_data(data.clone())
            .route("/api/config/server", web::get().to(get_server_routes)),
    )
    .await;

    let req = test::TestRequest::get()
        .uri("/api/config/server?page=1&page_size=10")
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);

    let body = body::to_bytes(resp.into_body()).await.unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

    // Expect empty list and zero totals
    assert_eq!(json["data"].as_array().unwrap().len(), 0);
    assert_eq!(json["pagination"]["total"].as_i64().unwrap(), 0);
    assert_eq!(json["pagination"]["total_pages"].as_i64().unwrap(), 0);
}

