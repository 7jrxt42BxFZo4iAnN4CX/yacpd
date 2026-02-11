//! Extended candlestick pattern detectors
//!
//! Additional patterns beyond TA-Lib: Price Lines, Windows, Meeting Lines,
//! Northern/Southern Doji, Opening Marubozu variants, Basic candle types, etc.

#![allow(
  clippy::collapsible_if,
  clippy::collapsible_else_if,
  clippy::default_constructed_unit_structs
)]

use std::collections::HashMap;

use crate::{params::{get_period, get_ratio, ParamMeta, ParamType, ParameterizedDetector}, Direction, MarketContext, OHLCVExt, PatternDetector, PatternId, PatternMatch, Period, Ratio, Result, OHLCV};

impl_with_defaults!(
  PriceLinesDetector,
  FallingWindowDetector,
  RisingWindowDetector,
  GappingDownDojiDetector,
  GappingUpDojiDetector,
  AboveTheStomachDetector,
  BelowTheStomachDetector,
  CollapsingDojiStarDetector,
  DeliberationDetector,
  LastEngulfingBottomDetector,
  LastEngulfingTopDetector,
  TwoBlackGappingDetector,
  MeetingLinesBearishDetector,
  MeetingLinesBullishDetector,
  NorthernDojiDetector,
  SouthernDojiDetector,
  BlackMarubozuDetector,
  WhiteMarubozuDetector,
  OpeningBlackMarubozuDetector,
  OpeningWhiteMarubozuDetector,
  BlackCandleDetector,
  WhiteCandleDetector,
  ShortBlackDetector,
  ShortWhiteDetector,
  LongBlackDayDetector,
  LongWhiteDayDetector,
  BlackSpinningTopDetector,
  WhiteSpinningTopDetector,
  ShootingStar2LinesDetector,
  DownsideGapThreeMethodsDetector,
  UpsideGapThreeMethodsDetector,
  DownsideTasukiGapDetector,
  UpsideTasukiGapDetector,
);

// ============================================================
// PRICE LINES (Consecutive Candles)
// ============================================================

/// Price Lines - N consecutive candles of the same direction
/// Signals overbought/oversold conditions and potential reversal
#[derive(Debug, Clone)]
pub struct PriceLinesDetector {
  /// Number of consecutive candles required
  pub count: usize,
}

impl Default for PriceLinesDetector {
  fn default() -> Self {
    Self { count: 8 }
  }
}

impl PriceLinesDetector {
  pub fn with_count(count: usize) -> Self {
    Self { count: count.max(3) }
  }

  /// Create detector for 8 price lines
  pub fn eight() -> Self {
    Self { count: 8 }
  }

  /// Create detector for 10 price lines
  pub fn ten() -> Self {
    Self { count: 10 }
  }

  /// Create detector for 12 price lines
  pub fn twelve() -> Self {
    Self { count: 12 }
  }

  /// Create detector for 13 price lines
  pub fn thirteen() -> Self {
    Self { count: 13 }
  }
}

impl PatternDetector for PriceLinesDetector {
  fn id(&self) -> PatternId {
    PatternId("PRICE_LINES")
  }

  fn min_bars(&self) -> usize {
    self.count
  }

  fn detect<T: OHLCV>(
    &self,
    bars: &[T],
    index: usize,
    _ctx: &MarketContext,
  ) -> Option<PatternMatch> {
    if index < self.count - 1 {
      return None;
    }

    let start = index + 1 - self.count;

    // Check if all candles are bullish
    let all_bullish = (start..=index).all(|i| bars.get(i).map(|b| b.is_bullish()).unwrap_or(false));

    if all_bullish {
      let strength = 0.5 + (self.count as f64 - 8.0) * 0.05;
      return Some(PatternMatch {
        pattern_id:  PatternDetector::id(self),
        direction:   Direction::Bearish, // Overbought -> potential reversal down
        strength:    strength.clamp(0.5, 1.0),
        start_index: start,
        end_index:   index,
      });
    }

    // Check if all candles are bearish
    let all_bearish = (start..=index).all(|i| bars.get(i).map(|b| b.is_bearish()).unwrap_or(false));

    if all_bearish {
      let strength = 0.5 + (self.count as f64 - 8.0) * 0.05;
      return Some(PatternMatch {
        pattern_id:  PatternDetector::id(self),
        direction:   Direction::Bullish, // Oversold -> potential reversal up
        strength:    strength.clamp(0.5, 1.0),
        start_index: start,
        end_index:   index,
      });
    }

    None
  }
}

// ============================================================
// WINDOWS (Gaps)
// ============================================================

/// Falling Window - Gap down between two candles (bearish continuation)
/// Current High < Previous Low
#[derive(Debug, Clone, Copy, Default)]
pub struct FallingWindowDetector;

impl PatternDetector for FallingWindowDetector {
  fn id(&self) -> PatternId {
    PatternId("FALLING_WINDOW")
  }

  fn min_bars(&self) -> usize {
    2
  }

  fn detect<T: OHLCV>(
    &self,
    bars: &[T],
    index: usize,
    _ctx: &MarketContext,
  ) -> Option<PatternMatch> {
    if index < 1 {
      return None;
    }

    let prev = bars.get(index - 1)?;
    let curr = bars.get(index)?;

    // Gap down: current high is below previous low
    if curr.high() < prev.low() {
      let gap_size = prev.low() - curr.high();
      let avg_range = (prev.range() + curr.range()) / 2.0;
      let strength =
        if avg_range > 0.0 { (gap_size / avg_range).min(1.0) * 0.5 + 0.5 } else { 0.7 };

      return Some(PatternMatch {
        pattern_id:  PatternDetector::id(self),
        direction:   Direction::Bearish,
        strength:    strength.clamp(0.5, 1.0),
        start_index: index - 1,
        end_index:   index,
      });
    }

    None
  }
}

/// Rising Window - Gap up between two candles (bullish continuation)
/// Current Low > Previous High
#[derive(Debug, Clone, Copy, Default)]
pub struct RisingWindowDetector;

impl PatternDetector for RisingWindowDetector {
  fn id(&self) -> PatternId {
    PatternId("RISING_WINDOW")
  }

  fn min_bars(&self) -> usize {
    2
  }

  fn detect<T: OHLCV>(
    &self,
    bars: &[T],
    index: usize,
    _ctx: &MarketContext,
  ) -> Option<PatternMatch> {
    if index < 1 {
      return None;
    }

    let prev = bars.get(index - 1)?;
    let curr = bars.get(index)?;

    // Gap up: current low is above previous high
    if curr.low() > prev.high() {
      let gap_size = curr.low() - prev.high();
      let avg_range = (prev.range() + curr.range()) / 2.0;
      let strength =
        if avg_range > 0.0 { (gap_size / avg_range).min(1.0) * 0.5 + 0.5 } else { 0.7 };

      return Some(PatternMatch {
        pattern_id:  PatternDetector::id(self),
        direction:   Direction::Bullish,
        strength:    strength.clamp(0.5, 1.0),
        start_index: index - 1,
        end_index:   index,
      });
    }

    None
  }
}

/// Gapping Down Doji - Doji that opened with a gap down
/// Signals indecision after decline
#[derive(Debug, Clone)]
pub struct GappingDownDojiDetector {
  /// Maximum body size as ratio of range
  pub body_pct: Ratio,
}

impl Default for GappingDownDojiDetector {
  fn default() -> Self {
    Self { body_pct: Ratio::new_const(0.1) }
  }
}

impl PatternDetector for GappingDownDojiDetector {
  fn id(&self) -> PatternId {
    PatternId("GAPPING_DOWN_DOJI")
  }

  fn min_bars(&self) -> usize {
    2
  }

  fn detect<T: OHLCV>(
    &self,
    bars: &[T],
    index: usize,
    _ctx: &MarketContext,
  ) -> Option<PatternMatch> {
    if index < 1 {
      return None;
    }

    let prev = bars.get(index - 1)?;
    let curr = bars.get(index)?;

    let range = curr.range();
    if range <= f64::EPSILON {
      return None;
    }

    // Check if current is a doji
    let body_ratio = curr.body() / range;
    if body_ratio > self.body_pct.get() {
      return None;
    }

    // Check for gap down: entire candle gaps below previous low
    if curr.high() < prev.low() {
      return Some(PatternMatch {
        pattern_id:  PatternDetector::id(self),
        direction:   Direction::Neutral, // Indecision
        strength:    0.6 + (1.0 - body_ratio) * 0.3,
        start_index: index - 1,
        end_index:   index,
      });
    }

    None
  }
}

/// Gapping Up Doji - Doji that opened with a gap up
/// Signals indecision after advance
#[derive(Debug, Clone)]
pub struct GappingUpDojiDetector {
  /// Maximum body size as ratio of range
  pub body_pct: Ratio,
}

impl Default for GappingUpDojiDetector {
  fn default() -> Self {
    Self { body_pct: Ratio::new_const(0.1) }
  }
}

impl PatternDetector for GappingUpDojiDetector {
  fn id(&self) -> PatternId {
    PatternId("GAPPING_UP_DOJI")
  }

  fn min_bars(&self) -> usize {
    2
  }

