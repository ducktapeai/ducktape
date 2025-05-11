#!/bin/bash
# Automated timezone testing script for Ducktape
# This script runs a series of commands through Ducktape and verifies timezone handling

# ANSI color codes
GREEN='\033[0;32m'
RED='\033[0;31m'
BLUE='\033[0;34m'
YELLOW='\033[0;33m'
RESET='\033[0m'

echo -e "${BLUE}=== Ducktape Timezone Integration Test ===${RESET}"
echo "This script will test timezone handling in the Ducktape application"
echo

# Determine local timezone
LOCAL_TZ=$(date +%Z)
LOCAL_TIME=$(date +"%H:%M")

echo -e "${BLUE}Current local timezone: ${YELLOW}$LOCAL_TZ${RESET}"
echo -e "${BLUE}Current local time: ${YELLOW}$LOCAL_TIME${RESET}"
echo

# Helper function to extract time from Ducktape log output
extract_time_from_log() {
    local log_file=$1
    grep -o '[0-9]\{2\}:[0-9]\{2\}' "$log_file" | head -1
}

# Helper function to run a test case
run_test_case() {
    local test_name=$1
    local command=$2
    local expected_behavior=$3
    
    echo -e "${YELLOW}Test: ${test_name}${RESET}"
    echo "Command: $command"
    echo "Expected: $expected_behavior"
    
    # Create a temporary log file
    local log_file=$(mktemp)
    
    # Run the command and capture output
    RUST_LOG=debug ./target/debug/ducktape ai "$command" > "$log_file" 2>&1 || true
    
    # Display the important parts of the output
    echo "Output:"
    grep -E 'Processing|Sanitized|INFO:' "$log_file" | sed 's/^/  /'
    
    # Extract the processed time
    local processed_time=$(grep -o '[0-9]\{2\}:[0-9]\{2\}' "$log_file" | head -2 | tail -1)
    echo -e "${GREEN}Processed time: $processed_time${RESET}"
    
    # Clean up
    rm -f "$log_file"
    echo ""
}

# Run a series of test cases
echo -e "${BLUE}Running timezone test cases...${RESET}"
echo "------------------------------------------------"

run_test_case "PST Timezone" \
    "schedule a meeting at 9pm PST called West Coast Sync" \
    "Should convert 9pm PST to the appropriate local time"

run_test_case "EST Timezone" \
    "schedule a call at 2pm EST called East Coast Checkin" \
    "Should convert 2pm EST to the appropriate local time"

run_test_case "GMT Timezone" \
    "set up a meeting for 10am GMT with international team" \
    "Should convert 10am GMT to the appropriate local time"

run_test_case "JST Timezone" \
    "create an event at 8am JST called Tokyo Team Sync" \
    "Should convert 8am JST to the appropriate local time"

run_test_case "No Timezone" \
    "schedule a meeting at 3pm called Local Office Hours" \
    "Should interpret 3pm as local time without conversion"

echo -e "${BLUE}All test cases completed.${RESET}"
echo
echo -e "${YELLOW}Note: Verify that the times are converted correctly based on your local timezone.${RESET}"
echo "If times are not being converted correctly, check the timezone implementation in the time parser."
