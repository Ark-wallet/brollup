![Brollup](https://i.ibb.co/tc7S2JL/brollup-github.png)
Brollup is a rollup designed for trustless, Bitcoin-native smart contracts. A true layer two with unilateral exit, users retain full custody of their funds, while rollup operators solely function as service providers.
> [!NOTE]
> Brollup is currently in the early development process.

## Installation

Ensure you have [Rust](https://www.rust-lang.org/tools/install) installed. Clone the repository and navigate into the project directory:

```sh
git clone https://github.com/brollup/brollup
cd brollup
```

## Usage

Run the program with the following command:

```sh
cargo run <chain> <mode> <bitcoin-rpc-url> <bitcoin-rpc-user> <bitcoin-rpc-password>
```

### Parameters:

- `<chain>`: The Bitcoin network to use. Supported values:
  - `signet`
  - `mainnet`
- `<mode>`: The mode in which the program runs. Supported values:
  - `node`: For running a Brollup node.
  - `operator`: For liquidity providers.
- `<bitcoin-rpc-url>`: The RPC URL of the Bitcoin node.
- `<bitcoin-rpc-user>`: The RPC username.
- `<bitcoin-rpc-password>`: The RPC password.

### Example:

```sh
cargo run signet node http://127.0.0.1:38332 user password
```

```sh
cargo run mainnet operator http://127.0.0.1:8332 user password
```

## Contributing

Contributions are welcome! Please submit a pull request or open an issue if you encounter any problems.

## License

This project is licensed under the CC0 1.0 Universal License. See the `LICENSE` file for details.

