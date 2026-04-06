#!/usr/bin/env python3

import argparse
import csv
import json
import sys
import urllib.error
import urllib.request
import unicodedata
from typing import Dict, List


def fetch_states(base_url: str, token: str) -> List[Dict]:
    request = urllib.request.Request(
        f"{base_url.rstrip('/')}/api/states",
        headers={
            "Authorization": f"Bearer {token}",
            "Content-Type": "application/json",
        },
    )

    try:
        with urllib.request.urlopen(request, timeout=15) as response:
            return json.load(response)
    except urllib.error.HTTPError as error:
        body = error.read().decode("utf-8", errors="replace")
        raise RuntimeError(f"HTTP {error.code}: {body}") from error
    except urllib.error.URLError as error:
        raise RuntimeError(f"网络请求失败: {error}") from error


def format_value(value: object) -> str:
    if value is None or value == "":
        return "-"
    return str(value)


def truncate(value: str, max_length: int) -> str:
    if len(value) <= max_length:
        return value
    return value[: max_length - 1] + "…"


def display_width(value: str) -> int:
    width = 0
    for char in value:
        width += 2 if unicodedata.east_asian_width(char) in {"W", "F"} else 1
    return width


def truncate_display(value: str, max_width: int) -> str:
    current = 0
    chars = []
    for char in value:
        char_width = 2 if unicodedata.east_asian_width(char) in {"W", "F"} else 1
        if current + char_width > max_width:
            break
        chars.append(char)
        current += char_width

    result = "".join(chars)
    if result == value:
        return result

    ellipsis = "…"
    while result and display_width(result) + display_width(ellipsis) > max_width:
        result = result[:-1]
    return result + ellipsis


def pad_display(value: str, width: int) -> str:
    return value + " " * max(0, width - display_width(value))


def trim_right_padding(value: str) -> str:
    return value.rstrip()


def main() -> int:
    parser = argparse.ArgumentParser(
        description="列出 Home Assistant 中的 climate 实体及关键信息"
    )
    parser.add_argument("--base-url", required=True, help="例如 http://192.168.1.10:8123")
    parser.add_argument("--token", required=True, help="Home Assistant 长期访问令牌")
    parser.add_argument(
        "--csv",
        default="climate_entities.csv",
        help="导出的 CSV 文件路径，默认 climate_entities.csv",
    )
    args = parser.parse_args()

    try:
        states = fetch_states(args.base_url, args.token)
    except RuntimeError as error:
        print(f"请求失败: {error}", file=sys.stderr)
        return 1

    climates = [item for item in states if str(item.get("entity_id", "")).startswith("climate.")]
    climates.sort(key=lambda item: str(item.get("entity_id", "")))

    if not climates:
        print("未找到 climate 实体。")
        return 0

    rows = []

    for item in climates:
        attributes = item.get("attributes", {}) or {}
        area_name = (
            attributes.get("area_name")
            or attributes.get("area")
            or attributes.get("room")
            or "-"
        )

        rows.append(
            {
                "entity_id": format_value(item.get("entity_id")),
                "name": format_value(attributes.get("friendly_name")),
                "area_name": format_value(area_name),
                "state": format_value(item.get("state")),
            }
        )

    entity_width = min(
        max(display_width("实体ID"), *(display_width(row["entity_id"]) for row in rows)),
        34,
    )
    name_width = min(
        max(display_width("名称"), *(display_width(row["name"]) for row in rows)),
        24,
    )
    area_width = min(
        max(display_width("区域名称"), *(display_width(row["area_name"]) for row in rows)),
        14,
    )
    state_width = min(
        max(display_width("状态"), *(display_width(row["state"]) for row in rows)),
        10,
    )

    header = (
        f"{pad_display('实体ID', entity_width)}  "
        f"{pad_display('名称', name_width)}  "
        f"{pad_display('区域名称', area_width)}  "
        f"{pad_display('状态', state_width)}"
    )
    print(header)
    print("-" * display_width(header))

    for row in rows:
        entity_text = pad_display(truncate_display(row["entity_id"], entity_width), entity_width)
        name_text = pad_display(truncate_display(row["name"], name_width), name_width)
        area_text = pad_display(truncate_display(row["area_name"], area_width), area_width)
        state_text = pad_display(truncate_display(row["state"], state_width), state_width)
        line = (
            f"{entity_text}  "
            f"{name_text}  "
            f"{area_text}  "
            f"{state_text}"
        )
        print(trim_right_padding(line))

    try:
        with open(args.csv, "w", newline="", encoding="utf-8-sig") as csv_file:
            writer = csv.DictWriter(
                csv_file,
                fieldnames=["entity_id", "name", "area_name", "state"],
            )
            writer.writeheader()
            writer.writerows(rows)
        print(f"\nCSV 已导出: {args.csv}")
    except OSError as error:
        print(f"\nCSV 导出失败: {error}", file=sys.stderr)
        return 1

    return 0


if __name__ == "__main__":
    raise SystemExit(main())
