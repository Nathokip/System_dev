// Axum imports for extracting state and headers, defining routes, and returning JSON.
use axum::{
    extract::State,
    http::HeaderMap,
    routing::get,
    Json,
    Router,
};

// Serde enables JSON serialization for API response structures.
use serde::Serialize;
// Environment access, socket addressing, and shared state utilities.
use std::{env, net::SocketAddr, sync::Arc};
// Tokio provides the asynchronous TCP listener used by Axum.
use tokio::net::TcpListener;

// Shared application state containing the active backend instance port.
#[derive(Clone)]
pub struct Appstate {
    pub port : u16,
}

// JSON response returned by the instance information endpoint.
#[derive(Serialize)]

pub struct InstanceInfo {
    pub instance_port: u16,
    pub host: Option<String>,
    pub x_real_ip: Option<String>,
    pub x_forwarded_for: Option<String>,

}

// Load the server port from PORT, defaulting to 3001 when it is not set.
pub fn load_port() -> u16 {
    env::var("PORT")
        .unwrap_or_else(|_| "3001".to_string())
        .parse()
        .expect("PORT must be a valid u16")

}

// Construct the Axum router and register application endpoints.
pub fn build_router(state: Arc<Appstate>) -> Router {
    Router::new()
        // Return instance details and relevant request headers.
        .route("/api/info", get(info_handler))
        // Expose a simple health check endpoint.
        .route("/health", get(||async {"Ok"}))
        // Attach shared state so handlers can access the instance port.
        .with_state(state)
}

// Bind an asynchronous TCP listener on the local loopback interface.
pub async fn bind_listener(port: u16) -> TcpListener {
     // Listen only on localhost using the requested port.
     let addr = SocketAddr::from(([127,0,0,1],port));

     // Bind the socket and stop startup if the address cannot be used.
     let listener = TcpListener::bind(addr)
        .await
        .expect("Failed to bind ");

    // Report the address used by this backend instance.
    println!("Backend instance listening on http/{}", addr);

    listener
}


// Start Axum and serve the configured application over the TCP listener.
pub async fn serve_app(listener: TcpListener, app: Router){
    axum::serve(listener, app)
        .await
        .expect("HTTP server encountered a fatal error");
}

// Handle requests for instance information and proxy-related headers.
async fn info_handler(
    State(state): State<Arc<Appstate>>,
    headers: HeaderMap,
) -> Json<InstanceInfo> {
    // Serialize the instance port and available header values as JSON.
    Json(InstanceInfo { 
        instance_port: state.port,
        host: extract_header(&headers, "host"), 
        x_real_ip: extract_header(&headers, "x_real_ip"),
         x_forwarded_for: extract_header(&headers, "x_forwarded_for"), 
        })
}

// Safely retrieve and convert a named HTTP header to a UTF-8 string.
fn extract_header(headers:&HeaderMap, name: &str ) -> Option<String> {
    headers
        // Look up the header without creating an error for missing values.
        .get(name)
        // Convert the raw header value to a valid UTF-8 string.
        .and_then(|value| value.to_str() .ok())
        // Allocate an owned String for the returned response structure.
        .map(String::from)
}