use std::collections::BTreeSet;
use std::fs;

use anyhow::{Context, Result, bail};
use regex::Regex;
use std::sync::LazyLock;

use crate::git;
use crate::report::Reporter;

use super::GateCtx;

pub fn run(ctx: &GateCtx, reporter: &mut Reporter) -> Result<()> {
    let registry_path = ctx.repo_root.join(&ctx.paths.claim_registry);
    let registry = fs::read_to_string(&registry_path)
        .with_context(|| format!("failed to read {}", registry_path.display()))?;
    let ids = claim_ids(&registry);
    if ids.is_empty() {
        bail!(
            "no S-nn claim identifiers found in {}",
            ctx.paths.claim_registry
        );
    }
    let duplicates = duplicate_ids(&ids);
    if !duplicates.is_empty() {
        bail!(
            "claim identifiers must be unique, but these repeat:\n{}\nan identifier names one row for the life of the document.",
            duplicates.join("\n")
        );
    }

    let defined: BTreeSet<_> = ids.iter().cloned().collect();
    let sources = git::ls_files(&ctx.repo_root, &["*.md", "*.rs"])?;
    let mut cited = BTreeSet::new();
    for rel in sources {
        let path = ctx.repo_root.join(&rel);
        if !path.is_file() {
            continue;
        }
        let text = fs::read_to_string(&path).with_context(|| format!("failed to read {rel}"))?;
        cited.extend(citations(&text));
    }

    let dangling: Vec<_> = cited.difference(&defined).cloned().collect();
    if !dangling.is_empty() {
        bail!(
            "citations name claim identifiers that no contract row defines: {}\nretired identifiers remain as tombstones, so this is mistyped or stale.",
            dangling.join(" ")
        );
    }

    reporter.note(&format!(
        "{} stable claim identifiers; every citation resolves",
        defined.len()
    ));
    Ok(())
}

pub fn claim_ids(registry: &str) -> Vec<String> {
    static HEADING: LazyLock<Regex> =
        LazyLock::new(|| Regex::new(r"^### (S-[0-9][0-9]*)$").expect("claim heading"));
    let mut ids = Vec::new();
    for line in registry.lines() {
        if let Some(cap) = HEADING.captures(line) {
            ids.push(cap[1].to_string());
        }
    }
    ids.sort();
    ids
}

pub fn citations(text: &str) -> Vec<String> {
    static CITE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"`(S-[0-9]+)`").expect("citation"));
    let mut ids: Vec<_> = CITE
        .captures_iter(text)
        .map(|cap| cap[1].to_string())
        .collect();
    ids.sort();
    ids.dedup();
    ids
}

fn duplicate_ids(sorted: &[String]) -> Vec<String> {
    let mut dups = Vec::new();
    for pair in sorted.windows(2) {
        if pair[0] == pair[1] && dups.last() != Some(&pair[0]) {
            dups.push(pair[0].clone());
        }
    }
    dups
}

// This file is itself a tracked `*.rs` source the check above scans, so any
// backticked identifier in these fixtures is a real citation. Use identifiers
// the contract registry defines, or the gate fails on its own test data.
#[cfg(test)]
mod tests {
    use super::{citations, claim_ids, duplicate_ids};

    #[test]
    fn reads_stable_headings() {
        let text = "### S-1\ntext\n### S-10\n### S-2\n";
        assert_eq!(claim_ids(text), vec!["S-1", "S-10", "S-2"]);
    }

    #[test]
    fn ignores_inline_mentions_as_definitions() {
        let text = "See `S-01` in prose.\n### Not-a-claim\n";
        assert!(claim_ids(text).is_empty());
    }

    #[test]
    fn extracts_backticked_citations() {
        assert_eq!(citations("cites `S-01` and `S-22`."), vec!["S-01", "S-22"]);
    }

    #[test]
    fn finds_repeated_sorted_ids() {
        let ids = vec![
            "S-1".into(),
            "S-1".into(),
            "S-2".into(),
            "S-2".into(),
            "S-2".into(),
        ];
        assert_eq!(duplicate_ids(&ids), vec!["S-1", "S-2"]);
    }
}
