#!/usr/bin/env python3
import os
import sys
import argparse


import utils


def main():
    """Main entry point for checking broken internal links in documentation."""
    parser = argparse.ArgumentParser(description="Check broken internal links in docs.")
    parser.add_argument("--root", default=".", help="Project root directory")
    args = parser.parse_args()

    # Use project root from args if provided, otherwise detect
    project_root = os.path.abspath(args.root)
    if args.root == ".":
        project_root = utils.get_project_root()

    docs_dir = os.path.join(project_root, "docs")

    if not os.path.exists(docs_dir):
        print(f"Error: docs directory not found at {docs_dir}")
        sys.exit(1)

    broken = []
    checked_files = 0
    checked_links = 0

    md_files = utils.find_md_files(docs_dir)

    for filepath in md_files:
        rel_filepath = os.path.relpath(filepath, project_root)
        checked_files += 1

        try:
            with open(filepath, "r", encoding="utf-8") as fh:
                content = fh.read()
        except Exception as e:
            print(f"Error reading {rel_filepath}: {e}")
            continue

        links = utils.extract_links(content)

        for text, link in links:
            checked_links += 1
            if (
                link.startswith("http")
                or link.startswith("mailto:")
                or link.startswith("ftp:")
            ):
                continue

            # Resolve target path
            if link.startswith("/"):
                # Absolute from project root (rarely used in MD but valid in some contexts)
                target = os.path.join(project_root, link.lstrip("/"))
            else:
                # Relative to current file
                target = os.path.normpath(os.path.join(os.path.dirname(filepath), link))

            if not os.path.exists(target):
                broken.append(
                    (
                        rel_filepath,
                        text,
                        link,
                        os.path.relpath(target, project_root),
                    )
                )

    print(f"Checked {checked_files} files, {checked_links} internal links.")

    if broken:
        print(f"Found {len(broken)} broken internal links:")
        for fp, text, link, target in sorted(broken):
            print(f"  {fp}: [{text}]({link}) -> {target} (missing)")
        sys.exit(1)
    else:
        print("No broken internal links found.")
        sys.exit(0)


if __name__ == "__main__":
    main()
