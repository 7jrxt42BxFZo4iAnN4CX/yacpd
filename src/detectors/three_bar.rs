//! Three-bar candlestick pattern detectors (TA-Lib compatible)
//!
//! TA-Lib patterns: CDL3WHITESOLDIERS, CDL3BLACKCROWS, CDL3INSIDE, CDL3OUTSIDE,
//! CDL3LINESTRIKE, CDL3STARSINSOUTH, CDLMORNINGSTAR, CDLEVENINGSTAR, CDLMORNINGDOJISTAR,
//! CDLEVENINGDOJISTAR, CDLABANDONEDBABY, CDL2CROWS, CDLUPSIDEGAP2CROWS, CDLIDENTICAL3CROWS,
//! CDLADVANCEBLOCK, CDLSTALLEDPATTERN, CDLSTICKSANDWICH, CDLTASUKIGAP, CDLTRISTAR, CDLUNIQUE3RIVER

#![allow(clippy::collapsible_if, clippy::default_constructed_unit_structs)]

use std::collections::HashMap;

use crate::{
  params::{get_ratio, ParamMeta, ParamType, ParameterizedDetector},
  Direction, MarketContext, OHLCVExt, PatternDetector, PatternId, PatternMatch, Ratio, Result,
  OHLCV,
};

use super::helpers::{self, is_body_long_f, is_body_short_f, is_doji_f, is_shadow_very_short, is_shadow_very_short_f};

// ============================================================
// THREE WHITE SOLDIERS / THREE BLACK CROWS
// ============================================================

/// CDL3WHITESOLDIERS - Three White Soldiers (TA-Lib compatible)
#[derive(Debug, Clone)]
pub struct ThreeWhiteSoldiersDetector {
  pub shadow_veryshort_factor: f64,
  pub near_factor: f64,
  pub far_factor: f64,
  pub body_short_factor: f64,
}

impl Default for ThreeWhiteSoldiersDetector {
  fn default() -> Self {
    Self {
      shadow_veryshort_factor: helpers::SHADOW_VERYSHORT_FACTOR,
      near_factor: helpers::NEAR_FACTOR,
      far_factor: helpers::FAR_FACTOR,
      body_short_factor: helpers::BODY_SHORT_FACTOR,
    }
  }
}

impl ThreeWhiteSoldiersDetector {
  pub fn with_defaults() -> Self {
    Self::default()
  }
}

impl PatternDetector for ThreeWhiteSoldiersDetector {
  fn id(&self) -> PatternId {
    PatternId("CDL_3WHITESOLDIERS")
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

    // TA-Lib: All three must be bullish
    if !first.is_bullish() || !second.is_bullish() || !third.is_bullish() {
      return None;
    }

    // TA-Lib: Ascending closes
    if second.close() <= first.close() || third.close() <= second.close() {
      return None;
    }

    let first_body = first.body();
    let second_body = second.body();
    let third_body = third.body();

    // TA-Lib: All three have very short upper shadows (ShadowVeryShort per-candle)
    let first_upper = first.high() - first.close();
    let second_upper = second.high() - second.close();
    let third_upper = third.high() - third.close();

    let svs_first = helpers::trailing_avg_range(bars, index - 2, 10) * self.shadow_veryshort_factor;
    let svs_second = helpers::trailing_avg_range(bars, index - 1, 10) * self.shadow_veryshort_factor;
    let svs_third = helpers::trailing_avg_range(bars, index, 10) * self.shadow_veryshort_factor;

    if first_upper >= svs_first || second_upper >= svs_second || third_upper >= svs_third {
      return None;
    }

    // TA-Lib: Opens within/near previous body
    // open[i-1] > open[i-2] AND open[i-1] <= close[i-2] + Near_avg_at(i-2)
    let near_first = helpers::trailing_avg_range(bars, index - 2, 5) * self.near_factor;
    let near_second = helpers::trailing_avg_range(bars, index - 1, 5) * self.near_factor;

    if second.open() <= first.open() || second.open() > first.close() + near_first {
      return None;
    }
    if third.open() <= second.open() || third.open() > second.close() + near_second {
      return None;
    }

    // TA-Lib: Bodies not getting significantly smaller (Far check)
    // body[i-1] > body[i-2] - Far_avg_at(i-2)
    let far_first = helpers::trailing_avg_range(bars, index - 2, 5) * self.far_factor;
    let far_second = helpers::trailing_avg_range(bars, index - 1, 5) * self.far_factor;

    if second_body <= first_body - far_first {
      return None;
    }
    if third_body <= second_body - far_second {
      return None;
    }

    // TA-Lib: Third candle body > BodyShort avg
    let body_short_avg = helpers::trailing_avg_body(bars, index, 10) * self.body_short_factor;
    if third_body < body_short_avg {
      return None;
    }

    Some(PatternMatch {
      pattern_id:  PatternDetector::id(self),
      direction:   Direction::Bullish,
      strength:    0.8,
      start_index: index - 2,
      end_index:   index,
    })
  }
}

/// CDL3BLACKCROWS - Three Black Crows (TA-Lib compatible)
#[derive(Debug, Clone)]
pub struct ThreeBlackCrowsDetector {
  pub shadow_veryshort_factor: f64,
}

impl Default for ThreeBlackCrowsDetector {
  fn default() -> Self {
    Self {
      shadow_veryshort_factor: helpers::SHADOW_VERYSHORT_FACTOR,
    }
  }
}

impl ThreeBlackCrowsDetector {
  pub fn with_defaults() -> Self {
    Self::default()
  }
}

impl PatternDetector for ThreeBlackCrowsDetector {
  fn id(&self) -> PatternId {
    PatternId("CDL_3BLACKCROWS")
  }

  fn min_bars(&self) -> usize {
    4
  }

  fn detect<T: OHLCV>(
    &self,
    bars: &[T],
    index: usize,
    _ctx: &MarketContext,
  ) -> Option<PatternMatch> {
    if index < 3 {
      return None;
    }
    let prior = bars.get(index - 3)?;
    let first = bars.get(index - 2)?;
    let second = bars.get(index - 1)?;
    let third = bars.get(index)?;

    // TA-Lib: Prior candle must be bullish (white)
    if !prior.is_bullish() {
      return None;
    }

    // TA-Lib: All three crows must be bearish
    if !first.is_bearish() || !second.is_bearish() || !third.is_bearish() {
      return None;
    }

    // TA-Lib: Declining closes
    if second.close() >= first.close() || third.close() >= second.close() {
      return None;
    }

    // TA-Lib: Each opens within previous body (strict inequalities)
    // inOpen[i-1] < inOpen[i-2] && inOpen[i-1] > inClose[i-2]
    if second.open() >= first.open() || second.open() <= first.close() {
      return None;
    }
    // inOpen[i] < inOpen[i-1] && inOpen[i] > inClose[i-1]
    if third.open() >= second.open() || third.open() <= second.close() {
      return None;
    }

    // TA-Lib: Prior candle's high > first crow's close
    // inHigh[i-3] > inClose[i-2]
    if prior.high() <= first.close() {
      return None;
    }

    // TA-Lib: All three have very short lower shadows (ShadowVeryShort per-candle)
    let first_lower = first.lower_shadow();
    let second_lower = second.lower_shadow();
    let third_lower = third.lower_shadow();

    let svs_first = helpers::trailing_avg_range(bars, index - 2, 10) * self.shadow_veryshort_factor;
    let svs_second = helpers::trailing_avg_range(bars, index - 1, 10) * self.shadow_veryshort_factor;
    let svs_third = helpers::trailing_avg_range(bars, index, 10) * self.shadow_veryshort_factor;

    if first_lower >= svs_first || second_lower >= svs_second || third_lower >= svs_third {
      return None;
    }

    Some(PatternMatch {
      pattern_id:  PatternDetector::id(self),
      direction:   Direction::Bearish,
      strength:    0.8,
      start_index: index - 2,
      end_index:   index,
    })
  }
}

// ============================================================
// THREE INSIDE / THREE OUTSIDE
// ============================================================

