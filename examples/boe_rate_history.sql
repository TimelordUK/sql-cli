-- #! ../data/boe_rate_history_enriched.csv
--
-- Bank of England Bank Rate, 1975-2025, tagged with economic eras and
-- the events behind the big moves. Source: bankofengland.co.uk, dates
-- converted to ISO so the engine treats them as real DATETIME values.

-- The famous moves: every rate change that is itself a piece of history
SELECT
    date,
    rate,
    change_bps,
    event_category,
    event
FROM boe_rate_history_enriched
WHERE event IS NOT NULL
ORDER BY date DESC;
GO

-- Fifty years in seventeen eras
SELECT
    era,
    COUNT(*) AS moves,
    MIN(rate) AS low,
    MAX(rate) AS high,
    ROUND(AVG(rate), 2) AS avg_rate
FROM boe_rate_history_enriched
GROUP BY era
ORDER BY MIN(date);
GO

-- The crisis playbook: how hard did the Bank cut, and how fast?
SELECT
    era,
    COUNT(*) AS cuts,
    SUM(change_bps) AS total_bps,
    MIN(change_bps) AS biggest_single_cut
FROM boe_rate_history_enriched
WHERE direction = 'Cut'
GROUP BY era
ORDER BY total_bps;
GO

-- Prime ministerial league table - who presided over the dearest money?
SELECT
    pm,
    party,
    COUNT(*) AS moves,
    SUM(CASE WHEN direction = 'Hike' THEN 1 ELSE 0 END) AS hikes,
    SUM(CASE WHEN direction = 'Cut' THEN 1 ELSE 0 END) AS cuts,
    ROUND(AVG(rate), 2) AS avg_rate,
    MAX(rate) AS peak
FROM boe_rate_history_enriched
GROUP BY pm, party
ORDER BY avg_rate DESC;
GO

-- Did independence calm things down? Compare the two regimes
SELECT
    boe_independent,
    COUNT(*) AS moves,
    ROUND(AVG(rate), 2) AS avg_rate,
    ROUND(AVG(ABS(change_bps)), 1) AS avg_move_size,
    MAX(ABS(change_bps)) AS biggest_move
FROM boe_rate_history_enriched
WHERE change_bps IS NOT NULL
GROUP BY boe_independent;
GO

-- The long silences: the biggest gaps between rate decisions
SELECT
    date,
    rate,
    days_since_prev,
    ROUND(days_since_prev / 365.25, 1) AS years_unchanged,
    era
FROM boe_rate_history_enriched
ORDER BY days_since_prev DESC
LIMIT 10;
GO

-- 1982 had 36 changes; 2015 had none. Busiest years on record
SELECT
    YEAR(date) AS yr,
    COUNT(*) AS moves,
    MIN(rate) AS low,
    MAX(rate) AS high,
    ROUND(MAX(rate) - MIN(rate), 2) AS swing
FROM boe_rate_history_enriched
GROUP BY YEAR(date)
ORDER BY moves DESC
LIMIT 10;
GO

-- Which month of the year does the Bank like to move in?
SELECT
    MONTH(date) AS month_no,
    MONTHNAME(date) AS month,
    COUNT(*) AS moves
FROM boe_rate_history_enriched
GROUP BY MONTH(date), MONTHNAME(date)
ORDER BY month_no;
GO

-- Every rate change of the Global Financial Crisis, peak to trough
SELECT
    date,
    DAYNAME(date) AS day,
    rate,
    change_bps,
    chancellor,
    event
FROM boe_rate_history_enriched
WHERE date >= '2007-07-01' AND date <= '2009-04-01'
ORDER BY date;
GO

-- Cross-check the precomputed prev_rate against a window function
SELECT
    date,
    rate,
    LAG(rate) OVER (ORDER BY date) AS lag_rate,
    prev_rate,
    change_bps
FROM boe_rate_history_enriched
WHERE era = 'Inflation Shock'
ORDER BY date;
GO

-- Banding: how much of the last 50 years was spent at each rate level?
WITH banded AS (
    SELECT
        date,
        rate,
        era,
        CASE
            WHEN rate < 1 THEN '0-1% (emergency)'
            WHEN rate < 5 THEN '1-5% (modern normal)'
            WHEN rate < 10 THEN '5-10% (old normal)'
            ELSE '10%+ (crisis money)'
        END AS band
    FROM boe_rate_history_enriched
)
SELECT
    band,
    COUNT(*) AS moves,
    MIN(YEAR(date)) AS first_year,
    MAX(YEAR(date)) AS last_year
FROM banded
GROUP BY band
ORDER BY first_year;
GO

-- Governors and the rates they inherited and left behind
SELECT
    governor,
    COUNT(*) AS moves,
    MIN(date) AS first_move,
    MAX(date) AS last_move,
    ROUND(AVG(rate), 2) AS avg_rate
FROM boe_rate_history_enriched
GROUP BY governor
ORDER BY first_move;
GO
