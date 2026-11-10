import json
import os
import subprocess
import sys
from datetime import datetime

script_dir = os.path.dirname(os.path.abspath(__file__))
root_dir = os.path.abspath(os.path.join(script_dir, ".."))
client_dir = "/home/sqrilizz/Downloads/Client"
parity_json_path = os.path.join(root_dir, "data", "parity.json")

def get_git_info():
    if not os.path.isdir(os.path.join(client_dir, ".git")):
        return {
            "commit": "df65652",
            "commit_title": "split the protocol translator by version and pick its rewrites from dispatch tables (#891)",
            "commit_date": "2026-11-07",
            "tag": "client-v0.2.10+26.2",
            "updated_at": datetime.now().strftime("%Y-%m-%d"),
            "test_count": 88
        }

    try:
        subprocess.run(["git", "fetch", "upstream"], cwd=client_dir, capture_output=True, timeout=10)
    except Exception:
        pass

    target_ref = "upstream/master"
    try:
        subprocess.run(["git", "rev-parse", target_ref], cwd=client_dir, capture_output=True, check=True)
    except Exception:
        target_ref = "HEAD"

    def run_cmd(args):
        return subprocess.run(["git"] + args, cwd=client_dir, capture_output=True, text=True, check=True).stdout.strip()

    commit_full = run_cmd(["rev-parse", target_ref])
    commit_short = commit_full[:7]
    commit_date = run_cmd(["log", "-1", "--format=%cs", target_ref])
    commit_title = run_cmd(["log", "-1", "--format=%s", target_ref])

    tag = "client-v0.2.10+26.2"
    try:
        tag = run_cmd(["describe", "--tags", target_ref])
    except Exception:
        pass

    tests_raw = run_cmd(["ls-tree", "-r", "--name-only", target_ref, "tests/ingame"])
    tests = [t for t in tests_raw.splitlines() if t.endswith(".toml")]

    return {
        "commit": commit_short,
        "commit_title": commit_title,
        "commit_date": commit_date,
        "tag": tag,
        "updated_at": datetime.now().strftime("%Y-%m-%d"),
        "test_count": len(tests)
    }

def main():
    meta = get_git_info()
    existing_milestones = []
    items = []

    if os.path.exists(parity_json_path):
        with open(parity_json_path, "r", encoding="utf-8") as f:
            existing = json.load(f)
            if isinstance(existing, dict):
                items = existing.get("items", [])
                existing_milestones = existing.get("recent_milestones", [])
            elif isinstance(existing, list):
                items = existing

    output = {
        "meta": meta,
        "recent_milestones": existing_milestones,
        "items": items
    }

    os.makedirs(os.path.dirname(parity_json_path), exist_ok=True)
    with open(parity_json_path, "w", encoding="utf-8") as f:
        json.dump(output, f, indent=2, ensure_ascii=False)

    print(f"Synced parity.json with commit {meta['commit']} ({meta['tag']}): {len(items)} items, {meta['test_count']} in-game tests.")

if __name__ == "__main__":
    main()
