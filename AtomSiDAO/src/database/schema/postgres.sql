-- AtomSi DAO PostgreSQL Database Schema

-- Create extension for UUID support
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

-- Members table
CREATE TABLE IF NOT EXISTS members (
    id VARCHAR(100) PRIMARY KEY,
    address VARCHAR(42) NOT NULL,
    name VARCHAR(100) NOT NULL,
    role VARCHAR(20) NOT NULL, -- 'Member', 'Delegate', 'Council', 'Admin'
    status VARCHAR(20) NOT NULL, -- 'Active', 'Inactive', 'Suspended'
    reputation INTEGER NOT NULL DEFAULT 0,
    joined_at BIGINT NOT NULL,
    last_active_at BIGINT NOT NULL,
    metadata JSONB,
    UNIQUE(address)
);

-- Member activities
CREATE TABLE IF NOT EXISTS member_activities (
    id VARCHAR(100) PRIMARY KEY,
    member_id VARCHAR(100) NOT NULL REFERENCES members(id),
    activity_type VARCHAR(50) NOT NULL, -- 'ProposalSubmission', 'Voting', 'Comment', 'Delegation', 'TreasuryTransaction', 'Other'
    related_id VARCHAR(100),
    timestamp BIGINT NOT NULL,
    description TEXT,
    reputation_change INTEGER NOT NULL DEFAULT 0,
    metadata JSONB,
    CONSTRAINT fk_member
        FOREIGN KEY(member_id)
        REFERENCES members(id)
        ON DELETE CASCADE
);

-- Proposals table
CREATE TABLE IF NOT EXISTS proposals (
    id VARCHAR(100) PRIMARY KEY,
    title VARCHAR(200) NOT NULL,
    description TEXT NOT NULL,
    proposer_id VARCHAR(100) NOT NULL REFERENCES members(id),
    proposal_type VARCHAR(50) NOT NULL, -- 'Governance', 'Treasury', 'Membership', 'Other'
    status VARCHAR(20) NOT NULL, -- 'Draft', 'Active', 'Passed', 'Failed', 'Executed', 'Cancelled'
    created_at BIGINT NOT NULL,
    voting_starts_at BIGINT,
    voting_ends_at BIGINT,
    executed_at BIGINT,
    execution_data JSONB,
    metadata JSONB,
    CONSTRAINT fk_proposer
        FOREIGN KEY(proposer_id)
        REFERENCES members(id)
);

-- Votes table
CREATE TABLE IF NOT EXISTS votes (
    id VARCHAR(100) PRIMARY KEY,
    proposal_id VARCHAR(100) NOT NULL REFERENCES proposals(id),
    voter_id VARCHAR(100) NOT NULL REFERENCES members(id),
    vote_choice VARCHAR(20) NOT NULL, -- 'For', 'Against', 'Abstain'
    vote_weight BIGINT NOT NULL,
    voted_at BIGINT NOT NULL,
    metadata JSONB,
    CONSTRAINT fk_proposal
        FOREIGN KEY(proposal_id)
        REFERENCES proposals(id)
        ON DELETE CASCADE,
    CONSTRAINT fk_voter
        FOREIGN KEY(voter_id)
        REFERENCES members(id),
    UNIQUE(proposal_id, voter_id)
);

-- Treasury transactions
CREATE TABLE IF NOT EXISTS treasury_transactions (
    id VARCHAR(100) PRIMARY KEY,
    description TEXT NOT NULL,
    recipient_address VARCHAR(42) NOT NULL,
    token_symbol VARCHAR(10) NOT NULL,
    amount BIGINT NOT NULL,
    status VARCHAR(20) NOT NULL, -- 'Pending', 'Approved', 'Executed', 'Rejected', 'Failed'
    created_at BIGINT NOT NULL,
    executed_at BIGINT,
    required_approvals INTEGER NOT NULL,
    current_approvals INTEGER NOT NULL DEFAULT 0,
    related_proposal_id VARCHAR(100),
    metadata JSONB,
    CONSTRAINT fk_related_proposal
        FOREIGN KEY(related_proposal_id)
        REFERENCES proposals(id)
);

-- Treasury transaction approvals
CREATE TABLE IF NOT EXISTS treasury_transaction_approvals (
    id VARCHAR(100) PRIMARY KEY,
    transaction_id VARCHAR(100) NOT NULL REFERENCES treasury_transactions(id),
    approver_id VARCHAR(100) NOT NULL REFERENCES members(id),
    approved_at BIGINT NOT NULL,
    CONSTRAINT fk_transaction
        FOREIGN KEY(transaction_id)
        REFERENCES treasury_transactions(id)
        ON DELETE CASCADE,
    CONSTRAINT fk_approver
        FOREIGN KEY(approver_id)
        REFERENCES members(id),
    UNIQUE(transaction_id, approver_id)
);

-- Tokens table
CREATE TABLE IF NOT EXISTS tokens (
    id VARCHAR(100) PRIMARY KEY,
    token_symbol VARCHAR(10) NOT NULL,
    token_name VARCHAR(100) NOT NULL,
    token_type VARCHAR(20) NOT NULL, -- 'Governance', 'Reward', 'Access', 'Other'
    decimals INTEGER NOT NULL,
    total_supply NUMERIC(78, 0),
    contract_address VARCHAR(42),
    chain_id BIGINT,
    created_at BIGINT NOT NULL,
    metadata JSONB,
    UNIQUE(token_symbol, chain_id)
);