  fn detect<T: OHLCV>(
    &self,
    bars: &[T],
    index: usize,
    _ctx: &MarketContext,
  ) -> Option<PatternMatch> {
    if index < 1 {
      return None;
    }

    let prev = bars.get(index - 1)?;
    let curr = bars.get(index)?;

    let range = curr.range();
    if range <= f64::EPSILON {
      return None;
    }

    // Check if current is a doji
    let body_ratio = curr.body() / range;
    if body_ratio > self.body_pct.get() {
      return None;
    }

    // Check for gap up: entire candle gaps above previous high
    if curr.low() > prev.high() {
      return Some(PatternMatch {
        pattern_id:  PatternDetector::id(self),
        direction:   Direction::Neutral, // Indecision
        strength:    0.6 + (1.0 - body_ratio) * 0.3,
        start_index: index - 1,
        end_index:   index,
      });
    }

    None
  }
}

// ============================================================
// REVERSAL PATTERNS
// ============================================================

/// Above the Stomach - Bullish reversal
/// White candle opens above midpoint of previous black candle's body
#[derive(Debug, Clone)]
pub struct AboveTheStomachDetector {
  /// Minimum penetration level (how far above midpoint)
  pub penetration: Ratio,
}

impl Default for AboveTheStomachDetector {
  fn default() -> Self {
    Self { penetration: Ratio::new_const(0.0) }
  }
}

impl PatternDetector for AboveTheStomachDetector {
  fn id(&self) -> PatternId {
    PatternId("ABOVE_THE_STOMACH")
  }

  fn min_bars(&self) -> usize {
    2
  }

  fn detect<T: OHLCV>(
    &self,
    bars: &[T],
    index: usize,
    ctx: &MarketContext,
  ) -> Option<PatternMatch> {
    if index < 1 {
      return None;
    }

    let prev = bars.get(index - 1)?;
    let curr = bars.get(index)?;

    // Previous must be bearish, current must be bullish
    if !prev.is_bearish() || !curr.is_bullish() {
      return None;
    }

    // Need downtrend context
    if !ctx.trend.is_down() {
      return None;
    }

    // Calculate midpoint of previous body
    let prev_midpoint = (prev.open() + prev.close()) / 2.0;
    let penetration_level = prev_midpoint + prev.body() * self.penetration.get();

    // Current opens above the midpoint (stomach)
    if curr.open() >= penetration_level {
      let strength = 0.6 + (curr.open() - prev_midpoint) / prev.body().max(f64::EPSILON) * 0.2;
      return Some(PatternMatch {
        pattern_id:  PatternDetector::id(self),
        direction:   Direction::Bullish,
        strength:    strength.clamp(0.5, 1.0),
        start_index: index - 1,
        end_index:   index,
      });
    }

    None
  }
}

/// Below the Stomach - Bearish reversal
/// Black candle opens below midpoint of previous white candle's body
#[derive(Debug, Clone)]
pub struct BelowTheStomachDetector {
  /// Minimum penetration level (how far below midpoint)
  pub penetration: Ratio,
}

impl Default for BelowTheStomachDetector {
  fn default() -> Self {
    Self { penetration: Ratio::new_const(0.0) }
  }
}

impl PatternDetector for BelowTheStomachDetector {
  fn id(&self) -> PatternId {
    PatternId("BELOW_THE_STOMACH")
  }

  fn min_bars(&self) -> usize {
    2
  }

  fn detect<T: OHLCV>(
    &self,
    bars: &[T],
    index: usize,
    ctx: &MarketContext,
  ) -> Option<PatternMatch> {
    if index < 1 {
      return None;
    }

    let prev = bars.get(index - 1)?;
    let curr = bars.get(index)?;

    // Previous must be bullish, current must be bearish
    if !prev.is_bullish() || !curr.is_bearish() {
      return None;
    }

    // Need uptrend context
    if !ctx.trend.is_up() {
      return None;
    }

    // Calculate midpoint of previous body
    let prev_midpoint = (prev.open() + prev.close()) / 2.0;
    let penetration_level = prev_midpoint - prev.body() * self.penetration.get();

    // Current opens below the midpoint (stomach)
    if curr.open() <= penetration_level {
      let strength = 0.6 + (prev_midpoint - curr.open()) / prev.body().max(f64::EPSILON) * 0.2;
      return Some(PatternMatch {
        pattern_id:  PatternDetector::id(self),
        direction:   Direction::Bearish,
        strength:    strength.clamp(0.5, 1.0),
        start_index: index - 1,
        end_index:   index,
      });
    }

    None
  }
}

/// Collapsing Doji Star - Doji star with gap after strong candle, then reversal
#[derive(Debug, Clone)]
pub struct CollapsingDojiStarDetector {
  /// Maximum body size for doji
  pub body_pct: Ratio,
  /// Minimum gap size as ratio of previous body
  pub gap_pct:  Ratio,
}

impl Default for CollapsingDojiStarDetector {
  fn default() -> Self {
    Self { body_pct: Ratio::new_const(0.1), gap_pct: Ratio::new_const(0.005) }
  }
}

impl PatternDetector for CollapsingDojiStarDetector {
  fn id(&self) -> PatternId {
    PatternId("COLLAPSING_DOJI_STAR")
  }

  fn min_bars(&self) -> usize {
    3
  }

  fn detect<T: OHLCV>(
    &self,
    bars: &[T],
    index: usize,
    _ctx: &MarketContext,
  ) -> Option<PatternMatch> {
    if index < 2 {
      return None;
    }

    let first = bars.get(index - 2)?;
    let doji = bars.get(index - 1)?;
    let third = bars.get(index)?;

    let doji_range = doji.range();
    if doji_range <= f64::EPSILON {
      return None;
    }

    // Middle candle must be a doji
    let doji_body_ratio = doji.body() / doji_range;
    if doji_body_ratio > self.body_pct.get() {
      return None;
    }

    let first_body = first.body();
    if first_body <= f64::EPSILON {
      return None;
    }

    // Bullish collapse (after downtrend)
    if first.is_bearish() {
      // Gap down to doji
      let gap = first.close().min(first.open()) - doji.high();
      if gap >= first_body * self.gap_pct.get() {
        // Third candle is bullish and closes into first candle's body
        if third.is_bullish() && third.close() > first.close() {
          return Some(PatternMatch {
            pattern_id:  PatternDetector::id(self),
            direction:   Direction::Bullish,
            strength:    0.7,
            start_index: index - 2,
            end_index:   index,
          });
        }
      }
    }

    // Bearish collapse (after uptrend)
    if first.is_bullish() {
      // Gap up to doji
      let gap = doji.low() - first.close().max(first.open());
      if gap >= first_body * self.gap_pct.get() {
        // Third candle is bearish and closes into first candle's body
        if third.is_bearish() && third.close() < first.close() {
          return Some(PatternMatch {
            pattern_id:  PatternDetector::id(self),
            direction:   Direction::Bearish,
            strength:    0.7,
            start_index: index - 2,
            end_index:   index,
          });
        }
      }
    }

    None
  }
}

/// Deliberation - Three white candles, third with small body (weakening trend)
#[derive(Debug, Clone)]
pub struct DeliberationDetector {
  /// Maximum body size for third candle as ratio of first two average
  pub body_pct:      Ratio,
  /// Minimum body ratio for first and second candles (long body requirement)
  pub long_body_pct: Ratio,
}

impl Default for DeliberationDetector {
  fn default() -> Self {
    Self { body_pct: Ratio::new_const(0.3), long_body_pct: Ratio::new_const(0.6) }
  }
}

impl PatternDetector for DeliberationDetector {
  fn id(&self) -> PatternId {
    PatternId("DELIBERATION")
  }

  fn min_bars(&self) -> usize {
    3
  }

  fn detect<T: OHLCV>(
    &self,
    bars: &[T],
    index: usize,
    ctx: &MarketContext,
  ) -> Option<PatternMatch> {
    if index < 2 {
      return None;
    }

    let first = bars.get(index - 2)?;
    let second = bars.get(index - 1)?;
    let third = bars.get(index)?;

    // All three must be bullish
    if !first.is_bullish() || !second.is_bullish() || !third.is_bullish() {
      return None;
    }

    // Need uptrend context
    if !ctx.trend.is_up() {
      return None;
    }

    // Each candle should close higher
    if second.close() <= first.close() || third.close() <= second.close() {
      return None;
    }

    // First and second candles must have long bodies (Nison requirement)
    let first_range = first.range();
    let second_range = second.range();
    if first_range <= f64::EPSILON || second_range <= f64::EPSILON {
      return None;
    }
    if first.body() / first_range < self.long_body_pct.get()
      || second.body() / second_range < self.long_body_pct.get()
    {
      return None;
    }

    // Third candle should open at or near second's close (or gap up)
    // Tolerance: within 10% of second's body below second's close
    let tolerance = second.body() * 0.1;
    if third.open() < second.close() - tolerance {
      return None;
    }

    let avg_body = (first.body() + second.body()) / 2.0;
    if avg_body <= f64::EPSILON {
      return None;
    }

    // Third candle has small body compared to first two
    if third.body() / avg_body <= self.body_pct.get() {
      return Some(PatternMatch {
        pattern_id:  PatternDetector::id(self),
        direction:   Direction::Bearish, // Weakening bullish momentum
        strength:    0.6 + (1.0 - third.body() / avg_body) * 0.3,
        start_index: index - 2,
        end_index:   index,
      });
    }

    None
  }
}

/// Last Engulfing Bottom - Final engulfing at bottom of downtrend
#[derive(Debug, Clone)]
pub struct LastEngulfingBottomDetector {
  /// Period for trend detection
  pub trend_period: Period,
}

