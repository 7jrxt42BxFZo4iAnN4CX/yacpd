//! Comprehensive tests for all 33 extended (non-TA-Lib) candlestick patterns.
//!
//! Each pattern has:
//! - Positive test: bars that clearly match the pattern
//! - Negative test: bars that violate one key condition
//! - Edge case tests where applicable

use yacpd::prelude::*;

// ============================================================
// TEST HELPERS
// ============================================================

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

/// Generate downtrend bars (bearish, each lower)
fn make_downtrend(n: usize) -> Vec<TestBar> {
  (0..n)
    .map(|i| {
      let base = 100.0 - (i as f64) * 2.0;
      TestBar::new(base + 1.0, base + 2.0, base - 1.0, base - 0.5)
    })
    .collect()
}

/// Generate uptrend bars (bullish, each higher)
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
  (0..n).map(|_| TestBar::new(100.0, 102.0, 98.0, 101.0)).collect()
}

/// Helper: scan specific detector, return matches for the last bar
fn detect_last<D: PatternDetector + Clone + 'static>(
  detector: BuiltinDetector,
  bars: &[TestBar],
) -> Vec<PatternMatch> {
  let engine = EngineBuilder::new().add(detector).build().unwrap();
  engine.scan(bars).unwrap()
}

/// Helper: check if pattern fires at the last index
fn fires_at_last(detector: BuiltinDetector, bars: &[TestBar]) -> bool {
  let last = bars.len() - 1;
  let engine = EngineBuilder::new().add(detector).build().unwrap();
  let patterns = engine.scan(bars).unwrap();
  patterns.iter().any(|p| p.end_index == last)
}

// ============================================================
// SINGLE-BAR PATTERNS
// ============================================================

// --- BlackCandle ---

#[test]
fn test_black_candle_positive() {
  let bars = vec![TestBar::new(105.0, 106.0, 99.0, 100.0)]; // close < open
  assert!(fires_at_last(BuiltinDetector::BlackCandle(BlackCandleDetector::with_defaults()), &bars));
}

#[test]
fn test_black_candle_negative() {
  let bars = vec![TestBar::new(100.0, 106.0, 99.0, 105.0)]; // close > open (bullish)
  assert!(!fires_at_last(
    BuiltinDetector::BlackCandle(BlackCandleDetector::with_defaults()),
    &bars
  ));
}

// --- WhiteCandle ---

#[test]
fn test_white_candle_positive() {
  let bars = vec![TestBar::new(100.0, 106.0, 99.0, 105.0)]; // close > open
  assert!(fires_at_last(BuiltinDetector::WhiteCandle(WhiteCandleDetector::with_defaults()), &bars));
}

#[test]
fn test_white_candle_negative() {
  let bars = vec![TestBar::new(105.0, 106.0, 99.0, 100.0)]; // close < open
  assert!(!fires_at_last(
    BuiltinDetector::WhiteCandle(WhiteCandleDetector::with_defaults()),
    &bars
  ));
}

// --- ShortBlack ---

#[test]
fn test_short_black_positive() {
  // Bearish, small body relative to range: body=1, range=10 -> ratio=0.1 < 0.3
  let bars = vec![TestBar::new(101.0, 105.0, 95.0, 100.0)];
  assert!(fires_at_last(BuiltinDetector::ShortBlack(ShortBlackDetector::with_defaults()), &bars));
}

#[test]
fn test_short_black_negative_bullish() {
  // Bullish candle - not a black candle
  let bars = vec![TestBar::new(100.0, 105.0, 95.0, 101.0)];
  assert!(!fires_at_last(BuiltinDetector::ShortBlack(ShortBlackDetector::with_defaults()), &bars));
}

#[test]
fn test_short_black_negative_long_body() {
  // Large body: body=9, range=10 -> ratio=0.9 > 0.3
  let bars = vec![TestBar::new(109.0, 110.0, 100.0, 100.5)];
  assert!(!fires_at_last(BuiltinDetector::ShortBlack(ShortBlackDetector::with_defaults()), &bars));
}

// --- ShortWhite ---

#[test]
fn test_short_white_positive() {
  // Bullish, small body: body=1, range=10 -> ratio=0.1 < 0.3
  let bars = vec![TestBar::new(100.0, 105.0, 95.0, 101.0)];
  assert!(fires_at_last(BuiltinDetector::ShortWhite(ShortWhiteDetector::with_defaults()), &bars));
}

#[test]
fn test_short_white_negative_long_body() {
  // Large body: body=9, range=10 -> ratio=0.9 > 0.3
  let bars = vec![TestBar::new(100.5, 110.0, 100.0, 109.0)];
  assert!(!fires_at_last(BuiltinDetector::ShortWhite(ShortWhiteDetector::with_defaults()), &bars));
}

// --- LongBlackDay ---

#[test]
fn test_long_black_day_positive() {
  // body=8, range=10 -> ratio=0.8 >= 0.7
  let bars = vec![TestBar::new(109.0, 110.0, 100.0, 101.0)];
  assert!(fires_at_last(
    BuiltinDetector::LongBlackDay(LongBlackDayDetector::with_defaults()),
    &bars
  ));
}

#[test]
fn test_long_black_day_negative_small_body() {
  // body=2, range=10 -> ratio=0.2 < 0.7
  let bars = vec![TestBar::new(102.0, 105.0, 95.0, 100.0)];
  assert!(!fires_at_last(
    BuiltinDetector::LongBlackDay(LongBlackDayDetector::with_defaults()),
    &bars
  ));
}

// --- LongWhiteDay ---

#[test]
fn test_long_white_day_positive() {
  // body=8, range=10 -> ratio=0.8 >= 0.7
  let bars = vec![TestBar::new(101.0, 110.0, 100.0, 109.0)];
  assert!(fires_at_last(
    BuiltinDetector::LongWhiteDay(LongWhiteDayDetector::with_defaults()),
    &bars
  ));
}

