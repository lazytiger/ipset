//! All the types used by libipset.

use std::error::Error as StdError;
use std::ffi::{CString, NulError};
use std::fmt::{Display, Formatter};
use std::net::{AddrParseError, IpAddr};
use std::num::ParseIntError;

use derive_more::{Display, From, Into};

use crate::{binding, Session};

/// list method
pub struct ListMethod;

/// bitmap method
pub struct BitmapMethod;

/// hash method
pub struct HashMethod;

/// ip data type
/// Ip wrapper including ipv4 and ipv6
pub enum IpDataType {
    IPv4(libc::in_addr),
    IPv6(libc::in6_addr),
}

impl<T: SetType> SetData<T> for IpDataType {
    /// get ip address pointer and ip family pointer.
    fn set_data(&self, session: &Session<T>, from: Option<bool>) -> Result<(), Error> {
        let (ip, family) = match self {
            IpDataType::IPv4(ip) => (ip as *const _ as _, &binding::NFPROTO_IPV4 as *const _ as _),
            IpDataType::IPv6(ip) => (ip as *const _ as _, &binding::NFPROTO_IPV6 as *const _ as _),
        };
        session.set_data(binding::ipset_opt_IPSET_OPT_FAMILY, family)?;
        let opt = match from {
            Some(true) => binding::ipset_opt_IPSET_OPT_IP_FROM,
            Some(false) => binding::ipset_opt_IPSET_OPT_IP_TO,
            None => binding::ipset_opt_IPSET_OPT_IP,
        };
        session.set_data(opt, ip)
    }
}

impl Parse for IpDataType {
    fn parse(&mut self, s: &str) -> Result<(), Error> {
        let ip: IpAddr = s.parse()?;
        *self = ip.into();
        Ok(())
    }
}

impl Default for IpDataType {
    fn default() -> Self {
        IpDataType::IPv4(libc::in_addr { s_addr: 0 })
    }
}

impl From<IpAddr> for IpDataType {
    fn from(ip: IpAddr) -> Self {
        match ip {
            IpAddr::V4(v4) => {
                let ip: u32 = v4.into();
                IpDataType::IPv4(libc::in_addr { s_addr: ip.to_be() })
            }
            IpAddr::V6(v6) => IpDataType::IPv6(libc::in6_addr {
                s6_addr: v6.octets(),
            }),
        }
    }
}

impl From<&IpDataType> for IpAddr {
    fn from(value: &IpDataType) -> Self {
        match value {
            IpDataType::IPv4(ip) => IpAddr::from(ip.s_addr.to_ne_bytes()),
            IpDataType::IPv6(ip) => IpAddr::from(ip.s6_addr),
        }
    }
}

impl Display for IpDataType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let ip: IpAddr = self.into();
        write!(f, "{}", ip)
    }
}

/// net data type
#[derive(Default, From, Into)]
pub struct NetDataType {
    ip: IpDataType,
    cidr: u8,
}

impl NetDataType {
    /// create net using ip and cidr
    pub fn new(ip: impl Into<IpDataType>, cidr: u8) -> Self {
        Self {
            ip: ip.into(),
            cidr,
        }
    }

    /// return ip of the net
    pub fn ip(&self) -> IpAddr {
        (&self.ip).into()
    }

    /// return cidr of the net
    pub fn cidr(&self) -> u8 {
        self.cidr
    }
}

impl<T: SetType> SetData<T> for NetDataType {
    fn set_data(&self, session: &Session<T>, from: Option<bool>) -> Result<(), Error> {
        self.ip.set_data(session, from)?;
        session.set_data(
            binding::ipset_opt_IPSET_OPT_CIDR,
            &self.cidr as *const _ as _,
        )
    }
}

impl Parse for NetDataType {
    fn parse(&mut self, s: &str) -> Result<(), Error> {
        let mut ss = s.split("/");
        if let Some(ip) = ss.next() {
            let ip: IpAddr = ip.parse()?;
            self.ip = ip.into();
        }
        if let Some(cidr) = ss.next() {
            self.cidr = cidr.parse()?;
        } else {
            self.cidr = 32;
        }
        Ok(())
    }
}

impl Display for NetDataType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}/{}", self.ip, self.cidr)
    }
}

/// mac data type, [u8; 6]
#[derive(Default, From, Into)]
pub struct MacDataType {
    mac: [u8; 6],
}

