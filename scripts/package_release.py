"""Collect verified installers and publish one complete GitHub Release (stdlib only)."""

import hashlib
import json
import os
from pathlib import Path
import re
import shutil
import subprocess
import sys
import zipfile


def command(*args):
    result = subprocess.run(args, text=True, capture_output=True, check=False)
    if result.returncode:
        raise ValueError(f"Command failed: {args[0]} {args[1]} (exit {result.returncode})")
    return result.stdout.strip()


def api(path, *, optional=False):
    result = subprocess.run(["gh", "api", path], text=True, capture_output=True, check=False)
    if result.returncode:
        # Auth, rate-limit and server failures must not look like a missing release.
        if optional and "(HTTP 404)" in result.stderr:
            return None
        raise ValueError(f"GitHub API request failed: {path}")
    return json.loads(result.stdout)


def tag_commit(repo, tag):
    ref = api(f"repos/{repo}/git/ref/tags/{tag}", optional=True)
    if ref is None:
        return None
    obj = ref["object"]
    for _ in range(8):
        if obj["type"] == "commit":
            return obj["sha"]
        if obj["type"] != "tag":
            break
        obj = api(f"repos/{repo}/git/tags/{obj['sha']}")["object"]
    raise ValueError("Release tag does not resolve to a commit")


def config():
    tag = os.environ["RELEASE_TAG"]
    if not re.fullmatch(r"papr-build-[A-Za-z0-9][A-Za-z0-9._-]{0,79}", tag):
        raise ValueError("Use a unique papr-build-* tag containing letters, numbers, '.', '_' or '-'")
    command("git", "check-ref-format", f"refs/tags/{tag}")
    sha = os.environ["BUILD_SHA"]
    if not re.fullmatch(r"[0-9a-f]{40}", sha):
        raise ValueError("BUILD_SHA must be a full commit SHA")
    platform = os.environ["PLATFORMS"]
    if platform not in ("windows", "android", "both"):
        raise ValueError("Select windows, android or both")
    platforms = ["windows", "android"] if platform == "both" else [platform]
    desktop = json.loads(Path("src-tauri/tauri.conf.json").read_text(encoding="utf-8"))["version"]
    mobile = re.search(r"^version:\s*([^\s+]+)\+(\d+)\s*$", Path("mobile/pubspec.yaml").read_text(), re.M)
    if not mobile:
        raise ValueError("pubspec must declare versionName+versionCode")
    code = os.environ.get("ANDROID_BUILD_NUMBER", "")
    if "android" in platforms:
        if not re.fullmatch(r"[1-9][0-9]{0,9}", code) or not int(mobile[2]) < int(code) <= 2100000000:
            raise ValueError("Android build number must exceed pubspec's code and be <= 2100000000")
    for version in (desktop, mobile[1]):
        if not re.fullmatch(r"[A-Za-z0-9][A-Za-z0-9.+-]*", version):
            raise ValueError("Invalid application version")
    return {"tag": tag, "sha": sha, "platforms": platforms, "desktop_version": desktop,
            "android_version": mobile[1], "android_build_number": int(code) if "android" in platforms else None}


def preflight(cfg):
    repo = os.environ["GITHUB_REPOSITORY"]
    commit = tag_commit(repo, cfg["tag"])
    if commit is not None and commit != cfg["sha"]:
        raise ValueError("Existing tag points to a different commit; use a new tag")
    release = api(f"repos/{repo}/releases/tags/{cfg['tag']}", optional=True)
    # The by-tag endpoint only finds published releases. Drafts are in the list,
    # and their tag may not exist until publication.
    if release is None:
        page = 1
        while True:
            releases = api(f"repos/{repo}/releases?per_page=100&page={page}")
            matches = [item for item in releases if item["tag_name"] == cfg["tag"]]
            if len(matches) > 1:
                raise ValueError("Multiple releases use this tag; resolve the ambiguity first")
            if matches:
                release = matches[0]
                break
            if len(releases) < 100:
                break
            page += 1
    if release is not None and not release["draft"]:
        raise ValueError("Release is already published; use a new tag instead of replacing assets")
    if release is not None and release["target_commitish"] != cfg["sha"]:
        raise ValueError("Existing draft targets a different commit; use a new tag")
    return release


def prepare(cfg):
    preflight(cfg)
    with open(os.environ["GITHUB_OUTPUT"], "a", encoding="utf-8") as output:
        output.write(f"sha={cfg['sha']}\n")