#[test]
fn test_long_white_day_negative_small_body() {
  // body=2, range=10 -> ratio=0.2 < 0.7
  let bars = vec![TestBar::new(100.0, 105.0, 95.0, 102.0)];
  assert!(!fires_at_last(
    BuiltinDetector::LongWhiteDay(LongWhiteDayDetector::with_defaults()),
    &bars
  ));
}

// --- BlackMarubozu ---

#[test]
fn test_black_marubozu_positive() {
  // Open=High, Close=Low (no shadows)
  let bars = vec![TestBar::new(110.0, 110.0, 100.0, 100.0)];
  assert!(fires_at_last(
    BuiltinDetector::BlackMarubozu(BlackMarubozuDetector::with_defaults()),
    &bars
  ));
}

#[test]
fn test_black_marubozu_negative_has_shadows() {
  // Has upper and lower shadows
  let bars = vec![TestBar::new(108.0, 112.0, 98.0, 102.0)];
  assert!(!fires_at_last(
    BuiltinDetector::BlackMarubozu(BlackMarubozuDetector::with_defaults()),
    &bars
  ));
}

// --- WhiteMarubozu ---

#[test]
fn test_white_marubozu_positive() {
  // Open=Low, Close=High (no shadows)
  let bars = vec![TestBar::new(100.0, 110.0, 100.0, 110.0)];
  assert!(fires_at_last(
    BuiltinDetector::WhiteMarubozu(WhiteMarubozuDetector::with_defaults()),
    &bars
  ));
}

#[test]
fn test_white_marubozu_negative_has_shadows() {
  let bars = vec![TestBar::new(102.0, 112.0, 98.0, 108.0)];
  assert!(!fires_at_last(
    BuiltinDetector::WhiteMarubozu(WhiteMarubozuDetector::with_defaults()),
    &bars
  ));
}

// --- OpeningBlackMarubozu ---

#[test]
fn test_opening_black_marubozu_positive() {
  // Open=High (no upper shadow), has lower shadow (Close > Low)
  let bars = vec![TestBar::new(110.0, 110.0, 98.0, 102.0)];
  assert!(fires_at_last(
    BuiltinDetector::OpeningBlackMarubozu(OpeningBlackMarubozuDetector::with_defaults()),
    &bars
  ));
}

#[test]
fn test_opening_black_marubozu_negative_full_marubozu() {
  // Open=High AND Close=Low -> full marubozu, not opening marubozu (no lower shadow)
  let bars = vec![TestBar::new(110.0, 110.0, 100.0, 100.0)];
  assert!(!fires_at_last(
    BuiltinDetector::OpeningBlackMarubozu(OpeningBlackMarubozuDetector::with_defaults()),
    &bars
  ));
}

// --- OpeningWhiteMarubozu ---

#[test]
fn test_opening_white_marubozu_positive() {
  // Open=Low (no lower shadow), has upper shadow (Close < High)
  let bars = vec![TestBar::new(100.0, 112.0, 100.0, 108.0)];
  assert!(fires_at_last(
    BuiltinDetector::OpeningWhiteMarubozu(OpeningWhiteMarubozuDetector::with_defaults()),
    &bars
  ));
}

#[test]
fn test_opening_white_marubozu_negative_full_marubozu() {
  // Open=Low AND Close=High -> full marubozu, not opening
  let bars = vec![TestBar::new(100.0, 110.0, 100.0, 110.0)];
  assert!(!fires_at_last(
    BuiltinDetector::OpeningWhiteMarubozu(OpeningWhiteMarubozuDetector::with_defaults()),
    &bars
  ));
}

// --- BlackSpinningTop ---

#[test]
fn test_black_spinning_top_positive() {
  // Bearish, small body, shadows on both sides
  // body=1 (102->101), upper=3 (105-102), lower=4 (101-97), range=8
  // body_ratio=1/8=0.125 < 0.3, shadows >= body*0.5=0.5 ✓
  let bars = vec![TestBar::new(102.0, 105.0, 97.0, 101.0)];
  assert!(fires_at_last(
    BuiltinDetector::BlackSpinningTop(BlackSpinningTopDetector::with_defaults()),
    &bars
  ));
}

#[test]
fn test_black_spinning_top_negative_large_body() {
  // Large body: body=8, range=10 -> ratio=0.8 > 0.3
  let bars = vec![TestBar::new(109.0, 110.0, 100.0, 101.0)];
  assert!(!fires_at_last(
    BuiltinDetector::BlackSpinningTop(BlackSpinningTopDetector::with_defaults()),
    &bars
  ));
}

// --- WhiteSpinningTop ---

#[test]
fn test_white_spinning_top_positive() {
  // Bullish, small body, shadows on both sides
  // body=1 (101->102), upper=3 (105-102), lower=4 (101-97), range=8
  let bars = vec![TestBar::new(101.0, 105.0, 97.0, 102.0)];
  assert!(fires_at_last(
    BuiltinDetector::WhiteSpinningTop(WhiteSpinningTopDetector::with_defaults()),
    &bars
  ));
}

#[test]
fn test_white_spinning_top_negative_large_body() {
  let bars = vec![TestBar::new(101.0, 110.0, 100.0, 109.0)];
  assert!(!fires_at_last(
    BuiltinDetector::WhiteSpinningTop(WhiteSpinningTopDetector::with_defaults()),
    &bars
  ));
}

// --- NorthernDoji ---

#[test]
fn test_northern_doji_positive() {
  let mut bars = make_uptrend(15); // Strong uptrend
                                   // Add doji at top: body=0, range=10
  bars.push(TestBar::new(130.0, 135.0, 125.0, 130.0));
  assert!(fires_at_last(
    BuiltinDetector::NorthernDoji(NorthernDojiDetector::with_defaults()),
    &bars
  ));
}