/// CDL3INSIDE - Three Inside Up/Down (TA-Lib compatible)
#[derive(Debug, Clone)]
pub struct ThreeInsideDetector {
  pub body_long_factor: f64,
  pub body_short_factor: f64,
}

impl Default for ThreeInsideDetector {
  fn default() -> Self {
    Self {
      body_long_factor: helpers::BODY_LONG_FACTOR,
      body_short_factor: helpers::BODY_SHORT_FACTOR,
    }
  }
}

impl ThreeInsideDetector {
  pub fn with_defaults() -> Self {
    Self::default()
  }
}

impl PatternDetector for ThreeInsideDetector {
  fn id(&self) -> PatternId {
    PatternId("CDL_3INSIDE")
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

    // TA-Lib: First bar must have a long body (BodyLong, per-candle trailing at i-2)
    let first_body = first.body();
    let first_range = first.range();
    let avg_body_first = helpers::trailing_avg_body(bars, index - 2, 10);
    if !is_body_long_f(first_body, avg_body_first, first_range, self.body_long_factor) {
      return None;
    }

    // TA-Lib: Second bar must have a short body (BodyShort, per-candle trailing at i-1)
    let second_body = second.body();
    let second_range = second.range();
    let avg_body_second = helpers::trailing_avg_body(bars, index - 1, 10);
    if !is_body_short_f(second_body, avg_body_second, second_range, self.body_short_factor) {
      return None;
    }

    // TA-Lib: Second bar body strictly inside first bar body (strict < and >)
    let first_high = first.open().max(first.close());
    let first_low = first.open().min(first.close());
    let second_high = second.open().max(second.close());
    let second_low = second.open().min(second.close());

    if second_high >= first_high || second_low <= first_low {
      return None;
    }

    // TA-Lib: TA_CANDLECOLOR convention (close >= open = white/+1)
    let first_white = first.close() >= first.open();
    let third_white = third.close() >= third.open();

    // Three Inside Up: first black, third white, closes above first's open
    if !first_white && third_white && third.close() > first.open() {
      return Some(PatternMatch {
        pattern_id:  PatternDetector::id(self),
        direction:   Direction::Bullish,
        strength:    0.7,
        start_index: index - 2,
        end_index:   index,
      });
    }

    // Three Inside Down: first white, third black, closes below first's open
    if first_white && !third_white && third.close() < first.open() {
      return Some(PatternMatch {
        pattern_id:  PatternDetector::id(self),
        direction:   Direction::Bearish,
        strength:    0.7,
        start_index: index - 2,
        end_index:   index,
      });
    }

    None
  }
}

/// CDL3OUTSIDE - Three Outside Up/Down
#[derive(Debug, Clone)]
pub struct ThreeOutsideDetector;

impl Default for ThreeOutsideDetector {
  fn default() -> Self {
    Self
  }
}

impl ThreeOutsideDetector {
  pub fn with_defaults() -> Self {
    Self::default()
  }
}

impl PatternDetector for ThreeOutsideDetector {
  fn id(&self) -> PatternId {
    PatternId("CDL_3OUTSIDE")
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

    // TA-Lib: TA_CANDLECOLOR: close >= open → white(+1), close < open → black(-1)
    // Three Outside Up: first black, second white, second strictly engulfs first, third closes above second
    if second.close() >= second.open() && first.close() < first.open() {
      // TA-Lib: close[i-1] > open[i-2] && open[i-1] < close[i-2] (strict engulf)
      if second.close() > first.open() && second.open() < first.close() {
        // TA-Lib: close[i] > close[i-1] (confirmation)
        if third.close() > second.close() {
          return Some(PatternMatch {
            pattern_id:  PatternDetector::id(self),
            direction:   Direction::Bullish,
            strength:    0.75,
            start_index: index - 2,
            end_index:   index,
          });
        }
      }
    }

    // Three Outside Down: first white, second black, second strictly engulfs first, third closes below second
    if second.close() < second.open() && first.close() >= first.open() {
      // TA-Lib: open[i-1] > close[i-2] && close[i-1] < open[i-2] (strict engulf)
      if second.open() > first.close() && second.close() < first.open() {
        // TA-Lib: close[i] < close[i-1] (confirmation)
        if third.close() < second.close() {
          return Some(PatternMatch {
            pattern_id:  PatternDetector::id(self),
            direction:   Direction::Bearish,
            strength:    0.75,
            start_index: index - 2,
            end_index:   index,
          });
        }
      }
    }

    None
  }
}

// ============================================================
// THREE LINE STRIKE
// ============================================================

/// CDL3LINESTRIKE - Three-Line Strike
#[derive(Debug, Clone)]
pub struct ThreeLineStrikeDetector {
  pub near_factor: f64,
}

impl Default for ThreeLineStrikeDetector {
  fn default() -> Self {
    Self {
      near_factor: helpers::NEAR_FACTOR,
    }
  }
}

impl ThreeLineStrikeDetector {
  pub fn with_defaults() -> Self {
    Self::default()
  }
}

impl PatternDetector for ThreeLineStrikeDetector {
  fn id(&self) -> PatternId {
    PatternId("CDL_3LINESTRIKE")
  }

  fn min_bars(&self) -> usize {
    4
  }

  fn detect<T: OHLCV>(
    &self,
    bars: &[T],
    index: usize,
    ctx: &MarketContext,
  ) -> Option<PatternMatch> {
    if index < 3 {
      return None;
    }
    let first = bars.get(index - 3)?;
    let second = bars.get(index - 2)?;
    let third = bars.get(index - 1)?;
    let fourth = bars.get(index)?;

    // TA-Lib: TA_CANDLECOLOR convention (close >= open = white/+1)
    let color_first = if first.close() >= first.open() { 1_i32 } else { -1 };
    let color_second = if second.close() >= second.open() { 1_i32 } else { -1 };
    let color_third = if third.close() >= third.open() { 1_i32 } else { -1 };
    let color_fourth = if fourth.close() >= fourth.open() { 1_i32 } else { -1 };

    // TA-Lib: first three same color, fourth opposite
    if color_first != color_second || color_second != color_third { return None; }
    if color_third == color_fourth { return None; }

    // TA-Lib: Near threshold per-candle for open checks
    let near_threshold_first = helpers::trailing_avg_range(bars, index - 3, 5) * self.near_factor;
    let near_threshold_second = helpers::trailing_avg_range(bars, index - 2, 5) * self.near_factor;

    // TA-Lib: bar 2 opens within/near bar 1's body
    if second.open() < first.open().min(first.close()) - near_threshold_first
      || second.open() > first.open().max(first.close()) + near_threshold_first
    {
      return None;
    }
    // TA-Lib: bar 3 opens within/near bar 2's body
    if third.open() < second.open().min(second.close()) - near_threshold_second
      || third.open() > second.open().max(second.close()) + near_threshold_second
    {
      return None;
    }

    if color_first == 1 {
      // Bullish: three ascending closes, 4th opens above 3rd close, closes below 1st open
      if second.close() > first.close() && third.close() > second.close()
        && fourth.open() > third.close() && fourth.close() < first.open()
      {
        return Some(PatternMatch {
          pattern_id:  PatternDetector::id(self),
          direction:   Direction::Bullish,
          strength:    0.7,
          start_index: index - 3,
          end_index:   index,
        });
      }
    } else {
      // Bearish: three descending closes, 4th opens below 3rd close, closes above 1st open
      if second.close() < first.close() && third.close() < second.close()
        && fourth.open() < third.close() && fourth.close() > first.open()
      {
        return Some(PatternMatch {
          pattern_id:  PatternDetector::id(self),
          direction:   Direction::Bearish,
          strength:    0.7,
          start_index: index - 3,
          end_index:   index,
        });
      }
    }

    None
  }
}

// ============================================================
// THREE STARS IN THE SOUTH
// ============================================================

/// CDL3STARSINSOUTH - Three Stars In The South
#[derive(Debug, Clone)]
pub struct ThreeStarsInSouthDetector {
  pub body_long_factor: f64,
  pub body_short_factor: f64,
  pub shadow_veryshort_factor: f64,
}

