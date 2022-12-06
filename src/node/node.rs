use database::*;

struct Peer {
    ip: &'static str,
    port: u16,
    account: &'static str,
    is_bootstrap: bool,
    connected: bool,
}

struct Node {
    info: Peer,
    state: Box<State>,
    known_peers: HashMap<String, Peer>,
    pending_txs: HashMap<String, SignedTx>,
    archived_txs: HashMap<String, SignedTx>,
}
