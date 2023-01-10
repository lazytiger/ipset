use std::ffi::{CString, NulError};
use std::fmt::{Display, Formatter};
use std::net::{AddrParseError, IpAddr};
use std::num::ParseIntError;

use derive_more::From;

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
    fn set_data(&self, session: &Session<T>) -> Result<(), Error> {
        let (ip, family) = match self {
            IpDataType::IPv4(ip) => (ip as *const _ as _, &binding::NFPROTO_IPV4 as *const _ as _),
            IpDataType::IPv6(ip) => (ip as *const _ as _, &binding::NFPROTO_IPV6 as *const _ as _),
        };
        session.set_data(binding::ipset_opt_IPSET_OPT_IP, ip)?;
        session.set_data(binding::ipset_opt_IPSET_OPT_FAMILY, family)
    }
}

impl Parse for IpDataType {
    fn parse(&mut self, s: &str) -> Result<(), Error> {
        let ip: IpAddr = s.parse()?;
        *self = ip.into();
        Ok(())
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
            IpDataType::IPv4(ip) => IpAddr::from(ip.s_addr.to_be_bytes()),
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
pub struct NetDataType {
    ip: IpDataType,
    cidr: u8,
}

impl<T: SetType> SetData<T> for NetDataType {
    fn set_data(&self, session: &Session<T>) -> Result<(), Error> {
        self.ip.set_data(session)?;
        session.set_data(
            binding::ipset_opt_IPSET_OPT_CIDR,
            &self.cidr as *const _ as _,
        )
    }
}

impl Parse for NetDataType {
    fn parse(&mut self, s: &str) -> Result<(), Error> {
        let mut ss = s.split("/");
        let ip = ss.next();
        let cidr = ss.next();
        if ip.is_none() || cidr.is_none() {
            return Err(Error::InvalidOutput(s.into()));
        }

        let ip = ip.unwrap();
        let cidr = cidr.unwrap();

        let ip: IpAddr = ip.parse()?;
        let cidr: u8 = cidr.parse()?;

        self.ip = ip.into();
        self.cidr = cidr;
        Ok(())
    }
}

impl Display for NetDataType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}/{}", self.ip, self.cidr)
    }
}

/// mac data type
pub struct MacDataType {
    mac: [u8; 6],
}

impl Parse for MacDataType {
    fn parse(&mut self, s: &str) -> Result<(), Error> {
        let mac: Vec<u8> = s.split(":").filter_map(|s| s.parse::<u8>().ok()).collect();
        if mac.len() != 6 {
            Err(Error::InvalidOutput(s.into()))
        } else {
            self.mac.copy_from_slice(mac.as_slice());
            Ok(())
        }
    }
}

impl<T: SetType> SetData<T> for MacDataType {
    fn set_data(&self, session: &Session<T>) -> Result<(), Error> {
        session.set_data(binding::ipset_opt_IPSET_OPT_ETHER, self.mac.as_ptr() as _)
    }
}

impl Display for MacDataType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let data = self.mac.map(|d| format!("{:02x}", d)).join(":");
        write!(f, "{}", data)
    }
}

/// port data type
pub struct PortDataType {
    port: u16,
}

impl<T: SetType> SetData<T> for PortDataType {
    fn set_data(&self, session: &Session<T>) -> Result<(), Error> {
        session.set_data(
            binding::ipset_opt_IPSET_OPT_PORT,
            &self.port as *const _ as _,
        )
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

/// iface data type
pub struct IfaceDataType {
    name: CString,
}

impl Parse for IfaceDataType {
    fn parse(&mut self, s: &str) -> Result<(), Error> {
        self.name = CString::new(s)?;
        Ok(())
    }
}

impl<T: SetType> SetData<T> for IfaceDataType {
    fn set_data(&self, session: &Session<T>) -> Result<(), Error> {
        session.set_data(binding::ipset_opt_IPSET_OPT_IFACE, self.name.as_ptr() as _)
    }
}

impl Display for IfaceDataType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name.to_string_lossy())
    }
}

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
    fn set_data(&self, session: &Session<T>) -> Result<(), Error> {
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

pub struct SetDataType {
    name: CString,
}

impl Parse for SetDataType {
    fn parse(&mut self, s: &str) -> Result<(), Error> {
        self.name = CString::new(s)?;
        Ok(())
    }
}

impl<T: SetType> SetData<T> for SetDataType {
    fn set_data(&self, session: &Session<T>) -> Result<(), Error> {
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
impl_name!(A, B);
impl_name!(A, B, C);

macro_rules! impl_set_data {
    ($($types:ident),+) => {
        #[allow(non_snake_case)]
        impl<T, $($types),+> SetData<T> for ($($types),+)
            where T:SetType,
                $($types:SetData<T>),+ {
            fn set_data(&self, session:&Session<T>) -> Result<(), Error> {
                let ($($types),+) = self;
                $($types.set_data(session)?;)+
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

/// All the supported ipset types.
/// TODO hash:net is not fully supported now.
pub trait SetType: Sized {
    type Method;
    type DataType: SetData<Self> + Parse;
}

pub trait TypeName {
    fn name() -> String;
}

pub trait SetData<T: SetType> {
    fn set_data(&self, session: &Session<T>) -> Result<(), Error>;
}

pub trait Parse {
    fn parse(&mut self, s: &str) -> Result<(), Error>;
}

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
#[derive(Debug, From)]
pub enum Error {
    #[from(ignore)]
    DataSet(String, bool),
    #[from(ignore)]
    Cmd(String, bool),
    #[from(ignore)]
    TypeGet(String, bool),
    #[from(ignore)]
    InvalidOutput(String),
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

macro_rules! impl_set_type {
    ($name:ident, $method:ident, $($types:ident),+) => {
        pub struct $name;
        #[allow(unused_parens)]
        impl SetType for $name {
            type Method = concat_idents!($method, Method);
            type DataType = ($(concat_idents!($types, DataType)),+);
        }
    }
}

impl_set_type!(BitmapIp, Bitmap, Ip);
impl_set_type!(BitmapIpMac, Bitmap, Ip, Mac);
impl_set_type!(BitmapPort, Bitmap, Port);
impl_set_type!(HashIp, Hash, Ip);
impl_set_type!(HashMac, Hash, Mac);
impl_set_type!(HashIpMac, Hash, Ip, Mac);
impl_set_type!(HashNet, Hash, Net);
impl_set_type!(HashNetNet, Hash, Net, Net);
impl_set_type!(HashIpPort, Hash, Ip, Port);
impl_set_type!(HashNetPort, Hash, Net, Port);
impl_set_type!(HashIpPortIp, Hash, Ip, Port, Ip);
impl_set_type!(HashIpPortNet, Hash, Ip, Port, Net);
impl_set_type!(HashIpMark, Hash, Ip, Mark);
impl_set_type!(HashNetPortNet, Hash, Net, Port, Net);
impl_set_type!(HashNetIface, Hash, Net, Iface);
impl_set_type!(ListSet, List, Set);
