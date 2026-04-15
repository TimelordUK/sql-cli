-- Bank of England spot FX rates via WEB CTE
--
-- Data source: BoE Statistical Interactive Database (IADB)
-- Endpoint:    /boeapps/database/_iadb-fromshowcolumns.asp
--
-- Query parameters:
--   csv.x=yes               return CSV
--   Datefrom / Dateto       DD/Mmm/YYYY (e.g. 01/Jan/2025)
--   SeriesCodes             comma-separated series codes
--   CSVF=TN                 titles + numeric values
--   UsingCodes=Y            column headers use series codes
--   VPD=Y, VFD=N            include primary data, exclude future/forecast
--
-- Common spot rate series codes:
--   XUDLUSS  - US Dollar  (USD per GBP)
--   XUDLERS  - Euro
--   XUDLJYS  - Japanese Yen
--   XUDLSFS  - Swiss Franc
--   XUDLCDS  - Canadian Dollar
--   XUDLBK73 - Sterling effective exchange rate index

-- Example 1: USD/GBP spot for January 2025
WITH WEB usd AS (
    URL 'https://www.bankofengland.co.uk/boeapps/database/_iadb-fromshowcolumns.asp?csv.x=yes&Datefrom=01/Jan/2025&Dateto=01/Feb/2025&SeriesCodes=XUDLUSS&CSVF=TN&UsingCodes=Y&VPD=Y&VFD=N'
    FORMAT CSV
)
SELECT DATE, XUDLUSS AS usd_per_gbp
FROM usd
ORDER BY DATE;
GO

-- Example 2: Multi-currency basket, most recent first
WITH WEB fx AS (
    URL 'https://www.bankofengland.co.uk/boeapps/database/_iadb-fromshowcolumns.asp?csv.x=yes&Datefrom=01/Jan/2025&Dateto=01/Apr/2025&SeriesCodes=XUDLUSS,XUDLERS,XUDLJYS,XUDLSFS,XUDLCDS&CSVF=TN&UsingCodes=Y&VPD=Y&VFD=N'
    FORMAT CSV
)
SELECT
    DATE,
    XUDLUSS AS usd,
    XUDLERS AS eur,
    XUDLJYS AS jpy,
    XUDLSFS AS chf,
    XUDLCDS AS cad
FROM fx
ORDER BY DATE DESC
LIMIT 20;
GO

-- Example 3: Monthly average USD/GBP using SUBSTRING on the date text
WITH WEB usd AS (
    URL 'https://www.bankofengland.co.uk/boeapps/database/_iadb-fromshowcolumns.asp?csv.x=yes&Datefrom=01/Jan/2024&Dateto=31/Dec/2024&SeriesCodes=XUDLUSS&CSVF=TN&UsingCodes=Y&VPD=Y&VFD=N'
    FORMAT CSV
)
SELECT
    SUBSTRING(DATE, 3, 8) AS month,
    AVG(XUDLUSS) AS avg_usd_per_gbp,
    MIN(XUDLUSS) AS min_rate,
    MAX(XUDLUSS) AS max_rate,
    COUNT('*')   AS observations
FROM usd
GROUP BY SUBSTRING(DATE, 3, 8)
ORDER BY month;
GO

-- Example 4: Join two series - EUR vs USD strength against GBP
WITH
    WEB usd AS (
        URL 'https://www.bankofengland.co.uk/boeapps/database/_iadb-fromshowcolumns.asp?csv.x=yes&Datefrom=01/Mar/2025&Dateto=01/Apr/2025&SeriesCodes=XUDLUSS&CSVF=TN&UsingCodes=Y&VPD=Y&VFD=N'
        FORMAT CSV
    ),
    WEB eur AS (
        URL 'https://www.bankofengland.co.uk/boeapps/database/_iadb-fromshowcolumns.asp?csv.x=yes&Datefrom=01/Mar/2025&Dateto=01/Apr/2025&SeriesCodes=XUDLERS&CSVF=TN&UsingCodes=Y&VPD=Y&VFD=N'
        FORMAT CSV
    )
SELECT
    usd.DATE,
    usd.XUDLUSS AS usd_per_gbp,
    eur.XUDLERS AS eur_per_gbp,
    usd.XUDLUSS / eur.XUDLERS AS usd_per_eur
FROM usd
INNER JOIN eur ON usd.DATE = eur.DATE
ORDER BY usd.DATE;
GO
