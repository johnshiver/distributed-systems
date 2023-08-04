use distributed_systems::in_memory_network::InMemoryNetwork;
use std::sync::Arc;

#[tokio::test]
async fn test_create_basic_network() {
    let net = InMemoryNetwork::new(false, false, 3);
    Arc::new(net).start().await;
}
