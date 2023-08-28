from haversine import inverse_haversine, Direction
import sys
from typing import List, Tuple
import matplotlib.pyplot as plt
from matplotlib.axes import Axes
import numpy as np
import math


COMMIT_OFFSET = 0.1
HOSTILE_OFFSET = 0.2

RED_START_FACTOR = 4.0


argv = sys.argv

lat = float(argv[1])
lon = float(argv[2])
alt = float(argv[3])

width = float(argv[4])
height = float(argv[5])

orientation = argv[6]

LLA = Tuple[float, float]


def plot_llas(ax: Axes, llas: List[LLA], label: str):
    ax.plot([lla[1] for lla in llas], [lla[0] for lla in llas], label=label)
    print("---------------")
    print(f"- { label } -")
    print("---------------")
    for lla in llas:
        print(f" - {{ lat: {lla[0]}_deg, lon: {lla[1]}_deg, alt: {alt}_deg }}")


def primary_secondary(width: float, height: float, orientation: str):
    if orientation in ["north", "south"]:
        return (height, width)
    else:
        return (width, height)


def orientation_to_direction(orientation: str):
    if orientation == "north":
        return Direction.NORTH
    elif orientation == "south":
        return Direction.SOUTH
    elif orientation == "east":
        return Direction.EAST
    elif orientation == "west":
        return Direction.WEST


def opposite_dir(direction: Direction) -> Direction:
    n_s = [Direction.NORTH, Direction.SOUTH]
    e_w = [Direction.EAST, Direction.WEST]

    if direction in n_s:
        return n_s[(n_s.index(direction) + 1) % 2]
    else:
        return e_w[(e_w.index(direction) + 1) % 2]


def build_box(center: LLA, primary: float, secondary: float):
    left = inverse_haversine(center, primary / 2.0, Direction.WEST)

    top_left = inverse_haversine(left, secondary / 2.0, Direction.NORTH)
    bottom_left = inverse_haversine(left, secondary / 2.0, Direction.SOUTH)

    right = inverse_haversine(center, primary / 2.0, Direction.EAST)

    top_right = inverse_haversine(right, secondary / 2.0, Direction.NORTH)
    bottom_right = inverse_haversine(right, secondary / 2.0, Direction.SOUTH)

    return [top_left, bottom_left, bottom_right, top_right, top_left]


def build_sanitize(center: LLA, width: float, height: float, orientation: str):
    (primary, secondary) = primary_secondary(width, height, orientation)

    return build_box(center, primary, secondary)


def build_commit(center: LLA, width: float, height: float, orientation: str):
    direction = orientation_to_direction(orientation)
    opposite = opposite_dir(direction)

    (primary, secondary) = primary_secondary(width, height, orientation)

    center = inverse_haversine(center, secondary * COMMIT_OFFSET, opposite)

    return build_box(center, primary, secondary * (1 - (COMMIT_OFFSET * 2)))


def build_hostile(center: LLA, width: float, height: float, orientation: str):
    direction = orientation_to_direction(orientation)
    opposite = opposite_dir(direction)

    (primary, secondary) = primary_secondary(width, height, orientation)

    center = inverse_haversine(center, secondary * HOSTILE_OFFSET, opposite)

    return build_box(center, primary, secondary * (1 - (HOSTILE_OFFSET * 2)))


def build_caps(center: LLA, width: float, height: float, orientation: str):
    direction = orientation_to_direction(orientation)
    opposite = opposite_dir(direction)

    (primary, secondary) = primary_secondary(width, height, orientation)

    down = inverse_haversine(center, secondary, opposite)
    start = inverse_haversine(down, primary / 2.0, opposite - (math.pi / 2))

    def make_caps(start: LLA, dir: Direction):
        # TODO(bjc)
        # Assumed moving counter clockwise, and starting with an arc
        return ([], [])

    return make_caps(start, direction)


def build_reds(center: LLA, width: float, height: float, orientation: str):
    res1 = []
    res2 = []
    direction = orientation_to_direction(orientation)
    opposite = opposite_dir(direction)
    (primary, secondary) = primary_secondary(width, height, orientation)

    start = inverse_haversine(center, (secondary / 2) * RED_START_FACTOR, direction)

    dist = (secondary / 2) * RED_START_FACTOR - (secondary / 2)
    for i in range(5):
        offset = inverse_haversine(start, dist * (i / 5.0), opposite)
        res1.append(inverse_haversine(offset, 0.5, opposite - math.pi / 2))
        res2.append(inverse_haversine(offset, 0.5, opposite + math.pi / 2))

    hostile_center = inverse_haversine(center, secondary * HOSTILE_OFFSET, opposite)
    hostile_tip = inverse_haversine(
        hostile_center, (secondary * (1 - (HOSTILE_OFFSET * 2))) / 3, direction
    )

    for i in range(5):
        dist = (primary / 2) * ((i + 2) / 6)
        res1.append(inverse_haversine(hostile_tip, dist, opposite - math.pi / 2))
        res2.append(inverse_haversine(hostile_tip, dist, opposite + math.pi / 2))

    for i in range(5, 10):
        dist = (primary / 2) + (5 * i / 10)
        delta_offset = (math.pi / 2) + (i / 6)
        res1.append(inverse_haversine(center, dist, opposite - delta_offset))
        res2.append(inverse_haversine(center, dist, opposite + delta_offset))

    return (res1, res2)


center = (lat, lon)

sanitize = build_sanitize(center, width, height, orientation)
commit = build_commit(center, width, height, orientation)
hostile = build_hostile(center, width, height, orientation)

(cap1, cap2) = build_caps(center, width, height, orientation)

(red1, red2) = build_reds(center, width, height, orientation)

fig, ax = plt.subplots()

plot_llas(ax, sanitize, "sanitize")
plot_llas(ax, commit, "commit")
plot_llas(ax, hostile, "hostile")
plot_llas(ax, cap1, "cap1")
plot_llas(ax, cap2, "cap2")

plot_llas(ax, red1, "red1")
plot_llas(ax, red2, "red2")

plt.show()
