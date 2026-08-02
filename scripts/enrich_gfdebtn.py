#!/usr/bin/env python3
"""Annotate the FRED GFDEBTN series (total US public debt, $ millions, quarterly)
with historical context: presidents, Fed chairs, NBER recessions, major wars and
the fiscal/monetary events that moved the line.

Reads  data/GFDEBTN.csv
Writes data/GFDEBTN_enriched.csv   (wide, one row per quarter)
       data/us_econ_events.csv     (narrow lookup, joinable on observation_date)

Stdlib only -- run with:  py -3 scripts/enrich_gfdebtn.py
Re-run after downloading a fresh GFDEBTN.csv from FRED.
"""

import csv
import os

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
SRC = os.path.join(ROOT, "data", "GFDEBTN.csv")
OUT_WIDE = os.path.join(ROOT, "data", "GFDEBTN_enriched.csv")
OUT_EVENTS = os.path.join(ROOT, "data", "us_econ_events.csv")


def quarters(start_y, start_q, end_y, end_q):
    """Inclusive range of quarter-start dates as 'YYYY-MM-01' strings."""
    out = []
    y, q = start_y, start_q
    while (y, q) <= (end_y, end_q):
        out.append("%04d-%02d-01" % (y, (q - 1) * 3 + 1))
        q += 1
        if q == 5:
            y, q = y + 1, 1
    return out


def spans_to_map(spans, value_index=4):
    """[(y1,q1,y2,q2,value), ...] -> {date: value}"""
    m = {}
    for s in spans:
        for d in quarters(s[0], s[1], s[2], s[3]):
            m[d] = s[value_index]
    return m


# --- Presidents: attributed to whoever held office on the observation date. ---
# Inaugurations are Jan 20, so the Jan-01 quarter still belongs to the outgoing
# president. `admin` is a term label so Trump's two non-consecutive terms don't
# collapse into one group.
PRESIDENTS = spans_to_map([
    (1966, 1, 1969, 1, ("Lyndon B. Johnson", "Democrat", "Johnson")),
    (1969, 2, 1974, 3, ("Richard Nixon", "Republican", "Nixon")),  # resigned 9 Aug 1974
    (1974, 4, 1977, 1, ("Gerald Ford", "Republican", "Ford")),
    (1977, 2, 1981, 1, ("Jimmy Carter", "Democrat", "Carter")),
    (1981, 2, 1989, 1, ("Ronald Reagan", "Republican", "Reagan")),
    (1989, 2, 1993, 1, ("George H. W. Bush", "Republican", "Bush 41")),
    (1993, 2, 2001, 1, ("Bill Clinton", "Democrat", "Clinton")),
    (2001, 2, 2009, 1, ("George W. Bush", "Republican", "Bush 43")),
    (2009, 2, 2017, 1, ("Barack Obama", "Democrat", "Obama")),
    (2017, 2, 2021, 1, ("Donald Trump", "Republican", "Trump I")),
    (2021, 2, 2025, 1, ("Joe Biden", "Democrat", "Biden")),
    (2025, 2, 2030, 4, ("Donald Trump", "Republican", "Trump II")),
])

FED_CHAIRS = spans_to_map([
    (1966, 1, 1970, 1, "William McChesney Martin"),
    (1970, 2, 1978, 1, "Arthur Burns"),
    (1978, 2, 1979, 3, "G. William Miller"),
    (1979, 4, 1987, 3, "Paul Volcker"),
    (1987, 4, 2006, 1, "Alan Greenspan"),
    (2006, 2, 2014, 1, "Ben Bernanke"),
    (2014, 2, 2018, 1, "Janet Yellen"),
    (2018, 2, 2030, 4, "Jerome Powell"),
])

ERAS = spans_to_map([
    (1966, 1, 1968, 4, "Vietnam & the Great Society"),
    (1969, 1, 1974, 4, "Nixon Shock & the End of Bretton Woods"),
    (1975, 1, 1980, 4, "Oil Shocks & Stagflation"),
    (1981, 1, 1989, 1, "Reagan Deficits"),
    (1989, 2, 1997, 4, "Cold War Dividend & Deficit Reduction"),
    (1998, 1, 2001, 1, "Surplus Years"),
    (2001, 2, 2007, 3, "War on Terror & Tax Cuts"),
    (2007, 4, 2014, 4, "Financial Crisis, Bailouts & QE"),
    (2015, 1, 2019, 4, "Long Recovery & the TCJA"),
    (2020, 1, 2021, 4, "COVID-19 Emergency Spending"),
    (2022, 1, 2030, 4, "Inflation, Rate Shock & Rising Interest Costs"),
])

