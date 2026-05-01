#!/bin/bash
set -e
cd "$(dirname "$0")"

# Run criterion bench and extract mean times for each workload
OUTPUT=$(cargo bench -p iratxo-core --bench engine 2>&1)

KEYWORD_US=$(echo "$OUTPUT" | grep -A2 'keyword/short' | grep 'time:' | sed -E 's/.*\[([0-9.]+) µs.*/\1/' | head -1)
REGEX_US=$(echo "$OUTPUT" | grep -A2 'regex/5kb_doc_6_rules' | grep 'time:' | sed -E 's/.*\[([0-9.]+) µs.*/\1/' | head -1)
SEMANTIC_US=$(echo "$OUTPUT" | grep -A2 'semantic/paragraph' | grep 'time:' | sed -E 's/.*\[([0-9.]+) µs.*/\1/' | head -1)

# If any are in ns instead of µs, convert
if echo "$OUTPUT" | grep -A2 'keyword/short' | grep 'time:' | grep -q 'ns'; then
    KEYWORD_US=$(echo "$OUTPUT" | grep -A2 'keyword/short' | grep 'time:' | sed -E 's/.*\[([0-9.]+) ns.*/\1/' | head -1)
    KEYWORD_US=$(python3 -c "print($KEYWORD_US / 1000.0)")
fi
if echo "$OUTPUT" | grep -A2 'regex/5kb_doc_6_rules' | grep 'time:' | grep -q 'ns'; then
    REGEX_US=$(echo "$OUTPUT" | grep -A2 'regex/5kb_doc_6_rules' | grep 'time:' | sed -E 's/.*\[([0-9.]+) ns.*/\1/' | head -1)
    REGEX_US=$(python3 -c "print($REGEX_US / 1000.0)")
fi
if echo "$OUTPUT" | grep -A2 'semantic/paragraph' | grep 'time:' | grep -q 'ns'; then
    SEMANTIC_US=$(echo "$OUTPUT" | grep -A2 'semantic/paragraph' | grep 'time:' | sed -E 's/.*\[([0-9.]+) ns.*/\1/' | head -1)
    SEMANTIC_US=$(python3 -c "print($SEMANTIC_US / 1000.0)")
fi

echo "keyword_us=$KEYWORD_US"
echo "regex_us=$REGEX_US"
echo "semantic_us=$SEMANTIC_US"

TOTAL_US=$(python3 -c "print($KEYWORD_US + $REGEX_US + $SEMANTIC_US)")
echo "METRIC total_µs=$TOTAL_US"
echo "METRIC keyword_µs=$KEYWORD_US"
echo "METRIC regex_µs=$REGEX_US"
echo "METRIC semantic_µs=$SEMANTIC_US"
