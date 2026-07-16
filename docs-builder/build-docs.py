#!/usr/bin/env python3
"""Build the Oak documentation with Sphinx, supporting multiple versions.

Environment variables:
  DOCS_SRC_DIR   Local documentation source directory (default /data/oak-docs).
  DOCS_OUT_DIR   Output directory for the built HTML (default /data/docs-html).
  DOCS_GIT_URL   If set, the docs repository is cloned (full clone, so tags
                 are available) from this URL instead of using DOCS_SRC_DIR.
  DOCS_VERSIONS  Comma-separated git refs to build, in order (first = latest).
                 If unset, all tags matching a loose semver pattern are built,
                 newest first; if there are none (or the source is not a git
                 repository), a single version named "latest" is built.

Output layout:
  {OUT_DIR}/versions.json           {"versions": [...], "latest": "..."}
  {OUT_DIR}/{version}/{en,zh}/      built HTML plus toc.json
"""
import json
import os
import re
import shutil
import subprocess
import sys
import tempfile

SRC_DIR = os.environ.get("DOCS_SRC_DIR", "/data/oak-docs")
OUT_DIR = os.environ.get("DOCS_OUT_DIR", "/data/docs-html")
DOCS_GIT_URL = os.environ.get("DOCS_GIT_URL", "")
DOCS_VERSIONS = os.environ.get("DOCS_VERSIONS", "")

# Loose semver tag: optional leading "v", digits/dots, optional suffix (e.g.
# v0.1.0, 0.2.0, v1.0.0-rc.1).
SEMVER_TAG_RE = re.compile(r"^v?(\d+(?:\.\d+)*)(?:-([0-9A-Za-z.-]+))?$")


def clone_docs_repo(url: str) -> str:
    """Clone the documentation repository into a temporary directory.

    A full clone (not --depth 1) is made so that all tags are available for
    version resolution.
    """
    tmp_dir = tempfile.mkdtemp(prefix="oak-docs-")
    print(f"Cloning docs repository from {url} into {tmp_dir}")
    result = subprocess.run(
        ["git", "clone", url, tmp_dir],
        capture_output=True,
        text=True,
    )
    if result.returncode != 0:
        print(f"ERROR: failed to clone docs repo: {result.stderr}", file=sys.stderr)
        sys.exit(1)
    return tmp_dir


def is_git_repo(path: str) -> bool:
    """Return True if path is a git working tree."""
    result = subprocess.run(
        ["git", "-C", path, "rev-parse"],
        capture_output=True,
        text=True,
    )
    return result.returncode == 0


def clone_local_repo(path: str) -> str:
    """Clone a local git repository into a temporary directory.

    Cloning a local path includes its tags, so versions can be checked out
    without touching the original working tree.
    """
    tmp_dir = tempfile.mkdtemp(prefix="oak-docs-")
    print(f"Cloning local docs repository {path} into {tmp_dir}")
    result = subprocess.run(
        ["git", "clone", path, tmp_dir],
        capture_output=True,
        text=True,
    )
    if result.returncode != 0:
        print(f"ERROR: failed to clone local docs repo: {result.stderr}", file=sys.stderr)
        sys.exit(1)
    return tmp_dir


def version_key(tag: str) -> tuple:
    """Return a sort key for a loose semver tag; higher sorts newer.

    The tag is split into its numeric release parts and an optional
    pre-release suffix. Numbers compare numerically, non-numeric suffix parts
    lexically, and a plain release sorts newer than a pre-release of the same
    version. The tag must already match SEMVER_TAG_RE.
    """
    m = SEMVER_TAG_RE.match(tag)
    release = tuple(int(p) for p in m.group(1).split("."))
    pre = m.group(2)
    if pre is None:
        pre_key = (1,)
    else:
        pre_key = (0,) + tuple(
            (0, int(p)) if p.isdigit() else (1, p)
            for p in re.split(r"[.-]", pre)
        )
    return (release, pre_key)


def list_semver_tags(repo: str) -> list[str]:
    """List tags in repo matching the loose semver pattern, newest first."""
    result = subprocess.run(
        ["git", "-C", repo, "tag", "--list"],
        capture_output=True,
        text=True,
    )
    if result.returncode != 0:
        print(f"WARNING: failed to list tags in {repo}: {result.stderr}", file=sys.stderr)
        return []
    tags = [t.strip() for t in result.stdout.splitlines() if SEMVER_TAG_RE.match(t.strip())]
    return sorted(tags, key=version_key, reverse=True)