#[test]
fn test_northern_doji_negative_downtrend() {
  let mut bars = make_downtrend(15);
  // Doji in downtrend should NOT fire as northern doji
  bars.push(TestBar::new(70.0, 75.0, 65.0, 70.0));
  assert!(!fires_at_last(
    BuiltinDetector::NorthernDoji(NorthernDojiDetector::with_defaults()),
    &bars
  ));
}

#[test]
fn test_northern_doji_negative_not_doji() {
  let mut bars = make_uptrend(15);
  // Large body, not a doji
  bars.push(TestBar::new(130.0, 140.0, 129.0, 139.0));
  assert!(!fires_at_last(
    BuiltinDetector::NorthernDoji(NorthernDojiDetector::with_defaults()),
    &bars
  ));
}

// --- SouthernDoji ---

#[test]
fn test_southern_doji_positive() {
  let mut bars = make_downtrend(15);
  // Doji at bottom of downtrend
  bars.push(TestBar::new(70.0, 75.0, 65.0, 70.0));
  assert!(fires_at_last(
    BuiltinDetector::SouthernDoji(SouthernDojiDetector::with_defaults()),
    &bars
  ));
}

#[test]
fn test_southern_doji_negative_uptrend() {
  let mut bars = make_uptrend(15);
  bars.push(TestBar::new(130.0, 135.0, 125.0, 130.0));
  assert!(!fires_at_last(
    BuiltinDetector::SouthernDoji(SouthernDojiDetector::with_defaults()),
    &bars
  ));
}

// ============================================================
// TWO-BAR PATTERNS
// ============================================================

// --- FallingWindow ---

#[test]
fn test_falling_window_positive() {
  let bars = vec![
    TestBar::new(105.0, 110.0, 100.0, 102.0), // first
    TestBar::new(97.0, 99.0, 93.0, 94.0),     // gap down: high(99) < low(100)
  ];
  assert!(fires_at_last(
    BuiltinDetector::FallingWindow(FallingWindowDetector::with_defaults()),
    &bars
  ));
}

#[test]
fn test_falling_window_negative_no_gap() {
  let bars = vec![
    TestBar::new(105.0, 110.0, 100.0, 102.0),
    TestBar::new(101.0, 105.0, 98.0, 99.0), // high(105) > low(100), no gap
  ];
  assert!(!fires_at_last(
    BuiltinDetector::FallingWindow(FallingWindowDetector::with_defaults()),
    &bars
  ));
}

// --- RisingWindow ---

#[test]
fn test_rising_window_positive() {
  let bars = vec![
    TestBar::new(98.0, 100.0, 95.0, 99.0),    // first
    TestBar::new(102.0, 108.0, 101.0, 107.0), // gap up: low(101) > high(100)
  ];
  assert!(fires_at_last(
    BuiltinDetector::RisingWindow(RisingWindowDetector::with_defaults()),
    &bars
  ));
}

#[test]
fn test_rising_window_negative_no_gap() {
  let bars = vec![
    TestBar::new(98.0, 102.0, 95.0, 99.0),
    TestBar::new(100.0, 108.0, 99.0, 107.0), // low(99) < high(102)
  ];
  assert!(!fires_at_last(
    BuiltinDetector::RisingWindow(RisingWindowDetector::with_defaults()),
    &bars
  ));
}

// --- GappingDownDoji ---

#[test]
fn test_gapping_down_doji_positive() {
  let bars = vec![
    TestBar::new(105.0, 110.0, 100.0, 102.0), // normal bar
    TestBar::new(98.0, 99.0, 93.0, 98.0),     // doji with gap: high(99) < low(100)
  ];
  assert!(fires_at_last(
    BuiltinDetector::GappingDownDoji(GappingDownDojiDetector::with_defaults()),
    &bars
  ));
}

#[test]
fn test_gapping_down_doji_negative_no_full_gap() {
  // Open gaps down but high overlaps previous low (Bug 1 fix test)
  let bars = vec![
    TestBar::new(105.0, 110.0, 100.0, 102.0),
    TestBar::new(98.0, 101.0, 95.0, 98.0), // high(101) > low(100), no full gap
  ];
  assert!(!fires_at_last(
    BuiltinDetector::GappingDownDoji(GappingDownDojiDetector::with_defaults()),
    &bars
  ));
}

#[test]
fn test_gapping_down_doji_negative_not_doji() {
  let bars = vec![
    TestBar::new(105.0, 110.0, 100.0, 102.0),
    TestBar::new(97.0, 99.0, 90.0, 91.0), // large body, not doji
  ];
  assert!(!fires_at_last(
    BuiltinDetector::GappingDownDoji(GappingDownDojiDetector::with_defaults()),
    &bars
  ));
}

// --- GappingUpDoji ---

#[test]
fn test_gapping_up_doji_positive() {
  let bars = vec![
    TestBar::new(98.0, 100.0, 95.0, 99.0),    // normal bar
    TestBar::new(102.0, 107.0, 101.0, 102.0), // doji with gap: low(101) > high(100)
  ];
  assert!(fires_at_last(
    BuiltinDetector::GappingUpDoji(GappingUpDojiDetector::with_defaults()),
    &bars
  ));
}

#[test]
fn test_gapping_up_doji_negative_no_full_gap() {
  // Open gaps up but low overlaps previous high (Bug 2 fix test)
  let bars = vec![
    TestBar::new(98.0, 100.0, 95.0, 99.0),
    TestBar::new(102.0, 107.0, 99.0, 102.0), // low(99) < high(100), no full gap
  ];
  assert!(!fires_at_last(
    BuiltinDetector::GappingUpDoji(GappingUpDojiDetector::with_defaults()),
    &bars
  ));
}

// --- AboveTheStomach ---

