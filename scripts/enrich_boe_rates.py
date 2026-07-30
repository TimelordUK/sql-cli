#!/usr/bin/env python3
"""
Enrich the Bank of England Bank Rate history with ISO dates and economic context.

The raw download from bankofengland.co.uk uses a "18 Dec 25" date format that
sql-cli treats as a plain string - ORDER BY sorts lexically and YEAR() returns
nonsense. This script rewrites the dates as ISO (YYYY-MM-DD) so the engine
infers a real DATETIME column, and denormalises economic context alongside
(sql-cli takes one data source per invocation, so a join is not an option).

Two-digit years are pivoted at 70: 75 -> 1975, 25 -> 2025.

Usage:
    python3 scripts/enrich_boe_rates.py
    python3 scripts/enrich_boe_rates.py --src other.csv --out other_enriched.csv

To add a new event, add an entry to EVENTS keyed by the ISO date of the rate
change it belongs to. The script warns if a key matches no rate change.
"""

import argparse
import csv
from datetime import date, datetime
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parent.parent
DEFAULT_SRC = REPO_ROOT / "data" / "boe_rate_history.csv"
DEFAULT_OUT = REPO_ROOT / "data" / "boe_rate_history_enriched.csv"

MONTHS = {m: i + 1 for i, m in enumerate(
    "Jan Feb Mar Apr May Jun Jul Aug Sep Oct Nov Dec".split())}

# Economic periods. Boundaries are the event that opened them where there is a
# clean one (ERM entry, Big Bang, BoE independence, Lehman week, first COVID cut).
ERAS = [
    ("1975-01-01", "1976-09-30", "Stagflation & Sterling Crisis"),
    ("1976-10-01", "1979-05-03", "IMF Bailout & Winter of Discontent"),
    ("1979-05-04", "1982-12-31", "Monetarist Shock"),
    ("1983-01-01", "1986-10-26", "Recovery & Deregulation"),
    ("1986-10-27", "1988-12-31", "Big Bang & Lawson Boom"),
    ("1989-01-01", "1990-10-07", "Overheating & Shadowing the D-Mark"),
    ("1990-10-08", "1992-09-15", "ERM Membership"),
    ("1992-09-16", "1997-05-05", "Post-ERM Inflation Targeting"),
    ("1997-05-06", "2000-03-09", "BoE Independence & Dotcom Bubble"),
    ("2000-03-10", "2003-03-31", "Dotcom Bust"),
    ("2003-04-01", "2007-08-08", "Great Moderation & Credit Boom"),
    ("2007-08-09", "2009-03-04", "Global Financial Crisis"),
    ("2009-03-05", "2016-06-23", "ZIRP & Quantitative Easing"),
    ("2016-06-24", "2020-03-10", "Brexit Era"),
    ("2020-03-11", "2021-12-15", "COVID-19 Pandemic"),
    ("2021-12-16", "2023-08-02", "Inflation Shock"),
    ("2023-08-03", None, "Disinflation & Easing Cycle"),
]

PMS = [
    ("1974-03-04", "1976-04-05", "Harold Wilson", "Labour"),
    ("1976-04-06", "1979-05-03", "James Callaghan", "Labour"),
    ("1979-05-04", "1990-11-27", "Margaret Thatcher", "Conservative"),
    ("1990-11-28", "1997-05-01", "John Major", "Conservative"),
    ("1997-05-02", "2007-06-26", "Tony Blair", "Labour"),
    ("2007-06-27", "2010-05-10", "Gordon Brown", "Labour"),
    ("2010-05-11", "2016-07-12", "David Cameron", "Conservative"),
    ("2016-07-13", "2019-07-23", "Theresa May", "Conservative"),
    ("2019-07-24", "2022-09-05", "Boris Johnson", "Conservative"),
    ("2022-09-06", "2022-10-24", "Liz Truss", "Conservative"),
    ("2022-10-25", "2024-07-04", "Rishi Sunak", "Conservative"),
    ("2024-07-05", None, "Keir Starmer", "Labour"),
]