def normalize_version_name(ref: str) -> str:
    """Turn a git ref into a directory-safe version name.

    A leading "v" followed by a digit is stripped (v0.1.0 -> 0.1.0) and
    path-hostile characters (slashes, backslashes) are replaced with "-".
    """
    name = re.sub(r"^v(?=\d)", "", ref)
    return name.replace("/", "-").replace("\\", "-")


def checkout_ref(repo: str, ref: str) -> bool:
    """Check out ref in repo and remove untracked files. Returns True on success."""
    for args in (["checkout", "--force", ref], ["clean", "-fdx"]):
        result = subprocess.run(
            ["git", "-C", repo, *args],
            capture_output=True,
            text=True,
        )
        if result.returncode != 0:
            print(f"ERROR: git {' '.join(args)} failed: {result.stderr.strip()}", file=sys.stderr)
            return False
    print(f"Checked out ref {ref}")
    return True


def copy_tree(src: str, dst: str, exclude: set[str]) -> None:
    """Copy src tree to dst, excluding top-level items in `exclude`."""
    os.makedirs(dst, exist_ok=True)
    for item in os.listdir(src):
        if item in exclude:
            continue
        s = os.path.join(src, item)
        d = os.path.join(dst, item)
        if os.path.isdir(s):
            shutil.copytree(s, d, dirs_exist_ok=True)
        else:
            shutil.copy2(s, d)


def read_toctree(index_rst: str) -> list[str]:
    """Read the toctree entries from an index.rst file.

    Returns the ordered list of document slugs as they appear in the toctree.
    Entries must be indented lines below the ``.. toctree::`` directive. Blank
    lines and option lines (e.g. ``:maxdepth: 2``) are skipped, and the block
    ends when a non-indented line or a new directive is reached.
    """
    if not os.path.isfile(index_rst):
        return []

    entries: list[str] = []
    in_toctree = False
    with open(index_rst, "r", encoding="utf-8") as f:
        for line in f:
            stripped = line.strip()
            if not in_toctree:
                if stripped.startswith(".. toctree::"):
                    in_toctree = True
                continue

            # End the block on a new top-level directive or a non-indented line.
            if stripped.startswith(".. ") or (stripped and not line[0].isspace()):
                break
            if not stripped or stripped.startswith(":"):
                continue

            # Document entries are indented; strip any .rst suffix.
            entry = stripped.removesuffix(".rst")
            if entry:
                entries.append(entry)
    return entries


def write_toc_json(out_dir: str, index_rst: str) -> None:
    """Write a toc.json file with the order defined by the index toctree."""
    toc = read_toctree(index_rst)
    if toc:
        with open(os.path.join(out_dir, "toc.json"), "w", encoding="utf-8") as f:
            json.dump(toc, f, ensure_ascii=False, indent=2)


def build_lang(src: str, out: str, index_rst: str, extra_sphinx_args: list[str] | None = None) -> bool:
    """Build Sphinx docs for a single language directory."""
    if not os.path.isdir(src):
        print(f"Source dir {src} does not exist, skipping")
        return False

    shutil.rmtree(out, ignore_errors=True)
    os.makedirs(out, exist_ok=True)

    cmd = [
        sys.executable, "-m", "sphinx",
        "-b", "html",
        "--keep-going",
    ]
    if extra_sphinx_args:
        cmd.extend(extra_sphinx_args)
    cmd.extend([src, out])
    print(f"Building docs: {' '.join(cmd)}")
    result = subprocess.run(cmd, cwd=src)
    if result.returncode == 0:
        write_toc_json(out, index_rst)
    return result.returncode == 0


def patch_language(conf_path: str, language: str) -> None:
    """Replace the language setting in a copied conf.py."""
    if not os.path.isfile(conf_path):
        return
    with open(conf_path, "r", encoding="utf-8") as f:
        content = f.read()
    # Replace either the English or the old Chinese default.
    content = content.replace("language = 'en'", f"language = '{language}'")
    content = content.replace("language = 'zh_CN'", f"language = '{language}'")
    with open(conf_path, "w", encoding="utf-8") as f:
        f.write(content)


def build_langs(src_dir: str, version_out: str) -> bool:
    """Build the en and zh docs from src_dir into version_out/{en,zh}.

    Returns True if at least one language built successfully.
    """
    with tempfile.TemporaryDirectory() as tmp_en_src:
        # Build English docs from the repository root, excluding the Chinese source tree.
        copy_tree(src_dir, tmp_en_src, {"zh", ".git"})
        en_index = os.path.join(src_dir, "index.rst")
        ok_en = build_lang(tmp_en_src, os.path.join(version_out, "en"), en_index)

    # Build Chinese docs from zh/; copy the root conf.py since zh/ lacks one.
    ok_zh = False
    zh_src = os.path.join(src_dir, "zh")
    if os.path.isdir(zh_src):
        with tempfile.TemporaryDirectory() as tmp_zh_src:
            copy_tree(zh_src, tmp_zh_src, set())
            root_conf = os.path.join(src_dir, "conf.py")
            if os.path.isfile(root_conf):
                shutil.copy2(root_conf, os.path.join(tmp_zh_src, "conf.py"))
                patch_language(os.path.join(tmp_zh_src, "conf.py"), "zh_CN")
            zh_index = os.path.join(zh_src, "index.rst")
            ok_zh = build_lang(tmp_zh_src, os.path.join(version_out, "zh"), zh_index)
    else:
        print(f"No zh/ directory in {src_dir}, skipping Chinese build")

    return ok_en or ok_zh