impl Default for LastEngulfingBottomDetector {
  fn default() -> Self {
    Self { trend_period: Period::new_const(14) }
  }
}

impl PatternDetector for LastEngulfingBottomDetector {
  fn id(&self) -> PatternId {
    PatternId("LAST_ENGULFING_BOTTOM")
  }

  fn min_bars(&self) -> usize {
    2
  }

  fn detect<T: OHLCV>(
    &self,
    bars: &[T],
    index: usize,
    ctx: &MarketContext,
  ) -> Option<PatternMatch> {
    if index < 1 {
      return None;
    }

    let prev = bars.get(index - 1)?;
    let curr = bars.get(index)?;

    // Need strong downtrend
    if !ctx.trend.is_down() {
      return None;
    }

    // Bullish engulfing pattern
    if !prev.is_bearish() || !curr.is_bullish() {
      return None;
    }

    // Current engulfs previous
    if curr.open() <= prev.close() && curr.close() >= prev.open() && curr.body() > prev.body() {
      return Some(PatternMatch {
        pattern_id:  PatternDetector::id(self),
        direction:   Direction::Bullish,
        strength:    0.75,
        start_index: index - 1,
        end_index:   index,
      });
    }

    None
  }
}

/// Last Engulfing Top - Final engulfing at top of uptrend
#[derive(Debug, Clone)]
pub struct LastEngulfingTopDetector {
  /// Period for trend detection
  pub trend_period: Period,
}

impl Default for LastEngulfingTopDetector {
  fn default() -> Self {
    Self { trend_period: Period::new_const(14) }
  }
}

impl PatternDetector for LastEngulfingTopDetector {
  fn id(&self) -> PatternId {
    PatternId("LAST_ENGULFING_TOP")
  }

  fn min_bars(&self) -> usize {
    2
  }

  fn detect<T: OHLCV>(
    &self,
    bars: &[T],
    index: usize,
    ctx: &MarketContext,
  ) -> Option<PatternMatch> {
    if index < 1 {
      return None;
    }

    let prev = bars.get(index - 1)?;
    let curr = bars.get(index)?;

    // Need strong uptrend
    if !ctx.trend.is_up() {
      return None;
    }

    // Bearish engulfing pattern
    if !prev.is_bullish() || !curr.is_bearish() {
      return None;
    }

    // Current engulfs previous
    if curr.open() >= prev.close() && curr.close() <= prev.open() && curr.body() > prev.body() {
      return Some(PatternMatch {
        pattern_id:  PatternDetector::id(self),
        direction:   Direction::Bearish,
        strength:    0.75,
        start_index: index - 1,
        end_index:   index,
      });
    }

    None
  }
}

/// Two Black Gapping - Two black candles with gap down between them
#[derive(Debug, Clone, Copy, Default)]
pub struct TwoBlackGappingDetector;

impl PatternDetector for TwoBlackGappingDetector {
  fn id(&self) -> PatternId {
    PatternId("TWO_BLACK_GAPPING")
  }

  fn min_bars(&self) -> usize {
    3
  }

  fn detect<T: OHLCV>(
    &self,
    bars: &[T],
    index: usize,
    _ctx: &MarketContext,
  ) -> Option<PatternMatch> {
    if index < 2 {
      return None;
    }

    let prior = bars.get(index - 2)?;
    let first = bars.get(index - 1)?;
    let second = bars.get(index)?;

    // Both candles in the pair must be bearish
    if !first.is_bearish() || !second.is_bearish() {
      return None;
    }

    // Gap down from prior candle to the first black candle
    if first.high() >= prior.low() {
      return None;
    }

    // Second black candle continues bearish (closes lower or equal)
    if second.close() > first.close() {
      return None;
    }

    let gap_size = prior.low() - first.high();
    let avg_body = (first.body() + second.body()) / 2.0;
    let strength = if avg_body > 0.0 { 0.6 + (gap_size / avg_body).min(0.4) } else { 0.7 };

    Some(PatternMatch {
      pattern_id:  PatternDetector::id(self),
      direction:   Direction::Bearish,
      strength:    strength.clamp(0.5, 1.0),
      start_index: index - 2,
      end_index:   index,
    })
  }
}

/// Meeting Lines Bearish - White candle followed by black with same close
#[derive(Debug, Clone)]
pub struct MeetingLinesBearishDetector {
  /// Tolerance for price equality
  pub tolerance: Ratio,
  /// Minimum body size as ratio of range (long body requirement)
  pub body_pct:  Ratio,
}

impl Default for MeetingLinesBearishDetector {
  fn default() -> Self {
    Self { tolerance: Ratio::new_const(0.001), body_pct: Ratio::new_const(0.6) }
  }
}

impl PatternDetector for MeetingLinesBearishDetector {
  fn id(&self) -> PatternId {
    PatternId("MEETING_LINES_BEARISH")
  }

  fn min_bars(&self) -> usize {
    2
  }

  fn detect<T: OHLCV>(
    &self,
    bars: &[T],
    index: usize,
    ctx: &MarketContext,
  ) -> Option<PatternMatch> {
    if index < 1 {
      return None;
    }

    let prev = bars.get(index - 1)?;
    let curr = bars.get(index)?;

    // Need uptrend
    if !ctx.trend.is_up() {
      return None;
    }

    // Previous bullish, current bearish
    if !prev.is_bullish() || !curr.is_bearish() {
      return None;
    }

    // Both candles must have long bodies (Nison/Bulkowski requirement)
    let prev_range = prev.range();
    let curr_range = curr.range();
    if prev_range <= f64::EPSILON || curr_range <= f64::EPSILON {
      return None;
    }
    if prev.body() / prev_range < self.body_pct.get()
      || curr.body() / curr_range < self.body_pct.get()
    {
      return None;
    }

    // Closes meet at same level
    let close_diff = (prev.close() - curr.close()).abs();
    let tolerance_val = prev.close() * self.tolerance.get();

    if close_diff <= tolerance_val {
      return Some(PatternMatch {
        pattern_id:  PatternDetector::id(self),
        direction:   Direction::Bearish,
        strength:    0.7 - (close_diff / tolerance_val.max(f64::EPSILON)) * 0.1,
        start_index: index - 1,
        end_index:   index,
      });
    }

    None
  }
}

/// Meeting Lines Bullish - Black candle followed by white with same close
#[derive(Debug, Clone)]
pub struct MeetingLinesBullishDetector {
  /// Tolerance for price equality
  pub tolerance: Ratio,
  /// Minimum body size as ratio of range (long body requirement)
  pub body_pct:  Ratio,
}

impl Default for MeetingLinesBullishDetector {
  fn default() -> Self {
    Self { tolerance: Ratio::new_const(0.001), body_pct: Ratio::new_const(0.6) }
  }
}

impl PatternDetector for MeetingLinesBullishDetector {
  fn id(&self) -> PatternId {
    PatternId("MEETING_LINES_BULLISH")
  }

  fn min_bars(&self) -> usize {
    2
  }

  fn detect<T: OHLCV>(
    &self,
    bars: &[T],
    index: usize,
    ctx: &MarketContext,
  ) -> Option<PatternMatch> {
    if index < 1 {
      return None;
    }

    let prev = bars.get(index - 1)?;
    let curr = bars.get(index)?;

    // Need downtrend
    if !ctx.trend.is_down() {
      return None;
    }

    // Previous bearish, current bullish
    if !prev.is_bearish() || !curr.is_bullish() {
      return None;
    }

    // Both candles must have long bodies (Nison/Bulkowski requirement)
    let prev_range = prev.range();
    let curr_range = curr.range();
    if prev_range <= f64::EPSILON || curr_range <= f64::EPSILON {
      return None;
    }
    if prev.body() / prev_range < self.body_pct.get()
      || curr.body() / curr_range < self.body_pct.get()
    {
      return None;
    }

    // Closes meet at same level
    let close_diff = (prev.close() - curr.close()).abs();
    let tolerance_val = prev.close() * self.tolerance.get();

    if close_diff <= tolerance_val {
      return Some(PatternMatch {
        pattern_id:  PatternDetector::id(self),
        direction:   Direction::Bullish,
        strength:    0.7 - (close_diff / tolerance_val.max(f64::EPSILON)) * 0.1,
        start_index: index - 1,
        end_index:   index,
      });
    }

    None
  }
}

// ============================================================
// DOJI VARIANTS
// ============================================================

/// Northern Doji - Doji in upper part of trading range (after advance)
#[derive(Debug, Clone)]
pub struct NorthernDojiDetector {
  /// Maximum body size as ratio of range
  pub body_pct:     Ratio,
  /// Period for trend detection
  pub trend_period: Period,
}

impl Default for NorthernDojiDetector {
  fn default() -> Self {
    Self { body_pct: Ratio::new_const(0.1), trend_period: Period::new_const(14) }
  }
}

impl PatternDetector for NorthernDojiDetector {
  fn id(&self) -> PatternId {
    PatternId("NORTHERN_DOJI")
  }

  fn min_bars(&self) -> usize {
    1
  }

  fn detect<T: OHLCV>(
    &self,
    bars: &[T],
    index: usize,
    ctx: &MarketContext,
  ) -> Option<PatternMatch> {
    let bar = bars.get(index)?;

    let range = bar.range();
    if range <= f64::EPSILON {
      return None;
    }

    // Check if it's a doji
    let body_ratio = bar.body() / range;
    if body_ratio > self.body_pct.get() {
      return None;
    }

    // Must be in uptrend (northern = upper part of range)
    if !ctx.trend.is_up() {
      return None;
    }

    Some(PatternMatch {
      pattern_id:  PatternDetector::id(self),
      direction:   Direction::Bearish, // Potential reversal
      strength:    0.6 + (1.0 - body_ratio) * 0.3,
      start_index: index,
      end_index:   index,
    })
  }
}

