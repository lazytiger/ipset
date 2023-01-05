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

Support the following type:

* hash:ip
* hash:net not fully supported

### Example

  ```rust
  use ipset::{Session, SetType};

fn main() {
    let mut session = Session::new();
    if let Err(err) = session.create("test", SetType::HashIp, |builder| {
        builder.with_ipv6(false)?.build()
    }) {
        println!("create ipset failed:{:?}", err);
        return;
    }

    if let Err(err) = session.add("test", "127.0.0.1".parse().unwrap()) {
        println!("add ip to ipset failed:{:?}", err);
        return;
    }

    if let Err(err) = session.list("test") {
        println!("list ip from ipset failed:{:?}", err);
        return;
    }

    if let Err(err) = session.del("test", "127.0.0.1".parse().unwrap()) {
        println!("delete ip from ipset failed:{:?}", err);
        return;
    }

    if let Err(err) = session.flush("test") {
        println!("flush ipset failed:{:?}", err);
    }

    if let Err(err) = session.destroy("test") {
        println!("destroy ipset failed:{:?}", err);
    }
}
```