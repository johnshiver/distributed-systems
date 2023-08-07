use crate::errors::NetworkErrors;
use crate::raft_node::{AppendEntriesRequest, RaftNode};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use serde_json::{to_value, Value};
use std::ops::Deref;
use std::sync::Arc;
use tokio::select;
use tokio::sync::{mpsc, Mutex};
use tokio::task::JoinHandle;
use tokio::time::{timeout, Duration};
use tokio_util::sync::CancellationToken;

pub struct InMemoryCluster {
    reliable: bool,
    long_delays: bool,
    token: CancellationToken,
    servers: Arc<Mutex<Vec<Server>>>,
    network_connections: Arc<Mutex<Vec<Vec<bool>>>>, // e.g. network_connections[x][y] == true means server x can talk to server y
    receive_requests: Arc<Mutex<mpsc::Receiver<NetworkRequest>>>,
    send_requests: Arc<Mutex<mpsc::Sender<NetworkRequest>>>,
}

impl InMemoryCluster {
    pub fn new(reliable: bool, long_delays: bool, server_count: i8) -> Self {
        let (send, recv) = mpsc::channel(1);
        let sender = Arc::new(Mutex::new(send));
        let receiver = Arc::new(Mutex::new(recv));

        // all servers are initially connected
        let network_connections = Arc::new(Mutex::new(vec![
            vec![true; server_count as usize];
            server_count as usize
        ]));

        let token = CancellationToken::new();

        // init servers
        let servers = Arc::new(Mutex::new(
            (0..server_count)
                .map(|i| {
                    Server::new(
                        i as usize,
                        InMemoryNetworkClient::new(server_count, sender.clone()),
                        token.clone(),
                    )
                })
                .collect::<Vec<_>>(),
        ));

        InMemoryCluster {
            reliable,
            long_delays,
            servers,
            receive_requests: receiver,
            send_requests: sender,
            network_connections,
            token,
        }
    }

    pub async fn servers_are_connected(&self, sender: usize, recipient: usize) -> bool {
        let connections = self.network_connections.lock().await;
        if sender >= connections.len() || recipient >= connections.len() {
            false
        } else {
            connections[sender][recipient]
        }
    }

    pub async fn add_server(&self, server: usize) -> bool {
        let mut servers = self.network_connections.lock().await;
        if server >= servers.len() {
            return false;
        }
        for (i, connections) in servers.iter_mut().enumerate() {
            for (j, is_connected) in connections.iter_mut().enumerate() {
                if i == server || j == server {
                    *is_connected = true;
                }
                println!(
                    "Connection status from server {} to server {}: {}",
                    i, j, *is_connected
                );
            }
        }
        true
    }

    // Removes a server from the network and update the network_connections matrix.
    pub async fn remove_server(&self, server: usize) -> bool {
        let mut servers = self.network_connections.lock().await;
        if server >= servers.len() {
            return false;
        }
        for (i, connections) in servers.iter_mut().enumerate() {
            for (j, is_connected) in connections.iter_mut().enumerate() {
                if i == server || j == server {
                    *is_connected = false;
                }
                println!(
                    "Connection status from server {} to server {}: {}",
                    i, j, *is_connected
                );
            }
        }
        true
    }

    pub async fn stop(&self) {
        self.token.cancel();
    }

    pub async fn start(self: Arc<Self>) -> JoinHandle<()> {
        {
            let mut servers = self.servers.lock().await;
            for server in servers.iter() {
                println!("Starting server");
                Arc::new(server).start().await;
            }
        }

        let cloned_token = self.token.clone();
        let join_handle = tokio::spawn(async move {
            let mut locked_receive_requests = self.receive_requests.lock().await;
            select! {
                _ = cloned_token.cancelled() => {
                    // The token was cancelled
                    println!("token was canceled");
                }
                Some(req) = locked_receive_requests.recv() => {
                    if !self
                        .servers_are_connected(req.origin_server, req.destination_server)
                        .await
                    {
                    println!("servers not connected");
                    let _ = req
                        .send_reply
                        .lock()
                        .await
                        .send(NetworkReply {
                            ok: false,
                            reply: Default::default(),
                        })
                        .await;
                } else {
                    println!("processing request");
                    let self_clone = Arc::clone(&self);
                    tokio::spawn(async move {
                        self_clone.process_request(req.clone()).await;
                    });
                }
                }
            }
        });
        join_handle
    }

    // if you want to send requests to the network use NetworkClient
    pub async fn create_client(&self) -> InMemoryNetworkClient {
        InMemoryNetworkClient::new(
            self.network_connections.lock().await.len() as i8,
            self.send_requests.clone(),
        )
    }