/// Southern Doji - Doji in lower part of trading range (after decline)
#[derive(Debug, Clone)]
pub struct SouthernDojiDetector {
  /// Maximum body size as ratio of range
  pub body_pct:     Ratio,
  /// Period for trend detection
  pub trend_period: Period,
}

impl Default for SouthernDojiDetector {
  fn default() -> Self {
    Self { body_pct: Ratio::new_const(0.1), trend_period: Period::new_const(14) }
  }
}

impl PatternDetector for SouthernDojiDetector {
  fn id(&self) -> PatternId {
    PatternId("SOUTHERN_DOJI")
  }

  fn min_bars(&self) -> usize {
    1
  }

  fn detect<T: OHLCV>(
    &self,
    bars: &[T],
    index: usize,
    ctx: &MarketContext,
  ) -> Option<PatternMatch> {
    let bar = bars.get(index)?;

    let range = bar.range();
    if range <= f64::EPSILON {
      return None;
    }

    // Check if it's a doji
    let body_ratio = bar.body() / range;
    if body_ratio > self.body_pct.get() {
      return None;
    }

    // Must be in downtrend (southern = lower part of range)
    if !ctx.trend.is_down() {
      return None;
    }

    Some(PatternMatch {
      pattern_id:  PatternDetector::id(self),
      direction:   Direction::Bullish, // Potential reversal
      strength:    0.6 + (1.0 - body_ratio) * 0.3,
      start_index: index,
      end_index:   index,
    })
  }
}

// ============================================================
// MARUBOZU VARIANTS
// ============================================================

/// Black Marubozu - Black candle with no shadows (Open=High, Close=Low)
#[derive(Debug, Clone)]
pub struct BlackMarubozuDetector {
  /// Maximum shadow tolerance as ratio of range
  pub shadow_tolerance: Ratio,
}

impl Default for BlackMarubozuDetector {
  fn default() -> Self {
    Self { shadow_tolerance: Ratio::new_const(0.01) }
  }
}

impl PatternDetector for BlackMarubozuDetector {
  fn id(&self) -> PatternId {
    PatternId("BLACK_MARUBOZU")
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

    // Must be bearish
    if !bar.is_bearish() {
      return None;
    }

    let range = bar.range();
    if range <= f64::EPSILON {
      return None;
    }

    let tolerance = range * self.shadow_tolerance.get();

    // Open = High (no upper shadow)
    let upper_shadow = bar.high() - bar.open();
    // Close = Low (no lower shadow)
    let lower_shadow = bar.close() - bar.low();

    if upper_shadow <= tolerance && lower_shadow <= tolerance {
      return Some(PatternMatch {
        pattern_id:  PatternDetector::id(self),
        direction:   Direction::Bearish,
        strength:    0.8 + (1.0 - (upper_shadow + lower_shadow) / range) * 0.2,
        start_index: index,
        end_index:   index,
      });
    }

    None
  }
}

/// White Marubozu - White candle with no shadows (Open=Low, Close=High)
#[derive(Debug, Clone)]
pub struct WhiteMarubozuDetector {
  /// Maximum shadow tolerance as ratio of range
  pub shadow_tolerance: Ratio,
}

impl Default for WhiteMarubozuDetector {
  fn default() -> Self {
    Self { shadow_tolerance: Ratio::new_const(0.01) }
  }
}

impl PatternDetector for WhiteMarubozuDetector {
  fn id(&self) -> PatternId {
    PatternId("WHITE_MARUBOZU")
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

    // Must be bullish
    if !bar.is_bullish() {
      return None;
    }

    let range = bar.range();
    if range <= f64::EPSILON {
      return None;
    }

    let tolerance = range * self.shadow_tolerance.get();

    // Open = Low (no lower shadow)
    let lower_shadow = bar.open() - bar.low();
    // Close = High (no upper shadow)
    let upper_shadow = bar.high() - bar.close();

    if upper_shadow <= tolerance && lower_shadow <= tolerance {
      return Some(PatternMatch {
        pattern_id:  PatternDetector::id(self),
        direction:   Direction::Bullish,
        strength:    0.8 + (1.0 - (upper_shadow + lower_shadow) / range) * 0.2,
        start_index: index,
        end_index:   index,
      });
    }

    None
  }
}

/// Opening Black Marubozu - Black candle with no upper shadow (Open=High), has lower shadow
#[derive(Debug, Clone)]
pub struct OpeningBlackMarubozuDetector {
  /// Maximum upper shadow tolerance as ratio of range
  pub shadow_tolerance: Ratio,
}

impl Default for OpeningBlackMarubozuDetector {
  fn default() -> Self {
    Self { shadow_tolerance: Ratio::new_const(0.01) }
  }
}

impl PatternDetector for OpeningBlackMarubozuDetector {
  fn id(&self) -> PatternId {
    PatternId("OPENING_BLACK_MARUBOZU")
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

    // Must be bearish
    if !bar.is_bearish() {
      return None;
    }

    let range = bar.range();
    if range <= f64::EPSILON {
      return None;
    }

    let tolerance = range * self.shadow_tolerance.get();

    // Open = High (no upper shadow)
    let upper_shadow = bar.high() - bar.open();
    // Has lower shadow (Close > Low)
    let lower_shadow = bar.close() - bar.low();

    if upper_shadow <= tolerance && lower_shadow > tolerance {
      return Some(PatternMatch {
        pattern_id:  PatternDetector::id(self),
        direction:   Direction::Bearish,
        strength:    0.7,
        start_index: index,
        end_index:   index,
      });
    }

    None
  }
}

/// Opening White Marubozu - White candle with no lower shadow (Open=Low), has upper shadow
#[derive(Debug, Clone)]
pub struct OpeningWhiteMarubozuDetector {
  /// Maximum lower shadow tolerance as ratio of range
  pub shadow_tolerance: Ratio,
}

impl Default for OpeningWhiteMarubozuDetector {
  fn default() -> Self {
    Self { shadow_tolerance: Ratio::new_const(0.01) }
  }
}

impl PatternDetector for OpeningWhiteMarubozuDetector {
  fn id(&self) -> PatternId {
    PatternId("OPENING_WHITE_MARUBOZU")
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

    // Must be bullish
    if !bar.is_bullish() {
      return None;
    }

    let range = bar.range();
    if range <= f64::EPSILON {
      return None;
    }

    let tolerance = range * self.shadow_tolerance.get();

    // Open = Low (no lower shadow)
    let lower_shadow = bar.open() - bar.low();
    // Has upper shadow (Close < High)
    let upper_shadow = bar.high() - bar.close();

    if lower_shadow <= tolerance && upper_shadow > tolerance {
      return Some(PatternMatch {
        pattern_id:  PatternDetector::id(self),
        direction:   Direction::Bullish,
        strength:    0.7,
        start_index: index,
        end_index:   index,
      });
    }

    None
  }
}

// ============================================================
// BASIC CANDLE PATTERNS
// ============================================================

/// Black Candle - Simple bearish candle (Close < Open)
#[derive(Debug, Clone, Copy, Default)]
pub struct BlackCandleDetector;

impl PatternDetector for BlackCandleDetector {
  fn id(&self) -> PatternId {
    PatternId("BLACK_CANDLE")
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

    if bar.is_bearish() {
      return Some(PatternMatch {
        pattern_id:  PatternDetector::id(self),
        direction:   Direction::Bearish,
        strength:    0.5,
        start_index: index,
        end_index:   index,
      });
    }

    None
  }
}

/// White Candle - Simple bullish candle (Close > Open)
#[derive(Debug, Clone, Copy, Default)]
pub struct WhiteCandleDetector;

impl PatternDetector for WhiteCandleDetector {
  fn id(&self) -> PatternId {
    PatternId("WHITE_CANDLE")
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

    if bar.is_bullish() {
      return Some(PatternMatch {
        pattern_id:  PatternDetector::id(self),
        direction:   Direction::Bullish,
        strength:    0.5,
        start_index: index,
        end_index:   index,
      });
    }

    None
  }
}

/// Short Black - Bearish candle with short body
#[derive(Debug, Clone)]
pub struct ShortBlackDetector {
  /// Maximum body size as ratio of ATR or range
  pub body_pct: Ratio,
}

impl Default for ShortBlackDetector {
  fn default() -> Self {
    Self { body_pct: Ratio::new_const(0.3) }
  }
}

impl PatternDetector for ShortBlackDetector {
  fn id(&self) -> PatternId {
    PatternId("SHORT_BLACK")
  }

  fn min_bars(&self) -> usize {
    1
  }

  fn detect<T: OHLCV>(
    &self,
    bars: &[T],
    index: usize,
    ctx: &MarketContext,
  ) -> Option<PatternMatch> {
    let bar = bars.get(index)?;

    if !bar.is_bearish() {
      return None;
    }

    let reference = if ctx.volatility > 0.0 { ctx.volatility } else { bar.range() };
    if reference <= f64::EPSILON {
      return None;
    }

    if bar.body() / reference <= self.body_pct.get() {
      return Some(PatternMatch {
        pattern_id:  PatternDetector::id(self),
        direction:   Direction::Bearish,
        strength:    0.5,
        start_index: index,
        end_index:   index,
      });
    }

    None
  }
}