def sha256(path):
    digest = hashlib.sha256()
    with path.open("rb") as source:
        for block in iter(lambda: source.read(1024 * 1024), b""):
            digest.update(block)
    return digest.hexdigest()


def verify_apk(path, abi, cfg):
    certificate = os.environ.get("ANDROID_SIGNING_CERT_SHA256", "").replace(":", "").lower()
    if not re.fullmatch(r"[0-9a-f]{64}", certificate):
        raise ValueError("Set the public ANDROID_SIGNING_CERT_SHA256 repository variable")
    signer = command(os.environ["APKSIGNER"], "verify", "--print-certs", str(path))
    digests = re.findall(r"Signer #\d+ certificate SHA-256 digest:\s*([0-9a-fA-F]+)", signer)
    if len(digests) != 1 or digests[0].lower() != certificate:
        raise ValueError("APK signing certificate differs from the configured persistent identity")
    badging = command(os.environ["AAPT"], "dump", "badging", str(path))
    package = re.search(r"package: name='([^']+)' versionCode='(\d+)' versionName='([^']+)'", badging)
    if not package or package.groups() != ("com.papr.papr_mobile", str(cfg["android_build_number"]), cfg["android_version"]):
        raise ValueError("APK package/version does not match the selected release")
    with zipfile.ZipFile(path) as apk:
        libs = [name for name in apk.namelist() if name.startswith("lib/") and name.endswith(".so")]
        if f"lib/{abi}/libpapr_flutter_bridge.so" not in libs or any(name.split("/")[1] != abi for name in libs):
            raise ValueError("APK native ABI/bridge inventory is incorrect")
    return certificate


def collect(cfg, platform):
    if platform not in cfg["platforms"]:
        raise ValueError("Platform was not selected")
    out = Path("release-assets")
    out.mkdir(exist_ok=True)
    files = []
    prefix = f"Papr-{platform}"
    certificate = None
    if platform == "windows":
        sources = list(Path("target/release/bundle/nsis").glob("*-setup.exe"))
        if len(sources) != 1:
            raise ValueError("Expected exactly one NSIS installer")
        with sources[0].open("rb") as installer:
            if installer.read(2) != b"MZ":
                raise ValueError("NSIS output is not a Windows executable")
        files.append((sources[0], f"{prefix}-x64-{cfg['desktop_version']}-{cfg['sha'][:8]}-setup.exe"))
    else:
        for abi in ("arm64-v8a", "armeabi-v7a"):
            source = Path(f"mobile/build/app/outputs/flutter-apk/app-{abi}-release.apk")
            certificate = verify_apk(source, abi, cfg)
            files.append((source, f"{prefix}-{abi}-{cfg['android_version']}-{cfg['android_build_number']}-{cfg['sha'][:8]}.apk"))
    assets = {}
    for source, name in files:
        if not 0 < source.stat().st_size < 2 * 1024**3:
            raise ValueError("Release asset must be non-empty and smaller than 2 GiB")
        shutil.copyfile(source, out / name)
        assets[name] = {"sha256": sha256(out / name), "size": source.stat().st_size}
    info = {**cfg, "platform": platform, "assets": assets, "rust": command("rustc", "--version"),
            "signing_certificate_sha256": certificate}
    if platform == "windows":
        info["node"] = command("node", "--version")
    else:
        info["flutter"] = json.loads(command("flutter", "--version", "--machine"))
    (out / f"{platform}-build-info.json").write_text(json.dumps(info, indent=2) + "\n", encoding="utf-8")


def validate_assets(cfg, folder):
    assets = {}
    builds = []
    expected = set()
    for platform in cfg["platforms"]:
        meta = folder / f"{platform}-build-info.json"
        info = json.loads(meta.read_text(encoding="utf-8"))
        if any(info.get(key) != value for key, value in cfg.items()) or info.get("platform") != platform:
            raise ValueError("Build metadata does not match this tag, SHA, platforms or versions")
        count = 1 if platform == "windows" else 2
        if len(info["assets"]) != count:
            raise ValueError("Selected platform has missing/extra installers")
        for name, evidence in info["assets"].items():
            if Path(name).name != name or "/" in name or "\\" in name:
                raise ValueError("Unsafe asset filename")
            path = folder / name
            if path.stat().st_size != evidence["size"] or sha256(path) != evidence["sha256"]:
                raise ValueError("Installer differs from its build evidence")
            assets[name] = evidence
        expected.update(info["assets"])
        expected.add(meta.name)
        builds.append(info)
    if {path.name for path in folder.iterdir()} != expected:
        raise ValueError("Unexpected/missing files in the publication staging directory")
    return assets, builds