def write_versions_json(versions: list[str]) -> None:
    """Write versions.json listing the built versions (newest first) and latest."""
    data = {"versions": versions, "latest": versions[0]}
    path = os.path.join(OUT_DIR, "versions.json")
    with open(path, "w", encoding="utf-8") as f:
        json.dump(data, f, ensure_ascii=False, indent=2)
    print(f"Wrote {path}: {json.dumps(data)}")


def cleanup_stale_outputs(keep: list[str]) -> None:
    """Remove top-level OUT_DIR entries that are not in the built version list.

    This covers legacy top-level en/zh dirs from the old layout and versions
    that are no longer built. Failures are logged as warnings, not fatal.
    """
    keep_set = set(keep) | {"versions.json"}
    for entry in os.listdir(OUT_DIR):
        if entry in keep_set:
            continue
        path = os.path.join(OUT_DIR, entry)
        try:
            if os.path.isdir(path):
                shutil.rmtree(path, ignore_errors=True)
            else:
                os.remove(path)
            print(f"Removed stale output {path}")
        except OSError as e:
            print(f"WARNING: failed to remove stale output {path}: {e}", file=sys.stderr)


def main():
    os.makedirs(OUT_DIR, exist_ok=True)

    requested = [r.strip() for r in DOCS_VERSIONS.split(",") if r.strip()]

    # Resolve the source. Git sources are cloned into a temp dir so refs can
    # be checked out freely; a plain directory source is used as-is and only
    # supports the single-version "latest" build.
    temp_clone = None
    if DOCS_GIT_URL:
        src_dir = temp_clone = clone_docs_repo(DOCS_GIT_URL)
        from_git = True
    elif is_git_repo(SRC_DIR):
        src_dir = temp_clone = clone_local_repo(SRC_DIR)
        from_git = True
    else:
        src_dir = SRC_DIR
        from_git = False

    try:
        if not os.path.isdir(src_dir):
            print(f"ERROR: docs source directory {src_dir} does not exist", file=sys.stderr)
            sys.exit(1)

        if requested and not from_git:
            print(
                "ERROR: DOCS_VERSIONS requires a git source (set DOCS_GIT_URL or "
                "point DOCS_SRC_DIR at a git repository)",
                file=sys.stderr,
            )
            sys.exit(1)

        # Resolve which refs to build: explicit DOCS_VERSIONS wins, then all
        # semver tags newest-first, otherwise a single "latest" build.
        if requested:
            refs = requested
        elif from_git:
            refs = list_semver_tags(src_dir)
            if refs:
                print(f"Found semver tags: {', '.join(refs)}")
        else:
            refs = []

        built: list[str] = []
        if refs:
            for ref in refs:
                name = normalize_version_name(ref)
                version_out = os.path.join(OUT_DIR, name)
                print(f"=== Building version {name} (ref {ref}) ===")
                if not checkout_ref(src_dir, ref):
                    print(f"ERROR: skipping version {name}: cannot check out {ref}", file=sys.stderr)
                    continue
                shutil.rmtree(version_out, ignore_errors=True)
                if build_langs(src_dir, version_out):
                    built.append(name)
                    print(f"Version {name} built into {version_out}")
                else:
                    print(f"ERROR: version {name} failed to build", file=sys.stderr)
        else:
            print("=== Building version latest ===")
            version_out = os.path.join(OUT_DIR, "latest")
            shutil.rmtree(version_out, ignore_errors=True)
            if build_langs(src_dir, version_out):
                built.append("latest")
                print(f"Version latest built into {version_out}")

        if not built:
            print("ERROR: all documentation builds failed", file=sys.stderr)
            sys.exit(1)

        write_versions_json(built)
        cleanup_stale_outputs(built)
        print(f"Docs built successfully into {OUT_DIR} (versions: {', '.join(built)})")
    finally:
        if temp_clone:
            shutil.rmtree(temp_clone, ignore_errors=True)


if __name__ == "__main__":
    main()