impl Default for ThreeStarsInSouthDetector {
  fn default() -> Self {
    Self {
      body_long_factor: helpers::BODY_LONG_FACTOR,
      body_short_factor: helpers::BODY_SHORT_FACTOR,
      shadow_veryshort_factor: helpers::SHADOW_VERYSHORT_FACTOR,
    }
  }
}

impl ThreeStarsInSouthDetector {
  pub fn with_defaults() -> Self {
    Self::default()
  }
}

impl PatternDetector for ThreeStarsInSouthDetector {
  fn id(&self) -> PatternId {
    PatternId("CDL_3STARSINSOUTH")
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

    // TA-Lib: All three bearish (black)
    if !first.is_bearish() || !second.is_bearish() || !third.is_bearish() {
      return None;
    }

    // TA-Lib: First has BodyLong (per-candle trailing at i-2)
    let first_body = first.body();
    let first_range = first.range();
    let avg_body_first = helpers::trailing_avg_body(bars, index - 2, 10);
    if !is_body_long_f(first_body, avg_body_first, first_range, self.body_long_factor) {
      return None;
    }

    // TA-Lib: First has ShadowLong (lower shadow > body, Period=0)
    let first_lower = first.lower_shadow();
    if !helpers::is_shadow_long(first_lower, first_body, first_range) {
      return None;
    }

    // TA-Lib: Second has smaller body than first
    let second_body = second.body();
    if second_body >= first_body {
      return None;
    }

    // TA-Lib: Second opens above first's close, within first's high-low range
    if second.open() <= first.close() {
      return None;
    }
    if second.open() > first.high() {
      return None;
    }

    // TA-Lib: Second low < first close, but second low >= first low
    if second.low() >= first.close() {
      return None;
    }
    if second.low() < first.low() {
      return None;
    }

    // TA-Lib: Second has lower shadow > ShadowVeryShort (per-candle at i-1)
    let second_lower = second.lower_shadow();
    let avg_range_second = helpers::trailing_avg_range(bars, index - 1, 10);
    let svs_threshold = avg_range_second * self.shadow_veryshort_factor;
    if second_lower <= svs_threshold {
      return None;
    }

    // TA-Lib: Third has BodyShort (per-candle trailing at i)
    let third_body = third.body();
    let third_range = third.range();
    let avg_body_third = helpers::trailing_avg_body(bars, index, 10);
    if !is_body_short_f(third_body, avg_body_third, third_range, self.body_short_factor) {
      return None;
    }

    // TA-Lib: Third has both shadows ShadowVeryShort (per-candle at i)
    let avg_range_third = helpers::trailing_avg_range(bars, index, 10);
    if !is_shadow_very_short_f(third.lower_shadow(), avg_range_third, third_range, self.shadow_veryshort_factor) {
      return None;
    }
    if !is_shadow_very_short_f(third.upper_shadow(), avg_range_third, third_range, self.shadow_veryshort_factor) {
      return None;
    }

    // TA-Lib: Third within second's range
    if third.low() < second.low() {
      return None;
    }
    if third.high() > second.high() {
      return None;
    }

    Some(PatternMatch {
      pattern_id:  PatternDetector::id(self),
      direction:   Direction::Bullish,
      strength:    0.65,
      start_index: index - 2,
      end_index:   index,
    })
  }
}

// ============================================================
// MORNING STAR / EVENING STAR
// ============================================================

/// CDLMORNINGSTAR - Morning Star (TA-Lib compatible)
#[derive(Debug, Clone)]
pub struct MorningStarDetector {
  pub body_long_factor: f64,
  pub body_short_factor: f64,
  pub penetration: f64,
}

impl Default for MorningStarDetector {
  fn default() -> Self {
    Self {
      body_long_factor: helpers::BODY_LONG_FACTOR,
      body_short_factor: helpers::BODY_SHORT_FACTOR,
      penetration: 0.3,
    }
  }
}

impl MorningStarDetector {
  pub fn with_defaults() -> Self {
    Self::default()
  }
}

impl PatternDetector for MorningStarDetector {
  fn id(&self) -> PatternId {
    PatternId("CDL_MORNINGSTAR")
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

    // TA-Lib: TA_CANDLECOLOR(i-2) == -1 (black/bearish: close < open)
    if first.close() >= first.open() {
      return None;
    }
    // TA-Lib: TA_CANDLECOLOR(i) == 1 (white/bullish: close >= open)
    if third.close() < third.open() {
      return None;
    }

    // TA-Lib: first BodyLong (per-candle trailing at i-2)
    let first_body = first.body();
    let avg_body_first = helpers::trailing_avg_body(bars, index - 2, 10);
    if !is_body_long_f(first_body, avg_body_first, first.range(), self.body_long_factor) {
      return None;
    }

    // TA-Lib: second body <= BodyShort average (per-candle trailing at i-1)
    let second_body = second.body();
    let avg_body_second = helpers::trailing_avg_body(bars, index - 1, 10);
    if !is_body_short_f(second_body, avg_body_second, second.range(), self.body_short_factor) {
      return None;
    }

    // TA-Lib: RealBodyGapDown between second and first
    // max(open[i-1], close[i-1]) < min(open[i-2], close[i-2])
    let second_body_top = second.open().max(second.close());
    let first_body_bottom = first.open().min(first.close());
    if second_body_top >= first_body_bottom {
      return None;
    }

    // TA-Lib: third body > BodyShort average (per-candle trailing at i)
    let third_body = third.body();
    let avg_body_third = helpers::trailing_avg_body(bars, index, 10);
    if third_body <= avg_body_third {
      return None;
    }

    // TA-Lib: close[i] > close[i-2] + body[i-2] * optInPenetration (default 0.3)
    let penetration = self.penetration;
    if third.close() <= first.close() + first_body * penetration {
      return None;
    }

    Some(PatternMatch {
      pattern_id:  PatternDetector::id(self),
      direction:   Direction::Bullish,
      strength:    0.75,
      start_index: index - 2,
      end_index:   index,
    })
  }
}

/// CDLEVENINGSTAR - Evening Star (TA-Lib compatible)
#[derive(Debug, Clone)]
pub struct EveningStarDetector {
  pub body_long_factor: f64,
  pub body_short_factor: f64,
  pub penetration: f64,
}

impl Default for EveningStarDetector {
  fn default() -> Self {
    Self {
      body_long_factor: helpers::BODY_LONG_FACTOR,
      body_short_factor: helpers::BODY_SHORT_FACTOR,
      penetration: 0.3,
    }
  }
}

impl EveningStarDetector {
  pub fn with_defaults() -> Self {
    Self::default()
  }
}

impl PatternDetector for EveningStarDetector {
  fn id(&self) -> PatternId {
    PatternId("CDL_EVENINGSTAR")
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

    // TA-Lib: TA_CANDLECOLOR(i-2) == 1 (white/bullish: close >= open)
    if first.close() < first.open() {
      return None;
    }
    // TA-Lib: TA_CANDLECOLOR(i) == -1 (black/bearish: close < open)
    if third.close() >= third.open() {
      return None;
    }

    // TA-Lib: first BodyLong (per-candle trailing at i-2)
    let first_body = first.body();
    let avg_body_first = helpers::trailing_avg_body(bars, index - 2, 10);
    if !is_body_long_f(first_body, avg_body_first, first.range(), self.body_long_factor) {
      return None;
    }

    // TA-Lib: second body <= BodyShort average (per-candle trailing at i-1)
    let second_body = second.body();
    let avg_body_second = helpers::trailing_avg_body(bars, index - 1, 10);
    if !is_body_short_f(second_body, avg_body_second, second.range(), self.body_short_factor) {
      return None;
    }

    // TA-Lib: RealBodyGapUp between second and first
    // min(open[i-1], close[i-1]) > max(open[i-2], close[i-2])
    let second_body_bottom = second.open().min(second.close());
    let first_body_top = first.open().max(first.close());
    if second_body_bottom <= first_body_top {
      return None;
    }

    // TA-Lib: third body > BodyShort average (per-candle trailing at i)
    let third_body = third.body();
    let avg_body_third = helpers::trailing_avg_body(bars, index, 10);
    if third_body <= avg_body_third {
      return None;
    }

    // TA-Lib: close[i] < close[i-2] - body[i-2] * optInPenetration (default 0.3)
    let penetration = self.penetration;
    if third.close() >= first.close() - first_body * penetration {
      return None;
    }

    Some(PatternMatch {
      pattern_id:  PatternDetector::id(self),
      direction:   Direction::Bearish,
      strength:    0.75,
      start_index: index - 2,
      end_index:   index,
    })
  }
}

