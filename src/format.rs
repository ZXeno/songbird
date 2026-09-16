use serde_json::Value;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BodyFormat {
    Json,
    Xml,
    Plain,
}

pub fn detect_format(body: &str) -> BodyFormat {
    let trimmed = body.trim_start();
    if trimmed.is_empty() {
        return BodyFormat::Plain;
    }
    if serde_json::from_str::<Value>(trimmed).is_ok() {
        return BodyFormat::Json;
    }
    if trimmed.starts_with('<') {
        return BodyFormat::Xml;
    }
    BodyFormat::Plain
}

pub fn format_body(body: &str, format: BodyFormat) -> String {
    match format {
        BodyFormat::Json => serde_json::from_str::<Value>(body)
            .ok()
            .and_then(|value| serde_json::to_string_pretty(&value).ok())
            .unwrap_or_else(|| body.to_owned()),
        BodyFormat::Xml => format_xml(body),
        BodyFormat::Plain => body.to_owned(),
    }
}

fn format_xml(input: &str) -> String {
    let trimmed = input.trim();
    let mut output = String::with_capacity(trimmed.len() + 64);
    let mut depth = 0usize;
    let mut rest = trimmed;

    while !rest.is_empty() {
        match rest.find('<') {
            Some(0) => {
                let Some(end) = find_tag_end(rest) else {
                    output.push_str(rest);
                    break;
                };
                let tag = &rest[..end];
                rest = &rest[end..];
                let indent_depth;
                if tag.starts_with("</") {
                    depth = depth.saturating_sub(1);
                    indent_depth = depth;
                } else if tag.starts_with("<?") || tag.starts_with("<!") || tag.ends_with("/>") {
                    indent_depth = depth;
                } else {
                    indent_depth = depth;
                    depth += 1;
                }
                output.push_str(&"  ".repeat(indent_depth));
                output.push_str(tag);
                output.push('\n');
            }
            Some(position) => {
                let text = rest[..position].trim();
                rest = &rest[position..];
                if !text.is_empty() {
                    output.push_str(&"  ".repeat(depth));
                    output.push_str(text);
                    output.push('\n');
                }
            }
            None => {
                let text = rest.trim();
                if !text.is_empty() {
                    output.push_str(&"  ".repeat(depth));
                    output.push_str(text);
                    output.push('\n');
                }
                break;
            }
        }
    }

    let trimmed_output = output.trim_end();
    format!("{trimmed_output}\n")
}

fn find_tag_end(from_tag_start: &str) -> Option<usize> {
    let mut quote: Option<char> = None;
    for (index, ch) in from_tag_start.char_indices() {
        match quote {
            Some(open) if ch == open => quote = None,
            Some(_) => {}
            None => {
                if ch == '"' || ch == '\'' {
                    quote = Some(ch);
                } else if ch == '>' {
                    return Some(index + 1);
                }
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_json() {
        assert_eq!(detect_format("{\"a\": 1}"), BodyFormat::Json);
        assert_eq!(detect_format("  [1, 2, 3]"), BodyFormat::Json);
    }

    #[test]
    fn detects_xml() {
        assert_eq!(detect_format("<root/>"), BodyFormat::Xml);
        assert_eq!(
            detect_format("<?xml version=\"1.0\"?><root/>"),
            BodyFormat::Xml
        );
    }

    #[test]
    fn detects_plain() {
        assert_eq!(detect_format("plain text"), BodyFormat::Plain);
        assert_eq!(detect_format("{broken"), BodyFormat::Plain);
        assert_eq!(detect_format(""), BodyFormat::Plain);
        assert_eq!(detect_format("   "), BodyFormat::Plain);
    }

    #[test]
    fn json_is_pretty_printed_with_key_order_preserved() {
        let formatted = format_body("{\"b\": 1, \"a\": {\"z\": 2, \"y\": 3}}", BodyFormat::Json);

        assert_eq!(
            formatted,
            "{\n  \"b\": 1,\n  \"a\": {\n    \"z\": 2,\n    \"y\": 3\n  }\n}"
        );
    }

    #[test]
    fn unparseable_json_falls_back_to_raw() {
        assert_eq!(format_body("{oops", BodyFormat::Json), "{oops");
    }

    #[test]
    fn xml_is_indented() {
        let input = "<?xml version=\"1.0\"?><root><item id=\"1\">alpha</item><empty/><child><x>1</x></child></root>";

        let formatted = format_body(input, BodyFormat::Xml);

        assert_eq!(
            formatted,
            "<?xml version=\"1.0\"?>\n<root>\n  <item id=\"1\">\n    alpha\n  </item>\n  <empty/>\n  <child>\n    <x>\n      1\n    </x>\n  </child>\n</root>\n"
        );
    }

    #[test]
    fn xml_attribute_values_may_contain_gt() {
        let formatted = format_body("<a title=\"x>y\"/>", BodyFormat::Xml);

        assert_eq!(formatted, "<a title=\"x>y\"/>\n");
    }

    #[test]
    fn xml_self_closing_tags_keep_depth() {
        let formatted = format_body("<root><item/><next>deep</next></root>", BodyFormat::Xml);

        assert_eq!(
            formatted,
            "<root>\n  <item/>\n  <next>\n    deep\n  </next>\n</root>\n"
        );
    }

    #[test]
    fn xml_text_between_tags_is_trimmed() {
        let formatted = format_body("<a>  spaced  </a>", BodyFormat::Xml);

        assert_eq!(formatted, "<a>\n  spaced\n</a>\n");
    }

    #[test]
    fn broken_xml_is_kept_verbatim() {
        assert_eq!(format_body("<broken", BodyFormat::Xml), "<broken\n");
    }

    #[test]
    fn plain_bodies_pass_through() {
        assert_eq!(
            format_body("  raw text  ", BodyFormat::Plain),
            "  raw text  "
        );
    }
}
