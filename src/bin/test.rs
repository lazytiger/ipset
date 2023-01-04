fn main() {
    let ipset = ipset::IPSet::new();
    let session = ipset.session();
    let ip = "192.168.3.1".parse().unwrap();
    let exists = session.add("test", ip).unwrap();
    println!("------------------{}", exists);
    let ip = "127.0.0.1".parse().unwrap();
    let exists = session.test("test", ip).unwrap();
    println!("------------------{}", exists);
}
