use crate::errors::NetworkErrors;
use crate::in_memory_cluster::InMemoryNetworkClient;
use crate::in_memory_cluster::ServiceMethod::AppendEntries;
use serde::{Deserialize, Serialize};
use serde_json::to_value;
use std::future::Future;
use std::time::Duration;

pub struct RaftNode {
    network_client: InMemoryNetworkClient,
    id: usize,
    stop_sender: Option<tokio::sync::mpsc::Sender<()>>,
}

impl RaftNode {
    pub fn new(id: usize, network_client: InMemoryNetworkClient) -> Self {
        RaftNode {
            id,
            network_client,
            stop_sender: None,
        }
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

    pub async fn start(&mut self) -> impl Future<Output = ()> + '_ + Send {
        let (stop_sender, mut stop_receiver) = tokio::sync::mpsc::channel(1);
        self.stop_sender = Some(stop_sender);
        let id = self.id;
        async move {
            println!("starting background server");
            let mut interval = tokio::time::interval(Duration::from_secs(1)); // adjust as needed
            loop {
                tokio::select! {
                    _ = interval.tick() => {
                        if id == 1 {
                            println!("sending request");
                            // Note: the error handling is omitted for brevity
                            let reply = self.send_append_entries(0, AppendEntriesRequest {}).await.unwrap();
                            println!("{} success", id)
                        }
                    }
                    _ = stop_receiver.recv()=> {
                        println!("Stopping");
                        break;
                    }
                }
            }
        }
    }

    pub async fn stop(&mut self) {
        if let Some(stop_sender) = self.stop_sender.take() {
            stop_sender.send(()).await.unwrap();
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct AppendEntriesRequest {}

#[derive(Serialize, Deserialize)]
pub struct AppendEntriesReply {}