#[test]
fn test_above_the_stomach_positive() {
  let mut bars = make_downtrend(15);
  // Previous: bearish with midpoint around 70.5
  bars.push(TestBar::new(72.0, 73.0, 69.0, 69.5)); // bearish: O=72, C=69.5, mid=70.75
                                                   // Current: bullish, opens above midpoint
  bars.push(TestBar::new(71.0, 74.0, 70.0, 73.0));
  assert!(fires_at_last(
    BuiltinDetector::AboveTheStomach(AboveTheStomachDetector::with_defaults()),
    &bars
  ));
}

#[test]
fn test_above_the_stomach_negative_not_downtrend() {
  let mut bars = make_uptrend(15);
  bars.push(TestBar::new(132.0, 133.0, 129.0, 129.5)); // bearish
  bars.push(TestBar::new(131.0, 134.0, 130.0, 133.0)); // bullish
  assert!(!fires_at_last(
    BuiltinDetector::AboveTheStomach(AboveTheStomachDetector::with_defaults()),
    &bars
  ));
}

// --- BelowTheStomach ---

#[test]
fn test_below_the_stomach_positive() {
  let mut bars = make_uptrend(15);
  // Previous bullish: O=130, C=132, mid=131
  bars.push(TestBar::new(130.0, 133.0, 129.0, 132.0));
  // Current bearish: opens below midpoint (131)
  bars.push(TestBar::new(130.5, 131.0, 128.0, 128.5));
  assert!(fires_at_last(
    BuiltinDetector::BelowTheStomach(BelowTheStomachDetector::with_defaults()),
    &bars
  ));
}

#[test]
fn test_below_the_stomach_negative_not_uptrend() {
  let mut bars = make_downtrend(15);
  bars.push(TestBar::new(70.0, 73.0, 69.0, 72.0));
  bars.push(TestBar::new(70.5, 71.0, 68.0, 68.5));
  assert!(!fires_at_last(
    BuiltinDetector::BelowTheStomach(BelowTheStomachDetector::with_defaults()),
    &bars
  ));
}

// --- LastEngulfingBottom ---

#[test]
fn test_last_engulfing_bottom_positive() {
  let mut bars = make_downtrend(15);
  // Small bearish candle
  bars.push(TestBar::new(71.0, 72.0, 69.0, 70.0));
  // Large bullish engulfing candle
  bars.push(TestBar::new(69.5, 73.0, 68.0, 72.0));
  assert!(fires_at_last(
    BuiltinDetector::LastEngulfingBottom(LastEngulfingBottomDetector::with_defaults()),
    &bars
  ));
}

#[test]
fn test_last_engulfing_bottom_negative_not_engulfing() {
  let mut bars = make_downtrend(15);
  bars.push(TestBar::new(71.0, 72.0, 69.0, 70.0));
  // Bullish but doesn't engulf
  bars.push(TestBar::new(70.2, 71.5, 69.5, 70.8));
  assert!(!fires_at_last(
    BuiltinDetector::LastEngulfingBottom(LastEngulfingBottomDetector::with_defaults()),
    &bars
  ));
}

// --- LastEngulfingTop ---

#[test]
fn test_last_engulfing_top_positive() {
  let mut bars = make_uptrend(15);
  // Small bullish candle
  bars.push(TestBar::new(130.0, 132.0, 129.0, 131.0));
  // Large bearish engulfing
  bars.push(TestBar::new(131.5, 133.0, 128.0, 129.0));
  assert!(fires_at_last(
    BuiltinDetector::LastEngulfingTop(LastEngulfingTopDetector::with_defaults()),
    &bars
  ));
}

#[test]
fn test_last_engulfing_top_negative_not_uptrend() {
  let mut bars = make_downtrend(15);
  bars.push(TestBar::new(70.0, 72.0, 69.0, 71.0));
  bars.push(TestBar::new(71.5, 73.0, 68.0, 69.0));
  assert!(!fires_at_last(
    BuiltinDetector::LastEngulfingTop(LastEngulfingTopDetector::with_defaults()),
    &bars
  ));
}

// --- MeetingLinesBearish ---

#[test]
fn test_meeting_lines_bearish_positive() {
  let mut bars = make_uptrend(15);
  // Bullish with long body: O=130, C=139, range=10, body=9, ratio=0.9
  bars.push(TestBar::new(130.0, 140.0, 130.0, 139.0));
  // Bearish with long body opening high, closing at same level: C≈139
  bars.push(TestBar::new(148.0, 148.5, 138.5, 139.0));
  assert!(fires_at_last(
    BuiltinDetector::MeetingLinesBearish(MeetingLinesBearishDetector::with_defaults()),
    &bars
  ));
}

#[test]
fn test_meeting_lines_bearish_negative_short_body() {
  // Bug 4 fix test: short-bodied candles should NOT trigger
  let mut bars = make_uptrend(15);
  // Bullish but short body: O=130, C=131, range=10, body=1, ratio=0.1
  bars.push(TestBar::new(130.0, 135.0, 125.0, 131.0));
  // Bearish but short body closing at same level
  bars.push(TestBar::new(133.0, 136.0, 126.0, 131.0));
  assert!(!fires_at_last(
    BuiltinDetector::MeetingLinesBearish(MeetingLinesBearishDetector::with_defaults()),
    &bars
  ));
}

#[test]
fn test_meeting_lines_bearish_negative_closes_differ() {
  let mut bars = make_uptrend(15);
  bars.push(TestBar::new(130.0, 140.0, 130.0, 139.0));
  bars.push(TestBar::new(148.0, 148.5, 130.0, 132.0)); // closes differ by 7
  assert!(!fires_at_last(
    BuiltinDetector::MeetingLinesBearish(MeetingLinesBearishDetector::with_defaults()),
    &bars
  ));
}

// --- MeetingLinesBullish ---

