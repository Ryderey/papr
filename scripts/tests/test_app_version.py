"""Version maintenance tests never touch the checkout or external accounts."""
import json
import os
from pathlib import Path
import sys
import tempfile
import unittest
from unittest.mock import patch

sys.path.insert(0, str(Path(__file__).parents[1]))
import app_version as version


def seed_versions(product="0.9.0", mobile="0.9.0+1", desktop_inherits=True):
    Path("Cargo.toml").write_text(f'[workspace.package]\nversion = "{product}"\n')
    for path in version.RUST_MANIFESTS:
        Path(path).parent.mkdir(parents=True, exist_ok=True)
        field = 'version.workspace = true' if desktop_inherits or path != "src-tauri/Cargo.toml" else f'version = "{product}"'
        Path(path).write_text(f'[package]\nname = "test"\n{field}\n')
    Path("package.json").write_text(json.dumps({"version": product}))
    Path("src-tauri/tauri.conf.json").write_text(json.dumps({"version": product}))
    Path("mobile").mkdir(exist_ok=True)
    Path("mobile/pubspec.yaml").write_text(f'version: {mobile}\n')
    Path("Cargo.lock").write_text('version = 4\n' + ''.join(
        f'\n[[package]]\nname = "{name}"\nversion = "{product}"\n'
        for name in sorted(version.LOCAL_PACKAGES)) +
        '\n[[package]]\nname = "serde"\nversion = "1.0.0"\nsource = "registry+example"\n')


class VersionMaintenance(unittest.TestCase):
    def setUp(self):
        temporary = tempfile.TemporaryDirectory()
        self.addCleanup(temporary.cleanup)
        previous = Path.cwd()
        os.chdir(temporary.name)
        self.addCleanup(os.chdir, previous)
        seed_versions(mobile="0.1.0+1", desktop_inherits=False)

    def snapshot(self):
        return {path: Path(path).read_bytes() for path in version.FILES}

    def test_normalization_preserves_dependencies_and_line_endings(self):
        for path in version.FILES:
            Path(path).write_bytes(Path(path).read_bytes().replace(b"\r\n", b"\n").replace(b"\n", b"\r\n"))
        version.set_version("0.9.1", 5)
        self.assertEqual(version.check(), {"version": "0.9.1", "android_build_number": 5})
        self.assertIn('name = "serde"\r\nversion = "1.0.0"', Path("Cargo.lock").read_bytes().decode())
        for data in self.snapshot().values():
            self.assertNotIn(b"\n", data.replace(b"\r\n", b""))
        before = self.snapshot()
        version.set_version("0.9.1", 5)
        self.assertEqual(self.snapshot(), before)

    def test_invalid_input_and_format_do_not_write(self):
        before = self.snapshot()
        for value, code in [("0.9.1\nunsafe", 5), ("0.09.1", 5), ("0.8.0", 5), ("0.9.1", "05"), ("0.9.1", 2100000001)]:
            with self.subTest(value=value, code=code), self.assertRaises(ValueError):
                version.set_version(value, code)
            self.assertEqual(self.snapshot(), before)
        Path("package.json").write_text('{"version":"0.9.0","nested":{"version":"12.0.0"}}')
        before = self.snapshot()
        with self.assertRaises(ValueError):
            version.set_version("0.9.1", 5)
        self.assertEqual(self.snapshot(), before)

    def test_version_drift_and_source_code_decrease_are_rejected(self):
        version.set_version("0.9.1", 5)
        with self.assertRaises(ValueError):
            version.set_version("0.9.2", 4)
        for path in ("package.json", "src-tauri/tauri.conf.json", "Cargo.lock", "mobile/pubspec.yaml"):
            before = Path(path).read_bytes()
            Path(path).write_bytes(before.replace(b"0.9.1", b"0.9.2"))
            with self.subTest(path=path), self.assertRaisesRegex(ValueError, "drift"):
                version.check()
            Path(path).write_bytes(before)

    def test_write_failure_restores_all_manifests(self):
        before = self.snapshot()
        write = Path.write_bytes
        failed = False
        def fail_once(path, data):
            nonlocal failed
            if path.as_posix() == "src-tauri/Cargo.toml" and not failed:
                failed = True
                write(path, b"partial write")
                raise OSError("fixture write failure")
            return write(path, data)
        with patch.object(Path, "write_bytes", fail_once), self.assertRaises(OSError):
            version.set_version("0.9.1", 5)
        self.assertEqual(self.snapshot(), before)

    def test_permanent_write_denial_does_not_prevent_restoring_other_files(self):
        before = self.snapshot()
        write = Path.write_bytes
        def denied(path, data):
            if path.as_posix() == "src-tauri/Cargo.toml":
                raise PermissionError("fixture denied")
            return write(path, data)
        with patch.object(Path, "write_bytes", denied), self.assertRaises(PermissionError):
            version.set_version("0.9.1", 5)
        self.assertEqual(self.snapshot(), before)


if __name__ == "__main__":
    unittest.main()
