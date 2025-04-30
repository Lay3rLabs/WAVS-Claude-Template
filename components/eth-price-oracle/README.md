# ETH Price Oracle Component

## Overview

The ETH Price Oracle component fetches cryptocurrency price data from CoinMarketCap's API and returns it in a format usable in smart contracts.

## What It Does

- Accepts a cryptocurrency ID (hex digit) as input
- Queries the CoinMarketCap API for current price information
- Returns structured price data including symbol, timestamp, and current price

## Key Features

- Properly formats HTTP requests with appropriate headers to avoid API restrictions
- Handles ABI encoding/decoding for blockchain compatibility
- Returns formatted JSON data for on-chain or CLI use

## Input Format

The component expects a single hex digit (0-9, a-f) as the first character of the input string, which is used as the cryptocurrency ID for CoinMarketCap.

## Output Format

```json
{
  "symbol": "BTC",
  "timestamp": "2023-04-24T15:30:00.000Z",
  "price": 64538.09
}
```

## WASI Execution Command

```bash
# Use this command to test the component locally:
export TRIGGER_DATA_INPUT=`cast abi-encode "f(string)" "1"`
export COMPONENT_FILENAME=eth_price_oracle.wasm
export SERVICE_CONFIG="'{\"fuel_limit\":100000000,\"max_gas\":5000000,\"host_envs\":[],\"kv\":[],\"workflow_id\":\"default\",\"component_id\":\"default\"}'"
make wasi-exec
```

Note: The input "1" corresponds to Bitcoin (CoinMarketCap ID 1).


export TRIGGER_DATA_INPUT=`cast abi-encode "f(string)" "dabit3"`
export COMPONENT_FILENAME=warpcast_eigenlayer_tracker.wasm
export SERVICE_CONFIG="'{\"fuel_limit\":2000000000,\"max_gas\":50000000,\"host_envs\":[],\"kv\":[],\"workflow_id\":\"default\",\"component_id\":\"default\"}'"
make wasi-exec
