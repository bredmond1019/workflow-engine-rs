Let's say I have a MCP server running for helpscout, notion, and slack just like the below in pythong

```python
# helpscout-server/server.py
from mcp.server.fastmcp import FastMCP
from mcp.server.http import HTTPServer

# Initialize FastMCP server
mcp = FastMCP("helpscout-server")

# ... your existing tool definitions ...

if __name__ == "__main__":
    # Create HTTP server
    http_server = HTTPServer(mcp)
    # Run on specific port
    http_server.run(host="0.0.0.0", port=8001)  # Helpscout server
```


Our goal is: we'll want to create a connection to these external mcp servers (and others in the future) so that we can add more nodes to our system to expand the workflows we may want to create. 

Let's create a NotionClientNode, HelpscoutClientNode, and SlackClientNode with the ability to connect to externally running mcp servers. Please use the example implementation below as a guide, but make sure to keep true to the architecture we have now. 

The Nodes should be able to connect either through HTTP and some given URL, or through websocket, or stdio. Please reference @src/core/mcp/transport and @src/core/mcp/client for some already implemented logic.

2. **Rust Web Server Implementation**
In your Rust application, you can create a reverse proxy to forward requests to the MCP servers:

```rust
use actix_web::{web, App, HttpServer, Responder, middleware};
use actix_web::http::header;
use reqwest::Client;
use serde_json::Value;

// Configuration for MCP servers
struct MCPServerConfig {
    helpscout_url: String,
    notion_url: String,
    slack_url: String,
}

// Client state
struct AppState {
    client: Client,
    config: MCPServerConfig,
}

// Proxy handler for Helpscout server
async fn helpscout_proxy(
    data: web::Data<AppState>,
    path: web::Path<String>,
    body: web::Json<Value>,
) -> impl Responder {
    let client = &data.client;
    let config = &data.config;
    
    // Forward the request to the Helpscout MCP server
    let response = client
        .post(format!("{}/{}", config.helpscout_url, path.into_inner()))
        .json(&body.into_inner())
        .send()
        .await
        .unwrap()
        .json::<Value>()
        .await
        .unwrap();

    web::Json(response)
}

// Similar handlers for Notion and Slack...

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Create HTTP client
    let client = Client::new();
    
    // Configure MCP server URLs
    let config = MCPServerConfig {
        helpscout_url: "http://localhost:8001".to_string(),
        notion_url: "http://localhost:8002".to_string(),
        slack_url: "http://localhost:8003".to_string(),
    };

    // Start web server
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(AppState {
                client: client.clone(),
                config: config.clone(),
            }))
            .wrap(middleware::Logger::default())
            // Helpscout endpoints
            .service(
                web::scope("/api/helpscout")
                    .route("/search_articles", web::post().to(helpscout_proxy))
                    .route("/get_article", web::post().to(helpscout_proxy))
                    .route("/list_articles", web::post().to(helpscout_proxy))
                    .route("/list_collections", web::post().to(helpscout_proxy))
                    .route("/get_collection", web::post().to(helpscout_proxy))
            )
            // Add similar scopes for Notion and Slack...
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
```

3. **Start the MCP Servers**
Start each MCP server on a different port:

```bash
# Terminal 1
cd helpscout-server
python server.py  # Runs on port 8001

# Terminal 2
cd notion-server
python server.py  # Runs on port 8002

# Terminal 3
cd slack-server
python server.py  # Runs on port 8003
```

4. **Example Usage**
Now you can call the MCP servers through your Rust web server:

```rust
// Example client code
async fn search_articles() {
    let client = reqwest::Client::new();
    
    let response = client
        .post("http://localhost:8080/api/helpscout/search_articles")
        .json(&json!({
            "keywords": "appointment",
            "page": 1,
            "per_page": 10
        }))
        .send()
        .await
        .unwrap()
        .json::<Value>()
        .await
        .unwrap();

    println!("Response: {:?}", response);
}
```

5. **Error Handling and Retries**
Add robust error handling and retry logic:

```rust
use tokio::time::{sleep, Duration};

async fn call_mcp_server_with_retry(
    client: &Client,
    url: &str,
    body: Value,
    max_retries: u32,
) -> Result<Value, reqwest::Error> {
    let mut attempts = 0;
    loop {
        match client.post(url).json(&body).send().await {
            Ok(response) => {
                return response.json::<Value>().await;
            }
            Err(e) => {
                attempts += 1;
                if attempts >= max_retries {
                    return Err(e);
                }
                sleep(Duration::from_secs(1)).await;
            }
        }
    }
}
```

6. **Configuration Management**
Use environment variables for configuration:

```rust
use std::env;

struct MCPServerConfig {
    helpscout_url: String,
    notion_url: String,
    slack_url: String,
}

impl MCPServerConfig {
    fn from_env() -> Self {
        Self {
            helpscout_url: env::var("HELPSCOUT_MCP_URL")
                .unwrap_or_else(|_| "http://localhost:8001".to_string()),
            notion_url: env::var("NOTION_MCP_URL")
                .unwrap_or_else(|_| "http://localhost:8002".to_string()),
            slack_url: env::var("SLACK_MCP_URL")
                .unwrap_or_else(|_| "http://localhost:8003".to_string()),
        }
    }
}
```

This implementation:
1. Exposes each MCP server on a different port
2. Creates a reverse proxy in your Rust application
3. Maintains the standard MCP response format
4. Provides proper error handling and retries
5. Uses environment variables for configuration
6. Keeps the MCP servers isolated and independently scalable

The MCP servers will handle the actual API calls (or dummy data in development mode) and return responses in the standardized format that your Rust application can easily process.
