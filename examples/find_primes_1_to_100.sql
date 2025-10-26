-- #! data/numbers_1_to_100.csv
with is_prime as
  (
    select
      n as n,
      is_prime(n) as n_prime 
    from numbers
  ) 
  select n,n_prime 
    from is_prime 
    where n_prime = true