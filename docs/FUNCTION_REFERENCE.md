# SQL CLI Function Reference

This document is auto-generated from the function registry.

## Astronomical Functions

### AU()

**Description:** Returns one Astronomical Unit in meters (1.496 × 10^11)

**Arguments:** no arguments

**Returns:** FLOAT

**Examples:**
```sql
SELECT AU()
SELECT distance_m / AU() AS distance_au
```

### LIGHT_YEAR()

**Description:** Returns one light year in meters (9.461 × 10^15)

**Arguments:** no arguments

**Returns:** FLOAT

**Examples:**
```sql
SELECT LIGHT_YEAR()
SELECT star_distance / LIGHT_YEAR() AS distance_ly
```

### MASS_EARTH()

**Description:** Returns Earth's mass in kg (5.972 × 10^24)

**Arguments:** no arguments

**Returns:** FLOAT

**Examples:**
```sql
SELECT MASS_EARTH()
SELECT asteroid_mass / MASS_EARTH() AS earth_masses
```

### MASS_JUPITER()

**Description:** Returns Jupiter's mass in kg (1.898 × 10^27)

**Arguments:** no arguments

**Returns:** FLOAT

**Examples:**
```sql
SELECT MASS_JUPITER()
SELECT exoplanet_mass / MASS_JUPITER() AS jupiter_masses
```

### MASS_MARS()

**Description:** Returns Mars's mass in kg (6.417 × 10^23)

**Arguments:** no arguments

**Returns:** FLOAT

**Examples:**
```sql
SELECT MASS_MARS()
```

### MASS_MERCURY()

**Description:** Returns Mercury's mass in kg (3.301 × 10^23)

**Arguments:** no arguments

**Returns:** FLOAT

**Examples:**
```sql
SELECT MASS_MERCURY()
```

### MASS_MOON()

**Description:** Returns the Moon's mass in kg (7.342 × 10^22)

**Arguments:** no arguments

**Returns:** FLOAT

**Examples:**
```sql
SELECT MASS_MOON()
SELECT satellite_mass / MASS_MOON() AS lunar_masses
```

### MASS_NEPTUNE()

**Description:** Returns Neptune's mass in kg (1.024 × 10^26)

**Arguments:** no arguments

**Returns:** FLOAT

**Examples:**
```sql
SELECT MASS_NEPTUNE()
```

### MASS_SATURN()

**Description:** Returns Saturn's mass in kg (5.683 × 10^26)

**Arguments:** no arguments

**Returns:** FLOAT

**Examples:**
```sql
SELECT MASS_SATURN()
```

### MASS_SUN()

**Description:** Returns the Sun's mass in kg (1.989 × 10^30)

**Arguments:** no arguments

**Returns:** FLOAT

**Examples:**
```sql
SELECT MASS_SUN()
SELECT star_mass / MASS_SUN() AS solar_masses
```

### MASS_URANUS()

**Description:** Returns Uranus's mass in kg (8.681 × 10^25)

**Arguments:** no arguments

**Returns:** FLOAT

**Examples:**
```sql
SELECT MASS_URANUS()
```

### MASS_VENUS()

**Description:** Returns Venus's mass in kg (4.867 × 10^24)

**Arguments:** no arguments

**Returns:** FLOAT

**Examples:**
```sql
SELECT MASS_VENUS()
```

### PARSEC()

**Description:** Returns one parsec in meters (3.086 × 10^16)

**Arguments:** no arguments

**Returns:** FLOAT

**Examples:**
```sql
SELECT PARSEC()
SELECT galaxy_distance / PARSEC() AS distance_pc
```

### RADIUS_EARTH()

**Description:** Returns Earth's radius in meters (6.371 × 10^6)

**Arguments:** no arguments

**Returns:** FLOAT

**Examples:**
```sql
SELECT RADIUS_EARTH()
SELECT planet_radius / RADIUS_EARTH() AS earth_radii
```

### RADIUS_JUPITER()

**Description:** Returns Jupiter's radius in meters (6.991 × 10^7)

**Arguments:** no arguments

**Returns:** FLOAT

**Examples:**
```sql
SELECT RADIUS_JUPITER()
SELECT exoplanet_radius / RADIUS_JUPITER() AS jupiter_radii
```

### RADIUS_MARS()

**Description:** Returns Mars's radius in meters (3.390 × 10^6)

**Arguments:** no arguments

**Returns:** FLOAT

**Examples:**
```sql
SELECT RADIUS_MARS()
```

### RADIUS_MERCURY()

**Description:** Returns Mercury's radius in meters (2.440 × 10^6)

**Arguments:** no arguments

**Returns:** FLOAT

**Examples:**
```sql
SELECT RADIUS_MERCURY()
```

### RADIUS_MOON()

**Description:** Returns the Moon's radius in meters (1.737 × 10^6)

**Arguments:** no arguments

**Returns:** FLOAT

**Examples:**
```sql
SELECT RADIUS_MOON()
SELECT satellite_radius / RADIUS_MOON() AS lunar_radii
```

### RADIUS_NEPTUNE()

