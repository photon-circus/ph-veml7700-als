#!/usr/bin/env python3
"""Validate the VEML7700 development pack and external contracts."""

from __future__ import annotations

import hashlib
import json
import re
import subprocess
import sys
import tomllib
from pathlib import Path

from pack_policy import automation_contains_publish_command, publication_is_hard_disabled

ROOT = Path(__file__).resolve().parents[1]

PACK_EXCLUDED_DIRS = {".git", "__pycache__", "target"}
PACK_EXCLUDED_FILES = {"Cargo.lock", "PACK_SHA256SUMS.txt"}

REQUIRED = [
    "AGENTS.md",
    "PACK_MANIFEST.md",
    "docs/HARDWARE_CONTRACT.md",
    "docs/INVARIANTS.md",
    "docs/ARCHITECTURE.md",
    "docs/API_CONTRACT.md",
    "docs/IMPLEMENTATION_PLAN.md",
    "docs/TEST_PLAN.md",
    "docs/HIL_EVIDENCE_PROTOCOL.md",
    "docs/HIL_TEST_CIRCUIT.md",
    "crates/veml7700/Cargo.toml",
    "crates/veml7700/src/lib.rs",
    "crates/veml7700/src/driver.rs",
    "crates/veml7700/src/config.rs",
    "crates/veml7700/src/illuminance.rs",
    "apps/hil-runner/capabilities.txt",
    "hil/contracts/veml7700.toml",
    "hil/plan.toml",
    "hil/plan-optical.template.toml",
    "hil/bench.mock.toml",
    "hil/firmware.mock.toml",
    "hil/mock/transcript.json",
    "hil/policy.lua",
    "hil/policy-optical.lua",
    "PACK_SHA256SUMS.txt",
]


def fail(message: str) -> None:
    raise AssertionError(message)


def load_toml(relative: str) -> dict:
    with (ROOT / relative).open("rb") as handle:
        return tomllib.load(handle)


def validate_required_files() -> None:
    missing = [path for path in REQUIRED if not (ROOT / path).is_file()]
    if missing:
        fail("missing required files: " + ", ".join(missing))


def validate_vendor_pdf_boundary() -> None:
    vendor_pdfs = sorted((ROOT / "docs/vendor").glob("*.pdf"))
    if (ROOT / ".git").exists():
        result = subprocess.run(
            ["git", "ls-files", "--", "docs/vendor/*.pdf"],
            cwd=ROOT,
            check=True,
            capture_output=True,
            text=True,
        )
        tracked = [line for line in result.stdout.splitlines() if line]
        if tracked:
            fail("vendor PDFs must not be tracked: " + ", ".join(tracked))
    elif vendor_pdfs:
        fail("vendor PDFs must not be included in an extracted development pack")

    gitignore = (ROOT / ".gitignore").read_text(encoding="utf-8").splitlines()
    if "docs/vendor/*.pdf" not in gitignore:
        fail(".gitignore must protect local vendor PDFs")


def validate_parseable_files() -> None:
    for path in ROOT.rglob("*.toml"):
        try:
            with path.open("rb") as handle:
                tomllib.load(handle)
        except Exception as error:
            fail(f"invalid TOML {path.relative_to(ROOT)}: {error}")
    for path in ROOT.rglob("*.json"):
        try:
            json.loads(path.read_text(encoding="utf-8"))
        except Exception as error:
            fail(f"invalid JSON {path.relative_to(ROOT)}: {error}")
    try:
        import yaml  # type: ignore
    except ImportError:
        yaml = None
    if yaml is not None:
        for path in ROOT.rglob("*.yml"):
            try:
                yaml.safe_load(path.read_text(encoding="utf-8"))
            except Exception as error:
                fail(f"invalid YAML {path.relative_to(ROOT)}: {error}")


