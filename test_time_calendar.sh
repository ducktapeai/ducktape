#!/bin/bash
# Test script for time parser and calendar detection

# ANSI color codes
GREEN='\033[0;32m'
RED='\033[0;31m'
BLUE='\033[0;34m'
RESET='\033[0m'

echo -e "${BLUE}=== Testing Ducktape Calendar and Time Parser ===${RESET}"
echo

# Test case 1: Tomorrow at specific time with specific calendar
echo -e "${BLUE}Test Case 1: Tomorrow at specific time with specific calendar${RESET}"
echo "Running: ./target/debug/ducktape ai create an event in the KIDS calendar for tomorrow at 4pm called Test Event"
output=$(./target/debug/ducktape ai create an event in the KIDS calendar for tomorrow at 4pm called Test Event 2>&1)
echo "$output" | grep -q 'KIDS'
cal_result=$?
echo "$output" | grep -q "$(date -v+1d '+%Y-%m-%d')"
date_result=$?
echo "$output" | grep -q '16:00'
time_result=$?

if [ $cal_result -eq 0 ] && [ $date_result -eq 0 ] && [ $time_result -eq 0 ]; then
    echo -e "${GREEN}✓ Test passed: Calendar, date and time correctly processed${RESET}"
else
    echo -e "${RED}✗ Test failed:${RESET}"
    [ $cal_result -ne 0 ] && echo -e "${RED}  - Calendar 'KIDS' not found in output${RESET}"
    [ $date_result -ne 0 ] && echo -e "${RED}  - Tomorrow's date not found in output${RESET}"
    [ $time_result -ne 0 ] && echo -e "${RED}  - Time '16:00' not found in output${RESET}"
fi
echo

# Test case 2: Today at specific time with default calendar
echo -e "${BLUE}Test Case 2: Today at specific time with default calendar${RESET}"
echo "Running: ./target/debug/ducktape ai schedule a meeting at 3:30pm called Afternoon Review"
output=$(./target/debug/ducktape ai schedule a meeting at 3:30pm called Afternoon Review 2>&1)
echo "$output" | grep -q "$(date '+%Y-%m-%d')"
date_result=$?
echo "$output" | grep -q '15:30'
time_result=$?

if [ $date_result -eq 0 ] && [ $time_result -eq 0 ]; then
    echo -e "${GREEN}✓ Test passed: Date and time correctly processed${RESET}"
else
    echo -e "${RED}✗ Test failed:${RESET}"
    [ $date_result -ne 0 ] && echo -e "${RED}  - Today's date not found in output${RESET}"
    [ $time_result -ne 0 ] && echo -e "${RED}  - Time '15:30' not found in output${RESET}"
fi
echo

# Test case 3: Tomorrow morning
echo -e "${BLUE}Test Case 3: Tomorrow morning${RESET}"
echo "Running: ./target/debug/ducktape ai create an event for tomorrow at 9am called Morning Meeting"
output=$(./target/debug/ducktape ai create an event for tomorrow at 9am called Morning Meeting 2>&1)
echo "$output" | grep -q "$(date -v+1d '+%Y-%m-%d')"
date_result=$?
echo "$output" | grep -q '09:00'
time_result=$?

if [ $date_result -eq 0 ] && [ $time_result -eq 0 ]; then
    echo -e "${GREEN}✓ Test passed: Tomorrow's date and morning time correctly processed${RESET}"
else
    echo -e "${RED}✗ Test failed:${RESET}"
    [ $date_result -ne 0 ] && echo -e "${RED}  - Tomorrow's date not found in output${RESET}"
    [ $time_result -ne 0 ] && echo -e "${RED}  - Time '09:00' not found in output${RESET}"
fi
echo

echo -e "${BLUE}=== Test Summary ===${RESET}"
echo "These tests verify the time parser can correctly handle:"
echo "1. Relative dates (tomorrow, today)"
echo "2. Times in AM/PM format"
echo "3. Specific calendar designations"
