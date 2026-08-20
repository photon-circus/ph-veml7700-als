use std::fs;
use std::path::Path;
use std::sync::LazyLock;

use anyhow::{Context, Result, bail};
use regex::Regex;

use crate::git;
use crate::report::Reporter;

use super::GateCtx;

pub fn run(ctx: &GateCtx, reporter: &mut Reporter) -> Result<()> {
    let files = git::ls_files(&ctx.repo_root, &["*.md"])?;
    let mut ok = 0;
    let mut failures = Vec::new();
    for rel in files {
        let path = ctx.repo_root.join(&rel);
        if !path.is_file() {
            continue;
        }
        let text = fs::read_to_string(&path).with_context(|| format!("failed to read {rel}"))?;
        for target in markdown_targets(&text) {
            match check_target(&ctx.repo_root, &rel, &target) {
                Check::Skip => {}
                Check::Ok => ok += 1,
                Check::Broken => failures.push(format!("BROKEN  {rel} -> {target}")),
                Check::Anchor => failures.push(format!("ANCHOR  {rel} -> {target}")),
            }
        }
    }
    if !failures.is_empty() {
        bail!(
            "local Markdown links that do not resolve:\n{}",
            failures.join("\n")
        );
    }
    reporter.note(&format!("{ok} local links resolve"));
    Ok(())
}

enum Check {
    Skip,
    Ok,
    Broken,
    Anchor,
}

fn check_target(repo_root: &Path, markdown_file: &str, target: &str) -> Check {
    if is_external(target) || target.is_empty() {
        return Check::Skip;
    }
    let (target_path, target_anchor) = split_anchor(target);
    let resolved = resolve_path(markdown_file, target_path);
    let resolved = collapse_dotdot(&resolved);
    let full = repo_root.join(&resolved);
    if !full.exists() {
        return Check::Broken;
    }
    if target_anchor.is_empty() || !resolved.ends_with(".md") {
        return Check::Ok;
    }
    let Ok(text) = fs::read_to_string(&full) else {
        return Check::Broken;
    };
    if heading_anchors(&text)
        .iter()
        .any(|slug| slug == target_anchor)
    {
        Check::Ok
    } else {
        Check::Anchor
    }
}

pub fn is_external(target: &str) -> bool {
    target.starts_with("http://") || target.starts_with("https://") || target.starts_with("mailto:")
}

pub fn split_anchor(target: &str) -> (&str, &str) {
    match target.split_once('#') {
        Some((path, anchor)) => (path, anchor),
        None => (target, ""),
    }
}

pub fn resolve_path(markdown_file: &str, target_path: &str) -> String {
    if target_path.is_empty() {
        return markdown_file.replace('\\', "/");
    }
    let normalized = markdown_file.replace('\\', "/");
    let dir = match normalized.rsplit_once('/') {
        Some((parent, _)) => parent,
        None => return target_path.to_string(),
    };
    if dir.is_empty() || dir == "." {
        target_path.to_string()
    } else {
        format!("{dir}/{target_path}")
    }
}

pub fn collapse_dotdot(path: &str) -> String {
    let mut parts: Vec<&str> = Vec::new();
    for component in path.split('/') {
        if component.is_empty() || component == "." {
            continue;
        }
        if component == ".." {
            if !parts.is_empty() {
                parts.pop();
            }
            continue;
        }
        parts.push(component);
    }
    parts.join("/")
}

