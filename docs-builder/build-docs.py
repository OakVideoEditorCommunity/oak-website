#!/usr/bin/env python3
import os
import shutil
import subprocess
import sys
import tempfile

SRC_DIR = os.environ.get("DOCS_SRC_DIR", "/data/oak-docs")
OUT_DIR = os.environ.get("DOCS_OUT_DIR", "/data/docs-html")


def copy_excluding(src: str, dst: str, exclude: set[str]) -> None:
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


def build_lang(src: str, out: str) -> bool:
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
        src, out,
    ]
    print(f"Building docs: {' '.join(cmd)}")
    result = subprocess.run(cmd, cwd=src)
    return result.returncode == 0


def main():
    if not os.path.isdir(SRC_DIR):
        print(f"ERROR: docs source directory {SRC_DIR} does not exist", file=sys.stderr)
        sys.exit(1)

    os.makedirs(OUT_DIR, exist_ok=True)

    # Build Chinese docs from root, excluding the `en` subdirectory
    with tempfile.TemporaryDirectory() as tmp_zh_src:
        copy_excluding(SRC_DIR, tmp_zh_src, {"en"})
        ok_zh = build_lang(tmp_zh_src, os.path.join(OUT_DIR, "zh"))

    # Build English docs from en/; copy conf.py since en/ lacks one
    with tempfile.TemporaryDirectory() as tmp_en_src:
        copy_excluding(os.path.join(SRC_DIR, "en"), tmp_en_src, set())
        root_conf = os.path.join(SRC_DIR, "conf.py")
        if os.path.isfile(root_conf):
            shutil.copy2(root_conf, os.path.join(tmp_en_src, "conf.py"))
            # Patch language to English
            conf_path = os.path.join(tmp_en_src, "conf.py")
            with open(conf_path, "r", encoding="utf-8") as f:
                content = f.read()
            content = content.replace("language = 'zh_CN'", "language = 'en'")
            with open(conf_path, "w", encoding="utf-8") as f:
                f.write(content)
        ok_en = build_lang(tmp_en_src, os.path.join(OUT_DIR, "en"))

    if not ok_zh and not ok_en:
        sys.exit(1)

    print(f"Docs built successfully into {OUT_DIR}")


if __name__ == "__main__":
    main()
