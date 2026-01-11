## setup
1. Install llvm `brew install llvm`
2. Download rustup `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`
2.1. If rust was installed with brew, uninstall it `brew uninstall rust`
3. run `. "$HOME/.cargo/env"`
3. run `rustup target add wasm32-unknown-unknown`

## commands 
5. `cd ../dxn_public/dxn_functions/
    cargo build --target wasm32-unknown-unknown --release
    cd ../../dxn-core`
 