/// CDLMORNINGDOJISTAR - Morning Doji Star (TA-Lib compatible)
#[derive(Debug, Clone)]
pub struct MorningDojiStarDetector {
  pub body_long_factor: f64,
  pub doji_factor: f64,
  pub penetration: f64,
}

impl Default for MorningDojiStarDetector {
  fn default() -> Self {
    Self {
      body_long_factor: helpers::BODY_LONG_FACTOR,
      doji_factor: helpers::DOJI_FACTOR,
      penetration: 0.3,
    }
  }
}

impl MorningDojiStarDetector {
  pub fn with_defaults() -> Self {
    Self::default()
  }
}

impl PatternDetector for MorningDojiStarDetector {
  fn id(&self) -> PatternId {
    PatternId("CDL_MORNINGDOJISTAR")
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

    // TA-Lib: TA_CANDLECOLOR(i-2) == -1 (black: close < open)
    if first.close() >= first.open() {
      return None;
    }
    // TA-Lib: TA_CANDLECOLOR(i) == 1 (white: close >= open)
    if third.close() < third.open() {
      return None;
    }

    // TA-Lib: first BodyLong (per-candle trailing at i-2)
    let first_body = first.body();
    let avg_body_first = helpers::trailing_avg_body(bars, index - 2, 10);
    if !is_body_long_f(first_body, avg_body_first, first.range(), self.body_long_factor) {
      return None;
    }

    // TA-Lib: second BodyDoji (per-candle trailing avg_range at i-1)
    let second_body = second.body();
    let avg_range_second = helpers::trailing_avg_range(bars, index - 1, 10);
    if !is_doji_f(second_body, avg_range_second, second.range(), self.doji_factor) {
      return None;
    }

    // TA-Lib: RealBodyGapDown between second and first
    let second_body_top = second.open().max(second.close());
    let first_body_bottom = first.open().min(first.close());
    if second_body_top >= first_body_bottom {
      return None;
    }

    // TA-Lib: third body > BodyShort average (per-candle trailing at i)
    let third_body = third.body();
    let avg_body_third = helpers::trailing_avg_body(bars, index, 10);
    if third_body <= avg_body_third {
      return None;
    }

    // TA-Lib: close[i] > close[i-2] + body[i-2] * optInPenetration (default 0.3)
    let penetration = self.penetration;
    if third.close() <= first.close() + first_body * penetration {
      return None;
    }

    Some(PatternMatch {
      pattern_id:  PatternDetector::id(self),
      direction:   Direction::Bullish,
      strength:    0.8,
      start_index: index - 2,
      end_index:   index,
    })
  }
}

/// CDLEVENINGDOJISTAR - Evening Doji Star (TA-Lib compatible)
#[derive(Debug, Clone)]
pub struct EveningDojiStarDetector {
  pub body_long_factor: f64,
  pub doji_factor: f64,
  pub penetration: f64,
}

impl Default for EveningDojiStarDetector {
  fn default() -> Self {
    Self {
      body_long_factor: helpers::BODY_LONG_FACTOR,
      doji_factor: helpers::DOJI_FACTOR,
      penetration: 0.3,
    }
  }
}

impl EveningDojiStarDetector {
  pub fn with_defaults() -> Self {
    Self::default()
  }
}

impl PatternDetector for EveningDojiStarDetector {
  fn id(&self) -> PatternId {
    PatternId("CDL_EVENINGDOJISTAR")
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

    // TA-Lib: TA_CANDLECOLOR(i-2) == 1 (white: close >= open)
    if first.close() < first.open() {
      return None;
    }
    // TA-Lib: TA_CANDLECOLOR(i) == -1 (black: close < open)
    if third.close() >= third.open() {
      return None;
    }

    // TA-Lib: first BodyLong (per-candle trailing at i-2)
    let first_body = first.body();
    let avg_body_first = helpers::trailing_avg_body(bars, index - 2, 10);
    if !is_body_long_f(first_body, avg_body_first, first.range(), self.body_long_factor) {
      return None;
    }

    // TA-Lib: second BodyDoji (per-candle trailing avg_range at i-1)
    let second_body = second.body();
    let avg_range_second = helpers::trailing_avg_range(bars, index - 1, 10);
    if !is_doji_f(second_body, avg_range_second, second.range(), self.doji_factor) {
      return None;
    }

    // TA-Lib: RealBodyGapUp between second and first
    let second_body_bottom = second.open().min(second.close());
    let first_body_top = first.open().max(first.close());
    if second_body_bottom <= first_body_top {
      return None;
    }

    // TA-Lib: third body > BodyShort average (per-candle trailing at i)
    let third_body = third.body();
    let avg_body_third = helpers::trailing_avg_body(bars, index, 10);
    if third_body <= avg_body_third {
      return None;
    }

    // TA-Lib: close[i] < close[i-2] - body[i-2] * optInPenetration (default 0.3)
    let penetration = self.penetration;
    if third.close() >= first.close() - first_body * penetration {
      return None;
    }

    Some(PatternMatch {
      pattern_id:  PatternDetector::id(self),
      direction:   Direction::Bearish,
      strength:    0.8,
      start_index: index - 2,
      end_index:   index,
    })
  }
}

// ============================================================
// ABANDONED BABY
// ============================================================

/// CDLABANDONEDBABY - Abandoned Baby (TA-Lib compatible)
#[derive(Debug, Clone)]
pub struct AbandonedBabyDetector {
  pub body_long_factor: f64,
  pub doji_factor: f64,
  pub penetration: f64,
}

impl Default for AbandonedBabyDetector {
  fn default() -> Self {
    Self {
      body_long_factor: helpers::BODY_LONG_FACTOR,
      doji_factor: helpers::DOJI_FACTOR,
      penetration: 0.3,
    }
  }
}

impl AbandonedBabyDetector {
  pub fn with_defaults() -> Self {
    Self::default()
  }
}

impl PatternDetector for AbandonedBabyDetector {
  fn id(&self) -> PatternId {
    PatternId("CDL_ABANDONEDBABY")
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

    // TA-Lib condition 1: first BodyLong
    let first_body = first.body();
    let avg_body_first = helpers::trailing_avg_body(bars, index - 2, 10);
    if !is_body_long_f(first_body, avg_body_first, first.range(), self.body_long_factor) {
      return None;
    }

    // TA-Lib condition 2: second BodyDoji
    let second_body = second.body();
    let avg_range_second = helpers::trailing_avg_range(bars, index - 1, 10);
    if !is_doji_f(second_body, avg_range_second, second.range(), self.doji_factor) {
      return None;
    }

    // TA-Lib condition 3: third body > BodyShort average
    let third_body = third.body();
    let avg_body_third = helpers::trailing_avg_body(bars, index, 10);
    if third_body <= avg_body_third {
      return None;
    }

    let penetration = self.penetration; // TA-Lib default optInPenetration

    // Bearish Abandoned Baby: first white, third black
    if first.close() >= first.open() && third.close() < third.open() {
      // TA-Lib: close[i] < close[i-2] - body[i-2] * penetration
      if third.close() >= first.close() - first_body * penetration {
        return None;
      }
      // TA-Lib: TA_CANDLEGAPUP(i-1, i-2) → low[i-1] > high[i-2]
      if second.low() <= first.high() {
        return None;
      }
      // TA-Lib: TA_CANDLEGAPDOWN(i, i-1) → high[i] < low[i-1]
      if third.high() >= second.low() {
        return None;
      }

      return Some(PatternMatch {
        pattern_id:  PatternDetector::id(self),
        direction:   Direction::Bearish,
        strength:    0.85,
        start_index: index - 2,
        end_index:   index,
      });
    }

    // Bullish Abandoned Baby: first black, third white
    if first.close() < first.open() && third.close() >= third.open() {
      // TA-Lib: close[i] > close[i-2] + body[i-2] * penetration
      if third.close() <= first.close() + first_body * penetration {
        return None;
      }
      // TA-Lib: TA_CANDLEGAPDOWN(i-1, i-2) → high[i-1] < low[i-2]
      if second.high() >= first.low() {
        return None;
      }
      // TA-Lib: TA_CANDLEGAPUP(i, i-1) → low[i] > high[i-1]
      if third.low() <= second.high() {
        return None;
      }

      return Some(PatternMatch {
        pattern_id:  PatternDetector::id(self),
        direction:   Direction::Bullish,
        strength:    0.85,
        start_index: index - 2,
        end_index:   index,
      });
    }

    None
  }
}

