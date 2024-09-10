<p align="center">
   <img src="doc/images/logo.png"
        alt="IG Client"
        width="400"
        title="https://www.ig.com">
</p>



[![Dual License](https://img.shields.io/badge/license-MIT%20and%20Apache%202.0-blue)](LICENSE)
[![Crates.io](https://img.shields.io/crates/v/ARPP.svg)](https://crates.io/crates/ARPP)
[![Downloads](https://img.shields.io/crates/d/ARPP.svg)](https://crates.io/crates/ARPP)
[![Stars](https://img.shields.io/github/stars/joaquinbejar/ARPP.svg)](https://github.com/joaquinbejar/ARPP/stargazers)

[![Build Status](https://img.shields.io/github/workflow/status/joaquinbejar/ARPP/CI)](https://github.com/joaquinbejar/ARPP/actions)
[![Coverage](https://img.shields.io/codecov/c/github/joaquinbejar/ARPP)](https://codecov.io/gh/joaquinbejar/ARPP)
[![Dependencies](https://img.shields.io/librariesio/github/joaquinbejar/ARPP)](https://libraries.io/github/joaquinbejar/ARPP)


# ARPP: Anchored Reference Price Protocol

## Table of Contents
1. [Introduction](#introduction)
2. [Features](#features)
3. [Project Structure](#project-structure)
4. [Setup Instructions](#setup-instructions)
5. [Library Usage](#library-usage)
6. [Usage Examples](#usage-examples)
7. [Testing](#testing)
8. [Contribution and Contact](#contribution-and-contact)


ARPP (Anchored Reference Price Protocol) is an innovative Automated Market Maker (AMM) formula implemented in Rust, designed to reduce price volatility in decentralized exchanges.

## Key Features:

- Reduced dependency on pool liquidity for price stability
- Oracle-based price anchoring for improved market alignment
- Configurable parameters for fine-tuned market behavior
- Rust implementation for high performance and safety
- Liquidity pool simulation engine
- Monte Carlo analysis for robustness testing
- Data visualization tools for performance analysis
- Configurable parameters for market behavior fine-tuning

ARPP aims to solve the high volatility issues in traditional AMM models, particularly in low liquidity scenarios. This repository contains the core implementation, testing suite, and documentation for integrating ARPP into DeFi protocols.

## Formula:

$$
P = P_{\text{ref}} \cdot \left( 1 + \alpha \cdot \arctan\left( \beta \cdot (R - 1) \right) \right)
$$

**Where:**

- $ P $: Asset price in the pool
- $ P_{\text{ref}} $: Reference price from oracle
- $\alpha, \beta $: Adjustable parameters
-$R $: Ratio of assets in the pool

## Components:

1. Core ARPP Implementation:
   - Efficient Rust code for ARPP formula calculations
   - Liquidity pool state management

2. Simulation Engine:
   - Monte Carlo simulations to test various market scenarios
   - Customizable trading strategies and market conditions

3. Data Analysis and Visualization:
   - Generation of performance metrics (e.g., price stability, slippage, impermanent loss)
   - Integration with plotting libraries (e.g., plotters) for creating graphs and charts

4. CLI Interface:
   - Easy-to-use command-line tools for running simulations and generating reports

## Technology Stack:

- Rust (core implementation and simulations)
- Rand crate for Monte Carlo simulations
- Plotters or similar for data visualization
- Rayon for parallel computing in simulations
- Serde for data serialization/deserialization

This project aims to provide researchers, DeFi developers, and financial analysts with a powerful tool to explore, validate, and optimize the ARPP formula for real-world applications in decentralized finance.

Contributions and feedback are welcome as we work towards creating more stable and efficient AMM solutions for the DeFi ecosystem.
Ideal for researchers, DeFi developers, and enthusiasts interested in next-generation AMM designs.

## Project Structure

The project is structured as follows:

1. **Configuration** (`src/config.rs`): Configuration settings for the IG Client.

2. **Application Layer** (`src/application/`):
    - **API** (`src/application/api/`):
        - **Client** (`src/application/api/client.rs`): Main client interface for interacting with the IG API.
        - **Endpoints** (`src/application/api/endpoints.rs`): Definitions of API endpoints.
    - **Models** (`src/application/models/`):
        - **Account** (`src/application/models/account.rs`): Account-related data structures.
        - **Market** (`src/application/models/market.rs`): Market-related data structures.
        - **Order** (`src/application/models/order.rs`): Order-related data structures.
    - **Services** (`src/application/services/`):
        - **Market Data** (`src/application/services/market_data.rs`): Functions for retrieving and processing market data.
        - **Order Execution** (`src/application/services/order_execution.rs`): Functions for executing orders.
        - **Order Management** (`src/application/services/order_management.rs`): Functions for managing existing orders.

3. **Presentation Layer** (`src/presentation/`):
    - **Encryption** (`src/presentation/encryption.rs`): Data encryption utilities.
    - **Serialization** (`src/presentation/serialization.rs`): Data serialization and deserialization utilities.

4. **Session Management** (`src/session/`):
    - **Authentication** (`src/session/auth.rs`): Authentication and session management.

5. **Transport Layer** (`src/transport/`):
    - **HTTP Client** (`src/transport/http_client.rs`): Core implementation of the HTTP client for interacting with the IG API.

6. **Utilities** (`src/utils/`):
    - **Error Handling** (`src/utils/error.rs`): Custom error types and error handling utilities.

7. **Tests** (`tests/`): Directory containing all unit tests.

8. **Benchmarks** (`benches/`): Directory containing benchmark tests.

9. **Examples** (`examples/`): Directory containing usage examples.

## Setup Instructions

1. Clone the repository:
```shell
git clone https://github.com/joaquinbejar/ARPP.git
cd ARPP
```

2. Build the project:
```shell
cargo build
```

3. Run tests:
```shell
cargo test
```

4. Format the code:
```shell
cargo fmt
```

5. Run linting:
```shell
cargo clippy
```

## Library Usage

To use the library in your project, add the following to your `Cargo.toml`:

```toml
[dependencies]
ARPP = { git = "https://github.com/joaquinbejar/ARPP.git" }
```


## Testing

To run unit tests:
```shell
cargo test
```

To run tests with coverage:
```shell
cargo tarpaulin
```

## Contribution and Contact

We welcome contributions to this project! If you would like to contribute, please follow these steps:

1. Fork the repository.
2. Create a new branch for your feature or bug fix.
3. Make your changes and ensure that the project still builds and all tests pass.
4. Commit your changes and push your branch to your forked repository.
5. Submit a pull request to the main repository.

If you have any questions, issues, or would like to provide feedback, please feel free to contact the project maintainer:

**Joaquín Béjar García**
- Email: jb@taunais.com
- GitHub: [joaquinbejar](https://github.com/joaquinbejar)

We appreciate your interest and look forward to your contributions!