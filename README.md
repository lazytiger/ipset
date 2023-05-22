## IPSet

[![Build Status](https://github.com/lazytiger/ipset/actions/workflows/rust.yml/badge.svg)](https://github.com/lazytiger/ipset/actions)
[![GitHub issues](https://img.shields.io/github/issues/lazytiger/ipset)](https://github.com/lazytiger/ipset/issues)
[![GitHub license](https://img.shields.io/github/license/lazytiger/ipset)](https://github.com/lazytiger/ipset/blob/master/LICENSE)
[![Releases](https://img.shields.io/github/v/release/lazytiger/ipset.svg?include_prereleases)](https://github.com/lazytiger/ipset/releases)

A library wrapper for `libipset`.  
Support the following commands:

* add
* del
* test
* create
* list
* destroy
* flush
* save
* restore

Support the following type:

* BitmapIp
* BitmapIpMac
* BitmapPort
* HashIp
* HashIpMac
* HashIpMark
* HashIpPort
* HashIpPortIp
* HashIpPortNet
* HashMac
* HashNet
* HashNetIface
* HashNetNet
* HashNetPort
* HashNetPortNet
* ListSet,

### Example

  ```rust
use std::net::IpAddr;

use ipset::{Error, HashIp, IPSet, Session};

fn main() -> Result<(), Error> {
    let mut session: Session<HashIp> = Session::<HashIp>::new("test".to_string());
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

    let ret = session.save("test.ipset")?;
    println!("save {}", ret);

    let ret = session.del(ip)?;
    println!("del {}", ret);

    let ret = session.flush()?;
    println!("flush {}", ret);

    let ret = session.destroy()?;
    println!("destroy {}", ret);

    let set = IPSet::new();
    set.restore("test.ipset")?;

    Ok(())
}
```
