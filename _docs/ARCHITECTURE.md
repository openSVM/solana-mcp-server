---
layout: docs
title: "Architecture Overview"
description: "System design, component interactions, and internal workings of the MCP server"
order: 2
category: architecture
---

# Solana MCP Server Architecture

## Overview

The Solana MCP Server is a Model Context Protocol (MCP) implementation that provides comprehensive access to Solana blockchain data through AI assistants like Claude. It acts as a bridge between natural language conversations and Solana RPC endpoints, enabling users to query blockchain information conversationally.

## System Architecture

```mermaid
graph TB
    subgraph "Client Layer"
        A[Claude Desktop/IDE] --> B[MCP Client]
    end
    
    subgraph "MCP Server"
        B --> C[Transport Layer]
        C --> D[Request Handler]
        D --> E[Tool Dispatcher]
        E --> F[RPC Manager]
        F --> G[Multi-Network Router]
    end
    
    subgraph "Configuration"
        H[Config.json] --> F
        I[Environment Variables] --> F
        J[SVM Networks Registry] --> G
    end
    
    subgraph "Blockchain Networks"
        G --> K[Solana Mainnet]
        G --> L[Solana Devnet]
        G --> M[Other SVM Networks]
        G --> N[Custom RPC Endpoints]
    end
    
    style A fill:#e1f5fe
    style B fill:#f3e5f5
    style C fill:#fff3e0
    style D fill:#fff3e0
    style E fill:#fff3e0
    style F fill:#fff3e0
    style G fill:#fff3e0
    style H fill:#f1f8e9
    style I fill:#f1f8e9
    style J fill:#f1f8e9
    style K fill:#ffebee
    style L fill:#ffebee
    style M fill:#ffebee
    style N fill:#ffebee
```

## Core Components

### 1. Transport Layer
- **Protocol**: JSON-RPC 2.0 over stdio
- **Communication**: Bidirectional message passing
- **Serialization**: JSON with serde
- **Error Handling**: Structured error responses with protocol version

### 2. Request Handler
- **Initialization**: MCP protocol handshake
- **Tool Discovery**: Dynamic tool listing
- **Request Routing**: Method-based dispatch
- **Response Formatting**: Standardized JSON-RPC responses

### 3. Multi-Network Router
- **Network Management**: Dynamic SVM network configuration
- **Load Balancing**: Parallel requests across enabled networks
- **Result Aggregation**: Unified response format for multi-network queries
- **Fallback Handling**: Graceful degradation on network failures

### 4. RPC Manager
- **Client Pool**: Persistent Solana RPC client connections
- **Commitment Levels**: Configurable transaction confirmation levels
- **Rate Limiting**: Built-in request throttling
- **Error Recovery**: Automatic retry mechanisms

## Request Flow

```mermaid
sequenceDiagram
    participant C as Claude
    participant M as MCP Server
    participant S as Solana RPC
    participant N as Other SVM Networks
    
    C->>M: Initialize MCP Connection
    M-->>C: Server Capabilities
    
    C->>M: tools/list Request
    M-->>C: Available Tools List
    
    C->>M: tools/call: getBalance
    M->>M: Parse Parameters
    M->>M: Check Enabled Networks
    
    alt Single Network Mode
        M->>S: getBalance RPC Call
        S-->>M: Account Balance
        M-->>C: Direct Response
    else Multi-Network Mode
        par Parallel Network Calls
            M->>S: getBalance RPC Call
            M->>N: getBalance RPC Call
        end
        S-->>M: Mainnet Balance
        N-->>M: Network Balance
        M->>M: Aggregate Results
        M-->>C: Multi-Network Response
    end
```

## Data Flow Architecture

```mermaid
flowchart LR
    subgraph "Input Processing"
        A[Natural Language Query] --> B[MCP Protocol]
        B --> C[Tool Identification]
        C --> D[Parameter Extraction]
    end
    
    subgraph "Network Layer"
        D --> E{Network Mode}
        E -->|Single| F[Primary RPC Client]
        E -->|Multi| G[Network Router]
        G --> H[Solana Mainnet]
        G --> I[Solana Devnet]
        G --> J[Custom Networks]
    end
    
    subgraph "Response Processing"
        F --> K[Format Response]
        H --> L[Aggregate Results]
        I --> L
        J --> L
        L --> K
        K --> M[JSON-RPC Response]
    end
    
    subgraph "Output"
        M --> N[MCP Protocol]
        N --> O[Natural Language Response]
    end
```

## Multi-Network Support

The server supports querying multiple SVM-compatible networks simultaneously:

### Network Configuration
```json
{
  "svm_networks": {
    "solana-mainnet": {
      "name": "Solana Mainnet",
      "rpc_url": "https://api.mainnet-beta.solana.com",
      "enabled": true
    },
    "solana-devnet": {
      "name": "Solana Devnet", 
      "rpc_url": "https://api.devnet.solana.com",
      "enabled": false
    }
  }
}
```

### Response Formats

**Single Network Response:**
```json
{
  "value": 1000000000,
  "context": {
    "slot": 12345
  }
}
```

**Multi-Network Response:**
```json
{
  "solana-mainnet": {
    "value": 1000000000,
    "context": { "slot": 12345 }
  },
  "custom-network": {
    "value": 500000000,
    "context": { "slot": 12340 }
  }
}
```

## Security Model

```mermaid
graph TD
    A[External Request] --> B[Input Validation]
    B --> C[Parameter Sanitization]
    C --> D[Rate Limiting]
    D --> E[Network Authorization]
    E --> F[RPC Client Security]
    F --> G[Response Filtering]
    G --> H[Output Sanitization]
    
    style B fill:#ffcdd2
    style C fill:#ffcdd2
    style D fill:#fff3e0
    style E fill:#fff3e0
    style F fill:#e8f5e8
    style G fill:#e8f5e8
    style H fill:#ffcdd2
```

### Security Features
- **Input Validation**: Strict parameter type checking
- **Rate Limiting**: Prevents RPC endpoint abuse
- **Network Isolation**: Sandboxed network configurations
- **Error Sanitization**: Prevents information leakage
- **Read-Only Operations**: No transaction signing or sending by default

## Performance Characteristics

### Latency Profile
- **Local Processing**: < 1ms
- **Single Network RPC**: 100-500ms (network dependent)
- **Multi-Network Queries**: 200-800ms (parallel execution)
- **Network Discovery**: 1-3s (cached after first load)

### Scalability
- **Concurrent Requests**: Limited by RPC endpoint rate limits
- **Memory Usage**: ~10MB base + ~2MB per active network
- **CPU Usage**: Minimal for request routing and JSON processing

### Optimization Features
- **Connection Pooling**: Persistent RPC client connections
- **Response Caching**: Configurable caching for static data
- **Parallel Execution**: Simultaneous multi-network queries
- **Lazy Loading**: Networks initialized on first use