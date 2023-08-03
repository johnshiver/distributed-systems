use distributed_systems::mock_network::InMemoryNetwork;

#[test]
fn test_create_basic_network() {
    let net = InMemoryNetwork::new(false, false, 3);
    net.start();
}