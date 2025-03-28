# AtomSi DAO API Summary

This document provides a summary of the AtomSi DAO API components, including both REST and WebSocket APIs.

## API Components

### REST API

The REST API provides endpoints for interacting with all aspects of the DAO:

#### Authentication
- **POST /api/auth/login** - Authenticate with a wallet signature
- **POST /api/auth/logout** - End the current session
- **GET /api/auth/check-session** - Verify if the current session is valid

#### Governance
- **GET /api/governance/proposals** - List all proposals with pagination
- **GET /api/governance/proposals/:id** - Get a specific proposal by ID
- **POST /api/governance/proposals** - Create a new proposal
- **POST /api/governance/proposals/:id/vote** - Vote on a specific proposal

#### Treasury
- **GET /api/treasury/transactions** - List all treasury transactions with pagination
- **GET /api/treasury/transactions/:id** - Get a specific transaction by ID
- **POST /api/treasury/transactions** - Create a new treasury transaction
- **POST /api/treasury/transactions/:id/approve** - Approve a transaction
- **GET /api/treasury/balances** - Get treasury token balances

#### Identity
- **GET /api/identity/members** - List all members with pagination
- **GET /api/identity/members/:id** - Get a specific member by ID
- **GET /api/identity/activities** - List activities with pagination

#### Token
- **GET /api/token/tokens** - List all tokens
- **GET /api/token/tokens/:id** - Get a specific token by ID
- **GET /api/token/tokens/:id/balances** - Get token balances
- **POST /api/token/transfer** - Transfer tokens

#### WebSocket Info
- **GET /api/ws/info** - Get information about WebSocket connections

### WebSocket API

The WebSocket API provides real-time updates about DAO activities:

- **Endpoint**: `ws://localhost:3000/ws`

#### Connection Parameters
The WebSocket connection accepts the following query parameters:
- `token` - Optional authentication token for identifying the client
- `events` - Comma-separated list of events to subscribe to (e.g. `proposal_created,transaction_created`)

#### Event Types
The following event types are supported:

- **Governance Events**
  - `proposal_created` - When a new proposal is created
  - `proposal_updated` - When a proposal is updated
  - `proposal_voted` - When a vote is cast on a proposal

- **Treasury Events**
  - `transaction_created` - When a new treasury transaction is created
  - `transaction_approved` - When a transaction is approved
  - `transaction_executed` - When a transaction is executed

- **Identity Events**
  - `member_registered` - When a new member is registered
  - `member_updated` - When a member's details are updated

- **Activity Events**
  - `activity_recorded` - When a member activity is recorded

#### Event Format
Events are sent as JSON objects with the following structure:

```json
{
  "event_type": "proposal_created",
  "timestamp": "2023-07-24T12:34:56Z",
  "data": {
    // Event-specific data
  }
}
```

## API Models

### Common Models

- **ApiResponse<T>** - Wrapper for all API responses
  ```typescript
  {
    success: boolean;
    data: T | null;
    error: string | null;
    code: number;
  }
  ```

- **PaginationParams** - Parameters for paginated requests
  ```typescript
  {
    page: number;
    limit: number;
  }
  ```

- **PaginatedResponse<T>** - Response for paginated results
  ```typescript
  {
    items: T[];
    meta: {
      page: number;
      limit: number;
      total: number;
      total_pages: number;
    };
  }
  ```

### Authentication Models

- **LoginRequest**
  ```typescript
  {
    address: string;
    signature: string;
    message: string;
  }
  ```

- **LoginResponse**
  ```typescript
  {
    token: string;
    expires_at: string;
    member: MemberResponse;
  }
  ```

### Member Models

- **MemberResponse**
  ```typescript
  {
    id: string;
    address: string;
    name: string;
    role: string;
    status: string;
    reputation: number;
    joined_at: string;
    last_active_at: string;
    metadata: Record<string, any>;
  }
  ```

- **ActivityResponse**
  ```typescript
  {
    id: string;
    member_id: string;
    activity_type: string;
    related_object_id: string;
    timestamp: string;
    description: string;
    reputation_change: number;
    metadata: Record<string, any>;
  }
  ```

### Governance Models

- **ProposalResponse**
  ```typescript
  {
    id: string;
    title: string;
    description: string;
    proposer: string;
    status: string;
    created_at: string;
    expires_at: string;
    vote_counts: {
      yes: number;
      no: number;
      abstain: number;
    };
    metadata: Record<string, any>;
  }
  ```

- **VoteResponse**
  ```typescript
  {
    id: string;
    proposal_id: string;
    voter: string;
    vote: string;
    voting_power: number;
    timestamp: string;
    metadata: Record<string, any>;
  }
  ```

### Treasury Models

- **TransactionResponse**
  ```typescript
  {
    id: string;
    description: string;
    recipient: string;
    token_symbol: string;
    amount: string;
    status: string;
    required_approvals: number;
    current_approvals: number;
    created_at: string;
    executed_at: string | null;
    approvers: string[];
    metadata: Record<string, any>;
  }
  ```

- **TokenBalanceResponse**
  ```typescript
  {
    token_symbol: string;
    token_name: string;
    balance: string;
    decimals: number;
    token_address: string | null;
  }
  ```

## API Authentication

The API uses token-based authentication. To authenticate:

1. Call `/api/auth/login` with your wallet address and a signed message
2. Receive a token in the response
3. Include the token in subsequent requests using the `Authorization` header:
   ```
   Authorization: Bearer <token>
   ```

Protected endpoints will return a 401 Unauthorized status if the token is invalid or missing.

## Error Handling

All API endpoints return errors in a consistent format:

```json
{
  "success": false,
  "data": null,
  "error": "Error message describing what went wrong",
  "code": 400
}
```

Common error codes:
- `400` - Bad Request (invalid parameters)
- `401` - Unauthorized (authentication required)
- `403` - Forbidden (insufficient permissions)
- `404` - Not Found
- `500` - Internal Server Error

## Using the WebSocket Client

A sample HTML WebSocket client is included in `examples/websocket_client.html`. This client provides:

- Connection management
- Event filtering
- Real-time event display
- Event history

To use it:
1. Start the WebSocket example server: `cargo run --example websocket`
2. Open the HTML file in a browser
3. Connect to the server (default: `ws://localhost:3000/ws`)
4. Watch real-time events appear as they're generated 