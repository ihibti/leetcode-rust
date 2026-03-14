pub fn extract_slug(url: &str) -> Option<String> {
    let url = url.trim();
    if url.is_empty() {
        return None;
    }

    let url = url.split('?').next().unwrap_or(url);
    let url = url.split('#').next().unwrap_or(url);

    let url = url
        .strip_prefix("https://")
        .or_else(|| url.strip_prefix("http://"))?;

    if !url.starts_with("leetcode.com/") {
        return None;
    }

    let path = url.strip_prefix("leetcode.com")?;
    let path = path.strip_prefix("/problems/")?;

    let slug = path.split('/').next().unwrap_or(path);
    let slug = slug.trim();

    if slug.is_empty() {
        return None;
    }

    Some(slug.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn slug_basic_url() {
        assert_eq!(
            extract_slug("https://leetcode.com/problems/two-sum/"),
            Some("two-sum".into())
        );
    }

    #[test]
    fn slug_no_trailing_slash() {
        assert_eq!(
            extract_slug("https://leetcode.com/problems/two-sum"),
            Some("two-sum".into())
        );
    }

    #[test]
    fn slug_with_description_suffix() {
        assert_eq!(
            extract_slug("https://leetcode.com/problems/two-sum/description/"),
            Some("two-sum".into())
        );
    }

    #[test]
    fn slug_with_query_params() {
        assert_eq!(
            extract_slug("https://leetcode.com/problems/two-sum/?envType=daily-question&envId=2024-01-01"),
            Some("two-sum".into())
        );
    }

    #[test]
    fn slug_with_fragment() {
        assert_eq!(
            extract_slug("https://leetcode.com/problems/two-sum/#description"),
            Some("two-sum".into())
        );
    }

    #[test]
    fn slug_http_url() {
        assert_eq!(
            extract_slug("http://leetcode.com/problems/two-sum/"),
            Some("two-sum".into())
        );
    }

    #[test]
    fn slug_not_leetcode() {
        assert_eq!(extract_slug("https://example.com/problems/two-sum/"), None);
    }

    #[test]
    fn slug_missing_problems_path() {
        assert_eq!(extract_slug("https://leetcode.com/contest/weekly/"), None);
    }

    #[test]
    fn slug_empty() {
        assert_eq!(extract_slug(""), None);
    }

    #[test]
    fn slug_garbage() {
        assert_eq!(extract_slug("not a url at all"), None);
    }

    #[test]
    fn slug_just_domain() {
        assert_eq!(extract_slug("https://leetcode.com/"), None);
    }

    #[test]
    fn slug_problems_but_no_slug() {
        assert_eq!(extract_slug("https://leetcode.com/problems/"), None);
    }

    #[test]
    fn slug_with_description_and_query() {
        assert_eq!(
            extract_slug("https://leetcode.com/problems/valid-parentheses/description/?envType=study-plan"),
            Some("valid-parentheses".into())
        );
    }

    #[test]
    fn slug_complex_name() {
        assert_eq!(
            extract_slug("https://leetcode.com/problems/longest-substring-without-repeating-characters/"),
            Some("longest-substring-without-repeating-characters".into())
        );
    }

    #[test]
    fn slug_numeric_name() {
        assert_eq!(
            extract_slug("https://leetcode.com/problems/3sum/"),
            Some("3sum".into())
        );
    }
}
