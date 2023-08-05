use crate::errors::NetworkErrors;
use crate::in_memory_network::ServiceMethod::AppendEntries;
use crate::in_memory_network::{InMemoryNetworkClient, NetworkRequest, ServiceMethod};
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

    pub fn handle_append_entries(request: AppendEntriesRequest) -> AppendEntriesReply {
        AppendEntriesReply {}
    }

    // TODO: should be able to make generic method that handles the nitty gritty request details
    //       let reply_channel = await self.client.send_request<AppendEntriesRequest>("hostname");
    //       let reply_channel = await self.client.broadcast_request<AppendEntriesRequest>();
    //       self.client.peers for access to each individual node;
    pub async fn send_append_entries(
        &self,
        destination: usize,
        request: AppendEntriesRequest,
    ) -> Result<AppendEntriesReply, NetworkErrors> {
        let raw_request = to_value(&request)?;
        let network_reply: AppendEntriesReply = self
            .network_client
            .send_request(self.id, destination, AppendEntries, raw_request)
            .await?;
        Ok(network_reply)
    }
}

#[derive(Serialize, Deserialize)]
pub struct AppendEntriesRequest {}

#[derive(Serialize, Deserialize)]
pub struct AppendEntriesReply {}
