import json
import os
import subprocess
import sys
from datetime import datetime

script_dir = os.path.dirname(os.path.abspath(__file__))
root_dir = os.path.abspath(os.path.join(script_dir, "..", ".."))
client_dir = "/home/sqrilizz/Downloads/Client"
parity_json_path = os.path.join(root_dir, "new-website", "data", "parity.json")

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

    if os.path.exists(parity_json_path):
        with open(parity_json_path, "r", encoding="utf-8") as f:
            existing = json.load(f)
            if isinstance(existing, dict) and "items" in existing:
                items = existing["items"]
            elif isinstance(existing, list):
                items = existing
            else:
                items = []
    else:
        items = []

    latest_items = [
        {
            "name": "Signs, Hanging Signs & Text Screen",
            "category": "blocks",
            "status": "Ready",
            "tag": "tests/ingame/blocks/signs.toml",
            "description": "Standing, wall, and hanging sign rendering in world space with interactive text editing GUI."
        },
        {
            "name": "Multipart Block Conditions",
            "category": "blocks",
            "status": "Ready",
            "tag": "tests/ingame/blocks/multipart_conditions.toml",
            "description": "Multipart condition evaluation for redstone wire connections, shelves, and chiseled bookshelves."
        },
        {
            "name": "3D Nameplates & Mob Tags",
            "category": "entities",
            "status": "Ready",
            "tag": "tests/ingame/entities/nameplates.toml",
            "description": "Billboarded 3D nameplates for players and custom named mobs with distance culling."
        },
        {
            "name": "3D Player Heads & Container Tooltips",
            "category": "items",
            "status": "Ready",
            "tag": "tests/ingame/items/heads_and_tooltips.toml",
            "description": "3D player head skin rendering in inventories and hover tooltip text evaluation."
        },
        {
            "name": "Third-Person Camera & Body Rotation",
            "category": "player",
            "status": "Ready",
            "tag": "tests/ingame/player/third_person.toml",
            "description": "Third-person F5 camera occlusion culling, head pitch decoupling, and body rotation interpolation."
        },
        {
            "name": "FPS Display & Debug Options",
            "category": "ui",
            "status": "Ready",
            "tag": "tests/ingame/ui/debug_options.toml",
            "description": "FPS performance counter overlay, min/max frame time telemetry, and debug toggles."
        },
        {
            "name": "Container Screen Auto-Close Chat",
            "category": "ui",
            "status": "Ready",
            "tag": "tests/ingame/ui/chat_replaced_by_container.toml",
            "description": "Smoothly closing active chat box when server opens a container screen without state desync."
        },
        {
            "name": "Multi-Version Dispatch Tables (1.20 - 26.3)",
            "category": "engine",
            "status": "Ready",
            "tag": "pomme-protocol::version",
            "description": "Packet rewrite tables split by version and chosen via compiled dispatch tables without runtime allocations."
        }
    ]

    existing_names = {i["name"]: idx for idx, i in enumerate(items)}
    for lit in latest_items:
        if lit["name"] in existing_names:
            items[existing_names[lit["name"]]] = lit
        else:
            items.append(lit)

    output = {
        "meta": meta,
        "items": items
    }

    os.makedirs(os.path.dirname(parity_json_path), exist_ok=True)
    with open(parity_json_path, "w", encoding="utf-8") as f:
        json.dump(output, f, indent=2, ensure_ascii=False)

    print(f"Synced parity.json with commit {meta['commit']} ({meta['tag']}): {len(items)} items, {meta['test_count']} in-game tests.")

if __name__ == "__main__":
    main()
