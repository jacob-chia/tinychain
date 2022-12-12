use async_trait::async_trait;
use log::info;
use reqwest::Client;
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

#[async_trait]
impl Peer for HttpPeer {
    async fn ping(&self, my_addr: &str, peer_addr: &str) -> Result<(), ChainError> {
        let url = format!("http://{}/peers", peer_addr);

        let mut body = HashMap::new();
        body.insert("addr", my_addr);

        let res = self.post(url).json(&body).send().await?;
        info!("<<< response: {:?}", res);

        Ok(())
    }
}
