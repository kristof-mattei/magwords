use std::net::SocketAddr;

pub struct Config {
    pub(crate) bind_to: SocketAddr,
}
