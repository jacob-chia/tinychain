#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Block {
    #[prost(message, optional, tag = "1")]
    pub header: ::core::option::Option<BlockHeader>,
    #[prost(message, repeated, tag = "2")]
    pub txs: ::prost::alloc::vec::Vec<SignedTx>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct BlockHeader {
    #[prost(bytes = "vec", tag = "1")]
    pub parent_hash: ::prost::alloc::vec::Vec<u8>,
    #[prost(uint64, tag = "2")]
    pub number: u64,
    #[prost(uint64, tag = "3")]
    pub nonce: u64,
    #[prost(uint64, tag = "4")]
    pub timestamp: u64,
    #[prost(string, tag = "5")]
    pub author: ::prost::alloc::string::String,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SignedTx {
    #[prost(message, optional, tag = "1")]
    pub tx: ::core::option::Option<Tx>,
    #[prost(bytes = "vec", tag = "2")]
    pub sig: ::prost::alloc::vec::Vec<u8>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Tx {
    #[prost(string, tag = "1")]
    pub from: ::prost::alloc::string::String,
    #[prost(string, tag = "2")]
    pub to: ::prost::alloc::string::String,
    #[prost(uint64, tag = "3")]
    pub value: u64,
    #[prost(uint64, tag = "4")]
    pub nonce: u64,
    #[prost(uint64, tag = "5")]
    pub gas: u64,
    #[prost(uint64, tag = "6")]
    pub gas_price: u64,
    #[prost(uint64, tag = "7")]
    pub timestamp: u64,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Request {
    #[prost(enumeration = "Method", tag = "1")]
    pub method: i32,
    #[prost(oneof = "request::Body", tags = "2, 3")]
    pub body: ::core::option::Option<request::Body>,
}
/// Nested message and enum types in `Request`.
pub mod request {
    #[allow(clippy::derive_partial_eq_without_eq)]
    #[derive(Clone, PartialEq, ::prost::Oneof)]
    pub enum Body {
        #[prost(message, tag = "2")]
        BlockHeightReq(super::BlockHeightReq),
        #[prost(message, tag = "3")]
        BlocksReq(super::BlocksReq),
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Response {
    #[prost(enumeration = "Method", tag = "1")]
    pub method: i32,
    #[prost(oneof = "response::Body", tags = "2, 3")]
    pub body: ::core::option::Option<response::Body>,
}
/// Nested message and enum types in `Response`.
pub mod response {
    #[allow(clippy::derive_partial_eq_without_eq)]
    #[derive(Clone, PartialEq, ::prost::Oneof)]
    pub enum Body {
        #[prost(message, tag = "2")]
        BlockHeightResp(super::BlockHeightResp),
        #[prost(message, tag = "3")]
        BlocksResp(super::BlocksResp),
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct BlockHeightReq {}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct BlockHeightResp {
    #[prost(uint64, tag = "1")]
    pub block_height: u64,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct BlocksReq {
    /// Start with given block number.
    #[prost(uint64, tag = "2")]
    pub from_number: u64,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct BlocksResp {
    #[prost(message, repeated, tag = "1")]
    pub blocks: ::prost::alloc::vec::Vec<Block>,
}
/// Request/response methods.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
#[repr(i32)]
pub enum Method {
    Height = 0,
    Blocks = 1,
}
impl Method {
    /// String value of the enum field names used in the ProtoBuf definition.
    ///
    /// The values are not transformed in any way and thus are considered stable
    /// (if the ProtoBuf definition does not change) and safe for programmatic use.
    pub fn as_str_name(&self) -> &'static str {
        match self {
            Method::Height => "HEIGHT",
            Method::Blocks => "BLOCKS",
        }
    }
    /// Creates an enum from field names used in the ProtoBuf definition.
    pub fn from_str_name(value: &str) -> ::core::option::Option<Self> {
        match value {
            "HEIGHT" => Some(Self::Height),
            "BLOCKS" => Some(Self::Blocks),
            _ => None,
        }
    }
}
