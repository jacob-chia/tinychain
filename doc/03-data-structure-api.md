# 03 | 定义数据结构与接口

> 本文为实战课，需要切换到对应的代码分支，并配合下方的参考文档一起学习。
>
> - 相关代码分支：`git fetch && git switch 03-data-structure-api`
> - [Workspace 官方文档](https://doc.rust-lang.org/cargo/reference/workspaces.html)
> - [Pre-commit 官方文档](https://pre-commit.com/)
> - [Github Action 官方文档](https://docs.github.com/en/actions)

## 0 写在前面

从本课开始，我们就要依赖各种库了，要学习一个库如何使用，没有什么比阅读库文档（有时需要配合阅读源代码）更好的方法了。任何转述概括类的文章都具有时效性，尤其是 Rust 生态还在快速发展中，各种库可能会做出向后不兼容的更新，所以你看到的转述类的文章很可能是过时的。比如本项目开发过程中就遇到过两次向后不兼容的更新，需要中依赖的 http 框架 `axum` 和 p2p 框架 `rust-libp2p`，

还好我们的架构设计保证和核心业务的稳定性，还记得我们的架构设计吗，tinyp2p 封装了 p2p 库的不稳定因素，而 network 封装了 http 库的不稳定性。

https://doc.rust-lang.org/book/ch15-02-deref.html

## 1 创建 workspace

## 99 小结

---

| [< 02-项目初始化：Pre-commit Hooks 与 Github Action](./02-init-project.md) | [04-钱包: 签名与验签 >](./04-wallet.md) |
| -------------------------------------------------------------------------- | --------------------------------------- |
