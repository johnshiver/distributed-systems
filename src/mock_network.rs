use std::collections::HashMap;
use std::sync::{Arc, mpsc, Mutex};
use crate::raft_node::{AppendEntriesReply, RaftNode};
use serde_json::{Error, Value};
use crate::errors::NetworkErrors;

pub struct Network {
    reliable: bool,
    long_delays: bool,
    servers: HashMap<String, Arc<Mutex<Server>>>,
    // reply_channel:
}

impl Network {
    pub fn new(reliable: bool, long_delays: bool) -> Self {
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

        Network{ reliable, long_delays, servers: Default::default() }
    }

    pub fn send_request(&self, request: NetworkRequest) -> Result<NetworkReply, NetworkErrors>{
        let raw_response = serde_json::to_value(&AppendEntriesReply{})?;
        Ok(NetworkReply::new(false, raw_response))
    }

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