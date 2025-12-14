 
// Define the messages that will be sent over TCP.
// We derive Serialize/Deserialize from the 'serde' crate for easy data transfer.
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct RequestMessage {
    pub id: u32,
    pub payload: String,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct ResponseMessage {
    pub success: bool,
    pub message: String,
}

// This trait defines the behavior contract.
// The server implements this, and the client "calls" these conceptual methods
// by sending the appropriate TCP messages defined above.
pub trait TcpServerContract {
    fn handle_request(&self, req: RequestMessage) -> ResponseMessage;
}

pub fn add(left: u64, right: u64) -> u64 {
    left + right
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
