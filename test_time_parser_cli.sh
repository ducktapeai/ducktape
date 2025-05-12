#!/bin/bash
# Test script for Ducktape time parser integration
# This script runs the ducktape CLI with different time expressions

# ANSI color codes
GREEN='\033[0;32m'
RED='\033[0;31m'
BLUE='\033[0;34m'
RESET='\033[0m'

echo -e "${BLUE}=== Testing Ducktape Time Parser Integration ===${RESET}"
echo "This script will execute several natural language commands to test time parsing"
echo

# Function to run a test command
run_test() {
    local command="$1"
    local expected="$2"
    
    echo -e "${BLUE}Testing:${RESET} $command"
    echo -e "${BLUE}Expecting:${RESET} $expected in the output"
    
    # Run the command with --print flag to see what command would be executed without actually executing it
    result=$(ducktape --print "$command" 2>&1)
    echo -e "${BLUE}Result:${RESET} $result"
    
    # Check if the result contains the expected time
    if echo "$result" | grep -q "$expected"; then
        echo -e "${GREEN}✓ PASS: Found expected time format${RESET}"
    else
        echo -e "${RED}✗ FAIL: Did not find expected time format${RESET}"
    fi
    echo
}

# Run test cases
run_test "create an event called Team Meeting tonight at 7pm" "19:00"
run_test "schedule a meeting called Review at 3:30pm" "15:30"
run_test "create an event called Breakfast at 9am" "09:00"
run_test "create an event called Lunch at 12pm" "12:00"
run_test "create an event called Midnight Party at 12am" "00:00"

echo -e "${BLUE}Tests complete${RESET}"
