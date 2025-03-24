#!/bin/bash
set -e

echo "🧪 Running DuckTape Integration Tests"

# Start services in the background
echo "Starting services..."
docker-compose up -d

# Wait for services to be ready
echo "Waiting for services to be ready..."
for i in {1..30}; do
    if curl -s http://localhost:3000/health > /dev/null; then
        echo "Services are ready!"
        break
    fi
    if [ $i -eq 30 ]; then
        echo "Services failed to start"
        docker-compose logs
        docker-compose down
        exit 1
    fi
    echo "Waiting... ($i/30)"
    sleep 1
done

# Run API tests
echo "Running API tests..."
cargo test --test '*' -- --test-threads=1

# Run WebSocket tests
echo "Running WebSocket tests..."
cargo test --test 'websocket_*' -- --test-threads=1

# Test calendar integration
echo "Testing calendar integration..."
cargo test --test 'calendar_*' -- --test-threads=1

# Test natural language processing
echo "Testing natural language processing..."
cargo test --test 'nlp_*' -- --test-threads=1

# Test security features
echo "Testing security features..."
cargo test --test 'security_*' -- --test-threads=1

# Test API key management
echo "Testing API key management..."
cargo test --test 'api_keys_*' -- --test-threads=1

# Run end-to-end calendar flow test
echo "Running end-to-end calendar flow test..."
curl -X POST http://localhost:3000/api/calendar/event \
    -H "Content-Type: application/json" \
    -H "Authorization: Bearer test_token" \
    -d '{
        "command": "schedule meeting tomorrow at 2pm",
        "timezone": "America/New_York"
    }'

# Test WebSocket connection
echo "Testing WebSocket connection..."
wscat -c ws://localhost:3000/ws \
    --execute 'echo "{\"type\":\"ping\",\"payload\":{}}"' \
    --timeout 5

# Clean up
echo "Cleaning up..."
docker-compose down

echo "✅ Integration tests completed!"