use std::collections::HashMap;
use std::sync::{Arc, mpsc, Mutex};
use crate::raft_node::{AppendEntriesReply, RaftNode};
use serde_json::{Error, Value};
use crate::errors::NetworkErrors;

pub struct InMemoryNetwork {
    reliable: bool,
    long_delays: bool,
    servers: HashMap<String, Arc<Mutex<Server>>>,
    recv_channel: mpsc::Receiver<NetworkRequest>,
    send_channel: Arc<Mutex<mpsc::Sender<NetworkRequest>>>,
}

impl InMemoryNetwork {
    pub fn new(reliable: bool, long_delays: bool, server_count: i8) -> Self {

        let (send, recv) = mpsc::channel();
        let sender = Arc::new(Mutex::new(send));
        InMemoryNetwork { reliable, long_delays, servers: Default::default(), recv_channel: recv, send_channel: sender}
    }

    pub async fn start(&self) {
        tokio::spawn(async move {
            while let Ok(req) = self.recv.recv() {
                // process request
            }
        });
    }

    // TODO: should have a way to make it easy to get copies of the send channel
    //       which nodes can use to make requests across the network
    // pub fn create_client() -> InMemoryNetworkClient {
    //
    // }

    // pub fn send_request(&self, request: NetworkRequest) -> Result<NetworkReply, NetworkErrors>{
    //     let raw_response = serde_json::to_value(&AppendEntriesReply{})?;
    //     Ok(NetworkReply::new(false, raw_response))
    // }

    // process request
    pub fn process_request(&self, request: NetworkRequest) {

    }

}


struct Server {
    raft: RaftNode
}

impl Server {
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
        let raw_response = serde_json::to_value(&AppendEntriesReply{})?;
        Ok(NetworkReply::new(false, raw_response))
    }
}


pub struct NetworkRequest {
    end_name: String, // e.g. server name "1" "192.0.0.5"
    svc_method: String, // e.g. "Raft.AppendEntries"
    raw_request: Value,
    // reply_ch: mpsc::Sender<NetworkReply>,
}

impl NetworkRequest {

    pub fn new(host: String, method: String, raw_request: Value) -> Self {
        NetworkRequest{
            end_name: host,
            svc_method: method,
            raw_request,
        }
    }

}

pub struct NetworkReply {
    ok: bool,
    reply: Value,
}

impl NetworkReply {

    pub fn new(ok: bool, raw_response: Value) -> Self {
        NetworkReply{
            ok, reply: raw_response
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