#[test]
fn test_meeting_lines_bullish_positive() {
  let mut bars = make_downtrend(15);
  // Bearish with long body: O=72, C=63, range=10, body=9
  bars.push(TestBar::new(72.0, 72.0, 62.0, 63.0));
  // Bullish with long body, closing at same level: C≈63
  bars.push(TestBar::new(54.0, 63.5, 53.5, 63.0));
  assert!(fires_at_last(
    BuiltinDetector::MeetingLinesBullish(MeetingLinesBullishDetector::with_defaults()),
    &bars
  ));
}

#[test]
fn test_meeting_lines_bullish_negative_short_body() {
  let mut bars = make_downtrend(15);
  // Short body candles
  bars.push(TestBar::new(72.0, 77.0, 67.0, 71.0)); // body=1, range=10
  bars.push(TestBar::new(66.0, 76.0, 66.0, 71.0)); // body=5, range=10
  assert!(!fires_at_last(
    BuiltinDetector::MeetingLinesBullish(MeetingLinesBullishDetector::with_defaults()),
    &bars
  ));
}

// --- ShootingStar2Lines ---

#[test]
fn test_shooting_star_2_lines_positive() {
  let mut bars = make_uptrend(15);
  // Previous bullish
  bars.push(TestBar::new(129.0, 132.0, 128.0, 131.0));
  // Shooting star: small body at bottom, long upper shadow
  // O=131, H=140, L=130.5, C=131.5 -> body=0.5, upper=8.5, lower=0.5
  bars.push(TestBar::new(131.0, 140.0, 130.5, 131.5));
  assert!(fires_at_last(
    BuiltinDetector::ShootingStar2Lines(ShootingStar2LinesDetector::with_defaults()),
    &bars
  ));
}

#[test]
fn test_shooting_star_2_lines_negative_downtrend() {
  let mut bars = make_downtrend(15);
  bars.push(TestBar::new(69.0, 72.0, 68.0, 71.0));
  bars.push(TestBar::new(71.0, 80.0, 70.5, 71.5));
  assert!(!fires_at_last(
    BuiltinDetector::ShootingStar2Lines(ShootingStar2LinesDetector::with_defaults()),
    &bars
  ));
}

#[test]
fn test_shooting_star_2_lines_negative_large_body() {
  let mut bars = make_uptrend(15);
  bars.push(TestBar::new(129.0, 132.0, 128.0, 131.0));
  // Large body - not a shooting star
  bars.push(TestBar::new(131.0, 140.0, 130.0, 138.0));
  assert!(!fires_at_last(
    BuiltinDetector::ShootingStar2Lines(ShootingStar2LinesDetector::with_defaults()),
    &bars
  ));
}

// ============================================================
// THREE-BAR PATTERNS
// ============================================================

// --- CollapsingDojiStar ---

#[test]
fn test_collapsing_doji_star_bullish_positive() {
  // Bearish first, gap down to doji, then bullish reversal
  // Bug 3 fix: gap_pct is now 0.005, so gap needs to be body * 0.005
  // First: bearish, O=110, C=100, body=10
  // Doji: gaps down, high < 100, body tiny. Gap = 100 - doji.high >= 10*0.005 = 0.05
  // Third: bullish, closes above first's close (100)
  let bars = vec![
    TestBar::new(110.0, 111.0, 99.0, 100.0), // bearish, body=10
    TestBar::new(99.8, 99.9, 98.0, 99.8),    // doji: gap = 100 - 99.9 = 0.1 > 0.05
    TestBar::new(99.0, 106.0, 98.0, 105.0),  // bullish, closes > 100
  ];
  assert!(fires_at_last(
    BuiltinDetector::CollapsingDojiStar(CollapsingDojiStarDetector::with_defaults()),
    &bars
  ));
}

#[test]
fn test_collapsing_doji_star_bearish_positive() {
  // Bullish first, gap up to doji, then bearish reversal
  let bars = vec![
    TestBar::new(100.0, 111.0, 99.0, 110.0),   // bullish, body=10
    TestBar::new(110.1, 112.0, 110.1, 110.15), // doji gapping up: low(110.1) > max(100,110)=110
    TestBar::new(111.0, 111.5, 104.0, 105.0),  // bearish, closes < 110
  ];
  assert!(fires_at_last(
    BuiltinDetector::CollapsingDojiStar(CollapsingDojiStarDetector::with_defaults()),
    &bars
  ));
}

#[test]
fn test_collapsing_doji_star_negative_no_gap() {
  let bars = vec![
    TestBar::new(110.0, 111.0, 99.0, 100.0),
    TestBar::new(100.5, 101.0, 99.0, 100.5), // no gap (high 101 > close 100)
    TestBar::new(99.0, 106.0, 98.0, 105.0),
  ];
  assert!(!fires_at_last(
    BuiltinDetector::CollapsingDojiStar(CollapsingDojiStarDetector::with_defaults()),
    &bars
  ));
}

// --- Deliberation ---

#[test]
fn test_deliberation_positive() {
  let mut bars = make_uptrend(15);
  // Three bullish candles with progressive closes
  // First: long body O=129, C=137, range=9, body=8, ratio=0.89
  bars.push(TestBar::new(129.0, 138.0, 129.0, 137.0));
  // Second: long body O=137, C=145, range=9, body=8, ratio=0.89
  bars.push(TestBar::new(137.0, 146.0, 137.0, 145.0));
  // Third: small body opening at/near second's close, body=1
  // third.open()=145 >= second.close()-tolerance = 145 - 0.8 = 144.2 ✓
  // body=1, avg_body=(8+8)/2=8, ratio=1/8=0.125 < 0.3 ✓
  bars.push(TestBar::new(145.0, 146.5, 144.5, 146.0));
  assert!(fires_at_last(
    BuiltinDetector::Deliberation(DeliberationDetector::with_defaults()),
    &bars
  ));
}

