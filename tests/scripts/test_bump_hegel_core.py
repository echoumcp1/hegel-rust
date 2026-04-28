from pathlib import Path

import bump_hegel_core
import pytest

ROOT = Path(__file__).resolve().parents[2]


# ---- parse_current_version --------------------------------------------------


@pytest.mark.parametrize(
    "prefix",
    ["", "pub ", "pub(crate) ", "pub(super) ", "pub(in crate::server) "],
)
def test_parse_current_version_handles_visibility(prefix: str) -> None:
    text = f'{prefix}const HEGEL_SERVER_VERSION: &str = "0.4.7";\n'
    assert bump_hegel_core.parse_current_version(text) == "0.4.7"


def test_parse_current_version_finds_constant_among_other_lines() -> None:
    text = (
        "use crate::backend::{DataSource};\n"
        "\n"
        'pub(super) const SUPPORTED_PROTOCOL_VERSIONS: (&str, &str) = ("0.10", "0.10");\n'
        'pub(super) const HEGEL_SERVER_VERSION: &str = "1.2.3";\n'
        "\n"
        "fn main() {}\n"
    )
    assert bump_hegel_core.parse_current_version(text) == "1.2.3"


def test_parse_current_version_raises_when_missing() -> None:
    with pytest.raises(ValueError, match="HEGEL_SERVER_VERSION"):
        bump_hegel_core.parse_current_version("// no version here\n")


def test_parse_current_version_against_actual_session_rs() -> None:
    """Regression test: the regex must match the file as it lives on main right now."""
    text = (ROOT / "src" / "server" / "session.rs").read_text()
    version = bump_hegel_core.parse_current_version(text)
    assert version
    # Sanity-check the shape — a bump never produces something like "0".
    assert version.count(".") == 2


# ---- update_session ---------------------------------------------------------


def test_update_session_changes_version_and_protocol_upper_bound() -> None:
    text = (
        'pub(super) const SUPPORTED_PROTOCOL_VERSIONS: (&str, &str) = ("0.10", "0.10");\n'
        'pub(super) const HEGEL_SERVER_VERSION: &str = "0.4.7";\n'
    )
    out = bump_hegel_core.update_session(text, "0.4.14", "0.11")
    assert (
        'pub(super) const HEGEL_SERVER_VERSION: &str = "0.4.14";' in out
    )
    assert (
        'pub(super) const SUPPORTED_PROTOCOL_VERSIONS: (&str, &str) = ("0.10", "0.11");'
        in out
    )


def test_update_session_preserves_visibility_modifier() -> None:
    text = (
        'const SUPPORTED_PROTOCOL_VERSIONS: (&str, &str) = ("0.10", "0.10");\n'
        'const HEGEL_SERVER_VERSION: &str = "0.4.7";\n'
    )
    out = bump_hegel_core.update_session(text, "0.4.14", "0.11")
    assert 'const HEGEL_SERVER_VERSION: &str = "0.4.14";' in out
    # ensure we didn't accidentally inject a `pub(super)` prefix
    assert "pub(super)" not in out


def test_update_session_only_replaces_upper_protocol_bound() -> None:
    text = (
        'pub(super) const SUPPORTED_PROTOCOL_VERSIONS: (&str, &str) = ("0.5", "0.10");\n'
        'pub(super) const HEGEL_SERVER_VERSION: &str = "0.4.7";\n'
    )
    out = bump_hegel_core.update_session(text, "0.4.14", "0.11")
    assert '("0.5", "0.11")' in out
    assert '("0.5", "0.10")' not in out


def test_update_session_raises_when_version_missing() -> None:
    text = (
        'pub(super) const SUPPORTED_PROTOCOL_VERSIONS: (&str, &str) = ("0.10", "0.10");\n'
    )
    with pytest.raises(ValueError, match="HEGEL_SERVER_VERSION"):
        bump_hegel_core.update_session(text, "0.4.14", "0.11")


def test_update_session_raises_when_protocol_missing() -> None:
    text = 'pub(super) const HEGEL_SERVER_VERSION: &str = "0.4.7";\n'
    with pytest.raises(ValueError, match="SUPPORTED_PROTOCOL_VERSIONS"):
        bump_hegel_core.update_session(text, "0.4.14", "0.11")


def test_update_session_against_actual_session_rs() -> None:
    """Smoke test: the substitution should apply cleanly to the real file."""
    text = (ROOT / "src" / "server" / "session.rs").read_text()
    out = bump_hegel_core.update_session(text, "9.9.9", "0.99")
    assert 'HEGEL_SERVER_VERSION: &str = "9.9.9";' in out
    assert ', "0.99");' in out


# ---- update_flake -----------------------------------------------------------


def test_update_flake_rewrites_tag() -> None:
    text = (
        'hegel.url = "git+https://github.com/hegeldev/hegel-core'
        '?dir=nix&ref=refs/tags/v0.4.6";\n'
    )
    out = bump_hegel_core.update_flake(text, "0.4.14")
    assert "refs/tags/v0.4.14" in out
    assert "refs/tags/v0.4.6" not in out


def test_update_flake_raises_when_tag_missing() -> None:
    with pytest.raises(ValueError, match="refs/tags"):
        bump_hegel_core.update_flake("nothing to see here\n", "0.4.14")


def test_update_flake_against_actual_flake_nix() -> None:
    text = (ROOT / "nix" / "flake.nix").read_text()
    out = bump_hegel_core.update_flake(text, "9.9.9")
    assert "refs/tags/v9.9.9" in out


# ---- format_release_md ------------------------------------------------------


def test_format_release_md_single_release() -> None:
    out = bump_hegel_core.format_release_md(
        "0.4.10",
        [{"version": "0.4.10", "body": "Fixed a bug."}],
    )
    assert out.startswith("RELEASE_TYPE: patch\n\n")
    assert "incorporating the following change:" in out  # singular
    assert "> Fixed a bug." in out
    assert "[v0.4.10](https://github.com/hegeldev/hegel-core/releases/tag/v0.4.10)" in out


def test_format_release_md_multiple_releases_uses_plural() -> None:
    out = bump_hegel_core.format_release_md(
        "0.4.14",
        [
            {"version": "0.4.10", "body": "First fix."},
            {"version": "0.4.14", "body": "Second fix."},
        ],
    )
    assert "incorporating the following changes:" in out  # plural
    assert "> First fix." in out
    assert "> Second fix." in out
    # Each section ends with a quoted link line.
    assert out.count("> — [v") == 2


def test_format_release_md_quotes_blank_lines_as_bare_caret() -> None:
    out = bump_hegel_core.format_release_md(
        "0.4.10",
        [{"version": "0.4.10", "body": "Line one.\n\nLine three."}],
    )
    assert "> Line one.\n>\n> Line three." in out
