//! Dependency-free structured build hook for the ph-hil-managed harness.

use std::{
    collections::BTreeSet,
    env,
    ffi::OsStr,
    fs,
    path::{Path, PathBuf},
    process::{Command, ExitCode},
};

const IMAGE_NAME: &str = "veml7700-harness.elf";
const MANIFEST_NAME: &str = "veml7700-harness.manifest.toml";

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("ph-veml7700 HIL build failed: {error}");
            ExitCode::FAILURE
        }
    }
}

fn run() -> Result<(), String> {
    let mock = parse_args()?;
    let root = repository_root()?;
    let capabilities = read_capabilities(&root)?;
    let version = checkout_version(&root);
    let output = root.join("target/ph-hil");
    fs::create_dir_all(&output).map_err(|error| format!("create {}: {error}", output.display()))?;
    let image = output.join(IMAGE_NAME);
    if mock {
        fs::write(&image, format!("deterministic mock image for veml7700-harness {version}\n"))
            .map_err(|error| format!("write {}: {error}", image.display()))?;
    } else {
        build_xtensa(&root, &image, &version)?;
    }
    write_manifest(&output.join(MANIFEST_NAME), &version, &capabilities)?;
    println!("{}", output.join(MANIFEST_NAME).display());
    Ok(())
}

fn parse_args() -> Result<bool, String> {
    let mut mock = false;
    for argument in env::args().skip(1) {
        match argument.as_str() {
            "--mock" if !mock => mock = true,
            "--mock" => return Err("--mock supplied more than once".to_owned()),
            "--help" | "-h" => {
                println!("Usage: ph-veml7700-hil-build [--mock]");
                std::process::exit(0);
            }
            other => return Err(format!("unknown argument '{other}'")),
        }
    }
    Ok(mock)
}

fn repository_root() -> Result<PathBuf, String> {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .map(Path::to_path_buf)
        .ok_or_else(|| "build tool is not under tools/ph-hil-build".to_owned())
}

fn read_capabilities(root: &Path) -> Result<Vec<String>, String> {
    let path = root.join("apps/hil-runner/capabilities.txt");
    let source = fs::read_to_string(&path)
        .map_err(|error| format!("read {}: {error}", path.display()))?;
    let values = source.lines().map(str::trim).filter(|line| !line.is_empty())
        .map(str::to_owned).collect::<Vec<_>>();
    if values.first().map(String::as_str) != Some("sys.reset@1") {
        return Err("capability inventory must start with sys.reset@1".to_owned());
    }
    let unique = values.iter().collect::<BTreeSet<_>>();
    if unique.len() != values.len() || values.iter().any(|value| !valid_capability(value)) {
        return Err("capability inventory contains a duplicate or invalid ID".to_owned());
    }
    Ok(values)
}

fn valid_capability(value: &str) -> bool {
    let Some((name, major)) = value.rsplit_once('@') else { return false };
    major.parse::<u32>().is_ok_and(|major| major > 0)
        && name.split('.').count() >= 2
        && name.split('.').all(|segment| {
            !segment.is_empty() && segment.bytes().all(|byte| {
                byte.is_ascii_lowercase() || byte.is_ascii_digit() || b"_-".contains(&byte)
            })
        })
}

fn checkout_version(root: &Path) -> String {
    let revision = command_output(root, "git", ["rev-parse", "--short=12", "HEAD"])
        .unwrap_or_else(|_| "unversioned".to_owned());
    let dirty = Command::new("git")
        .args(["diff", "--quiet", "--ignore-submodules", "HEAD", "--"])
        .current_dir(root).status().map(|status| !status.success()).unwrap_or(true);
    format!("0.1.0-dev.g{revision}{}", if dirty { "-dirty" } else { "" })
}

fn build_xtensa(root: &Path, destination: &Path, version: &str) -> Result<(), String> {
    let marker = root.join("apps/hil-runner/.REAL_FIRMWARE_IMPLEMENTED");
    if !marker.exists() {
        return Err("real HIL runner is intentionally unimplemented; complete the board/optical adapters and add apps/hil-runner/.REAL_FIRMWARE_IMPLEMENTED".to_owned());
    }
    let app = root.join("apps/hil-runner");
    let status = Command::new("cargo")
        .args(["+esp", "build", "-Zbuild-std=core", "--release", "--bin", "veml7700-hil-runner"])
        .current_dir(&app).env("PH_HIL_FIRMWARE_VERSION", version).status()
        .map_err(|error| format!("start pinned Xtensa cargo build: {error}"))?;
    if !status.success() { return Err(format!("pinned Xtensa build exited with {status}")) }
    let built = app.join("target/xtensa-esp32-none-elf/release/veml7700-hil-runner");
    fs::copy(&built, destination).map_err(|error| {
        format!("copy {} to {}: {error}", built.display(), destination.display())
    })?;
    Ok(())
}

fn write_manifest(path: &Path, version: &str, capabilities: &[String]) -> Result<(), String> {
    let mut document = format!(
        "schema = 1\n\n[firmware]\nname = \"veml7700-harness\"\nversion = \"{version}\"\nimage = \"{IMAGE_NAME}\"\ntargets = [\"esp32\"]\ntransport = {{ kind = \"serial\", framing = \"dutlink-v1\" }}\nprovides = [\n"
    );
    for capability in capabilities { document.push_str(&format!("  \"{capability}\",\n")); }
    document.push_str("]\n");
    fs::write(path, document).map_err(|error| format!("write {}: {error}", path.display()))
}

fn command_output<I, S>(root: &Path, program: &str, arguments: I) -> Result<String, String>
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    let output = Command::new(program).args(arguments).current_dir(root).output()
        .map_err(|error| format!("start {program}: {error}"))?;
    if !output.status.success() { return Err(format!("{program} exited with {}", output.status)) }
    String::from_utf8(output.stdout).map(|text| text.trim().to_owned())
        .map_err(|error| format!("{program} output was not UTF-8: {error}"))
}