impl Parse for MacDataType {
    fn parse(&mut self, s: &str) -> Result<(), Error> {
        let mac: Vec<u8> = s
            .split(":")
            .filter_map(|s| u8::from_str_radix(s, 16).ok())
            .collect();
        if mac.len() != 6 {
            Err(Error::InvalidOutput(s.into()))
        } else {
            self.mac.copy_from_slice(mac.as_slice());
            Ok(())
        }
    }
}

impl<T: SetType> SetData<T> for MacDataType {
    fn set_data(&self, session: &Session<T>, _from: Option<bool>) -> Result<(), Error> {
        session.set_data(binding::ipset_opt_IPSET_OPT_ETHER, self.mac.as_ptr() as _)
    }
}

impl Display for MacDataType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let data = self.mac.map(|d| format!("{:02x}", d)).join(":");
        write!(f, "{}", data)
    }
}

/// port data type, u16
#[derive(Default, From, Into)]
pub struct PortDataType {
    port: u16,
}

impl<T: SetType> SetData<T> for PortDataType {
    fn set_data(&self, session: &Session<T>, from: Option<bool>) -> Result<(), Error> {
        let opt = match from {
            Some(true) => binding::ipset_opt_IPSET_OPT_PORT_FROM,
            Some(false) => binding::ipset_opt_IPSET_OPT_PORT_TO,
            None => binding::ipset_opt_IPSET_OPT_PORT,
        };
        session.set_data(opt, &self.port as *const _ as _)
    }
}

impl Parse for PortDataType {
    fn parse(&mut self, s: &str) -> Result<(), Error> {
        self.port = s.parse()?;
        Ok(())
    }
}

impl Display for PortDataType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.port)
    }
}

/// iface data type, CString
#[derive(Default)]
pub struct IfaceDataType {
    name: CString,
}

impl From<String> for IfaceDataType {
    fn from(value: String) -> Self {
        Self {
            name: CString::new(value).unwrap(),
        }
    }
}

impl From<IfaceDataType> for String {
    fn from(value: IfaceDataType) -> Self {
        value.name.to_string_lossy().to_string()
    }
}

impl Parse for IfaceDataType {
    fn parse(&mut self, s: &str) -> Result<(), Error> {
        self.name = CString::new(s)?;
        Ok(())
    }
}

impl<T: SetType> SetData<T> for IfaceDataType {
    fn set_data(&self, session: &Session<T>, _from: Option<bool>) -> Result<(), Error> {
        session.set_data(binding::ipset_opt_IPSET_OPT_IFACE, self.name.as_ptr() as _)
    }
}

impl Display for IfaceDataType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name.to_string_lossy())
    }
}

/// mark data type, u32
#[derive(Default, From, Into)]
pub struct MarkDataType {
    mark: u32,
}

impl Parse for MarkDataType {
    fn parse(&mut self, s: &str) -> Result<(), Error> {
        self.mark = s.parse()?;
        Ok(())
    }
}

impl<T: SetType> SetData<T> for MarkDataType {
    fn set_data(&self, session: &Session<T>, _: Option<bool>) -> Result<(), Error> {
        session.set_data(
            binding::ipset_opt_IPSET_OPT_MARK,
            &self.mark as *const _ as _,
        )
    }
}

impl Display for MarkDataType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.mark)
    }
}

/// set name, CString
#[derive(Default)]
pub struct SetDataType {
    name: CString,
}

impl From<String> for SetDataType {
    fn from(value: String) -> Self {
        Self {
            name: CString::new(value).unwrap(),
        }
    }
}

impl From<SetDataType> for String {
    fn from(value: SetDataType) -> Self {
        value.name.to_string_lossy().to_string()
    }
}

impl Parse for SetDataType {
    fn parse(&mut self, s: &str) -> Result<(), Error> {
        self.name = CString::new(s)?;
        Ok(())
    }
}

impl<T: SetType> SetData<T> for SetDataType {
    fn set_data(&self, session: &Session<T>, _: Option<bool>) -> Result<(), Error> {
        session.set_data(binding::ipset_opt_IPSET_OPT_NAME, self.name.as_ptr() as _)
    }
}

impl Display for SetDataType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name.to_string_lossy())
    }
}

macro_rules! impl_name {
    ($($types:ident),+) => {
        impl<$($types,)+> TypeName for ($($types),+)
            where $($types:TypeName),+ {
            fn name() -> String {
                [$($types::name(),)+].join(",")
            }
        }
    };

    ($ty:ty, $name:expr) => {
        impl TypeName for $ty {
            fn name() -> String {
                $name.into()
            }
        }
    }
}

