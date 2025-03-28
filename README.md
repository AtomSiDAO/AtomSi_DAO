# AtomSi DAO

<div align="center">
  ![AtomSi DAO Logo](https://raw.githubusercontent.com/AtomSiDAO/AtomSi_DAO/main/atomsi-dao-logo-v12.jpg)
  <br><br>
  <a href="https://crates.io/crates/atomsidao"><img src="https://img.shields.io/crates/v/atomsidao.svg" alt="Crates.io"></a>
  <a href="https://docs.rs/atomsidao"><img src="https://docs.rs/atomsidao/badge.svg" alt="Documentation"></a>
  <a href="LICENSE"><img src="https://img.shields.io/badge/license-MIT-blue.svg" alt="License: MIT"></a>
  <img src="https://img.shields.io/badge/rust-1.60%2B-orange.svg" alt="Rust: 1.60+">
</div>

AtomSi DAO is a Rust-based framework for building decentralized autonomous organizations (DAOs), providing comprehensive tools and APIs for managing governance, treasury, identity, and tokens.

## Features

- **Governance**: Proposal creation, voting and execution
- **Treasury Management**: Multi-signature transactions, funds management
- **Identity Management**: Member registration, roles and reputation
- **Token Management**: Token allocation, transfers and balance queries
- **Security**: Authentication, authorization and cryptographic operations
- **Blockchain Integration**: Support for Ethereum and other blockchains
- **API**: RESTful API with OpenAPI documentation
- **WebSockets**: Real-time updates and notifications

## Quick Start

### Installation

```bash
cargo install atomsidao
```

### Using the CLI

```bash
# Initialize a new DAO
atomsidao init my-dao

# Start the API server
atomsidao serve
```

### Using as a Library

Add the dependency to your `Cargo.toml`:

```toml
[dependencies]
atomsidao = "0.1.0"
```

Basic code example:

```rust
use atomsidao::DAOContext;
use atomsidao::governance::ProposalManager;
use atomsidao::database::SqliteAdapter;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize database
    let db = SqliteAdapter::new("sqlite://dao.db").await?;
    
    // Create DAO context
    let context = DAOContext::new(config, db, blockchain);
    
    // Use the governance manager
    let proposal_mgr = ProposalManager::new(&context);
    let proposals = proposal_mgr.get_all_proposals().await?;
    
    println!("Current proposals: {:?}", proposals);
    Ok(())
}
```

## API Server

AtomSi DAO provides a complete RESTful API for interacting with the DAO.

### Starting the API Server

```rust
use atomsidao::api::{ApiServer, ApiConfig};
use atomsidao::DAOContext;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Create API configuration
    let api_config = ApiConfig {
        bind_address: "127.0.0.1:3000".parse()?,
        enable_cors: true,
        enable_logging: true,
        enable_docs: true,
        enable_websockets: true,
    };
    
    // Create and start API server
    let api_server = ApiServer::new(api_config, context);
    api_server.start().await?;
    
    Ok(())
}
```

### API Endpoints

The API is organized into the following main sections:

- **Authentication**: `/api/auth/*` - Login, logout and session management
- **Governance**: `/api/governance/*` - Proposal management and voting
- **Treasury**: `/api/treasury/*` - Transactions and balances
- **Identity**: `/api/identity/*` - Members and activities
- **Token**: `/api/token/*` - Token management and transfers

All API endpoints return a unified JSON response format:

```json
{
  "success": true,
  "data": { ... },
  "error": null,
  "code": 200
}
```

### API Documentation

API documentation is available at runtime:

- Swagger UI: `http://localhost:3000/docs/swagger-ui`
- OpenAPI specification: `http://localhost:3000/docs/openapi.json`

## WebSocket API

AtomSi DAO provides a WebSocket API for real-time updates about DAO activities.

### WebSocket Endpoint

The WebSocket endpoint is available at `ws://localhost:3000/ws`.

### Event Types

The WebSocket API emits events for various DAO activities:

- **Governance Events**: `proposal_created`, `proposal_updated`, `proposal_voted`
- **Treasury Events**: `transaction_created`, `transaction_approved`, `transaction_executed`
- **Identity Events**: `member_registered`, `member_updated`
- **Activity Events**: `activity_recorded`

### Example WebSocket Client

A sample WebSocket client is available in `examples/websocket_client.html`. To use it:

1. Start the WebSocket example server: `cargo run --example websocket`
2. Open the WebSocket client in a browser: `examples/websocket_client.html`
3. Connect to the server to see real-time events

### Broadcasting Events

You can broadcast WebSocket events from your code:

```rust
// Create and broadcast an event
api_server.broadcast_ws_event(
    EventType::ProposalCreated,
    serde_json::json!({
        "proposal_id": "prop_123",
        "title": "New Proposal",
        "description": "A description of the proposal",
        "proposer": "0x123456789abcdef",
    }),
)?;
```

## Database Support

AtomSi DAO supports the following databases:

- PostgreSQL
- SQLite

## Blockchain Support

Currently supported blockchains:

- Solana

## Development

### Environment Setup

```bash
# Clone the repository
git clone https://github.com/atomsidao/atomsidao.git
cd atomsidao

# Build the project
cargo build

# Run tests
cargo test
```

### Running Examples

```bash
# Run API server example
cargo run --example api_server

# Run WebSocket example
cargo run --example websocket
```

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details. 
