- [02 | Initialization: Pre-commit Hooks \& Github Action](#02--initialization-pre-commit-hooks--github-action)
  - [1 Creating A Workspace](#1-creating-a-workspace)
  - [2 Pre-commit Hooks](#2-pre-commit-hooks)
  - [3 Github Action](#3-github-action)
  - [4 Summary](#4-summary)

# 02 | Initialization: Pre-commit Hooks & Github Action

> This is a hands-on tutorial, so please switch to the corresponding code branch before reading.
>
> - Repo: `https://github.com/jacob-chia/tinychain.git`
> - Branch：`git fetch && git switch 02-init-project`
> - [Workspace Doc](https://doc.rust-lang.org/cargo/reference/workspaces.html)
> - [Pre-commit Doc](https://pre-commit.com/)
> - [Github Action Doc](https://docs.github.com/en/actions)

Before writing code, we need to create a workspace and configure the necessary "guards" to ensure that the code we commit locally and to Github meets basic requirements.

## 1 Creating A Workspace

Our workspace is divided into three crates: `tinychain` (root crate), `wallet` (sub crate), and `tinyp2p` (sub crate). Let's do it step by step:

1. Create the root crate: `cargo new tinychain`
2. Modify `tinychain/Cargo.toml`, content as follows:

```toml
# Workspace -------------------------------

[workspace]

members = ["wallet", "tinyp2p"]

# Root Crate ----------------------------

[package]

name = "tinychain"
version = "0.1.0"
edition = "2021"
```

3. Create the sub crates: `cargo new wallet --lib && cargo new tinyp2p --lib`

That's it.

## 2 Pre-commit Hooks

Pre-commit hooks can do a series of checks before we commit code each time to ensure code quality. Let's do it step by step:

1. Install pre-commit: `pip3 install pre-commit`
2. Add `.pre-commit-config.yaml` in the root directory of the workspace, content as follows:

```yml
repos:
  # Some pre-defined hooks
  - repo: https://github.com/pre-commit/pre-commit-hooks
    rev: v4.4.0
    hooks:
      - id: check-added-large-files
      - id: check-case-conflict
      - id: check-merge-conflict
      - id: fix-byte-order-marker
  # Custom hooks
  - repo: local
    hooks:
      - id: cargo-fmt # Formatting
        name: cargo fmt
        description: Format files with rustfmt.
        entry: bash -c 'cargo fmt -- --check'
        language: rust
        pass_filenames: false
      - id: cargo-clippy # Lint
        name: cargo clippy
        description: Lint rust sources
        entry: bash -c 'cargo clippy --all-targets --all-features --tests --benches -- -D warnings'
        language: rust
        pass_filenames: false
      - id: cargo-test # Unit Test
        name: cargo test
        description: unit test for the project
        entry: bash -c 'cargo test --all-features --all'
        language: rust
        pass_filenames: false
```

3. Install hooks: `pre-commit install`

Then, after each `git commit`, the above hooks will be automatically executed. The commit will only happen if all hooks have passed. Here is an example that `cargo-clippy` will treat as an error: I need to decode the hex string into bytes, and the function should to be compatible with the case of having and not having the `0x` prefix. First I wrote it like this:

```rs
fn try_from(value: &str) -> Result<Self, Self::Error> {
    let val = if value.starts_with("0x") {
        &value[2..]
    } else {
        &value
    };

    hex::decode(val).map(Self::from)
}
```

When I did `cargo clippy --all-targets --all-features --tests --benches -- -D warnings`, I got the following prompt:

![](../img/02-precommit.png)

## 3 Github Action

Pre-commit can ensure that the code committed to `local` meets the requirements, while Github Action can ensure that the code committed to `PR` or merged into the `main` branch meets the requirements. Let's add `.github/workflows/build.yml` in the root directory, content as follows:

```yml
name: build

on:
  push:
    branches:
      - main
  pull_request:
    branches:
      - main

jobs:
  build-rust:
    strategy:
      matrix:
        platform: [ubuntu-latest]
    runs-on: ${{ matrix.platform }}
    steps:
      - uses: actions/checkout@v2
      - name: Cache cargo registry
        uses: actions/cache@v1
        with:
          path: ~/.cargo/registry
          key: ${{ runner.os }}-cargo-registry
      - name: Cache cargo index
        uses: actions/cache@v1
        with:
          path: ~/.cargo/git
          key: ${{ runner.os }}-cargo-index
      - name: Cache cargo build
        uses: actions/cache@v1
        with:
          path: target
          key: ${{ runner.os }}-cargo-build-target
      - name: Install stable
        uses: actions-rs/toolchain@v1
        with:
          profile: minimal
          toolchain: stable
          override: true
      # Protoc is used to compile protobuf files, which we will use later
      - name: Install Protoc
        uses: arduino/setup-protoc@v2
        with:
          version: "23.x"
      - name: Check code format
        run: cargo fmt -- --check
      - name: Check the package for errors
        run: cargo check --all
      - name: Lint rust sources
        run: cargo clippy --all-targets --all-features --tests --benches -- -D warnings
      - name: Run tests
        run: cargo test --all-features -- --test-threads=1 --nocapture
```

## 4 Summary

In this lesson, we configured the project, understood the directory structure of the workspace, and learned about pre-commit hooks and GitHub actions. We are now ready to start writing code with confidence.

---

| [< 01-Architecture](./01-architecture.md) | [03-Defining Data Structure & API >](./03-data-structure-api.md) |
| ----------------------------------------- | ---------------------------------------------------------------- |
