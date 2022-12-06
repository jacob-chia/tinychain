use database::*;

struct Peer {
    ip: String,
    port: u16,
    account: String,
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