pub fn markdown_targets(content: &str) -> Vec<String> {
    static RE: LazyLock<Regex> =
        LazyLock::new(|| Regex::new(r#"\]\(<?[^)>"\s]+>?\)"#).expect("target regex"));
    RE.find_iter(content)
        .map(|m| normalize_target(m.as_str()))
        .collect()
}

fn normalize_target(raw: &str) -> String {
    let mut s = raw.strip_prefix(']').unwrap_or(raw);
    s = s.strip_prefix('(').unwrap_or(s);
    s = s.strip_suffix(')').unwrap_or(s);
    s = s.strip_prefix('<').unwrap_or(s);
    s = s.strip_suffix('>').unwrap_or(s);
    s.to_string()
}

pub fn heading_anchors(markdown: &str) -> Vec<String> {
    let mut seen = std::collections::HashMap::<String, u32>::new();
    let mut out = Vec::new();
    for line in markdown.lines() {
        let Some(heading) = heading_text(line) else {
            continue;
        };
        let slug = slugify(heading);
        let count = seen.entry(slug.clone()).or_insert(0);
        *count += 1;
        if *count == 1 {
            out.push(slug);
        } else {
            out.push(format!("{}-{}", slug, *count - 1));
        }
    }
    out
}

fn heading_text(line: &str) -> Option<&str> {
    let bytes = line.as_bytes();
    let mut n = 0;
    while n < bytes.len() && bytes[n] == b'#' {
        n += 1;
    }
    if n == 0 || n > 6 {
        return None;
    }
    let rest = &line[n..];
    let trimmed = rest.trim_start();
    if trimmed.len() == rest.len() {
        return None;
    }
    Some(trimmed)
}

pub fn slugify(heading: &str) -> String {
    let mut s = heading.replace('`', "");
    s = s.replace("**", "");
    s = s.replace('*', "");
    s = strip_markdown_links(&s);
    s = s.to_ascii_lowercase();
    s = s
        .chars()
        .filter(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || *c == ' ' || *c == '-')
        .collect();
    collapse_whitespace_to_hyphens(&s)
}

fn strip_markdown_links(s: &str) -> String {
    static RE: LazyLock<Regex> =
        LazyLock::new(|| Regex::new(r"\[([^]]*)\]\([^)]*\)").expect("link regex"));
    RE.replace_all(s, "$1").into_owned()
}

fn collapse_whitespace_to_hyphens(s: &str) -> String {
    let mut out = String::new();
    let mut in_space = false;
    for c in s.chars() {
        if c == ' ' {
            if !in_space {
                out.push('-');
                in_space = true;
            }
        } else {
            in_space = false;
            out.push(c);
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::{
        collapse_dotdot, heading_anchors, markdown_targets, resolve_path, slugify, split_anchor,
    };

    #[test]
    fn slugs_match_the_github_algorithm() {
        assert_eq!(
            slugify("Claim registry identifiers"),
            "claim-registry-identifiers"
        );
        assert_eq!(slugify("D-017 — One script"), "d-017-one-script");
        assert_eq!(slugify("`S-nn` IDs"), "s-nn-ids");
        assert_eq!(
            slugify("See [Verification](docs/VERIFICATION.md)"),
            "see-verification"
        );
    }

    #[test]
    fn repeated_headings_get_numeric_suffixes() {
        let md = "# Same\n# Same\n# Same\n";
        assert_eq!(heading_anchors(md), vec!["same", "same-1", "same-2"]);
    }

    #[test]
    fn extracts_paren_and_angle_targets() {
        let md = "[a](docs/a.md) [b](<docs/b.md#anchor>)";
        assert_eq!(markdown_targets(md), vec!["docs/a.md", "docs/b.md#anchor"]);
    }

    #[test]
    fn splits_path_and_anchor() {
        assert_eq!(split_anchor("docs/a.md#s-1"), ("docs/a.md", "s-1"));
        assert_eq!(split_anchor("#local"), ("", "local"));
        assert_eq!(split_anchor("docs/a.md"), ("docs/a.md", ""));
    }

    #[test]
    fn resolves_relative_to_the_source_file() {
        assert_eq!(resolve_path("README.md", "docs/a.md"), "docs/a.md");
        assert_eq!(
            resolve_path("docs/vendor/README.md", "../HARDWARE_CONTRACT.md"),
            "docs/vendor/../HARDWARE_CONTRACT.md"
        );
        assert_eq!(resolve_path("docs/a.md", ""), "docs/a.md");
    }

    #[test]
    fn collapses_parent_segments() {
        assert_eq!(
            collapse_dotdot("docs/vendor/../HARDWARE_CONTRACT.md"),
            "docs/HARDWARE_CONTRACT.md"
        );
        assert_eq!(collapse_dotdot("./README.md"), "README.md");
    }
}
