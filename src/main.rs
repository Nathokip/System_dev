mod server;

// Application dependencies and server helpers.
use std::sync::Arc;
use server::{bind_listener, build_router, load_port, serve_app, Appstate};

// Start the asynchronous Tokio runtime and launch the web server.
#[tokio::main]
async fn main() {
    // Read the configured listening port from the environment.
    let port = load_port();
    // Share immutable application state between request handlers.
    let state = Arc::new(Appstate {port});
    // Build the Axum routes and attach the shared state.
    let app = build_router(state);
    // Bind the TCP listener before starting the HTTP server.
    let listener = bind_listener(port).await;

    // Run the server until it stops or encounters a fatal error.
    serve_app(listener, app).await;

}
