# DuckTape Performance Guide

This guide provides information about DuckTape's performance characteristics, optimization strategies, and monitoring tools.

## Performance Goals

DuckTape aims to achieve the following performance targets:

### Command Processing
- Command parsing: < 5ms per command
- Natural language processing: < 100ms per command
- Calendar operations: < 50ms per operation
- Total command round-trip: < 200ms (95th percentile)

### WebSocket Performance
- Connection establishment: < 100ms
- Message latency: < 50ms
- Support for 10,000+ concurrent connections
- Message throughput: 1000+ messages/second

### Memory Usage
- Base memory footprint: < 50MB
- Per-connection overhead: < 50KB
- Maximum memory usage: < 2GB at 10,000 concurrent users

### API Response Times
- Health check: < 10ms
- Calendar queries: < 100ms
- Event creation: < 150ms
- Batch operations: < 500ms

## Running Benchmarks

Use the provided benchmark script:
```bash
./run-benchmarks.sh
```

This will:
1. Build in release mode
2. Run Criterion.rs benchmarks
3. Generate flamegraphs (if cargo-flamegraph is installed)
4. Show performance changes
5. Open HTML reports

### Benchmark Categories

1. **Command Parsing**
   - Natural language command parsing
   - Reminder command parsing
   - Calendar command parsing

2. **Calendar Operations**
   - Event creation
   - Event search
   - Recurring event expansion

3. **NLP Processing**
   - Command interpretation
   - Date/time parsing
   - Intent recognition

## Performance Best Practices

### Async Operations
- Use `async/await` for I/O operations
- Implement proper connection pooling
- Avoid blocking operations in async contexts

### Memory Management
- Minimize allocations in hot paths
- Use appropriate data structures
- Implement proper cleanup in destructors
- Consider using arena allocators for temporary data

### Calendar Operations
- Cache frequently accessed data
- Use efficient date/time algorithms
- Implement proper indexing for searches
- Batch operations when possible

### WebSocket Performance
- Use proper connection pooling
- Implement heartbeat mechanism
- Handle backpressure appropriately
- Consider using compression for large payloads

## Profiling

### CPU Profiling
```bash
# Install flamegraph
cargo install flamegraph

# Generate flamegraph
cargo flamegraph --bench benchmark
```

### Memory Profiling
```bash
# Install heaptrack
# On macOS:
brew install heaptrack

# Profile memory usage
heaptrack ./target/release/ducktape
```

### Common Performance Issues

1. **Memory Leaks**
   - Check for unclosed resources
   - Verify proper cleanup in drop implementations
   - Monitor long-running operations

2. **CPU Hotspots**
   - Excessive string allocations
   - Inefficient regex usage
   - Unoptimized date calculations

3. **I/O Bottlenecks**
   - Blocking operations in async contexts
   - Excessive calendar API calls
   - Unoptimized file operations

## Performance Monitoring

### Metrics Collection
```rust
// Example metrics collection
use metrics::{counter, gauge, histogram};

// Track command processing time
histogram!("command.processing_time", duration);

// Monitor active connections
gauge!("websocket.active_connections", count);

// Count events processed
counter!("calendar.events_processed").increment(1);
```

### Monitoring Dashboard
- Grafana dashboards available in `monitoring/dashboards`
- Prometheus metrics exposed at `/metrics`
- Custom health checks at `/health`

### Alert Rules
- Response time > 500ms
- Error rate > 1%
- Memory usage > 80%
- CPU usage > 70%

### Logging Performance Data
```rust
debug!("Command processed in {}ms", duration.as_millis());
```

## Optimization Guidelines

1. **Measure First**
   - Always benchmark before optimizing
   - Identify actual bottlenecks
   - Document performance changes

2. **Low-Hanging Fruit**
   - Use appropriate algorithms
   - Optimize data structures
   - Cache frequently accessed data
   - Batch operations when possible

3. **Advanced Optimizations**
   - Consider using SIMD operations
   - Implement custom allocators
   - Use lock-free data structures
   - Profile-guided optimization

### Command Processing
1. Use command caching
```rust
use moka::sync::Cache;

let cache: Cache<String, ParsedCommand> = Cache::builder()
    .max_capacity(10_000)
    .time_to_live(Duration::from_secs(3600))
    .build();
```

2. Implement batch processing
```rust
impl CommandProcessor {
    pub async fn process_batch(&self, commands: Vec<Command>) -> Vec<Result<Response>> {
        stream::iter(commands)
            .map(|cmd| self.process(cmd))
            .buffer_unordered(4)
            .collect()
            .await
    }
}
```

### Calendar Optimization
1. Index optimization
```rust
// Create indexes for common queries
CREATE INDEX idx_calendar_date ON events (start_date, end_date);
CREATE INDEX idx_calendar_user ON events (user_id, start_date);
```

2. Query optimization
```rust
impl CalendarStore {
    pub async fn query_events(&self, range: DateRange) -> Result<Vec<Event>> {
        // Use covering indexes
        sqlx::query!(
            r#"
            SELECT id, title, start_date, end_date
            FROM events
            WHERE start_date >= $1 AND end_date <= $2
            AND user_id = $3
            USE INDEX (idx_calendar_date)
            "#,
            range.start, range.end, user_id
        )
        .fetch_all(&self.pool)
        .await
    }
}
```

