use crate::mock_network::{Network, NetworkRequest};
use serde::{Deserialize, Serialize};
use serde_json::*;

pub struct RaftNode {
    network: Network,
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
        RaftNode{network: Network::new(false, false)}
    }

    pub fn handle_append_entries(request: AppendEntriesRequest) -> AppendEntriesReply {

        AppendEntriesReply{}
    }

    pub fn send_append_entries(&self, host: String, request: AppendEntriesRequest) -> AppendEntriesReply {

        // let raw_request = to_value(&request)?;
        // let raw_request = NetworkRequest::new(host, "AppendEntries".to_string(), raw_request);
        // let reply = self.network.send();

        AppendEntriesReply{}
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
pub struct AppendEntriesRequest {

}

#[derive(Serialize, Deserialize)]
pub struct AppendEntriesReply {

}
