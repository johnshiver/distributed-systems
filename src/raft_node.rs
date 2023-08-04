use crate::in_memory_network::{NetworkClient, NetworkRequest};
use serde::{Deserialize, Serialize};

pub struct RaftNode {
    network_client: NetworkClient,
    // peers
}

impl RaftNode {
    pub fn new() -> Self {
        /*
           // single goroutine to handle all ClientEnd.Call()s
           go func() {
               for {
                   select {
                   case xreq := <-rn.endCh:
                       atomic.AddInt32(&rn.count, 1)
                       atomic.AddInt64(&rn.bytes, int64(len(xreq.args)))
                       go rn.processReq(xreq)
                   case <-rn.done:
                       return
                   }
               }
           }()
        */
        RaftNode {}
    }

    pub fn handle_append_entries(request: AppendEntriesRequest) -> AppendEntriesReply {
        AppendEntriesReply {}
    }

    // TODO: should be able to make generic method that handles the nitty gritty request details
    //       let reply_channel = await self.client.send_request<AppendEntriesRequest>("hostname");
    //       let reply_channel = await self.client.broadcast_request<AppendEntriesRequest>();
    //       self.client.peers for access to each individual node;
    pub fn send_append_entries(
        &self,
        host: String,
        request: AppendEntriesRequest,
    ) -> AppendEntriesReply {
        // let raw_request = to_value(&request)?;
        // let raw_request = NetworkRequest::new(host, "AppendEntries".to_string(), raw_request);
        // let reply = self.network.send();

        AppendEntriesReply {}
    }

    // pub fn connect(node: RaftNode) {
    //
    // }
    //
    // pub fn disconnect(node: RaftNode) {
    //
    // }
}

#[derive(Serialize, Deserialize)]
pub struct AppendEntriesRequest {}

#[derive(Serialize, Deserialize)]
pub struct AppendEntriesReply {}
