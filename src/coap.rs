use bytes::{BufMut, BytesMut};
use coap_lite::error::MessageError;
use coap_lite::{CoapRequest, CoapResponse, MessageType, Packet};
use std::net::{Ipv4Addr, SocketAddr};
use tokio::net::{ToSocketAddrs, UdpSocket};

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("IO Error '{0}'")]
    IO(#[from] std::io::Error),

    #[error("Packet error '{0}'")]
    Packet(#[from] coap_lite::error::MessageError),
}

type Result<T> = core::result::Result<T, Error>;

struct Buffer {
    inner: BytesMut,
}

impl Buffer {
    pub fn new() -> Self {
        Self {
            inner: BytesMut::with_capacity(1200),
        }
    }

    pub unsafe fn writable_buffer(&mut self) -> &mut [u8] {
        std::slice::from_raw_parts_mut(self.inner.as_mut_ptr(), self.inner.capacity())
    }

    pub unsafe fn readable_buffer(&self, size: usize) -> &[u8] {
        std::slice::from_raw_parts(self.inner.as_ptr(), self.inner.capacity())
    }
}

struct ResponseCacheItem {
    message_id: u16,
    inserted_timestamp: std::time::Instant,
    response: Vec<u8>,
}

#[derive(Default)]
struct ResponseCache {
    storage: Vec<ResponseCacheItem>,
}

impl ResponseCache {
    fn add(&mut self, message_id: u16, response: Vec<u8>) {
        let new_item = ResponseCacheItem {
            message_id,
            inserted_timestamp: std::time::Instant::now(),
            response,
        };

        if self.storage.is_empty() {
            self.storage.push(new_item);
            return;
        }

        let mut oldest_item = 0;
        for i in 0..self.storage.len() {
            if self.storage[i].message_id == message_id {
                self.storage[i] = new_item;
                return;
            } else if self.storage[i].inserted_timestamp
                < self.storage[oldest_item].inserted_timestamp
            {
                oldest_item = i;
            }
        }

        let now = std::time::Instant::now();

        if (now - self.storage[oldest_item].inserted_timestamp).as_secs() > 60 {
            self.storage[oldest_item] = new_item;
        } else {
            self.storage.push(new_item);
        }
    }

    fn get_response(&self, message_id: u16) -> Option<&Vec<u8>> {
        self.storage
            .iter()
            .find(|x| x.message_id == message_id)
            .map(|x| &x.response)
    }
}

pub struct Client {
    socket: UdpSocket,
    read_buffer: Buffer,
    next_message_id: u16,
    response_cache: ResponseCache,
}

impl Client {
    fn next_message_id(&mut self) -> u16 {
        let ret = self.next_message_id;
        self.next_message_id = self.next_message_id.wrapping_add(1);
        ret
    }

    async fn receive_from_peer(&mut self) -> Result<Packet> {
        loop {
            let (amount, from) = self
                .socket
                .recv_from(unsafe { self.read_buffer.writable_buffer() })
                .await?;
            if from == self.socket.peer_addr().unwrap() {
                let packet = Packet::from_bytes(unsafe {
                    self.read_buffer.readable_buffer(amount)
                })?;

                return Ok(packet);
            }
        }
    }

    async fn send_cached_response(&mut self, packet: &Packet) -> Result<bool> {
        if matches!(packet.header.get_type(), MessageType::Confirmable) {
            if let Some(response) = self.response_cache.get_response(packet.header.message_id) {
                self.socket.send(response.as_slice()).await?;

                return Ok(true);
            }
        }
        Ok(false)
    }

    pub async fn new<A: ToSocketAddrs>(target: A) -> Result<Self> {
        let socket = UdpSocket::bind((Ipv4Addr::new(0, 0, 0, 0), 0)).await?;
        socket.connect(target).await?;
        Ok(Self {
            socket,
            read_buffer: Buffer::new(),
            next_message_id: 1231,
            response_cache: Default::default()
        })
    }

    /// TODO! Handle tokens
    pub async fn send(&mut self, mut packet: Packet) -> Result<()> {
        packet.header.message_id = self.next_message_id();
        self.socket.send(packet.to_bytes()?.as_slice()).await?;
        Ok(())
    }

    /// TODO! Handle tokens
    pub async fn receive(&mut self) -> Result<Packet> {
        loop {
            let packet = self.receive_from_peer().await?;
            if !self.send_cached_response(&packet).await? {
                return Ok(packet);
            }
        }
    }

    pub async fn send_response(&mut self, response: Packet) -> Result<()> {
        match response.header.get_type() {
            MessageType::Acknowledgement => {}
            MessageType::Reset => {}
            _ => return Err(Error::Packet(MessageError::InvalidHeader)),
        };

        let bytes = response.to_bytes()?;

        Ok(())
    }
}
