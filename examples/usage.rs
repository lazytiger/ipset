use std::net::IpAddr;

use ipset::{Error, HashIp, Session};

fn test() -> Result<(), Error> {
    let mut session: Session<HashIp> = Session::<HashIp>::new("test".to_string());
    let ip: IpAddr = "192.168.3.1".parse().unwrap();
    session.create(|builder| builder.with_ipv6(false)?.build())?;

    let ret = session.add(ip)?;
    println!("add {}", ret);

    let exists = session.test(ip)?;
    println!("test {}", exists);

    let ips = session.list()?;
    for ip in ips {
        println!("list {}", ip);
    }

    let ret = session.del(ip)?;
    println!("del {}", ret);

    let ret = session.flush()?;
    println!("flush {}", ret);

    let ret = session.destroy()?;
    println!("destroy {}", ret);

    Ok(())
}

fn main() {
    if let Err(err) = test() {
        println!("test failed:{:?}", err);
    }
}
