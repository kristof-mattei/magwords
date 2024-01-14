use url::Url;

#[allow(dead_code)]
pub fn add_segments(mut base_url: Url, segments: &[&str]) -> Url {
    {
        let mut s = base_url.path_segments_mut().expect("URL is relative");
        s.extend(segments);
    }

    base_url
}
