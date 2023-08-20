- [06 | Thinking in Libp2p](#06--thinking-in-libp2p)
  - [1 What is libp2p?](#1-what-is-libp2p)
    - [1.1 Layering](#11-layering)
    - [1.2 Transport](#12-transport)
    - [1.3 Protocols](#13-protocols)
    - [1.4 Swarm](#14-swarm)
  - [2 How to extend libp2p?](#2-how-to-extend-libp2p)
    - [2.1 Put Business Logic in Custom Protocol](#21-put-business-logic-in-custom-protocol)
      - [Version 1: Use `behaviour(ignore)` macro](#version-1-use-behaviourignore-macro)
      - [Version 2: Implement `NetworkBehaviour` manually](#version-2-implement-networkbehaviour-manually)
    - [2.2 Put Business Logic in SwarmEventHandler](#22-put-business-logic-in-swarmeventhandler)
  - [3 Which libp2p modules should we use?](#3-which-libp2p-modules-should-we-use)
    - [3.1 Peer Discovery](#31-peer-discovery)
      - [(1) Kademlia Bootstrap](#1-kademlia-bootstrap)
      - [(2) Should be used with Identify](#2-should-be-used-with-identify)
      - [(3) Should be used with Ping](#3-should-be-used-with-ping)
      - [(4) Need to switch to Server mode](#4-need-to-switch-to-server-mode)
      - [(5) Summary](#5-summary)
    - [3.2 Request/Response](#32-requestresponse)
    - [3.3 Broadcast](#33-broadcast)
  - [4 Summary](#4-summary)

# 06 | Thinking in Libp2p

> - Repo: `https://github.com/jacob-chia/tinychain.git`
>
> Important crates and docs related to this lesson:
>
> - [rust-libp2p](https://docs.rs/libp2p/latest/libp2p/index.html): a modular peer-to-peer networking framework.
> - [libp2p concepts](https://docs.libp2p.io/concepts/introduction/overview/)：A lot of concepts about libp2p are introduced in this doc.
> - [libp2p spec](https://github.com/libp2p/specs)：The official spec of libp2p.

In the next two lessons, we will build a tinychain-specific P2P network library based on rust-libp2p, including the following components:

- **Peer Discovery**: Responsible for discovering nodes on the network.
- **Request/Response**: Responsible for interacting with other nodes on the network.
- **Broadcast**: Responsible for broadcasting messages to other nodes on the network.

And to achieve these requirements, we need to answer the following questions:

1. What is libp2p?
2. How to extend libp2p?
3. Which libp2p modules should we use?

## 1 What is libp2p?

### 1.1 Layering

![](../img/06-libp2p-architecture.png)

[libp2p concepts](https://docs.libp2p.io/concepts/introduction/overview/) is a must-read, and most of the new concepts mentioned below are in this doc. In a nutshell, libp2p can be divided into three layers:

- `Transport`: responsible for data transmission.
- `Protocols`: responsible for data processing.
- `Swarm`: responsible for combining `Transport` and `Protocols` together.

### 1.2 Transport

The `Transport` in libp2p supports a variety of configurations, including:

- Underlying network protocol: `TCP` / UDP / QUIC, etc.
- Security protocol: TLS 1.3 / `Noise`.
- Stream Multiplexing: `Yamux` / mplex (removed since libp2p-0.52).

In the next lesson, we will use `TCP` + `Noise` + `Yamux` to build the `Transport`. For those who are not familiar with Noise and Yamux, here is a brief introduction:

- Compared with TLS, Noise is a more lightweight security protocol, and it **does not require a CA certificate**.
- Yamux can **simulate multiple streams on a single TCP connection**. For example, if a P2P node needs to use many protocols, including Kademlia, Identify, Gossipsub, etc., then Yamux can simulate multiple streams on a single TCP connection, each stream handling a protocol.

  ![](../img/06-yamux.png)

### 1.3 Protocols

There are many official protocols defined in `rust-libp2p`, and we can also implement our own protocols, which is one of the ways to extend libp2p.
And a protocol contains two parts: `Behaviour` and `BehaviourEvent`. We need `Behaviour` when constructing a Swarm, and we need to handle `BehaviourEvent` when processing SwarmEvent.

The simplest way to implement a protocol is to use the `#[derive(NetworkBehaviour)]` macro, for example:

```rs
// my_protocol.rs

#[derive(NetworkBehaviour)]
pub struct Behaviour {
    ping: ping::Behaviour,
    identify: identify::Behaviour,
}
```

The above code will automatically implement the `NetworkBehaviour` trait for `Behaviour`, and generate the corresponding `BehaviourEvent` in this module. In other words, this macro can turn this module into a Protocol.

### 1.4 Swarm

Let's see how swarm works through some pseudo code:

```rs
let local_key = config.gen_keypair()?;
let local_peer_id = local_key.public().to_peer_id();

// Create a transport
let transport = transport::build_transport(local_key.clone());
// [NOTE 1] Create a custom behaviour (Actually, we should define a custom protocol first)
let behaviour = custom_protocol::Behaviour::new(local_key)?;
// Create a swarm
SwarmBuilder::with_tokio_executor(transport, behaviour, local_peer_id).build()
// Listen on an address
swarm.listen_on(addr)?;

// Handle SwarmEvent
while let event = swarm.select_next_some() {
    // [NOTE 2] Handle BehaviourEvent
}
```

Through the above code, we can not only see the construction process of a P2P node, but also see two places where we can add our own logic:

- **[NOTE 1]**: We can define a protocol in the `Protocol` layer.
- **[NOTE 2]**: We can handle the events we care about in the `Swarm` layer.

But, which one is better? I see the `Protocol` as the **business** layer, and the `Swarm` as the **control** layer. So the business logic should be placed in the business layer, right? What's the problem with this approach?

## 2 How to extend libp2p?

> tldr: It's very easy to define a custom protocol by composing several official protocols, but it's hard to add custom logic to it. So I recommend to put our business logic in the SwarmEvent Handler.

### 2.1 Put Business Logic in Custom Protocol

At first I thought this was the certain way to go, so I tried it.

#### Version 1: Use `behaviour(ignore)` macro

```rs
// Simply put, the `#[derive(NetworkBehaviour)]` can turn this module into a Protocol.
#[derive(NetworkBehaviour)]
pub struct Behaviour {
    // An official protocol to handle request/response
    req_resp: request_response::Behaviour<GenericCodec>,
    // Other official protocols

    // A field to store the state of the protocol, which should be ignored by the macro
    #[behaviour(ignore)]
    pending_outbound_requests: HashMap<RequestId, oneshot::Sender<ResponseType>>,
}

// Other code omitted
```

Unfortunately, this approach doesn't work. And I found the CHANGELOG in [swarm-derive/CHANGELOG.md](https://github.com/libp2p/rust-libp2p/blob/master/swarm-derive/CHANGELOG.md#0300)

```md
- Remove support for non-`NetworkBehaviour` fields on main `struct` via `#[behaviour(ignore)]`. See
  [PR 2842].

[PR 2842]: https://github.com/libp2p/rust-libp2p/pull/2842
```

#### Version 2: Implement `NetworkBehaviour` manually

> Until 2023-06-20 (before libp2p-v0.52.0 is released), the following code is OK.

Through version 1, we know that `#[derive(NetworkBehaviour)]` can turn a struct into a NetworkBehaviour, but all fields in the struct must be a sub-behaviour, and cannot contain other fields.
So we need put our logic into a sub-procotol, e.g. `req_resp` (a custom request/response protocol).

By reading the source code of request_response, we can know that to define a protocol, we need to do the following:

- Define the `Event` of the protocol;
- Define the `struct Behaviour` of the protocol;
- Implement the `trait NetworkBehaviour` for the `Behaviour`;
- (Optional) Define the `struct Handler` of the protocol;
- (Optional) Implement the `trait ConnectionHandler` for the `Handler`.

So, we can define our own protocol like this:

```rs
// my_protocol.rs

// There is no custom fields other than the sub-behaviours
#[derive(NetworkBehaviour)]
pub struct Behaviour {
    req_resp: req_resp::Behaviour,
    // Other sub-behaviours
}
```

And we can define the `req_resp` protocol like this:

```rs
// req_resp.rs

pub enum Event { /* ... */ };

pub struct Behaviour {
    // The real request/response behaviour
    inner: request_response::Behaviour<GenericCodec>,

    // Our own logic
    pending_outbound_requests: HashMap<RequestId, oneshot::Sender<ResponseType>>,
}

impl NetworkBehaviour for Behaviour {
    // The Handler from the official request_response protocol
    type ConnectionHandler = request_response::Handler;

    // The custom `Event`
    type OutEvent = Event;

    // Other code omitted
}
```

Note that the key code in the above is `type ConnectionHandler = request_response::Handler;`, which saves a lot of work, and we only need to implement the `trait NetworkBehaviour` ourselves.
This approach works until I update the libp2p version to `v0.52.0` (released on 2023-06-20). I found that the official `Handlers` have become private, and if we want to define a protocol, we have to do it from scratch, even if we only need to add a little custom logic to the official protocol. I think this is not the right way to go.

> The motivation for libp2p to make handlers private may be this [issue](https://github.com/libp2p/rust-libp2p/issues/3608)
>
> To have the ability to refactor implementation details without causing breaking changes.

### 2.2 Put Business Logic in SwarmEventHandler

It's much easier to do it this way, and we don't need to use code to explain it. The process is as follows:

1. Select the official protocols you need and combine them into a struct;
2. Use `#[derive(NetworkBehaviour)]` to decorate this struct;
3. Check the docs/source code to see which events the protocols you use have, and find out the events that need to be handled;
4. Add the event handler when `while let event = swarm.select_next_some()` in the Swarm layer;

So, we will use this approach to build the tinyp2p in the next lesson.

## 3 Which libp2p modules should we use?

### 3.1 Peer Discovery

#### (1) Kademlia Bootstrap

Kademlia is one of the most important protocols in libp2p and a little complicated, but for this project, we only need to know one concept and one function:

- One concept: [What is Kad-DHT?](https://docs.libp2p.io/concepts/discovery-routing/kaddht/)
- One function: [Bootstrap](https://docs.rs/libp2p/latest/libp2p/kad/struct.Kademlia.html#method.bootstrap)

In a nutshell, the Kademlia Distributed Hash Table (DHT), or Kad-DHT, is a distributed hash table that offers a way to find nodes on the network. And the Bootstrap function is used to maintain the Kad-DHT.

During the Bootstrap process, Kademlia will try to discover new nodes and add them to the Kad-DHT, which is easy to understand, but what will Kademlia do when a node cannot be connected? Let's take a look at the source code of rust-libp2p:

```rs
// https://github.com/libp2p/rust-libp2p/blob/master/protocols/kad/src/behaviour.rs#L1765

fn address_failed(&mut self, peer_id: PeerId, address: &Multiaddr) {
    let key = kbucket::Key::from(peer_id);

    if let Some(addrs) = self.kbuckets.entry(&key).value() {
        // TODO: Ideally, the address should only be removed if the error can
        // be classified as "permanent" but since `err` is currently a borrowed
        // trait object without a `'static` bound, even downcasting for inspection
        // of the error is not possible (and also not truly desirable or ergonomic).
        // The error passed in should rather be a dedicated enum.
        if addrs.remove(address).is_ok() {
            debug!(
                "Address '{}' removed from peer '{}' due to error.",
                address, peer_id
            );
        } else {
            // Despite apparently having no reachable address (any longer),
            // the peer is kept in the routing table with the last address to avoid
            // (temporary) loss of network connectivity to "flush" the routing
            // table. Once in, a peer is only removed from the routing table
            // if it is the least recently connected peer, currently disconnected
            // and is unreachable in the context of another peer pending insertion
            // into the same bucket. This is handled transparently by the
            // `KBucketsTable` and takes effect through `KBucketsTable::take_applied_pending`
            // within `Kademlia::poll`.
            debug!(
                "Last remaining address '{}' of peer '{}' is unreachable.",
                address, peer_id,
            )
        }
    }
    // ...
}

// The `remove` function that the above code calls
pub fn remove(&mut self, addr: &Multiaddr) -> Result<(), ()> {
    if self.addrs.len() == 1 {
        return Err(());
    }

    if let Some(pos) = self.addrs.iter().position(|a| a == addr) {
        self.addrs.remove(pos);
        if self.addrs.len() <= self.addrs.inline_size() {
            self.addrs.shrink_to_fit();
        }
    }

    Ok(())
}
```

We can find that when a node cannot be connected, Kademlia will try to remove the address from the Peer, but at least one address will be retained, even if this address is unreachable. In other words, Kademlia will not remove the unreachable node. But for our project, we need to see the dynamic changes of the DHT, so we need to manually remove the node when it cannot be connected.

This error is represented by the `SwarmEvent::OutgoingConnectionError` event, and we can handle it like this:

```rs
SwarmEvent::OutgoingConnectionError { peer_id: Some(peer), .. } => return remove_peer(&peer),
```

#### (2) Should be used with Identify

> [Identify Doc](https://docs.libp2p.io/concepts/fundamentals/protocols/#identify)
> Simply put, identify is used to exchange node information, including address information.

Let's take a look at the usage instructions of Kademlia in the rust-libp2p documentation:

```md
// https://docs.rs/libp2p/latest/libp2p/kad/index.html#important-discrepancies

Peer Discovery with Identify In other libp2p implementations, the Identify protocol might be seen as a core protocol.
Rust-libp2p tries to stay as generic as possible, and does not make this assumption.
This means that the Identify protocol must be manually hooked up to Kademlia through calls to `Kademlia::add_address`.
If you choose not to use the Identify protocol, and do not provide an alternative peer discovery mechanism,
a Kademlia node will not discover nodes beyond the network’s boot nodes. Without the Identify protocol,
existing nodes in the kademlia network cannot obtain the listen addresses of nodes querying them, and thus will not be
able to add them to their routing table.
```

In other words, the Bootstrap process will only try to discover new nodes and establish connections with them, but will not add them to the DHT. We need to rely on the Identify protocol to explicitly call `Kademlia::add_address` to add them to the DHT when exchanging node information (including addresses).

#### (3) Should be used with Ping

Let's review what we said when we talked about Bootstrap in the previous section: We should handle the `SwarmEvent::OutgoingConnectionError` event and manually remove the node when it cannot be connected. But what if the connection is established, but the node does not respond? We can't find out this error through the `SwarmEvent::OutgoingConnectionError` event, but we can find it through the Ping protocol by doing this:

```rs
BehaviourEvent::Ping(ping::Event { peer, result: Err(_), .. }) => remove_peer(&peer),
```

#### (4) Need to switch to Server mode

> [Server mode doc](https://github.com/libp2p/specs/tree/master/kad-dht#client-and-server-mode)
> Simply put, only Peers in Server mode will join the DHT, and Peers in Client mode can only access the DHT, but cannot join it.

This is a pitfall! After updating the version of libp2p, I found that Kademlia didn't work, and I couldn't find the cause by checking the logs and source code. So I went to the release note and Changelog.

Let's take a look at the [release note of libp2p-v0.52.0](https://github.com/libp2p/rust-libp2p/releases/tag/libp2p-v0.52.0) first.

```md
## Automatic kademlia client/server mode

Let's get the biggest one out the way first, I promise the other points are easier explained but equally exciting.
The tl;dr is: Healthier Kademlia routing tables and an improved developer experience.

If you don't know about Kademlia's client/server mode, checkout the specs.

With the v0.52 release, rust-libp2p automatically configures Kademlia in client or server mode depending on our
external addresses. If we have a confirmed, external address, we will operate in server-mode, otherwise client-mode.
This is entirely configuration-free (yay!) although follow-up work is under-way to allow setting this manually
in certain situations: #4074.
```

In my development environment, there is no public IP, so libp2p automatically starts in Client mode, and it is not configurable yet! (I almost rolled back the version when I saw this.)
Then I looked at the Changelog of Kademlia, which is described as follows:

```md
<!-- https://github.com/libp2p/rust-libp2p/blob/master/protocols/kad/CHANGELOG.md#0440 -->

Automatically configure client/server mode based on external addresses. If we have or learn about an external
address of our node, e.g. through `Swarm::add_external_address` or automated through libp2p-autonat, we operate
in server-mode and thus allow inbound requests. By default, a node is in client-mode and only allows outbound requests.
If you want to maintain the status quo,
i.e. **always operate in server mode, make sure to add at least one external address through `Swarm::add_external_address`**.
See also Kademlia specification for an introduction to Kademlia client/server mode. See PR 3877.
```

Fortunately, we can still switch to Server mode by calling `Swarm::add_external_address`.

#### (5) Summary

In order to do the Peer Discovery, we need to do five things:

- When constructing a `Peer`, we need to execute `Swarm::add_external_address` to switch to the `Server` mode;
- Call bootstrap regularly;
- Add the address to the DHT when `Identify` receives node information;
- Manually remove the node when the "Outgoing Connection Error" event is received;
- Manually remove the node when the "Ping Error" event is received.

### 3.2 Request/Response

We use the official request-response protocol to interact with other nodes, and we only need to do one thing:

- Define a `Codec` to specify how to send/receive a complete message. Our message is a protobuf-encoded binary stream, so we need to specify the length of the binary stream.

### 3.3 Broadcast

This can be done using the gossipsub protocol, which is very easy to use, just use two functions:

- `subscribe(topic)`: Subscribe to a topic
- `publish(topic, msg)`: Publish a message to nodes that subscribe to this topic

## 4 Summary

This lesson is a little long, but the purpose is clear. We found the answers to the following questions:

1. What is libp2p?

- libp2p can be divided into three layers: `transport`, `swarm`, `protocol`
- The steps to run a P2P node: `build transport` -> `build protocol` -> `build swarm` -> `run swarm` -> `handle swarm events`

2. How to extend libp2p?

- `Define custom protocol`: a lot of work, may need to re-implement the official protocols from scratch
- `Define custom swarm event handlers`: easier, just find out which events need to be handled, and handle them

3. Which libp2p modules should we use?

- Peer Discovery: `kademlia`, `identify`, `ping`
- Request/Response: `request-response`
- Broadcast: `gossipsub`

---

| [< 05-Command Line & Config File](./05-cmd-config.md) | [07-tinyp2p：Tinyp2p: A CSP Concurrency Model >](./07-tinyp2p.md) |
| ----------------------------------------------------- | ----------------------------------------------------------------- |