CHANCELLORS = [
    ("1974-03-05", "1979-05-03", "Denis Healey"),
    ("1979-05-04", "1983-06-10", "Geoffrey Howe"),
    ("1983-06-11", "1989-10-25", "Nigel Lawson"),
    ("1989-10-26", "1990-11-27", "John Major"),
    ("1990-11-28", "1993-05-26", "Norman Lamont"),
    ("1993-05-27", "1997-05-01", "Kenneth Clarke"),
    ("1997-05-02", "2007-06-27", "Gordon Brown"),
    ("2007-06-28", "2010-05-10", "Alistair Darling"),
    ("2010-05-11", "2016-07-12", "George Osborne"),
    ("2016-07-13", "2019-07-23", "Philip Hammond"),
    ("2019-07-24", "2020-02-12", "Sajid Javid"),
    ("2020-02-13", "2022-07-04", "Rishi Sunak"),
    ("2022-07-05", "2022-09-05", "Nadhim Zahawi"),
    ("2022-09-06", "2022-10-13", "Kwasi Kwarteng"),
    ("2022-10-14", "2024-07-04", "Jeremy Hunt"),
    ("2024-07-05", None, "Rachel Reeves"),
]

GOVERNORS = [
    ("1973-07-01", "1983-06-30", "Gordon Richardson"),
    ("1983-07-01", "1993-06-30", "Robin Leigh-Pemberton"),
    ("1993-07-01", "2003-06-30", "Eddie George"),
    ("2003-07-01", "2013-06-30", "Mervyn King"),
    ("2013-07-01", "2020-03-15", "Mark Carney"),
    ("2020-03-16", None, "Andrew Bailey"),
]

# Rate changes that are themselves a piece of economic history, keyed by the
# ISO date of the change. Note the BoE series does not contain Black Wednesday's
# intraday 12% -> 15% hikes (both reversed the same day), so 1992-09-22 is
# tagged as the ERM-exit aftermath instead.
EVENTS = {
    "1975-01-20": ("Post oil-shock easing begins", "Oil Shock"),
    "1976-10-07": ("Sterling crisis: MLR to 15%", "Currency Crisis"),
    "1976-11-22": ("IMF loan negotiations under way", "Currency Crisis"),
    "1977-10-17": ("Sterling stabilised, rate at decade low", "Recovery"),
    "1979-02-08": ("Winter of Discontent", "Industrial Unrest"),
    "1979-06-13": ("Howe's first Budget: VAT nearly doubled", "Fiscal Policy"),
    "1979-11-15": ("Record high 17% - Thatcher's monetarist squeeze", "Monetary Squeeze"),
    "1980-07-03": ("Recession bites, squeeze eases", "Recession"),
    "1981-03-11": ("The 364 economists' letter Budget", "Fiscal Policy"),
    "1984-07-11": ("Miners' strike & sterling pressure", "Industrial Unrest"),
    "1985-01-14": ("Sterling near parity with the dollar", "Currency Crisis"),
    "1986-10-15": ("Big Bang deregulates the City", "Deregulation"),
    "1987-10-23": ("Post-Black Monday emergency easing", "Market Crash"),
    "1987-11-04": ("Crash response continues", "Market Crash"),
    "1988-06-03": ("Lawson Boom: start of 12 hikes in 18 months", "Credit Boom"),
    "1989-10-06": ("Rate to 15% - Lawson resigns weeks later", "Political Crisis"),
    "1990-10-08": ("UK joins the ERM at DM 2.95", "ERM"),
    "1992-09-22": ("Black Wednesday aftermath - ERM exit", "Currency Crisis"),
    "1992-10-16": ("Post-ERM easing under inflation targeting", "Policy Regime Change"),
    "1997-05-06": ("Bank of England granted independence", "Policy Regime Change"),
    "1998-10-08": ("LTCM collapse & Russian default", "Financial Crisis"),
    "1999-06-10": ("Dotcom bubble inflating", "Asset Bubble"),
    "2000-02-10": ("Peak rate weeks before Nasdaq tops out", "Asset Bubble"),
    "2001-09-18": ("Emergency coordinated cut after 9/11", "Geopolitical Shock"),
    "2003-07-10": ("48-year low - dotcom bust trough", "Recession"),
    "2007-12-06": ("First cut of the credit crunch", "Financial Crisis"),
    "2008-10-08": ("Coordinated global emergency cut", "Financial Crisis"),
    "2008-11-06": ("150bp cut - largest since 1981", "Financial Crisis"),
    "2009-03-05": ("Record low 0.5% and QE launched", "Quantitative Easing"),
    "2016-08-04": ("Post-Brexit-referendum cut", "Brexit"),
    "2017-11-02": ("First hike in over ten years", "Normalisation"),
    "2020-03-11": ("COVID-19 emergency cut", "Pandemic"),
    "2020-03-19": ("Record low 0.1% as lockdown begins", "Pandemic"),
    "2021-12-16": ("First hike of the inflation cycle", "Inflation Shock"),
    "2022-09-22": ("Day before the Truss mini-Budget", "Fiscal Crisis"),
    "2022-11-03": ("75bp - biggest hike in 33 years", "Inflation Shock"),
    "2023-08-03": ("Peak of the tightening cycle", "Inflation Shock"),
    "2024-08-01": ("First cut of the easing cycle", "Disinflation"),
}