# --- NBER recession quarters: any quarter overlapping a peak->trough window. ---
RECESSIONS = spans_to_map([
    (1969, 4, 1970, 4, "1969-70 recession"),
    (1973, 4, 1975, 1, "1973-75 oil shock recession"),
    (1980, 1, 1980, 3, "1980 recession"),
    (1981, 3, 1982, 4, "1981-82 Volcker recession"),
    (1990, 3, 1991, 1, "1990-91 Gulf War recession"),
    (2001, 1, 2001, 4, "2001 dot-com recession"),
    (2007, 4, 2009, 2, "2007-09 Great Recession"),
    (2020, 1, 2020, 2, "2020 COVID-19 recession"),
])

CONFLICTS = spans_to_map([
    (1966, 1, 1973, 1, "Vietnam War"),          # to the Paris Peace Accords
    (1990, 3, 1991, 1, "Gulf War"),
    (2001, 4, 2002, 4, "Afghanistan"),
    (2003, 1, 2011, 4, "Afghanistan + Iraq"),
    (2012, 1, 2021, 3, "Afghanistan"),          # to the Aug 2021 withdrawal
])

# --- Point events, keyed to the quarter they fall in. -------------------------
EVENTS = {
    "1966-01-01": ("US troop levels in Vietnam pass 200,000", "War"),
    "1968-04-01": ("Revenue & Expenditure Control Act: 10% Vietnam surtax", "Fiscal"),
    "1969-10-01": ("Recession begins (Dec 1969)", "Recession"),
    "1971-07-01": ("Nixon closes the gold window - Bretton Woods ends", "Monetary"),
    "1973-01-01": ("Paris Peace Accords end US combat in Vietnam", "War"),
    "1973-10-01": ("OPEC oil embargo; recession begins (Nov 1973)", "Crisis"),
    "1974-07-01": ("Nixon resigns over Watergate (Aug 1974)", "Political"),
    "1975-04-01": ("Fall of Saigon - Vietnam War ends", "War"),
    "1979-01-01": ("Iranian Revolution triggers the second oil shock", "Crisis"),
    "1979-07-01": ("Volcker takes over the Fed and attacks inflation", "Monetary"),
    "1980-01-01": ("Fed funds rate peaks near 20%", "Monetary"),
    "1981-07-01": ("Reagan's ERTA tax cuts; double-dip recession begins", "Fiscal"),
    "1982-07-01": ("TEFRA tax rise; Mexico default opens LatAm debt crisis", "Crisis"),
    "1983-04-01": ("Social Security Amendments of 1983", "Fiscal"),
    "1985-10-01": ("Gramm-Rudman-Hollings Balanced Budget Act", "Fiscal"),
    "1987-10-01": ("Black Monday - Dow falls 22.6% in a day", "Crisis"),
    "1989-07-01": ("FIRREA: the savings & loan bailout", "Crisis"),
    "1990-07-01": ("Iraq invades Kuwait; recession begins (Jul 1990)", "War"),
    "1990-10-01": ("OBRA-90 - Bush breaks the 'read my lips' pledge", "Fiscal"),
    "1991-01-01": ("Operation Desert Storm", "War"),
    "1993-07-01": ("OBRA-93 raises top rates; deficit reduction begins", "Fiscal"),
    "1995-10-01": ("Gingrich-Clinton government shutdowns", "Political"),
    "1997-07-01": ("Balanced Budget Act of 1997; Asian financial crisis", "Fiscal"),
    "1998-07-01": ("Russian default and the LTCM collapse", "Crisis"),
    "1998-10-01": ("First federal budget surplus since 1969 (FY1998)", "Milestone"),
    "2000-01-01": ("Dot-com bubble peaks (Mar 2000)", "Crisis"),
    "2001-04-01": ("EGTRRA - the first Bush tax cuts", "Fiscal"),
    "2001-07-01": ("9/11 attacks; war in Afghanistan begins", "War"),
    "2003-01-01": ("Iraq War begins; JGTRRA accelerates the tax cuts", "War"),
    "2005-07-01": ("Hurricane Katrina and emergency relief spending", "Crisis"),
    "2007-07-01": ("Subprime crisis erupts - BNP freezes its funds", "Crisis"),
    "2008-07-01": ("Lehman collapses; $700bn TARP bailout", "Crisis"),
    "2009-01-01": ("ARRA - $787bn stimulus; Fed begins QE1", "Fiscal"),
    "2010-01-01": ("Eurozone sovereign debt crisis (Greece)", "Crisis"),
    "2011-07-01": ("Debt ceiling crisis; S&P strips the US of its AAA", "Political"),
    "2013-01-01": ("Fiscal cliff deal, then sequestration cuts", "Fiscal"),
    "2013-10-01": ("16-day government shutdown", "Political"),
    "2017-10-01": ("Tax Cuts and Jobs Act (Dec 2017)", "Fiscal"),
    "2018-10-01": ("35-day shutdown - the longest to that point", "Political"),
    "2020-01-01": ("COVID-19 pandemic; $2.2tn CARES Act", "Crisis"),
    "2020-10-01": ("$900bn December relief package", "Fiscal"),
    "2021-01-01": ("American Rescue Plan - $1.9tn", "Fiscal"),
    "2021-07-01": ("US withdraws from Afghanistan (Aug 2021)", "War"),
    "2021-10-01": ("Infrastructure Investment and Jobs Act", "Fiscal"),
    "2022-01-01": ("Russia invades Ukraine; Fed starts hiking (Mar 2022)", "War"),
    "2022-07-01": ("CPI peaks at 9.1%; Inflation Reduction Act", "Fiscal"),
    "2023-01-01": ("Debt ceiling standoff; Silicon Valley Bank fails", "Crisis"),
    "2023-04-01": ("Fiscal Responsibility Act suspends the debt ceiling", "Political"),
    "2023-07-01": ("Fitch downgrades the US from AAA to AA+", "Political"),
    "2025-04-01": ("Sweeping US tariffs announced; Moody's downgrade", "Policy"),
    "2025-07-01": ("One Big Beautiful Bill Act", "Fiscal"),
    "2025-10-01": ("Longest government shutdown in US history", "Political"),
}