// ============================================================
// TWO CROWS / UPSIDE GAP TWO CROWS
// ============================================================

/// CDL2CROWS - Two Crows
#[derive(Debug, Clone)]
pub struct TwoCrowsDetector {
  pub body_long_factor: f64,
}

impl Default for TwoCrowsDetector {
  fn default() -> Self {
    Self {
      body_long_factor: helpers::BODY_LONG_FACTOR,
    }
  }
}

impl TwoCrowsDetector {
  pub fn with_defaults() -> Self {
    Self::default()
  }
}

impl PatternDetector for TwoCrowsDetector {
  fn id(&self) -> PatternId {
    PatternId("CDL_2CROWS")
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

    // TA-Lib: TA_CANDLECOLOR(i-2) == 1 (white: close >= open)
    if first.close() < first.open() {
      return None;
    }
    // TA-Lib: first BodyLong (per-candle trailing at i-2)
    let avg_body_first = helpers::trailing_avg_body(bars, index - 2, 10);
    if !is_body_long_f(first.body(), avg_body_first, first.range(), self.body_long_factor) {
      return None;
    }
    // TA-Lib: TA_CANDLECOLOR(i-1) == -1 (black: close < open)
    if second.close() >= second.open() {
      return None;
    }
    // TA-Lib: TA_REALBODYGAPUP(i-1, i-2)
    // min(open[i-1], close[i-1]) > max(open[i-2], close[i-2])
    let second_body_bottom = second.open().min(second.close());
    let first_body_top = first.open().max(first.close());
    if second_body_bottom <= first_body_top {
      return None;
    }
    // TA-Lib: TA_CANDLECOLOR(i) == -1 (black: close < open)
    if third.close() >= third.open() {
      return None;
    }
    // TA-Lib: open[i] < open[i-1]
    if third.open() >= second.open() {
      return None;
    }
    // TA-Lib: open[i] > close[i-1]
    if third.open() <= second.close() {
      return None;
    }
    // TA-Lib: close[i] > open[i-2]
    if third.close() <= first.open() {
      return None;
    }
    // TA-Lib: close[i] < close[i-2]
    if third.close() >= first.close() {
      return None;
    }

    Some(PatternMatch {
      pattern_id:  PatternDetector::id(self),
      direction:   Direction::Bearish,
      strength:    0.7,
      start_index: index - 2,
      end_index:   index,
    })
  }
}

/// CDLUPSIDEGAP2CROWS - Upside Gap Two Crows
#[derive(Debug, Clone)]
pub struct UpsideGapTwoCrowsDetector {
  pub body_long_factor: f64,
}

impl Default for UpsideGapTwoCrowsDetector {
  fn default() -> Self {
    Self {
      body_long_factor: helpers::BODY_LONG_FACTOR,
    }
  }
}

impl UpsideGapTwoCrowsDetector {
  pub fn with_defaults() -> Self {
    Self::default()
  }
}

impl PatternDetector for UpsideGapTwoCrowsDetector {
  fn id(&self) -> PatternId {
    PatternId("CDL_UPSIDEGAP2CROWS")
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

    // First long bullish (TA-Lib: BodyLong at i-2, per-candle trailing)
    if !first.is_bullish() {
      return None;
    }
    let first_body = first.body();
    let first_range = first.range();
    let avg_body_first = helpers::trailing_avg_body(bars, index - 2, 10);
    if !is_body_long_f(first_body, avg_body_first, first_range, self.body_long_factor) {
      return None;
    }

    // Second and third bearish
    if !second.is_bearish() || !third.is_bearish() {
      return None;
    }

    // Second gaps up (body above first's body)
    if second.close() <= first.close() {
      return None;
    }

    // Third engulfs second but stays above first's close (gap remains)
    if third.open() <= second.open() {
      return None;
    }
    if third.close() >= second.close() {
      return None;
    }
    if third.close() <= first.close() {
      return None;
    }

    Some(PatternMatch {
      pattern_id:  PatternDetector::id(self),
      direction:   Direction::Bearish,
      strength:    0.65,
      start_index: index - 2,
      end_index:   index,
    })
  }
}

// ============================================================
// IDENTICAL THREE CROWS
// ============================================================

/// CDLIDENTICAL3CROWS - Identical Three Crows
#[derive(Debug, Clone)]
pub struct IdenticalThreeCrowsDetector {
  pub tolerance: Ratio,
}

impl Default for IdenticalThreeCrowsDetector {
  fn default() -> Self {
    Self { tolerance: Ratio::new_const(0.01) }
  }
}

impl IdenticalThreeCrowsDetector {
  pub fn with_defaults() -> Self {
    Self::default()
  }
}

impl PatternDetector for IdenticalThreeCrowsDetector {
  fn id(&self) -> PatternId {
    PatternId("CDL_IDENTICAL3CROWS")
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

    // All three bearish
    if !first.is_bearish() || !second.is_bearish() || !third.is_bearish() {
      return None;
    }

    // Each must have very short lower shadow (TA-Lib: ShadowVeryShort, per-candle trailing)
    let first_lower = first.close() - first.low();
    let second_lower = second.close() - second.low();
    let third_lower = third.close() - third.low();
    let avg_range_first = helpers::trailing_avg_range(bars, index - 2, 10);
    let avg_range_second = helpers::trailing_avg_range(bars, index - 1, 10);
    let avg_range_third = helpers::trailing_avg_range(bars, index, 10);
    if !is_shadow_very_short(first_lower, avg_range_first, first.range())
      || !is_shadow_very_short(second_lower, avg_range_second, second.range())
      || !is_shadow_very_short(third_lower, avg_range_third, third.range())
    {
      return None;
    }

    // TA-Lib: Each opens at or very near previous close (Equal: HighLow, Period=5, per-candle)
    let equal_first = helpers::trailing_avg_range(bars, index - 2, 5) * helpers::EQUAL_FACTOR;
    let equal_second = helpers::trailing_avg_range(bars, index - 1, 5) * helpers::EQUAL_FACTOR;
    if (second.open() - first.close()).abs() > equal_first {
      return None;
    }
    if (third.open() - second.close()).abs() > equal_second {
      return None;
    }

    // Descending closes
    if second.close() >= first.close() || third.close() >= second.close() {
      return None;
    }

    Some(PatternMatch {
      pattern_id:  PatternDetector::id(self),
      direction:   Direction::Bearish,
      strength:    0.8,
      start_index: index - 2,
      end_index:   index,
    })
  }
}

// ============================================================
// ADVANCE BLOCK / STALLED PATTERN
// ============================================================

/// CDLADVANCEBLOCK - Advance Block
#[derive(Debug, Clone)]
pub struct AdvanceBlockDetector {
  pub body_long_factor: f64,
  pub near_factor: f64,
  pub far_factor: f64,
}

impl Default for AdvanceBlockDetector {
  fn default() -> Self {
    Self {
      body_long_factor: helpers::BODY_LONG_FACTOR,
      near_factor: helpers::NEAR_FACTOR,
      far_factor: helpers::FAR_FACTOR,
    }
  }
}

impl AdvanceBlockDetector {
  pub fn with_defaults() -> Self {
    Self::default()
  }
}

