use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};

use crate::report::Reporter;

use super::GateCtx;

pub fn run(ctx: &GateCtx, reporter: &mut Reporter) -> Result<()> {
    let source_path = ctx.repo_root.join(&ctx.paths.error_source);
    let source = fs::read_to_string(&source_path)
        .with_context(|| format!("failed to read {}", source_path.display()))?;
    let variants = public_enum_variants(&source)?;
    if variants.is_empty() {
        bail!(
            "no public error variants found in {}",
            ctx.paths.error_source
        );
    }

    let mut rust_files = Vec::new();
    walk_rs(&ctx.repo_root.join(&ctx.paths.crates_dir), &mut rust_files)?;
    let contents = read_all(&rust_files)?;

    let mut unreachable = Vec::new();
    for variant in &variants {
        if !contents.iter().any(|text| text.contains(variant)) {
            unreachable.push(variant.as_str());
        }
    }
    if !unreachable.is_empty() {
        bail!(
            "public error variants that no code names: {}",
            unreachable.join(" ")
        );
    }
    reporter.note(&format!(
        "{} public error variants, all reachable",
        variants.len()
    ));
    Ok(())
}

pub fn public_enum_variants(source: &str) -> Result<Vec<String>> {
    let file = syn::parse_file(source).context("failed to parse error.rs")?;
    let mut out = Vec::new();
    for item in file.items {
        let syn::Item::Enum(en) = item else {
            continue;
        };
        if !matches!(en.vis, syn::Visibility::Public(_)) {
            continue;
        }
        let name = en.ident.to_string();
        for variant in en.variants {
            out.push(format!("{name}::{}", variant.ident));
        }
    }
    Ok(out)
}

fn walk_rs(dir: &Path, files: &mut Vec<PathBuf>) -> Result<()> {
    for entry in fs::read_dir(dir).with_context(|| format!("failed to read {}", dir.display()))? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            walk_rs(&path, files)?;
        } else if path.extension().is_some_and(|ext| ext == "rs") {
            files.push(path);
        }
    }
    Ok(())
}

fn read_all(files: &[PathBuf]) -> Result<Vec<String>> {
    let mut out = Vec::with_capacity(files.len());
    for path in files {
        out.push(
            fs::read_to_string(path)
                .with_context(|| format!("failed to read {}", path.display()))?,
        );
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::public_enum_variants;

    #[test]
    fn names_public_variants_including_generics() {
        let src = r#"
            pub enum Operation { Inspect, Snapshot }
            pub enum Error<E> { Bus { source: E }, Configuration }
            enum Private { Hidden }
        "#;
        let names = public_enum_variants(src).expect("parse");
        assert_eq!(
            names,
            vec![
                "Operation::Inspect",
                "Operation::Snapshot",
                "Error::Bus",
                "Error::Configuration",
            ]
        );
    }
}
