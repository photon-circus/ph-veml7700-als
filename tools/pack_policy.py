"""Pure policy predicates shared by pack validation and its regression tests."""


def publication_is_hard_disabled(cargo: dict) -> bool:
    """Return whether a Cargo document explicitly disables publication."""
    return cargo.get("package", {}).get("publish") is False


def automation_contains_publish_command(source: str) -> bool:
    """Return whether automation can invoke or authenticate publication."""
    forbidden = (
        "cargo publish",
        "CARGO_REGISTRY_TOKEN",
        "CRATES_IO_TOKEN",
        "crates.io/api",
    )
    return any(marker in source for marker in forbidden)