impl PatternDetector for AdvanceBlockDetector {
  fn id(&self) -> PatternId {
    PatternId("CDL_ADVANCEBLOCK")
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

    // TA-Lib: All three white (bullish)
    if !first.is_bullish() || !second.is_bullish() || !third.is_bullish() {
      return None;
    }

    // TA-Lib: Higher closes
    if second.close() <= first.close() || third.close() <= second.close() {
      return None;
    }

    let first_body = first.body();
    let first_range = first.range();
    let second_body = second.body();
    let third_body = third.body();

    // Per-candle Near/Far trailing averages
    let near_first = helpers::trailing_avg_range(bars, index - 2, 5) * self.near_factor;
    let near_second = helpers::trailing_avg_range(bars, index - 1, 5) * self.near_factor;
    let far_first = helpers::trailing_avg_range(bars, index - 2, 5) * self.far_factor;
    let far_second = helpers::trailing_avg_range(bars, index - 1, 5) * self.far_factor;

    // TA-Lib: Second opens within/near first body
    if second.open() <= first.open() || second.open() > first.close() + near_first {
      return None;
    }
    // TA-Lib: Third opens within/near second body
    if third.open() <= second.open() || third.open() > second.close() + near_second {
      return None;
    }

    // TA-Lib: First candle must have long body (BodyLong) — per-candle trailing avg
    let avg_body_first = helpers::trailing_avg_body(bars, index - 2, 10);
    if !is_body_long_f(first_body, avg_body_first, first_range, self.body_long_factor) {
      return None;
    }

    // TA-Lib: First candle must have short upper shadow (ShadowShort) — per-candle trailing avg
    let first_upper = first.high() - first.close();
    let avg_shadow_first = helpers::trailing_avg_shadow(bars, index - 2, 10);
    if !super::helpers::is_shadow_short(first_upper, avg_shadow_first, first_range) {
      return None;
    }

    // TA-Lib: one of 4 weakening patterns must be true
    let second_upper = second.high() - second.close();
    let third_upper = third.high() - third.close();

    // Per-candle Near thresholds for patterns A/C
    let near_at_second = helpers::trailing_avg_range(bars, index - 1, 5) * self.near_factor;

    // Pattern A: 2nd body far smaller than 1st AND 3rd not longer than 2nd
    let pattern_a = second_body < first_body - far_first
      && third_body < second_body + near_at_second;

    // Pattern B: 3rd body far smaller than 2nd
    let pattern_b = third_body < second_body - far_second;

    // Pattern C: all three bodies decreasing AND (3rd or 2nd has non-short upper shadow)
    // ShadowShort per-candle at i and i-1
    let avg_shadow_third = helpers::trailing_avg_shadow(bars, index, 10);
    let avg_shadow_second = helpers::trailing_avg_shadow(bars, index - 1, 10);
    let pattern_c = third_body < second_body
      && second_body < first_body
      && (!super::helpers::is_shadow_short(third_upper, avg_shadow_third, third.range())
        || !super::helpers::is_shadow_short(second_upper, avg_shadow_second, second.range()));

    // Pattern D: 3rd body smaller than 2nd AND 3rd has long upper shadow
    let pattern_d = third_body < second_body
      && super::helpers::is_shadow_long(third_upper, third_body, third.range());

    if !pattern_a && !pattern_b && !pattern_c && !pattern_d {
      return None;
    }

    Some(PatternMatch {
      pattern_id:  PatternDetector::id(self),
      direction:   Direction::Bearish,
      strength:    0.6,
      start_index: index - 2,
      end_index:   index,
    })
  }
}

/// CDLSTALLEDPATTERN - Stalled Pattern (Deliberation)
#[derive(Debug, Clone)]
pub struct StalledPatternDetector {
  pub body_long_factor: f64,
  pub body_short_factor: f64,
  pub shadow_veryshort_factor: f64,
  pub near_factor: f64,
}

impl Default for StalledPatternDetector {
  fn default() -> Self {
    Self {
      body_long_factor: helpers::BODY_LONG_FACTOR,
      body_short_factor: helpers::BODY_SHORT_FACTOR,
      shadow_veryshort_factor: helpers::SHADOW_VERYSHORT_FACTOR,
      near_factor: helpers::NEAR_FACTOR,
    }
  }
}

impl StalledPatternDetector {
  pub fn with_defaults() -> Self {
    Self::default()
  }
}

impl PatternDetector for StalledPatternDetector {
  fn id(&self) -> PatternId {
    PatternId("CDL_STALLEDPATTERN")
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

    // TA-Lib: All three white (bullish)
    if !first.is_bullish() || !second.is_bullish() || !third.is_bullish() {
      return None;
    }

    // TA-Lib: Higher closes (strict >)
    if second.close() <= first.close() || third.close() <= second.close() {
      return None;
    }

    // TA-Lib: First and second candles must have long bodies (BodyLong) — per-candle trailing avg
    let first_body = first.body();
    let first_range = first.range();
    let second_body = second.body();
    let second_range = second.range();
    let avg_body_first = helpers::trailing_avg_body(bars, index - 2, 10);
    let avg_body_second = helpers::trailing_avg_body(bars, index - 1, 10);
    if !is_body_long_f(first_body, avg_body_first, first_range, self.body_long_factor) {
      return None;
    }
    if !is_body_long_f(second_body, avg_body_second, second_range, self.body_long_factor) {
      return None;
    }

    // TA-Lib: Second candle has very short upper shadow (ShadowVeryShort) — per-candle trailing avg
    let second_upper = second.high() - second.close();
    let avg_range_second = helpers::trailing_avg_range(bars, index - 1, 10);
    if !is_shadow_very_short_f(second_upper, avg_range_second, second_range, self.shadow_veryshort_factor) {
      return None;
    }

    // TA-Lib: Second opens higher than first, within/near first body — per-candle Near
    let near_first = helpers::trailing_avg_range(bars, index - 2, 5) * self.near_factor;
    let near_second = helpers::trailing_avg_range(bars, index - 1, 5) * self.near_factor;
    if second.open() <= first.open() {
      return None;
    }
    if second.open() > first.close() + near_first {
      return None;
    }

    // TA-Lib: Third candle has short body (BodyShort) — per-candle trailing avg
    let third_body = third.body();
    let third_range = third.range();
    let avg_body_third = helpers::trailing_avg_body(bars, index, 10);
    if !is_body_short_f(third_body, avg_body_third, third_range, self.body_short_factor) {
      return None;
    }

    // TA-Lib: Third candle rides on the shoulder of second body
    // open[i] >= close[i-1] - body[i] - Near_avg at i-1
    if third.open() < second.close() - third_body - near_second {
      return None;
    }

    Some(PatternMatch {
      pattern_id:  PatternDetector::id(self),
      direction:   Direction::Bearish,
      strength:    0.55,
      start_index: index - 2,
      end_index:   index,
    })
  }
}

// ============================================================
// STICK SANDWICH
// ============================================================

/// CDLSTICKSANDWICH - Stick Sandwich
#[derive(Debug, Clone)]
pub struct StickSandwichDetector {
  pub tolerance: Ratio,
}

impl Default for StickSandwichDetector {
  fn default() -> Self {
    Self { tolerance: Ratio::new_const(0.005) }
  }
}

impl StickSandwichDetector {
  pub fn with_defaults() -> Self {
    Self::default()
  }
}

impl PatternDetector for StickSandwichDetector {
  fn id(&self) -> PatternId {
    PatternId("CDL_STICKSANDWICH")
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

    // TA-Lib StickSandwich (TA_CANDLECOLOR convention):
    // 1. first black (close < open), second white (close >= open), third black (close < open)
    if first.close() >= first.open() { return None; }
    if second.close() < second.open() { return None; }
    if third.close() >= third.open() { return None; }

    // 2. second bar's LOW must be above first bar's close (gap up)
    // TA-Lib: low[i-1] > close[i-2] (strict >)
    if second.low() <= first.close() {
      return None;
    }

    // 3. first and third closes are equal (Equal: HighLow, Period=5, Factor=0.05, per-candle at i-2)
    let equal_threshold = helpers::trailing_avg_range(bars, index - 2, 5) * helpers::EQUAL_FACTOR;
    let diff = (first.close() - third.close()).abs();
    if diff > equal_threshold {
      return None;
    }

    Some(PatternMatch {
      pattern_id:  PatternDetector::id(self),
      direction:   Direction::Bullish,
      strength:    0.6,
      start_index: index - 2,
      end_index:   index,
    })
  }
}