impl_name!(ListMethod, "list");
impl_name!(BitmapMethod, "bitmap");
impl_name!(HashMethod, "hash");
impl_name!(IpDataType, "ip");
impl_name!(NetDataType, "net");
impl_name!(MacDataType, "mac");
impl_name!(PortDataType, "port");
impl_name!(IfaceDataType, "iface");
impl_name!(MarkDataType, "mark");
impl_name!(SetDataType, "set");
impl_name!(A, B);
impl_name!(A, B, C);

macro_rules! impl_set_data {
    ($($types:ident),+) => {
        #[allow(non_snake_case)]
        impl<T, $($types),+> SetData<T> for ($($types),+)
            where T:SetType,
                $($types:SetData<T>),+ {
            fn set_data(&self, session:&Session<T>, from:Option<bool>) -> Result<(), Error> {
                let ($($types),+) = self;
                $($types.set_data(session, from)?;)+
                Ok(())
            }
        }
    };
}

impl_set_data!(A, B);
impl_set_data!(A, B, C);

macro_rules! impl_parse {
   ($($types:ident),+) => {
       #[allow(non_snake_case)]
       impl<$($types),+> Parse for ($($types),+)
            where  $($types:Parse),+ {
            fn parse(&mut self, s:&str) -> Result<(), Error> {
                let ($($types),+) = self;
                let mut ss = s.split(",");
                $(
                    if let Some(item) = ss.next() {
                        $types.parse(item)?;
                    } else {
                        return Err(Error::InvalidOutput(s.into()));
                    };
                )+
                Ok(())
            }
        }
    };
}

impl_parse!(A, B);
impl_parse!(A, B, C);

/// A set type comprises of the storage method by which the data is stored and the data type(s) which are stored in the set.
/// Therefore the TYPENAME parameter  of the create command follows the syntax
/// `TYPENAME := method:datatype[,datatype[,datatype]]`
/// where the current list of the methods are bitmap, hash, and list and the possible data types are ip, net, mac, port and iface.
pub trait SetType: Sized {
    type Method;
    type DataType: SetData<Self> + Parse + Default;
}

/// A trait used for generate name for the ipset type and method, such as ip, net, etc.
pub trait TypeName {
    fn name() -> String;
}

/// Set data in session for the data type.
pub trait SetData<T: SetType> {
    fn set_data(&self, session: &Session<T>, from: Option<bool>) -> Result<(), Error>;
}

/// parse data type from string.
pub trait Parse {
    fn parse(&mut self, s: &str) -> Result<(), Error>;
}

/// A trait to generate literal name for a ipset method:type composition.
pub trait ToCString {
    fn to_cstring() -> CString;
}

impl<T> ToCString for T
where
    T: SetType,
    T::Method: TypeName,
    T::DataType: TypeName,
{
    fn to_cstring() -> CString {
        CString::new([T::Method::name(), T::DataType::name()].join(":")).unwrap()
    }
}

/// Errors defined in this crate.
#[derive(Debug, From, Display)]
pub enum Error {
    #[from(ignore)]
    #[display(fmt = "DataSet:['{}', {}", _0, _1)]
    DataSet(String, bool),
    #[from(ignore)]
    #[display(fmt = "Cmd:['{}', {}", _0, _1)]
    Cmd(String, bool),
    #[from(ignore)]
    #[display(fmt = "TypeGet:['{}', {}", _0, _1)]
    TypeGet(String, bool),
    #[from(ignore)]
    InvalidOutput(String),
    #[from(ignore)]
    SaveRestore(String),
    AddrParse(AddrParseError),
    ParseInt(ParseIntError),
    Nul(NulError),
}

impl Error {
    pub(crate) fn cmd_contains(&self, m: &str) -> bool {
        if let Error::Cmd(message, _) = self {
            message.contains(m)
        } else {
            false
        }
    }

    pub fn is_error(&self) -> bool {
        match self {
            Error::DataSet(_, error) => *error,
            Error::Cmd(_, error) => *error,
            Error::TypeGet(_, error) => *error,
            _ => false,
        }
    }
}

impl StdError for Error {}

macro_rules! impl_set_type {
    ($method:ident, $($types:ident),+) => {
        #[allow(unused_parens)]
        impl SetType for concat_idents!($method, $($types),+) {
            type Method = concat_idents!($method, Method);
            type DataType = ($(concat_idents!($types, DataType)),+);
        }
    }
}

