- [08 | 网络层](#08--网络层)
  - [1 P2P](#1-p2p)
    - [1.1 实现 trait PeerClient](#11-实现-trait-peerclient)
    - [1.2 实现 trait EventHandler](#12-实现-trait-eventhandler)
    - [1.3 识别出的 biz 接口](#13-识别出的-biz-接口)
  - [2 HTTP](#2-http)
    - [2.1 根据错误类型返回 HTTP Status Code](#21-根据错误类型返回-http-status-code)
    - [2.2 识别出的 biz 接口](#22-识别出的-biz-接口)
  - [3 小结](#3-小结)

# 08 | 网络层

> 本文为实战课，需要切换到对应的代码分支，并配合依赖库的文档一起学习。
>
> - Repo: `https://github.com/jacob-chia/tinychain.git`
> - 分支：`git fetch && git switch 08-network`
> - [axum](https://docs.rs/axum/latest/axum/): HTTP Server 框架，由 tokio 团队维护
>
> 其他 crates 使用简单，不再一一列举，清单在`Cargo.toml`中

实现 network 要依赖 biz 提供的接口，那为什么我们先写 network？

回顾一下第一课的架构设计，我们的接口定义只做到了 crate 级别，crate 内部各层之间的交互并没有想清楚，如果需要做各层之间的接口定义，就需要画时序图了。其实对于小项目有个偷懒的办法，就是自顶向下的写代码，我们直接写 network，在实现过程中需要 biz 的什么功能就定义什么接口（内部不实现），等 network 写完了，biz 层的接口就识别出来了。

## 1 P2P

network::p2p 是 biz 与 tinyp2p 交互的桥梁，需要做两件事：

- 封装 tinyp2p::Client，实现 biz 定义的 `trait PeerClient`;
- 封装 biz::Node，实现 tinyp2p 定义的`triat EventHandler`;

### 1.1 实现 trait PeerClient

```rs
// src/network/p2p.rs 只保留了核心代码

#[derive(Debug, Clone)]
pub struct P2pClient(Client);

// Implement `Deref` so that we can call `Client` methods on `P2pClient`.
impl Deref for P2pClient {
    type Target = Client;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl PeerClient for P2pClient {
    fn known_peers(&self) -> Vec<String> {
        self.get_known_peers();
    }

    fn get_block_height(&self, peer_id: &str) -> Result<u64, Error> {
        let req = Request::new_block_height_req();
        let resp: Response = self.blocking_request(peer_id, req.into())?.try_into()?;
        Ok(BlockHeightResp::from(resp).block_height)
    }

    fn get_blocks(&self, peer_id: &str, from_number: u64) -> Result<Vec<Block>, Error> {
        let req = Request::new_blocks_req(from_number);
        let resp: Response = self.blocking_request(peer_id, req.into())?.try_into()?;
        let blocks = BlocksResp::from(resp).blocks;

        Ok(blocks)
    }

    fn broadcast_tx(&self, tx: SignedTx) {
        self.broadcast(Topic::Tx, tx.into());
    }

    fn broadcast_block(&self, block: Block) {
        self.broadcast(Topic::Block, Vec::from(&block));
    }
}
```

### 1.2 实现 trait EventHandler

```rs
// src/network/p2p.rs 只保留了核心代码

#[derive(Debug, Clone)]
pub struct EventHandlerImpl<S: State>(Node<S>);

impl<S: State> EventHandler for EventHandlerImpl<S> {
    fn handle_inbound_request(&self, request: Vec<u8>) -> Result<Vec<u8>, P2pError> {
        let req = Request::try_from(request).unwrap();
        let resp = match req.method() {
            Method::Height => {
                let block_height = self.block_height();
                Response::new_block_height_resp(block_height)
            }
            Method::Blocks => {
                let blocks = match req.body.unwrap() {
                    request::Body::BlocksReq(req) => self.get_blocks(req.from_number),
                    _ => vec![],
                };
                Response::new_blocks_resp(blocks)
            }
        };

        Ok(resp.into())
    }

    fn handle_broadcast(&self, topic: &str, message: Vec<u8>) {
        match Topic::from(topic) {
            Topic::Block => {
                if let Ok(block) = Block::try_from(message) {
                    self.handle_broadcast_block(block);
                }
            }
            Topic::Tx => {
                if let Ok(tx) = SignedTx::try_from(message) {
                    self.handle_broadcast_tx(tx);
                }
            }
        }
    }
}
```

### 1.3 识别出的 biz 接口

上文 EventHandlerImpl 是对 biz::Node 的封装，内部调用的`self.xxx()`都是 biz::Node 需要实现的接口。如下：

- `self.block_height()`
- `self.get_blocks(req.from_number)`
- `self.handle_broadcast_block(block)`
- `self.handle_broadcast_tx(tx)`

## 2 HTTP

network::http 接口的实现非常简单，都是 biz 层接口的转发，直接看源码[src/network/http/mod.rs](../../src/network/http/mod.rs)吧。但有个需求：如何根据错误类型返回不同的 HTTP Status Code？

### 2.1 根据错误类型返回 HTTP Status Code

简单来说，就是需要为 Error 类型实现 `trait IntoResponse`，但我们的全局 Error 不应依赖 axum，所以在 http 模块内部新增了一个 `HTTPError`，并为 HTTPError 实现了 IntoResponse。

```rs
// src/network/http/mod.rs

#[derive(thiserror::Error, Debug)]
enum HttpError {
    #[error("Bad request: {0}")]
    BadRequest(Error),
    #[error("Internal server error: {0}")]
    InternalServerError(Error),
}

impl From<Error> for HttpError {
    fn from(err: Error) -> Self {
        match err {
            Error::BadRequest(..) => HttpError::BadRequest(err),
            _ => HttpError::InternalServerError(err),
        }
    }
}

impl IntoResponse for HttpError {
    fn into_response(self) -> Response {
        let status = match self {
            HttpError::BadRequest(_) => StatusCode::BAD_REQUEST,
            HttpError::InternalServerError(_) => StatusCode::INTERNAL_SERVER_ERROR,
        };

        let body = Json(json!({ "error": self.to_string() }));

        (status, body).into_response()
    }
}
```

### 2.2 识别出的 biz 接口

在 http 接口内部调用的`node.xxx()` 都是 biz::Node 需要实现的接口，如下：

- `node.get_blocks(params.from_number)`
- `node.get_block(number)`
- `node.last_block_hash()`
- `node.get_balances()`
- `node.next_account_nonce(&params.account)`
- `node.transfer(&tx.from, &tx.to, tx.value, tx.nonce)`

## 3 小结

写完 network 我们再次体会到了 network 的作用：做类型转换和接口转发，不包含业务逻辑，目的是将 biz 与外部环境解耦，提升 biz 的稳定性。

另外，我们还识别出了 biz::Node 的接口，为下节课实现 biz 层打下了基础。

---

| [< 07-tinyp2p：基于 CSP 的无锁并发模型](./07-tinyp2p.md) | [09-业务层：在业务层如何做读写分离？ >](./09-biz.md) |
| -------------------------------------------------------- | ---------------------------------------------------- |
