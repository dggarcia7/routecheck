// Route pattern parsing and matching.
//
// Patterns look like path segments separated by '/':
//   /users/:id          -> named parameter
//   /files/*rest        -> wildcard, must be the last segment, captures the remainder
//   /users              -> static segment, matched literally

#[derive(Debug, Clone, PartialEq)]
pub enum Segment {
    Static(String),
    Param(String),
    Wildcard(String),
}

#[derive(Debug)]
pub struct Route {
    pub method: String,
    pub pattern: String,
    pub segments: Vec<Segment>,
    pub line: usize,
}

/// Splits a path or pattern into its non-empty segments, so "/a//b/" and "a/b"
/// both become ["a", "b"]. This is what lets us compare a request path against
/// a pattern segment by segment instead of doing string matching on the whole thing.
pub fn split_path(path: &str) -> Vec<&str> {
    path.trim_matches('/').split('/').filter(|s| !s.is_empty()).collect()
}

pub fn parse_pattern(pattern: &str) -> Result<Vec<Segment>, String> {
    let raw = split_path(pattern);
    let mut segments = Vec::with_capacity(raw.len());
    for (i, part) in raw.iter().enumerate() {
        if let Some(name) = part.strip_prefix(':') {
            if name.is_empty() {
                return Err("parameter segment ':' is missing a name".to_string());
            }
            segments.push(Segment::Param(name.to_string()));
        } else if let Some(name) = part.strip_prefix('*') {
            if name.is_empty() {
                return Err("wildcard segment '*' is missing a name".to_string());
            }
            if i != raw.len() - 1 {
                return Err(format!("wildcard '*{name}' must be the last segment"));
            }
            segments.push(Segment::Wildcard(name.to_string()));
        } else {
            segments.push(Segment::Static(part.to_string()));
        }
    }
    Ok(segments)
}

/// Parses one line of a routes file. Expected shape: "METHOD /pattern", where
/// METHOD is a verb like GET or POST, or "*" to match any method. Blank lines
/// and lines starting with '#' are skipped by the caller before this runs.
pub fn parse_route_line(line: &str, line_no: usize) -> Result<Route, String> {
    let mut parts = line.splitn(2, char::is_whitespace);
    let method = parts.next().unwrap_or("").trim();
    let pattern = parts.next().unwrap_or("").trim();
    if method.is_empty() || pattern.is_empty() {
        return Err(format!(
            "line {line_no}: expected \"METHOD /pattern\", got {line:?}"
        ));
    }
    let segments = parse_pattern(pattern).map_err(|e| format!("line {line_no}: {e}"))?;
    Ok(Route {
        method: method.to_string(),
        pattern: pattern.to_string(),
        segments,
        line: line_no,
    })
}

/// Tries to match a route's segments against a request path's segments.
/// Returns the captured parameters in the order they appear in the pattern,
/// or None if the shapes don't line up.
pub fn match_segments(segments: &[Segment], path_segs: &[&str]) -> Option<Vec<(String, String)>> {
    let mut params = Vec::new();
    let mut i = 0;
    for segment in segments {
        match segment {
            Segment::Wildcard(name) => {
                let rest = path_segs[i..].join("/");
                params.push((name.clone(), rest));
                return Some(params);
            }
            Segment::Static(expected) => {
                if path_segs.get(i) != Some(&expected.as_str()) {
                    return None;
                }
                i += 1;
            }
            Segment::Param(name) => {
                let value = path_segs.get(i)?;
                params.push((name.clone(), value.to_string()));
                i += 1;
            }
        }
    }
    if i == path_segs.len() {
        Some(params)
    } else {
        None
    }
}

pub fn method_matches(route_method: &str, request_method: &str) -> bool {
    route_method == "*" || route_method.eq_ignore_ascii_case(request_method)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn static_route_matches_exact_path() {
        let segs = parse_pattern("/users/all").unwrap();
        assert_eq!(match_segments(&segs, &["users", "all"]), Some(vec![]));
        assert_eq!(match_segments(&segs, &["users", "other"]), None);
    }

    #[test]
    fn param_captures_single_segment() {
        let segs = parse_pattern("/users/:id").unwrap();
        assert_eq!(
            match_segments(&segs, &["users", "42"]),
            Some(vec![("id".to_string(), "42".to_string())])
        );
    }

    #[test]
    fn wildcard_captures_remainder() {
        let segs = parse_pattern("/files/*rest").unwrap();
        assert_eq!(
            match_segments(&segs, &["files", "a", "b.txt"]),
            Some(vec![("rest".to_string(), "a/b.txt".to_string())])
        );
    }

    #[test]
    fn wildcard_must_be_last() {
        assert!(parse_pattern("/files/*rest/more").is_err());
    }
}
