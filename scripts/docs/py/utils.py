import os
import re


def get_project_root():
    """Returns the absolute path to the project root."""
    # scripts/docs/py/utils.py -> ../../../
    return os.path.abspath(os.path.join(os.path.dirname(__file__), "../../../"))


def find_md_files(root_dir, exclude_dirs=None):
    """
    Recursively finds all .md files in root_dir, skipping excluded directories.
    """
    if exclude_dirs is None:
        exclude_dirs = {".git", "fixtures", "node_modules", "target", "generated"}

    md_files = []
    for root, dirs, files in os.walk(root_dir):
        # Filter excludes in-place to prevent traversing them
        # We start iterating from a copy of the list to safely modify it
        dirs[:] = [d for d in dirs if d not in exclude_dirs and not d.startswith(".")]

        for f in files:
            if f.endswith(".md"):
                md_files.append(os.path.join(root, f))
    return md_files


def extract_links(content):
    """
    Extracts links from markdown content.
    Returns list of (text, url) tuples.
    """
    # Strip HTML comments to avoid false positives in templates
    content = re.sub(r"<!--.*?-->", "", content, flags=re.DOTALL)

    # Find links [text](url)
    # Regex captures: 1=text, 2=url (without anchor)
    return re.findall(r"\[([^\]]*)\]\(([^)#\s]+)(?:#[^)]*)?\)", content)
