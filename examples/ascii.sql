-- ASCII and Character Code Functions Demo

-- Get ASCII/Unicode code of characters
SELECT ASCII('A') as letter_A,
       ASCII('Z') as letter_Z,
       ASCII('0') as digit_0,
       ASCII('€') as euro_symbol,
       ASCII('❤') as heart_emoji;
GO

-- Convert codes back to characters
SELECT CHAR(65) as code_65,
       CHAR(90) as code_90,
       CHAR(8364) as euro_code,
       CHAR(10084) as heart_code;
GO

-- Get all Unicode codes in a string
SELECT UNICODE('Hello') as hello_codes,
       UNICODE('SQL') as sql_codes,
       UNICODE('€$¥') as currency_codes;
GO

-- Type conversion functions
SELECT TO_INT('123') as string_to_int,
       TO_INT('45.67') as float_string_to_int,
       TO_DECIMAL('123.45') as string_to_decimal,
       TO_STRING(123) as int_to_string,
       TO_STRING(45.67) as float_to_string;
GO

-- Encoding and decoding
SELECT ENCODE('Hello World', 'base64') as base64_encoded,
       ENCODE('Hello World', 'hex') as hex_encoded;
GO

SELECT DECODE('SGVsbG8gV29ybGQ=', 'base64') as base64_decoded,
       DECODE('48656c6c6f20576f726c64', 'hex') as hex_decoded;
GO