def capability_inventory() -> list[str]:
    values = [
        line.strip()
        for line in (ROOT / "apps/hil-runner/capabilities.txt")
        .read_text(encoding="utf-8")
        .splitlines()
        if line.strip()
    ]
    if not values or values[0] != "sys.reset@1":
        fail("capability inventory must start with sys.reset@1")
    if len(values) != 12 or len(values) != len(set(values)):
        fail("expected 12 unique capabilities")
    for value in values:
        name, separator, major = value.rpartition("@")
        if separator != "@" or not major.isdigit() or int(major) < 1:
            fail(f"invalid capability ID: {value}")
        if any(
            not segment
            or any(not (char.islower() or char.isdigit() or char in "_-") for char in segment)
            for segment in name.split(".")
        ):
            fail(f"invalid capability name: {value}")
    return values


def validate_capability_contracts(values: list[str]) -> None:
    contract = load_toml("hil/contracts/veml7700.toml")["capability"]
    firmware = load_toml("hil/firmware.mock.toml")["firmware"]["provides"]
    transcript = json.loads((ROOT / "hil/mock/transcript.json").read_text(encoding="utf-8"))
    if list(contract.keys()) != values:
        fail("capability contract order/content differs from capabilities.txt")
    if firmware != values:
        fail("mock firmware manifest differs from capabilities.txt")
    if transcript["capabilities"] != values:
        fail("mock transcript differs from capabilities.txt")


def validate_plan_and_transcript(values: list[str]) -> None:
    plan = load_toml("hil/plan.toml")
    required = plan["plan"]["requires"]["dut_caps"]
    if any(value not in values for value in required):
        fail("plan requires an unknown capability")
    if "veml7700.optical@1" in required:
        fail("core plan must not require calibrated optical capability")
    metadata = plan["plan"]["requires"]["metadata"]
    if metadata.get("veml7700.circuit_checks_passed") is not True:
        fail("core plan lacks circuit gate")
    if metadata.get("veml7700.light_source_repeatable") is not True:
        fail("core plan lacks repeatable-source gate")

    transcript = json.loads((ROOT / "hil/mock/transcript.json").read_text(encoding="utf-8"))
    commands = {item["command"]: item["reply"] for item in transcript["commands"]}
    blocks = plan["block"]
    block_names = [block["name"] for block in blocks]
    if plan["phase"][0]["blocks"] != block_names:
        fail("phase block order differs from block declarations")
    for block in blocks:
        for module in block["modules"]:
            params = module["params"]
            command = params["command"]
            if command not in commands:
                fail(f"mock transcript lacks command: {command}")
            reply = commands[command]
            if reply["kind"] != "ok" or not reply["payload"].startswith("json:"):
                fail(f"malformed mock reply for {command}")
            evidence = json.loads(reply["payload"][5:])
            if evidence["schema"] != 1 or evidence["capability"] != params["capability"]:
                fail(f"schema/capability mismatch for {command}")
            observed = [case["case"] for case in evidence["cases"]]
            if observed != params["expected_cases"]:
                fail(f"case inventory mismatch for {command}")

    optical = load_toml("hil/plan-optical.template.toml")
    optical_meta = optical["plan"]["requires"]["metadata"]
    for gate in [
        "veml7700.circuit_checks_passed",
        "veml7700.light_source_repeatable",
        "veml7700.optical_fixture_characterized",
        "veml7700.reference_calibrated",
    ]:
        if optical_meta.get(gate) is not True:
            fail(f"optical plan lacks gate: {gate}")


