# Palomadex Generator Vesting

The Generator Vesting contract progressively unlocks PDEX that can then be distributed to token / LP stakers via the Generator contract.

---

## InstantiateMsg

Initializes the contract with the address of the PDEX token.

```json
{
  "lock_denom": "factory/paloma...",
  "owner": "paloma...",
}
```

### `receive`

CW20 receive msg.

```json
{
  "receive": {
    "sender": "paloma...",
    "amount": "123",
    "msg": "<base64_encoded_json_string>"
  }
}
```

### `claim`

Transfer reward tokens.

```json
{
  "claim": {
    "recipient": "paloma...",
    "amount": "123"
  }
}
```

## QueryMsg

All query messages are described below. A custom struct is defined for each query response.

### `config`

Returns the vesting token contract address (the PDEX token address).

```json
{
  "config": {}
}
```

### `available amount`

Returns the claimable amount (vested but not yet claimed) of PDEX tokens that a vesting target can claim.

```json
{
  "available_amount": {
    "address": "paloma..."
  }
}
```