/// Short White - Bullish candle with short body
#[derive(Debug, Clone)]
pub struct ShortWhiteDetector {
  /// Maximum body size as ratio of ATR or range
  pub body_pct: Ratio,
}

impl Default for ShortWhiteDetector {
  fn default() -> Self {
    Self { body_pct: Ratio::new_const(0.3) }
  }
}

impl PatternDetector for ShortWhiteDetector {
  fn id(&self) -> PatternId {
    PatternId("SHORT_WHITE")
  }

  fn min_bars(&self) -> usize {
    1
  }

  fn detect<T: OHLCV>(
    &self,
    bars: &[T],
    index: usize,
    ctx: &MarketContext,
  ) -> Option<PatternMatch> {
    let bar = bars.get(index)?;

    if !bar.is_bullish() {
      return None;
    }

    let reference = if ctx.volatility > 0.0 { ctx.volatility } else { bar.range() };
    if reference <= f64::EPSILON {
      return None;
    }

    if bar.body() / reference <= self.body_pct.get() {
      return Some(PatternMatch {
        pattern_id:  PatternDetector::id(self),
        direction:   Direction::Bullish,
        strength:    0.5,
        start_index: index,
        end_index:   index,
      });
    }

    None
  }
}

/// Long Black Day - Bearish candle with long body
#[derive(Debug, Clone)]
pub struct LongBlackDayDetector {
  /// Minimum body size as ratio of range (typically >70%)
  pub body_pct: Ratio,
}

impl Default for LongBlackDayDetector {
  fn default() -> Self {
    Self { body_pct: Ratio::new_const(0.7) }
  }
}

impl PatternDetector for LongBlackDayDetector {
  fn id(&self) -> PatternId {
    PatternId("LONG_BLACK_DAY")
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

    if !bar.is_bearish() {
      return None;
    }

    let range = bar.range();
    if range <= f64::EPSILON {
      return None;
    }

    let body_ratio = bar.body() / range;
    if body_ratio >= self.body_pct.get() {
      return Some(PatternMatch {
        pattern_id:  PatternDetector::id(self),
        direction:   Direction::Bearish,
        strength:    0.6 + body_ratio * 0.3,
        start_index: index,
        end_index:   index,
      });
    }

    None
  }
}

/// Long White Day - Bullish candle with long body
#[derive(Debug, Clone)]
pub struct LongWhiteDayDetector {
  /// Minimum body size as ratio of range (typically >70%)
  pub body_pct: Ratio,
}

impl Default for LongWhiteDayDetector {
  fn default() -> Self {
    Self { body_pct: Ratio::new_const(0.7) }
  }
}

impl PatternDetector for LongWhiteDayDetector {
  fn id(&self) -> PatternId {
    PatternId("LONG_WHITE_DAY")
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

    if !bar.is_bullish() {
      return None;
    }

    let range = bar.range();
    if range <= f64::EPSILON {
      return None;
    }

    let body_ratio = bar.body() / range;
    if body_ratio >= self.body_pct.get() {
      return Some(PatternMatch {
        pattern_id:  PatternDetector::id(self),
        direction:   Direction::Bullish,
        strength:    0.6 + body_ratio * 0.3,
        start_index: index,
        end_index:   index,
      });
    }

    None
  }
}

/// Black Spinning Top - Black candle with small body and shadows on both sides
#[derive(Debug, Clone)]
pub struct BlackSpinningTopDetector {
  /// Maximum body size as ratio of range
  pub body_pct:     Ratio,
  /// Minimum shadow ratio (each shadow vs body)
  pub shadow_ratio: Ratio,
}

impl Default for BlackSpinningTopDetector {
  fn default() -> Self {
    Self { body_pct: Ratio::new_const(0.3), shadow_ratio: Ratio::new_const(0.5) }
  }
}

impl PatternDetector for BlackSpinningTopDetector {
  fn id(&self) -> PatternId {
    PatternId("BLACK_SPINNING_TOP")
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

    if !bar.is_bearish() {
      return None;
    }

    let range = bar.range();
    if range <= f64::EPSILON {
      return None;
    }

    let body = bar.body();
    let body_ratio = body / range;

    // Small body
    if body_ratio > self.body_pct.get() {
      return None;
    }

    // Shadows on both sides
    let upper_shadow = bar.high() - bar.open();
    let lower_shadow = bar.close() - bar.low();
    let min_shadow = body * self.shadow_ratio.get();

    if upper_shadow >= min_shadow && lower_shadow >= min_shadow {
      return Some(PatternMatch {
        pattern_id:  PatternDetector::id(self),
        direction:   Direction::Neutral, // Indecision
        strength:    0.5 + (1.0 - body_ratio) * 0.3,
        start_index: index,
        end_index:   index,
      });
    }

    None
  }
}

/// White Spinning Top - White candle with small body and shadows on both sides
#[derive(Debug, Clone)]
pub struct WhiteSpinningTopDetector {
  /// Maximum body size as ratio of range
  pub body_pct:     Ratio,
  /// Minimum shadow ratio (each shadow vs body)
  pub shadow_ratio: Ratio,
}

impl Default for WhiteSpinningTopDetector {
  fn default() -> Self {
    Self { body_pct: Ratio::new_const(0.3), shadow_ratio: Ratio::new_const(0.5) }
  }
}

impl PatternDetector for WhiteSpinningTopDetector {
  fn id(&self) -> PatternId {
    PatternId("WHITE_SPINNING_TOP")
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

    if !bar.is_bullish() {
      return None;
    }

    let range = bar.range();
    if range <= f64::EPSILON {
      return None;
    }

    let body = bar.body();
    let body_ratio = body / range;

    // Small body
    if body_ratio > self.body_pct.get() {
      return None;
    }

    // Shadows on both sides
    let upper_shadow = bar.high() - bar.close();
    let lower_shadow = bar.open() - bar.low();
    let min_shadow = body * self.shadow_ratio.get();

    if upper_shadow >= min_shadow && lower_shadow >= min_shadow {
      return Some(PatternMatch {
        pattern_id:  PatternDetector::id(self),
        direction:   Direction::Neutral, // Indecision
        strength:    0.5 + (1.0 - body_ratio) * 0.3,
        start_index: index,
        end_index:   index,
      });
    }

    None
  }
}

// ============================================================
// SHOOTING STAR 2-LINES
// ============================================================

/// Shooting Star 2-Lines - Classic shooting star pattern with trend context
#[derive(Debug, Clone)]
pub struct ShootingStar2LinesDetector {
  /// Maximum body size as ratio of range
  pub body_pct:     Ratio,
  /// Minimum upper shadow to body ratio
  pub shadow_ratio: Ratio,
}

impl Default for ShootingStar2LinesDetector {
  fn default() -> Self {
    Self { body_pct: Ratio::new_const(0.3), shadow_ratio: Ratio::new_const(2.0) }
  }
}

impl PatternDetector for ShootingStar2LinesDetector {
  fn id(&self) -> PatternId {
    PatternId("SHOOTING_STAR_2_LINES")
  }

  fn min_bars(&self) -> usize {
    2
  }

  fn detect<T: OHLCV>(
    &self,
    bars: &[T],
    index: usize,
    ctx: &MarketContext,
  ) -> Option<PatternMatch> {
    if index < 1 {
      return None;
    }

    let prev = bars.get(index - 1)?;
    let curr = bars.get(index)?;

    // Need uptrend
    if !ctx.trend.is_up() {
      return None;
    }

    // Previous should be bullish (continuation of uptrend)
    if !prev.is_bullish() {
      return None;
    }

    let range = curr.range();
    if range <= f64::EPSILON {
      return None;
    }

    let body = curr.body();
    let body_ratio = body / range;

    // Small body
    if body_ratio > self.body_pct.get() {
      return None;
    }

    // Long upper shadow
    let upper_shadow = curr.high() - curr.open().max(curr.close());
    let lower_shadow = curr.open().min(curr.close()) - curr.low();

    // Upper shadow should be at least shadow_ratio times the body
    // Lower shadow should be minimal
    if body > f64::EPSILON
      && upper_shadow / body >= self.shadow_ratio.get()
      && lower_shadow < upper_shadow * 0.3
    {
      return Some(PatternMatch {
        pattern_id:  PatternDetector::id(self),
        direction:   Direction::Bearish,
        strength:    0.7 + (upper_shadow / body - self.shadow_ratio.get()).min(0.3) * 0.1,
        start_index: index - 1,
        end_index:   index,
      });
    }

    None
  }
}

// ============================================================
// GAP THREE METHODS
// ============================================================

/// Downside Gap Three Methods - Gap down, then white candle closes the gap (bearish continuation)
#[derive(Debug, Clone, Copy, Default)]
pub struct DownsideGapThreeMethodsDetector;

impl PatternDetector for DownsideGapThreeMethodsDetector {
  fn id(&self) -> PatternId {
    PatternId("DOWNSIDE_GAP_THREE_METHODS")
  }

  fn min_bars(&self) -> usize {
    3
  }

