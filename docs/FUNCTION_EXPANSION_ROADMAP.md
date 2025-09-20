# SQL CLI Function Expansion Roadmap

**Date**: September 18, 2025
**Current Function Count**: ~200 functions
**Goal**: Comprehensive function library for business analytics and data science

## 📊 Statistical Functions (High Business Value)

### Priority 1 - Essential Statistics
- [x] **MEDIAN()** - Middle value in sorted dataset ✅
- [x] **MODE()** - Most frequent value(s) ✅ (numeric only)
- [x] **PERCENTILE(value, n)** - Nth percentile (0-100) ✅
- [ ] **QUARTILE(value, n)** - Quartiles (1-4)
- [x] **VARIANCE() / VAR()** - Statistical variance ✅
- [x] **VAR_POP()** - Population variance ✅
- [x] **VAR_SAMP()** - Sample variance ✅

### Priority 2 - Advanced Statistics
- [x] **CORR(x, y)** - Pearson correlation coefficient ✅
- [ ] **COVAR_POP(x, y)** - Population covariance
- [ ] **COVAR_SAMP(x, y)** - Sample covariance
- [ ] **SKEW() / SKEWNESS()** - Distribution skewness
- [ ] **KURTOSIS()** - Distribution kurtosis
- [ ] **MAD()** - Median Absolute Deviation
- [ ] **IQR()** - Interquartile Range
- [ ] **QUANTILE(value, q)** - General quantile function

### Priority 3 - Specialized Statistics
- [ ] **WEIGHTED_AVG(value, weight)** - Weighted average
- [ ] **GEOMETRIC_MEAN()** - Geometric mean
- [ ] **HARMONIC_MEAN()** - Harmonic mean
- [ ] **STDERR()** - Standard error
- [ ] **CONFIDENCE_INTERVAL()** - CI calculation
- [ ] **T_TEST()** - Student's t-test
- [ ] **CHI_SQUARE()** - Chi-square test

## 🔢 Trigonometric Functions (Mathematical Completeness)

### Basic Trigonometry
- [ ] **SIN(radians)** - Sine
- [ ] **COS(radians)** - Cosine
- [ ] **TAN(radians)** - Tangent
- [ ] **COT(radians)** - Cotangent
- [ ] **SEC(radians)** - Secant
- [ ] **CSC(radians)** - Cosecant

### Inverse Trigonometry
- [ ] **ASIN(value)** - Arc sine
- [ ] **ACOS(value)** - Arc cosine
- [ ] **ATAN(value)** - Arc tangent
- [ ] **ATAN2(y, x)** - Two-argument arc tangent
- [ ] **ACOT(value)** - Arc cotangent

### Hyperbolic Functions
- [ ] **SINH(value)** - Hyperbolic sine
- [ ] **COSH(value)** - Hyperbolic cosine
- [ ] **TANH(value)** - Hyperbolic tangent
- [ ] **ASINH(value)** - Inverse hyperbolic sine
- [ ] **ACOSH(value)** - Inverse hyperbolic cosine
- [ ] **ATANH(value)** - Inverse hyperbolic tangent

## 🔤 String Manipulation Functions

### Text Transformation
- [ ] **REVERSE(string)** - Reverse string
- [ ] **INITCAP(string)** / **PROPER(string)** - Capitalize each word
- [ ] **SWAPCASE(string)** - Swap case
- [ ] **ROT13(string)** - ROT13 encoding
- [ ] **NORMALIZE(string)** - Unicode normalization

### Pattern Matching
- [ ] **REGEXP_MATCH(string, pattern)** - Regex matching
- [ ] **REGEXP_REPLACE(string, pattern, replacement)** - Regex replacement
- [ ] **REGEXP_EXTRACT(string, pattern, group)** - Extract regex group
- [ ] **REGEXP_SPLIT(string, pattern)** - Split by regex
- [ ] **GLOB(string, pattern)** - Glob pattern matching

### Phonetic & Fuzzy Matching
- [ ] **SOUNDEX(string)** - Soundex algorithm
- [ ] **METAPHONE(string)** - Metaphone algorithm
- [ ] **DOUBLE_METAPHONE(string)** - Double Metaphone
- [ ] **NYSIIS(string)** - NYSIIS algorithm
- [ ] **MATCH_RATING(s1, s2)** - Match rating comparison

### String Utilities
- [ ] **CONCAT(s1, s2, ...)** - Concatenate strings
- [ ] **CONCAT_WS(separator, s1, s2, ...)** - Concatenate with separator
- [ ] **TRANSLATE(string, from, to)** - Character translation
- [ ] **OVERLAY(string, replacement, start, length)** - Replace substring
- [ ] **POSITION(substring IN string)** - Find position
- [ ] **QUOTE(string)** - Quote string
- [ ] **UNQUOTE(string)** - Remove quotes
- [ ] **ESCAPE(string, chars)** - Escape special characters

### Character Functions
- [ ] **ASCII(char)** - ASCII value
- [ ] **CHAR(code)** - Character from code
- [ ] **UNICODE(char)** - Unicode code point
- [ ] **ORD(char)** - Ordinal value

## 📅 Date/Time Functions

### Date Arithmetic
- [ ] **ADD_MONTHS(date, n)** - Add months
- [ ] **ADD_YEARS(date, n)** - Add years
- [ ] **MONTHS_BETWEEN(date1, date2)** - Months between dates
- [ ] **YEARS_BETWEEN(date1, date2)** - Years between dates
- [ ] **AGE(date1, date2)** - Age calculation

### Date Extraction
- [ ] **WEEK(date)** - Week number
- [ ] **YEARWEEK(date)** - Year and week
- [ ] **DAYOFYEAR(date)** - Day of year (1-366)
- [ ] **LAST_DAY(date)** - Last day of month
- [ ] **NEXT_DAY(date, weekday)** - Next occurrence of weekday

