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

// Define Solidity function signature that matches your input format
sol! {
    function trackEigenLayerMentions(string warpcastUsername) external;
}

// Create separate solidity module
mod solidity {
    use alloy_sol_macro::sol;
    pub use ITypes::*;

    sol!("../../src/interfaces/ITypes.sol");
}

// API response structure for username lookup
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UserNameProof {
    pub fid: u64,
    pub owner: String,
    pub name: String,
    pub timestamp: u64,
}

// API response structures for casts
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CastsResponse {
    pub messages: Vec<CastMessage>,
    #[serde(rename = "nextPageToken")]
    pub next_page_token: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CastMessage {
    pub data: CastData,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CastData {
    #[serde(rename = "castAddBody")]
    pub cast_add_body: Option<CastAddBody>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CastAddBody {
    pub text: String,
}

// Result data structure
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct EigenLayerMentionResult {
    pub username: String,
    pub wallet_address: String,
    pub total_mentions: u64,
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

        // Decode the username string using proper ABI decoding
        let username =
            if let Ok(decoded) = trackEigenLayerMentionsCall::abi_decode(&req_clone, false) {
                // Successfully decoded as function call
                decoded.warpcastUsername
            } else {
                // Try decoding just as a string parameter
                match String::abi_decode(&req_clone, false) {
                    Ok(s) => s,
                    Err(e) => return Err(format!("Failed to decode input as ABI string: {}", e)),
                }
            };

        // Process the request
        let res = block_on(async move {
            let result = track_eigenlayer_mentions(&username).await?;
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

// Trigger event handling
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

pub fn encode_trigger_output(trigger_id: u64, output: impl AsRef<[u8]>) -> Vec<u8> {
    solidity::DataWithId { triggerId: trigger_id, data: output.as_ref().to_vec().into() }
        .abi_encode()
}

// Implementation of the main functionality
async fn track_eigenlayer_mentions(username: &str) -> Result<EigenLayerMentionResult, String> {
    // Step 1: Get user's FID and wallet address
    let user_info = get_user_info(username).await?;

    // Step 2: Get all user's casts and count EigenLayer mentions
    let mentions_count = count_eigenlayer_mentions(user_info.fid).await?;

    // Return the result
    Ok(EigenLayerMentionResult {
        username: username.to_string(),
        wallet_address: user_info.owner,
        total_mentions: mentions_count,
    })
}

// Get user info (FID and wallet address) from username
async fn get_user_info(username: &str) -> Result<UserNameProof, String> {
    let url = format!("https://hoyt.farcaster.xyz:2281/v1/userNameProofByName?name={}", username);

    let req = http_request_get(&url).map_err(|e| format!("Failed to create request: {}", e))?;

    let user_info: UserNameProof =
        fetch_json(req).await.map_err(|e| format!("Failed to fetch user info: {}", e))?;

    Ok(user_info)
}

// Count EigenLayer mentions in all user's casts
async fn count_eigenlayer_mentions(fid: u64) -> Result<u64, String> {
    let mut total_mentions = 0;
    let mut next_page_token: Option<String> = None;

    // Loop to handle pagination
    loop {
        let url = match &next_page_token {
            Some(token) => format!(
                "https://hoyt.farcaster.xyz:2281/v1/castsByFid?fid={}&pageToken={}&limit=100",
                fid, token
            ),
            None => format!("https://hoyt.farcaster.xyz:2281/v1/castsByFid?fid={}&limit=100", fid),
        };

        let req = http_request_get(&url).map_err(|e| format!("Failed to create request: {}", e))?;

        let response: CastsResponse =
            fetch_json(req).await.map_err(|e| format!("Failed to fetch casts: {}", e))?;

        // Process each cast and count mentions
        for message in &response.messages {
            if let Some(cast_body) = &message.data.cast_add_body {
                // Case-insensitive search for "EigenLayer"
                if cast_body.text.to_lowercase().contains("eigenlayer") {
                    total_mentions += 1;
                }
            }
        }

        // Check if there are more pages
        if let Some(token) = response.next_page_token {
            next_page_token = Some(token);
        } else {
            break;
        }
    }

    Ok(total_mentions)
}