// ============================================================
// TASUKI GAP
// ============================================================

/// CDLTASUKIGAP - Tasuki Gap
#[derive(Debug, Clone)]
pub struct TasukiGapDetector {
  pub near_factor: f64,
}

impl Default for TasukiGapDetector {
  fn default() -> Self {
    Self {
      near_factor: helpers::NEAR_FACTOR,
    }
  }
}

impl TasukiGapDetector {
  pub fn with_defaults() -> Self {
    Self::default()
  }
}

impl PatternDetector for TasukiGapDetector {
  fn id(&self) -> PatternId {
    PatternId("CDL_TASUKIGAP")
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

    // TA-Lib: Near threshold at i-1 for body size comparison
    let near_threshold = helpers::trailing_avg_range(bars, index - 1, 5) * self.near_factor;

    // TA-Lib: TA_CANDLECOLOR convention (close >= open = white/+1)
    let second_white = second.close() >= second.open();
    let third_white = third.close() >= third.open();

    // Upside Tasuki Gap: 2nd white, 3rd black
    if second_white && !third_white {
      // TA-Lib: RealBodyGapUp between i-1 and i-2
      let first_body_hi = first.open().max(first.close());
      let second_body_lo = second.open().min(second.close());
      if second_body_lo <= first_body_hi {
        return None;
      }

      // TA-Lib: third opens within second's real body
      // open[i] < close[i-1] && open[i] > open[i-1]
      if third.open() >= second.close() || third.open() <= second.open() {
        return None;
      }
      // TA-Lib: third closes below second's open (into the gap)
      // close[i] < open[i-1]
      if third.close() >= second.open() {
        return None;
      }
      // TA-Lib: third close stays above max(close[i-2], open[i-2]) — gap not filled
      if third.close() <= first_body_hi {
        return None;
      }

      // TA-Lib: body sizes near the same
      if (second.body() - third.body()).abs() >= near_threshold {
        return None;
      }

      return Some(PatternMatch {
        pattern_id:  PatternDetector::id(self),
        direction:   Direction::Bullish,
        strength:    0.6,
        start_index: index - 2,
        end_index:   index,
      });
    }

    // Downside Tasuki Gap: 2nd black, 3rd white
    if !second_white && third_white {
      // TA-Lib: RealBodyGapDown between i-1 and i-2
      let first_body_lo = first.open().min(first.close());
      let second_body_hi = second.open().max(second.close());
      if second_body_hi >= first_body_lo {
        return None;
      }

      // TA-Lib: third opens within second's real body
      // open[i] < open[i-1] && open[i] > close[i-1]
      if third.open() >= second.open() || third.open() <= second.close() {
        return None;
      }
      // TA-Lib: third closes above second's open (into the gap)
      // close[i] > open[i-1]
      if third.close() <= second.open() {
        return None;
      }
      // TA-Lib: third close stays below min(close[i-2], open[i-2]) — gap not filled
      if third.close() >= first_body_lo {
        return None;
      }

      // TA-Lib: body sizes near the same
      if (second.body() - third.body()).abs() >= near_threshold {
        return None;
      }

      return Some(PatternMatch {
        pattern_id:  PatternDetector::id(self),
        direction:   Direction::Bearish,
        strength:    0.6,
        start_index: index - 2,
        end_index:   index,
      });
    }

    None
  }
}

// ============================================================
// TRISTAR
// ============================================================

/// CDLTRISTAR - Tristar Pattern (TA-Lib compatible)
#[derive(Debug, Clone)]
pub struct TristarDetector {
  pub doji_factor: f64,
}

impl Default for TristarDetector {
  fn default() -> Self {
    Self {
      doji_factor: helpers::DOJI_FACTOR,
    }
  }
}

impl TristarDetector {
  pub fn with_defaults() -> Self {
    Self::default()
  }
}

impl PatternDetector for TristarDetector {
  fn id(&self) -> PatternId {
    PatternId("CDL_TRISTAR")
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

    // TA-Lib: All three must be doji
    // TA-Lib uses TA_CANDLEAVERAGE(BodyDoji, BodyPeriodTotal, i-2) for ALL THREE candles
    // (same average computed at position i-2)
    let first_body = first.body();
    let first_range = first.range();
    let second_body = second.body();
    let second_range = second.range();
    let third_body = third.body();
    let third_range = third.range();

    let avg_range_first = helpers::trailing_avg_range(bars, index - 2, 10);

    if !is_doji_f(first_body, avg_range_first, first_range, self.doji_factor)
      || !is_doji_f(second_body, avg_range_first, second_range, self.doji_factor)
      || !is_doji_f(third_body, avg_range_first, third_range, self.doji_factor)
    {
      return None;
    }

    // TA-Lib uses REALBODYGAPUP/DOWN macros for proper body gap detection
    let first_body_hi = first.open().max(first.close());
    let first_body_lo = first.open().min(first.close());
    let second_body_hi = second.open().max(second.close());
    let second_body_lo = second.open().min(second.close());
    let third_body_hi = third.open().max(third.close());
    let third_body_lo = third.open().min(third.close());

    // Bearish Tristar: 2nd gaps up from 1st, 3rd body top below 2nd body top
    if second_body_lo > first_body_hi && third_body_hi < second_body_hi {
      return Some(PatternMatch {
        pattern_id:  PatternDetector::id(self),
        direction:   Direction::Bearish,
        strength:    0.7,
        start_index: index - 2,
        end_index:   index,
      });
    }

    // Bullish Tristar: 2nd gaps down from 1st, 3rd body bottom above 2nd body bottom
    if second_body_hi < first_body_lo && third_body_lo > second_body_lo {
      return Some(PatternMatch {
        pattern_id:  PatternDetector::id(self),
        direction:   Direction::Bullish,
        strength:    0.7,
        start_index: index - 2,
        end_index:   index,
      });
    }

    None
  }
}

// ============================================================
// UNIQUE 3 RIVER
// ============================================================

/// CDLUNIQUE3RIVER - Unique 3 River
#[derive(Debug, Clone)]
pub struct Unique3RiverDetector {
  pub body_long_factor: f64,
  pub body_short_factor: f64,
}

impl Default for Unique3RiverDetector {
  fn default() -> Self {
    Self {
      body_long_factor: helpers::BODY_LONG_FACTOR,
      body_short_factor: helpers::BODY_SHORT_FACTOR,
    }
  }
}

impl Unique3RiverDetector {
  pub fn with_defaults() -> Self {
    Self::default()
  }
}

impl PatternDetector for Unique3RiverDetector {
  fn id(&self) -> PatternId {
    PatternId("CDL_UNIQUE3RIVER")
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

    // TA-Lib condition 1: first BodyLong + black (close < open)
    if first.close() >= first.open() {
      return None;
    }
    let avg_body_first = helpers::trailing_avg_body(bars, index - 2, 10);
    if !is_body_long_f(first.body(), avg_body_first, first.range(), self.body_long_factor) {
      return None;
    }

    // TA-Lib condition 2: second black (close < open)
    if second.close() >= second.open() {
      return None;
    }
    // TA-Lib condition 3: close[i-1] > close[i-2] (second close above first close)
    if second.close() <= first.close() {
      return None;
    }
    // TA-Lib condition 4: open[i-1] <= open[i-2] (second open at or below first open)
    if second.open() > first.open() {
      return None;
    }
    // TA-Lib condition 5: low[i-1] < low[i-2] (second makes new low)
    if second.low() >= first.low() {
      return None;
    }

    // TA-Lib condition 6: third BodyShort + white (close >= open)
    if third.close() < third.open() {
      return None;
    }
    let avg_body_third = helpers::trailing_avg_body(bars, index, 10);
    if !is_body_short_f(third.body(), avg_body_third, third.range(), self.body_short_factor) {
      return None;
    }
    // TA-Lib condition 7: open[i] > low[i-1]
    if third.open() <= second.low() {
      return None;
    }

    Some(PatternMatch {
      pattern_id:  PatternDetector::id(self),
      direction:   Direction::Bullish,
      strength:    0.6,
      start_index: index - 2,
      end_index:   index,
    })
  }
}

