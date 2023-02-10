mod tlv;
mod coap;

use ::coap::CoAPClient;
use coap_lite::{CoapOption, CoapRequest, MessageClass, MessageType, Packet, RequestType as Method, RequestType};

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
            version
        } = value;

        let mut ret = Packet::new();
        ret.header.code = MessageClass::Request(RequestType::Post);
        ret.add_option(CoapOption::UriPath,"rd".to_string().into_bytes());
        ret.add_option(CoapOption::UriQuery, format!("ep={}", endpoint).into_bytes());
        ret.add_option(CoapOption::UriQuery, format!("lt={}", lifetime).into_bytes());
        ret.add_option(CoapOption::UriQuery, format!("lwm2m={}", version).into_bytes());
        ret.add_option(CoapOption::ContentFormat, format!("40").into_bytes());
        ret.payload = format!("</>;ct=110,</1/0>").into_bytes();
        ret.header.set_type(MessageType::Confirmable);
        ret
    }
}

#[tokio::main]
async fn main() {
    let mut client = coap::Client::new("127.0.0.1:5683").await.unwrap();
    let register = Register {
        endpoint: "abcd=0".to_string(),
        lifetime: 270,
        version: VERSION_1_2
    };

    let mut register = register.into();
    println!("TX: {:?}", client.send(&register));
    let response = client.receive2(&mut register);
    println!("{:?}", response);
}
