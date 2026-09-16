use base64::Engine;
use base64::engine::general_purpose::STANDARD as BASE64;
use shlex::split;
use thiserror::Error;

use crate::core::{Header, HttpMethod, HttpRequest};

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum CurlError {
    #[error("command is empty")]
    Empty,
    #[error("no URL found in command")]
    MissingUrl,
    #[error("command has unterminated quotes")]
    UnterminatedQuote,
    #[error("invalid HTTP method '{0}'")]
    InvalidMethod(String),
}

const VALUE_FLAGS_IGNORED: &[&str] = &[
    "output",
    "max-time",
    "connect-timeout",
    "retry",
    "cacert",
    "capath",
    "cert",
    "key",
    "proxy",
    "noproxy",
    "interface",
    "limit-rate",
    "resolve",
    "dump-header",
    "cookie-jar",
    "config",
    "netrc-file",
    "engine",
];

const BOOL_FLAGS_IGNORED: &[&str] = &[
    "compressed",
    "insecure",
    "silent",
    "show-error",
    "verbose",
    "fail",
    "location",
    "no-buffer",
    "progress-bar",
    "http1.0",
    "http1.1",
    "http2",
    "http2-prior-knowledge",
    "http3",
    "ipv4",
    "ipv6",
    "next",
];

pub fn parse(input: &str) -> Result<HttpRequest, CurlError> {
    let normalized = input.replace("\r\n", "\n");
    let tokens = split(normalized.trim()).ok_or(CurlError::UnterminatedQuote)?;

    let args = match tokens.split_first() {
        Some((first, rest)) if is_curl_program(first) => rest,
        Some(_) => &tokens[..],
        None => return Err(CurlError::Empty),
    };
    if args.is_empty() {
        return Err(CurlError::Empty);
    }

    let mut method: Option<HttpMethod> = None;
    let mut force_get = false;
    let mut url: Option<String> = None;
    let mut headers: Vec<Header> = Vec::new();
    let mut data_parts: Vec<String> = Vec::new();
    let mut is_form = false;

    let mut index = 0;
    while index < args.len() {
        let token = args[index].as_str();
        index += 1;

        if token == "--" {
            if url.is_none() {
                url = args.get(index).cloned();
            }
            break;
        }

        if let Some(long) = token.strip_prefix("--") {
            let (name, inline_value) = match long.split_once('=') {
                Some((name, value)) => (name, Some(value)),
                None => (long, None),
            };
            match name {
                "request" => {
                    if let Some(value) = take_value(inline_value, args, &mut index) {
                        method = Some(
                            value
                                .parse()
                                .map_err(|_| CurlError::InvalidMethod(value.clone()))?,
                        );
                    }
                }
                "header" => {
                    if let Some(value) = take_value(inline_value, args, &mut index) {
                        push_header(&mut headers, &value);
                    }
                }
                "url" => {
                    if let Some(value) = take_value(inline_value, args, &mut index) {
                        url = Some(value);
                    }
                }
                "data" | "data-raw" | "data-binary" | "data-ascii" | "data-urlencode" => {
                    if let Some(value) = take_value(inline_value, args, &mut index) {
                        data_parts.push(value);
                    }
                }
                "json" => {
                    if let Some(value) = take_value(inline_value, args, &mut index) {
                        data_parts.push(value);
                        push_header(&mut headers, "Content-Type: application/json");
                        push_header(&mut headers, "Accept: application/json");
                    }
                }
                "form" => {
                    if let Some(value) = take_value(inline_value, args, &mut index) {
                        is_form = true;
                        data_parts.push(value);
                    }
                }
                "user" => {
                    if let Some(value) = take_value(inline_value, args, &mut index) {
                        let encoded = BASE64.encode(value.as_bytes());
                        push_header(&mut headers, &format!("Authorization: Basic {encoded}"));
                    }
                }
                "user-agent" => {
                    if let Some(value) = take_value(inline_value, args, &mut index) {
                        push_header(&mut headers, &format!("User-Agent: {value}"));
                    }
                }
                "referer" => {
                    if let Some(value) = take_value(inline_value, args, &mut index) {
                        push_header(&mut headers, &format!("Referer: {value}"));
                    }
                }
                "cookie" => {
                    if let Some(value) = take_value(inline_value, args, &mut index) {
                        if !value.starts_with('@') {
                            push_header(&mut headers, &format!("Cookie: {value}"));
                        }
                    }
                }
                "get" => force_get = true,
                "head" => method = Some(HttpMethod::Head),
                _ if BOOL_FLAGS_IGNORED.contains(&name) => {}
                _ if VALUE_FLAGS_IGNORED.contains(&name) => {
                    take_value(inline_value, args, &mut index);
                }
                _ => {}
            }
            continue;
        }

        if let Some(shorts) = token.strip_prefix('-') {
            let mut rest = shorts;
            while let Some(flag) = rest.chars().next() {
                rest = &rest[flag.len_utf8()..];
                match flag {
                    'X' => {
                        if let Some(value) = short_value(&mut rest, args, &mut index) {
                            method = Some(
                                value
                                    .parse()
                                    .map_err(|_| CurlError::InvalidMethod(value.clone()))?,
                            );
                        }
                    }
                    'H' => {
                        if let Some(value) = short_value(&mut rest, args, &mut index) {
                            push_header(&mut headers, &value);
                        }
                    }
                    'd' => {
                        if let Some(value) = short_value(&mut rest, args, &mut index) {
                            data_parts.push(value);
                        }
                    }
                    'u' => {
                        if let Some(value) = short_value(&mut rest, args, &mut index) {
                            let encoded = BASE64.encode(value.as_bytes());
                            push_header(&mut headers, &format!("Authorization: Basic {encoded}"));
                        }
                    }
                    'A' => {
                        if let Some(value) = short_value(&mut rest, args, &mut index) {
                            push_header(&mut headers, &format!("User-Agent: {value}"));
                        }
                    }
                    'e' => {
                        if let Some(value) = short_value(&mut rest, args, &mut index) {
                            push_header(&mut headers, &format!("Referer: {value}"));
                        }
                    }
                    'b' => {
                        if let Some(value) = short_value(&mut rest, args, &mut index) {
                            if !value.starts_with('@') {
                                push_header(&mut headers, &format!("Cookie: {value}"));
                            }
                        }
                    }
                    'F' => {
                        if let Some(value) = short_value(&mut rest, args, &mut index) {
                            is_form = true;
                            data_parts.push(value);
                        }
                    }
                    'G' => force_get = true,
                    'I' => method = Some(HttpMethod::Head),
                    'o' | 'D' | 'm' | 'x' | 'c' | 'C' | 'Q' | 'T' | 'K' => {
                        short_value(&mut rest, args, &mut index);
                    }
                    'q' | 's' | 'S' | 'k' | 'v' | 'i' | 'L' | 'f' | '#' | 'g' | 'B' | 'n' | 'N'
                    | '0' | '1' | '2' | '4' | '_' | '-' => {}
                    _ => {}
                }
            }
            continue;
        }

        if url.is_none() {
            url = Some(token.to_owned());
        }
    }

    let mut url = url.map(complete_scheme).ok_or(CurlError::MissingUrl)?;

    if force_get && !data_parts.is_empty() {
        let separator = if url.contains('?') { '&' } else { '?' };
        url = format!("{url}{separator}{}", data_parts.join("&"));
        data_parts.clear();
    }

    let resolved_method = method.unwrap_or({
        if data_parts.is_empty() {
            HttpMethod::Get
        } else {
            HttpMethod::Post
        }
    });

    let mut request = HttpRequest::new(resolved_method, url).with_headers(headers);

    if !data_parts.is_empty() {
        let body = data_parts.join("&");
        request = request.with_body(body);
        if is_form {
            let has_content_type = request
                .headers
                .iter()
                .any(|header| header.name.eq_ignore_ascii_case("Content-Type"));
            if !has_content_type {
                request = request.with_header("Content-Type", "application/x-www-form-urlencoded");
            }
        }
    }

    Ok(request)
}

