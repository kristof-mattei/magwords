use std::net::SocketAddr;

#[derive(Copy, Clone)]
pub struct FridgeDimensions {
    pub fridge_width: u32,
    pub fridge_height: u32,
}
pub struct Config {
    pub bind_to: SocketAddr,
    pub fridge_dimensions: FridgeDimensions,
}
