//! Integration tests for YACPD candlestick pattern detection library.
//!
//! These tests validate the API and core functionality.

use yacpd::prelude::*;

/// Simple test bar structure
#[derive(Debug, Clone, Copy)]
struct TestBar {
    o: f64,
    h: f64,
    l: f64,
    c: f64,
}

impl TestBar {
    fn new(o: f64, h: f64, l: f64, c: f64) -> Self {
        Self { o, h, l, c }
    }
}

impl OHLCV for TestBar {
    fn open(&self) -> f64 {
        self.o
    }

    fn high(&self) -> f64 {
        self.h
    }

    fn low(&self) -> f64 {
        self.l
    }

    fn close(&self) -> f64 {
        self.c
    }

    fn volume(&self) -> f64 {
        1000.0
    }
}

/// Generate downtrend bars
fn make_downtrend(n: usize) -> Vec<TestBar> {
    (0..n)
        .map(|i| {
            let base = 100.0 - (i as f64) * 2.0;
            TestBar::new(base + 1.0, base + 2.0, base - 1.0, base - 0.5)
        })
        .collect()
}

/// Generate uptrend bars
fn make_uptrend(n: usize) -> Vec<TestBar> {
    (0..n)
        .map(|i| {
            let base = 100.0 + (i as f64) * 2.0;
            TestBar::new(base - 0.5, base + 1.5, base - 1.5, base + 1.0)
        })
        .collect()
}

/// Generate sideways bars
fn make_sideways(n: usize) -> Vec<TestBar> {
    (0..n)
        .map(|_| TestBar::new(100.0, 102.0, 98.0, 101.0))
        .collect()
}

// ============================================================
// SINGLE BAR PATTERN TESTS
// ============================================================

#[test]
fn test_doji_detection() {
    let mut bars = make_downtrend(10);
    // Perfect doji: open = close
    bars.push(TestBar::new(80.0, 85.0, 75.0, 80.0));

    let engine = EngineBuilder::new()
        .add(BuiltinDetector::Doji(DojiDetector::with_defaults()))
        .build()
        .unwrap();

    let patterns = engine.scan(&bars).unwrap();
    assert!(!patterns.is_empty(), "Doji should be detected");
    assert_eq!(patterns[0].pattern_id.0, "CDL_DOJI");
}

#[test]
fn test_dragonfly_doji_detection() {
    let mut bars = make_downtrend(10);
    // Dragonfly doji: open = close = high, long lower shadow
    bars.push(TestBar::new(80.0, 80.0, 70.0, 80.0));

    let engine = EngineBuilder::new()
        .add(BuiltinDetector::DragonflyDoji(
            DragonflyDojiDetector::with_defaults(),
        ))
        .build()
        .unwrap();

    let patterns = engine.scan(&bars).unwrap();
    assert!(!patterns.is_empty(), "Dragonfly Doji should be detected");
}

#[test]
fn test_gravestone_doji_detection() {
    let mut bars = make_uptrend(10);
    // Gravestone doji: open = close = low, long upper shadow
    bars.push(TestBar::new(120.0, 130.0, 120.0, 120.0));

    let engine = EngineBuilder::new()
        .add(BuiltinDetector::GravestoneDoji(
            GravestoneDojiDetector::with_defaults(),
        ))
        .build()
        .unwrap();

    let patterns = engine.scan(&bars).unwrap();
    assert!(!patterns.is_empty(), "Gravestone Doji should be detected");
}

#[test]
fn test_marubozu_detection() {
    let mut bars = make_sideways(10);
    // Perfect bullish marubozu: open = low, close = high
    bars.push(TestBar::new(100.0, 110.0, 100.0, 110.0));

    let engine = EngineBuilder::new()
        .add(BuiltinDetector::Marubozu(MarubozuDetector::with_defaults()))
        .build()
        .unwrap();

    let patterns = engine.scan(&bars).unwrap();
    assert!(!patterns.is_empty(), "Marubozu should be detected");
}

// ============================================================
// ENGINE API TESTS
// ============================================================

#[test]
fn test_engine_with_all_defaults() {
    let engine = EngineBuilder::new().with_all_defaults().build().unwrap();

    let bars = make_downtrend(20);
    let result = engine.scan(&bars);

    assert!(result.is_ok(), "Scan should not fail");
}

#[test]
fn test_engine_with_single_bar_defaults() {
    let engine = EngineBuilder::new()
        .with_single_bar_defaults()
        .build()
        .unwrap();

    let bars = make_sideways(20);
    let result = engine.scan(&bars);

    assert!(result.is_ok());
}

#[test]
fn test_engine_with_two_bar_defaults() {
    let engine = EngineBuilder::new()
        .with_two_bar_defaults()
        .build()
        .unwrap();

    let bars = make_sideways(20);
    let result = engine.scan(&bars);

    assert!(result.is_ok());
}

