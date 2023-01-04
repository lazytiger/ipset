#[test]
fn test() {
    let ipset = ipset::IPSet::new();
    let session = ipset.session();
    let ip = "192.168.3.1".parse().unwrap();
    session.test("test", ip).unwrap();
}
