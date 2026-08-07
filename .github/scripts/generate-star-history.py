#!/usr/bin/env python3
"""Render repository-scoped GitHub stargazer data as static SVG charts."""

from __future__ import annotations

import argparse
from collections import Counter
from dataclasses import dataclass
from datetime import date
from html import escape
import json
import math
from pathlib import Path
import sys


WIDTH = 960
HEIGHT = 520
LEFT = 78
RIGHT = 32
TOP = 62
BOTTOM = 72
PLOT_WIDTH = WIDTH - LEFT - RIGHT
PLOT_HEIGHT = HEIGHT - TOP - BOTTOM


@dataclass(frozen=True)
class Theme:
    """Colors used to render one chart variant."""

    background: str
    border: str
    grid: str
    text: str
    muted_text: str
    line: str = "#FF7A1A"


LIGHT = Theme(
    background="#ffffff",
    border="#d0d7de",
    grid="#d8dee4",
    text="#1f2328",
    muted_text="#59636e",
)

DARK = Theme(
    background="#0d1117",
    border="#30363d",
    grid="#30363d",
    text="#f0f6fc",
    muted_text="#9198a1",
)


def parse_args() -> argparse.Namespace:
    """Parse input, output, and repository labels supplied by the workflow."""

    parser = argparse.ArgumentParser()
    parser.add_argument("--input", required=True, type=Path)
    parser.add_argument("--repository", required=True)
    parser.add_argument("--light-output", required=True, type=Path)
    parser.add_argument("--dark-output", required=True, type=Path)
    return parser.parse_args()


def load_star_dates(path: Path) -> list[date]:
    """Load timestamped stargazers from a JSON file or standard input."""

    raw_input = sys.stdin.read() if path == Path("-") else path.read_text(encoding="utf-8")
    payload = json.loads(raw_input)
    if not isinstance(payload, list):
        raise ValueError("stargazer input must be a JSON array")

    entries = []
    for item in payload:
        if isinstance(item, list):
            entries.extend(item)
        else:
            entries.append(item)

    dates = []
    for entry in entries:
        if not isinstance(entry, dict) or not isinstance(entry.get("starred_at"), str):
            raise ValueError("stargazer entries must include a starred_at timestamp")
        dates.append(date.fromisoformat(entry["starred_at"][:10]))

    if not dates:
        raise ValueError("cannot render a star history without stargazers")

    return sorted(dates)


def cumulative_points(star_dates: list[date]) -> list[tuple[date, int]]:
    """Collapse individual timestamps into cumulative daily totals."""

    total = 0
    points = []
    for day, count in sorted(Counter(star_dates).items()):
        total += count
        points.append((day, total))
    return points


def nice_ceiling(value: int) -> int:
    """Round a positive value up to a readable chart-axis ceiling."""

    if value <= 5:
        return 5

    magnitude = 10 ** math.floor(math.log10(value))
    normalized = value / magnitude
    for factor in (1, 1.2, 1.5, 2, 2.5, 3, 4, 5, 6, 8, 10):
        if normalized <= factor:
            return round(factor * magnitude)
    raise AssertionError("unreachable axis ceiling")


def date_ticks(start: date, end: date, count: int = 6) -> list[date]:
    """Return evenly spaced, deduplicated dates for the horizontal axis."""

    span = end.toordinal() - start.toordinal()
    if span == 0:
        return [start]
    return [
        date.fromordinal(round(start.toordinal() + span * index / (count - 1)))
        for index in range(count)
    ]


def text_width(value: str, font_size: int) -> float:
    """Estimate system-font text width so labels stay inside simple SVG renderers."""

    return len(value) * font_size * 0.62


