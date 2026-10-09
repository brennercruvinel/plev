//! Launch arguments in the browser, which has no process arguments: the
//! values of the page URL's query, in the order the app asks for them
//! (`?page=Data%20Table&theme=hoff` gives `["Data Table", "hoff"]` for
//! `["page", "theme"]`). A missing key ends the list, so the arguments
//! keep their positions as on the desktop's command line.

/// The values of `keys` in the query of `url`, percent-decoded (`+` is a
/// space), up to the first key the query does not have.
pub fn launch_query(url: &str, keys: &[&str]) -> Vec<String> {
    let query = url
        .split_once('?')
        .map_or("", |(_, q)| q)
        .split('#')
        .next()
        .unwrap_or("");
    let pairs: Vec<(String, String)> = query
        .split('&')
        .filter_map(|pair| {
            let (k, v) = pair.split_once('=')?;
            Some((decode(k)?, decode(v)?))
        })
        .collect();
    keys.iter()
        .map_while(|key| pairs.iter().find(|(k, _)| k == key).map(|(_, v)| v.clone()))
        .collect()
}

/// [`launch_query`] of the page's own URL; empty when the document is not
/// there.
#[cfg(target_arch = "wasm32")]
pub fn launch_args(keys: &[&str]) -> Vec<String> {
    web_sys::window()
        .and_then(|w| w.document())
        .and_then(|d| d.url().ok())
        .map_or_else(Vec::new, |url| launch_query(&url, keys))
}

/// Percent-decoding of one query component; `None` for a broken escape
/// or bytes that are not UTF-8.
fn decode(text: &str) -> Option<String> {
    let mut bytes = Vec::with_capacity(text.len());
    let mut rest = text.bytes();
    while let Some(b) = rest.next() {
        match b {
            b'+' => bytes.push(b' '),
            b'%' => {
                let hi = (rest.next()? as char).to_digit(16)?;
                let lo = (rest.next()? as char).to_digit(16)?;
                bytes.push((hi * 16 + lo) as u8);
            }
            b => bytes.push(b),
        }
    }
    String::from_utf8(bytes).ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    const KEYS: &[&str] = &["page", "theme"];

    #[test]
    fn the_values_come_in_the_order_asked() {
        let url = "http://127.0.0.1:8090/?theme=hoff&page=Data%20Table";
        assert_eq!(launch_query(url, KEYS), vec!["Data Table", "hoff"]);
    }

    #[test]
    fn a_plus_is_a_space_and_the_fragment_is_not_the_query() {
        let url = "http://h/?page=Alert+Dialog&theme=dark#top";
        assert_eq!(launch_query(url, KEYS), vec!["Alert Dialog", "dark"]);
    }

    #[test]
    fn a_missing_key_ends_the_list() {
        assert_eq!(
            launch_query("http://h/?theme=hoff", KEYS),
            Vec::<String>::new()
        );
        assert_eq!(launch_query("http://h/?page=Card", KEYS), vec!["Card"]);
        assert_eq!(launch_query("http://h/", KEYS), Vec::<String>::new());
    }

    #[test]
    fn a_broken_escape_is_no_value() {
        assert_eq!(
            launch_query("http://h/?page=%E&theme=x", KEYS),
            Vec::<String>::new()
        );
        assert_eq!(launch_query("http://h/?page=%C3%A9", KEYS), vec!["é"]);
    }
}