/// The bitmap:ip set type uses a memory range to store either IPv4 host (default) or IPv4 network addresses.
/// A bitmap:ip type of set can store up to 65536 entries.
pub struct BitmapIp;
impl_set_type!(Bitmap, Ip);

/// The bitmap:ip,mac set type uses a memory range to store IPv4 and a MAC address pairs.
/// A bitmap:ip,mac type of set can store up to 65536 entries.
pub struct BitmapIpMac;
impl_set_type!(Bitmap, Ip, Mac);

/// The bitmap:port set type uses a memory range to store port numbers and such a set can store up to 65536 ports.
pub struct BitmapPort;
impl_set_type!(Bitmap, Port);

/// The hash:ip set type uses a hash to store IP host addresses (default) or network addresses.
/// Zero valued IP address cannot be stored in a hash:ip type of set.
pub struct HashIp;
impl_set_type!(Hash, Ip);

/// The hash:mac set type uses a hash to store MAC addresses.
/// Zero valued MAC addresses cannot be stored in a hash:mac type of set.
pub struct HashMac;
impl_set_type!(Hash, Mac);

/// The hash:ip,mac set type uses a hash to store IP and a MAC address pairs.
/// Zero valued MAC addresses cannot be stored in a hash:ip,mac type of set.
pub struct HashIpMac;
impl_set_type!(Hash, Ip, Mac);

/// The hash:net set type uses a hash to store different sized IP network addresses.  
/// Network address with zero prefix size cannot be stored in this type of sets.
pub struct HashNet;
impl_set_type!(Hash, Net);

/// The hash:net,net set type uses a hash to store pairs of different sized IP network addresses.  
/// Bear  in  mind  that  the  first parameter has precedence over  the second,  
/// so a nomatch entry could be potentially be ineffective if a more specific first parameter existed with a suitable second parameter.  
/// Network address with zero prefix size cannot be stored in this type of set.
pub struct HashNetNet;
impl_set_type!(Hash, Net, Net);

/// The hash:ip,port set type uses a hash to store IP address and port number pairs.  
/// The port number is interpreted together with a protocol (default TCP)  and  zero protocol number cannot be used.
pub struct HashIpPort;
impl_set_type!(Hash, Ip, Port);

/// The  hash:net,port  set  type uses a hash to store different sized IP network address and port pairs.
/// The port number is interpreted together with a protocol (de‐fault TCP) and zero protocol number cannot be used.
/// Network address with zero prefix size is not accepted either.
pub struct HashNetPort;
impl_set_type!(Hash, Net, Port);

/// The hash:ip,port,ip set type uses a hash to store IP address, port number and a second IP address triples.
/// The port number is interpreted together with a protocol (default TCP) and zero protocol number cannot be used.
pub struct HashIpPortIp;
impl_set_type!(Hash, Ip, Port, Ip);

/// The hash:ip,port,net set type uses a hash to store IP address, port number and IP network address triples.
/// The port number is interpreted together with a protocol (default TCP) and zero protocol number cannot be used.
/// Network address with zero prefix size cannot be stored either.
pub struct HashIpPortNet;
impl_set_type!(Hash, Ip, Port, Net);

/// The hash:ip,mark set type uses a hash to store IP address and packet mark pairs.
pub struct HashIpMark;
impl_set_type!(Hash, Ip, Mark);

/// The hash:net,port,net set type behaves similarly to hash:ip,port,net but accepts a cidr value for both the first and last parameter.
/// Either subnet is permitted to be a /0 should you wish to match port between all destinations.
pub struct HashNetPortNet;
impl_set_type!(Hash, Net, Port, Net);

/// The hash:net,iface set type uses a hash to store different sized IP network address and interface name pairs.
pub struct HashNetIface;
impl_set_type!(Hash, Net, Iface);

/// The list:set type uses a simple list in which you can store set names.
pub struct ListSet;
impl_set_type!(List, Set);

#[allow(unused_imports)]
mod tests {
    use std::net::IpAddr;

    use crate::types::{
        BitmapIp, BitmapIpMac, BitmapPort, HashIp, HashIpMac, HashIpMark, HashIpPort, HashIpPortIp,
        HashIpPortNet, HashMac, HashNet, HashNetIface, HashNetNet, HashNetPort, HashNetPortNet,
        ListSet,
    };
    use crate::types::{
        IfaceDataType, IpDataType, MacDataType, MarkDataType, NetDataType, Parse, PortDataType,
        SetDataType, ToCString,
    };

