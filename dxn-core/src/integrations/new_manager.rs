use crate::integrations::models::{SystemIntegrationModel, IntegrationError};

// tcp_client_app/src/main.rs
//use dxn_shared::{RequestMessage, ResponseMessage};
use tokio::net::TcpStream;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use serde_json;

pub fn load_public_integrations(integrations: Vec<SystemIntegrationModel>) {
    unsafe {
        PUBLIC_INTEGRATIONS = integrations;
    }
}