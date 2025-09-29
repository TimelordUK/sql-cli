-- Example: Testing SQL formatter with WEB CTE and JSON pretty-printing
-- This file demonstrates how the formatter handles compact JSON in FORM_FIELD values

WITH
    WEB fix_custom AS (
        URL 'http://localhost:5050/api/messagequery/upload' METHOD POST FORMAT CSV
        FORM_FILE 'file' 'data/fix_messages.json'
       FORM_FIELD 'query' '{"MessageTypeField":"header.MsgType","MessageTypes":{"8":{"Select":{"msg_type":"header.MsgType","price":"body.LastPx","quantity":"body.LastQty","side":"body.Side","symbol":"body.Symbol","trader":"Parties[PartyRole=11].PartyID"},"Where":{"body.ExecType":"F"}},"J":{"Select":{"msg_type":"header.MsgType","price":"body.AvgPx","quantity":"body.Quantity","side":"body.Side","symbol":"body.Symbol","trader":"null"}}},"OutputColumns":["msg_type","symbol","side","quantity","price","trader"],"OutputFormat":"csv"}'
    )
SELECT
    msg_type,
    CASE
        WHEN msg_type = '8' THEN 'Execution'
        WHEN msg_type = 'J' THEN 'Allocation'
    END AS message_type,
    symbol,
    quantity,
    price,
    trader
FROM fix_custom
WHERE quantity > 500
ORDER BY price DESC;