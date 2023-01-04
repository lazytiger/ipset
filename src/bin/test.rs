fn main() {
    let mut ipset = ipset::IPSet::new();
    let mut session = ipset.session();
    session.list("test").unwrap();
    session.add("test", "192.168.3.2".parse().unwrap()).unwrap();
}