#[test]
fn test_deliberation_negative_short_first_body() {
  // Bug 6 fix: first candle must have long body
  let mut bars = make_uptrend(15);
  // First: short body ratio=0.2 < 0.6
  bars.push(TestBar::new(129.0, 134.0, 124.0, 131.0));
  // Second: long body
  bars.push(TestBar::new(131.0, 140.0, 131.0, 139.0));
  // Third: small body
  bars.push(TestBar::new(139.0, 140.5, 138.5, 140.0));
  assert!(!fires_at_last(
    BuiltinDetector::Deliberation(DeliberationDetector::with_defaults()),
    &bars
  ));
}

#[test]
fn test_deliberation_negative_third_opens_low() {
  // Bug 6 fix: third candle must open near second's close
  let mut bars = make_uptrend(15);
  // First: long body
  bars.push(TestBar::new(129.0, 138.0, 129.0, 137.0));
  // Second: long body, close=145
  bars.push(TestBar::new(137.0, 146.0, 137.0, 145.0));
  // Third: opens way below second's close (145)
  // tolerance = 8 * 0.1 = 0.8, so open must be >= 144.2
  // Opens at 140 which is < 144.2
  bars.push(TestBar::new(140.0, 146.5, 139.5, 146.0));
  assert!(!fires_at_last(
    BuiltinDetector::Deliberation(DeliberationDetector::with_defaults()),
    &bars
  ));
}

#[test]
fn test_deliberation_negative_not_uptrend() {
  let mut bars = make_downtrend(15);
  bars.push(TestBar::new(69.0, 78.0, 69.0, 77.0));
  bars.push(TestBar::new(77.0, 86.0, 77.0, 85.0));
  bars.push(TestBar::new(85.0, 86.5, 84.5, 86.0));
  assert!(!fires_at_last(
    BuiltinDetector::Deliberation(DeliberationDetector::with_defaults()),
    &bars
  ));
}

// --- TwoBlackGapping ---

#[test]
fn test_two_black_gapping_positive() {
  // Bug 5 fix: now a 3-bar pattern
  // Prior candle, then gap down to two bearish candles
  let bars = vec![
    TestBar::new(105.0, 110.0, 100.0, 107.0), // prior
    TestBar::new(98.0, 99.0, 93.0, 94.0),     // first black: high(99) < prior.low(100) ✓
    TestBar::new(93.0, 94.0, 88.0, 89.0),     // second black: close(89) <= first.close(94) ✓
  ];
  assert!(fires_at_last(
    BuiltinDetector::TwoBlackGapping(TwoBlackGappingDetector::with_defaults()),
    &bars
  ));
}

#[test]
fn test_two_black_gapping_negative_no_gap() {
  let bars = vec![
    TestBar::new(105.0, 110.0, 100.0, 107.0),
    TestBar::new(101.0, 102.0, 96.0, 97.0), // high(102) >= prior.low(100), no gap
    TestBar::new(96.0, 97.0, 91.0, 92.0),
  ];
  assert!(!fires_at_last(
    BuiltinDetector::TwoBlackGapping(TwoBlackGappingDetector::with_defaults()),
    &bars
  ));
}

#[test]
fn test_two_black_gapping_negative_bullish_second() {
  let bars = vec![
    TestBar::new(105.0, 110.0, 100.0, 107.0),
    TestBar::new(98.0, 99.0, 93.0, 94.0),
    TestBar::new(93.0, 97.0, 92.0, 96.0), // bullish, not bearish
  ];
  assert!(!fires_at_last(
    BuiltinDetector::TwoBlackGapping(TwoBlackGappingDetector::with_defaults()),
    &bars
  ));
}

// --- DownsideGapThreeMethods ---

#[test]
fn test_downside_gap_three_methods_positive() {
  let bars = vec![
    TestBar::new(110.0, 111.0, 103.0, 104.0), // first bearish: O=110, C=104
    TestBar::new(100.0, 102.0, 95.0, 96.0),   // second bearish: gap down (H=102 < L=103)
    // Third bullish: opens in second body (96..100), closes in first body (104..110)
    TestBar::new(97.0, 107.0, 96.0, 106.0),
  ];
  assert!(fires_at_last(
    BuiltinDetector::DownsideGapThreeMethods(DownsideGapThreeMethodsDetector::with_defaults()),
    &bars
  ));
}

#[test]
fn test_downside_gap_three_methods_negative_no_gap() {
  let bars = vec![
    TestBar::new(110.0, 111.0, 103.0, 104.0),
    TestBar::new(104.0, 106.0, 99.0, 100.0), // high(106) >= low(103), no gap
    TestBar::new(101.0, 107.0, 100.0, 106.0),
  ];
  assert!(!fires_at_last(
    BuiltinDetector::DownsideGapThreeMethods(DownsideGapThreeMethodsDetector::with_defaults()),
    &bars
  ));
}

// --- UpsideGapThreeMethods ---

#[test]
fn test_upside_gap_three_methods_positive() {
  let bars = vec![
    TestBar::new(100.0, 107.0, 99.0, 106.0), // first bullish: O=100, C=106
    TestBar::new(109.0, 115.0, 108.0, 114.0), // second bullish: gap up (L=108 > H=107)
    // Third bearish: opens in second body (109..114), closes in first body (100..106)
    TestBar::new(112.0, 113.0, 102.0, 103.0),
  ];
  assert!(fires_at_last(
    BuiltinDetector::UpsideGapThreeMethods(UpsideGapThreeMethodsDetector::with_defaults()),
    &bars
  ));
}

#[test]
fn test_upside_gap_three_methods_negative_no_gap() {
  let bars = vec![
    TestBar::new(100.0, 107.0, 99.0, 106.0),
    TestBar::new(105.0, 115.0, 104.0, 114.0), // low(104) < high(107), no gap
    TestBar::new(112.0, 113.0, 102.0, 103.0),
  ];
  assert!(!fires_at_last(
    BuiltinDetector::UpsideGapThreeMethods(UpsideGapThreeMethodsDetector::with_defaults()),
    &bars
  ));
}

