use std::{
    collections::HashMap,
    ops::{Deref, DerefMut},
};

use reqwest::{blocking::Client, Url};

use crate::{error::ChainError, node::Peer};

#[derive(Debug)]
pub struct HttpPeer(Client);

impl HttpPeer {
    pub fn new() -> Self {
        Self(Client::new())
    }
}

/// Deref/DerefMut 可以让 HttpPeer 直接使用 Client 的方法。
/// 比如 self.get(url)。
impl Deref for HttpPeer {
    type Target = Client;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for HttpPeer {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Peer for HttpPeer {
    fn ping(&self, my_addr: &str, peer_addr: &str) -> Result<(), ChainError> {
        let url = format!("http://{}/peer/ping", peer_addr);

        let mut body = HashMap::new();
        body.insert("addr", my_addr);

        self.post(url).json(&body).send()?;

        Ok(())
    }

    fn get_status(&self, peer_addr: &str) -> Result<crate::node::PeerStatus, ChainError> {
        let url = format!("http://{}/peer/status", peer_addr);
        let resp = self.get(url).send()?.json()?;

        Ok(resp)
    }

    fn get_blocks(
        &self,
        peer_addr: &str,
        offset: u64,
    ) -> Result<Vec<crate::node::Block>, ChainError> {
        let url = format!("http://{}/blocks", peer_addr);
        let params = [("offset", offset.to_string())];
        let url = Url::parse_with_params(&url, &params).unwrap();

        let resp = self.get(url).send()?.json()?;

        Ok(resp)
    }
}
