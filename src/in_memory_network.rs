use crate::errors::NetworkErrors;
use crate::raft_node::{AppendEntriesReply, RaftNode};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use serde_json::{to_value, Value};
use std::sync::Arc;
use tokio::sync::{mpsc, Mutex};
use tokio::time::{timeout, Duration};

pub struct InMemoryNetwork {
    reliable: bool,
    long_delays: bool,
    servers: Arc<Mutex<Vec<Server>>>,
    network_connections: Arc<Mutex<Vec<Vec<bool>>>>, // e.g. network_connections[x][y] == true means server x can talk to server y
    receive_requests: Arc<Mutex<mpsc::Receiver<NetworkRequest>>>,
    send_requests: Arc<Mutex<mpsc::Sender<NetworkRequest>>>,
}

impl InMemoryNetwork {
    pub fn new(reliable: bool, long_delays: bool, server_count: i8) -> Self {
        let (send, recv) = mpsc::channel(1);
        let sender = Arc::new(Mutex::new(send));
        let receiver = Arc::new(Mutex::new(recv));

        // all servers are initially connected
        let network_connections = Arc::new(Mutex::new(vec![
            vec![true; server_count as usize];
            server_count as usize
        ]));

        // init servers
        let servers = Arc::new(Mutex::new(
            (0..server_count)
                .map(|i| {
                    Server::new(
                        i as usize,
                        InMemoryNetworkClient::new(server_count, sender.clone()),
                    )
                })
                .collect::<Vec<_>>(),
        ));

        InMemoryNetwork {
            reliable,
            long_delays,
            servers,
            receive_requests: receiver,
            send_requests: sender,
            network_connections,
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

    // Gets a server from the network by index.
    // pub async fn get_server(&self, index: usize) -> Option<Server> {
    //     let servers = self.servers.lock().await;
    //     let server = if index < servers.len() {
    //         Some(servers[index].clone()) // This requires that Server implements Clone.
    //     } else {
    //         None
    //     };
    //     drop(servers); // Explicitly drop the lock to release it.
    //     server
    // }

    pub async fn start(self: Arc<Self>) {
        let receiver = Arc::clone(&self.receive_requests);

        tokio::spawn(async move {
            while let Some(req) = receiver.lock().await.recv().await {
                // check if there is an active connection between the servers
                if !self
                    .servers_are_connected(req.origin_server, req.destination_server)
                    .await
                {
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
                    //
                    let servers = self.servers.lock().await;
                    if let Some(target_server) = servers.get(req.destination_server as usize) {}

                    // process request
                }
            }
        });
    }

    // if you want to send requests to the network use NetworkClient
    pub async fn create_client(&self) -> InMemoryNetworkClient {
        InMemoryNetworkClient::new(
            self.network_connections.lock().await.len() as i8,
            self.send_requests.clone(),
        )
    }

    // pub fn send_request(&self, request: NetworkRequest) -> Result<NetworkReply, NetworkErrors>{
    //     let raw_response = serde_json::to_value(&AppendEntriesReply{})?;
    //     Ok(NetworkReply::new(false, raw_response))
    // }

    // process request
    pub fn process_request(&self, request: NetworkRequest) {}
}

#[derive(Serialize, Deserialize, Debug)]
pub enum ServiceMethod {
    AppendEntries,
    // Add other variants here...
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
        let raw_request = NetworkRequest::new(origin, destination, svc_method, raw_request);
        let network_reply = self.send_raw_request(raw_request).await?;
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
            Err(_) => Err(NetworkErrors::Timeout),
        };
    }
}

struct Server {
    raft: RaftNode,
}

impl Server {
    pub fn new(id: usize, network_client: InMemoryNetworkClient) -> Self {
        Server {
            raft: RaftNode::new(id, network_client),
        }
    }
    fn dispatch(req: NetworkRequest) -> Result<NetworkReply, NetworkErrors> {
        // rs.mu.Lock()
        //
        // rs.count += 1
        //
        // // split Raft.AppendEntries into service and method
        // dot := strings.LastIndex(req.svcMeth, ".")
        // serviceName := req.svcMeth[:dot]
        // methodName := req.svcMeth[dot+1:]
        //
        // service, ok := rs.services[serviceName]
        //
        // rs.mu.Unlock()
        //
        // if ok {
        // return service.dispatch(methodName, req)
        // } else {
        // choices := []string{}
        // for k, _ := range rs.services {
        // choices = append(choices, k)
        // }
        // log.Fatalf("labrpc.Server.dispatch(): unknown service %v in %v.%v; expecting one of %v\n",
        // serviceName, serviceName, methodName, choices)
        // return replyMsg{false, nil}
        // }
        let raw_response = serde_json::to_value(&AppendEntriesReply {})?;
        Ok(NetworkReply::new(false, raw_response))
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
        let (send, recv) = mpsc::channel(0);
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

// Call send an RPC, wait for the reply.
// the return value indicates success; false means that
// no reply was received from the server.
// func (e *ClientEnd) Call(svcMeth string, args interface{}, reply interface{}) bool {
// req := reqMsg{}
// req.endname = e.endname
// req.svcMeth = svcMeth
// req.argsType = reflect.TypeOf(args)
// req.replyCh = make(chan replyMsg)
//
// qb := new(bytes.Buffer)
// qe := labgob.NewEncoder(qb)
// if err := qe.Encode(args); err != nil {
// panic(err)
// }
// req.args = qb.Bytes()
//
// //
// // send the request.
// //
// select {
// case e.ch <- req:
// // the request has been sent.
// case <-e.done:
// // entire Network has been destroyed.
// return false
// }
//
// //
// // wait for the reply.
// //
// rep := <-req.replyCh
// if rep.ok {
// rb := bytes.NewBuffer(rep.reply)
// rd := labgob.NewDecoder(rb)
// if err := rd.Decode(reply); err != nil {
// log.Fatalf("ClientEnd.Call(): decode reply: %v\n", err)
// }
// return true
// } else {
// return false
// }
// }

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_servers_are_connected() {
        let network = InMemoryNetwork::new(true, false, 3);

        // Since the initial network_connections are set to true, servers 1 and 2 should be connected.
        assert_eq!(network.servers_are_connected(0, 1).await, true);
    }

    #[tokio::test]
    async fn test_servers_remove_and_add() {
        let network = InMemoryNetwork::new(true, false, 3);
        let removed = network.remove_server(1).await;
        assert_eq!(removed, true);
        assert_eq!(network.servers_are_connected(0, 1).await, false);

        let added = network.add_server(1).await;
        assert_eq!(added, true);
        assert_eq!(network.servers_are_connected(0, 1).await, true);
    }
}
