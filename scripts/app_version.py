"""Maintain Papr's shared product version using only the Python standard library."""

import argparse
import json
from pathlib import Path
import re
try:
    import tomllib
except ModuleNotFoundError:
    raise SystemExit("Version maintenance requires Python 3.11 or newer (CI uses 3.12).")


VERSION = r"(?:0|[1-9][0-9]*)\.(?:0|[1-9][0-9]*)\.(?:0|[1-9][0-9]*)"
MAX_ANDROID_CODE = 2100000000
RUST_MANIFESTS = (
    "src-tauri/Cargo.toml", "crates/papr-core/Cargo.toml",
    "crates/papr-flutter-bridge/Cargo.toml",
)
FILES = ("Cargo.toml", *RUST_MANIFESTS, "package.json", "src-tauri/tauri.conf.json",
         "mobile/pubspec.yaml", "Cargo.lock")
LOCAL_PACKAGES = {"papr", "papr-core", "papr-flutter-bridge"}


def android_code(value):
    if not re.fullmatch(r"[1-9][0-9]{0,9}", str(value)) or int(value) > MAX_ANDROID_CODE:
        raise ValueError("Android build number must be a positive integer <= 2100000000")
    return int(value)


def _check(texts, consistent=True):
    version = tomllib.loads(texts["Cargo.toml"])["workspace"]["package"]["version"]
    versions = [version, json.loads(texts["package.json"])["version"],
                json.loads(texts["src-tauri/tauri.conf.json"])["version"]]
    mobile = re.search(rf"^version:\s*({VERSION})\+([1-9][0-9]*)\s*$",
                       texts["mobile/pubspec.yaml"], re.M)
    if not mobile:
        raise ValueError("pubspec must declare a product version and positive build number")
    versions.append(mobile[1])
    code = android_code(mobile[2])
    if any(not isinstance(v, str) or not re.fullmatch(VERSION, v) for v in versions):
        raise ValueError("Product version must use MAJOR.MINOR.PATCH without leading zeros")
    for path in RUST_MANIFESTS:
        field = tomllib.loads(texts[path])["package"]["version"]
        if not isinstance(field, dict) or len(field) != 1 or field.get("workspace") is not True:
            if consistent or not isinstance(field, str) or not re.fullmatch(VERSION, field):
                raise ValueError(f"{path} must inherit workspace version")
            versions.append(field)
    packages = [p for p in tomllib.loads(texts["Cargo.lock"])["package"]
                if p["name"] in LOCAL_PACKAGES and "source" not in p]
    if len(packages) != 3 or {p["name"] for p in packages} != LOCAL_PACKAGES:
        raise ValueError("Cargo.lock must contain all three local packages")
    if consistent and (any(v != version for v in versions)
                       or any(p["version"] != version for p in packages)):
        raise ValueError("Product version drift detected; run scripts/app_version.py set")
    return {"version": version, "android_build_number": code}


def _read():
    # Read bytes so maintenance preserves each file's original line endings.
    return {path: Path(path).read_bytes().decode("utf-8") for path in FILES}


def check():
    return _check(_read())


def _replace(text, pattern, replacement, count=1):
    result, matched = re.subn(pattern, replacement, text, flags=re.M)
    if matched != count:
        raise ValueError("Unexpected manifest format; refusing a partial version update")
    return result


def set_version(version, code):
    if not re.fullmatch(VERSION, version):
        raise ValueError("Product version must use MAJOR.MINOR.PATCH without leading zeros")
    code = android_code(code)
    original = _read()
    previous = _check(original, consistent=False)
    if tuple(map(int, version.split("."))) < tuple(map(int, previous["version"].split("."))):
        raise ValueError("Product version cannot decrease")
    if code < previous["android_build_number"]:
        raise ValueError("Android source build number cannot decrease")
    updated = original.copy()
    updated["Cargo.toml"] = _replace(original["Cargo.toml"], r'^version\s*=\s*"[^"\r\n]+"',
                                      f'version = "{version}"')
    for path in RUST_MANIFESTS:
        updated[path] = _replace(original[path], r"^version(?:\.workspace)?\s*=[^\r\n]+",
                                 "version.workspace = true")
    for path in ("package.json", "src-tauri/tauri.conf.json"):
        updated[path] = _replace(original[path], r'("version"\s*:\s*)"[^"\r\n]+"',
                                 lambda match: f'{match[1]}"{version}"')
    updated["mobile/pubspec.yaml"] = _replace(original["mobile/pubspec.yaml"],
                                              r"^version:[^\r\n]+", f"version: {version}+{code}")
    updated["Cargo.lock"] = _replace(original["Cargo.lock"],
        r'(\[\[package\]\]\r?\nname = "(?:papr|papr-core|papr-flutter-bridge)"\r?\nversion = )"[^"\r\n]+"',
        lambda match: f'{match[1]}"{version}"', count=3)
    result = _check(updated)
    written = []
    try:
        for path, text in updated.items():
            if text != original[path]:
                written.append(path)
                Path(path).write_bytes(text.encode("utf-8"))
    except OSError as error:
        failed = []
        for path in reversed(written):
            try:
                data = original[path].encode("utf-8")
                if Path(path).read_bytes() != data:
                    Path(path).write_bytes(data)
            except OSError:
                failed.append(path)
        if failed:
            raise OSError(f"Version update failed; could not restore: {', '.join(failed)}") from error
        raise
    return result


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    actions = parser.add_subparsers(dest="action", required=True)
    actions.add_parser("check")
    setter = actions.add_parser("set")
    setter.add_argument("version")
    setter.add_argument("--android-code", required=True)
    args = parser.parse_args()
    try:
        print(json.dumps(check() if args.action == "check" else set_version(args.version, args.android_code)))
    except (ValueError, KeyError, OSError) as error:
        parser.exit(1, f"Version check failed: {error}\n")
