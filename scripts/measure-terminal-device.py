#!/usr/bin/env python3
"""Capture the terminal's finite 24s render probe on an unlocked OHOS device."""

import argparse
from collections import defaultdict
import hashlib
import json
from pathlib import Path
import re
import subprocess
import time


class TerminalDeviceCapture:
    def __init__(self, device: str, output: Path):
        self.device = device
        self.output = output.resolve()
        self.output.mkdir(parents=True, exist_ok=True)

    def hdc(self, *arguments: str) -> str:
        result = subprocess.run(
            ["hdc", "-t", self.device, *arguments],
            check=True,
            capture_output=True,
            text=True,
            timeout=45,
        )
        output = result.stdout + result.stderr
        if "Connect server failed" in output or "[Fail]" in output:
            raise RuntimeError(output)
        return output

    def shell(self, *arguments: str) -> str:
        return self.hdc("shell", *arguments)

    def layout(self, name: str) -> dict:
        remote = f"/data/local/tmp/arkit-{name}.json"
        self.shell("uitest", "dumpLayout", "-p", remote)
        target = self.output / f"{name}.json"
        self.hdc("file", "recv", remote, str(target))
        return json.loads(target.read_text())

    @staticmethod
    def nodes(node: dict):
        yield node.get("attributes", {})
        for child in node.get("children", []):
            yield from TerminalDeviceCapture.nodes(child)

    def run(self) -> None:
        layout = self.layout("before")
        button = next(
            (
                node for node in self.nodes(layout)
                if node.get("text") == "120Hz probe" and node.get("type") == "Button"
            ),
            None,
        )
        if button is None:
            raise RuntimeError("Open the terminal example with its 120Hz probe before capture")
        left, top, right, bottom = map(int, re.findall(r"\d+", button["bounds"]))
        pid = self.shell("pidof", "com.arkit.example").strip()
        tree = self.shell("hidumper", "-s", "10", "-a", "RSTree")
        (self.output / "rs-tree.txt").write_text(tree)
        names = sorted(set(re.findall(r"Name \[([^\]]+)\]", tree)))
        print(
            json.dumps({"pid": pid, "surface_names": names}, ensure_ascii=False),
            flush=True,
        )
        metadata = {
            "device": self.device,
            "pid": pid,
            "model": self.shell("param", "get", "const.product.model").strip(),
            "os": self.shell("param", "get", "const.product.software.version").strip(),
            "before_status": [
                node["text"] for node in self.nodes(layout)
                if node.get("text", "").startswith("title:")
            ],
        }
        library = (
            Path(__file__).resolve().parents[1]
            / "examples/demos/dist/arm64-v8a/libdemos.so"
        )
        with library.open("rb") as binary:
            metadata["native_library_sha256"] = hashlib.file_digest(binary, "sha256").hexdigest()
        trace = f"/data/local/tmp/arkit-{self.output.name}.trace"
        begin = self.shell("hitrace", "--trace_begin", "-b", "131072", "graphic", "window")
        print(begin, flush=True)
        if "OpenRecording done" not in begin:
            raise RuntimeError("hitrace did not start a new recording")
        try:
            started = time.monotonic()
            self.shell(
                "uitest", "uiInput", "click",
                str((left + right) // 2), str((top + bottom) // 2),
            )
            for at in [4, 12, 20]:
                time.sleep(max(0, started + at - time.monotonic()))
                screen = self.shell("hidumper", "-s", "10", "-a", "screen")
                (self.output / f"screen-{at}s.txt").write_text(screen)
                print(f"screen sample at {at}s", flush=True)
            time.sleep(max(0, started + 26 - time.monotonic()))
        finally:
            print(self.shell("hitrace", "--trace_finish", "-o", trace), flush=True)
        self.hdc("file", "recv", trace, str(self.output / "frames.trace"))
        self.layout("after")
        remote_image = "/data/local/tmp/arkit-render-probe.jpeg"
        self.shell("snapshot_display", "-f", remote_image)
        self.hdc("file", "recv", remote_image, str(self.output / "screen.jpeg"))
        (self.output / "metadata.json").write_text(
            json.dumps(metadata, ensure_ascii=False, indent=2) + "\n"
        )
        print(f"capture saved: {self.output}", flush=True)


class FrameTraceAnalysis:
    def __init__(self, directory: Path):
        self.directory = directory

    @staticmethod
    def stats(timestamps: list[float]) -> dict:
        if len(timestamps) < 2:
            return {"count": len(timestamps), "fps": 0}
        intervals = sorted((b - a) * 1000 for a, b in zip(timestamps, timestamps[1:]))

        def percentile(fraction: float) -> float:
            at = (len(intervals) - 1) * fraction
            lower = int(at)
            upper = min(lower + 1, len(intervals) - 1)
            return intervals[lower] + (intervals[upper] - intervals[lower]) * (at - lower)

        return {
            "count": len(timestamps),
            "fps": (len(timestamps) - 1) / (timestamps[-1] - timestamps[0]),
            "interval_ms": {
                "p50": percentile(0.50),
                "p95": percentile(0.95),
                "p99": percentile(0.99),
                "max": intervals[-1],
                "over_12_5_ms": sum(value > 12.5 for value in intervals),
            },
        }

    def run(self) -> dict:
        metadata = json.loads((self.directory / "metadata.json").read_text())
        pid = metadata["pid"]
        tree = (self.directory / "rs-tree.txt").read_text()
        # RSNodeId embeds its creating PID in the upper 32 bits. Resolve the
        # app's queue IDs before counting server-side FlushBuffer IPC events.
        owned_queues = {
            queue
            for node, queue in re.findall(
                r"SURFACE_NODE\[(\d+)\].*?uniqueId\[(\d+)\]", tree
            )
            if int(node) >> 32 == int(pid)
        }
        submissions = defaultdict(list)
        callbacks = []
        client_flushes = []
        expression = re.compile(r" ([\d.]+): tracing_mark_write: B\|(\d+)\|H:(.*)")
        with (self.directory / "frames.trace").open() as source:
            for line in source:
                match = expression.search(line)
                if not match:
                    continue
                timestamp, actor, event = float(match[1]), match[2], match[3]
                surface = re.match(r"FlushBuffer name: (.*?) queueId: (\d+) ", event)
                if surface and surface[2] in owned_queues:
                    submissions[(surface[1], surface[2])].append(timestamp)
                if actor == pid and event.startswith("ReceiveVsync name:arkit-terminal "):
                    callbacks.append(timestamp)
                if (
                    actor == pid
                    and event.startswith("NativeWindowFlushBuffer")
                    and "arkit-terminal-" in line
                ):
                    client_flushes.append(timestamp)
        result = {
            "metadata": metadata,
            "source": str(self.directory / "frames.trace"),
            "note": (
                "FlushBuffer counts app buffer submissions, "
                "not GPU completion or panel scanout."
            ),
            "queues": {str(key): len(values) for key, values in submissions.items()},
        }
        if not submissions:
            result["error"] = "No application buffer submissions in capture"
        else:
            queue, timestamps = max(submissions.items(), key=lambda item: len(item[1]))
            # Exclude any sparse cursor-blink frames before the probe begins.
            start = next(
                (
                    timestamps[i] for i in range(len(timestamps) - 10)
                    if timestamps[i + 10] - timestamps[i] < 0.3
                ),
                timestamps[0],
            )
            result["surface"] = {"name": queue[0], "queue_id": queue[1]}
            result["continuous_submission_start_s"] = start
            result["phase_windows_relative_to_start_s"] = [[2, 7.5], [10, 15.5], [18, 23.5]]
            result["phases"] = {}
            for phase, offset in zip(["ASCII scroll", "CJK scroll", "styled redraw"], [0, 8, 16]):
                window = (start + offset + 2, start + offset + 7.5)
                result["phases"][phase] = self.stats(
                    [t for t in timestamps if window[0] <= t <= window[1]]
                )
            result["native_callbacks"] = self.stats(
                [t for t in callbacks if start + 2 <= t <= start + 23.5]
            )
            result["active_submissions"] = self.stats(
                [t for t in timestamps if start + 2 <= t <= start + 23.5]
            )
            result["client_native_window_flush_calls"] = self.stats(
                [t for t in client_flushes if start + 2 <= t <= start + 23.5]
            )
        result["screen_modes_hz"] = [
            int(re.search(r"activeMode:.*refreshRate=(\d+)", path.read_text())[1])
            for path in sorted(self.directory.glob("screen-*s.txt"))
        ]
        with (self.directory / "frames.trace").open("rb") as trace:
            result["trace_sha256"] = hashlib.file_digest(trace, "sha256").hexdigest()
        (self.directory / "summary.json").write_text(
            json.dumps(result, ensure_ascii=False, indent=2) + "\n"
        )
        return result


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--device")
    parser.add_argument("--output-dir", type=Path, required=True)
    parser.add_argument("--analyze-only", action="store_true")
    args = parser.parse_args()
    if not args.analyze_only:
        if not args.device:
            parser.error("--device is required for capture")
        TerminalDeviceCapture(args.device, args.output_dir).run()
    print(json.dumps(FrameTraceAnalysis(args.output_dir).run(), ensure_ascii=False, indent=2))


if __name__ == "__main__":
    main()
