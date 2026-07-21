#!/usr/bin/env python3
"""Fetch CI errors from GitHub Actions job annotations."""
import json
import subprocess
import sys

jobs = [
    ("Test", 88653014817),
    ("Docs Build", 88653014838),
    ("Clippy", 88653014863),
    ("Build (Windows)", 88653014864),
]

gh_token = "ghp_GvJ5q7pX1R2mN9kL4wF8Tzp"
repo = "azreenariff/wikilabs-ai-copilot"

for name, job_id in jobs:
    print(f"\n{'='*60}")
    print(f"JOB: {name} (ID: {job_id})")
    print(f"{'='*60}")
    
    # Fetch annotations via API
    r = subprocess.run(
        ["curl", "-s", "-u", f"azreenariff:{gh_token}",
         f"https://api.github.com/repos/{repo}/actions/jobs/{job_id}/annotations"],
        capture_output=True, text=True
    )
    try:
        data = json.loads(r.stdout)
        if "error" in data:
            print(f"API ERROR: {data['error']}")
            continue
        for ann in data.get("annotations", []):
            print(f"  [{ann['level']}] {ann['message']}")
            if ann.get('start_line'):
                print(f"    Line {ann['start_line']}: {ann.get('start_column', '')}")
    except Exception as e:
        print(f"Parse error: {e}")
        print(r.stdout[:500])