def validate_workspace_and_runtime() -> None:
    workspace = load_toml("Cargo.toml")
    if workspace["workspace"]["members"] != ["crates/veml7700"]:
        fail("unexpected workspace members")
    cargo = load_toml("crates/veml7700/Cargo.toml")
    if cargo["package"]["name"] != "ph-veml7700-als":
        fail("unexpected package name")
    for manifest_path in ROOT.rglob("Cargo.toml"):
        if "target" in manifest_path.relative_to(ROOT).parts:
            continue
        with manifest_path.open("rb") as handle:
            manifest = tomllib.load(handle)
        if "package" in manifest and not publication_is_hard_disabled(manifest):
            fail(
                "publication must remain hard-disabled with publish = false: "
                + manifest_path.relative_to(ROOT).as_posix()
            )
    dependencies = set(cargo.get("dependencies", {}))
    if dependencies != {"embedded-hal-async", "defmt"}:
        fail(f"unexpected runtime dependencies: {sorted(dependencies)}")

    automation_paths = [ROOT / "tools/check.sh", ROOT / "tools/check.ps1"]
    automation_paths.extend((ROOT / ".github/workflows").glob("*.yml"))
    automation_paths.extend((ROOT / ".github/workflows").glob("*.yaml"))
    for path in automation_paths:
        if automation_contains_publish_command(path.read_text(encoding="utf-8")):
            fail(
                "publication command or credential is forbidden in automation: "
                + path.relative_to(ROOT).as_posix()
            )

    lib = (ROOT / "crates/veml7700/src/lib.rs").read_text(encoding="utf-8")
    for required in ["#![no_std]", "#![forbid(unsafe_code)]", "#![deny(missing_docs)]"]:
        if required not in lib:
            fail(f"lib.rs missing {required}")
    for path in (ROOT / "crates/veml7700/src").glob("*.rs"):
        source = path.read_text(encoding="utf-8")
        runtime = source.split("#[cfg(test)]", 1)[0]
        if "std::" in runtime or "alloc::" in runtime:
            fail(f"runtime source uses std/alloc: {path.name}")
        if "unsafe {" in runtime or "unsafe fn" in runtime:
            fail(f"runtime source contains unsafe: {path.name}")
        if "from_be_bytes" in runtime or "to_be_bytes" in runtime:
            fail(f"runtime source uses wrong register byte order: {path.name}")


def validate_architecture() -> None:
    driver = (ROOT / "crates/veml7700/src/driver.rs").read_text(encoding="utf-8")
    match = re.search(r"pub struct Veml7700<I2C>\s*\{(?P<body>.*?)\n\}", driver, re.S)
    if match is None:
        fail("Veml7700<I2C> facade not found")
    fields = [
        line.strip().rstrip(",")
        for line in match.group("body").splitlines()
        if line.strip() and not line.strip().startswith("///")
    ]
    if fields != ["i2c: I2C"]:
        fail(f"driver caches unexpected state: {fields}")
    if "pub async fn init" in driver or "pub fn init" in driver:
        fail("driver exposes rejected init()")
    if "from_le_bytes" not in driver or "to_le_bytes" not in driver:
        fail("driver lacks explicit low-byte-first word conversion")
    all_runtime = "\n".join(
        path.read_text(encoding="utf-8")
        for path in (ROOT / "crates/veml7700/src").glob("*.rs")
    )
    for rejected in ["InputPin", "ALERT/RDY", "pub mod register", "pub use register"]:
        if rejected in all_runtime:
            fail(f"rejected API/term leaked into runtime source: {rejected}")
    if "I2C_ADDRESS: u8 = 0x10" not in driver:
        fail("fixed address contract missing")
    if "ThresholdMonitorOwnsDomain" not in driver:
        fail("monitor-domain protection missing")
    if driver.find("WriteLowThreshold") > driver.find("EnableMonitor"):
        fail("threshold enable-last sequence appears reversed")



