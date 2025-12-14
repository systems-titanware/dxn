// tcp_server_app/src/main.rs
use dxn_shared::{RequestMessage, ResponseMessage, TcpServerContract};
use tokio::net::TcpListener;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use serde_json;

struct MyServerImpl;

// The server implements the shared trait with its specific logic
impl TcpServerContract for MyServerImpl {
    fn handle_request(&self, req: RequestMessage) -> ResponseMessage {
        println!("Server received request ID: {}", req.id);
        ResponseMessage {
            success: true,
            message: format!("Processed request: {}", req.payload),
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Server starting on 0.0.0.0:8080");
    let listener = TcpListener::bind("0.0.0.0:8080").await?;
    println!("Server running on 0.0.0.0:8080");

    loop {
        let (mut socket, _) = listener.accept().await?;
        let server_impl = MyServerImpl;

        tokio::spawn(async move {
            let mut buf = vec![0; 1024];
            loop {
                // Read the message
                let n = match socket.read(&mut buf).await {
                    Ok(n) if n == 0 => return, // Connection closed
                    Ok(n) => n,
                    Err(e) => {
                        eprintln!("failed to read from socket; err = {:?}", e);
                        return;
                    }
                };

                // Deserialize, handle the request using the trait method, and serialize response
                if let Ok(request) = serde_json::from_slice::<RequestMessage>(&buf[..n]) {
                    let response = server_impl.handle_request(request);
                    let response_json = serde_json::to_vec(&response).expect("Failed to serialize response");

                    // Write the response back
                    if let Err(e) = socket.write_all(&response_json).await {
                        eprintln!("failed to write to socket; err = {:?}", e);
                        return;
                    }
                }
            }
        });
    }
}
