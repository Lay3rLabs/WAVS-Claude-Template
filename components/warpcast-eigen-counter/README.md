# Warpcast EigenLayer Mention Counter Component

## Overview

This component takes a Warpcast username as input, retrieves the user's data including wallet address and casts (posts), counts how many times they've mentioned "EigenLayer" in their casts, and returns that data.

## Test it locally

After following the setup instructions in the [setup section of the readme](../../README.md#setup), run these commands:

```bash
# build the component
make wasi-build
```

```bash
# Make sure Docker is running
# Test the component
  export TRIGGER_DATA_INPUT=`cast abi-encode "f(string)" "dabit3"`
  export COMPONENT_FILENAME=warpcast_eigen_counter.wasm
  export SERVICE_CONFIG="'{\"fuel_limit\":100000000,\"max_gas\":5000000,\"host_envs\":[\"WAVS_ENV_API_KEY\"],\"kv\":[],\"workflow_id\":\"default\",\"component_id\":\"default\"}'"
  make wasi-exec
```

## API Endpoints
- User info: `https://hoyt.farcaster.xyz:2281/v1/userNameProofByName?name={username}`
- User casts: `https://hoyt.farcaster.xyz:2281/v1/castsByFid?fid={fid}`

## Data Flow
1. Take a string input of Warpcast username
2. Call the first API endpoint to get user info and FID (Farcaster ID)
3. Use the FID to call the second API endpoint to get the user's casts
4. Count occurrences of "EigenLayer" (case-insensitive) in cast text
5. Return the count and user's wallet address

## Required Imports
```rust
use alloy_primitives::Address;
use alloy_sol_types::{sol, SolCall, SolValue};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use wavs_wasi_chain::decode_event_log_data;
use wavs_wasi_chain::http::{fetch_json, http_request_get};
use wstd::{http::HeaderValue, runtime::block_on};
```

## API Response Structures
Need to define proper response structures for:
- User name proof API response
- Casts API response

## Solidity Types
```rust
// Input function signature
sol! {
    function countEigenLayerMentions(string username) external;
}

// Define solidity module
mod solidity {
    use alloy_sol_macro::sol;
    pub use ITypes::*;

    sol!("../../src/interfaces/ITypes.sol");
}
```

## Response Structure
```rust
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct EigenCountResult {
    username: String,
    wallet_address: String,
    eigen_mentions: u64,
    total_casts: u64,
    timestamp: String,
}
```

## Validation Checklist

### Common errors:
- [x] ✅ ALWAYS use `{ workspace = true }` in your component Cargo.toml
- [x] ✅ ALWAYS verify API response structures by using curl on the endpoints
- [x] ✅ ALWAYS Read any documentation given to you in a prompt
- [x] ✅ ALWAYS implement the Guest trait and export your component
- [x] ✅ ALWAYS use `export!(Component with_types_in bindings)`
- [x] ✅ ALWAYS use `clone()` before consuming data to avoid ownership issues
- [x] ✅ ALWAYS derive `Clone` for API response data structures
- [x] ✅ ALWAYS decode ABI data properly, never with `String::from_utf8`
- [x] ✅ ALWAYS use `ok_or_else()` for Option types, `map_err()` for Result types
- [x] ✅ ALWAYS use string parameters for CLI testing
- [x] ✅ ALWAYS use `.to_string()` to convert string literals (&str) to String types
- [x] ✅ NEVER edit bindings.rs - it's auto-generated

### Component structure:
- [x] Implements Guest trait
- [x] Exports component correctly
- [x] Properly handles TriggerAction and TriggerData

### ABI handling:
- [x] Properly decodes function calls
- [x] Avoids String::from_utf8 on ABI data

### Data ownership:
- [x] All API structures derive Clone
- [x] Clones data before use
- [x] Avoids moving out of collections
- [x] Avoids all ownership issues and "Move out of index" errors

### Error handling:
- [x] Uses ok_or_else() for Option types
- [x] Uses map_err() for Result types
- [x] Provides descriptive error messages

### Imports:
- [x] Includes all required traits and types
- [x] Uses correct import paths
- [x] Properly imports SolCall for encoding
- [x] Each and every method and type is used properly and has the proper import
- [x] Both structs and their traits are imported
- [x] Verify all required imports are imported properly
- [x] All dependencies are in Cargo.toml with `{workspace = true}`
- [x] Any unused imports are removed

### Component structure:
- [x] Uses proper sol! macro with correct syntax
- [x] Correctly defines Solidity types in solidity module
- [x] Implements required functions

### Security:
- [x] No hardcoded API keys or secrets
- [x] Uses environment variables for sensitive data

### Dependencies:
- [x] Uses workspace dependencies correctly
- [x] Includes all required dependencies

### Solidity types:
- [x] Properly imports sol macro
- [x] Uses solidity module correctly
- [x] Handles numeric conversions safely
- [x] Uses .to_string() for all string literals in struct initialization

### Network requests:
- [x] Uses block_on for async functions
- [x] Uses fetch_json with correct headers
- [x] ALL API endpoints have been tested with curl and responses handled correctly
- [x] Uses #[serde(default)] and Option<T> for fields that might be missing in API responses