HEADER = [
    "date", "rate", "prev_rate", "change_bps", "direction", "days_since_prev",
    "era", "event", "event_category",
    "pm", "party", "chancellor", "governor", "boe_independent",
]

INDEPENDENCE = date(1997, 5, 6)


def parse_boe_date(s):
    """'18 Dec 25' -> date(2025, 12, 18). Two-digit years >= 70 are 19xx."""
    day, mon, year = s.strip().split()
    yy = int(year)
    return date(1900 + yy if yy >= 70 else 2000 + yy, MONTHS[mon], int(day))


def lookup(d, table):
    """Find the row of `table` whose [start, end] range covers `d`.

    Each row is (start_iso, end_iso_or_None, *values); returns the values
    tuple, or None if no range matches. An end of None means "to date".
    """
    for start, end, *values in table:
        lo = datetime.strptime(start, "%Y-%m-%d").date()
        hi = datetime.strptime(end, "%Y-%m-%d").date() if end else date(9999, 1, 1)
        if lo <= d <= hi:
            return values
    return None


def one(d, table):
    """lookup() for single-value tables, blank when nothing matches."""
    values = lookup(d, table)
    return values[0] if values else ""


def build_row(d, rate, prev):
    """Assemble one output row. `prev` is the (date, rate) before it, or None."""
    iso = d.isoformat()
    if prev:
        prev_date, prev_rate = prev
        bps = round((rate - prev_rate) * 100)
        days = (d - prev_date).days
        direction = "Hike" if bps > 0 else "Cut" if bps < 0 else "Hold"
        prev_rate_out = f"{prev_rate:.2f}"
    else:
        prev_rate_out, bps, days, direction = "", "", "", "First"

    pm = lookup(d, PMS) or ["", ""]
    event, category = EVENTS.get(iso, ("", ""))

    return [
        iso, f"{rate:.2f}", prev_rate_out, bps, direction, days,
        one(d, ERAS), event, category,
        pm[0], pm[1], one(d, CHANCELLORS), one(d, GOVERNORS),
        "true" if d >= INDEPENDENCE else "false",
    ]


def enrich(src, out):
    with open(src, newline="") as f:
        rows = [r for r in csv.DictReader(f) if r.get("Date Changed")]

    series = sorted(
        ((parse_boe_date(r["Date Changed"]), float(r["Rate"])) for r in rows),
        key=lambda t: t[0],
    )

    with open(out, "w", newline="") as f:
        writer = csv.writer(f)
        writer.writerow(HEADER)
        for i, (d, rate) in enumerate(series):
            writer.writerow(build_row(d, rate, series[i - 1] if i else None))

    print(f"wrote {out}: {len(series)} rows, {len(HEADER)} columns")
    print(f"date range: {series[0][0]} to {series[-1][0]}")

    orphans = sorted(set(EVENTS) - {d.isoformat() for d, _ in series})
    if orphans:
        print(f"WARNING - {len(orphans)} event dates match no rate change: {orphans}")
    else:
        print(f"events tagged: {len(EVENTS)}/{len(EVENTS)}")


def main():
    parser = argparse.ArgumentParser(description=__doc__.splitlines()[1])
    parser.add_argument("--src", type=Path, default=DEFAULT_SRC,
                        help=f"raw BoE CSV (default: {DEFAULT_SRC.relative_to(REPO_ROOT)})")
    parser.add_argument("--out", type=Path, default=DEFAULT_OUT,
                        help=f"enriched output (default: {DEFAULT_OUT.relative_to(REPO_ROOT)})")
    args = parser.parse_args()
    enrich(args.src, args.out)


if __name__ == "__main__":
    main()