// --- DownsideTasukiGap ---

#[test]
fn test_downside_tasuki_gap_positive() {
  let bars = vec![
    TestBar::new(110.0, 111.0, 103.0, 104.0), // first bearish
    TestBar::new(100.0, 102.0, 95.0, 96.0),   // second bearish: gap down (H=102 < L=103)
    // Third bullish: partially fills gap (close enters gap but doesn't close it)
    // gap_top=103, gap_bottom=102, gap_size=1
    // fill_amount = close - gap_bottom = 102.5 - 102 = 0.5, fill_pct = 0.5 < 0.7
    TestBar::new(97.0, 103.0, 96.0, 102.5),
  ];
  assert!(fires_at_last(
    BuiltinDetector::DownsideTasukiGap(DownsideTasukiGapDetector::with_defaults()),
    &bars
  ));
}

#[test]
fn test_downside_tasuki_gap_negative_gap_closed() {
  let bars = vec![
    TestBar::new(110.0, 111.0, 103.0, 104.0),
    TestBar::new(100.0, 102.0, 95.0, 96.0), // gap: 103-102 = 1
    // Third closes fully above gap_top: fill_pct > 0.7
    TestBar::new(97.0, 104.0, 96.0, 103.5),
  ];
  assert!(!fires_at_last(
    BuiltinDetector::DownsideTasukiGap(DownsideTasukiGapDetector::with_defaults()),
    &bars
  ));
}

// --- UpsideTasukiGap ---

#[test]
fn test_upside_tasuki_gap_positive() {
  let bars = vec![
    TestBar::new(100.0, 107.0, 99.0, 106.0),  // first bullish
    TestBar::new(109.0, 115.0, 108.0, 114.0), // second bullish: gap up (L=108 > H=107)
    // Third bearish: partially fills gap
    // gap_bottom=107, gap_top=108, gap_size=1
    // fill_amount = gap_top - close = 108 - 107.5 = 0.5, fill_pct = 0.5 < 0.7
    TestBar::new(113.0, 114.0, 107.0, 107.5),
  ];
  assert!(fires_at_last(
    BuiltinDetector::UpsideTasukiGap(UpsideTasukiGapDetector::with_defaults()),
    &bars
  ));
}

#[test]
fn test_upside_tasuki_gap_negative_gap_closed() {
  let bars = vec![
    TestBar::new(100.0, 107.0, 99.0, 106.0),
    TestBar::new(109.0, 115.0, 108.0, 114.0), // gap: 108-107=1
    // Third fully closes gap
    TestBar::new(113.0, 114.0, 106.0, 106.5),
  ];
  assert!(!fires_at_last(
    BuiltinDetector::UpsideTasukiGap(UpsideTasukiGapDetector::with_defaults()),
    &bars
  ));
}

// ============================================================
// MULTI-BAR PATTERNS
// ============================================================

// --- PriceLines ---

#[test]
fn test_price_lines_bullish_positive() {
  // 8 consecutive bullish candles (overbought -> bearish signal)
  let bars: Vec<TestBar> = (0..8)
    .map(|i| {
      let base = 100.0 + i as f64 * 2.0;
      TestBar::new(base, base + 1.5, base - 0.5, base + 1.0)
    })
    .collect();
  let engine = EngineBuilder::new()
    .add(BuiltinDetector::PriceLines(PriceLinesDetector::with_defaults()))
    .build()
    .unwrap();
  let patterns = engine.scan(&bars).unwrap();
  let pl = patterns.iter().find(|p| p.pattern_id.0 == "PRICE_LINES");
  assert!(pl.is_some());
  assert_eq!(pl.unwrap().direction, Direction::Bearish); // overbought reversal
}

#[test]
fn test_price_lines_bearish_positive() {
  // 8 consecutive bearish candles (oversold -> bullish signal)
  let bars: Vec<TestBar> = (0..8)
    .map(|i| {
      let base = 100.0 - i as f64 * 2.0;
      TestBar::new(base + 1.0, base + 1.5, base - 0.5, base - 0.2)
    })
    .collect();
  let engine = EngineBuilder::new()
    .add(BuiltinDetector::PriceLines(PriceLinesDetector::with_defaults()))
    .build()
    .unwrap();
  let patterns = engine.scan(&bars).unwrap();
  let pl = patterns.iter().find(|p| p.pattern_id.0 == "PRICE_LINES");
  assert!(pl.is_some());
  assert_eq!(pl.unwrap().direction, Direction::Bullish);
}

#[test]
fn test_price_lines_negative_mixed() {
  // Mix of bullish and bearish — should not fire
  let bars = vec![
    TestBar::new(100.0, 102.0, 99.0, 101.0),  // bullish
    TestBar::new(101.0, 103.0, 100.0, 100.5), // bearish
    TestBar::new(100.5, 102.5, 99.5, 101.5),  // bullish
    TestBar::new(101.5, 103.5, 100.5, 101.0), // bearish
    TestBar::new(101.0, 103.0, 100.0, 102.0), // bullish
    TestBar::new(102.0, 104.0, 101.0, 101.5), // bearish
    TestBar::new(101.5, 103.5, 100.5, 102.5), // bullish
    TestBar::new(102.5, 104.5, 101.5, 102.0), // bearish
  ];
  assert!(!fires_at_last(BuiltinDetector::PriceLines(PriceLinesDetector::with_defaults()), &bars));
}

#[test]
fn test_price_lines_negative_only_7() {
  // Only 7 consecutive bullish — default is 8
  let bars: Vec<TestBar> = (0..7)
    .map(|i| {
      let base = 100.0 + i as f64 * 2.0;
      TestBar::new(base, base + 1.5, base - 0.5, base + 1.0)
    })
    .collect();
  assert!(!fires_at_last(BuiltinDetector::PriceLines(PriceLinesDetector::with_defaults()), &bars));
}