// ============================================================
// TWEEZER TOP / TWEEZER BOTTOM
// ============================================================

/// Tweezer Top
#[derive(Debug, Clone)]
pub struct TweezerTopDetector {
  pub tolerance: Ratio,
}

impl Default for TweezerTopDetector {
  fn default() -> Self {
    Self { tolerance: Ratio::new_const(0.001) }
  }
}

impl TweezerTopDetector {
  pub fn with_defaults() -> Self {
    Self::default()
  }
}

impl PatternDetector for TweezerTopDetector {
  fn id(&self) -> PatternId {
    PatternId("CDL_TWEEZERTOP")
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

    if !ctx.trend.is_up() {
      return None;
    }
    if !prev.is_bullish() || !curr.is_bearish() {
      return None;
    }

    // Matching highs
    let diff = (prev.high() - curr.high()).abs();
    let avg = (prev.high() + curr.high()) / 2.0;
    if avg <= 0.0 || diff / avg > self.tolerance.get() {
      return None;
    }

    Some(PatternMatch {
      pattern_id:  PatternDetector::id(self),
      direction:   Direction::Bearish,
      strength:    0.6,
      start_index: index - 1,
      end_index:   index,
    })
  }
}

/// Tweezer Bottom
#[derive(Debug, Clone)]
pub struct TweezerBottomDetector {
  pub tolerance: Ratio,
}

impl Default for TweezerBottomDetector {
  fn default() -> Self {
    Self { tolerance: Ratio::new_const(0.001) }
  }
}

impl TweezerBottomDetector {
  pub fn with_defaults() -> Self {
    Self::default()
  }
}

impl PatternDetector for TweezerBottomDetector {
  fn id(&self) -> PatternId {
    PatternId("CDL_TWEEZERBOTTOM")
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

    if !ctx.trend.is_down() {
      return None;
    }
    if !prev.is_bearish() || !curr.is_bullish() {
      return None;
    }

    // Matching lows
    let diff = (prev.low() - curr.low()).abs();
    let avg = (prev.low() + curr.low()) / 2.0;
    if avg <= 0.0 || diff / avg > self.tolerance.get() {
      return None;
    }

    Some(PatternMatch {
      pattern_id:  PatternDetector::id(self),
      direction:   Direction::Bullish,
      strength:    0.6,
      start_index: index - 1,
      end_index:   index,
    })
  }
}

// ============================================================
// PARAMETERIZED DETECTOR IMPLEMENTATIONS
// ============================================================

static THREE_WHITE_SOLDIERS_PARAMS: &[ParamMeta] = &[
  ParamMeta { name: "shadow_veryshort_factor", param_type: ParamType::Ratio, default: 0.1, range: (0.05, 0.2, 0.05), description: "Shadow very short threshold factor" },
  ParamMeta { name: "near_factor", param_type: ParamType::Ratio, default: 0.2, range: (0.1, 0.4, 0.1), description: "Near threshold factor" },
  ParamMeta { name: "far_factor", param_type: ParamType::Ratio, default: 0.6, range: (0.3, 0.9, 0.1), description: "Far threshold factor" },
  ParamMeta { name: "body_short_factor", param_type: ParamType::Ratio, default: 1.0, range: (0.5, 1.5, 0.1), description: "Body short threshold factor" },
];

static THREE_BLACK_CROWS_PARAMS: &[ParamMeta] = &[
  ParamMeta { name: "shadow_veryshort_factor", param_type: ParamType::Ratio, default: 0.1, range: (0.05, 0.2, 0.05), description: "Shadow very short threshold factor" },
];

static IDENTICAL_THREE_CROWS_PARAMS: &[ParamMeta] = &[ParamMeta {
  name:        "tolerance",
  param_type:  ParamType::Ratio,
  default:     0.01,
  range:       (0.005, 0.03, 0.005),
  description: "Open/close tolerance",
}];

static STICK_SANDWICH_PARAMS: &[ParamMeta] = &[ParamMeta {
  name:        "tolerance",
  param_type:  ParamType::Ratio,
  default:     0.005,
  range:       (0.002, 0.01, 0.002),
  description: "Close price tolerance",
}];

static TWEEZER_TOP_PARAMS: &[ParamMeta] = &[ParamMeta {
  name:        "tolerance",
  param_type:  ParamType::Ratio,
  default:     0.001,
  range:       (0.0005, 0.003, 0.0005),
  description: "High price tolerance",
}];

static TWEEZER_BOTTOM_PARAMS: &[ParamMeta] = &[ParamMeta {
  name:        "tolerance",
  param_type:  ParamType::Ratio,
  default:     0.001,
  range:       (0.0005, 0.003, 0.0005),
  description: "Low price tolerance",
}];

impl ParameterizedDetector for ThreeWhiteSoldiersDetector {
  fn param_meta() -> &'static [ParamMeta] {
    THREE_WHITE_SOLDIERS_PARAMS
  }

  fn with_params(params: &HashMap<&str, f64>) -> Result<Self> {
    Ok(Self {
      shadow_veryshort_factor: params.get("shadow_veryshort_factor").copied().unwrap_or(helpers::SHADOW_VERYSHORT_FACTOR),
      near_factor: params.get("near_factor").copied().unwrap_or(helpers::NEAR_FACTOR),
      far_factor: params.get("far_factor").copied().unwrap_or(helpers::FAR_FACTOR),
      body_short_factor: params.get("body_short_factor").copied().unwrap_or(helpers::BODY_SHORT_FACTOR),
    })
  }

  fn pattern_id_str() -> &'static str {
    "CDL_3WHITESOLDIERS"
  }
}

impl ParameterizedDetector for ThreeBlackCrowsDetector {
  fn param_meta() -> &'static [ParamMeta] {
    THREE_BLACK_CROWS_PARAMS
  }

  fn with_params(params: &HashMap<&str, f64>) -> Result<Self> {
    Ok(Self {
      shadow_veryshort_factor: params.get("shadow_veryshort_factor").copied().unwrap_or(helpers::SHADOW_VERYSHORT_FACTOR),
    })
  }

  fn pattern_id_str() -> &'static str {
    "CDL_3BLACKCROWS"
  }
}

impl ParameterizedDetector for IdenticalThreeCrowsDetector {
  fn param_meta() -> &'static [ParamMeta] {
    IDENTICAL_THREE_CROWS_PARAMS
  }

  fn with_params(params: &HashMap<&str, f64>) -> Result<Self> {
    Ok(Self { tolerance: get_ratio(params, "tolerance", 0.01)? })
  }

  fn pattern_id_str() -> &'static str {
    "CDL_IDENTICAL3CROWS"
  }
}

impl ParameterizedDetector for StickSandwichDetector {
  fn param_meta() -> &'static [ParamMeta] {
    STICK_SANDWICH_PARAMS
  }

  fn with_params(params: &HashMap<&str, f64>) -> Result<Self> {
    Ok(Self { tolerance: get_ratio(params, "tolerance", 0.005)? })
  }

  fn pattern_id_str() -> &'static str {
    "CDL_STICKSANDWICH"
  }
}

impl ParameterizedDetector for TweezerTopDetector {
  fn param_meta() -> &'static [ParamMeta] {
    TWEEZER_TOP_PARAMS
  }

  fn with_params(params: &HashMap<&str, f64>) -> Result<Self> {
    Ok(Self { tolerance: get_ratio(params, "tolerance", 0.001)? })
  }

  fn pattern_id_str() -> &'static str {
    "CDL_TWEEZERTOP"
  }
}

impl ParameterizedDetector for TweezerBottomDetector {
  fn param_meta() -> &'static [ParamMeta] {
    TWEEZER_BOTTOM_PARAMS
  }

  fn with_params(params: &HashMap<&str, f64>) -> Result<Self> {
    Ok(Self { tolerance: get_ratio(params, "tolerance", 0.001)? })
  }

  fn pattern_id_str() -> &'static str {
    "CDL_TWEEZERBOTTOM"
  }
}