**Description:** Returns Neptune's radius in meters (2.462 × 10^7)

**Arguments:** no arguments

**Returns:** FLOAT

**Examples:**
```sql
SELECT RADIUS_NEPTUNE()
```

### RADIUS_SATURN()

**Description:** Returns Saturn's radius in meters (5.823 × 10^7)

**Arguments:** no arguments

**Returns:** FLOAT

**Examples:**
```sql
SELECT RADIUS_SATURN()
```

### RADIUS_SUN()

**Description:** Returns the Sun's radius in meters (6.96 × 10^8)

**Arguments:** no arguments

**Returns:** FLOAT

**Examples:**
```sql
SELECT RADIUS_SUN()
SELECT star_radius / RADIUS_SUN() AS solar_radii
```

### RADIUS_URANUS()

**Description:** Returns Uranus's radius in meters (2.536 × 10^7)

**Arguments:** no arguments

**Returns:** FLOAT

**Examples:**
```sql
SELECT RADIUS_URANUS()
```

### RADIUS_VENUS()

**Description:** Returns Venus's radius in meters (6.052 × 10^6)

**Arguments:** no arguments

**Returns:** FLOAT

**Examples:**
```sql
SELECT RADIUS_VENUS()
```

## Chemical Functions

### ATOMIC_MASS()

**Description:** Returns the atomic mass of an element or molecular formula in amu

**Arguments:** 1 argument

**Returns:** FLOAT

**Examples:**
```sql
SELECT ATOMIC_MASS('H')
SELECT ATOMIC_MASS('Carbon')
SELECT ATOMIC_MASS('H2O') AS water_mass
SELECT ATOMIC_MASS('Ca(OH)2') AS calcium_hydroxide
SELECT ATOMIC_MASS('water') AS water_mass
```

### ATOMIC_NUMBER()

**Description:** Returns the atomic number of an element

**Arguments:** 1 argument

**Returns:** INTEGER

**Examples:**
```sql
SELECT ATOMIC_NUMBER('H')
SELECT ATOMIC_NUMBER('Carbon')
SELECT ATOMIC_NUMBER('Au') AS gold_number
```

### AVOGADRO()

**Description:** Returns Avogadro's number (6.022 × 10^23)

**Arguments:** no arguments

**Returns:** FLOAT

**Examples:**
```sql
SELECT AVOGADRO()
SELECT molecules / AVOGADRO() AS moles
```

## Constant Functions

### E()

**Description:** Returns Euler's number (e ≈ 2.71828)

**Arguments:** no arguments

**Returns:** FLOAT

**Examples:**
```sql
SELECT E()
SELECT POW(E(), x) AS exp_x
```

### MASS_ELECTRON()

**Description:** Alias for ME() - Returns the mass of an electron in kg

**Arguments:** no arguments

**Returns:** FLOAT

**Examples:**
```sql
SELECT MASS_ELECTRON()
```

### ME()

**Description:** Returns the mass of an electron in kg (9.10938356 × 10^-31)

**Arguments:** no arguments

**Returns:** FLOAT

**Examples:**
```sql
SELECT ME()
SELECT mass / ME() AS electron_masses
```

### PI()

**Description:** Returns the value of π (pi)

**Arguments:** no arguments

**Returns:** FLOAT

**Examples:**
```sql
SELECT PI()
SELECT radius * 2 * PI() AS circumference
```

## Mathematical Functions

### COALESCE()

**Description:** Returns the first non-null value from a list

**Arguments:** any number of arguments

**Returns:** ANY

**Examples:**
```sql
SELECT COALESCE(NULL, 'default', 'backup')
SELECT COALESCE(phone, mobile, email) as contact FROM users
SELECT COALESCE(discount, 0) as discount_amount
```

### GREATEST()

**Description:** Returns the greatest value from a list of values

**Arguments:** any number of arguments

**Returns:** ANY

**Examples:**
```sql
SELECT GREATEST(10, 20, 5)
SELECT GREATEST(salary, bonus, commission) as max_pay FROM employees
SELECT GREATEST('apple', 'banana', 'cherry')
SELECT GREATEST(date1, date2, date3) as latest_date
```

### GREATEST_LABEL()

**Description:** Returns the label associated with the greatest value from label/value pairs

**Arguments:** any number of arguments

**Returns:** STRING

**Examples:**
```sql
SELECT GREATEST_LABEL('earth', MASS_EARTH(), 'sun', MASS_SUN()) as bigger_body
SELECT GREATEST_LABEL('jan', 100, 'feb', 150, 'mar', 120) as best_month
SELECT GREATEST_LABEL('product_a', sales_a, 'product_b', sales_b) as top_product
```

### IIF()

**Description:** Returns second argument if first is true, third if false

**Arguments:** 3 arguments

**Returns:** ANY

**Examples:**
```sql
SELECT IIF(1 > 0, 'positive', 'negative')
SELECT IIF(MASS_SUN() > MASS_EARTH(), 'sun', 'earth') as bigger
SELECT IIF(price > 100, 'expensive', 'affordable') as price_category
```

### IS_PRIME()