### WebSocket Optimization
1. Message batching
```rust
impl WebSocketConnection {
    pub async fn send_batch(&self, messages: Vec<Message>) -> Result<()> {
        let batch = messages.into_iter()
            .map(|msg| msg.into_bytes())
            .collect::<Vec<_>>();
        
        self.tx.send_batch(batch).await
    }
}
```

2. Connection pooling
```rust
impl ConnectionPool {
    pub fn new(capacity: usize) -> Self {
        let pool = deadpool::Pool::builder()
            .max_size(capacity)
            .build();
        
        Self { pool }
    }
}
```

### Memory Optimization
1. Use arena allocation
```rust
use bumpalo::Bump;

struct CommandArena {
    arena: Bump,
}

impl CommandArena {
    pub fn parse_command(&mut self, input: &str) -> &ParsedCommand {
        let parsed = parse_command(input);
        self.arena.alloc(parsed)
    }
}
```

2. Implement memory limits
```rust
impl Config {
    pub fn memory_limits() -> Self {
        Self {
            max_message_size: 64 * 1024,  // 64KB
            max_batch_size: 1000,
            max_connections: 10_000,
        }
    }
}
```

## Benchmarking

### Running Benchmarks
```bash
# Run all benchmarks
./run-benchmarks.sh

# Run specific benchmark
cargo bench --bench websocket_bench
```

### Benchmark Categories
1. Command processing
2. Calendar operations
3. WebSocket performance
4. Memory usage
5. API endpoints

### Profiling Tools
- Flamegraphs: `cargo flamegraph`
- Heap profiling: `heaptrack`
- CPU profiling: `perf`

## Performance Testing

### Load Testing
```bash
# Test WebSocket connections
./bench/websocket-load-test.sh --connections 1000 --duration 300

# Test API endpoints
hey -n 10000 -c 100 http://localhost:3000/api/calendar/query
```

### Stress Testing
```bash
# Run stress test suite
./bench/stress-test.sh --users 10000 --duration 3600
```

### Performance Regression Testing
- Automated tests in CI/CD pipeline
- Baseline comparison
- Performance budget enforcement

## Common Issues and Solutions

### High Command Processing Time
1. Check command cache hit ratio
2. Monitor NLP service performance
3. Optimize regex patterns
4. Use command batching

### WebSocket Performance Issues
1. Enable message compression
2. Implement connection pooling
3. Use binary message format
4. Monitor connection lifecycle

### Memory Leaks
1. Use memory profiling
2. Implement proper cleanup
3. Monitor resource usage
4. Set appropriate limits

### Database Performance
1. Optimize indexes
2. Use query caching
3. Implement connection pooling
4. Monitor query patterns

## Performance Tooling

### Monitoring Tools
- Prometheus
- Grafana
- Custom metrics
- Log analysis

### Profiling Tools
- Flamegraph
- heaptrack
- perf
- DTrace

### Testing Tools
- Criterion
- hey
- k6
- Custom benchmarks

## Best Practices

### Code Level
1. Use async/await properly
2. Implement caching strategies
3. Optimize data structures
4. Profile hot code paths

### System Level
1. Configure resource limits
2. Monitor system metrics
3. Use appropriate scaling
4. Implement circuit breakers

### Database Level
1. Optimize queries
2. Use appropriate indexes
3. Monitor query performance
4. Implement caching

### Network Level
1. Use connection pooling
2. Implement retries
3. Monitor latency
4. Use appropriate protocols

## Performance Checklist

### Development
- [ ] Run benchmarks
- [ ] Profile hot paths
- [ ] Review memory usage
- [ ] Check query performance

### Deployment
- [ ] Set resource limits
- [ ] Configure monitoring
- [ ] Enable metrics
- [ ] Set up alerts

### Maintenance
- [ ] Monitor metrics
- [ ] Review performance
- [ ] Update baselines
- [ ] Optimize bottlenecks

## Contribution Guidelines

When submitting performance-related changes:

1. Include benchmark results
2. Document optimization rationale
3. Consider all platforms
4. Maintain code readability
5. Add relevant tests

## Platform-Specific Considerations

### macOS
- Monitor calendar API usage
- Handle sandbox restrictions
- Consider energy impact

### Linux
- Profile system call overhead
- Monitor memory pressure
- Consider container limits

## Tools and Resources

### Performance Analysis
- Criterion.rs for benchmarking
- flamegraph for CPU profiling
- heaptrack for memory analysis
- perf for system profiling

### Monitoring
- Custom metrics collection
- Error rate tracking
- Resource usage monitoring

## Performance Regression Testing

1. **Automated Checks**
   - Run benchmarks in CI
   - Compare against baseline
   - Alert on significant regressions

2. **Manual Review**
   - Review flamegraphs
   - Analyze memory patterns
   - Check platform differences

## Getting Help

- Check existing performance issues
- Run benchmarks locally
- Share profiling results
- Document reproduction steps