-- Token balances
CREATE TABLE IF NOT EXISTS token_balances (
    id VARCHAR(100) PRIMARY KEY,
    token_id VARCHAR(100) NOT NULL REFERENCES tokens(id),
    member_id VARCHAR(100) NOT NULL REFERENCES members(id),
    balance NUMERIC(78, 0) NOT NULL DEFAULT 0,
    last_updated BIGINT NOT NULL,
    CONSTRAINT fk_token
        FOREIGN KEY(token_id)
        REFERENCES tokens(id)
        ON DELETE CASCADE,
    CONSTRAINT fk_balance_member
        FOREIGN KEY(member_id)
        REFERENCES members(id)
        ON DELETE CASCADE,
    UNIQUE(token_id, member_id)
);

-- Token transfers
CREATE TABLE IF NOT EXISTS token_transfers (
    id VARCHAR(100) PRIMARY KEY,
    token_id VARCHAR(100) NOT NULL REFERENCES tokens(id),
    from_member_id VARCHAR(100) REFERENCES members(id),
    to_member_id VARCHAR(100) REFERENCES members(id),
    amount NUMERIC(78, 0) NOT NULL,
    timestamp BIGINT NOT NULL,
    transaction_hash VARCHAR(66),
    description TEXT,
    metadata JSONB,
    CONSTRAINT fk_transfer_token
        FOREIGN KEY(token_id)
        REFERENCES tokens(id)
        ON DELETE CASCADE,
    CONSTRAINT fk_from_member
        FOREIGN KEY(from_member_id)
        REFERENCES members(id),
    CONSTRAINT fk_to_member
        FOREIGN KEY(to_member_id)
        REFERENCES members(id)
);

-- Sessions table
CREATE TABLE IF NOT EXISTS sessions (
    id VARCHAR(100) PRIMARY KEY,
    member_id VARCHAR(100) NOT NULL REFERENCES members(id),
    created_at BIGINT NOT NULL,
    expires_at BIGINT NOT NULL,
    last_active_at BIGINT NOT NULL,
    ip_address VARCHAR(45),
    user_agent TEXT,
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    CONSTRAINT fk_session_member
        FOREIGN KEY(member_id)
        REFERENCES members(id)
        ON DELETE CASCADE
);

-- Settings table
CREATE TABLE IF NOT EXISTS settings (
    key VARCHAR(100) PRIMARY KEY,
    value JSONB NOT NULL,
    updated_at BIGINT NOT NULL
);

-- Blockchain transactions
CREATE TABLE IF NOT EXISTS blockchain_transactions (
    id VARCHAR(100) PRIMARY KEY,
    transaction_hash VARCHAR(66) NOT NULL,
    chain_id BIGINT NOT NULL,
    from_address VARCHAR(42) NOT NULL,
    to_address VARCHAR(42),
    value NUMERIC(78, 0) NOT NULL DEFAULT 0,
    gas_used BIGINT,
    gas_price BIGINT,
    status VARCHAR(20) NOT NULL, -- 'Pending', 'Confirmed', 'Failed'
    block_number BIGINT,
    timestamp BIGINT NOT NULL,
    related_id VARCHAR(100),
    related_type VARCHAR(50), -- 'Proposal', 'TreasuryTransaction', 'TokenTransfer', 'Other'
    metadata JSONB
);

-- Notifications
CREATE TABLE IF NOT EXISTS notifications (
    id VARCHAR(100) PRIMARY KEY,
    member_id VARCHAR(100) NOT NULL REFERENCES members(id),
    notification_type VARCHAR(50) NOT NULL,
    title VARCHAR(200) NOT NULL,
    body TEXT,
    is_read BOOLEAN NOT NULL DEFAULT FALSE,
    created_at BIGINT NOT NULL,
    related_id VARCHAR(100),
    related_type VARCHAR(50),
    metadata JSONB,
    CONSTRAINT fk_notification_member
        FOREIGN KEY(member_id)
        REFERENCES members(id)
        ON DELETE CASCADE
);

-- Delegations
CREATE TABLE IF NOT EXISTS delegations (
    id VARCHAR(100) PRIMARY KEY,
    delegator_id VARCHAR(100) NOT NULL REFERENCES members(id),
    delegate_id VARCHAR(100) NOT NULL REFERENCES members(id),
    token_id VARCHAR(100) REFERENCES tokens(id),
    amount NUMERIC(78, 0),
    starts_at BIGINT NOT NULL,
    ends_at BIGINT,
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    metadata JSONB,
    CONSTRAINT fk_delegator
        FOREIGN KEY(delegator_id)
        REFERENCES members(id),
    CONSTRAINT fk_delegate
        FOREIGN KEY(delegate_id)
        REFERENCES members(id),
    CONSTRAINT fk_delegation_token
        FOREIGN KEY(token_id)
        REFERENCES tokens(id),
    UNIQUE(delegator_id, delegate_id, token_id)
); 