  fn detect<T: OHLCV>(
    &self,
    bars: &[T],
    index: usize,
    _ctx: &MarketContext,
  ) -> Option<PatternMatch> {
    if index < 2 {
      return None;
    }

    let first = bars.get(index - 2)?;
    let second = bars.get(index - 1)?;
    let third = bars.get(index)?;

    // First two are bearish with gap down between them
    if !first.is_bearish() || !second.is_bearish() {
      return None;
    }

    // Gap down: second high < first low
    if second.high() >= first.low() {
      return None;
    }

    // Third is bullish and closes the gap (opens in second body, closes in first body)
    if !third.is_bullish() {
      return None;
    }

    // Third opens within second candle's body
    if third.open() < second.close() || third.open() > second.open() {
      return None;
    }

    // Third closes within first candle's body (closing the gap)
    if third.close() < first.close() || third.close() > first.open() {
      return None;
    }

    Some(PatternMatch {
      pattern_id:  PatternDetector::id(self),
      direction:   Direction::Bearish, // Continuation pattern
      strength:    0.7,
      start_index: index - 2,
      end_index:   index,
    })
  }
}

/// Upside Gap Three Methods - Gap up, then black candle closes the gap (bullish continuation)
#[derive(Debug, Clone, Copy, Default)]
pub struct UpsideGapThreeMethodsDetector;

impl PatternDetector for UpsideGapThreeMethodsDetector {
  fn id(&self) -> PatternId {
    PatternId("UPSIDE_GAP_THREE_METHODS")
  }

  fn min_bars(&self) -> usize {
    3
  }

  fn detect<T: OHLCV>(
    &self,
    bars: &[T],
    index: usize,
    _ctx: &MarketContext,
  ) -> Option<PatternMatch> {
    if index < 2 {
      return None;
    }

    let first = bars.get(index - 2)?;
    let second = bars.get(index - 1)?;
    let third = bars.get(index)?;

    // First two are bullish with gap up between them
    if !first.is_bullish() || !second.is_bullish() {
      return None;
    }

    // Gap up: second low > first high
    if second.low() <= first.high() {
      return None;
    }

    // Third is bearish and closes the gap (opens in second body, closes in first body)
    if !third.is_bearish() {
      return None;
    }

    // Third opens within second candle's body
    if third.open() > second.close() || third.open() < second.open() {
      return None;
    }

    // Third closes within first candle's body (closing the gap)
    if third.close() > first.close() || third.close() < first.open() {
      return None;
    }

    Some(PatternMatch {
      pattern_id:  PatternDetector::id(self),
      direction:   Direction::Bullish, // Continuation pattern
      strength:    0.7,
      start_index: index - 2,
      end_index:   index,
    })
  }
}

// ============================================================
// TASUKI GAP VARIANTS
// ============================================================

/// Downside Tasuki Gap - Gap down + white candle partially fills gap but doesn't close it
#[derive(Debug, Clone)]
pub struct DownsideTasukiGapDetector {
  /// Maximum gap fill percentage (pattern invalid if gap is fully closed)
  pub gap_fill_pct: Ratio,
}

impl Default for DownsideTasukiGapDetector {
  fn default() -> Self {
    Self { gap_fill_pct: Ratio::new_const(0.7) }
  }
}

impl PatternDetector for DownsideTasukiGapDetector {
  fn id(&self) -> PatternId {
    PatternId("DOWNSIDE_TASUKI_GAP")
  }

  fn min_bars(&self) -> usize {
    3
  }

  fn detect<T: OHLCV>(
    &self,
    bars: &[T],
    index: usize,
    _ctx: &MarketContext,
  ) -> Option<PatternMatch> {
    if index < 2 {
      return None;
    }

    let first = bars.get(index - 2)?;
    let second = bars.get(index - 1)?;
    let third = bars.get(index)?;

    // First two are bearish
    if !first.is_bearish() || !second.is_bearish() {
      return None;
    }

    // Gap down between first and second
    let gap_top = first.low();
    let gap_bottom = second.high();
    if gap_bottom >= gap_top {
      return None;
    }

    let gap_size = gap_top - gap_bottom;

    // Third is bullish
    if !third.is_bullish() {
      return None;
    }

    // Third partially fills the gap but doesn't close it completely
    let fill_amount = third.close() - gap_bottom;
    if fill_amount <= 0.0 {
      return None; // Doesn't enter the gap at all
    }

    let fill_pct = fill_amount / gap_size;
    if fill_pct > self.gap_fill_pct.get() {
      return None; // Gap closed too much
    }

    Some(PatternMatch {
      pattern_id:  PatternDetector::id(self),
      direction:   Direction::Bearish, // Bearish continuation
      strength:    0.65 + (1.0 - fill_pct) * 0.2,
      start_index: index - 2,
      end_index:   index,
    })
  }
}

/// Upside Tasuki Gap - Gap up + black candle partially fills gap but doesn't close it
#[derive(Debug, Clone)]
pub struct UpsideTasukiGapDetector {
  /// Maximum gap fill percentage (pattern invalid if gap is fully closed)
  pub gap_fill_pct: Ratio,
}

impl Default for UpsideTasukiGapDetector {
  fn default() -> Self {
    Self { gap_fill_pct: Ratio::new_const(0.7) }
  }
}

impl PatternDetector for UpsideTasukiGapDetector {
  fn id(&self) -> PatternId {
    PatternId("UPSIDE_TASUKI_GAP")
  }

  fn min_bars(&self) -> usize {
    3
  }

  fn detect<T: OHLCV>(
    &self,
    bars: &[T],
    index: usize,
    _ctx: &MarketContext,
  ) -> Option<PatternMatch> {
    if index < 2 {
      return None;
    }

    let first = bars.get(index - 2)?;
    let second = bars.get(index - 1)?;
    let third = bars.get(index)?;

    // First two are bullish
    if !first.is_bullish() || !second.is_bullish() {
      return None;
    }

    // Gap up between first and second
    let gap_bottom = first.high();
    let gap_top = second.low();
    if gap_top <= gap_bottom {
      return None;
    }

    let gap_size = gap_top - gap_bottom;

    // Third is bearish
    if !third.is_bearish() {
      return None;
    }

    // Third partially fills the gap but doesn't close it completely
    let fill_amount = gap_top - third.close();
    if fill_amount <= 0.0 {
      return None; // Doesn't enter the gap at all
    }

    let fill_pct = fill_amount / gap_size;
    if fill_pct > self.gap_fill_pct.get() {
      return None; // Gap closed too much
    }

    Some(PatternMatch {
      pattern_id:  PatternDetector::id(self),
      direction:   Direction::Bullish, // Bullish continuation
      strength:    0.65 + (1.0 - fill_pct) * 0.2,
      start_index: index - 2,
      end_index:   index,
    })
  }
}

// ============================================================
// PARAMETERIZED DETECTOR IMPLEMENTATIONS
// ============================================================

static GAPPING_DOWN_DOJI_PARAMS: &[ParamMeta] = &[ParamMeta {
  name:        "body_pct",
  param_type:  ParamType::Ratio,
  default:     0.1,
  range:       (0.05, 0.2, 0.05),
  description: "Maximum doji body ratio",
}];

static GAPPING_UP_DOJI_PARAMS: &[ParamMeta] = &[ParamMeta {
  name:        "body_pct",
  param_type:  ParamType::Ratio,
  default:     0.1,
  range:       (0.05, 0.2, 0.05),
  description: "Maximum doji body ratio",
}];

static ABOVE_THE_STOMACH_PARAMS: &[ParamMeta] = &[ParamMeta {
  name:        "penetration",
  param_type:  ParamType::Ratio,
  default:     0.0,
  range:       (0.0, 0.3, 0.1),
  description: "Minimum penetration above midpoint",
}];

static BELOW_THE_STOMACH_PARAMS: &[ParamMeta] = &[ParamMeta {
  name:        "penetration",
  param_type:  ParamType::Ratio,
  default:     0.0,
  range:       (0.0, 0.3, 0.1),
  description: "Minimum penetration below midpoint",
}];

static COLLAPSING_DOJI_STAR_PARAMS: &[ParamMeta] = &[
  ParamMeta {
    name:        "body_pct",
    param_type:  ParamType::Ratio,
    default:     0.1,
    range:       (0.05, 0.2, 0.05),
    description: "Maximum doji body ratio",
  },
  ParamMeta {
    name:        "gap_pct",
    param_type:  ParamType::Ratio,
    default:     0.005,
    range:       (0.002, 0.01, 0.002),
    description: "Minimum gap percentage",
  },
];

static DELIBERATION_PARAMS: &[ParamMeta] = &[
  ParamMeta {
    name:        "body_pct",
    param_type:  ParamType::Ratio,
    default:     0.3,
    range:       (0.2, 0.5, 0.1),
    description: "Maximum third candle body ratio",
  },
  ParamMeta {
    name:        "long_body_pct",
    param_type:  ParamType::Ratio,
    default:     0.6,
    range:       (0.5, 0.8, 0.05),
    description: "Minimum body ratio for first/second candles",
  },
];

static LAST_ENGULFING_BOTTOM_PARAMS: &[ParamMeta] = &[ParamMeta {
  name:        "trend_period",
  param_type:  ParamType::Period,
  default:     14.0,
  range:       (10.0, 20.0, 2.0),
  description: "Trend detection period",
}];

static LAST_ENGULFING_TOP_PARAMS: &[ParamMeta] = &[ParamMeta {
  name:        "trend_period",
  param_type:  ParamType::Period,
  default:     14.0,
  range:       (10.0, 20.0, 2.0),
  description: "Trend detection period",
}];

