-- #! ../data/GFDEBTN_enriched.csv
--
-- US total public debt (FRED GFDEBTN, $ millions, quarterly) annotated with
-- presidents, Fed chairs, NBER recessions, wars and major fiscal events.
--
-- NOTE: FRED stamps each quarter with its START date, but the value is the
-- END-of-quarter debt level. So an event sits on the row whose figure already
-- reflects it: Lehman (Sep 2008) is on the 2008-07-01 row.

-- Every annotated quarter: what happened, and what debt did that quarter
SELECT
    observation_date,
    debt_trillions,
    qoq_pct,
    event_category,
    event
FROM GFDEBTN_enriched
WHERE event <> ''
ORDER BY observation_date;
GO

-- The round-trillion milestones, and how long each took to reach
SELECT
    observation_date,
    debt_trillions,
    milestone,
    president,
    era
FROM GFDEBTN_enriched
WHERE milestone <> '';
GO

-- Debt by administration: where it started, where it ended, how fast it grew
SELECT
    admin,
    party,
    COUNT(*) AS quarters,
    MIN(debt_trillions) AS from_trn,
    MAX(debt_trillions) AS to_trn,
    ROUND(AVG(qoq_pct), 2) AS avg_qoq_pct
FROM GFDEBTN_enriched
GROUP BY admin, party
ORDER BY MIN(observation_date);
GO

-- Does debt grow faster in recessions? (spoiler: yes, a lot)
SELECT
    in_recession,
    COUNT(*) AS quarters,
    ROUND(AVG(qoq_pct), 2) AS avg_qoq_pct,
    ROUND(MAX(qoq_pct), 2) AS worst_quarter_pct
FROM GFDEBTN_enriched
WHERE qoq_pct IS NOT NULL
GROUP BY in_recession;
GO

-- The ten fastest-growing quarters in 60 years, with their context
SELECT
    observation_date,
    debt_trillions,
    qoq_pct,
    qoq_change_bn,
    president,
    recession_name,
    event
FROM GFDEBTN_enriched
WHERE qoq_pct IS NOT NULL
ORDER BY qoq_pct DESC
LIMIT 10;
GO

-- Wartime quarters: cost of the Vietnam, Gulf, Afghanistan and Iraq eras
SELECT
    major_conflict,
    COUNT(*) AS quarters,
    MIN(observation_date) AS from_date,
    MAX(observation_date) AS to_date,
    ROUND(AVG(yoy_pct), 2) AS avg_yoy_pct
FROM GFDEBTN_enriched
WHERE major_conflict <> ''
GROUP BY major_conflict
ORDER BY MIN(observation_date);
GO

-- Which Fed chair presided over the fastest debt growth?
SELECT
    fed_chair,
    COUNT(*) AS quarters,
    ROUND(AVG(yoy_pct), 2) AS avg_yoy_pct
FROM GFDEBTN_enriched
WHERE yoy_pct IS NOT NULL
GROUP BY fed_chair
ORDER BY AVG(yoy_pct) DESC;
GO

-- The Clinton surplus years: the only sustained flat stretch in the series
SELECT
    observation_date,
    debt_trillions,
    qoq_pct,
    yoy_pct,
    event
FROM GFDEBTN_enriched
WHERE era = 'Surplus Years'
ORDER BY observation_date;
GO
