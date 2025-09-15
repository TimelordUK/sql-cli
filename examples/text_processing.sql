-- Text processing functions showcase

-- Basic text cleaning
SELECT
    'Hello, world!' as original,
    STRIP_PUNCTUATION('Hello, world!') as no_punct,
    TOKENIZE('Hello, world!', 'lower') as tokenized,
    WORD_COUNT('Hello, world!') as word_count;
GO

-- Clean messy text
SELECT
    '  The   quick  brown   fox...  ' as messy,
    CLEAN_TEXT('  The   quick  brown   fox...  ') as cleaned,
    STRIP_PUNCTUATION('  The   quick  brown   fox...  ') as no_punct;
GO

-- Word frequency analysis (much cleaner now!)
WITH
    words AS (
        SELECT value AS word
        FROM SPLIT(TOKENIZE('The quick brown fox jumps over the lazy dog. The fox is quick, very quick!', 'lower'))
    )
SELECT
    word,
    COUNT('*') AS frequency
FROM words
WHERE LENGTH(word) > 0
GROUP BY word
ORDER BY frequency DESC
LIMIT 10;
GO

-- Extract only longer words
SELECT
    EXTRACT_WORDS('The quick brown fox jumps over the lazy dog', 4, 'upper') as long_words_upper,
    EXTRACT_WORDS('The quick brown fox jumps over the lazy dog', 5, 'lower') as longer_words_lower;
GO

-- Analyze text statistics
SELECT
    WORD_COUNT('Data processing is essential for modern analytics. Clean data leads to better insights!') as total_words,
    LENGTH('Data processing is essential for modern analytics. Clean data leads to better insights!') as total_chars,
    LENGTH(STRIP_PUNCTUATION('Data processing is essential for modern analytics. Clean data leads to better insights!', '')) as alphanumeric_chars,
    LENGTH('Data processing is essential for modern analytics. Clean data leads to better insights!') - LENGTH(STRIP_PUNCTUATION('Data processing is essential for modern analytics. Clean data leads to better insights!', '')) as punctuation_count;
GO

-- Process file names and identifiers
SELECT
    'my-file_v2.0.txt' as filename,
    STRIP_PUNCTUATION('my-file_v2.0.txt', '') as clean_id,
    TOKENIZE('my-file_v2.0.txt') as tokens,
    EXTRACT_WORDS('my-file_v2.0.txt', 2) as words;
GO

-- Tokenize and count unique words
WITH
    tokens AS (
        SELECT value as word
        FROM SPLIT(
            TOKENIZE('To be or not to be, that is the question', 'lower')
        )
    )
SELECT
    COUNT(DISTINCT word) as unique_words,
    COUNT('*') as total_words
FROM tokens
WHERE LENGTH(word) > 0;
GO