use std::net::IpAddr;

use ipset::{HashIp, Session};

fn main() {
    let mut session: Session<HashIp> = Session::<HashIp>::new("test".to_string());
    let ip: IpAddr = "192.168.3.1".parse().unwrap();
    if let Err(err) = session.test(ip) {
        println!("test ipset failed:{:?}", err);
        return;
    }
    if let Err(err) = session.create(|builder| builder.with_ipv6(false)?.build()) {
        println!("create ipset failed:{:?}", err);
        return;
    }

    if let Err(err) = session.add(ip) {
        println!("add ip to ipset failed:{:?}", err);
        return;
    }

    if let Err(err) = session.list() {
        println!("list ip from ipset failed:{:?}", err);
        return;
    }

    if let Err(err) = session.del(ip) {
        println!("delete ip from ipset failed:{:?}", err);
        return;
    }

    if let Err(err) = session.flush() {
        println!("flush ipset failed:{:?}", err);
    }

    if let Err(err) = session.destroy() {
        println!("destroy ipset failed:{:?}", err);
    }
}