#[test]
fn test_engine_with_three_bar_defaults() {
    let engine = EngineBuilder::new()
        .with_three_bar_defaults()
        .build()
        .unwrap();

    let bars = make_sideways(20);
    let result = engine.scan(&bars);

    assert!(result.is_ok());
}

#[test]
fn test_engine_scan_grouped() {
    let mut bars = make_downtrend(10);
    bars.push(TestBar::new(80.0, 85.0, 75.0, 80.0)); // Doji

    let engine = EngineBuilder::new()
        .add(BuiltinDetector::Doji(DojiDetector::with_defaults()))
        .build()
        .unwrap();

    let grouped = engine.scan_grouped(&bars).unwrap();
    assert_eq!(grouped.len(), bars.len());
    assert!(!grouped[10].is_empty(), "Pattern should be at index 10");
}

#[test]
fn test_engine_iterator() {
    let mut bars = make_downtrend(10);
    bars.push(TestBar::new(80.0, 85.0, 75.0, 80.0)); // Doji

    let engine = EngineBuilder::new()
        .add(BuiltinDetector::Doji(DojiDetector::with_defaults()))
        .build()
        .unwrap();

    let mut found = false;
    for bar_patterns in engine.iter(&bars) {
        if bar_patterns.index == 10 && !bar_patterns.patterns.is_empty() {
            found = true;
        }
    }
    assert!(found, "Should find doji at index 10");
}

#[test]
fn test_engine_iterator_exact_size() {
    let bars = make_sideways(50);

    let engine = EngineBuilder::new().with_all_defaults().build().unwrap();

    let iter = engine.iter(&bars);
    assert_eq!(iter.len(), 50);
}

#[test]
fn test_engine_scan_range() {
    let mut bars = make_downtrend(20);
    bars[10] = TestBar::new(70.0, 75.0, 65.0, 70.0); // Doji at index 10

    let engine = EngineBuilder::new()
        .add(BuiltinDetector::Doji(DojiDetector::with_defaults()))
        .build()
        .unwrap();

    let contexts = engine.compute_contexts(&bars);
    let patterns = engine.scan_range(&bars, 8..15, &contexts);

    // Should find doji within range
    let has_doji = patterns.iter().any(|p| p.end_index == 10);
    assert!(has_doji, "Should find doji in range");
}

#[test]
fn test_engine_scan_at() {
    let mut bars = make_downtrend(20);
    bars[10] = TestBar::new(70.0, 75.0, 65.0, 70.0); // Doji at index 10

    let engine = EngineBuilder::new()
        .add(BuiltinDetector::Doji(DojiDetector::with_defaults()))
        .build()
        .unwrap();

    let ctx = engine.compute_context_at(&bars, 10);
    let patterns = engine.scan_at(&bars, 10, &ctx);

    assert!(!patterns.is_empty(), "Should find doji at index 10");
}

#[test]
fn test_compute_contexts() {
    let bars = make_sideways(50);

    let engine = EngineBuilder::new().with_all_defaults().build().unwrap();

    let contexts = engine.compute_contexts(&bars);
    assert_eq!(contexts.len(), bars.len());
}

#[test]
fn test_min_strength_filter() {
    let mut bars = make_downtrend(10);
    bars.push(TestBar::new(80.0, 85.0, 75.0, 80.0)); // Doji

    // Very high threshold - may filter some patterns
    let engine = EngineBuilder::new()
        .add(BuiltinDetector::Doji(DojiDetector::with_defaults()))
        .min_strength(0.99)
        .build()
        .unwrap();

    let result = engine.scan(&bars);
    assert!(result.is_ok());
}

#[test]
fn test_pattern_filter() {
    let engine = EngineBuilder::new()
        .with_all_defaults()
        .only_patterns([PatternId("CDL_DOJI"), PatternId("CDL_HAMMER")])
        .build()
        .unwrap();

    let bars = make_sideways(20);
    let result = engine.scan(&bars);
    assert!(result.is_ok());
}

#[test]
fn test_parallel_scan() {
    let bars1 = make_downtrend(50);
    let bars2 = make_uptrend(50);

    let instruments: Vec<(&str, &[TestBar])> = vec![("SYM1", &bars1), ("SYM2", &bars2)];

    let engine = EngineBuilder::new().with_all_defaults().build().unwrap();

    let (results, errors) = scan_parallel(&engine, instruments);

    assert_eq!(results.len(), 2);
    assert!(errors.is_empty());
    assert_eq!(results[0].symbol, "SYM1");
    assert_eq!(results[1].symbol, "SYM2");
}

// ============================================================
// EDGE CASES
// ============================================================

