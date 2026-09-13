#!/usr/bin/env python3
"""
Zyanya SourceForge Release Publisher & Mirror Automation
Integrates with the SourceForge File Release System (FRS) and REST API.
"""

import sys
import os
import argparse
import subprocess
from pathlib import Path
import urllib.request
import urllib.parse
import urllib.error
import json

# Ensure UTF-8 output
if sys.stdout.encoding != 'utf-8':
    try:
        sys.stdout.reconfigure(encoding='utf-8')
        sys.stderr.reconfigure(encoding='utf-8')
    except Exception:
        pass

DEFAULT_API_KEY = "521f5694-40ce-467f-a91c-440574030733"
DEFAULT_PROJECT = "zyanya"
DEFAULT_VERSION = "v1.0.0"

REPO_ROOT = Path(__file__).resolve().parent.parent
DIST_DIR = REPO_ROOT.parent / "releases"


def check_project_status(project: str) -> bool:
    """Checks if the SourceForge project page is publicly reachable."""
    url = f"https://sourceforge.net/projects/{project}/"
    print(f"\n🔍 Checking SourceForge project status: {url}")
    req = urllib.request.Request(url, headers={"User-Agent": "curl/8.4.0"})
    try:
        with urllib.request.urlopen(req, timeout=10) as resp:
            print(f"  ✅ Project '{project}' is active and reachable (HTTP {resp.status})")
            return True
    except urllib.error.HTTPError as e:
        if e.code == 404:
            print(f"  ⚠️ Project '{project}' returned HTTP 404 (Not Found).")
            print("  👉 Please verify the project Unix name or complete registration at:")
            print("     https://sourceforge.net/p/add_project")
        else:
            print(f"  ⚠️ HTTP {e.code}: {e.reason}")
        return False
    except Exception as e:
        print(f"  ❌ Error querying SourceForge: {e}")
        return False


def set_file_default(project: str, api_key: str, file_path: str, defaults: list, label: str = None) -> bool:
    """
    Sets default download platform(s) for a file via the SourceForge File Release REST API.
    Endpoint: PUT https://sourceforge.net/projects/[PROJECT]/files/[FILE PATH]
    """
    url = f"https://sourceforge.net/projects/{project}/files/{file_path}"
    print(f"\n⚙️ Configuring download properties for: {file_path}")
    print(f"  Target: {url}")
    print(f"  Default platforms: {', '.join(defaults)}")

    params = [("api_key", api_key)]
    for d in defaults:
        params.append(("default", d))
    if label:
        params.append(("download_label", label))

    data = urllib.parse.urlencode(params).encode("utf-8")
    req = urllib.request.Request(
        url,
        data=data,
        headers={
            "Accept": "application/json",
            "User-Agent": "curl/8.4.0",
            "Content-Type": "application/x-www-form-urlencoded"
        },
        method="PUT"
    )

    try:
        with urllib.request.urlopen(req, timeout=15) as resp:
            response_body = resp.read().decode("utf-8")
            result = json.loads(response_body)
            if "error" in result and result["error"]:
                print(f"  ❌ SourceForge API returned error: {result['error']}")
                return False
            print("  ✅ File release properties updated successfully!")
            return True
    except urllib.error.HTTPError as e:
        body = ""
        try:
            body = e.read().decode("utf-8")
        except Exception:
            pass
        print(f"  ❌ HTTP {e.code} Error: {e.reason}")
        if body:
            print(f"  Response: {body[:300]}")
        return False
    except Exception as e:
        print(f"  ❌ Request failed: {e}")
        return False


def print_upload_instructions(project: str, version: str, username: str, files_dir: Path):
    """Outputs the standard rsync and scp commands for SourceForge FRS upload."""
    remote_path = f"/home/frs/project/{project}/{version}"
    print("\n" + "=" * 70)
    print("  SOURCEFORGE FRS (FILE RELEASE SYSTEM) UPLOAD GUIDE")
    print("=" * 70)
    print(f"  Project:     {project}")
    print(f"  Release:     {version}")
    print(f"  Remote Path: {remote_path}")
    print("=" * 70)
    print("\n1. RSYNC (Recommended for reliability and resume support):")
    print(f"   rsync -avP -e ssh {files_dir}/* {username}@frs.sourceforge.net:{remote_path}/")
    print("\n2. SCP (Direct single or recursive copy):")
    print(f"   scp {files_dir}/* {username}@frs.sourceforge.net:{remote_path}/")
    print("\n3. SFTP Interactive:")
    print(f"   sftp {username}@frs.sourceforge.net")
    print(f"   sftp> mkdir {remote_path}")
    print(f"   sftp> cd {remote_path}")
    print(f"   sftp> put {files_dir}/*")
    print("=" * 70)


def main():
    parser = argparse.ArgumentParser(description="Zyanya SourceForge Release Publisher")
    parser.add_argument("--api-key", default=os.environ.get("SOURCEFORGE_API_KEY", DEFAULT_API_KEY),
                        help="SourceForge Release API Key")
    parser.add_argument("--project", default=DEFAULT_PROJECT, help="SourceForge project Unix name (default: zyanya)")
    parser.add_argument("--version", default=DEFAULT_VERSION, help="Release version (default: v1.0.0)")
    parser.add_argument("--username", default=os.environ.get("SOURCEFORGE_USER", "scotthawk"),
                        help="SourceForge username for SFTP/rsync")
    parser.add_argument("--action", choices=["check", "instructions", "set-defaults", "all"], default="check",
                        help="Action to perform")
    args = parser.parse_args()

    print("=" * 70)
    print(f"  ZYANYA SOURCEFORGE MIRROR AUTOMATION ({args.project} - {args.version})")
    print("=" * 70)
    print(f"  API Key: {args.api_key[:8]}...{args.api_key[-4:]}")

    if args.action in ["check", "all"]:
        is_live = check_project_status(args.project)
        if not is_live and args.action == "all":
            print("\n  ⚠️ Aborting automated API updates until project is registered and live.")
            return

    if args.action in ["instructions", "all"]:
        print_upload_instructions(args.project, args.version, args.username, DIST_DIR)

    if args.action in ["set-defaults"]:
        print("\n🚀 Configuring default download files on SourceForge...")
        win_file = f"{args.version}/zyanya-{args.version}-mainnet-windows-x64.zip"
        linux_file = f"{args.version}/zyanya-{args.version}-mainnet-linux-x86_64.tar.gz"

        # Windows default
        set_file_default(
            project=args.project,
            api_key=args.api_key,
            file_path=win_file,
            defaults=["windows"],
            label=f"Download Zyanya {args.version} for Windows"
        )

        # Linux / BSD default
        set_file_default(
            project=args.project,
            api_key=args.api_key,
            file_path=linux_file,
            defaults=["linux", "bsd", "solaris", "others"],
            label=f"Download Zyanya {args.version} for Linux"
        )


if __name__ == "__main__":
    main()
