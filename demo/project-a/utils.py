#!/usr/bin/env python3
"""Shared helpers used by the calculator demos."""


def format_result(value: float, precision: int = 2) -> str:
    return f"{value:.{precision}f}"


def clamp(value: float, low: float, high: float) -> float:
    return max(low, min(high, value))
