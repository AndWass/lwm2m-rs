mod coap;
mod tlv;

use ::coap::CoAPClient;
use coap_lite::{
    CoapOption, CoapRequest, MessageClass, MessageType, Packet, RequestType as Method, RequestType,
};

pub const VERSION_1_2: &str = "1.2";

pub struct Register {
    pub endpoint: String,
    pub lifetime: u16,
    pub version: &'static str,
}

impl From<Register> for Packet {
    fn from(value: Register) -> Self {
        let Register {
            endpoint,
            lifetime,
            version,
        } = value;

        let mut ret = Packet::new();
        ret.header.code = MessageClass::Request(RequestType::Post);
        ret.add_option(CoapOption::UriPath, "rd".to_string().into_bytes());
        ret.add_option(
            CoapOption::UriQuery,
            format!("ep={}", endpoint).into_bytes(),
        );
        ret.add_option(
            CoapOption::UriQuery,
            format!("lt={}", lifetime).into_bytes(),
        );
        ret.add_option(
            CoapOption::UriQuery,
            format!("lwm2m={}", version).into_bytes(),
        );
        ret.add_option(CoapOption::ContentFormat, format!("40").into_bytes());
        ret.payload = format!("</>;ct=110,</1/0>").into_bytes();
        ret.header.set_type(MessageType::Confirmable);
        ret
    }
}

#[tokio::main]
async fn main() {}
