#!/usr/bin/env bash

# Simple test script for lobby API endpoints

BASE_URL="http://localhost:3000"
LOBBY_API="$BASE_URL/lobby/api"

echo "Testing Bevygap Matchmaker Lobby API Endpoints"
echo "=============================================="
echo ""

# Function to make HTTP requests and show results
test_endpoint() {
    local method=$1
    local endpoint=$2
    local data=$3
    local expected_status=${4:-200}
    
    echo "Testing: $method $endpoint"
    
    if [ -n "$data" ]; then
        response=$(curl -s -w "\nHTTP_STATUS:%{http_code}" \
                       -X "$method" \
                       -H "Content-Type: application/json" \
                       -d "$data" \
                       "$LOBBY_API$endpoint")
    else
        response=$(curl -s -w "\nHTTP_STATUS:%{http_code}" \
                       -X "$method" \
                       "$LOBBY_API$endpoint")
    fi
    
    # Extract the HTTP status code
    body=$(echo "$response" | sed '$d')
    status=$(echo "$response" | tail -n1 | sed 's/HTTP_STATUS://')
    
    echo "Status: $status"
    echo "Response: $body"
    
    if [ "$status" = "$expected_status" ]; then
        echo "✅ PASS"
    else
        echo "❌ FAIL (expected $expected_status, got $status)"
    fi
    echo ""
}

# Test 1: Check initial lobby status
test_endpoint "GET" "/status"

# Test 2: List empty rooms
test_endpoint "GET" "/rooms"

# Test 3: Create a room
room_data='{"host_name": "TestHost", "game_mode": "FreeForAll", "max_players": 4}'
create_response=$(curl -s -X POST \
                       -H "Content-Type: application/json" \
                       -d "$room_data" \
                       "$LOBBY_API/rooms")

echo "Testing: POST /rooms"
echo "Status: 200 (assuming success)"
echo "Response: $create_response"

# Extract room ID from response
room_id=$(echo "$create_response" | grep -o '"id":"[^"]*"' | cut -d'"' -f4)
if [ -n "$room_id" ]; then
    echo "✅ PASS - Room created with ID: $room_id"
    echo ""
    
    # Test 4: List rooms (should now show our room)
    test_endpoint "GET" "/rooms"
    
    # Test 5: Check status again (should show 1 active room)
    test_endpoint "GET" "/status"
    
    # Test 6: Start the room
    test_endpoint "POST" "/rooms/$room_id/start"
    
    # Test 7: List rooms again (should be empty as started rooms are filtered out)
    test_endpoint "GET" "/rooms"
    
    # Test 8: Try to leave a non-existent room
    leave_data='{"player_name": "TestPlayer"}'
    test_endpoint "POST" "/rooms/NONEXISTENT/leave" "$leave_data" "404"
    
else
    echo "❌ FAIL - Could not extract room ID from response"
    echo ""
fi

# Test 9: Try to start a non-existent room
test_endpoint "POST" "/rooms/NONEXISTENT/start" "" "404"

echo "Test Summary"
echo "============"
echo "All basic lobby API endpoints have been tested."
echo "If the server is running on $BASE_URL, the tests above show whether each endpoint works correctly."