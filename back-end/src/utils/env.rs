use color_eyre::eyre::Context;
use color_eyre::Report;
use url::Url;

pub fn get_env_as_url(key: &str) -> Result<Url, Report> {
    let value = std::env::var(key)?;

    Url::parse(&value).wrap_err_with(|| format!("Couldn't convert {:?} to URL", value))
}
