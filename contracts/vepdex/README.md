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

### `create_lock`


Lock PDEX and get VePDEX with end lock timestamp.
```json
{
    "create_lock": {
        "end_lock_time": 10000
    }
}
```


### `increase_lock_amount`

Lock more PDEX and get VePDEX.
```json
{
    "increase_lock_amount": {

    }
}
```

### `withdraw`

Withdraw locked PDEX

```json
{
  "withdraw": {
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
