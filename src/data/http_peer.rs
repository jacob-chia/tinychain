use log::info;
use reqwest::blocking::Client;
use std::{
    collections::HashMap,
    ops::{Deref, DerefMut},
};

use crate::{error::ChainError, node::Peer};

#[derive(Debug)]
pub struct HttpPeer(Client);

impl HttpPeer {
    pub fn new() -> Self {
        Self(Client::new())
    }
}

// TODO
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

        let res = self.post(url).json(&body).send()?;
        info!("<<< response: {:?}", res);

        Ok(())
    }
}
