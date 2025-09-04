SELECT
    CASE
      WHEN MASS_SUN() > MASS_EARTH() THEN 'sun'
      ELSE 'earth'
    END as bigger_body