static MEETING_LINES_BEARISH_PARAMS: &[ParamMeta] = &[
  ParamMeta {
    name:        "tolerance",
    param_type:  ParamType::Ratio,
    default:     0.001,
    range:       (0.0005, 0.003, 0.0005),
    description: "Close price tolerance",
  },
  ParamMeta {
    name:        "body_pct",
    param_type:  ParamType::Ratio,
    default:     0.6,
    range:       (0.5, 0.8, 0.05),
    description: "Minimum body ratio for long body requirement",
  },
];

static MEETING_LINES_BULLISH_PARAMS: &[ParamMeta] = &[
  ParamMeta {
    name:        "tolerance",
    param_type:  ParamType::Ratio,
    default:     0.001,
    range:       (0.0005, 0.003, 0.0005),
    description: "Close price tolerance",
  },
  ParamMeta {
    name:        "body_pct",
    param_type:  ParamType::Ratio,
    default:     0.6,
    range:       (0.5, 0.8, 0.05),
    description: "Minimum body ratio for long body requirement",
  },
];

static NORTHERN_DOJI_PARAMS: &[ParamMeta] = &[
  ParamMeta {
    name:        "body_pct",
    param_type:  ParamType::Ratio,
    default:     0.1,
    range:       (0.05, 0.2, 0.05),
    description: "Maximum doji body ratio",
  },
  ParamMeta {
    name:        "trend_period",
    param_type:  ParamType::Period,
    default:     5.0,
    range:       (3.0, 10.0, 1.0),
    description: "Trend lookback period",
  },
];

static SOUTHERN_DOJI_PARAMS: &[ParamMeta] = &[
  ParamMeta {
    name:        "body_pct",
    param_type:  ParamType::Ratio,
    default:     0.1,
    range:       (0.05, 0.2, 0.05),
    description: "Maximum doji body ratio",
  },
  ParamMeta {
    name:        "trend_period",
    param_type:  ParamType::Period,
    default:     5.0,
    range:       (3.0, 10.0, 1.0),
    description: "Trend lookback period",
  },
];

static BLACK_MARUBOZU_PARAMS: &[ParamMeta] = &[ParamMeta {
  name:        "shadow_tolerance",
  param_type:  ParamType::Ratio,
  default:     0.01,
  range:       (0.005, 0.03, 0.005),
  description: "Maximum shadow tolerance",
}];

static WHITE_MARUBOZU_PARAMS: &[ParamMeta] = &[ParamMeta {
  name:        "shadow_tolerance",
  param_type:  ParamType::Ratio,
  default:     0.01,
  range:       (0.005, 0.03, 0.005),
  description: "Maximum shadow tolerance",
}];

static OPENING_BLACK_MARUBOZU_PARAMS: &[ParamMeta] = &[ParamMeta {
  name:        "shadow_tolerance",
  param_type:  ParamType::Ratio,
  default:     0.01,
  range:       (0.005, 0.03, 0.005),
  description: "Maximum shadow tolerance",
}];

static OPENING_WHITE_MARUBOZU_PARAMS: &[ParamMeta] = &[ParamMeta {
  name:        "shadow_tolerance",
  param_type:  ParamType::Ratio,
  default:     0.01,
  range:       (0.005, 0.03, 0.005),
  description: "Maximum shadow tolerance",
}];

static SHORT_BLACK_PARAMS: &[ParamMeta] = &[ParamMeta {
  name:        "body_pct",
  param_type:  ParamType::Ratio,
  default:     0.3,
  range:       (0.2, 0.5, 0.1),
  description: "Maximum body ratio",
}];

static SHORT_WHITE_PARAMS: &[ParamMeta] = &[ParamMeta {
  name:        "body_pct",
  param_type:  ParamType::Ratio,
  default:     0.3,
  range:       (0.2, 0.5, 0.1),
  description: "Maximum body ratio",
}];

static LONG_BLACK_DAY_PARAMS: &[ParamMeta] = &[ParamMeta {
  name:        "body_pct",
  param_type:  ParamType::Ratio,
  default:     0.7,
  range:       (0.6, 0.85, 0.05),
  description: "Minimum body ratio",
}];

static LONG_WHITE_DAY_PARAMS: &[ParamMeta] = &[ParamMeta {
  name:        "body_pct",
  param_type:  ParamType::Ratio,
  default:     0.7,
  range:       (0.6, 0.85, 0.05),
  description: "Minimum body ratio",
}];

static BLACK_SPINNING_TOP_PARAMS: &[ParamMeta] = &[
  ParamMeta {
    name:        "body_pct",
    param_type:  ParamType::Ratio,
    default:     0.3,
    range:       (0.2, 0.4, 0.05),
    description: "Maximum body ratio",
  },
  ParamMeta {
    name:        "shadow_ratio",
    param_type:  ParamType::Ratio,
    default:     0.5,
    range:       (0.3, 0.7, 0.1),
    description: "Minimum shadow ratio",
  },
];

static WHITE_SPINNING_TOP_PARAMS: &[ParamMeta] = &[
  ParamMeta {
    name:        "body_pct",
    param_type:  ParamType::Ratio,
    default:     0.3,
    range:       (0.2, 0.4, 0.05),
    description: "Maximum body ratio",
  },
  ParamMeta {
    name:        "shadow_ratio",
    param_type:  ParamType::Ratio,
    default:     0.5,
    range:       (0.3, 0.7, 0.1),
    description: "Minimum shadow ratio",
  },
];

static SHOOTING_STAR_2_LINES_PARAMS: &[ParamMeta] = &[
  ParamMeta {
    name:        "body_pct",
    param_type:  ParamType::Ratio,
    default:     0.3,
    range:       (0.2, 0.4, 0.05),
    description: "Maximum body ratio",
  },
  ParamMeta {
    name:        "shadow_ratio",
    param_type:  ParamType::Ratio,
    default:     2.0,
    range:       (1.5, 3.0, 0.5),
    description: "Minimum upper shadow to body ratio",
  },
];

static DOWNSIDE_TASUKI_GAP_PARAMS: &[ParamMeta] = &[ParamMeta {
  name:        "gap_fill_pct",
  param_type:  ParamType::Ratio,
  default:     0.7,
  range:       (0.5, 0.9, 0.1),
  description: "Maximum gap fill percentage",
}];

static UPSIDE_TASUKI_GAP_PARAMS: &[ParamMeta] = &[ParamMeta {
  name:        "gap_fill_pct",
  param_type:  ParamType::Ratio,
  default:     0.7,
  range:       (0.5, 0.9, 0.1),
  description: "Maximum gap fill percentage",
}];

impl ParameterizedDetector for GappingDownDojiDetector {
  fn param_meta() -> &'static [ParamMeta] {
    GAPPING_DOWN_DOJI_PARAMS
  }

  fn with_params(params: &HashMap<&str, f64>) -> Result<Self> {
    Ok(Self { body_pct: get_ratio(params, "body_pct", 0.1)? })
  }

  fn pattern_id_str() -> &'static str {
    "GAPPING_DOWN_DOJI"
  }
}

impl ParameterizedDetector for GappingUpDojiDetector {
  fn param_meta() -> &'static [ParamMeta] {
    GAPPING_UP_DOJI_PARAMS
  }

  fn with_params(params: &HashMap<&str, f64>) -> Result<Self> {
    Ok(Self { body_pct: get_ratio(params, "body_pct", 0.1)? })
  }

  fn pattern_id_str() -> &'static str {
    "GAPPING_UP_DOJI"
  }
}

impl ParameterizedDetector for AboveTheStomachDetector {
  fn param_meta() -> &'static [ParamMeta] {
    ABOVE_THE_STOMACH_PARAMS
  }

  fn with_params(params: &HashMap<&str, f64>) -> Result<Self> {
    Ok(Self { penetration: get_ratio(params, "penetration", 0.0)? })
  }

  fn pattern_id_str() -> &'static str {
    "ABOVE_THE_STOMACH"
  }
}

impl ParameterizedDetector for BelowTheStomachDetector {
  fn param_meta() -> &'static [ParamMeta] {
    BELOW_THE_STOMACH_PARAMS
  }

  fn with_params(params: &HashMap<&str, f64>) -> Result<Self> {
    Ok(Self { penetration: get_ratio(params, "penetration", 0.0)? })
  }

  fn pattern_id_str() -> &'static str {
    "BELOW_THE_STOMACH"
  }
}

impl ParameterizedDetector for CollapsingDojiStarDetector {
  fn param_meta() -> &'static [ParamMeta] {
    COLLAPSING_DOJI_STAR_PARAMS
  }

  fn with_params(params: &HashMap<&str, f64>) -> Result<Self> {
    Ok(Self {
      body_pct: get_ratio(params, "body_pct", 0.1)?,
      gap_pct:  get_ratio(params, "gap_pct", 0.005)?,
    })
  }

  fn pattern_id_str() -> &'static str {
    "COLLAPSING_DOJI_STAR"
  }
}

impl ParameterizedDetector for DeliberationDetector {
  fn param_meta() -> &'static [ParamMeta] {
    DELIBERATION_PARAMS
  }

  fn with_params(params: &HashMap<&str, f64>) -> Result<Self> {
    Ok(Self {
      body_pct:      get_ratio(params, "body_pct", 0.3)?,
      long_body_pct: get_ratio(params, "long_body_pct", 0.6)?,
    })
  }

  fn pattern_id_str() -> &'static str {
    "DELIBERATION"
  }
}