**Description:** Returns true if the number is prime, false otherwise

**Arguments:** 1 argument

**Returns:** BOOLEAN

**Examples:**
```sql
SELECT IS_PRIME(17)
SELECT IS_PRIME(100)
SELECT IS_PRIME(104729)
```

### LEAST()

**Description:** Returns the smallest value from a list of values

**Arguments:** any number of arguments

**Returns:** ANY

**Examples:**
```sql
SELECT LEAST(10, 20, 5)
SELECT LEAST(salary, min_wage) as lower_bound FROM employees
SELECT LEAST('apple', 'banana', 'cherry')
SELECT LEAST(date1, date2, date3) as earliest_date
```

### LEAST_LABEL()

**Description:** Returns the label associated with the smallest value from label/value pairs

**Arguments:** any number of arguments

**Returns:** STRING

**Examples:**
```sql
SELECT LEAST_LABEL('mercury', MASS_MERCURY(), 'earth', MASS_EARTH()) as smaller_planet
SELECT LEAST_LABEL('jan', 100, 'feb', 150, 'mar', 120) as worst_month
SELECT LEAST_LABEL('cost_a', 50, 'cost_b', 30) as cheapest_option
```

### NEXT_PRIME()

**Description:** Returns the smallest prime number >= n

**Arguments:** 1 argument

**Returns:** INTEGER

**Examples:**
```sql
SELECT NEXT_PRIME(100)
SELECT NEXT_PRIME(97)
SELECT NEXT_PRIME(1000)
```

### NULLIF()

**Description:** Returns NULL if two values are equal, otherwise returns the first value

**Arguments:** 2 arguments

**Returns:** ANY

**Examples:**
```sql
SELECT NULLIF(0, 0)
SELECT NULLIF(price, 0) as non_zero_price
SELECT NULLIF(status, 'DELETED') as active_status
```

### PREV_PRIME()

**Description:** Returns the largest prime number <= n

**Arguments:** 1 argument

**Returns:** INTEGER

**Examples:**
```sql
SELECT PREV_PRIME(100)
SELECT PREV_PRIME(97)
SELECT PREV_PRIME(1000)
```

### PRIME()

**Description:** Returns the Nth prime number (1-indexed)

**Arguments:** 1 argument

**Returns:** INTEGER

**Examples:**
```sql
SELECT PRIME(1)
SELECT PRIME(100)
SELECT PRIME(10000)
```

### PRIME_COUNT()

**Description:** Returns the count of prime numbers up to n (π(n))

**Arguments:** 1 argument

**Returns:** INTEGER

**Examples:**
```sql
SELECT PRIME_COUNT(10)
SELECT PRIME_COUNT(100)
SELECT PRIME_COUNT(1000)
```

## String Functions

### CONTAINS()

**Description:** Checks if string contains substring

**Arguments:** 2 arguments

**Returns:** BOOLEAN

**Examples:**
```sql
SELECT * FROM users WHERE name.Contains('john')
SELECT CONTAINS(name, 'john') FROM users
```

### ENDSWITH()

**Description:** Checks if string ends with suffix

**Arguments:** 2 arguments

**Returns:** BOOLEAN

**Examples:**
```sql
SELECT * FROM users WHERE email.EndsWith('.com')
SELECT ENDSWITH(email, '.com') FROM users
```

### LENGTH()

**Description:** Returns the length of a string

**Arguments:** 1 argument

**Returns:** INTEGER

**Examples:**
```sql
SELECT name.Length() FROM users
SELECT LENGTH(name) FROM users
```

### REPLACE()

**Description:** Replaces all occurrences of a substring

**Arguments:** 3 arguments

**Returns:** STRING

**Examples:**
```sql
SELECT name.Replace('John', 'Jane') FROM users
SELECT REPLACE(name, 'John', 'Jane') FROM users
```

### STARTSWITH()

**Description:** Checks if string starts with prefix

**Arguments:** 2 arguments

**Returns:** BOOLEAN

**Examples:**
```sql
SELECT * FROM users WHERE name.StartsWith('John')
SELECT STARTSWITH(name, 'John') FROM users
```

### SUBSTRING()

**Description:** Extracts substring from string

**Arguments:** 2 to 3 arguments

**Returns:** STRING

**Examples:**
```sql
SELECT name.Substring(0, 5) FROM users
SELECT SUBSTRING(name, 0, 5) FROM users
```

### TOLOWER()

**Description:** Converts string to lowercase

**Arguments:** 1 argument

**Returns:** STRING

**Examples:**
```sql
SELECT name.ToLower() FROM users
SELECT TOLOWER(name) FROM users
```

### TOUPPER()

**Description:** Converts string to uppercase

**Arguments:** 1 argument

**Returns:** STRING

**Examples:**
```sql
SELECT name.ToUpper() FROM users
SELECT TOUPPER(name) FROM users
```

### TRIM()

**Description:** Removes leading and trailing whitespace

**Arguments:** 1 argument

**Returns:** STRING

**Examples:**
```sql
SELECT name.Trim() FROM users
SELECT TRIM(name) FROM users
```

