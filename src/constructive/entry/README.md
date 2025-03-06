# Combinators
`Brollup` employs of 8 types of combinators:

| Entry Type       |  Description                                                                                  |
|:-----------------|:----------------------------------------------------------------------------------------------|
| Liftup ⬆️        | Lifts one or more `Lift` outputs.                                                             |
| Recharge 🔋      | Refreshes one or more `Channel` liquidity into a fresh, new `VTXO`.                           |
| Move 💸          | Moves sats.                                                                                   |
| Call 📡          | Calls a smart contract. This may internally involve moving sats.                              |
| Add ➕           | Adds liquidity.                                                                               |
| Remove ➖        | Removes liquidity.                                                                            |
| Swapout ⬇️       | Swaps out sats into a bare taproot address.                                                   |
| Reserved 📁      | Fails the entry. Reserved for future upgrades.                                                |


## Entry Encoding
                                                    
    ┌──────────────────────────┐    ┌─────────────────────────────────────────────────────────────────────────────┐     
    │ Uppermost Left Branch    │    │ Uppermost Right Branch                                                      │
    │ b:0 => off               │    │ b:0 => off                                                                  │
    │ b:1 => on                │    │ b:1 => on                                                                   │
    └──────────────────────────┘    └─────────────────────────────────────────────────────────────────────────────┘         
            ┌────┘└────┐                        ┌─────────────────────────┘└─────────────┐
    ┌────────────┐┌────────────┐    ┌──────────────────────┐     ┌────────────────────────────────────────────────┐
    │ Liftup     ││ Recharge   │    │ Transact             │     │ Upper Right Branch                             │  
    │ b:0 => off ││ b:0 => off │    │ b:0                  │     │ b:1                                            │
    │ b:1 => on  ││ b:1 => on  │    └──────────────────────┘     └────────────────────────────────────────────────┘
    └────────────┘└────────────┘          ┌────┘└────┐                       ┌───────────┘└───────────┐
                                    ┌──────────┐┌──────────┐     ┌──────────────────────┐  ┌──────────────────────┐
                                    │ Move     ││ Call     │     │ Liquidity            │  │ Right Branch         │
                                    │ b:0      ││ b:1      │     │ b:0                  │  │ b:1                  │
                                    └──────────┘└──────────┘     └──────────────────────┘  └──────────────────────┘
                                                                       ┌────┘└────┐              ┌────┘└────┐
                                                                 ┌──────────┐┌──────────┐  ┌──────────┐┌──────────┐
                                                                 │ Add      ││ Remove   │  │ Swapout  ││ Reserved │
                                                                 │ b:0      ││ b:1      │  │ b:0      ││ b:1      │
                                                                 └──────────┘└──────────┘  └──────────┘└──────────┘

- `Uppermost Left Branch` and `Uppermost Right Branch` can be both set to `on`.

- If `Uppermost Left Branch` set to `on`;
    - 1. `Liftup` and `Recharge` can be both set to `on`.
    - 2. `Liftup` can be set to `on` and `Recharge` be set to `off`.
    - 3. `Liftup` can be set to `off` and `Recharge` be set to `on`.

- If `Uppermost Right Branch` is set to `on`;
    - 1. `Transact` can be set to `on` and `Upper Right Branch` be set to `off`.
    - 2. `Transact` can be set to `off` and `Upper Right Branch` be set to `on`.

    - If `Transact` is set to `on`;
        - 1. `Move` can be set to `on` and `Call` be set to `off`.
        - 2. `Move` can be set to `off` and `Call` be set to `on`.

    - If `Upper Right Branch` is set to `on`;
        - 1. `Liquidity` can be set to `on` and `Right Branch` be set to `off`.
        - 2. `Right Branch` can be set to `off` and `Liquidity` be set to `on`.

            - If `Liquidity` is set to `on`;
                - 1. `Add` can be set to `on` and `Remove` be set to `off`.
                - 2. `Remove` can be set to `off` and `Add` be set to `on`.

            - If `Right Branch` is set to `on`;
                - 1. `Swapout` can be set to `on` and `Reserved` be set to `off`.
                - 2. `Reserved` can be set to `off` and `Swapout` be set to `on`.