def validate_measurement_and_monitor_semantics() -> None:
    driver = (ROOT / "crates/veml7700/src/driver.rs").read_text(encoding="utf-8")
    timing = (ROOT / "crates/veml7700/src/timing.rs").read_text(encoding="utf-8")
    threshold = (ROOT / "crates/veml7700/src/threshold.rs").read_text(encoding="utf-8")
    illuminance = (ROOT / "crates/veml7700/src/illuminance.rs").read_text(encoding="utf-8")

    if "pub struct MeasurementTiming" not in timing:
        fail("MeasurementTiming type missing")
    timing_body = re.search(
        r"pub struct MeasurementTiming\s*\{(?P<body>.*?)\n\}", timing, re.S
    )
    if timing_body is None or "pub " in timing_body.group("body"):
        fail("MeasurementTiming fields must remain private")
    for required in [
        "with_additional_margin_us",
        "INTEGRATION_TOLERANCE_PERCENT",
        "saturating_add",
    ]:
        if required not in timing:
            fail(f"conservative timing guard missing: {required}")

    start = driver.find("pub async fn measure_once_with_timing")
    end = driver.find("pub async fn arm_threshold_monitor", start)
    if start < 0 or end < 0:
        fail("fresh measurement implementation boundary not found")
    fresh = driver[start:end]
    required_order = [
        "TimingIntegrationMismatch",
        "ObserveConfiguration",
        "PowerSavingConfig::new(false",
        "with_power_state(PowerState::Shutdown)",
        "with_power_state(PowerState::Active)",
        "delay.delay_us(timing.total_us())",
        "MeasureStage::FreezeResult",
        "read_als_for",
        "read_white_for",
        "restore_state",
    ]
    positions = [fresh.find(marker) for marker in required_order]
    if any(position < 0 for position in positions):
        missing = [
            marker for marker, position in zip(required_order, positions) if position < 0
        ]
        fail("fresh measurement guard missing: " + ", ".join(missing))
    if positions != sorted(positions):
        fail("fresh measurement sequence does not follow the contracted order")

    if "let reserved = word & 0x3FFF" not in threshold:
        fail("threshold status does not reject reserved bits")
    if "all_documented_gain_and_integration_pairs_match_the_nominal_table" not in illuminance:
        fail("full nominal illuminance table test missing")
    if "ThresholdMonitorOwnsDomain" not in driver:
        fail("threshold monitor domain is not protected")

def validate_public_api() -> None:
    contract = (ROOT / "docs/API_CONTRACT.md").read_text(encoding="utf-8")
    public_functions: set[str] = set()
    for path in (ROOT / "crates/veml7700/src").glob("*.rs"):
        source = path.read_text(encoding="utf-8")
        public_functions.update(
            match.group(1)
            for match in re.finditer(
                r"\bpub(?:\s+async)?\s+(?:const\s+)?fn\s+([A-Za-z_][A-Za-z0-9_]*)",
                source,
            )
        )
    missing = sorted(name for name in public_functions if re.search(rf"\b{re.escape(name)}\b", contract) is None)
    if missing:
        fail("public functions absent from API contract: " + ", ".join(missing))
    for constant in [
        "I2C_ADDRESS",
        "WAKE_UP_DELAY_US",
        "INTEGRATION_TOLERANCE_PERCENT",
        "MEASUREMENT_MARGIN_US",
    ]:
        if constant not in contract:
            fail(f"public constant absent from API contract: {constant}")


def validate_hil_boundary() -> None:
    crate_cargo = (ROOT / "crates/veml7700/Cargo.toml").read_text(encoding="utf-8").lower()
    if "ph-hil" in crate_cargo:
        fail("publishable crate depends on ph-hil")
    policy = (ROOT / "hil/policy.lua").read_text(encoding="utf-8")
    plan = load_toml("hil/plan.toml")
    for block in plan["block"]:
        for module in block["modules"]:
            artifact = module["params"].get("artifact")
            if artifact and f'ctx.decode_i2c("{block["name"]}", "{artifact}")' not in policy:
                fail(f"captured artifact is not decoded: {block['name']}/{artifact}")
    if "mock evidence cannot support a hardware capability" not in policy:
        fail("mock evidence is not forced void")


