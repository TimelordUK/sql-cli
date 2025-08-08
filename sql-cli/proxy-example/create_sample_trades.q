/ Create sample trades data for testing SQL to q translation
/ This creates a trades table similar to your JSON structure

/ Create sample trades table with mixed case values for testing
trades:([] 
    id:1 2 3 4 5 6 7 8 9 10;
    platformOrderId:`ORDER001`ORDER002`ORDER003`ORDER004`ORDER005`ORDER006`ORDER007`ORDER008`ORDER009`ORDER010;
    confirmationStatus:`Pending`pending`PENDING`Confirmed`confirmed`Pending`Active`pending`Settled`Pending;
    status:`Active`Active`Pending`Completed`Active`Pending`Active`Active`Completed`Pending;
    commission:45.5 120.0 75.25 30.0 95.5 55.0 200.0 85.5 40.0 65.75;
    quantity:1000 500 750 2000 1500 800 1200 900 1100 1300;
    price:150.50 155.25 148.75 152.00 149.50 151.75 153.25 150.00 154.50 149.25;
    trader:`$("John Smith";"Jane Doe";"Bob Johnson";"Alice Brown";"John Smith";"Jane Doe";"Charlie Wilson";"John Smith";"Diana Prince";"Bob Johnson");
    traderId:`TR001`TR002`TR003`TR004`TR001`TR002`TR005`TR001`TR006`TR003;
    counterparty:`$("Bank of America";"Goldman Sachs";"JP Morgan";"Morgan Stanley";"Bank of America";"Citibank";"Wells Fargo";"Bank of America";"Deutsche Bank";"JP Morgan");
    counterpartyCountry:`US`US`US`US`US`US`US`US`DE`US;
    createdDate:2025.07.01 2025.07.05 2025.07.08 2025.07.10 2025.07.12 2025.07.15 2025.07.18 2025.07.20 2025.07.22 2025.07.25;
    settlementDate:2025.07.03 2025.07.07 2025.07.10 2025.07.12 2025.07.14 2025.07.17 2025.07.20 2025.07.22 2025.07.24 2025.07.27;
    instrumentName:`$("AAPL BOND";"MSFT EQUITY";"GOOGL BOND";"AMZN EQUITY";"TSLA BOND";"FB EQUITY";"NFLX BOND";"AAPL EQUITY";"MSFT BOND";"GOOGL EQUITY");
    instrumentType:`BOND`EQUITY`BOND`EQUITY`BOND`EQUITY`BOND`EQUITY`BOND`EQUITY;
    book:`Book1`Book2`Book1`Book3`Book2`Book1`Book3`Book1`Book2`Book3;
    desk:`Desk1`Desk2`Desk1`Desk3`Desk2`Desk1`Desk3`Desk1`Desk2`Desk3;
    portfolio:`Portfolio1`Portfolio2`Portfolio1`Portfolio3`Portfolio2`Portfolio1`Portfolio3`Portfolio1`Portfolio2`Portfolio3
);

show "Sample trades table created with ", string[count trades], " records";
show "";
show "Table structure:";
meta trades;
show "";
show "First 5 records:";
show 5#trades;

show "";
show "========================================";
show "TESTING SQL TO Q TRANSLATIONS";
show "========================================";

show "";
show "Test 1: Case-Insensitive Equality";
show "SQL: WHERE confirmationStatus = 'pending'";
show "";
show "q (case-sensitive):";
show select from trades where confirmationStatus=`pending;
show "Found: ", string count select from trades where confirmationStatus=`pending;
show "";
show "q (case-insensitive):";
show select from trades where lower[confirmationStatus]=lower[`pending];
show "Found: ", string count select from trades where lower[confirmationStatus]=lower[`pending];

show "";
show "Test 2: Complex Query";
show "SQL: WHERE confirmationStatus.StartsWith('pend') AND commission BETWEEN 30 AND 100 AND createdDate > '2025-07-10'";
show "";
show "q translation:";
result:select from trades where 
    (lower[confirmationStatus] like lower["pend*"]),
    (commission within 30 100),
    (createdDate>2025.07.10);
show result;
show "Found: ", string count result;

show "";
show "Test 3: IN Clause (Case-Insensitive)";
show "SQL: WHERE status IN ('pending', 'active')";
show "";
show "q translation:";
inResult:select from trades where lower[status] in lower[`pending`active];
show inResult;
show "Found: ", string count inResult;

show "";
show "Test 4: NOT EQUAL";
show "SQL: WHERE trader != 'John Smith'";
show "";
show "q translation:";
show select from trades where not trader=`$"John Smith";
show "Found: ", string count select from trades where not trader=`$"John Smith";

show "";
show "Test 5: Multiple Numeric Conditions";
show "SQL: WHERE quantity >= 1000 AND price < 152";
show "";
show "q translation:";
show select from trades where (quantity>=1000),(price<152);
show "Found: ", string count select from trades where (quantity>=1000),(price<152);

show "";
show "Test 6: String Pattern Matching";
show "SQL: WHERE instrumentName.Contains('BOND')";
show "";
show "q translation (using like):";
show select from trades where instrumentName like "*BOND*";
show "Found: ", string count select from trades where instrumentName like "*BOND*";

show "";
show "Test 7: Date Range with BETWEEN";
show "SQL: WHERE settlementDate BETWEEN '2025-07-10' AND '2025-07-20'";
show "";
show "q translation:";
show select from trades where settlementDate within 2025.07.10 2025.07.20;
show "Found: ", string count select from trades where settlementDate within 2025.07.10 2025.07.20;

show "";
show "========================================";
show "PERFORMANCE COMPARISON";
show "========================================";

show "";
show "Case-sensitive vs Case-insensitive for 'pending':";
\t:100 csSensitive:select from trades where confirmationStatus=`pending;
show "Case-sensitive time (100 iterations)";
\t:100 csInsensitive:select from trades where lower[confirmationStatus]=lower[`pending];
show "Case-insensitive time (100 iterations)";

show "";
show "Script complete. Table 'trades' is available for testing.";
show "Try: select from trades where commission>50";