    #[test]
    fn test_ip() {
        let ip: IpAddr = "127.0.0.1".parse().unwrap();
        let mut data: IpDataType = ip.into();
        let ip1: IpAddr = (&data).into();
        assert_eq!(ip, ip1);
        assert_eq!("127.0.0.1", format!("{}", data));
        data.parse("192.168.3.1").unwrap();
        assert_eq!("192.168.3.1", format!("{}", data));
    }

    #[test]
    fn test_net() {
        let ip: IpAddr = "127.0.0.1".parse().unwrap();
        let mut net = NetDataType::new(ip, 8);
        assert_eq!("127.0.0.1/8", format!("{}", net));
        net.parse("192.168.3.1/24").unwrap();
        assert_eq!("192.168.3.1/24", format!("{}", net));
    }

    #[test]
    fn test_mac() {
        let mut mac: MacDataType = [124u8, 24u8, 32u8, 129u8, 84u8, 223u8].into();
        assert_eq!("7c:18:20:81:54:df", format!("{}", mac));
        mac.parse("00:15:5d:37:d9:2f").unwrap();
        assert_eq!("00:15:5d:37:d9:2f", format!("{}", mac));
    }

    #[test]
    fn test_mark() {
        let mut mark: MarkDataType = 32u32.into();
        assert_eq!("32", format!("{}", mark));
        mark.parse("123").unwrap();
        assert_eq!("123", format!("{}", 123));
    }

    #[test]
    fn test_port() {
        let mut port: PortDataType = 1235u16.into();
        assert_eq!("1235", format!("{}", port));
        port.parse("1234").unwrap();
        assert_eq!("1234", format!("{}", port));
    }

    #[test]
    fn test_iface() {
        let mut iface: IfaceDataType = String::from("abc").into();
        assert_eq!("abc", format!("{}", iface));
        iface.parse("test").unwrap();
        assert_eq!("test", format!("{}", iface));
    }

    #[test]
    fn test_set() {
        let mut set: SetDataType = String::from("abc").into();
        assert_eq!("abc", format!("{}", set));
        set.parse("test").unwrap();
        assert_eq!("test", format!("{}", set));
    }

    #[test]
    fn test_ip_port_ip() {
        let mut data = (
            IpDataType::default(),
            PortDataType::default(),
            IpDataType::default(),
        );
        data.parse("192.168.3.1,8080,192.168.3.2").unwrap();
        assert_eq!("192.168.3.1", format!("{}", data.0));
        assert_eq!("8080", format!("{}", data.1));
        assert_eq!("192.168.3.2", format!("{}", data.2));
    }

    #[test]
    fn test_type_name() {
        assert_eq!(HashIp::to_cstring().to_str().unwrap(), "hash:ip");
        assert_eq!(
            HashNetIface::to_cstring().to_str().unwrap(),
            "hash:net,iface"
        );
        assert_eq!(HashNetNet::to_cstring().to_str().unwrap(), "hash:net,net");
        assert_eq!(HashNetPort::to_cstring().to_str().unwrap(), "hash:net,port");
        assert_eq!(HashNet::to_cstring().to_str().unwrap(), "hash:net");
        assert_eq!(HashIpPort::to_cstring().to_str().unwrap(), "hash:ip,port");
        assert_eq!(HashIpMark::to_cstring().to_str().unwrap(), "hash:ip,mark");
        assert_eq!(
            HashIpPortNet::to_cstring().to_str().unwrap(),
            "hash:ip,port,net"
        );
        assert_eq!(HashIpMac::to_cstring().to_str().unwrap(), "hash:ip,mac");
        assert_eq!(
            HashIpPortIp::to_cstring().to_str().unwrap(),
            "hash:ip,port,ip"
        );
        assert_eq!(
            HashNetPortNet::to_cstring().to_str().unwrap(),
            "hash:net,port,net"
        );
        assert_eq!(HashMac::to_cstring().to_str().unwrap(), "hash:mac");
        assert_eq!(ListSet::to_cstring().to_str().unwrap(), "list:set");
        assert_eq!(BitmapPort::to_cstring().to_str().unwrap(), "bitmap:port");
        assert_eq!(BitmapIp::to_cstring().to_str().unwrap(), "bitmap:ip");
        assert_eq!(BitmapIpMac::to_cstring().to_str().unwrap(), "bitmap:ip,mac");
    }
}
