#!/usr/bin/env python3
"""Capture Sublemon and compare it with a supplied reference screenshot.

Dependencies: Python 3, Pillow, and (for live capture) grim.

Examples:
  python3 scripts/ui_screenshot_test.py
  python3 scripts/ui_screenshot_test.py --capture --geometry 0,0,941,1038
  python3 scripts/ui_screenshot_test.py --actual /tmp/sublemon.png --max-diff-ratio 0.08
"""

from __future__ import annotations

import argparse
import shutil
import subprocess
import sys
from pathlib import Path

from PIL import Image, ImageChops, ImageColor, ImageStat

ROOT = Path(__file__).resolve().parents[1]


def parse_geometry(value: str) -> tuple[int, int, int, int]:
    try:
        x, y, width, height = (int(part.strip()) for part in value.split(","))
    except (TypeError, ValueError) as error:
        raise argparse.ArgumentTypeError("geometry must be x,y,width,height") from error
    if x < 0 or y < 0 or width <= 0 or height <= 0:
        raise argparse.ArgumentTypeError("geometry needs nonnegative position and positive size")
    return x, y, width, height


def capture_screen(destination: Path, geometry: tuple[int, int, int, int] | None) -> None:
    grim = shutil.which("grim")
    if grim is None:
        raise RuntimeError("live capture requires grim; pass --actual to compare an existing screenshot")
    command = [grim]
    if geometry:
        x, y, width, height = geometry
        command.extend(["-g", f"{x},{y} {width}x{height}"])
    command.append(str(destination))
    subprocess.run(command, check=True)


def save_diff(reference: Image.Image, actual: Image.Image, destination: Path) -> None:
    difference = ImageChops.difference(reference, actual)
    maximum_channel = ImageChops.lighter(
        ImageChops.lighter(*difference.split()[:2]), difference.split()[2]
    )
    amplified = Image.eval(maximum_channel, lambda value: min(255, value * 4))
    heatmap = ImageColor.getrgb("#ff3b5c")
    diff_image = Image.merge(
        "RGB",
        tuple(Image.eval(amplified, lambda value, channel=channel: value * channel // 255)
              for channel in heatmap),
    )
    destination.parent.mkdir(parents=True, exist_ok=True)
    diff_image.save(destination)


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--reference", type=Path, default=ROOT / "sublime_goal.png",
                        help="target screenshot (default: sublime_goal.png)")
    parser.add_argument("--actual", type=Path, default=ROOT / "sublemon_current.png",
                        help="existing app screenshot (default: sublemon_current.png)")
    parser.add_argument("--capture", action="store_true",
                        help="capture the screen with grim instead of reading --actual")
    parser.add_argument("--geometry", type=parse_geometry,
                        help="capture rectangle as x,y,width,height")
    parser.add_argument("--output-dir", type=Path, default=Path("/tmp/sublemon-ui-screenshot-test"),
                        help="directory for captured screenshot and difference image")
    parser.add_argument("--pixel-tolerance", type=int, default=16,
                        help="per-channel tolerance before a pixel counts as different (default: 16)")
    parser.add_argument("--max-diff-ratio", type=float, default=0.20,
                        help="maximum fraction of differing pixels for a pass (default: 0.20 = 80%% similarity)")
    parser.add_argument("--scale-actual", action="store_true",
                        help="resize actual image to reference dimensions before comparison")
    args = parser.parse_args()

    if not 0 <= args.pixel_tolerance <= 255:
        parser.error("--pixel-tolerance must be between 0 and 255")
    if not 0.0 <= args.max_diff_ratio <= 1.0:
        parser.error("--max-diff-ratio must be between 0 and 1")

    actual_path = args.output_dir / "actual.png" if args.capture else args.actual
    diff_path = args.output_dir / "difference.png"
    try:
        if args.capture:
            args.output_dir.mkdir(parents=True, exist_ok=True)
            capture_screen(actual_path, args.geometry)

        with Image.open(args.reference) as image:
            reference = image.convert("RGB")
        with Image.open(actual_path) as image:
            actual = image.convert("RGB")

        if actual.size != reference.size:
            if not args.scale_actual:
                print(
                    f"FAIL: screenshot sizes differ: reference={reference.size}, actual={actual.size}. "
                    "Use --geometry to capture the matching region or --scale-actual.",
                    file=sys.stderr,
                )
                return 1
            actual = actual.resize(reference.size, Image.Resampling.LANCZOS)

        difference = ImageChops.difference(reference, actual)
        max_channel = ImageChops.lighter(
            ImageChops.lighter(difference.getchannel("R"), difference.getchannel("G")),
            difference.getchannel("B"),
        )
        over_tolerance = max_channel.point(
            lambda value: 255 if value > args.pixel_tolerance else 0
        )
        different_pixels = over_tolerance.histogram()[255]
        total_pixels = reference.width * reference.height
        diff_ratio = different_pixels / total_pixels
        similarity = 1.0 - diff_ratio
        mean_absolute_error = sum(ImageStat.Stat(difference).mean) / 3

        save_diff(reference, actual, diff_path)
        passed = diff_ratio <= args.max_diff_ratio
        print(f"{'PASS' if passed else 'FAIL'}: pixel similarity {similarity:.2%} "
              f"({different_pixels:,}/{total_pixels:,} differ; tolerance={args.pixel_tolerance}/255)")
        print(f"Mean absolute channel error: {mean_absolute_error:.2f}/255")
        print(f"Actual screenshot: {actual_path}")
        print(f"Difference heatmap: {diff_path}")
        return 0 if passed else 1
    except (OSError, subprocess.CalledProcessError, RuntimeError) as error:
        print(f"Screenshot test error: {error}", file=sys.stderr)
        return 2


if __name__ == "__main__":
    raise SystemExit(main())
