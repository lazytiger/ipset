//! All the types used by libipset.

use std::error::Error as StdError;
use std::ffi::{CString, NulError};
use std::fmt::Formatter;
use std::net::{AddrParseError, IpAddr, Ipv4Addr, Ipv6Addr};
use std::num::ParseIntError;

use derive_more::{Display, From, Into};
use ipset_derive::SetType;

use crate::{binding, Session};

/// list method
pub struct ListMethod;

/// bitmap method
pub struct BitmapMethod;

/// hash method
pub struct HashMethod;

/// ip data type
/// Ip wrapper including ipv4 and ipv6
#[derive(Copy, Clone)]
pub enum IpDataType {
    IPv4(libc::in_addr),
    IPv6(libc::in6_addr),
}

impl IpDataType {
    pub fn to_ip_addr(&self) -> IpAddr {
        match self {
            IpDataType::IPv4(addr) => {
                let octets = addr.s_addr.to_ne_bytes();
                IpAddr::V4(Ipv4Addr::new(octets[0], octets[1], octets[2], octets[3]))
            }
            IpDataType::IPv6(addr) => {
                let segments = [
                    u16::from_be_bytes([addr.s6_addr[0], addr.s6_addr[1]]),
                    u16::from_be_bytes([addr.s6_addr[2], addr.s6_addr[3]]),
                    u16::from_be_bytes([addr.s6_addr[4], addr.s6_addr[5]]),
                    u16::from_be_bytes([addr.s6_addr[6], addr.s6_addr[7]]),
                    u16::from_be_bytes([addr.s6_addr[8], addr.s6_addr[9]]),
                    u16::from_be_bytes([addr.s6_addr[10], addr.s6_addr[11]]),
                    u16::from_be_bytes([addr.s6_addr[12], addr.s6_addr[13]]),
                    u16::from_be_bytes([addr.s6_addr[14], addr.s6_addr[15]]),
                ];
                IpAddr::V6(Ipv6Addr::new(
                    segments[0],
                    segments[1],
                    segments[2],
                    segments[3],
                    segments[4],
                    segments[5],
                    segments[6],
                    segments[7],
                ))
            }
        }
    }
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
        let s = s.split(" ").next().ok_or(Error::DataParse(s.to_string()))?;
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

impl From<Ipv4Addr> for IpDataType {
    fn from(ip: Ipv4Addr) -> Self {
        IpDataType::IPv4(libc::in_addr {
            s_addr: u32::from(ip).to_be(),
        })
    }
}

impl From<Ipv6Addr> for IpDataType {
    fn from(ip: Ipv6Addr) -> Self {
        IpDataType::IPv6(libc::in6_addr {
            s6_addr: ip.octets(),
        })
    }
}

impl From<IpAddr> for IpDataType {
    fn from(ip: IpAddr) -> Self {
        match ip {
            IpAddr::V4(v4) => v4.into(),
            IpAddr::V6(v6) => v6.into(),
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
    #[display("DataSet:['{}', {}", _0, _1)]
    DataSet(String, bool),
    #[from(ignore)]
    #[display("Cmd:['{}', {}", _0, _1)]
    Cmd(String, bool),
    #[from(ignore)]
    #[display("TypeGet:['{}', {}", _0, _1)]
    TypeGet(String, bool),
    #[from(ignore)]
    InvalidOutput(String),
    #[from(ignore)]
    SaveRestore(String),
    AddrParse(AddrParseError),
    ParseInt(ParseIntError),
    Nul(NulError),
    #[from(ignore)]
    CAOption(String),
    #[from(ignore)]
    DataParse(String),
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

/// The bitmap:ip set type uses a memory range to store either IPv4 host (default) or IPv4 network addresses.
/// A bitmap:ip type of set can store up to 65536 entries.
#[derive(SetType)]
pub struct BitmapIp;

/// The bitmap:ip,mac set type uses a memory range to store IPv4 and a MAC address pairs.
/// A bitmap:ip,mac type of set can store up to 65536 entries.
#[derive(SetType)]
pub struct BitmapIpMac;

/// The bitmap:port set type uses a memory range to store port numbers and such a set can store up to 65536 ports.
#[derive(SetType)]
pub struct BitmapPort;

/// The hash:ip set type uses a hash to store IP host addresses (default) or network addresses.
/// Zero valued IP address cannot be stored in a hash:ip type of set.
#[derive(SetType)]
pub struct HashIp;

/// The hash:mac set type uses a hash to store MAC addresses.
/// Zero valued MAC addresses cannot be stored in a hash:mac type of set.
#[derive(SetType)]
pub struct HashMac;

/// The hash:ip,mac set type uses a hash to store IP and a MAC address pairs.
/// Zero valued MAC addresses cannot be stored in a hash:ip,mac type of set.
#[derive(SetType)]
pub struct HashIpMac;

/// The hash:net set type uses a hash to store different sized IP network addresses.  
/// Network address with zero prefix size cannot be stored in this type of sets.
#[derive(SetType)]
pub struct HashNet;

/// The hash:net,net set type uses a hash to store pairs of different sized IP network addresses.  
/// Bear  in  mind  that  the  first parameter has precedence over  the second,  
/// so a nomatch entry could be potentially be ineffective if a more specific first parameter existed with a suitable second parameter.  
/// Network address with zero prefix size cannot be stored in this type of set.
#[derive(SetType)]
pub struct HashNetNet;

/// The hash:ip,port set type uses a hash to store IP address and port number pairs.  
/// The port number is interpreted together with a protocol (default TCP)  and  zero protocol number cannot be used.
#[derive(SetType)]
pub struct HashIpPort;

/// The  hash:net,port  set  type uses a hash to store different sized IP network address and port pairs.
/// The port number is interpreted together with a protocol (de‐fault TCP) and zero protocol number cannot be used.
/// Network address with zero prefix size is not accepted either.
#[derive(SetType)]
pub struct HashNetPort;

/// The hash:ip,port,ip set type uses a hash to store IP address, port number and a second IP address triples.
/// The port number is interpreted together with a protocol (default TCP) and zero protocol number cannot be used.
#[derive(SetType)]
pub struct HashIpPortIp;

/// The hash:ip,port,net set type uses a hash to store IP address, port number and IP network address triples.
/// The port number is interpreted together with a protocol (default TCP) and zero protocol number cannot be used.
/// Network address with zero prefix size cannot be stored either.
#[derive(SetType)]
pub struct HashIpPortNet;

/// The hash:ip,mark set type uses a hash to store IP address and packet mark pairs.
#[derive(SetType)]
pub struct HashIpMark;

/// The hash:net,port,net set type behaves similarly to hash:ip,port,net but accepts a cidr value for both the first and last parameter.
/// Either subnet is permitted to be a /0 should you wish to match port between all destinations.
#[derive(SetType)]
pub struct HashNetPortNet;

/// The hash:net,iface set type uses a hash to store different sized IP network address and interface name pairs.
#[derive(SetType)]
pub struct HashNetIface;

/// The list:set type uses a simple list in which you can store set names.
#[derive(SetType)]
pub struct ListSet;

pub trait WithNetmask {}

impl WithNetmask for BitmapMethod {}

impl WithNetmask for HashMethod {}

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

/// Options which ipset supported
pub enum EnvOption {
    /// Sorted output. When listing or saving sets, the entries are listed sorted.
    Sorted,
    /// Suppress any output to stdout and stderr.  ipset will still exit with error if it cannot continue.
    Quiet,
    /// When listing sets, enforce name lookup. The program will try to display the IP entries resolved to host names which requires slow DNS lookups.
    Resolve,
    /// Ignore errors when exactly the same set is to be created or already added entry is added or missing entry is deleted
    Exist,
    /// List just the names of the existing sets, i.e. suppress listing of set headers and members.
    ListSetName,
    /// List the set names and headers, i.e. suppress listing of set members.
    ListHeader,
}

impl EnvOption {
    /// convert to bindings type.
    pub(crate) fn to_option(self) -> binding::ipset_envopt {
        match self {
            EnvOption::Sorted => binding::ipset_envopt_IPSET_ENV_SORTED,
            EnvOption::Quiet => binding::ipset_envopt_IPSET_ENV_QUIET,
            EnvOption::Resolve => binding::ipset_envopt_IPSET_ENV_RESOLVE,
            EnvOption::Exist => binding::ipset_envopt_IPSET_ENV_EXIST,
            EnvOption::ListSetName => binding::ipset_envopt_IPSET_ENV_LIST_SETNAME,
            EnvOption::ListHeader => binding::ipset_envopt_IPSET_ENV_LIST_HEADER,
        }
    }
}

/// Options for creation and addition.
#[derive(Debug)]
pub enum AddOption {
    /// The value of the timeout parameter for the create command means the default timeout value
    /// (in seconds) for new entries. If a set is created with timeout support, then the same
    /// timeout option can  be  used  to  specify  non-default timeout  values when adding entries.
    /// Zero timeout value means the entry is added permanent to the set.
    Timeout(u32),
    /// bytes counter for the element.
    Bytes(u64),
    /// packets counter for the element.
    Packets(u64),
    /// skbmark option format: MARK or MARK/MASK, where MARK and  MASK  are  32bit  hex
    /// numbers  with  0x  prefix. If only mark is specified mask 0xffffffff are used.
    SkbMark(u32, u32),
    /// skbprio option has tc class format: MAJOR:MINOR, where major and minor numbers are
    /// hex without 0x prefix.
    SkbPrio(u16, u16),
    /// skbqueue option is just decimal number.
    SkbQueue(u16),
    /// All set types support the optional comment extension.  Enabling this extension on an ipset
    /// enables you to annotate an ipset entry  with  an  arbitrary  string. This  string is
    /// completely ignored by both the kernel and ipset itself and is purely for providing a
    /// convenient means to document the reason for an entry's existence. Comments must not contain
    /// any quotation marks and the usual escape character (\) has no meaning
    Comment(String),
    /// The  hash  set  types which can store net type of data (i.e. hash:*net*) support the
    /// optional nomatch option when adding entries. When matching elements in the set, entries
    /// marked as nomatch are skipped as if those were not added to the set, which makes possible
    /// to build up sets with exceptions.  See  the  example  at hash type hash:net below.
    /// When  elements  are  tested  by ipset, the nomatch flags are taken into account.
    /// If one wants to test the existence of an element marked with nomatch in a set,
    /// then the flag must be specified too.
    Nomatch,
}

pub struct NormalListResult<T: SetType> {
    pub name: String,
    pub typ: String,
    pub revision: u32,
    pub header: ListHeader,
    pub size_in_memory: u32,
    pub references: u32,
    pub entry_size: u32,
    pub items: Option<Vec<(T::DataType, Option<Vec<AddOption>>)>>,
}

impl<T: SetType> Default for NormalListResult<T> {
    fn default() -> Self {
        Self {
            name: "".to_string(),
            typ: "".to_string(),
            revision: 0,
            header: Default::default(),
            size_in_memory: 0,
            references: 0,
            entry_size: 0,
            items: None,
        }
    }
}

pub enum ListResult<T: SetType> {
    Normal(NormalListResult<T>),
    Terse(Vec<String>),
}

impl<T: SetType> NormalListResult<T> {
    pub(crate) fn update_from_str(&mut self, line: &str) -> Result<(), Error> {
        if self.items.is_none() {
            let fields: Vec<_> = line.splitn(2, ":").collect();
            match fields[0] {
                "Name" => {
                    self.name = fields[1].trim().to_string();
                }
                "Type" => {
                    self.typ = fields[1].trim().to_string();
                }
                "Revision" => {
                    self.revision = fields[1].trim().parse()?;
                }
                "Header" => {
                    self.header = ListHeader::from_str(fields[1].trim());
                }
                "Size in memory" => {
                    self.size_in_memory = fields[1].trim().parse()?;
                }
                "References" => {
                    self.references = fields[1].trim().parse()?;
                }
                "Number of entries" => {
                    self.entry_size = fields[1].trim().parse()?;
                }
                "Members" => {
                    self.items = Some(Vec::new());
                }
                _ => {
                    unreachable!("unexpected {}", fields[0])
                }
            }
        } else {
            let fields: Vec<_> = line.split_ascii_whitespace().collect();
            let mut data = T::DataType::default();
            let mut add_options = None;
            if fields.len() == 0 || data.parse(fields[0]).is_err() {
                return Err(Error::InvalidOutput(String::from(line)));
            } else if fields.len() > 1 {
                let mut i = 1;
                let mut options = vec![];
                while i < fields.len() {
                    match fields[i] {
                        "timeout" => {
                            options.push(AddOption::Timeout(fields[i + 1].parse()?));
                        }
                        "packets" => {
                            options.push(AddOption::Packets(fields[i + 1].parse()?));
                        }
                        "bytes" => {
                            options.push(AddOption::Bytes(fields[i + 1].trim().replace("\0", "").parse()?));
                        }
                        "comment" => {
                            options.push(AddOption::Comment(fields[i + 1].to_string()));
                        }
                        "skbmark" => {
                            let values: Vec<_> = fields[i + 1].split('/').collect();
                            let v0 =
                                u32::from_str_radix(values[0].strip_prefix("0x").unwrap(), 16)?;
                            let v1 = if values.len() > 1 {
                                u32::from_str_radix(values[1].strip_prefix("0x").unwrap(), 16)?
                            } else {
                                u32::MAX
                            };
                            options.push(AddOption::SkbMark(v0, v1));
                        }
                        "skbprio" => {
                            let values: Vec<_> = fields[i + 1].split(':').collect();
                            let v0 = u16::from_str_radix(values[0], 16)?;
                            let v1 = u16::from_str_radix(values[1], 16)?;
                            options.push(AddOption::SkbPrio(v0, v1));
                        }
                        "skbqueue" => {
                            options.push(AddOption::SkbQueue(fields[i + 1].parse()?));
                        }
                        "nomatch" => {
                            options.push(AddOption::Nomatch);
                            i += 1;
                            continue;
                        }
                        _ => {
                            unreachable!("{} not supported", fields[i]);
                        }
                    }
                    i += 2
                }
                add_options = Some(options);
            }
            self.items.as_mut().unwrap().push((data, add_options));
        }
        Ok(())
    }
}

#[derive(Default, Debug)]
pub struct ListHeader {
    ipv6: bool,
    hash_size: u32,
    bucket_size: Option<u32>,
    max_elem: u32,
    counters: bool,
    comment: bool,
    skbinfo: bool,
    initval: Option<u32>
}

impl ListHeader {
    pub fn from_str(s: &str) -> Self {
        let s: Vec<_> = s.split_whitespace().collect();
        let mut header = ListHeader::default();
        let mut i = 0;
        while i < s.len() {
            match s[i] {
                "family" => {
                    header.ipv6 = s[i + 1] == "inet6";
                    i += 2;
                }
                "hashsize" => {
                    header.hash_size = s[i + 1].parse().unwrap();
                    i += 2;
                }
                "bucketsize" => {
                    header.bucket_size = Some(s[i + 1].parse().unwrap());
                    i += 2;
                },
                "maxelem" => {
                    header.max_elem = s[i + 1].parse().unwrap();
                    i += 2;
                }
                "counters" => {
                    header.counters = true;
                    i += 1;
                }
                "comment" => {
                    header.comment = true;
                    i += 1;
                }
                "skbinfo" => {
                    header.skbinfo = true;
                    i += 1;
                }
                "initval" => {
                    if let Some(initval) = s[i + 1].strip_prefix("0x") {
                        header.initval = Some(u32::from_str_radix(initval, 16).unwrap());
                    }
                    i += 2;
                }

                _ => {
                    unreachable!("{} not supported", s[i]);
                }
            }
        }
        header
    }
}
