use crate::errors::NetworkErrors;
use crate::in_memory_cluster::ServiceMethod::AppendEntries;
use crate::in_memory_cluster::{InMemoryNetworkClient, NetworkRequest, ServiceMethod};
use serde::{Deserialize, Serialize};
use serde_json::to_value;

pub struct RaftNode {
    network_client: InMemoryNetworkClient,
    id: usize,
}

impl RaftNode {
    pub fn new(id: usize, network_client: InMemoryNetworkClient) -> Self {
        RaftNode { id, network_client }
    }

    pub async fn handle_append_entries(&self, request: AppendEntriesRequest) -> AppendEntriesReply {
        println!("{} received append entries request", self.id);
        AppendEntriesReply {}
    }

    pub async fn send_append_entries(
        &self,
        destination: usize,
        request: AppendEntriesRequest,
    ) -> Result<AppendEntriesReply, NetworkErrors> {
        println!("{} sending append entries request", self.id);
        let raw_request = to_value(&request)?;
        let reply: AppendEntriesReply = self
            .network_client
            .send_request(self.id, destination, AppendEntries, raw_request)
            .await?;
        println!("{} received append entries reply", self.id);
        Ok(reply)
    }

    pub async fn start(&self) {
        if self.id == 1 {
            println!("sending request");
            let reply = self.send_append_entries(0, AppendEntriesRequest {}).await;
            if reply.is_ok() {
                println!("{} success", self.id)
            }
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct AppendEntriesRequest {}

#[derive(Serialize, Deserialize)]
pub struct AppendEntriesReply {}
