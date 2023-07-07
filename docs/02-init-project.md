- [02 | 项目初始化：Pre-commit Hooks 与 Github Action](#02--项目初始化pre-commit-hooks-与-github-action)
  - [1 创建 workspace](#1-创建-workspace)
  - [2 Pre-commit Hooks](#2-pre-commit-hooks)
  - [3 Github Action](#3-github-action)
  - [4 小结](#4-小结)

# 02 | 项目初始化：Pre-commit Hooks 与 Github Action

> 本文为实战课，需要切换到对应的代码分支，并配合下方的参考文档一起学习。
>
> - Repo: `https://github.com/jacob-chia/tinychain.git`
> - 分支：`git fetch && git switch 02-init-project`
> - [Workspace 官方文档](https://doc.rust-lang.org/cargo/reference/workspaces.html)
> - [Pre-commit 官方文档](https://pre-commit.com/)
> - [Github Action 官方文档](https://docs.github.com/en/actions)

在写代码之前，我们需要创建一个 workspace，并配置必要的“Guard”，用来确保提交到本地和 Github 的代码满足最基本的要求。

## 1 创建 workspace

我们的 workspace 分为三个 crates：`tinychain` (root crate)、`wallet` (sub crate)、和 `tinyp2p` (sub crate)。具体构造步骤如下：

1. 在根目录执行：`cargo init`
2. 修改`./Cargo.toml`，内容如下：

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

3. 在根目录执行：`cargo new wallet --lib && cargo new tinyp2p --lib`

这样，一个 workspace 就初始化好了。

## 2 Pre-commit Hooks

Pre-commit hooks 可以在我们每次提交代码之前做一系列检查，来保证代码质量。具体配置过程如下：

1. 安装 pre-commit: `pip3 install pre-commit`
2. 在项目根目录添加`.pre-commit-config.yaml`，内容如下：

```yml
repos:
  # 一些现成的 hooks，还有更多hooks详见下方的 repo 链接
  - repo: https://github.com/pre-commit/pre-commit-hooks
    rev: v4.4.0
    hooks:
      - id: check-added-large-files
      - id: check-case-conflict
      - id: check-merge-conflict
      - id: fix-byte-order-marker
  # 一些本地 hooks
  - repo: local
    hooks:
      - id: cargo-fmt # 代码格式化
        name: cargo fmt
        description: Format files with rustfmt.
        entry: bash -c 'cargo fmt -- --check'
        language: rust
        pass_filenames: false
      - id: cargo-clippy # 静态检查
        name: cargo clippy
        description: Lint rust sources
        entry: bash -c 'cargo clippy --all-targets --all-features --tests --benches -- -D warnings'
        language: rust
        pass_filenames: false
      - id: cargo-test # 单元测试
        name: cargo test
        description: unit test for the project
        entry: bash -c 'cargo test --all-features --all'
        language: rust
        pass_filenames: false
```

3. 安装 hooks: `pre-commit install`

之后，在每次执行`git commit`时，会先自动执行上面的 hooks，所有 hooks 检查通过后才会 commit。

其中值得一提的是`cargo clippy`，它比`cargo check`能给出更多的代码改进建议，简直可以面向 clippy 编程。比如我遇到的这个例子：我需要把 hex string 解码成 bytes，需要兼容有`0x`前缀和无`0x`前缀的情况。一开始我是这么写的：

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

当我执行`cargo clippy --all-targets --all-features --tests --benches -- -D warnings`时会得到下面的提示：

![](img/02-precommit.png)

clippy 告诉我不要手动删除字符串前缀，而应该使用标准库提供的接口，并且给出了参考代码。

## 3 Github Action

Pre-commit 可以保证提交到`本地的代码`满足最基本的要求，而 Github Action 则可以保证提交到`PR`或合并到`main`分支的代码满足最基本的要求。

一个最基本的 Rust 项目 Github Action 配置如下，在根目录添加`.github/workflows/build.yml`，内容：

```yml
# 在`ubuntu-latest`中安装必要的依赖，然后执行代码风格检查和测试
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
      # 安装protobuf编译器，是在之后执行测试时依赖的编译环境
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

## 4 小结

本课，我们做好了项目的基本配置、了解了 workspace 的目录结构、以及如何配置 Pre-commit Hooks 和 Github Action，之后就可以放心写代码了。

另外，给您推荐两个页面作为扩展阅读：

- [Pre-commit Hooks](https://github.com/pre-commit/pre-commit-hooks): 这里中有很多现成的 Hooks
- [Github Action 市场](https://github.com/marketplace?type=actions): 这里有很多现成的 Actions。

---

| [< 01-架构设计](./01-architecture.md) | [03-定义数据结构与接口 >](./03-data-structure-api.md) |
| ------------------------------------- | ----------------------------------------------------- |
