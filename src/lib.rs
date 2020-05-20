// Copyright 2020 Folyd
// Copyright 1999 Google LLC
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     https://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.
//

#![allow(unused_variables)]

pub use matcher::RobotsMatcher;
use parser::RobotsTxtParser;

pub mod matcher;
pub mod parser;

/// Handler for directives found in robots.txt.
pub trait RobotsParseHandler {
    fn handle_robots_start(&mut self);
    fn handle_robots_end(&mut self);
    fn handle_user_agent(&mut self, line_num: u32, user_agent: &str);
    fn handle_allow(&mut self, line_num: u32, value: &str);
    fn handle_disallow(&mut self, line_num: u32, value: &str);
    fn handle_sitemap(&mut self, line_num: u32, value: &str);
    /// Any other unrecognized name/value pairs.
    fn handle_unknown_action(&mut self, line_num: u32, action: &str, value: &str);
}

/// Extracts path (with params) and query part from URL. Removes scheme,
/// authority, and fragment. Result always starts with "/".
/// Returns "/" if the url doesn't have a path or is not valid.
pub fn get_path_params_query(url: &str) -> String {
    fn find_first_of(s: &str, pattern: &str, start_position: usize) -> Option<usize> {
        s[start_position..]
            .find(|c| pattern.contains(c))
            .map(|pos| pos + start_position)
    }
    fn find(s: &str, pattern: &str, start_position: usize) -> Option<usize> {
        s[start_position..]
            .find(pattern)
            .map(|pos| pos + start_position)
    }

    // Initial two slashes are ignored.
    let search_start = if url.len() >= 2 && url.get(..2) == Some("//") {
        2
    } else {
        0
    };
    let early_path = find_first_of(url, "/?;", search_start);
    let mut protocol_end = find(url, "://", search_start);

    if early_path.is_some() && early_path < protocol_end {
        // If path, param or query starts before ://, :// doesn't indicate protocol.
        protocol_end = None;
    }
    if protocol_end.is_none() {
        protocol_end = Some(search_start);
    } else {
        protocol_end = protocol_end.map(|pos| pos + 3)
    }

    if let Some(path_start) = find_first_of(url, "/?;", protocol_end.unwrap()) {
        let hash_pos = find(url, "#", search_start);
        if hash_pos.is_some() && hash_pos.unwrap() < path_start {
            return "/".into();
        }

        let path_end = hash_pos.unwrap_or_else(|| url.len());
        if url.get(path_start..=path_start) != Some("/") {
            // Prepend a slash if the result would start e.g. with '?'.
            return format!("/{}", &url[path_start..path_end]);
        }
        return String::from(&url[path_start..path_end]);
    }

    "/".into()
}

/// Parses body of a robots.txt and emits parse callbacks. This will accept
/// typical typos found in robots.txt, such as 'disalow'.
///
/// Note, this function will accept all kind of input but will skip
/// everything that does not look like a robots directive.
pub fn parse_robotstxt(robots_body: &str, parse_callback: &mut impl RobotsParseHandler) {
    let mut parser = RobotsTxtParser::new(robots_body, parse_callback);
    parser.parse();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Default)]
    struct RobotsStatsReporter {
        last_line_seen: u32,
        valid_directives: u32,
        unknown_directives: u32,
        sitemap: String,
    }

    impl RobotsStatsReporter {
        fn digest(&mut self, line_num: u32) {
            assert!(line_num >= self.last_line_seen);
            self.last_line_seen = line_num;
            self.valid_directives += 1;
        }
    }

    impl RobotsParseHandler for RobotsStatsReporter {
        fn handle_robots_start(&mut self) {
            self.last_line_seen = 0;
            self.valid_directives = 0;
            self.unknown_directives = 0;
            self.sitemap.clear();
        }

        fn handle_robots_end(&mut self) {}

        fn handle_user_agent(&mut self, line_num: u32, user_agent: &str) {
            self.digest(line_num);
        }

        fn handle_allow(&mut self, line_num: u32, value: &str) {
            self.digest(line_num);
        }

        fn handle_disallow(&mut self, line_num: u32, value: &str) {
            self.digest(line_num);
        }

        fn handle_sitemap(&mut self, line_num: u32, value: &str) {
            self.digest(line_num);
            self.sitemap.push_str(value);
        }

        // Any other unrecognized name/v pairs.
        fn handle_unknown_action(&mut self, line_num: u32, action: &str, value: &str) {
            self.last_line_seen = line_num;
            self.unknown_directives += 1;
        }
    }

    #[test]
    // Different kinds of line endings are all supported: %x0D / %x0A / %x0D.0A
    fn test_lines_numbers_are_counted_correctly() {
        let mut report = RobotsStatsReporter::default();
        let unix_file = "User-Agent: foo\n\
        Allow: /some/path\n\
        User-Agent: bar\n\
        \n\
        \n\
        Disallow: /\n";
        super::parse_robotstxt(unix_file, &mut report);
        assert_eq!(4, report.valid_directives);
        assert_eq!(6, report.last_line_seen);

        let mac_file = "User-Agent: foo\r\
        Allow: /some/path\r\
        User-Agent: bar\r\
        \r\
        \r\
        Disallow: /\r";
        super::parse_robotstxt(mac_file, &mut report);
        assert_eq!(4, report.valid_directives);
        assert_eq!(6, report.last_line_seen);

        let no_final_new_line = "User-Agent: foo\n\
        Allow: /some/path\n\
        User-Agent: bar\n\
        \n\
        \n\
        Disallow: /";
        super::parse_robotstxt(no_final_new_line, &mut report);
        assert_eq!(4, report.valid_directives);
        assert_eq!(6, report.last_line_seen);

        let mixed_file = "User-Agent: foo\n\
        Allow: /some/path\r\n\
        User-Agent: bar\n\
        \r\n\
        \n\
        Disallow: /";
        super::parse_robotstxt(mixed_file, &mut report);
        assert_eq!(4, report.valid_directives);
        assert_eq!(6, report.last_line_seen);
    }

    #[test]
    fn test_get_path_params_query() {
        let f = get_path_params_query;
        assert_eq!("/", f(""));
        assert_eq!("/", f("http://www.example.com"));
        assert_eq!("/", f("http://www.example.com/"));
        assert_eq!("/a", f("http://www.example.com/a"));
        assert_eq!("/a/", f("http://www.example.com/a/"));
        assert_eq!(
            "/a/b?c=http://d.e/",
            f("http://www.example.com/a/b?c=http://d.e/")
        );
        assert_eq!(
            "/a/b?c=d&e=f",
            f("http://www.example.com/a/b?c=d&e=f#fragment")
        );
        assert_eq!("/", f("example.com"));
        assert_eq!("/", f("example.com/"));
        assert_eq!("/a", f("example.com/a"));
        assert_eq!("/a/", f("example.com/a/"));
        assert_eq!("/a/b?c=d&e=f", f("example.com/a/b?c=d&e=f#fragment"));
        assert_eq!("/", f("a"));
        assert_eq!("/", f("a/"));
        assert_eq!("/a", f("/a"));
        assert_eq!("/b", f("a/b"));
        assert_eq!("/?a", f("example.com?a"));
        assert_eq!("/a;b", f("example.com/a;b#c"));
        assert_eq!("/b/c", f("//a/b/c"));
    }
}
