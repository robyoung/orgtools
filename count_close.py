import sys
from collections import Counter
from datetime import date, timedelta
from enum import Enum
from pathlib import Path
from typing import Generator
from typing_extensions import assert_never

import orgparse


class TimePeriod(Enum):
    YEAR = 1
    MONTH = 2
    WEEK = 3
    DAY = 4

    def get_date(self, target: date) -> date:
        match self:
            case TimePeriod.YEAR:
                return target.replace(month=1, day=1)
            case TimePeriod.MONTH:
                return target.replace(day=1)
            case TimePeriod.WEEK:
                return target - timedelta(days=target.weekday())
            case TimePeriod.DAY:
                return target
            case _:
                assert_never(self)

    def dates(self, start: date, end: date) -> Generator[date, None, None]:
        match self:
            case TimePeriod.YEAR:
                return (date(year, 1, 1) for year in range(start.year, end.year + 1))
            case TimePeriod.MONTH:
                return (
                    date(year, month, 1)
                    for year in range(start.year, end.year + 1)
                    for month in range(
                        start.month if year == start.year else 1,
                        (end.month if year == end.year else 12) + 1,
                    )
                )
            case TimePeriod.WEEK:
                return (
                    start + timedelta(days=days)
                    for days in range(0, (end - start).days, 7)
                )
            case TimePeriod.DAY:
                return (
                    start + timedelta(days=days) for days in range((end - start).days)
                )
            case _:
                assert_never(self)


def dates_closed(path: str, by: TimePeriod) -> Generator[date, None, None]:
    for org_file in Path(path).expanduser().glob("**/*.org"):
        for node in orgparse.load(org_file)[1:]:
            if node.closed:
                yield by.get_date(node.closed.start.date())


def print_frequency(path: str, by: TimePeriod) -> None:
    completed = Counter(dates_closed(path, by))

    for d in by.dates(min(completed.keys()), max(completed.keys())):
        count = completed.get(d, 0)
        print(f"{d.isoformat()}  {count:3}  {'=' * count}")


def main():
    if len(sys.argv) > 1:
        by = getattr(TimePeriod, sys.argv[1].upper())
    else:
        by = TimePeriod.WEEK

    print_frequency("~/Notes", by)


main()
