-- AtomSi DAO SQLite Database Schema

-- Members table
CREATE TABLE IF NOT EXISTS members (
    id TEXT PRIMARY KEY,
    address TEXT NOT NULL,
    name TEXT NOT NULL,
    role TEXT NOT NULL, -- 'Member', 'Delegate', 'Council', 'Admin'
    status TEXT NOT NULL, -- 'Active', 'Inactive', 'Suspended'
    reputation INTEGER NOT NULL DEFAULT 0,
    joined_at INTEGER NOT NULL,
    last_active_at INTEGER NOT NULL,
    metadata TEXT,
    UNIQUE(address)
);

-- Member activities
CREATE TABLE IF NOT EXISTS member_activities (
    id TEXT PRIMARY KEY,
    member_id TEXT NOT NULL,
    activity_type TEXT NOT NULL, -- 'ProposalSubmission', 'Voting', 'Comment', 'Delegation', 'TreasuryTransaction', 'Other'
    related_id TEXT,
    timestamp INTEGER NOT NULL,
    description TEXT,
    reputation_change INTEGER NOT NULL DEFAULT 0,
    metadata TEXT,
    FOREIGN KEY(member_id) REFERENCES members(id) ON DELETE CASCADE
);

-- Proposals table
CREATE TABLE IF NOT EXISTS proposals (
    id TEXT PRIMARY KEY,
    title TEXT NOT NULL,
    description TEXT NOT NULL,
    proposer_id TEXT NOT NULL,
    proposal_type TEXT NOT NULL, -- 'Governance', 'Treasury', 'Membership', 'Other'
    status TEXT NOT NULL, -- 'Draft', 'Active', 'Passed', 'Failed', 'Executed', 'Cancelled'
    created_at INTEGER NOT NULL,
    voting_starts_at INTEGER,
    voting_ends_at INTEGER,
    executed_at INTEGER,
    execution_data TEXT,
    metadata TEXT,
    FOREIGN KEY(proposer_id) REFERENCES members(id)
);

-- Votes table
CREATE TABLE IF NOT EXISTS votes (
    id TEXT PRIMARY KEY,
    proposal_id TEXT NOT NULL,
    voter_id TEXT NOT NULL,
    vote_choice TEXT NOT NULL, -- 'For', 'Against', 'Abstain'
    vote_weight INTEGER NOT NULL,
    voted_at INTEGER NOT NULL,
    metadata TEXT,
    FOREIGN KEY(proposal_id) REFERENCES proposals(id) ON DELETE CASCADE,
    FOREIGN KEY(voter_id) REFERENCES members(id),
    UNIQUE(proposal_id, voter_id)
);

-- Treasury transactions
CREATE TABLE IF NOT EXISTS treasury_transactions (
    id TEXT PRIMARY KEY,
    description TEXT NOT NULL,
    recipient_address TEXT NOT NULL,
    token_symbol TEXT NOT NULL,
    amount INTEGER NOT NULL,
    status TEXT NOT NULL, -- 'Pending', 'Approved', 'Executed', 'Rejected', 'Failed'
    created_at INTEGER NOT NULL,
    executed_at INTEGER,
    required_approvals INTEGER NOT NULL,
    current_approvals INTEGER NOT NULL DEFAULT 0,
    related_proposal_id TEXT,
    metadata TEXT,
    FOREIGN KEY(related_proposal_id) REFERENCES proposals(id)
);

-- Treasury transaction approvals
CREATE TABLE IF NOT EXISTS treasury_transaction_approvals (
    id TEXT PRIMARY KEY,
    transaction_id TEXT NOT NULL,
    approver_id TEXT NOT NULL,
    approved_at INTEGER NOT NULL,
    FOREIGN KEY(transaction_id) REFERENCES treasury_transactions(id) ON DELETE CASCADE,
    FOREIGN KEY(approver_id) REFERENCES members(id),
    UNIQUE(transaction_id, approver_id)
);

