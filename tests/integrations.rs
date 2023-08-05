use distributed_systems::in_memory_cluster::InMemoryCluster;
use std::sync::Arc;

#[tokio::test]
async fn test_create_basic_network() {
    let net = InMemoryCluster::new(false, false, 3);
    Arc::new(net).start().await;
}
