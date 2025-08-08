/ Load trades.json data into kdb+
/ Run this script in your q session to set up the test data

/ Load JSON library (if not already loaded)
\l json.k

/ Function to load JSON file
loadJSON:{[filename]
    data:.j.k raze read0 hsym `$filename;
    data
    }

/ Load the trades.json file
/ Adjust path as needed for your environment
tradesJSON:loadJSON["../data/trades.json"];

/ Convert JSON data to a kdb+ table
/ First, let's examine the structure
show "Sample record structure:";
show first tradesJSON;

/ Create the trades table with proper types
/ Note: Adjust column types based on your actual data
trades:flip `id`platformOrderId`tradeDate`executionSide`quantity`price`counterparty`counterpartyCountry`commission`status`confirmationStatus`trader`traderId`createdDate`settlementDate`instrumentName`instrumentType`book`desk`portfolio!
    (`long$tradesJSON[;`id];
     `$tradesJSON[;`platformOrderId];
     "D"$tradesJSON[;`tradeDate];
     `$tradesJSON[;`executionSide];
     `float$tradesJSON[;`quantity];
     `float$tradesJSON[;`price];
     `$tradesJSON[;`counterparty];
     `$tradesJSON[;`counterpartyCountry];
     `float$tradesJSON[;`commission];
     `$tradesJSON[;`status];
     `$tradesJSON[;`confirmationStatus];
     `$tradesJSON[;`trader];
     `$tradesJSON[;`traderId];
     "D"$tradesJSON[;`createdDate];
     "D"$tradesJSON[;`settlementDate];
     `$tradesJSON[;`instrumentName];
     `$tradesJSON[;`instrumentType];
     `$tradesJSON[;`book];
     `$tradesJSON[;`desk];
     `$tradesJSON[;`portfolio]);

/ Display table info
show "Trades table loaded:";
show count trades;
show " records";
show "Columns: ", cols trades;
show "First 5 records:";
show 5#trades;

/ Save table for persistence (optional)
`:trades/ set trades;
show "Table saved to trades/";

/ Example queries matching our test cases
show "";
show "========================================";
show "TEST QUERIES - Matching C# Translation";
show "========================================";

/ Test 1: Simple case-insensitive equality
show "";
show "Test 1: Find all pending trades (case-insensitive)";
show "SQL: SELECT * FROM trades WHERE confirmationStatus = 'pending'";
show "q translation:";
pendingTrades:select from trades where lower[confirmationStatus]=lower[`pending];
show count[pendingTrades], " records found";
show 3#pendingTrades;

/ Test 2: Complex query with multiple conditions
show "";
show "Test 2: Complex WHERE clause";
show "SQL: WHERE confirmationStatus.StartsWith('pend') AND commission BETWEEN 30 AND 100";
show "q translation:";
complexResult:select from trades where 
    (lower[confirmationStatus] like lower["pend*"]),
    (commission within 30 100);
show count[complexResult], " records found";
show 3#complexResult;

/ Test 3: IN clause (case-insensitive)
show "";
show "Test 3: IN clause for multiple statuses";
show "SQL: WHERE status IN ('Active', 'Pending', 'Confirmed')";
show "q translation:";
inResult:select from trades where lower[status] in lower[`Active`Pending`Confirmed];
show count[inResult], " records found";

/ Test 4: Date comparison
show "";
show "Test 4: Date range query";
show "SQL: WHERE createdDate > '2025-01-01'";
show "q translation:";
/ Note: Date format in q is yyyy.mm.dd
dateResult:select from trades where createdDate>2025.01.01;
show count[dateResult], " records found";

/ Test 5: String contains (case-insensitive)
show "";
show "Test 5: String contains";
show "SQL: WHERE counterparty.Contains('Bank')";
show "q translation:";
/ In q, we use ss (string search) or like for pattern matching
containsResult:select from trades where counterparty like "*Bank*";
show count[containsResult], " records found";

/ Helper functions for case-insensitive operations
/ These would be used by the proxy

/ Case-insensitive equality
ciEqual:{[col;val] lower[col]=lower[val]};

/ Case-insensitive IN
ciIn:{[col;vals] lower[col] in lower[vals]};

/ Case-insensitive LIKE
ciLike:{[col;pattern] lower[col] like lower[pattern]};

show "";
show "Helper functions defined: ciEqual, ciIn, ciLike";

/ Performance comparison
show "";
show "Performance Comparison:";
show "----------------------";

/ Case-sensitive query
\t sensitiveResult:select from trades where confirmationStatus=`Pending;

/ Case-insensitive query
\t insensitiveResult:select from trades where lower[confirmationStatus]=lower[`pending];

show "Case-sensitive found: ", count[sensitiveResult];
show "Case-insensitive found: ", count[insensitiveResult];

show "";
show "Setup complete! You can now test queries.";
show "Example: select from trades where commission>50";