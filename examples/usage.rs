use std::net::IpAddr;

use ipset::types::{BitmapIp, Error, HashIp, IpDataType};
use ipset::{IPSet, Session};

fn test_hash_ip() -> Result<(), Error> {
    let mut session: Session<HashIp> = Session::new("test".to_string());
    let ip: IpAddr = "192.168.3.1".parse().unwrap();
    session.create(|builder| builder.with_ipv6(false)?.build())?;

    let ret = session.add(ip, None)?;
    println!("add {}", ret);

    let exists = session.test(ip)?;
    println!("test {}", exists);

    let ips = session.list()?;
    for ip in ips {
        println!("list {}", ip);
    }

    let ret = session.save("test.ipset".to_string())?;
    println!("save {}", ret);

    let ret = session.del(ip)?;
    println!("del {}", ret);

    let ret = session.flush()?;
    println!("flush {}", ret);

    let ret = session.destroy()?;
    println!("destroy {}", ret);

    Ok(())
}

fn test_bitmap_ip() -> Result<(), Error> {
    let mut session: Session<BitmapIp> = Session::new("test".into());
    let from: IpAddr = "192.168.3.1".parse().unwrap();
    let to: IpAddr = "192.168.3.255".parse().unwrap();
    let from: IpDataType = from.into();
    let to: IpDataType = to.into();
    session.create(|builder| builder.with_range(&from, &to)?.build())?;
    Ok(())
}

fn main() {
    let set = IPSet::new();
    //set.restore("test.ipset".to_string()).unwrap();
    println!("restore");

    if let Err(err) = test_hash_ip() {
        println!("test failed:{:?}", err);
    }
    if let Err(err) = test_bitmap_ip() {
        println!("test failed:{:?}", err);
    }
}
