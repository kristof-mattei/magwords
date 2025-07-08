use color_eyre::eyre::{self, Context};
use url::Url;

#[expect(dead_code)]
pub fn get_env_as_url(key: &str) -> Result<Url, eyre::Report> {
    let value = std::env::var(key)?;

    Url::parse(&value).wrap_err_with(|| format!("Couldn't convert {:?} to URL", value))
}
