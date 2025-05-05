// Required imports
use alloy_sol_types::{sol, SolCall, SolValue};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use wavs_wasi_chain::decode_event_log_data;
use wavs_wasi_chain::http::{fetch_json, http_request_get};
use wstd::{http::HeaderValue, runtime::block_on};

pub mod bindings; // Never edit bindings.rs!
use crate::bindings::wavs::worker::layer_types::{TriggerData, TriggerDataEthContractEvent};
use crate::bindings::{export, Guest, TriggerAction};

// Define destination for output
pub enum Destination {
    Ethereum,
    CliOutput,
}

// Farcaster API Response Types
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UserNameProofResponse {
    #[serde(default)]
    name: Option<String>,
    #[serde(default)]
    owner: Option<String>,
    #[serde(default)]
    fid: Option<u64>,
    #[serde(default)]
    timestamp: Option<u64>,
    #[serde(default)]
    type_field: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CastAddBody {
    #[serde(default)]
    text: Option<String>,
    #[serde(default)]
    embeds: Option<Vec<serde_json::Value>>,
    #[serde(default)]
    mentions: Option<Vec<serde_json::Value>>,
    #[serde(default)]
    parent_cast_id: Option<serde_json::Value>,
    #[serde(default)]
    parent_url: Option<String>,
    #[serde(default)]
    mentions_positions: Option<Vec<serde_json::Value>>,
    #[serde(default)]
    type_field: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CastData {
    #[serde(default)]
    type_field: Option<String>,
    #[serde(default)]
    fid: Option<u64>,
    #[serde(default)]
    timestamp: Option<u64>,
    #[serde(default)]
    network: Option<String>,
    #[serde(default, rename = "castAddBody")]
    cast_add_body: Option<CastAddBody>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CastMessage {
    #[serde(default)]
    data: Option<CastData>,
    #[serde(default)]
    hash: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CastsResponse {
    #[serde(default)]
    messages: Option<Vec<CastMessage>>,
    #[serde(default)]
    next_page_token: Option<String>,
}

// Component Result Type
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct EigenCountResult {
    username: String,
    wallet_address: String,
    eigen_mentions: u64,
    total_casts: u64,
    timestamp: String,
}

// Define Solidity function signature for input
sol! {
    function countEigenLayerMentions(string username) external;
}

// Create separate solidity module for ITypes
mod solidity {
    use alloy_sol_macro::sol;
    pub use ITypes::*;

    sol!("../../src/interfaces/ITypes.sol");
}

// Component struct declaration
struct Component;
export!(Component with_types_in bindings);

// Main component implementation
impl Guest for Component {
    fn run(action: TriggerAction) -> std::result::Result<Option<Vec<u8>>, String> {
        let (trigger_id, req, dest) =
            decode_trigger_event(action.data).map_err(|e| e.to_string())?;

        // Clone request data to avoid ownership issues
        let req_clone = req.clone();

        // Decode the username from the input
        let username =
            if let Ok(decoded) = countEigenLayerMentionsCall::abi_decode(&req_clone, false) {
                // Successfully decoded as function call
                decoded.username
            } else {
                // Try decoding just as a string parameter
                match String::abi_decode(&req_clone, false) {
                    Ok(s) => s,
                    Err(e) => return Err(format!("Failed to decode input as ABI string: {}", e)),
                }
            };

        // Process the request
        let res = block_on(async move {
            let result = process_warpcast_request(&username).await?;
            serde_json::to_vec(&result).map_err(|e| e.to_string())
        })?;

        // Return result based on destination
        let output = match dest {
            Destination::Ethereum => Some(encode_trigger_output(trigger_id, &res)),
            Destination::CliOutput => Some(res),
        };
        Ok(output)
    }
}

// Helper function to decode trigger event
pub fn decode_trigger_event(trigger_data: TriggerData) -> Result<(u64, Vec<u8>, Destination)> {
    match trigger_data {
        TriggerData::EthContractEvent(TriggerDataEthContractEvent { log, .. }) => {
            let event: solidity::NewTrigger = decode_event_log_data!(log)?;
            let trigger_info =
                <solidity::TriggerInfo as SolValue>::abi_decode(&event._triggerInfo, false)?;
            Ok((trigger_info.triggerId, trigger_info.data.to_vec(), Destination::Ethereum))
        }
        TriggerData::Raw(data) => Ok((0, data.clone(), Destination::CliOutput)),
        _ => Err(anyhow::anyhow!("Unsupported trigger data type")),
    }
}

// Helper function to encode trigger output
pub fn encode_trigger_output(trigger_id: u64, output: impl AsRef<[u8]>) -> Vec<u8> {
    solidity::DataWithId { triggerId: trigger_id, data: output.as_ref().to_vec().into() }
        .abi_encode()
}

// Main request processing function
async fn process_warpcast_request(username: &str) -> Result<EigenCountResult, String> {
    // 1. Get user info to obtain FID and wallet address
    let user_info = get_warpcast_user_info(username).await?;

    // 2. Get user's casts and count EigenLayer mentions
    let (eigen_mentions, total_casts) = count_eigen_mentions(user_info.fid.unwrap_or(0)).await?;

    // 3. Create the result
    let result = EigenCountResult {
        username: username.to_string(),
        wallet_address: user_info.owner.unwrap_or_else(|| "unknown".to_string()),
        eigen_mentions,
        total_casts,
        timestamp: get_current_timestamp(),
    };

    Ok(result)
}

// Get user info from Warpcast API
async fn get_warpcast_user_info(username: &str) -> Result<UserNameProofResponse, String> {
    let url = format!("https://hoyt.farcaster.xyz:2281/v1/userNameProofByName?name={}", username);

    let mut req = http_request_get(&url).map_err(|e| format!("Failed to create request: {}", e))?;

    req.headers_mut().insert("Accept", HeaderValue::from_static("application/json"));

    let user_info: UserNameProofResponse =
        fetch_json(req).await.map_err(|e| format!("Failed to fetch user info: {}", e))?;

    // Validate that we have the required data
    if user_info.fid.is_none() {
        return Err(format!("User '{}' not found or has no FID", username));
    }

    Ok(user_info)
}

// Count EigenLayer mentions in user's casts
async fn count_eigen_mentions(fid: u64) -> Result<(u64, u64), String> {
    let url = format!("https://hoyt.farcaster.xyz:2281/v1/castsByFid?fid={}", fid);

    let mut req = http_request_get(&url).map_err(|e| format!("Failed to create request: {}", e))?;

    req.headers_mut().insert("Accept", HeaderValue::from_static("application/json"));

    let casts_response: CastsResponse =
        fetch_json(req).await.map_err(|e| format!("Failed to fetch casts: {}", e))?;

    let messages = casts_response.messages.unwrap_or_default();
    let total_casts = messages.len() as u64;

    // Count mentions of "EigenLayer" (case-insensitive)
    let mut eigen_count = 0;
    for message in messages {
        if let Some(data) = message.data {
            if let Some(cast_body) = data.cast_add_body {
                if let Some(text) = cast_body.text {
                    let text_lower = text.to_lowercase();
                    if text_lower.contains("eigenlayer") {
                        eigen_count += 1;
                    }
                }
            }
        }
    }

    Ok((eigen_count, total_casts))
}

// Helper function to get current timestamp
fn get_current_timestamp() -> String {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    now.to_string()
}
