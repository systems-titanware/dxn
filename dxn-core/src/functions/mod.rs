pub mod models;
pub mod manager;
pub mod executors;

// Re-export for convenience
pub use manager::{init, call_function, get_function, run};
pub use models::{SystemFunctions, SystemFunctionModel, FunctionType, FunctionError};