def check_remote_assets(release, files, *, complete):
    remote = {asset["name"]: asset for asset in release["assets"]}
    if not set(remote) <= set(files) or (complete and set(remote) != set(files)):
        raise ValueError("Release asset inventory does not match this build")
    for name, asset in remote.items():
        if asset.get("digest") != f"sha256:{sha256(files[name])}" or asset["size"] != files[name].stat().st_size:
            raise ValueError("Existing asset has different/unverifiable contents; use a new tag")
    return remote


def publish(cfg):
    repo = os.environ["GITHUB_REPOSITORY"]
    folder = Path("release-assets")
    assets, builds = validate_assets(cfg, folder)
    run_url = f"https://github.com/{repo}/actions/runs/{os.environ['GITHUB_RUN_ID']}"
    (folder / "build-info.json").write_text(json.dumps({**cfg, "builds": builds, "run_url": run_url}, indent=2) + "\n", encoding="utf-8")
    files = {path.name: path for path in folder.iterdir()}
    (folder / "SHA256SUMS.txt").write_text("".join(f"{sha256(path)}  {name}\n" for name, path in sorted(files.items())), encoding="utf-8")
    files["SHA256SUMS.txt"] = folder / "SHA256SUMS.txt"
    notes = Path(os.environ["RUNNER_TEMP"]) / "papr-release-notes.md"
    text = f"Built from `{cfg['sha']}`. [Build log]({run_url}).\n\n"
    if "windows" in cfg["platforms"]:
        text += f"Windows: {cfg['desktop_version']}. Download `setup.exe` for Windows x64.\n\n"
    if "android" in cfg["platforms"]:
        text += f"Android: {cfg['android_version']} (versionCode {cfg['android_build_number']}). "
        text += "Choose `arm64-v8a.apk` for a 64-bit ARM phone, or `armeabi-v7a.apk` for a 32-bit ARM phone. "
        text += "Updates require the same signing identity. Back up/sync data before a first signing migration.\n\n"
    if "windows" in cfg["platforms"]:
        text += "Windows installer is not publisher-code-signed.\n\n"
    text += "| Installer | SHA-256 |\n| --- | --- |\n"
    text += "".join(f"| `{name}` | `{value['sha256']}` |\n" for name, value in assets.items())
    notes.write_text(text, encoding="utf-8")
    release = preflight(cfg)
    prerelease = os.environ["PUBLISH_PRERELEASE"] == "true"
    if release is None:
        args = ["gh", "release", "create", cfg["tag"], "--repo", repo, "--target", cfg["sha"],
                "--draft", "--latest=false", "--title", f"Papr {cfg['tag']}", "--notes-file", str(notes)]
        if prerelease:
            args.append("--prerelease")
        command(*args)
        release = preflight(cfg)
        if release is None:
            raise ValueError("Created release draft could not be found")
    remote = check_remote_assets(release, files, complete=False)
    missing = [str(path) for name, path in files.items() if name not in remote]
    if missing:
        command("gh", "release", "upload", cfg["tag"], *missing, "--repo", repo)
    release = preflight(cfg)
    if release is None:
        raise ValueError("Release/tag changed before publication")
    check_remote_assets(release, files, complete=True)
    command("gh", "release", "edit", cfg["tag"], "--repo", repo, "--draft=false", "--latest=false",
            f"--prerelease={str(prerelease).lower()}", "--notes-file", str(notes))
    if tag_commit(repo, cfg["tag"]) != cfg["sha"]:
        raise ValueError("Published tag does not match the verified build commit")
    with open(os.environ["GITHUB_STEP_SUMMARY"], "a", encoding="utf-8") as summary:
        summary.write(f"[Download Papr {cfg['tag']}](https://github.com/{repo}/releases/tag/{cfg['tag']})\n\nCommit: `{cfg['sha']}`\n")


if __name__ == "__main__":
    try:
        cfg = config()
        if sys.argv[1:] == ["prepare"]:
            prepare(cfg)
        elif sys.argv[1:] == ["publish"]:
            publish(cfg)
        elif len(sys.argv) == 3 and sys.argv[1] == "collect":
            collect(cfg, sys.argv[2])
        else:
            raise ValueError("Usage: package_release.py prepare | collect windows/android | publish")
    except (ValueError, KeyError, OSError, zipfile.BadZipFile) as error:
        print(f"::error::{error}", file=sys.stderr)
        sys.exit(1)
