- [07 | tinyp2p：基于 CSP 的无锁并发模型](#07--tinyp2p基于-csp-的无锁并发模型)
  - [1 CSP 并发模型](#1-csp-并发模型)
    - [1.1 架构](#11-架构)
    - [1.2 如何同步获取远端响应](#12-如何同步获取远端响应)
    - [1.3 如何异步处理远端数据](#13-如何异步处理远端数据)
  - [2 搭框架](#2-搭框架)
    - [2.1 构造 transport](#21-构造-transport)
    - [2.2 自定义 protocol](#22-自定义-protocol)
    - [2.3 封装 swarm (P2P Server)](#23-封装-swarm-p2p-server)
    - [2.4 定义 P2P Client](#24-定义-p2p-client)
    - [2.5 提供给用户的接口](#25-提供给用户的接口)
  - [3 实现需求](#3-实现需求)
    - [3.1 节点发现](#31-节点发现)
    - [3.2 向远端发送同步请求](#32-向远端发送同步请求)
    - [3.3 收到来自远端的请求](#33-收到来自远端的请求)
    - [3.4 向网络广播消息](#34-向网络广播消息)
    - [3.5 收到网络中的广播消息](#35-收到网络中的广播消息)
  - [4 功能演示](#4-功能演示)
  - [5 小结](#5-小结)

# 07 | tinyp2p：基于 CSP 的无锁并发模型

> 本文为实战课，需要切换到对应的代码分支，并配合依赖库的文档一起学习。
>
> - Repo: `https://github.com/jacob-chia/tinychain.git`
> - 分支：`git fetch && git switch 07-tinyp2p`
> - [rust-libp2p](https://docs.rs/libp2p/latest/libp2p/index.html): libp2p 的 Rust 实现。
> - [rust-libp2p examples](https://github.com/libp2p/rust-libp2p/tree/master/examples): 演示了各种 protocols 如何使用，本项目用到的 protocols 示例一定要看，尤其是`file-sharing` 用的是 CSP 并发模型，本项目的代码架构就是参考了`file-sharing`的实现。
>
> 其他 crates 使用简单，不再一一列举，清单在`tinyp2p/Cargo.toml`中

本课内容是上节课的实践课，请先阅读[06-libp2p: 需求分析与封装思路](./06-libp2p.md)再开始写代码。

## 1 CSP 并发模型

tinyp2p 参考上文提到的例子`file-sharing`，使用 CSP (Communicating Sequential Process) 并发模型。所以我们先介绍一下这个模型，搞清楚这个模型的代码结构之后，源码就没什么难度了。

### 1.1 架构

![](../img/07-csp.png)

上图中，`p2p_server` 用来处理用户请求。如果是基于锁的并发模型，需要在 p2p_server 外面加一层锁，每处理一个请求就要获取一次锁，这样显然是低效的。而 CSP 模型是这样的：

- `p2p_client` 用来处理用户请求，在 `p2p_client` 内部将请求转为 `cmd` 发送到 channel 中；
- 一个后台进程独占 `mut p2p_server`，逐个从 channel 中获取 cmd 执行；

那么，用户如何获取 p2p_server 的处理结果（响应）呢？用户如何处理来自远端的请求/广播消息呢？我们分两种情况讨论：

### 1.2 如何同步获取远端响应

可以基于`oneshot` channel 实现。假设用户需要发送一个`blocking_request`。伪代码如下：

- 对应的 cmd 定义：

```rs
pub enum Command {
    SendRequest {
        target: PeerId,
        request: Vec<u8>,
        // 在 cmd 中添加一个 oneshot::Sender
        responder: oneshot::Sender<ResponseType>,
    },
    // ...
}
```

- p2p_client 的接口:

```rs
pub fn blocking_request(&self, target: &PeerId, request: Vec<u8>) -> Result<Vec<u8>, P2pError> {
    // 创建一个 oneshot::channel
    let (responder, receiver) = oneshot::channel();

    // 发给 p2p_server 处理
    let _ = self.cmd_sender.send(Command::SendRequest {
        target,
        request,
        responder,
    });

    // 用 oneshot::Receiver 接收 p2p_server 的处理结果并返回给用户
    Ok(receiver.blocking_recv()?)
}
```

- p2p_server 的 cmd_handler:

```rs
fn handle_command(&mut self, cmd: Command) {
    match cmd {
        Command::SendRequest {
            target,
            request,
            responder,
        } => {
            // 处理过程略，假设拿到了远端的响应
            // 通过responder将结果发给p2p_client
            let _ = responder.send(response);
        }
        _other_cmds => {/* ... */}
    }
}
```

### 1.3 如何异步处理远端数据

> 这个“远端数据”包括远端对本地请求的响应、远端向本地发起的请求、远端广播的消息等。

一种方式是 p2p_server 通过 Event 将数据发送给用户，但用户层需要启动一个进程不断地监听来自 p2p_server 的 Event，这样就增加了用户的使用难度（`file-sharing` 就是这么做的）。

让用户更轻松的方式是：p2p_server 对外提供`event_handler`的注册接口，用户通过 event_handler 告诉 p2p_server 当收到远端数据时，应该怎么做。这样 Event 的监听工作就移到了 p2p_server 中，而 p2p_server 本来就需要监听来自远端的 Event，并没有增加工作量。

tinyp2p 需要两个 event_handers 来分别处理来自远端的请求和广播消息。

- 定义 EventHandler trait

```rs
pub trait EventHandler: Debug + Send + 'static {
    // 处理来自远端的请求
    fn handle_inbound_request(&self, request: Vec<u8>) -> Result<Vec<u8>, P2pError>;
    // 处理来自远端的广播
    fn handle_broadcast(&self, topic: &str, message: Vec<u8>);
}
```

- 对外提供注册 EventHandler 的接口

```rs
pub fn set_event_handler(&mut self, handler: E) {
    self.event_handler.set(handler).unwrap();
}
```

- 当收到远端数据时调用 EventHandler

```rs
// 监听SwarmEvent，细节略，假设收到了远端的请求
if let Some(handler) = self.event_handler.get() {
    let response = handler.handle_inbound_request(request);
    // 调用 request-response协议提供的send_response接口将response发送给远端
}
```

了解了 CSP 模型的架构，我们就可以开始写代码了。

## 2 搭框架

### 2.1 构造 transport

这个很简单，直接看代码：

```rs
// tinyp2p/src/transport.rs

pub fn build_transport(keypair: identity::Keypair) -> Boxed<(PeerId, StreamMuxerBox)> {
    let noise_config = noise::Config::new(&keypair).expect("failed to construct the noise config");

    tcp::tokio::Transport::default()
        .upgrade(Version::V1Lazy)
        .authenticate(noise_config)
        .multiplex(yamux::Config::default())
        .boxed()
}
```

### 2.2 自定义 protocol

我们的自定义 protocol 是官方 protocols 的组合，只需要定义一个结构体和一些转发接口即可：

```rs
// tinyp2p/src/protocol/mod.rs

#[derive(NetworkBehaviour)]
pub struct Behaviour {
    // `kad`, `identify`, and `ping` are used for peer discovery.
    kad: Kademlia<MemoryStore>,
    identify: identify::Behaviour,
    ping: ping::Behaviour,
    // `req_resp` is used for sending requests and responses.
    req_resp: request_response::Behaviour<GenericCodec>,
    // `pubsub` is used for broadcasting messages.
    pubsub: gossipsub::Behaviour,
}

impl Behaviour {
    // 构造函数
    pub fn new(/* config */) -> Result<Self, P2pError>;
    // 执行 Kademlia Bootstrap
    pub fn discover_peers(&mut self);
    // 当前DHT中有哪些节点
    pub fn known_peers(&mut self) -> HashMap<PeerId, Vec<Multiaddr>>;
    // 向远端节点 target 发送请求
    pub fn send_request(&mut self, target: &PeerId, request: Vec<u8>);
    // 通过 channel 向远端节点发送响应
    pub fn send_response(&mut self, ch: ResponseChannel<ResponseType>, response: ResponseType);
    // 向网络广播消息
    pub fn broadcast(&mut self, topic: String, message: Vec<u8>) -> Result<(), P2pError>;
    // 将地址添加至 DHT 中
    pub fn add_address(&mut self, peer_id: &PeerId, addr: Multiaddr);
    // 将节点从 DHT 中移除
    pub fn remove_peer(&mut self, peer_id: &PeerId);
}
```

### 2.3 封装 swarm (P2P Server)

Swarm 是真正对外提供服务的结构体，还记得我们上节课的内容吗，我们把自定义逻辑都放到这里。先看结构体的定义：

```rs
// trait EventHandler 是需要用户实现的event_handlers
pub struct Server<E: EventHandler> {
    /// Swarm
    network_service: Swarm<Behaviour>,

    /// CSP模型中 cmd 的接收端
    cmd_receiver: UnboundedReceiver<Command>,
    /// 用来处理远端发过来的数据
    event_handler: OnceCell<E>,

    /// 一个定时器，定时执行节点发现
    discovery_ticker: Interval,
    /// 用于实现CSP模型中的同步请求，稍后在实现需求时会解释如何使用
    pending_outbound_requests: HashMap<RequestId, oneshot::Sender<ResponseType>>,
    /// Gossipsub中的主题
    pubsub_topics: Vec<String>,

    /// 以下两个字段用于日志和调试
    local_peer_id: PeerId,
    listened_addresses: Vec<Multiaddr>,
}

impl<E: EventHandler> Server<E> {
    /// 构造函数
    pub fn new(/* params */) -> Result<Self, P2pError>;
    /// 注册 EventHandler
    pub fn set_event_handler(&mut self, handler: E);
    /// 运行服务，处理三类工作：
    /// - discovery_ticker 的定时任务
    /// - 来自 p2p_client 的 cmd
    /// - SwarmEvent
    pub async fn run(mut self);
}
```

- event_handler 为什么是 `OnceCell`?

与 http_server 不同，p2p 节点既是客户端（向外发送请求），又是服务端（处理收到的外部请求），所以 tinyp2p 的构造函数会同时构造一个 p2p_client 和 p2p_server。然后上层应用（对于本课程来说是 tinychain）使用 p2p_client 构造自己的 Node，再将 Node 封装为 EventHandler 注册到 p2p_server 中。

所以，在构造 Server 时还不知道具体的 EventHandler 是什么，需要在后期**注册且仅能注册一次**。`OnceCell` 刚好满足这个需求。

### 2.4 定义 P2P Client

p2p_client 用于接收来自用户的请求，将请求转为 cmd 发送至 p2p_server，对于同步请求，还负责接收 p2p_server 返回的响应。

```rs
// tinyp2p/src/service.rs

#[derive(Clone, Debug)]
pub struct Client {
    cmd_sender: UnboundedSender<Command>,
}

impl Client {
    /// 发送一条同步请求
    pub fn blocking_request(&self, target: &str, request: Vec<u8>) -> Result<Vec<u8>, P2pError>;
    /// 广播消息
    pub fn broadcast(&self, topic: impl Into<String>, message: Vec<u8>);
    /// 获取已知的节点PeerID
    pub fn get_known_peers(&self) -> Vec<String>;
}
```

### 2.5 提供给用户的接口

1. trait EventHandler，上文已多次提到，直接看定义。

```rs
pub trait EventHandler: Debug + Send + 'static {
    /// Handles an inbound request from a remote peer.
    fn handle_inbound_request(&self, request: Vec<u8>) -> Result<Vec<u8>, P2pError>;

    /// Handles an broadcast message from a remote peer.
    fn handle_broadcast(&self, topic: &str, message: Vec<u8>);
}
```

2. 提供一个构造函数，同时构造 p2p_client 和 p2p_server。

```rs
pub fn new<E: EventHandler>(config: P2pConfig) -> Result<(Client, Server<E>), P2pError> {
    let (cmd_sender, cmd_receiver) = mpsc::unbounded_channel();

    let server = Server::new(config, cmd_receiver)?;
    let client = Client { cmd_sender };

    Ok((client, server))
}
```

## 3 实现需求

> 这里只解释关键代码，完整的实现需要阅读源码。

### 3.1 节点发现

我们把上节课需求分析时列出的 TODO 贴出来，分别实现。

- 构造 Peer 时要执行`Swarm::add_external_address`，切换为 Server 模式；

```rs
// tinyp2p/src/service.rs

impl<E: EventHandler> Server<E> {
    pub fn new(
        config: P2pConfig,
        cmd_receiver: UnboundedReceiver<Command>,
    ) -> Result<Self, P2pError> {
        // ...

        let mut swarm = {
            let transport = transport::build_transport(local_key.clone());
            let behaviour = Behaviour::new(local_key, pubsub_topics.clone(), config.req_resp)?;
            SwarmBuilder::with_tokio_executor(transport, behaviour, local_peer_id).build()
        };
        // 这一行切换到Server模式
        swarm.add_external_address(addr.clone());
        swarm.listen_on(addr)?;

        // ...
    }
}
```

- 定期执行 Bootstrap;

```rs
// tinyp2p/src/service.rs

impl<E: EventHandler> Server<E> {
    pub async fn run(mut self) {
        loop {
            select! {
                // 定期执行节点发现
                _ = self.discovery_ticker.tick() => {
                    self.network_service.behaviour_mut().discover_peers();
                },

                // ...
            }
        }
    }
}
```

- Identify 收到节点信息时将地址加入 DHT;

```rs
// tinyp2p/src/service.rs

impl<E: EventHandler> Server<E> {
    fn handle_behaviour_event(&mut self, ev: BehaviourEvent) {
        match ev {
            // ...
            BehaviourEvent::Identify(identify::Event::Received {
                peer_id,
                info: identify::Info { listen_addrs, .. },
            }) => self.add_addresses(&peer_id, listen_addrs),
            // ...
        }
    }
}
```

- 收到“建立连接失败”事件时手动移除该节点；

```rs
// tinyp2p/src/service.rs

impl<E: EventHandler> Server<E> {
    fn handle_swarm_event(&mut self, event: SwarmEvent<BehaviourEvent, BehaviourErr>) {
        match event {
            // ...
            SwarmEvent::OutgoingConnectionError {
                peer_id: Some(peer),
                ..
            } => return self.network_service.behaviour_mut().remove_peer(&peer),
            // ...
        };
    }
}
```

- 收到“Ping 失败”事件时手动移除该节点

```rs
// tinyp2p/src/service.rs

impl<E: EventHandler> Server<E> {
    fn handle_behaviour_event(&mut self, ev: BehaviourEvent) {
        match ev {
            // ...
            BehaviourEvent::Ping(ping::Event {
                peer,
                result: Err(_),
                ..
            }) => self.network_service.behaviour_mut().remove_peer(&peer),
            // ...
        }
    }
}
```

### 3.2 向远端发送同步请求

1. 一个同步请求的 cmd 是这么定义的：

```rs
// tinyp2p/src/service.rs

pub enum Command {
    SendRequest {
        target: PeerId,
        request: Vec<u8>,
        // 在收到远端响应时，通过这个Sender将响应返回
        responder: oneshot::Sender<ResponseType>,
    },
    // ...
}
```

2. 向远端发送请求，并不会立刻得到响应，而是先返回一个 RequestID，将这个 RequestID 和 Responder 关联起来

```rs
// tinyp2p/src/service.rs

fn handle_outbound_request(
    &mut self,
    target: PeerId,
    request: Vec<u8>,
    responder: oneshot::Sender<ResponseType>,
) {
    let req_id = self
        .network_service
        .behaviour_mut()
        .send_request(&target, request);
    self.pending_outbound_requests.insert(req_id, responder);
}
```

3. 收到来自远端的响应，通过 RequestID 取出 Responder，再通过 Responder 将数据返回给 p2p_client

```rs
// tinyp2p/src/service.rs

fn handle_inbound_response(&mut self, request_id: RequestId, response: ResponseType) {
    if let Some(responder) = self.pending_outbound_requests.remove(&request_id) {
        let _ = responder.send(response);
    } else {
        warn!("❗ Received response for unknown request: {}", request_id);
        debug_assert!(false);
    }
}
```

### 3.3 收到来自远端的请求

调用 trait EventHandler 中的接口处理；调用 request-response 中的接口将 response 返回给远端

```rs
// tinyp2p/src/service.rs

fn handle_inbound_request(&mut self, request: Vec<u8>, ch: ResponseChannel<ResponseType>) {
    if let Some(handler) = self.event_handler.get() {
        let response = handler.handle_inbound_request(request).map_err(|_| ());
        self.network_service
            .behaviour_mut()
            .send_response(ch, response);
    }
}
```

### 3.4 向网络广播消息

调用 gossipsub 的接口。

```rs
// tinyp2p/src/service.rs

fn handle_outbound_broadcast(&mut self, topic: String, message: Vec<u8>) {
    let _ = self
        .network_service
        .behaviour_mut()
        .broadcast(topic, message);
}
```

### 3.5 收到网络中的广播消息

调用 trait EventHandler 中的接口处理。

```rs
// tinyp2p/src/service.rs

fn handle_inbound_broadcast(&mut self, message: gossipsub::Message) {
    if let Some(handler) = self.event_handler.get() {
        let topic_hash = message.topic;
        match self.get_topic(&topic_hash) {
            Some(topic) => handler.handle_broadcast(&topic, message.data),
            None => {
                warn!("❗ Received broadcast for unknown topic: {:?}", topic_hash);
                debug_assert!(false);
            }
        }
    }
}
```

## 4 功能演示

> 功能演示源码：`tinyp2p/examples/main.rs`

1. 在根目录运行：`RUST_LOG=DEBUG cargo run -p tinyp2p --example main`，在日志中找到该节点的 PeerID 和绑定的地址；

```log
INFO  tinyp2p::service > 📣 Local peer id: PeerId("12D3KooWCQwu2jCgGvSHabjMLE7YkxocuRkAB5vYo2i1sU9MdMN2")
INFO  tinyp2p::service > 📣 P2P node listening on "/ip4/172.28.132.160/tcp/35229"
```

2. 新开一个 Terminal，同样在项目根目录运行：`RUST_LOG=DEBUG cargo run -p tinyp2p --example main /ip4/172.28.132.160/tcp/35229/p2p/12D3KooWCQwu2jCgGvSHabjMLE7YkxocuRkAB5vYo2i1sU9MdMN2`。命令行参数是上面日志中查到的地址和 PeerID。从日志中可以找到节点发现、请求响应、广播消息的日志。

```log
DEBUG tinyp2p::protocol > ☕ Adding address /ip4/172.28.132.160/tcp/35229 from PeerId("12D3KooWCQwu2jCgGvSHabjMLE7YkxocuRkAB5vYo2i1sU9MdMN2") to the DHT.
INFO  main              > 📣 >>>> Outbound request: "Hello, request!"
INFO  main              > 📣 <<<< Inbound response: "Hello, request!"
INFO  main              > 📣 <<<< Inbound broadcast: "block" "Hello, a new block!"
```

3. 关闭其中一个节点，查看另一个节点的日志，可以看到关闭的节点已从 DHT 中移除。

```log
DEBUG libp2p_swarm      > Connection attempt to PeerId("12D3KooWCQwu2jCgGvSHabjMLE7YkxocuRkAB5vYo2i1sU9MdMN2") failed with ... message: "Connection refused").
DEBUG tinyp2p::protocol > ☕ Removing peer 12D3KooWCQwu2jCgGvSHabjMLE7YkxocuRkAB5vYo2i1sU9MdMN2 from the DHT.
```

## 5 小结

我们在本课不仅实现了上节课的需求，还掌握了 CSP 无锁并发模型。通过`tinyp2p/examples/main.rs`可以看出来，相比于 libp2p 来说，tinyp2p 的使用非常简单，后面的工作可以轻松很多了 🎉🎉🎉

---

| [< 06-libp2p: 需求分析与封装思路](./06-libp2p.md) | [08-网络层 >](./08-network.md) |
| ------------------------------------------------- | ------------------------------ |
