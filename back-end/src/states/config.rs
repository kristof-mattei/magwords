use std::net::SocketAddr;

#[derive(Clone)]
pub struct Config {
    pub(crate) bind_to: SocketAddr,
}
