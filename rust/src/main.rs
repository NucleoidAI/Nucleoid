use std::sync::{Arc, Mutex};

use nucleoid::{http, runtime::NucleoidRuntime};

#[tokio::main]
async fn main() {
    println!("Nucleoid runtime is started");
    println!("Inspired by Nature\n");

    let runtime = NucleoidRuntime::new().expect("failed to start JS engine");
    let state: http::AppState = Arc::new(Mutex::new(runtime));

    let app = http::router(state);

    // Matches upstream's default terminal port (`src/config.js`).
    let port = 8448;
    let listener = tokio::net::TcpListener::bind(("127.0.0.1", port))
        .await
        .expect("failed to bind terminal port");

    println!("[OK] Terminal is ready on http://127.0.0.1:{port}");

    axum::serve(listener, app)
        .await
        .expect("terminal server crashed");
}
