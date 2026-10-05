"""Regression tests for release input and publication safety; no network writes."""

import importlib.util
import json
import os
from pathlib import Path
import subprocess
import tempfile
import unittest
from unittest.mock import patch
import zipfile

SPEC = importlib.util.spec_from_file_location("release", Path(__file__).parents[1] / "package_release.py")
release = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(release)
SHA = "a" * 40


class ReleaseSafety(unittest.TestCase):
    def setUp(self):
        # Release counters can advance without changing these boundary fixtures.
        temporary = tempfile.TemporaryDirectory()
        self.addCleanup(temporary.cleanup)
        previous = Path.cwd()
        os.chdir(temporary.name)
        self.addCleanup(os.chdir, previous)
        Path("src-tauri").mkdir()
        Path("mobile").mkdir()
        Path("src-tauri/tauri.conf.json").write_text('{"version": "0.9.0"}')
        Path("mobile/pubspec.yaml").write_text("version: 0.1.0+1\n")
        self.environment = patch.dict(os.environ, {"RELEASE_TAG": "papr-build-20261006-01", "BUILD_SHA": SHA,
            "PLATFORMS": "both", "ANDROID_BUILD_NUMBER": "2", "GITHUB_REPOSITORY": "example/papr"})
        self.environment.start()
        self.addCleanup(self.environment.stop)

    def test_injection_and_invalid_git_refs_are_rejected(self):
        for tag in ("$(echo unsafe)", "papr-build-a\nsha=bad", "papr-build-a..b", "papr-build-a.lock", "v0.9.0"):
            with self.subTest(tag=tag), patch.dict(os.environ, {"RELEASE_TAG": tag}):
                with self.assertRaises(ValueError):
                    release.config()

    def test_android_requires_explicit_increasing_bounded_code(self):
        for code in ("", "1", "-1", "02", "2100000001", "2\nsha=bad"):
            with self.subTest(code=code), patch.dict(os.environ, {"ANDROID_BUILD_NUMBER": code}):
                with self.assertRaises(ValueError):
                    release.config()
        self.assertEqual(release.config()["android_build_number"], 2)
        with patch.dict(os.environ, {"PLATFORMS": "windows", "ANDROID_BUILD_NUMBER": ""}):
            self.assertIsNone(release.config()["android_build_number"])

    def test_tag_conflict_and_published_release_block_preflight(self):
        cfg = release.config()
        with patch.object(release, "tag_commit", return_value="b" * 40), patch.object(release, "api") as api:
            with self.assertRaisesRegex(ValueError, "different commit"):
                release.preflight(cfg)
            api.assert_not_called()
        with patch.object(release, "tag_commit", return_value=SHA), patch.object(release, "api", return_value={"draft": False}):
            with self.assertRaisesRegex(ValueError, "already published"):
                release.preflight(cfg)

    def test_api_failure_is_not_mistaken_for_missing_release(self):
        for error in ("Forbidden (HTTP 403)", "Bad Gateway (HTTP 502)"):
            result = subprocess.CompletedProcess([], 1, "", error)
            with patch.object(subprocess, "run", return_value=result):
                with self.assertRaises(ValueError):
                    release.api("example", optional=True)
        result = subprocess.CompletedProcess([], 1, "", "Not Found (HTTP 404)")
        with patch.object(subprocess, "run", return_value=result):
            self.assertIsNone(release.api("example", optional=True))

    def test_draft_lookup_paginates_and_checks_target_without_a_tag(self):
        cfg = release.config()
        draft = {"tag_name": cfg["tag"], "draft": True, "target_commitish": SHA, "assets": []}
        page = [{"tag_name": f"other-{index}"} for index in range(100)]
        with patch.object(release, "tag_commit", return_value=None), \
             patch.object(release, "api", side_effect=[None, page, [draft]]) as api:
            self.assertEqual(release.preflight(cfg), draft)
            self.assertTrue(api.call_args.args[0].endswith("page=2"))
        with patch.object(release, "tag_commit", return_value=None), \
             patch.object(release, "api", side_effect=[None, [{**draft, "target_commitish": "b" * 40}]]):
            with self.assertRaisesRegex(ValueError, "draft targets a different commit"):
                release.preflight(cfg)

    def test_annotated_tag_is_peeled_to_its_actual_commit(self):
        with patch.object(release, "api", side_effect=[{"object": {"type": "tag", "sha": "b" * 40}},
                                                     {"object": {"type": "commit", "sha": SHA}}]):
            self.assertEqual(release.tag_commit("example/papr", "papr-build-test"), SHA)

    def test_partial_builds_and_corrupted_installer_cannot_publish(self):
        cfg = release.config()
        with tempfile.TemporaryDirectory() as temporary:
            folder = Path(temporary)
            exe = folder / "Papr-setup.exe"
            exe.write_bytes(b"MZoriginal installer")
            info = {**cfg, "platform": "windows", "assets": {exe.name: {"sha256": release.sha256(exe), "size": exe.stat().st_size}}}
            (folder / "windows-build-info.json").write_text(json.dumps(info))
            with self.assertRaises(FileNotFoundError):
                release.validate_assets(cfg, folder)
            windows_cfg = {**cfg, "platforms": ["windows"], "android_build_number": None}
            info.update(windows_cfg)
            (folder / "windows-build-info.json").write_text(json.dumps(info))
            self.assertEqual(len(release.validate_assets(windows_cfg, folder)[0]), 1)
            exe.write_bytes(b"MZmodified installer")
            with self.assertRaisesRegex(ValueError, "differs"):
                release.validate_assets(windows_cfg, folder)

    def test_draft_retry_never_clobbers_conflicting_assets(self):
        with tempfile.TemporaryDirectory() as temporary:
            file = Path(temporary) / "Papr.apk"
            file.write_bytes(b"APK")
            correct = {"name": file.name, "size": 3, "digest": "sha256:" + release.sha256(file)}
            self.assertEqual(len(release.check_remote_assets({"assets": [correct]}, {file.name: file}, complete=True)), 1)
            for asset in ({**correct, "digest": "sha256:" + "0" * 64}, {**correct, "digest": None}, {**correct, "name": "unknown.apk"}):
                with self.subTest(asset=asset), self.assertRaises(ValueError):
                    release.check_remote_assets({"assets": [asset]}, {file.name: file}, complete=False)
            with self.assertRaises(ValueError):
                release.check_remote_assets({"assets": []}, {file.name: file}, complete=True)

    def test_apk_wrong_certificate_version_or_abi_blocks_collection(self):
        cfg = release.config()
        fingerprint = "c" * 64
        env = {"ANDROID_SIGNING_CERT_SHA256": fingerprint, "APKSIGNER": "signer", "AAPT": "aapt"}
        signer = f"Signer #1 certificate SHA-256 digest: {fingerprint}"
        badging = "package: name='com.papr.papr_mobile' versionCode='2' versionName='0.1.0'"
        with tempfile.TemporaryDirectory() as temporary, patch.dict(os.environ, env):
            apk = Path(temporary) / "test.apk"
            with zipfile.ZipFile(apk, "w") as archive:
                archive.writestr("lib/arm64-v8a/libpapr_flutter_bridge.so", b"fixture")
            with patch.object(release, "command", side_effect=[signer, badging]):
                self.assertEqual(release.verify_apk(apk, "arm64-v8a", cfg), fingerprint)
            with patch.object(release, "command", return_value=signer.replace(fingerprint, "d" * 64)):
                with self.assertRaisesRegex(ValueError, "certificate"):
                    release.verify_apk(apk, "arm64-v8a", cfg)
            with patch.object(release, "command", side_effect=[signer, badging.replace("versionCode='2'", "versionCode='1'")]):
                with self.assertRaisesRegex(ValueError, "version"):
                    release.verify_apk(apk, "arm64-v8a", cfg)
            with patch.object(release, "command", side_effect=[signer, badging]):
                with self.assertRaisesRegex(ValueError, "ABI"):
                    release.verify_apk(apk, "armeabi-v7a", cfg)

    def test_publish_uploads_complete_assets_before_leaving_draft(self):
        with patch.dict(os.environ, {"PLATFORMS": "windows", "ANDROID_BUILD_NUMBER": ""}):
            cfg = release.config()
        previous = Path.cwd()
        with tempfile.TemporaryDirectory() as temporary:
            folder = Path(temporary) / "release-assets"
            folder.mkdir()
            exe = folder / "Papr-setup.exe"
            exe.write_bytes(b"MZfixture")
            (folder / "windows-build-info.json").write_text(json.dumps({**cfg, "platform": "windows",
                "assets": {exe.name: {"size": exe.stat().st_size, "sha256": release.sha256(exe)}}}))
            remote = None
            writes = []

            def fake_command(*args):
                nonlocal remote
                writes.append(args)
                if args[2] == "create":
                    remote = {"tag_name": cfg["tag"], "target_commitish": SHA, "draft": True, "assets": []}
                elif args[2] == "upload":
                    for name, path in [(p.name, p) for p in folder.iterdir()]:
                        remote["assets"].append({"name": name, "size": path.stat().st_size, "digest": "sha256:" + release.sha256(path)})
                elif args[2] == "edit":
                    self.assertEqual(len(remote["assets"]), 4)
                    remote["draft"] = False
                return ""

            def fake_api(path, **kwargs):
                if "/releases/tags/" in path:
                    # Match GitHub: the by-tag endpoint cannot find a draft.
                    return remote if remote and not remote["draft"] else None
                return [remote] if remote else []

            env = {"RUNNER_TEMP": temporary, "GITHUB_RUN_ID": "123", "PUBLISH_PRERELEASE": "true",
                   "GITHUB_STEP_SUMMARY": str(Path(temporary) / "summary.md")}
            try:
                os.chdir(temporary)
                with patch.dict(os.environ, env), patch.object(release, "api", side_effect=fake_api), \
                     patch.object(release, "tag_commit", side_effect=lambda *args: SHA if remote and not remote["draft"] else None), \
                     patch.object(release, "command", side_effect=fake_command):
                    release.publish(cfg)
            finally:
                os.chdir(previous)
            self.assertEqual([args[2] for args in writes], ["create", "upload", "edit"])
            self.assertIn("--draft", writes[0])
            self.assertIn("--draft=false", writes[-1])
            self.assertFalse(any("--clobber" in args for args in writes))
            self.assertFalse(remote["draft"])


if __name__ == "__main__":
    unittest.main()