fn is_curl_program(first: &str) -> bool {
    first.eq_ignore_ascii_case("curl") || first.ends_with("/curl")
}

fn take_value(inline: Option<&str>, args: &[String], index: &mut usize) -> Option<String> {
    if let Some(value) = inline {
        return Some(value.to_owned());
    }
    args.get(*index).map(|value| {
        *index += 1;
        value.clone()
    })
}

fn short_value(rest: &mut &str, args: &[String], index: &mut usize) -> Option<String> {
    if rest.is_empty() {
        take_value(None, args, index)
    } else {
        let value = (*rest).to_owned();
        *rest = "";
        Some(value)
    }
}

fn push_header(headers: &mut Vec<Header>, raw: &str) {
    if let Some((name, value)) = raw.split_once(':') {
        let name = name.trim();
        if !name.is_empty() {
            headers.push(Header::new(name, value.trim()));
        }
    }
}

fn complete_scheme(url: String) -> String {
    if url.contains("://") {
        url
    } else {
        format!("https://{url}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_plain_get() {
        let request = parse("curl https://example.test/api").unwrap();

        assert_eq!(request.method, HttpMethod::Get);
        assert_eq!(request.url, "https://example.test/api");
        assert!(request.headers.is_empty());
        assert!(request.body.is_none());
    }

    #[test]
    fn parses_method_headers_and_body() {
        let request = parse(
            "curl -X POST https://api.test/users -H 'X-Token: abc' \
             -H 'Content-Type: application/json' -d '{\"name\": \"jessie\"}'",
        )
        .unwrap();

        assert_eq!(request.method, HttpMethod::Post);
        assert_eq!(request.body.as_deref(), Some("{\"name\": \"jessie\"}"));
        assert!(request.headers.contains(&Header::new("X-Token", "abc")));
        assert!(
            request
                .headers
                .contains(&Header::new("Content-Type", "application/json"))
        );
    }

    #[test]
    fn infers_post_from_data() {
        let request = parse("curl -d 'a=1' https://api.test").unwrap();

        assert_eq!(request.method, HttpMethod::Post);
    }

    #[test]
    fn joins_multiple_data_parts() {
        let request = parse("curl -d 'a=1' -d 'b=2' https://api.test").unwrap();

        assert_eq!(request.body.as_deref(), Some("a=1&b=2"));
    }

    #[test]
    fn parses_json_flag() {
        let request = parse(r#"curl --json '{"ok":true}' https://api.test"#).unwrap();

        assert_eq!(request.method, HttpMethod::Post);
        assert_eq!(request.body.as_deref(), Some("{\"ok\":true}"));
        assert!(
            request
                .headers
                .contains(&Header::new("Content-Type", "application/json"))
        );
        assert!(
            request
                .headers
                .contains(&Header::new("Accept", "application/json"))
        );
    }

    #[test]
    fn encodes_basic_auth() {
        let request = parse("curl -u jessie:s3cret https://api.test").unwrap();

        assert!(
            request
                .headers
                .contains(&Header::new("Authorization", "Basic amVzc2llOnMzY3JldA=="))
        );
    }

    #[test]
    fn parses_attached_short_value() {
        let request = parse("curl -XPOST https://api.test").unwrap();

        assert_eq!(request.method, HttpMethod::Post);
    }

    #[test]
    fn parses_long_flag_with_equals() {
        let request = parse("curl --header='X-A: 1' https://api.test").unwrap();

        assert!(request.headers.contains(&Header::new("X-A", "1")));
    }

    #[test]
    fn handles_clustered_flags() {
        let request = parse("curl -sSk https://api.test").unwrap();

        assert_eq!(request.method, HttpMethod::Get);
        assert_eq!(request.url, "https://api.test");
        assert!(request.headers.is_empty());
    }

    #[test]
    fn defaults_scheme_to_https() {
        let request = parse("curl example.test/api").unwrap();

        assert_eq!(request.url, "https://example.test/api");
    }

    #[test]
    fn get_with_data_moves_body_into_query() {
        let request = parse("curl -G -d 'q=rust' -d 'page=2' https://api.test/search").unwrap();

        assert_eq!(request.method, HttpMethod::Get);
        assert_eq!(request.url, "https://api.test/search?q=rust&page=2");
        assert!(request.body.is_none());
    }

    #[test]
    fn form_data_gets_urlencoded_content_type() {
        let request = parse("curl -F 'name=jessie' https://api.test").unwrap();

        assert_eq!(request.method, HttpMethod::Post);
        assert_eq!(request.body.as_deref(), Some("name=jessie"));
        assert!(request.headers.contains(&Header::new(
            "Content-Type",
            "application/x-www-form-urlencoded"
        )));
    }

    #[test]
    fn ignores_unknown_flags_and_noise() {
        let request =
            parse("curl --compressed --max-time 5 -sS https://api.test -o out.json").unwrap();

        assert_eq!(request.url, "https://api.test");
        assert!(request.headers.is_empty());
    }

    #[test]
    fn head_request_is_detected() {
        let request = parse("curl --head https://api.test").unwrap();

        assert_eq!(request.method, HttpMethod::Head);
    }

    #[test]
    fn double_dash_prefixes_positional_url() {
        let request = parse("curl -X GET -- https://api.test").unwrap();

        assert_eq!(request.url, "https://api.test");
    }

    #[test]
    fn parses_backslash_continued_multiline_command() {
        let request = parse(concat!(
            "curl -X POST 'https://api.test/users' \\\n",
            "  -H 'X-Token: abc' \\\n",
            "  -d '{\"name\": \"jessie\"}'",
        ))
        .unwrap();

        assert_eq!(request.method, HttpMethod::Post);
        assert_eq!(request.url, "https://api.test/users");
        assert_eq!(request.body.as_deref(), Some("{\"name\": \"jessie\"}"));
        assert!(request.headers.contains(&Header::new("X-Token", "abc")));
    }

    #[test]
    fn parses_command_split_across_raw_newlines() {
        let request = parse("curl https://api.test/api\n  -H 'X-A: 1'\n  -d 'x=1'").unwrap();

        assert_eq!(request.url, "https://api.test/api");
        assert_eq!(request.method, HttpMethod::Post);
        assert!(request.headers.contains(&Header::new("X-A", "1")));
    }

    #[test]
    fn parses_multiline_command_with_windows_line_endings() {
        let request = parse(
            "curl -X POST 'https://api.test/users' \\\r\n  -H 'X-Token: abc' \\\r\n  -d 'a=1'",
        )
        .unwrap();

        assert_eq!(request.url, "https://api.test/users");
        assert!(request.headers.contains(&Header::new("X-Token", "abc")));
        assert_eq!(request.body.as_deref(), Some("a=1"));
    }

    #[test]
    fn empty_command_is_rejected() {
        assert_eq!(parse("   "), Err(CurlError::Empty));
        assert_eq!(parse("curl"), Err(CurlError::Empty));
    }

    #[test]
    fn missing_url_is_rejected() {
        assert_eq!(parse("curl -X POST"), Err(CurlError::MissingUrl));
    }

    #[test]
    fn unterminated_quotes_are_rejected() {
        assert_eq!(
            parse("curl 'https://api.test"),
            Err(CurlError::UnterminatedQuote)
        );
    }

    #[test]
    fn invalid_method_is_rejected() {
        assert_eq!(
            parse("curl -X BOGUS https://api.test"),
            Err(CurlError::InvalidMethod("BOGUS".to_owned()))
        );
    }
}