impl ParameterizedDetector for LastEngulfingBottomDetector {
  fn param_meta() -> &'static [ParamMeta] {
    LAST_ENGULFING_BOTTOM_PARAMS
  }

  fn with_params(params: &HashMap<&str, f64>) -> Result<Self> {
    Ok(Self { trend_period: get_period(params, "trend_period", 14)? })
  }

  fn pattern_id_str() -> &'static str {
    "LAST_ENGULFING_BOTTOM"
  }
}

impl ParameterizedDetector for LastEngulfingTopDetector {
  fn param_meta() -> &'static [ParamMeta] {
    LAST_ENGULFING_TOP_PARAMS
  }

  fn with_params(params: &HashMap<&str, f64>) -> Result<Self> {
    Ok(Self { trend_period: get_period(params, "trend_period", 14)? })
  }

  fn pattern_id_str() -> &'static str {
    "LAST_ENGULFING_TOP"
  }
}

impl ParameterizedDetector for MeetingLinesBearishDetector {
  fn param_meta() -> &'static [ParamMeta] {
    MEETING_LINES_BEARISH_PARAMS
  }

  fn with_params(params: &HashMap<&str, f64>) -> Result<Self> {
    Ok(Self {
      tolerance: get_ratio(params, "tolerance", 0.001)?,
      body_pct:  get_ratio(params, "body_pct", 0.6)?,
    })
  }

  fn pattern_id_str() -> &'static str {
    "MEETING_LINES_BEARISH"
  }
}

impl ParameterizedDetector for MeetingLinesBullishDetector {
  fn param_meta() -> &'static [ParamMeta] {
    MEETING_LINES_BULLISH_PARAMS
  }

  fn with_params(params: &HashMap<&str, f64>) -> Result<Self> {
    Ok(Self {
      tolerance: get_ratio(params, "tolerance", 0.001)?,
      body_pct:  get_ratio(params, "body_pct", 0.6)?,
    })
  }

  fn pattern_id_str() -> &'static str {
    "MEETING_LINES_BULLISH"
  }
}

impl ParameterizedDetector for NorthernDojiDetector {
  fn param_meta() -> &'static [ParamMeta] {
    NORTHERN_DOJI_PARAMS
  }

  fn with_params(params: &HashMap<&str, f64>) -> Result<Self> {
    Ok(Self {
      body_pct:     get_ratio(params, "body_pct", 0.1)?,
      trend_period: get_period(params, "trend_period", 5)?,
    })
  }

  fn pattern_id_str() -> &'static str {
    "NORTHERN_DOJI"
  }
}

impl ParameterizedDetector for SouthernDojiDetector {
  fn param_meta() -> &'static [ParamMeta] {
    SOUTHERN_DOJI_PARAMS
  }

  fn with_params(params: &HashMap<&str, f64>) -> Result<Self> {
    Ok(Self {
      body_pct:     get_ratio(params, "body_pct", 0.1)?,
      trend_period: get_period(params, "trend_period", 5)?,
    })
  }

  fn pattern_id_str() -> &'static str {
    "SOUTHERN_DOJI"
  }
}

impl ParameterizedDetector for BlackMarubozuDetector {
  fn param_meta() -> &'static [ParamMeta] {
    BLACK_MARUBOZU_PARAMS
  }

  fn with_params(params: &HashMap<&str, f64>) -> Result<Self> {
    Ok(Self { shadow_tolerance: get_ratio(params, "shadow_tolerance", 0.01)? })
  }

  fn pattern_id_str() -> &'static str {
    "BLACK_MARUBOZU"
  }
}

impl ParameterizedDetector for WhiteMarubozuDetector {
  fn param_meta() -> &'static [ParamMeta] {
    WHITE_MARUBOZU_PARAMS
  }

  fn with_params(params: &HashMap<&str, f64>) -> Result<Self> {
    Ok(Self { shadow_tolerance: get_ratio(params, "shadow_tolerance", 0.01)? })
  }

  fn pattern_id_str() -> &'static str {
    "WHITE_MARUBOZU"
  }
}

impl ParameterizedDetector for OpeningBlackMarubozuDetector {
  fn param_meta() -> &'static [ParamMeta] {
    OPENING_BLACK_MARUBOZU_PARAMS
  }

  fn with_params(params: &HashMap<&str, f64>) -> Result<Self> {
    Ok(Self { shadow_tolerance: get_ratio(params, "shadow_tolerance", 0.01)? })
  }

  fn pattern_id_str() -> &'static str {
    "OPENING_BLACK_MARUBOZU"
  }
}

impl ParameterizedDetector for OpeningWhiteMarubozuDetector {
  fn param_meta() -> &'static [ParamMeta] {
    OPENING_WHITE_MARUBOZU_PARAMS
  }

  fn with_params(params: &HashMap<&str, f64>) -> Result<Self> {
    Ok(Self { shadow_tolerance: get_ratio(params, "shadow_tolerance", 0.01)? })
  }

  fn pattern_id_str() -> &'static str {
    "OPENING_WHITE_MARUBOZU"
  }
}

impl ParameterizedDetector for ShortBlackDetector {
  fn param_meta() -> &'static [ParamMeta] {
    SHORT_BLACK_PARAMS
  }

  fn with_params(params: &HashMap<&str, f64>) -> Result<Self> {
    Ok(Self { body_pct: get_ratio(params, "body_pct", 0.3)? })
  }

  fn pattern_id_str() -> &'static str {
    "SHORT_BLACK"
  }
}

impl ParameterizedDetector for ShortWhiteDetector {
  fn param_meta() -> &'static [ParamMeta] {
    SHORT_WHITE_PARAMS
  }

  fn with_params(params: &HashMap<&str, f64>) -> Result<Self> {
    Ok(Self { body_pct: get_ratio(params, "body_pct", 0.3)? })
  }

  fn pattern_id_str() -> &'static str {
    "SHORT_WHITE"
  }
}

impl ParameterizedDetector for LongBlackDayDetector {
  fn param_meta() -> &'static [ParamMeta] {
    LONG_BLACK_DAY_PARAMS
  }

  fn with_params(params: &HashMap<&str, f64>) -> Result<Self> {
    Ok(Self { body_pct: get_ratio(params, "body_pct", 0.7)? })
  }

  fn pattern_id_str() -> &'static str {
    "LONG_BLACK_DAY"
  }
}

impl ParameterizedDetector for LongWhiteDayDetector {
  fn param_meta() -> &'static [ParamMeta] {
    LONG_WHITE_DAY_PARAMS
  }

  fn with_params(params: &HashMap<&str, f64>) -> Result<Self> {
    Ok(Self { body_pct: get_ratio(params, "body_pct", 0.7)? })
  }

  fn pattern_id_str() -> &'static str {
    "LONG_WHITE_DAY"
  }
}

impl ParameterizedDetector for BlackSpinningTopDetector {
  fn param_meta() -> &'static [ParamMeta] {
    BLACK_SPINNING_TOP_PARAMS
  }

  fn with_params(params: &HashMap<&str, f64>) -> Result<Self> {
    Ok(Self {
      body_pct:     get_ratio(params, "body_pct", 0.3)?,
      shadow_ratio: get_ratio(params, "shadow_ratio", 0.5)?,
    })
  }

  fn pattern_id_str() -> &'static str {
    "BLACK_SPINNING_TOP"
  }
}

impl ParameterizedDetector for WhiteSpinningTopDetector {
  fn param_meta() -> &'static [ParamMeta] {
    WHITE_SPINNING_TOP_PARAMS
  }

  fn with_params(params: &HashMap<&str, f64>) -> Result<Self> {
    Ok(Self {
      body_pct:     get_ratio(params, "body_pct", 0.3)?,
      shadow_ratio: get_ratio(params, "shadow_ratio", 0.5)?,
    })
  }

  fn pattern_id_str() -> &'static str {
    "WHITE_SPINNING_TOP"
  }
}

impl ParameterizedDetector for ShootingStar2LinesDetector {
  fn param_meta() -> &'static [ParamMeta] {
    SHOOTING_STAR_2_LINES_PARAMS
  }

  fn with_params(params: &HashMap<&str, f64>) -> Result<Self> {
    Ok(Self {
      body_pct:     get_ratio(params, "body_pct", 0.3)?,
      shadow_ratio: get_ratio(params, "shadow_ratio", 2.0)?,
    })
  }

  fn pattern_id_str() -> &'static str {
    "SHOOTING_STAR_2_LINES"
  }
}

impl ParameterizedDetector for DownsideTasukiGapDetector {
  fn param_meta() -> &'static [ParamMeta] {
    DOWNSIDE_TASUKI_GAP_PARAMS
  }

  fn with_params(params: &HashMap<&str, f64>) -> Result<Self> {
    Ok(Self { gap_fill_pct: get_ratio(params, "gap_fill_pct", 0.7)? })
  }

  fn pattern_id_str() -> &'static str {
    "DOWNSIDE_TASUKI_GAP"
  }
}

impl ParameterizedDetector for UpsideTasukiGapDetector {
  fn param_meta() -> &'static [ParamMeta] {
    UPSIDE_TASUKI_GAP_PARAMS
  }

  fn with_params(params: &HashMap<&str, f64>) -> Result<Self> {
    Ok(Self { gap_fill_pct: get_ratio(params, "gap_fill_pct", 0.7)? })
  }

  fn pattern_id_str() -> &'static str {
    "UPSIDE_TASUKI_GAP"
  }
}
