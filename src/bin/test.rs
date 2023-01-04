fn main() {
    let mut ipset = ipset::IPSet::new();
    let mut session = ipset.session();
    session.flush("test").unwrap();
    let ip = "192.168.3.2".parse().unwrap();
    let ret = session.add("test", ip).unwrap();
    println!("add {} return {}", ip, ret);
    let ips = session.list("test").unwrap();
    for ip in ips {
        println!("{}", ip);
    }
    let ret = session.del("test", ip).unwrap();
    println!("delete:{} return:{}", ip, ret);
    let ip = "192.168.3.1".parse().unwrap();
    let ret = session.del("test", ip).unwrap();
    println!("delete:{} return:{}", ip, ret);
    let ips = session.list("test").unwrap();
    for ip in ips {
        println!("after delete:{}", ip);
    }

    let ret = session.add("test", ip).unwrap();
    println!("add {} return {}", ip, ret);
    session.flush("test").unwrap();
    let ips = session.list("test").unwrap();
    for ip in ips {
        println!("after flush:{}", ip);
    }
    session.flush("abc").unwrap();
}
