import argparse
import json


TARGETS = [
    {
        "platform": "macos",
        "arch": "arm64",
        "os": "macos-15",
        "target": "aarch64-apple-darwin",
        "sherpa": "osx-arm64",
    },
    {
        "platform": "macos",
        "arch": "x86_64",
        "os": "macos-15-intel",
        "target": "x86_64-apple-darwin",
        "sherpa": "osx-x64",
    },
    {
        "platform": "linux",
        "arch": "x86_64",
        "os": "ubuntu-22.04",
        "target": "x86_64-unknown-linux-gnu",
        "ffmpeg": "linux64",
        "sherpa": "linux-x64",
        "ort": "x64",
        "nfpm": "amd64",
        "appimage": "x86_64",
    },
    {
        "platform": "linux",
        "arch": "aarch64",
        "os": "ubuntu-22.04-arm",
        "target": "aarch64-unknown-linux-gnu",
        "ffmpeg": "linuxarm64",
        "sherpa": "linux-aarch64",
        "ort": "aarch64",
        "nfpm": "arm64",
        "appimage": "aarch64",
    },
    # Windows ARM stays disabled until concat.exe can start on that target (#326).
    {
        "platform": "windows",
        "arch": "x86_64",
        "os": "windows-2022",
        "target": "x86_64-pc-windows-msvc",
        "ffmpeg": "win64",
        "sherpa": "win-x64",
        "ort": "win-x64",
        "llvm": "win64",
        "vc": "x64",
        "inno": "x64compatible",
        "wix": "x64",
    },
]


def get_matrix(windows_smoke_only: bool) -> dict[str, list[dict[str, str]]]:
    targets = [
        target
        for target in TARGETS
        if not windows_smoke_only or target["platform"] == "windows"
    ]
    return {"include": targets}


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--windows-smoke-only", action="store_true")
    args = parser.parse_args()
    print(json.dumps(get_matrix(args.windows_smoke_only), separators=(",", ":")))


if __name__ == "__main__":
    main()
