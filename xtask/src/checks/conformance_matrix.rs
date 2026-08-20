use std::fs;

use anyhow::{Context, Result, bail};
use regex::Regex;
use std::sync::LazyLock;

use crate::report::Reporter;

use super::GateCtx;

pub fn run(ctx: &GateCtx, reporter: &mut Reporter) -> Result<()> {
    let source_path = ctx.repo_root.join(&ctx.paths.conformance_tests);
    let source = fs::read_to_string(&source_path)
        .with_context(|| format!("failed to read {}", source_path.display()))?;
    let actual = test_fn_names(&source)?;
    if actual.is_empty() {
        bail!(
            "no conformance tests found in {}",
            ctx.paths.conformance_tests
        );
    }

    let matrix_path = ctx.repo_root.join(&ctx.paths.verification);
    let matrix = fs::read_to_string(&matrix_path)
        .with_context(|| format!("failed to read {}", matrix_path.display()))?;
    let Some(block) = section_between(&matrix, &ctx.paths.coverage_start, &ctx.paths.coverage_end)
    else {
        bail!("no coverage matrix found in {}", ctx.paths.verification);
    };

    let missing_from_matrix: Vec<_> = actual
        .iter()
        .filter(|name| !block.contains(name.as_str()))
        .cloned()
        .collect();
    if !missing_from_matrix.is_empty() {
        bail!(
            "conformance tests absent from the maintained coverage matrix: {}\ncoverage that the maintained inventory does not disclose is the failure mode this check exists for.",
            missing_from_matrix.join(" ")
        );
    }

    let named = covered_test_names(block, &ctx.paths.covered_start, &ctx.paths.covered_end);
    let actual_set: Vec<_> = actual.iter().map(String::as_str).collect();
    let missing_from_tests: Vec<_> = named
        .iter()
        .filter(|name| !actual_set.contains(&name.as_str()))
        .cloned()
        .collect();
    if !missing_from_tests.is_empty() {
        bail!(
            "coverage matrix names traces that are not executable tests: {}\nthey may be missing entirely, or present without #[test].",
            missing_from_tests.join(" ")
        );
    }

    reporter.note(&format!(
        "{} conformance tests, all disclosed",
        actual.len()
    ));
    Ok(())
}

pub fn test_fn_names(source: &str) -> Result<Vec<String>> {
    let file = syn::parse_file(source).context("failed to parse conformance tests")?;
    let mut names = Vec::new();
    collect_tests(&file.items, &mut names);
    names.sort();
    names.dedup();
    Ok(names)
}

fn collect_tests(items: &[syn::Item], names: &mut Vec<String>) {
    for item in items {
        match item {
            syn::Item::Fn(func) if is_test_fn(func) => {
                names.push(func.sig.ident.to_string());
            }
            syn::Item::Mod(module) => {
                if let Some((_, items)) = &module.content {
                    collect_tests(items, names);
                }
            }
            _ => {}
        }
    }
}

fn is_test_fn(func: &syn::ItemFn) -> bool {
    func.attrs.iter().any(|attr| attr.path().is_ident("test"))
}

pub fn section_between<'a>(text: &'a str, start: &str, end: &str) -> Option<&'a str> {
    let start_idx = text.find(start)?;
    let from_start = &text[start_idx..];
    let end_rel = from_start[start.len()..].find(end)?;
    Some(&from_start[..start.len() + end_rel])
}

pub fn covered_test_names(matrix_block: &str, start: &str, end: &str) -> Vec<String> {
    static TICK: LazyLock<Regex> =
        LazyLock::new(|| Regex::new(r"`([a-z0-9_]+)`").expect("tick regex"));
    let Some(covered) = section_between(matrix_block, start, end) else {
        return Vec::new();
    };
    let mut names = Vec::new();
    for line in covered.lines() {
        let fields: Vec<_> = line.split('|').collect();
        if fields.len() < 5 {
            continue;
        }
        let test_col = fields[fields.len() - 2];
        for cap in TICK.captures_iter(test_col) {
            names.push(cap[1].to_string());
        }
    }
    names.sort();
    names.dedup();
    names
}

#[cfg(test)]
mod tests {
    use super::{covered_test_names, section_between, test_fn_names};

    #[test]
    fn finds_top_level_test_functions() {
        let src = r#"
            #[test]
            fn alpha() {}

            fn not_a_test() {}

            #[test]
            fn beta() {}
        "#;
        let names = test_fn_names(src).expect("parse");
        assert_eq!(names, vec!["alpha", "beta"]);
    }

    #[test]
    fn ignores_functions_without_test() {
        let src = "fn looks_like_a_trace() {}";
        assert!(test_fn_names(src).expect("parse").is_empty());
    }

    #[test]
    fn extracts_backticked_names_from_the_last_table_column() {
        let block = "\
### Covered
| Public operation | State | Config | Conformance test |
| --- | --- | --- | --- |
| `measure_once` | reset | — | `trace_one`, `trace_two` |
| `probe` | reset | — | `trace_one` |
### Untraced public operations
";
        let names = covered_test_names(block, "### Covered", "### Untraced public operations");
        assert_eq!(names, vec!["trace_one", "trace_two"]);
    }

    #[test]
    fn section_stops_before_the_end_heading() {
        let text = "## Start\nkeep\n## End\ndrop\n";
        assert_eq!(
            section_between(text, "## Start", "## End"),
            Some("## Start\nkeep\n")
        );
    }
}
