# UI screenshot comparison

Use `scripts/ui_screenshot_test.py` to compare Sublemon with the supplied target image and produce a heatmap of visual differences.

## Compare an existing screenshot

```sh
python3 scripts/ui_screenshot_test.py \
  --reference sublime_goal.png \
  --actual sublemon_current.png
```

## Capture the current screen

Install Pillow for Python and `grim` for Wayland capture, place Sublemon at the same size and screen position as the reference, then pass the app's client-area rectangle:

```sh
python3 scripts/ui_screenshot_test.py \
  --capture \
  --geometry 0,0,941,1038 \
  --max-diff-ratio 0.20
```

`--geometry` is `x,y,width,height` in screen pixels. The capture must include only the same app area as the reference. Captures and the generated `difference.png` are written under `/tmp/sublemon-ui-screenshot-test/` by default.

The test counts a pixel as different when any RGB channel differs by more than 16 levels (the `--pixel-tolerance` default). It reports similarity as the percentage of pixels within tolerance, and passes at 80% similarity by default (`--max-diff-ratio 0.20`). Adjust the tolerance only to account for known display-rendering differences; keep screenshots at identical dimensions for meaningful results. The command exits 0 on pass, 1 on a visual mismatch, and 2 on capture or image errors.

## Current baseline

The provided `sublemon_current.png` currently scores 41.28% pixel similarity to `sublime_goal.png` at tolerance 16, below the 80% goal. The heatmap identifies layout, panel, and content differences to address as the workspace UI is implemented.
