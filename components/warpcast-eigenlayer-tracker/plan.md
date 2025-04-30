# Warpcast EigenLayer Tracker Component Plan

## Component Overview
This component will take a Warpcast username, retrieve all their casts, count mentions of "EigenLayer" (case-insensitive), and return this count along with the user's wallet address.

## Flow Chart
1. Receive Warpcast username as input
2. Query the Warpcast API to get the FID and wallet address from username
3. Use the FID to query all the user's casts
4. Count occurrences of "EigenLayer" in all casts (case-insensitive)
5. Return the count and wallet address

## Required Imports
```rust
use alloy_sol_types::{sol, SolCall, SolValue};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use wavs_wasi_chain::decode_event_log_data;
use wavs_wasi_chain::http::{fetch_json, http_request_get};
use wstd::{http::HeaderValue, runtime::block_on};
use std::str::FromStr;
```

## API Response Structures
Based on the API testing, we'll need these response structures:

1. Username lookup response:
```rust
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UserNameProof {
    pub fid: u64,
    pub owner: String,
    pub name: String,
    pub timestamp: u64,
    // Other fields not used
}
```

2. Casts response:
```rust
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CastsResponse {
    pub messages: Vec<CastMessage>,
    pub next_page_token: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CastMessage {
    pub data: CastData,
    // Other fields not used
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CastData {
    #[serde(rename = "castAddBody")]
    pub cast_add_body: Option<CastAddBody>,
    // Other fields not used
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CastAddBody {
    pub text: String,
    // Other fields not used
}
```

3. Result data:
```rust
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct EigenLayerMentionResult {
    pub username: String,
    pub wallet_address: String,
    pub total_mentions: u64,
}
```

## Solidity Interface
```rust
sol! {
    function trackEigenLayerMentions(string warpcastUsername) external;
}
```

## Validation Checklist
- [x] ✅ ALWAYS use `{ workspace = true }` in your component Cargo.toml
- [x] ✅ ALWAYS verify API response structures by using curl on the endpoints
- [x] ✅ ALWAYS Read any documentation given to you in a prompt
- [x] ✅ ALWAYS implement the Guest trait and export your component
- [x] ✅ ALWAYS use `export!(Component with_types_in bindings)`
- [x] ✅ ALWAYS use `clone()` before consuming data to avoid ownership issues
- [x] ✅ ALWAYS derive `Clone` for API response data structures
- [x] ✅ ALWAYS decode ABI data properly, never with `String::from_utf8`
- [x] ✅ ALWAYS use `ok_or_else()` for Option types, `map_err()` for Result types
- [x] ✅ ALWAYS use string parameters for CLI testing (`cast abi-encode "f(string)" "5"` instead of `f(uint256)`)
- [x] ✅ ALWAYS use `.to_string()` to convert string literals (&str) to String types in struct field assignments
- [x] ✅ NEVER edit bindings.rs - it's auto-generated
