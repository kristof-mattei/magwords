use clap::Parser;
use tracing::{Level, event};

#[derive(Parser, Debug)]
pub struct Cli {
    #[clap(env, long, default_value_t = 990)]
    pub fridge_width: u32,

    #[clap(env, long, default_value_t = 1600)]
    pub fridge_height: u32,
}
impl Cli {
    pub fn print(&self) {
        event!(Level::INFO, fridge_width = %self.fridge_width, fridge_height = %self.fridge_height, "Fridge dimensions");
    }
}