def render_svg(repository: str, points: list[tuple[date, int]], theme: Theme) -> str:
    """Render one deterministic light or dark SVG from cumulative star totals."""

    start = points[0][0]
    end = points[-1][0]
    day_span = max(1, end.toordinal() - start.toordinal())
    star_count = points[-1][1]
    y_max = nice_ceiling(star_count)

    def x_for(day: date) -> float:
        return LEFT + (day.toordinal() - start.toordinal()) / day_span * PLOT_WIDTH

    def y_for(value: int) -> float:
        return TOP + PLOT_HEIGHT - value / y_max * PLOT_HEIGHT

    line_points = " ".join(f"{x_for(day):.1f},{y_for(total):.1f}" for day, total in points)
    first_x = x_for(points[0][0])
    last_x = x_for(points[-1][0])
    baseline = TOP + PLOT_HEIGHT
    area_points = f"{first_x:.1f},{baseline:.1f} {line_points} {last_x:.1f},{baseline:.1f}"

    y_grid = []
    for index in range(6):
        value = round(y_max * index / 5)
        label = str(value)
        y = y_for(value)
        y_grid.append(
            f'  <line x1="{LEFT}" y1="{y:.1f}" x2="{WIDTH - RIGHT}" y2="{y:.1f}" class="grid" />\n'
            f'  <text x="{LEFT - 14 - text_width(label, 12):.1f}" y="{y + 4:.1f}" class="tick">{label}</text>'
        )

    x_grid = []
    for tick in date_ticks(start, end):
        x = x_for(tick)
        label = f"{tick:%Y-%m}"
        x_grid.append(
            f'  <line x1="{x:.1f}" y1="{TOP}" x2="{x:.1f}" y2="{baseline}" class="grid" />\n'
            f'  <text x="{x - text_width(label, 12) / 2:.1f}" y="{baseline + 30}" class="tick">{label}</text>'
        )

    repository_summary = f"{repository} · {star_count:,} stars"
    repository_label = escape(repository_summary)
    repository_x = WIDTH - RIGHT - text_width(repository_summary, 14)
    aria_label = escape(
        f"Star history for {repository}: {star_count} stars from {start.isoformat()} to {end.isoformat()}"
    )

    return f"""<svg xmlns="http://www.w3.org/2000/svg" width="{WIDTH}" height="{HEIGHT}" viewBox="0 0 {WIDTH} {HEIGHT}" role="img" aria-label="{aria_label}">
  <defs>
    <linearGradient id="star-area" x1="0" y1="0" x2="0" y2="1">
      <stop offset="0%" stop-color="{theme.line}" stop-opacity="0.28" />
      <stop offset="100%" stop-color="{theme.line}" stop-opacity="0.03" />
    </linearGradient>
  </defs>
  <style>
    text {{ font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif; }}
    .title {{ fill: {theme.text}; font-size: 20px; font-weight: 600; }}
    .summary {{ fill: {theme.muted_text}; font-size: 14px; }}
    .tick {{ fill: {theme.muted_text}; font-size: 12px; }}
    .grid {{ stroke: {theme.grid}; stroke-width: 1; }}
  </style>
  <rect x="0.5" y="0.5" width="{WIDTH - 1}" height="{HEIGHT - 1}" rx="8" fill="{theme.background}" stroke="{theme.border}" />
  <text x="{LEFT}" y="35" class="title">Star History</text>
  <text x="{repository_x:.1f}" y="35" class="summary">{repository_label}</text>
{chr(10).join(y_grid)}
{chr(10).join(x_grid)}
  <polygon points="{area_points}" fill="url(#star-area)" />
  <polyline points="{line_points}" fill="none" stroke="{theme.line}" stroke-width="3" stroke-linecap="round" stroke-linejoin="round" />
  <circle cx="{last_x:.1f}" cy="{y_for(star_count):.1f}" r="5" fill="{theme.line}" stroke="{theme.background}" stroke-width="2" />
  <text x="{LEFT}" y="{HEIGHT - 20}" class="summary">Source: GitHub repository stargazers API</text>
</svg>
"""


def write_svg(path: Path, content: str) -> None:
    """Create the destination directory and write a UTF-8 SVG file."""

    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(content, encoding="utf-8")


def main() -> None:
    """Generate the light and dark chart assets requested by the workflow."""

    args = parse_args()
    points = cumulative_points(load_star_dates(args.input))
    write_svg(args.light_output, render_svg(args.repository, points, LIGHT))
    write_svg(args.dark_output, render_svg(args.repository, points, DARK))


if __name__ == "__main__":
    main()
