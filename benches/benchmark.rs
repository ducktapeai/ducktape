use criterion::{black_box, criterion_group, criterion_main, Criterion};
use ducktape::command_parser::CommandParser;
use ducktape::command_processor::CommandProcessor;
use ducktape::config::Config;
use ducktape::calendar::Calendar;

fn benchmark_command_parsing(c: &mut Criterion) {
    let mut group = c.benchmark_group("command_parsing");
    
    group.bench_function("parse_calendar_command", |b| {
        b.iter(|| {
            let input = black_box("schedule a meeting with John tomorrow at 2pm");
            let parser = CommandParser::new();
            parser.parse(input)
        });
    });

    group.bench_function("parse_reminder_command", |b| {
        b.iter(|| {
            let input = black_box("remind me to buy groceries next Monday morning");
            let parser = CommandParser::new();
            parser.parse(input)
        });
    });

    group.finish();
}

fn benchmark_calendar_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("calendar_operations");
    let calendar = Calendar::new().unwrap();
    
    group.bench_function("search_events", |b| {
        b.iter(|| {
            calendar.search_events(black_box("meeting"), None, None)
        });
    });

    group.bench_function("create_simple_event", |b| {
        b.iter(|| {
            calendar.create_event(
                black_box("Test Event"),
                black_box("2024-03-24T14:00:00Z"),
                black_box("2024-03-24T15:00:00Z"),
                black_box(None),
                black_box(None),
                black_box(None),
            )
        });
    });

    group.finish();
}

fn benchmark_nlp_processing(c: &mut Criterion) {
    let mut group = c.benchmark_group("nlp_processing");
    let config = Config::load().unwrap();
    let processor = CommandProcessor::new(config);
    
    group.bench_function("process_natural_command", |b| {
        b.iter(|| {
            processor.process_command(
                black_box("create a weekly team meeting every Tuesday at 10am")
            )
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    benchmark_command_parsing,
    benchmark_calendar_operations,
    benchmark_nlp_processing
);
criterion_main!(benches);