TRILLION_MARKS = [1, 5, 10, 15, 20, 25, 30, 35]


def main():
    with open(SRC, newline="") as fh:
        rows = [(r["observation_date"], float(r["GFDEBTN"]))
                for r in csv.DictReader(fh)]

    # Which quarter first crossed each round trillion?
    crossings = {}
    for mark in TRILLION_MARKS:
        threshold = mark * 1_000_000.0  # series is in $ millions
        for i, (date, val) in enumerate(rows):
            if val >= threshold:
                if i > 0 and rows[i - 1][1] < threshold:
                    crossings[date] = "Debt crosses $%dtn" % mark
                break

    wide_cols = [
        "observation_date", "year", "quarter",
        "debt_millions", "debt_trillions",
        "qoq_change_bn", "qoq_pct", "yoy_pct",
        "era", "president", "party", "admin", "fed_chair",
        "in_recession", "recession_name", "major_conflict",
        "event", "event_category", "milestone",
    ]

    with open(OUT_WIDE, "w", newline="") as fh:
        w = csv.writer(fh, lineterminator="\n")
        w.writerow(wide_cols)
        for i, (date, val) in enumerate(rows):
            year, month = int(date[:4]), int(date[5:7])
            prev = rows[i - 1][1] if i >= 1 else None
            prev_yr = rows[i - 4][1] if i >= 4 else None
            pres, party, admin = PRESIDENTS.get(date, ("", "", ""))
            evt, cat = EVENTS.get(date, ("", ""))
            w.writerow([
                date,
                year,
                (month - 1) // 3 + 1,
                "%.3f" % val,
                "%.4f" % (val / 1_000_000.0),
                "" if prev is None else "%.1f" % ((val - prev) / 1000.0),
                "" if prev is None else "%.2f" % ((val / prev - 1) * 100),
                "" if prev_yr is None else "%.2f" % ((val / prev_yr - 1) * 100),
                ERAS.get(date, ""),
                pres,
                party,
                admin,
                FED_CHAIRS.get(date, ""),
                "true" if date in RECESSIONS else "false",
                RECESSIONS.get(date, ""),
                CONFLICTS.get(date, ""),
                evt,
                cat,
                crossings.get(date, ""),
            ])

    # Narrow lookup table, for JOIN demos.
    with open(OUT_EVENTS, "w", newline="") as fh:
        w = csv.writer(fh, lineterminator="\n")
        w.writerow(["observation_date", "event", "event_category"])
        for date in sorted(set(EVENTS) | set(crossings)):
            if date in EVENTS:
                w.writerow([date, EVENTS[date][0], EVENTS[date][1]])
            if date in crossings:
                w.writerow([date, crossings[date], "Milestone"])

    print("wrote %s (%d rows)" % (OUT_WIDE, len(rows)))
    print("wrote %s" % OUT_EVENTS)
    print("trillion crossings: %s" % ", ".join(
        "%s %s" % (d, t) for d, t in sorted(crossings.items())))


if __name__ == "__main__":
    main()
