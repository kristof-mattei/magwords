use url::Url;

#[cfg_attr(not(test), expect(dead_code))]
pub fn add_segments(mut base_url: Url, segments: &[&str]) -> Url {
    {
        let mut s = base_url.path_segments_mut().expect("URL is relative");
        s.extend(segments);
    }

    base_url
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use url::Url;

    use crate::utils::url::add_segments;

    #[test]
    fn test_add_single_segment() {
        let url = Url::from_str("https://example.com").unwrap();

        let new_url = add_segments(url, &["foobar"]);

        assert_eq!(new_url.as_str(), "https://example.com/foobar");
    }

    #[test]
    fn test_multple_segments() {
        let url = Url::from_str("https://example.com").unwrap();

        let new_url = add_segments(url, &["foo", "bar"]);

        assert_eq!(new_url.as_str(), "https://example.com/foo/bar");
    }
}