#[test]
fn test_empty_bars() {
    let bars: Vec<TestBar> = vec![];

    let engine = EngineBuilder::new().with_all_defaults().build().unwrap();

    let patterns = engine.scan(&bars).unwrap();
    assert!(patterns.is_empty());
}

#[test]
fn test_single_bar() {
    let bars = vec![TestBar::new(100.0, 105.0, 95.0, 102.0)];

    let engine = EngineBuilder::new().with_all_defaults().build().unwrap();

    let result = engine.scan(&bars);
    assert!(result.is_ok());
}

#[test]
fn test_two_bars() {
    let bars = vec![
        TestBar::new(100.0, 105.0, 95.0, 102.0),
        TestBar::new(102.0, 107.0, 97.0, 104.0),
    ];

    let engine = EngineBuilder::new().with_all_defaults().build().unwrap();

    let result = engine.scan(&bars);
    assert!(result.is_ok());
}

#[test]
fn test_no_false_positives_on_flat_data() {
    // Completely flat data - should not detect hammer, engulfing, etc.
    let bars: Vec<TestBar> = (0..100)
        .map(|_| TestBar::new(100.0, 100.5, 99.5, 100.1))
        .collect();

    let engine = EngineBuilder::new()
        .add(BuiltinDetector::Hammer(HammerDetector::with_defaults()))
        .add(BuiltinDetector::MorningStar(
            MorningStarDetector::with_defaults(),
        ))
        .build()
        .unwrap();

    let patterns = engine.scan(&bars).unwrap();
    assert!(
        patterns.is_empty(),
        "Should not detect patterns on flat data"
    );
}

// ============================================================
// PATTERN MATCH PROPERTIES
// ============================================================

#[test]
fn test_pattern_match_has_correct_fields() {
    let mut bars = make_downtrend(10);
    bars.push(TestBar::new(80.0, 85.0, 75.0, 80.0)); // Doji

    let engine = EngineBuilder::new()
        .add(BuiltinDetector::Doji(DojiDetector::with_defaults()))
        .build()
        .unwrap();

    let patterns = engine.scan(&bars).unwrap();
    assert!(!patterns.is_empty());

    let p = &patterns[0];
    assert_eq!(p.pattern_id.0, "CDL_DOJI");
    assert_eq!(p.end_index, 10);
    assert!(p.start_index <= p.end_index);
    assert!(p.strength >= 0.0 && p.strength <= 1.0);
}

#[test]
fn test_direction_enum() {
    // Test that Direction enum is properly exported and usable
    let bullish = Direction::Bullish;
    let bearish = Direction::Bearish;
    let neutral = Direction::Neutral;

    assert_ne!(bullish, bearish);
    assert_ne!(bullish, neutral);
    assert_ne!(bearish, neutral);
}

// ============================================================
// CUSTOM DETECTOR TEST
// ============================================================

struct CustomDetector;

impl PatternDetector for CustomDetector {
    fn id(&self) -> PatternId {
        PatternId("custom_pattern")
    }

    fn min_bars(&self) -> usize {
        1
    }

    fn detect<T: OHLCV>(
        &self,
        bars: &[T],
        index: usize,
        _ctx: &MarketContext,
    ) -> Option<PatternMatch> {
        let bar = bars.get(index)?;

        // Custom logic: detect when close > open by more than 5%
        if bar.close() > bar.open() * 1.05 {
            Some(PatternMatch {
                pattern_id: PatternId("custom_pattern"),
                direction: Direction::Bullish,
                strength: 0.8,
                start_index: index,
                end_index: index,
            })
        } else {
            None
        }
    }
}

#[test]
fn test_custom_detector() {
    let mut bars = make_sideways(10);
    // Add bar with >5% gain
    bars.push(TestBar::new(100.0, 110.0, 99.0, 106.0));

    let engine = EngineBuilder::new()
        .add_custom(CustomDetector)
        .build()
        .unwrap();

    let patterns = engine.scan(&bars).unwrap();
    assert!(!patterns.is_empty(), "Custom pattern should be detected");
    assert_eq!(patterns[0].pattern_id.0, "custom_pattern");
}

// ============================================================
// RATIO AND PERIOD VALIDATION
// ============================================================

#[test]
fn test_ratio_validation() {
    // Valid ratios
    assert!(Ratio::new(0.0).is_ok());
    assert!(Ratio::new(0.5).is_ok());
    assert!(Ratio::new(1.0).is_ok());

    // Invalid ratios
    assert!(Ratio::new(-0.1).is_err());
    assert!(Ratio::new(1.1).is_err());
}

#[test]
fn test_period_validation() {
    // Valid periods
    assert!(Period::new(1).is_ok());
    assert!(Period::new(100).is_ok());

    // Invalid period
    assert!(Period::new(0).is_err());
}