### Date Construction
- [ ] **MAKE_DATE(year, month, day)** - Create date
- [ ] **MAKE_TIME(hour, min, sec)** - Create time
- [ ] **MAKE_TIMESTAMP(year, month, day, hour, min, sec)** - Create timestamp
- [ ] **DATE_FROM_PARTS(year, month, day)** - Build date from parts

### Date Truncation
- [ ] **DATE_TRUNC(unit, date)** - Truncate to unit
- [ ] **TRUNC_YEAR(date)** - Truncate to year
- [ ] **TRUNC_MONTH(date)** - Truncate to month
- [ ] **TRUNC_WEEK(date)** - Truncate to week

## 💰 Financial Functions

### Time Value of Money
- [ ] **NPV(rate, values...)** - Net Present Value
- [ ] **IRR(values...)** - Internal Rate of Return
- [ ] **MIRR(values, finance_rate, reinvest_rate)** - Modified IRR
- [ ] **XNPV(rate, dates, values)** - NPV for irregular periods
- [ ] **XIRR(dates, values)** - IRR for irregular periods

### Loan Calculations
- [ ] **PMT(rate, nper, pv, fv, type)** - Payment calculation
- [ ] **PPMT(rate, per, nper, pv, fv, type)** - Principal payment
- [ ] **IPMT(rate, per, nper, pv, fv, type)** - Interest payment
- [ ] **FV(rate, nper, pmt, pv, type)** - Future value
- [ ] **PV(rate, nper, pmt, fv, type)** - Present value
- [ ] **RATE(nper, pmt, pv, fv, type)** - Interest rate
- [ ] **NPER(rate, pmt, pv, fv, type)** - Number of periods

### Investment Analysis
- [ ] **COMPOUND(principal, rate, periods)** - Compound interest
- [ ] **DISCOUNT(price, redemption, basis)** - Discount rate
- [ ] **EFFECT(nominal_rate, npery)** - Effective interest rate
- [ ] **NOMINAL(effect_rate, npery)** - Nominal interest rate
- [ ] **SLN(cost, salvage, life)** - Straight-line depreciation
- [ ] **DDB(cost, salvage, life, period)** - Double declining balance
- [ ] **DB(cost, salvage, life, period)** - Declining balance

## 🎲 Utility Functions

### Hashing & Encoding
- [ ] **HASH(value)** - Generic hash
- [ ] **SHA1(string)** - SHA-1 hash
- [ ] **SHA256(string)** - SHA-256 hash (have this?)
- [ ] **SHA512(string)** - SHA-512 hash
- [ ] **BASE64_ENCODE(string)** - Base64 encode
- [ ] **BASE64_DECODE(string)** - Base64 decode
- [ ] **HEX_ENCODE(string)** - Hex encode
- [ ] **HEX_DECODE(string)** - Hex decode
- [ ] **URL_ENCODE(string)** - URL encoding
- [ ] **URL_DECODE(string)** - URL decoding

### UUID & Random
- [ ] **UUID()** / **GENERATE_UUID()** - Generate UUID v4
- [ ] **UUID_V1()** - Generate UUID v1
- [ ] **RANDOM_STRING(length, charset)** - Random string
- [ ] **RANDOM_BYTES(n)** - Random bytes
- [ ] **RANDOM_CHOICE(v1, v2, ...)** - Random selection

### Data Generation
- [ ] **GENERATE_SERIES(start, stop, step)** - Number series
- [ ] **GENERATE_DATE_SERIES(start, stop, interval)** - Date series
- [ ] **GENERATE_ARRAY(start, stop, step)** - Array generation
- [ ] **SEQUENCE(n)** - Simple sequence 1..n

### Encryption (Simple)
- [ ] **ENCRYPT(text, key)** - Simple encryption
- [ ] **DECRYPT(text, key)** - Simple decryption
- [ ] **ROT47(string)** - ROT47 encoding
- [ ] **XOR(text, key)** - XOR cipher

## 🎨 Fun Functions

### Text Effects
- [ ] **EMOJI(name)** - Convert to emoji (":smile:" → 😊)
- [ ] **EMOTICON(name)** - Text emoticons
- [ ] **ASCII_ART(text, font)** - ASCII art text
- [ ] **BANNER(text)** - Banner text
- [ ] **SPARKLINE(numbers)** - Mini chart in text

### Games & Puzzles
- [ ] **MORSE_CODE(text)** - Convert to Morse
- [ ] **FROM_MORSE(code)** - From Morse code
- [ ] **PIG_LATIN(text)** - Pig Latin
- [ ] **LEETSPEAK(text)** - L33t speak
- [ ] **SCRAMBLE(text)** - Scramble letters

### Conversational
- [ ] **GREETING(time)** - Time-appropriate greeting
- [ ] **FORTUNE()** - Random fortune
- [ ] **JOKE()** - Random joke
- [ ] **FACT()** - Random fact
- [ ] **QUOTE()** - Random quote

## Implementation Priority

### Phase 1: Core Statistics (Today!)
1. MEDIAN()
2. PERCENTILE()
3. MODE()
4. VARIANCE() family
5. CORRELATION()

### Phase 2: Trigonometry
Complete mathematical foundation with all trig functions

### Phase 3: Advanced String
Regex support and phonetic matching

### Phase 4: Financial
High-value business functions

### Phase 5: Fun!
Because data analysis should be enjoyable

## Notes

- Functions should handle NULL values gracefully
- Aggregate functions need special handling in GROUP BY context
- Window function variants where applicable
- Consider vectorized implementations for performance
- Add comprehensive examples for each function
- Ensure consistent naming (SQL standard where possible)