-- Tokens table
CREATE TABLE IF NOT EXISTS tokens (
    id TEXT PRIMARY KEY,
    token_symbol TEXT NOT NULL,
    token_name TEXT NOT NULL,
    token_type TEXT NOT NULL, -- 'Governance', 'Reward', 'Access', 'Other'
    decimals INTEGER NOT NULL,
    total_supply TEXT,
    contract_address TEXT,
    chain_id INTEGER,
    created_at INTEGER NOT NULL,
    metadata TEXT,
    UNIQUE(token_symbol, chain_id)
);

-- Token balances
CREATE TABLE IF NOT EXISTS token_balances (
    id TEXT PRIMARY KEY,
    token_id TEXT NOT NULL,
    member_id TEXT NOT NULL,
    balance TEXT NOT NULL DEFAULT '0',
    last_updated INTEGER NOT NULL,
    FOREIGN KEY(token_id) REFERENCES tokens(id) ON DELETE CASCADE,
    FOREIGN KEY(member_id) REFERENCES members(id) ON DELETE CASCADE,
    UNIQUE(token_id, member_id)
);

-- Token transfers
CREATE TABLE IF NOT EXISTS token_transfers (
    id TEXT PRIMARY KEY,
    token_id TEXT NOT NULL,
    from_member_id TEXT,
    to_member_id TEXT,
    amount TEXT NOT NULL,
    timestamp INTEGER NOT NULL,
    transaction_hash TEXT,
    description TEXT,
    metadata TEXT,
    FOREIGN KEY(token_id) REFERENCES tokens(id) ON DELETE CASCADE,
    FOREIGN KEY(from_member_id) REFERENCES members(id),
    FOREIGN KEY(to_member_id) REFERENCES members(id)
);

-- Sessions table
CREATE TABLE IF NOT EXISTS sessions (
    id TEXT PRIMARY KEY,
    member_id TEXT NOT NULL,
    created_at INTEGER NOT NULL,
    expires_at INTEGER NOT NULL,
    last_active_at INTEGER NOT NULL,
    ip_address TEXT,
    user_agent TEXT,
    is_active INTEGER NOT NULL DEFAULT 1,
    FOREIGN KEY(member_id) REFERENCES members(id) ON DELETE CASCADE
);

-- Settings table
CREATE TABLE IF NOT EXISTS settings (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL,
    updated_at INTEGER NOT NULL
);

-- Blockchain transactions
CREATE TABLE IF NOT EXISTS blockchain_transactions (
    id TEXT PRIMARY KEY,
    transaction_hash TEXT NOT NULL,
    chain_id INTEGER NOT NULL,
    from_address TEXT NOT NULL,
    to_address TEXT,
    value TEXT NOT NULL DEFAULT '0',
    gas_used INTEGER,
    gas_price INTEGER,
    status TEXT NOT NULL, -- 'Pending', 'Confirmed', 'Failed'
    block_number INTEGER,
    timestamp INTEGER NOT NULL,
    related_id TEXT,
    related_type TEXT, -- 'Proposal', 'TreasuryTransaction', 'TokenTransfer', 'Other'
    metadata TEXT
);

-- Notifications
CREATE TABLE IF NOT EXISTS notifications (
    id TEXT PRIMARY KEY,
    member_id TEXT NOT NULL,
    notification_type TEXT NOT NULL,
    title TEXT NOT NULL,
    body TEXT,
    is_read INTEGER NOT NULL DEFAULT 0,
    created_at INTEGER NOT NULL,
    related_id TEXT,
    related_type TEXT,
    metadata TEXT,
    FOREIGN KEY(member_id) REFERENCES members(id) ON DELETE CASCADE
);

-- Delegations
CREATE TABLE IF NOT EXISTS delegations (
    id TEXT PRIMARY KEY,
    delegator_id TEXT NOT NULL,
    delegate_id TEXT NOT NULL,
    token_id TEXT,
    amount TEXT,
    starts_at INTEGER NOT NULL,
    ends_at INTEGER,
    is_active INTEGER NOT NULL DEFAULT 1,
    metadata TEXT,
    FOREIGN KEY(delegator_id) REFERENCES members(id),
    FOREIGN KEY(delegate_id) REFERENCES members(id),
    FOREIGN KEY(token_id) REFERENCES tokens(id),
    UNIQUE(delegator_id, delegate_id, token_id)
); 