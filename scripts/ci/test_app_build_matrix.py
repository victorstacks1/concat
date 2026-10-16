import unittest

from scripts.ci.app_build_matrix import get_matrix


class AppBuildMatrixTests(unittest.TestCase):
    def test_default_matrix_contains_all_desktop_targets(self):
        targets = get_matrix(False)["include"]
        self.assertEqual(len(targets), 5)
        self.assertEqual(
            [
                (
                    target["platform"],
                    target["arch"],
                    target["os"],
                    target["target"],
                )
                for target in targets
            ],
            [
                ("macos", "arm64", "macos-15", "aarch64-apple-darwin"),
                ("macos", "x86_64", "macos-15-intel", "x86_64-apple-darwin"),
                ("linux", "x86_64", "ubuntu-22.04", "x86_64-unknown-linux-gnu"),
                ("linux", "aarch64", "ubuntu-22.04-arm", "aarch64-unknown-linux-gnu"),
                ("windows", "x86_64", "windows-2022", "x86_64-pc-windows-msvc"),
            ],
        )

    def test_smoke_only_matrix_contains_windows_x86_64(self):
        targets = get_matrix(True)["include"]
        self.assertEqual(len(targets), 1)
        self.assertEqual(
            (targets[0]["platform"], targets[0]["arch"]),
            ("windows", "x86_64"),
        )
        self.assertEqual(targets[0]["os"], "windows-2022")
        self.assertEqual(targets[0]["target"], "x86_64-pc-windows-msvc")


if __name__ == "__main__":
    unittest.main()