def validate_no_stale_device_content() -> None:
    stale = re.compile(r"ADS1115|ads1115|ALERT/RDY|full-scale range|single-shot", re.I)
    for path in ROOT.rglob("*"):
        if (
            not path.is_file()
            or path.name == "PACK_SHA256SUMS.txt"
            or path.suffix == ".bin"
            or path == ROOT / "tools/validate-pack.py"
        ):
            continue
        try:
            text = path.read_text(encoding="utf-8")
        except UnicodeDecodeError:
            continue
        match = stale.search(text)
        if match:
            fail(f"stale ADS-oriented content in {path.relative_to(ROOT)}: {match.group(0)}")


def validate_relative_markdown_links() -> None:
    pattern = re.compile(r"\[[^\]]*\]\(([^)]+)\)")
    for path in ROOT.rglob("*.md"):
        text = path.read_text(encoding="utf-8")
        for target in pattern.findall(text):
            target = target.split("#", 1)[0]
            if not target or target.startswith(("http://", "https://", "mailto:")):
                continue
            resolved = (path.parent / target).resolve()
            try:
                resolved.relative_to(ROOT.resolve())
            except ValueError:
                fail(f"markdown link escapes pack: {path.relative_to(ROOT)} -> {target}")
            if not resolved.exists():
                fail(f"broken markdown link: {path.relative_to(ROOT)} -> {target}")


def validate_mock_image() -> None:
    firmware = load_toml("hil/firmware.mock.toml")["firmware"]
    image = ROOT / "hil" / firmware["image"]
    if not image.is_file():
        fail(f"mock firmware image does not exist: {image.relative_to(ROOT)}")


def is_pack_file(path: Path) -> bool:
    relative = path.relative_to(ROOT)
    if not path.is_file():
        return False
    if any(part in PACK_EXCLUDED_DIRS for part in relative.parts):
        return False
    if path.name in PACK_EXCLUDED_FILES:
        return False
    if relative.parent.as_posix() == "docs/vendor" and relative.suffix.lower() == ".pdf":
        return False
    return True


def validate_checksums() -> None:
    entries: dict[str, str] = {}
    for line in (ROOT / "PACK_SHA256SUMS.txt").read_text(encoding="utf-8").splitlines():
        if not line.strip():
            continue
        digest, relative = line.split("  ", 1)
        entries[relative] = digest
    expected_paths = sorted(
        path.relative_to(ROOT).as_posix()
        for path in ROOT.rglob("*")
        if is_pack_file(path)
    )
    if sorted(entries) != expected_paths:
        fail("checksum inventory differs from pack file inventory")
    for relative, expected in entries.items():
        observed = hashlib.sha256((ROOT / relative).read_bytes()).hexdigest()
        if observed != expected:
            fail(f"checksum mismatch: {relative}")


def write_checksums() -> None:
    lines = []
    for path in sorted((path for path in ROOT.rglob("*") if is_pack_file(path))):
        relative = path.relative_to(ROOT).as_posix()
        digest = hashlib.sha256(path.read_bytes()).hexdigest()
        lines.append(f"{digest}  {relative}\n")
    (ROOT / "PACK_SHA256SUMS.txt").write_text("".join(lines), encoding="utf-8")


def main() -> int:
    validate_required_files()
    validate_vendor_pdf_boundary()
    validate_parseable_files()
    validate_workspace_and_runtime()
    validate_architecture()
    validate_measurement_and_monitor_semantics()
    validate_public_api()
    validate_hil_boundary()
    validate_no_stale_device_content()
    validate_relative_markdown_links()
    validate_mock_image()
    values = capability_inventory()
    validate_capability_contracts(values)
    validate_plan_and_transcript(values)
    validate_checksums()
    print(f"development pack valid: {len(values)} capabilities, schema-1 contracts aligned")
    return 0


if __name__ == "__main__":
    try:
        if sys.argv[1:] == ["--write-checksums"]:
            write_checksums()
        elif sys.argv[1:]:
            fail("usage: validate-pack.py [--write-checksums]")
        raise SystemExit(main())
    except AssertionError as error:
        print(f"validation failed: {error}", file=sys.stderr)
        raise SystemExit(1)
