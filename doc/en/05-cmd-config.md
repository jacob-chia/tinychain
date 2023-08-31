- [05 | Command Line \& Config File](#05--command-line--config-file)
  - [1 Defining Command Line Arguments](#1-defining-command-line-arguments)
  - [2 Parsing and Executing Commands](#2-parsing-and-executing-commands)
  - [3 Demo](#3-demo)
  - [4 Summary](#4-summary)

# 05 | Command Line & Config File

> This is a hands-on tutorial, so please switch to the corresponding code branch before reading.
>
> - Repo: `https://github.com/jacob-chia/tinychain.git`
> - Branch：`git fetch && git switch 05-cmd-config`
>
> Important crates used in this lesson:
>
> - [clap](https://docs.rs/clap/latest/clap/): Command Line Argument Parser for Rust
> - [toml](https://docs.rs/toml/latest/toml/): A serde-compatible TOML-parsing library

In this lesson, we will add two commands:

- `new-account`: create an account and return the address. The account storage path can be specified by parameters.
- `run`: run the tinychain node service. The running parameters are passed in through the configuration file.

## 1 Defining Command Line Arguments

Define the `Opts` and `SubCommand` according to the requirements of [clap](https://docs.rs/clap/latest/clap/). Let's look at the code directly.

```rs
// src/main.rs

/// The command of tinychain
#[derive(Debug, Parser)]
struct Opts {
    #[command(subcommand)]
    subcmd: SubCommand,
}

#[derive(Debug, Subcommand)]
enum SubCommand {
    /// Create a new account for signing transactions
    NewAccount {
        /// the keystore directory, default is `./db/keystore/`
        #[arg(short, long, default_value_t = String::from("./db/keystore/"))]
        keystore_dir: String,
    },
    /// Run the node
    Run {
        /// the config file path, default is `config.toml`
        #[arg(short, long, default_value_t = String::from("config.toml"))]
        config: String,
    },
}
```

## 2 Parsing and Executing Commands

```rs
// src/main.rs

#[tokio::main]
async fn main() {
    pretty_env_logger::init();
    let opts = Opts::parse();
    match opts.subcmd {
        SubCommand::NewAccount { keystore_dir } => new_account(&keystore_dir),
        SubCommand::Run { config } => run(&config).await,
    }
}

fn new_account(keystore_dir: &str) {
    let wallet = Wallet::new(keystore_dir);
    let acc = wallet.new_account().unwrap();
    info!("📣 New account: {:?}", acc);
}

async fn run(config_file: &str) {
    let cfg = Config::load(config_file).unwrap();
    info!("📣 Config loaded: {:?}", cfg);

    // ...
}
```

When it comes to `SubCommand::Run`, we need to specify a configuration file, and here is a template: [./config-template.toml](../../config-template.toml).
Accordingly, a struct should be defined to deserialize the configuration file. The code is as follows:

```rs
// src/config.rs

#[derive(Clone, Debug, Default, Deserialize)]
pub struct Config {
    /// The path to the data directory.
    pub data_dir: String,
    /// The path to the genesis file.
    pub genesis_file: String,
    /// The address to listen on for HTTP Server.
    pub http_addr: String,
    /// The miner account to receive mining rewards.
    pub author: String,
    /// Wallet configuration.
    pub wallet: WalletConfig,
}

impl Config {
    /// Load the configuration from the given path.
    pub fn load(path: &str) -> Result<Self, Error> {
        let content =
            fs::read_to_string(path).map_err(|_| Error::ConfigNotExist(path.to_string()))?;

        let config: Config = toml::from_str(&content)?;
        Ok(config)
    }
}
```

## 3 Demo

> Note: The log level in the code is `info`, so we need to add the `RUST_LOG=info` parameter when running.

1. Build: `cargo build`;
2. Check the commands: `RUST_LOG=info ./target/debug/tinychain`:

   ![](../img/05-cmd-help.png)

3. Create an account: `RUST_LOG=info ./target/debug/tinychain new-account`:

   ![](../img/05-cmd-new-account.png)

4. Run the service: `RUST_LOG=info ./target/debug/tinychain run -c ./config-template.toml`:

   ![](../img/05-cmd-run.png)

## 4 Summary

There is no difficulty in this lesson. It mainly demonstrates two functions:

- Parsing command line arguments
- Reading configuration files

---

| [< 04-Wallet: Sign & Verify](./04-wallet.md) | [06-Thinking in Libp2p >](./06-libp2p.md) |
| -------------------------------------------- | ----------------------------------------- |