    pub async fn process_request(&self, request: NetworkRequest) {
        println!("processing request");
        // let servers = self.servers.lock().await;
        // if let Some(target_server) = servers.get(request.destination_server) {
        //     target_server.dispatch(request).await;
        // } else {
        //     // log something / send network request 404
        // }
    }
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub enum ServiceMethod {
    AppendEntries,
}

pub struct InMemoryNetworkClient {
    send_requests: Arc<Mutex<mpsc::Sender<NetworkRequest>>>,
    peers: Vec<i8>,
}

impl InMemoryNetworkClient {
    pub fn new(peer_count: i8, sender: Arc<Mutex<mpsc::Sender<NetworkRequest>>>) -> Self {
        InMemoryNetworkClient {
            send_requests: sender,
            peers: vec![],
        }
    }

    pub async fn send_request<R, S>(
        &self,
        origin: usize,
        destination: usize,
        svc_method: ServiceMethod,
        request: R,
    ) -> Result<S, NetworkErrors>
    where
        R: Serialize,
        S: DeserializeOwned,
    {
        let raw_request = to_value(&request)?;
        let network_request = NetworkRequest::new(origin, destination, svc_method, raw_request);
        let network_reply = self.send_raw_request(network_request).await?;
        if !network_reply.ok {
            return Err(NetworkErrors::Timeout);
        }
        let reply: Result<S, serde_json::Error> = serde_json::from_value(network_reply.reply);
        match reply {
            Ok(reply) => Ok(reply),
            Err(err) => Err(NetworkErrors::from(err)),
        }
    }

    async fn send_raw_request(
        &self,
        request: NetworkRequest,
    ) -> Result<NetworkReply, NetworkErrors> {
        self.send_requests
            .lock()
            .await
            .send(request.clone())
            .await?;
        return match timeout(
            Duration::from_secs(3),
            request.receive_reply.lock().await.recv(),
        )
        .await
        {
            Ok(Some(message)) => Ok(message),
            Ok(None) => Err(NetworkErrors::Unknown),
            Err(err) => {
                println!("received error {}", err);
                Err(NetworkErrors::Timeout)
            }
        };
    }
}

struct Server {
    raft: Arc<Mutex<RaftNode>>,
    token: CancellationToken,
}

impl Server {
    pub fn new(id: usize, network_client: InMemoryNetworkClient, token: CancellationToken) -> Self {
        Server {
            raft: Arc::new(Mutex::new(RaftNode::new(id, network_client))),
            token,
        }
    }

    pub async fn start(&self) {
        tokio::spawn(async move {
            let raft_clone = self.raft.clone();
            let _ = raft_clone.lock().await.start().await;
        });
    }

    pub async fn dispatch(&self, req: NetworkRequest) {
        match req.svc_method {
            ServiceMethod::AppendEntries => {
                let append_entries_request: AppendEntriesRequest =
                    serde_json::from_value(req.raw_request).unwrap();
                let response = self
                    .raft
                    .lock()
                    .await
                    .handle_append_entries(append_entries_request)
                    .await;
                let raw_response = to_value(response).unwrap();
                let _ = req
                    .send_reply
                    .lock()
                    .await
                    .send(NetworkReply::new(true, raw_response))
                    .await;
            }
        }
    }
}

#[derive(Clone)]
pub struct NetworkRequest {
    origin_server: usize,      // e.g. which server initiated the request
    destination_server: usize, // e.g. which server to send request to
    svc_method: ServiceMethod, // e.g. "Raft.AppendEntries"
    raw_request: Value,
    receive_reply: Arc<Mutex<mpsc::Receiver<NetworkReply>>>,
    send_reply: Arc<Mutex<mpsc::Sender<NetworkReply>>>,
}
impl NetworkRequest {
    pub fn new(
        origin: usize,
        destination: usize,
        method: ServiceMethod,
        raw_request: Value,
    ) -> Self {
        let (send, recv) = mpsc::channel(1);
        NetworkRequest {
            origin_server: origin,
            destination_server: destination,
            svc_method: method,
            raw_request,
            send_reply: Arc::new(Mutex::new(send)),
            receive_reply: Arc::new(Mutex::new(recv)),
        }
    }
}

#[derive(Clone, Deserialize)]
pub struct NetworkReply {
    pub ok: bool,
    pub reply: Value,
}

impl NetworkReply {
    pub fn new(ok: bool, raw_response: Value) -> Self {
        NetworkReply {
            ok,
            reply: raw_response,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_servers_are_connected() {
        let network = InMemoryCluster::new(true, false, 3);

        // Since the initial network_connections are set to true, servers 1 and 2 should be connected.
        assert_eq!(network.servers_are_connected(0, 1).await, true);
    }

    #[tokio::test]
    async fn test_servers_remove_and_add() {
        let network = InMemoryCluster::new(true, false, 3);
        let removed = network.remove_server(1).await;
        assert_eq!(removed, true);
        assert_eq!(network.servers_are_connected(0, 1).await, false);

        let added = network.add_server(1).await;
        assert_eq!(added, true);
        assert_eq!(network.servers_are_connected(0, 1).await, true);
    }
}
