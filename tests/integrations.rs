use distributed_systems::in_memory_cluster::InMemoryCluster;
use std::sync::Arc;

#[tokio::test]
async fn test_create_basic_network() {
    let net = Arc::new(InMemoryCluster::new(false, false, 3));
    let join_handle = net.clone().start().await;
    tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;
    println!("stopping");
    net.stop().await;
    println!("waiting");
    let _ = join_handle.await;
}