// ============================================================
// DIRECTION CHECKS
// ============================================================

#[test]
fn test_pattern_directions() {
  // Verify directions from matches are correct
  let bars = vec![TestBar::new(109.0, 110.0, 100.0, 101.0)]; // long black day

  let engine = EngineBuilder::new()
    .add(BuiltinDetector::LongBlackDay(LongBlackDayDetector::with_defaults()))
    .build()
    .unwrap();
  let patterns = engine.scan(&bars).unwrap();
  assert_eq!(patterns[0].direction, Direction::Bearish);

  let bars = vec![TestBar::new(101.0, 110.0, 100.0, 109.0)]; // long white day
  let engine = EngineBuilder::new()
    .add(BuiltinDetector::LongWhiteDay(LongWhiteDayDetector::with_defaults()))
    .build()
    .unwrap();
  let patterns = engine.scan(&bars).unwrap();
  assert_eq!(patterns[0].direction, Direction::Bullish);
}

// ============================================================
// EDGE CASE: Doji-like candles (open == close)
// ============================================================

#[test]
fn test_doji_edge_not_black_or_white() {
  // open == close: neither bullish nor bearish
  let bars = vec![TestBar::new(100.0, 105.0, 95.0, 100.0)];
  assert!(!fires_at_last(
    BuiltinDetector::BlackCandle(BlackCandleDetector::with_defaults()),
    &bars
  ));
  assert!(!fires_at_last(
    BuiltinDetector::WhiteCandle(WhiteCandleDetector::with_defaults()),
    &bars
  ));
}

// ============================================================
// PARAMETERIZED DETECTOR CONSTRUCTION
// ============================================================

#[test]
fn test_meeting_lines_bearish_with_params() {
  use std::collections::HashMap;
  let mut params = HashMap::new();
  params.insert("tolerance", 0.002);
  params.insert("body_pct", 0.7);
  let det = MeetingLinesBearishDetector::with_params(&params).unwrap();
  assert!((det.tolerance.get() - 0.002).abs() < 1e-9);
  assert!((det.body_pct.get() - 0.7).abs() < 1e-9);
}

#[test]
fn test_meeting_lines_bullish_with_params() {
  use std::collections::HashMap;
  let mut params = HashMap::new();
  params.insert("tolerance", 0.002);
  params.insert("body_pct", 0.7);
  let det = MeetingLinesBullishDetector::with_params(&params).unwrap();
  assert!((det.tolerance.get() - 0.002).abs() < 1e-9);
  assert!((det.body_pct.get() - 0.7).abs() < 1e-9);
}

#[test]
fn test_deliberation_with_params() {
  use std::collections::HashMap;
  let mut params = HashMap::new();
  params.insert("body_pct", 0.25);
  params.insert("long_body_pct", 0.65);
  let det = DeliberationDetector::with_params(&params).unwrap();
  assert!((det.body_pct.get() - 0.25).abs() < 1e-9);
  assert!((det.long_body_pct.get() - 0.65).abs() < 1e-9);
}

#[test]
fn test_collapsing_doji_star_default_matches_meta() {
  // Bug 3 regression test: default gap_pct must match ParamMeta default (0.005)
  let det = CollapsingDojiStarDetector::with_defaults();
  assert!((det.gap_pct.get() - 0.005).abs() < 1e-9);
}

// ============================================================
// ALL EXTENDED PATTERNS SCAN SMOKE TEST
// ============================================================

#[test]
fn test_all_extended_defaults_no_panic() {
  let engine = EngineBuilder::new().with_extended_defaults().build().unwrap();

  // Downtrend data
  let bars = make_downtrend(30);
  let result = engine.scan(&bars);
  assert!(result.is_ok());

  // Uptrend data
  let bars = make_uptrend(30);
  let result = engine.scan(&bars);
  assert!(result.is_ok());

  // Sideways data
  let bars = make_sideways(30);
  let result = engine.scan(&bars);
  assert!(result.is_ok());

  // Empty data
  let bars: Vec<TestBar> = vec![];
  let result = engine.scan(&bars);
  assert!(result.is_ok());

  // Single bar
  let bars = vec![TestBar::new(100.0, 105.0, 95.0, 102.0)];
  let result = engine.scan(&bars);
  assert!(result.is_ok());
}

// ============================================================
// STRENGTH RANGE CHECKS
// ============================================================

#[test]
fn test_all_strengths_in_valid_range() {
  let engine = EngineBuilder::new().with_extended_defaults().build().unwrap();

  let bars = make_downtrend(30);
  let patterns = engine.scan(&bars).unwrap();
  for p in &patterns {
    assert!(
      p.strength >= 0.0 && p.strength <= 1.0,
      "Pattern {:?} has strength {} out of [0,1] range",
      p.pattern_id.0,
      p.strength
    );
  }

  let bars = make_uptrend(30);
  let patterns = engine.scan(&bars).unwrap();
  for p in &patterns {
    assert!(
      p.strength >= 0.0 && p.strength <= 1.0,
      "Pattern {:?} has strength {} out of [0,1] range",
      p.pattern_id.0,
      p.strength
    );
  }
}

// ============================================================
// INDEX RANGE CHECKS
// ============================================================

#[test]
fn test_all_indices_valid() {
  let engine = EngineBuilder::new().with_extended_defaults().build().unwrap();

  let bars = make_downtrend(30);
  let patterns = engine.scan(&bars).unwrap();
  for p in &patterns {
    assert!(
      p.start_index <= p.end_index,
      "Pattern {:?} has start {} > end {}",
      p.pattern_id.0,
      p.start_index,
      p.end_index
    );
    assert!(
      p.end_index < bars.len(),
      "Pattern {:?} has end_index {} >= bars.len() {}",
      p.pattern_id.0,
      p.end_index,
      bars.len()
    );
  }
}
