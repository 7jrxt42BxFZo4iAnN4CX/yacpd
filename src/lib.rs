//! # YACPD - Yet Another Candlestick Pattern Detector
//!
//! High-performance candlestick pattern detection library for technical analysis.
//!
//! ## Quick Start
//!
//! ```rust
//! use yacpd::prelude::*;
//!
//! // Define your OHLCV data
//! struct Bar { o: f64, h: f64, l: f64, c: f64, v: f64 }
//!
//! impl OHLCV for Bar {
//!     fn open(&self) -> f64 { self.o }
//!     fn high(&self) -> f64 { self.h }
//!     fn low(&self) -> f64 { self.l }
//!     fn close(&self) -> f64 { self.c }
//!     fn volume(&self) -> f64 { self.v }
//! }
//!
//! // Create engine with default detectors
//! let engine = EngineBuilder::new()
//!     .with_all_defaults()
//!     .build()
//!     .unwrap();
//!
//! // Scan your data
//! let bars: Vec<Bar> = vec![];
//! let patterns = engine.scan(&bars).unwrap();
//! ```

pub mod detectors;
pub mod params;

pub mod prelude {
    pub use crate::{
        // Detectors
        detectors::*,
        // Parameters
        params::{get_period, get_ratio, ParamMeta, ParamType, ParameterizedDetector},
        // Parallel
        scan_parallel,
        // Iterator
        BarPatterns,
        // Engine
        BuiltinDetector,
        // Types
        ContextProvider,
        Direction,
        // Core traits
        DynPatternDetector,
        EngineBuilder,
        MarketContext,
        OHLCVExt,
        PatternDetector,
        PatternEngine,
        // Errors
        PatternError,
        PatternId,
        PatternIterator,
        PatternMatch,
        Period,
        Ratio,
        Result,
        ScanError,
        ScanResult,
        Trend,
        OHLCV,
    };
}

// ============================================================
// ERRORS
// ============================================================

pub type Result<T> = std::result::Result<T, PatternError>;

/// Errors that can occur during pattern detection
#[derive(Debug, Clone, thiserror::Error)]
pub enum PatternError {
    #[error("Invalid value: {0}")]
    InvalidValue(&'static str),

    #[error("{field} = {value} out of range [{min}, {max}]")]
    OutOfRange {
        field: &'static str,
        value: f64,
        min: f64,
        max: f64,
    },

    #[error("Invalid config: {0}")]
    InvalidConfig(String),

    #[error("Insufficient data: need {need} bars, got {got}")]
    InsufficientData { need: usize, got: usize },

    #[error("Invalid OHLCV at index {index}: {reason}")]
    InvalidOHLCV { index: usize, reason: &'static str },
}

// ============================================================
// VALIDATED TYPES
// ============================================================

/// Normalized value in range 0.0..=1.0
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct Ratio(f64);

impl Ratio {
    /// Create a new Ratio, validating the value is in [0.0, 1.0]
    pub fn new(value: f64) -> Result<Self> {
        if value.is_nan() || value.is_infinite() {
            return Err(PatternError::InvalidValue(
                "Ratio cannot be NaN or infinite",
            ));
        }
        if !(0.0..=1.0).contains(&value) {
            return Err(PatternError::OutOfRange {
                field: "Ratio",
                value,
                min: 0.0,
                max: 1.0,
            });
        }
        Ok(Self(value))
    }

    /// Create a Ratio from a compile-time constant (library internal use)
    #[doc(hidden)]
    pub const fn new_const(value: f64) -> Self {
        Self(value)
    }

    #[inline]
    pub fn get(self) -> f64 {
        self.0
    }
}

impl serde::Serialize for Ratio {
    fn serialize<S: serde::Serializer>(&self, s: S) -> std::result::Result<S::Ok, S::Error> {
        self.0.serialize(s)
    }
}

impl<'de> serde::Deserialize<'de> for Ratio {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> std::result::Result<Self, D::Error> {
        let value = f64::deserialize(d)?;
        Ratio::new(value).map_err(serde::de::Error::custom)
    }
}

/// Period (must be > 0)
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Period(usize);

impl Period {
    /// Create a new Period, validating value is > 0
    pub fn new(value: usize) -> Result<Self> {
        if value == 0 {
            return Err(PatternError::InvalidValue("Period must be > 0"));
        }
        Ok(Self(value))
    }

    #[doc(hidden)]
    pub const fn new_const(value: usize) -> Self {
        Self(value)
    }

    #[inline]
    pub fn get(self) -> usize {
        self.0
    }
}

impl serde::Serialize for Period {
    fn serialize<S: serde::Serializer>(&self, s: S) -> std::result::Result<S::Ok, S::Error> {
        self.0.serialize(s)
    }
}

impl<'de> serde::Deserialize<'de> for Period {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> std::result::Result<Self, D::Error> {
        let value = usize::deserialize(d)?;
        Period::new(value).map_err(serde::de::Error::custom)
    }
}

// ============================================================
// OHLCV TRAITS
// ============================================================

/// Core OHLCV data trait
pub trait OHLCV {
    fn open(&self) -> f64;
    fn high(&self) -> f64;
    fn low(&self) -> f64;
    fn close(&self) -> f64;
    fn volume(&self) -> f64;

    fn timestamp(&self) -> Option<i64> {
        None
    }
}

/// Blanket impl for references to dyn OHLCV
impl OHLCV for &dyn OHLCV {
    fn open(&self) -> f64 {
        (*self).open()
    }

    fn high(&self) -> f64 {
        (*self).high()
    }

    fn low(&self) -> f64 {
        (*self).low()
    }

    fn close(&self) -> f64 {
        (*self).close()
    }

    fn volume(&self) -> f64 {
        (*self).volume()
    }

    fn timestamp(&self) -> Option<i64> {
        (*self).timestamp()
    }
}

/// Extension trait with computed properties for OHLCV data
pub trait OHLCVExt: OHLCV {
    #[inline]
    fn body(&self) -> f64 {
        (self.close() - self.open()).abs()
    }

    #[inline]
    fn range(&self) -> f64 {
        self.high() - self.low()
    }

    #[inline]
    fn upper_shadow(&self) -> f64 {
        self.high() - self.open().max(self.close())
    }

    #[inline]
    fn lower_shadow(&self) -> f64 {
        self.open().min(self.close()) - self.low()
    }

    #[inline]
    fn is_bullish(&self) -> bool {
        self.close() > self.open()
    }

    #[inline]
    fn is_bearish(&self) -> bool {
        self.close() < self.open()
    }

    /// Body as ratio of range. Returns None if range ≈ 0
    #[inline]
    fn body_ratio(&self) -> Option<f64> {
        let range = self.range();
        (range > f64::EPSILON).then(|| self.body() / range)
    }

    #[inline]
    fn upper_shadow_ratio(&self) -> Option<f64> {
        let range = self.range();
        (range > f64::EPSILON).then(|| self.upper_shadow() / range)
    }

    #[inline]
    fn lower_shadow_ratio(&self) -> Option<f64> {
        let range = self.range();
        (range > f64::EPSILON).then(|| self.lower_shadow() / range)
    }

    /// Validate OHLCV data consistency
    fn validate(&self) -> Result<()> {
        if self.high() < self.low() {
            return Err(PatternError::InvalidOHLCV {
                index: 0,
                reason: "high < low",
            });
        }
        if self.open().is_nan()
            || self.high().is_nan()
            || self.low().is_nan()
            || self.close().is_nan()
        {
            return Err(PatternError::InvalidOHLCV {
                index: 0,
                reason: "NaN in OHLCV",
            });
        }
        if self.open().is_infinite()
            || self.high().is_infinite()
            || self.low().is_infinite()
            || self.close().is_infinite()
        {
            return Err(PatternError::InvalidOHLCV {
                index: 0,
                reason: "Infinite value in OHLCV",
            });
        }
        Ok(())
    }
}

impl<T: OHLCV> OHLCVExt for T {}

// ============================================================
// PATTERN MATCH - result of detection (Copy, no allocations)
// ============================================================

/// Unique identifier for a pattern type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PatternId(pub &'static str);

impl PatternId {
    /// Returns the string identifier
    #[inline]
    pub fn as_str(&self) -> &'static str {
        self.0
    }

    /// Returns the typical/expected direction of this pattern.
    ///
    /// - `Some(Direction::Bullish)` - pattern typically signals bullish moves
    /// - `Some(Direction::Bearish)` - pattern typically signals bearish moves
    /// - `Some(Direction::Neutral)` - pattern has no directional bias
    /// - `None` - pattern is bidirectional (can be bullish or bearish depending on context)
    pub fn typical_direction(&self) -> Option<Direction> {
        match self.0 {
            // Bullish patterns
            "CDL_3WHITESOLDIERS"
            | "CDL_3STARSINSOUTH"
            | "CDL_HAMMER"
            | "CDL_INVERTEDHAMMER"
            | "CDL_MORNINGSTAR"
            | "CDL_MORNINGDOJISTAR"
            | "CDL_PIERCING"
            | "CDL_LADDERBOTTOM"
            | "CDL_HOMINGPIGEON"
            | "CDL_TAKURI"
            | "CDL_UNIQUE3RIVER"
            | "CDL_MATCHINGLOW"
            | "CDL_STICKSANDWICH"
            | "CDL_DRAGONFLYDOJI"
            | "CDL_MATHOLD"
            | "CDL_GAPSIDESIDEWHITE"
            | "RISING_WINDOW"
            | "GAPPING_UP_DOJI"
            | "WHITE_MARUBOZU"
            | "OPENING_WHITE_MARUBOZU"
            | "LONG_WHITE_DAY"
            | "WHITE_CANDLE"
            | "WHITE_SPINNING_TOP"
            | "SHORT_WHITE"
            | "ABOVE_THE_STOMACH"
            | "LAST_ENGULFING_BOTTOM"
            | "MEETING_LINES_BULLISH"
            | "UPSIDE_TASUKI_GAP"
            | "UPSIDE_GAP_THREE_METHODS"
            | "CDL_TWEEZERBOTTOM" => Some(Direction::Bullish),
            // Bearish patterns
            "CDL_3BLACKCROWS"
            | "CDL_2CROWS"
            | "CDL_IDENTICAL3CROWS"
            | "CDL_EVENINGSTAR"
            | "CDL_EVENINGDOJISTAR"
            | "CDL_SHOOTINGSTAR"
            | "CDL_HANGINGMAN"
            | "CDL_DARKCLOUDCOVER"
            | "CDL_ADVANCEBLOCK"
            | "CDL_STALLEDPATTERN"
            | "CDL_UPSIDEGAP2CROWS"
            | "CDL_THRUSTING"
            | "CDL_INNECK"
            | "CDL_ONNECK"
            | "CDL_CONCEALBABYSWALL"
            | "CDL_GRAVESTONEDOJI"
            | "FALLING_WINDOW"
            | "GAPPING_DOWN_DOJI"
            | "BLACK_MARUBOZU"
            | "OPENING_BLACK_MARUBOZU"
            | "LONG_BLACK_DAY"
            | "BLACK_CANDLE"
            | "BLACK_SPINNING_TOP"
            | "SHORT_BLACK"
            | "BELOW_THE_STOMACH"
            | "LAST_ENGULFING_TOP"
            | "MEETING_LINES_BEARISH"
            | "DOWNSIDE_TASUKI_GAP"
            | "DOWNSIDE_GAP_THREE_METHODS"
            | "TWO_BLACK_GAPPING"
            | "SHOOTING_STAR_2_LINES"
            | "COLLAPSING_DOJI_STAR"
            | "DELIBERATION"
            | "CDL_TWEEZERTOP" => Some(Direction::Bearish),
            // Neutral patterns
            "CDL_DOJI"
            | "CDL_LONGLEGGEDDOJI"
            | "CDL_RICKSHAWMAN"
            | "CDL_HIGHWAVE"
            | "CDL_SPINNINGTOP"
            | "CDL_CLOSINGMARUBOZU"
            | "CDL_MARUBOZU"
            | "CDL_LONGLINE"
            | "CDL_SHORTLINE"
            | "PRICE_LINES"
            | "NORTHERN_DOJI"
            | "SOUTHERN_DOJI" => Some(Direction::Neutral),
            // Bidirectional patterns (return None)
            "CDL_ENGULFING"
            | "CDL_3INSIDE"
            | "CDL_3OUTSIDE"
            | "CDL_3LINESTRIKE"
            | "CDL_HARAMI"
            | "CDL_HARAMICROSS"
            | "CDL_KICKING"
            | "CDL_KICKINGBYLENGTH"
            | "CDL_BELTHOLD"
            | "CDL_COUNTERATTACK"
            | "CDL_SEPARATINGLINES"
            | "CDL_RISEFALL3METHODS"
            | "CDL_BREAKAWAY"
            | "CDL_ABANDONEDBABY"
            | "CDL_TASUKIGAP"
            | "CDL_XSIDEGAP3METHODS"
            | "CDL_HIKKAKE"
            | "CDL_HIKKAKEMOD"
            | "CDL_TRISTAR"
            | "CDL_DOJISTAR" => None,
            // Default to None for unknown patterns
            _ => None,
        }
    }

    /// Returns true if this pattern can signal both bullish and bearish moves
    /// depending on the market context.
    pub fn is_bidirectional(&self) -> bool {
        self.typical_direction().is_none()
    }

    /// Returns true if this pattern typically signals bullish moves
    pub fn is_typically_bullish(&self) -> bool {
        matches!(self.typical_direction(), Some(Direction::Bullish))
    }

    /// Returns true if this pattern typically signals bearish moves
    pub fn is_typically_bearish(&self) -> bool {
        matches!(self.typical_direction(), Some(Direction::Bearish))
    }

    /// Returns true if this pattern has no directional bias
    pub fn is_neutral(&self) -> bool {
        matches!(self.typical_direction(), Some(Direction::Neutral))
    }
}

/// Direction/bias of a pattern
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum Direction {
    Bullish,
    Neutral,
    Bearish,
}

impl Direction {
    #[inline]
    pub fn is_bullish(self) -> bool {
        matches!(self, Direction::Bullish)
    }

    #[inline]
    pub fn is_bearish(self) -> bool {
        matches!(self, Direction::Bearish)
    }
}

/// Result of pattern detection - Copy, no allocations
#[derive(Debug, Clone, Copy)]
pub struct PatternMatch {
    pub pattern_id: PatternId,
    pub direction: Direction,
    /// Quality/confidence score 0.0..=1.0
    pub strength: f64,
    pub start_index: usize,
    pub end_index: usize,
}

// ============================================================
// MARKET CONTEXT
// ============================================================

/// Market trend classification
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum Trend {
    StrongUp,
    WeakUp,
    #[default]
    Sideways,
    WeakDown,
    StrongDown,
}

impl Trend {
    #[inline]
    pub fn is_down(self) -> bool {
        matches!(self, Trend::WeakDown | Trend::StrongDown)
    }

    #[inline]
    pub fn is_up(self) -> bool {
        matches!(self, Trend::WeakUp | Trend::StrongUp)
    }
}

/// Market context at a specific bar (TA-Lib compatible)
#[derive(Debug, Clone, Copy, Default)]
pub struct MarketContext {
    pub trend: Trend,
    pub volatility: f64,
    pub avg_volume: f64,
    /// Average body size over lookback period (TA-Lib: TA_CANDLEAVGPERIOD = 10)
    pub avg_body: f64,
    /// Average upper shadow over lookback period
    pub avg_upper_shadow: f64,
    /// Average lower shadow over lookback period
    pub avg_lower_shadow: f64,
    /// Average range (high - low) over lookback period
    pub avg_range: f64,
    /// Average per-shadow length: mean(upper + lower) / 2 over lookback period.
    /// TA-Lib: ShadowShort uses RangeType=Shadows (upper+lower), then CandleAverage divides by 2.
    pub avg_shadow: f64,
    /// Average range (high - low) over 5-bar trailing period.
    /// TA-Lib uses Period=5 for Near, Far, and Equal candle settings.
    pub avg_range_5: f64,
}

/// Provider of market context - precomputes context for all bars
pub trait ContextProvider: Send + Sync {
    fn compute_all<T: OHLCV>(&self, bars: &[T]) -> Vec<MarketContext>;
}

/// Default context provider using simple moving averages (TA-Lib compatible)
#[derive(Debug, Clone)]
pub struct DefaultContextProvider {
    pub trend_period: Period,
    pub volume_period: Period,
    /// TA-Lib uses 10 bars for candle averaging (TA_CANDLEAVGPERIOD)
    pub candle_period: Period,
}

impl Default for DefaultContextProvider {
    fn default() -> Self {
        Self {
            trend_period: Period::new_const(14),
            volume_period: Period::new_const(20),
            candle_period: Period::new_const(10), // TA-Lib default
        }
    }
}

impl ContextProvider for DefaultContextProvider {
    fn compute_all<T: OHLCV>(&self, bars: &[T]) -> Vec<MarketContext> {
        let len = bars.len();
        let mut contexts = Vec::with_capacity(len);

        for i in 0..len {
            // TA-Lib compatible: trailing average over bars BEFORE the current bar.
            // At bar i, average is computed from bars[max(0, i-period)..i] (NOT including bar i).
            // This matches TA-Lib's rolling sum which updates AFTER the pattern check.
            let candle_period = self.candle_period.get();

            let (avg_body, avg_upper_shadow, avg_lower_shadow, avg_range, avg_shadow) = if i == 0 {
                // No trailing bars available; use current bar as fallback
                let bar = &bars[0];
                let body = bar.body();
                let upper = bar.upper_shadow();
                let lower = bar.lower_shadow();
                let range = bar.range();
                (body, upper, lower, range, (upper + lower) / 2.0)
            } else {
                let trail_start = i.saturating_sub(candle_period);
                let trail_slice = &bars[trail_start..i]; // exclude bar i
                let trail_count = trail_slice.len() as f64;

                let (sum_body, sum_upper, sum_lower, sum_range, sum_shadow) = trail_slice
                    .iter()
                    .fold((0.0, 0.0, 0.0, 0.0, 0.0), |(b, u, l, r, s), bar| {
                        let upper = bar.upper_shadow();
                        let lower = bar.lower_shadow();
                        (
                            b + bar.body(),
                            u + upper,
                            l + lower,
                            r + bar.range(),
                            s + upper + lower,
                        )
                    });

                (
                    sum_body / trail_count,
                    sum_upper / trail_count,
                    sum_lower / trail_count,
                    sum_range / trail_count,
                    sum_shadow / trail_count / 2.0,
                )
            };

            // Near/Far/Equal use Period=5
            let avg_range_5 = if i == 0 {
                bars[0].range()
            } else {
                let s5 = i.saturating_sub(5);
                let slice5 = &bars[s5..i];
                slice5.iter().map(|b| OHLCVExt::range(b)).sum::<f64>() / slice5.len() as f64
            };

            contexts.push(MarketContext {
                trend: self.compute_trend(bars, i),
                volatility: self.compute_volatility(bars, i),
                avg_volume: self.compute_avg_volume(bars, i),
                avg_body,
                avg_upper_shadow,
                avg_lower_shadow,
                avg_range,
                avg_shadow,
                avg_range_5,
            });
        }

        contexts
    }
}

impl DefaultContextProvider {
    fn compute_trend<T: OHLCV>(&self, bars: &[T], index: usize) -> Trend {
        let period = self.trend_period.get();
        if index < period {
            return Trend::Sideways;
        }

        let start = index.saturating_sub(period);
        let first_close = bars[start].close();
        let last_close = bars[index].close();

        if first_close <= f64::EPSILON {
            return Trend::Sideways;
        }

        let change = (last_close - first_close) / first_close;

        match change {
            c if c > 0.05 => Trend::StrongUp,
            c if c > 0.02 => Trend::WeakUp,
            c if c < -0.05 => Trend::StrongDown,
            c if c < -0.02 => Trend::WeakDown,
            _ => Trend::Sideways,
        }
    }

    fn compute_volatility<T: OHLCV>(&self, bars: &[T], index: usize) -> f64 {
        let period = self.trend_period.get();
        if index < period {
            return 0.0;
        }

        let start = index.saturating_sub(period);
        let slice = &bars[start..=index];
        let sum: f64 = slice.iter().map(|b| OHLCVExt::range(b)).sum();
        sum / slice.len() as f64
    }

    fn compute_avg_volume<T: OHLCV>(&self, bars: &[T], index: usize) -> f64 {
        let period = self.volume_period.get();
        if index < period {
            return bars[index].volume();
        }

        let start = index.saturating_sub(period);
        let sum: f64 = bars[start..=index].iter().map(|b| b.volume()).sum();
        sum / (index - start + 1) as f64
    }
}

// ============================================================
// PATTERN DETECTOR TRAITS
// ============================================================

/// Category of pattern by number of bars
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PatternCategory {
    SingleBar,
    TwoBar,
    ThreeBar,
    MultiBar,
}

/// Additional metadata about a pattern
#[derive(Debug, Clone)]
pub struct PatternMetadata {
    pub name: &'static str,
    pub description: &'static str,
    pub category: PatternCategory,
}

/// Generic pattern detector trait - for concrete types
pub trait PatternDetector: Send + Sync {
    fn id(&self) -> PatternId;
    fn min_bars(&self) -> usize;
    fn detect<T: OHLCV>(
        &self,
        bars: &[T],
        index: usize,
        ctx: &MarketContext,
    ) -> Option<PatternMatch>;

    fn validate_config(&self) -> Result<()> {
        Ok(())
    }

    fn metadata(&self) -> PatternMetadata {
        PatternMetadata {
            name: self.id().0,
            description: "",
            category: match self.min_bars() {
                1 => PatternCategory::SingleBar,
                2 => PatternCategory::TwoBar,
                3 => PatternCategory::ThreeBar,
                _ => PatternCategory::MultiBar,
            },
        }
    }
}

/// Object-safe pattern detector trait - for custom detectors
pub trait DynPatternDetector: Send + Sync {
    fn id(&self) -> PatternId;
    fn min_bars(&self) -> usize;
    fn detect(
        &self,
        bars: &[&dyn OHLCV],
        index: usize,
        ctx: &MarketContext,
    ) -> Option<PatternMatch>;
    fn validate_config(&self) -> Result<()>;
}

impl<D: PatternDetector> DynPatternDetector for D {
    fn id(&self) -> PatternId {
        PatternDetector::id(self)
    }

    fn min_bars(&self) -> usize {
        PatternDetector::min_bars(self)
    }

    fn detect(
        &self,
        bars: &[&dyn OHLCV],
        index: usize,
        ctx: &MarketContext,
    ) -> Option<PatternMatch> {
        PatternDetector::detect(self, bars, index, ctx)
    }

    fn validate_config(&self) -> Result<()> {
        PatternDetector::validate_config(self)
    }
}

// ============================================================
// BUILTIN DETECTORS - generated via macro
// ============================================================

use detectors::*;

/// Macro to generate BuiltinDetector enum without boilerplate
macro_rules! define_builtin_detectors {
    (
        $(
            $variant:ident($detector:ty)
        ),* $(,)?
    ) => {
        /// All builtin detectors - fast path via enum dispatch
        #[derive(Debug, Clone)]
        pub enum BuiltinDetector {
            $($variant($detector)),*
        }

        impl BuiltinDetector {
            #[inline]
            pub fn detect<T: OHLCV>(
                &self,
                bars: &[T],
                index: usize,
                ctx: &MarketContext,
            ) -> Option<PatternMatch> {
                match self {
                    $(Self::$variant(d) => PatternDetector::detect(d, bars, index, ctx)),*
                }
            }

            #[inline]
            pub fn id(&self) -> PatternId {
                match self {
                    $(Self::$variant(d) => PatternDetector::id(d)),*
                }
            }

            #[inline]
            pub fn min_bars(&self) -> usize {
                match self {
                    $(Self::$variant(d) => PatternDetector::min_bars(d)),*
                }
            }

            pub fn validate_config(&self) -> Result<()> {
                match self {
                    $(Self::$variant(d) => PatternDetector::validate_config(d)),*
                }
            }
        }
    };
}

// Apply macro - all 61 TA-Lib patterns
define_builtin_detectors! {
    // Single bar (17)
    Doji(DojiDetector),
    DragonflyDoji(DragonflyDojiDetector),
    GravestoneDoji(GravestoneDojiDetector),
    LongLeggedDoji(LongLeggedDojiDetector),
    RickshawMan(RickshawManDetector),
    Hammer(HammerDetector),
    HangingMan(HangingManDetector),
    InvertedHammer(InvertedHammerDetector),
    ShootingStar(ShootingStarDetector),
    Takuri(TakuriDetector),
    Marubozu(MarubozuDetector),
    ClosingMarubozu(ClosingMarubozuDetector),
    LongLine(LongLineDetector),
    ShortLine(ShortLineDetector),
    SpinningTop(SpinningTopDetector),
    HighWave(HighWaveDetector),
    BeltHold(BeltHoldDetector),

    // Two bar (17)
    Engulfing(EngulfingDetector),
    Harami(HaramiDetector),
    HaramiCross(HaramiCrossDetector),
    Piercing(PiercingDetector),
    DarkCloudCover(DarkCloudCoverDetector),
    DojiStar(DojiStarDetector),
    Counterattack(CounterattackDetector),
    InNeck(InNeckDetector),
    OnNeck(OnNeckDetector),
    Thrusting(ThrustingDetector),
    Kicking(KickingDetector),
    KickingByLength(KickingByLengthDetector),
    MatchingLow(MatchingLowDetector),
    HomingPigeon(HomingPigeonDetector),
    SeparatingLines(SeparatingLinesDetector),
    GapSideSideWhite(GapSideSideWhiteDetector),
    TweezerTop(TweezerTopDetector),
    TweezerBottom(TweezerBottomDetector),

    // Three bar (22)
    ThreeWhiteSoldiers(ThreeWhiteSoldiersDetector),
    ThreeBlackCrows(ThreeBlackCrowsDetector),
    ThreeInside(ThreeInsideDetector),
    ThreeOutside(ThreeOutsideDetector),
    ThreeLineStrike(ThreeLineStrikeDetector),
    ThreeStarsInSouth(ThreeStarsInSouthDetector),
    MorningStar(MorningStarDetector),
    EveningStar(EveningStarDetector),
    MorningDojiStar(MorningDojiStarDetector),
    EveningDojiStar(EveningDojiStarDetector),
    AbandonedBaby(AbandonedBabyDetector),
    TwoCrows(TwoCrowsDetector),
    UpsideGapTwoCrows(UpsideGapTwoCrowsDetector),
    IdenticalThreeCrows(IdenticalThreeCrowsDetector),
    AdvanceBlock(AdvanceBlockDetector),
    StalledPattern(StalledPatternDetector),
    StickSandwich(StickSandwichDetector),
    TasukiGap(TasukiGapDetector),
    Tristar(TristarDetector),
    Unique3River(Unique3RiverDetector),

    // Multi-bar (8)
    Breakaway(BreakawayDetector),
    ConcealingBabySwallow(ConcealingBabySwallowDetector),
    Hikkake(HikkakeDetector),
    HikkakeMod(HikkakeModDetector),
    LadderBottom(LadderBottomDetector),
    MatHold(MatHoldDetector),
    RiseFallThreeMethods(RiseFallThreeMethodsDetector),
    XSideGapThreeMethods(XSideGapThreeMethodsDetector),

    // Extended patterns (30+)
    // Price Lines
    PriceLines(PriceLinesDetector),

    // Windows (Gaps)
    FallingWindow(FallingWindowDetector),
    RisingWindow(RisingWindowDetector),
    GappingDownDoji(GappingDownDojiDetector),
    GappingUpDoji(GappingUpDojiDetector),

    // Reversal
    AboveTheStomach(AboveTheStomachDetector),
    BelowTheStomach(BelowTheStomachDetector),
    CollapsingDojiStar(CollapsingDojiStarDetector),
    Deliberation(DeliberationDetector),
    LastEngulfingBottom(LastEngulfingBottomDetector),
    LastEngulfingTop(LastEngulfingTopDetector),
    TwoBlackGapping(TwoBlackGappingDetector),
    MeetingLinesBearish(MeetingLinesBearishDetector),
    MeetingLinesBullish(MeetingLinesBullishDetector),

    // Doji variants
    NorthernDoji(NorthernDojiDetector),
    SouthernDoji(SouthernDojiDetector),

    // Marubozu variants
    BlackMarubozu(BlackMarubozuDetector),
    WhiteMarubozu(WhiteMarubozuDetector),
    OpeningBlackMarubozu(OpeningBlackMarubozuDetector),
    OpeningWhiteMarubozu(OpeningWhiteMarubozuDetector),

    // Basic candles
    BlackCandle(BlackCandleDetector),
    WhiteCandle(WhiteCandleDetector),
    ShortBlack(ShortBlackDetector),
    ShortWhite(ShortWhiteDetector),
    LongBlackDay(LongBlackDayDetector),
    LongWhiteDay(LongWhiteDayDetector),
    BlackSpinningTop(BlackSpinningTopDetector),
    WhiteSpinningTop(WhiteSpinningTopDetector),

    // Shooting Star variant
    ShootingStar2Lines(ShootingStar2LinesDetector),

    // Gap Three Methods
    DownsideGapThreeMethods(DownsideGapThreeMethodsDetector),
    UpsideGapThreeMethods(UpsideGapThreeMethodsDetector),

    // Tasuki Gap variants
    DownsideTasukiGap(DownsideTasukiGapDetector),
    UpsideTasukiGap(UpsideTasukiGapDetector),
}

// ============================================================
// PATTERN ENGINE
// ============================================================

/// Engine configuration
#[derive(Debug, Clone, Default)]
pub struct EngineConfig {
    pub min_strength: Option<f64>,
    pub validate_data: bool,
    pub pattern_filter: Option<Vec<PatternId>>,
}

/// Main pattern detection engine
pub struct PatternEngine<C: ContextProvider = DefaultContextProvider> {
    builtin: Vec<BuiltinDetector>,
    custom: Vec<Box<dyn DynPatternDetector>>,
    context_provider: C,
    config: EngineConfig,
}

impl<C: ContextProvider> PatternEngine<C> {
    pub fn new(context_provider: C) -> Self {
        Self {
            builtin: Vec::new(),
            custom: Vec::new(),
            context_provider,
            config: EngineConfig::default(),
        }
    }

    // ===========================================
    // LOW-LEVEL: Primitives
    // ===========================================

    /// Precompute contexts for all bars.
    /// User stores and reuses the result.
    #[inline]
    pub fn compute_contexts<T: OHLCV>(&self, bars: &[T]) -> Vec<MarketContext> {
        self.context_provider.compute_all(bars)
    }

    /// Compute context for a single bar.
    /// For incremental/realtime scenarios.
    #[inline]
    pub fn compute_context_at<T: OHLCV>(&self, bars: &[T], index: usize) -> MarketContext {
        let contexts = self.context_provider.compute_all(bars);
        contexts.get(index).copied().unwrap_or_default()
    }

    // ===========================================
    // MID-LEVEL: Single-bar / Range
    // ===========================================

    /// Detect patterns at a single bar index.
    pub fn scan_at<T: OHLCV>(
        &self,
        bars: &[T],
        index: usize,
        ctx: &MarketContext,
    ) -> Vec<PatternMatch> {
        if self.custom.is_empty() {
            self.scan_at_internal(bars, &[], index, ctx)
        } else {
            let bar_refs: Vec<&dyn OHLCV> = bars.iter().map(|b| b as &dyn OHLCV).collect();
            self.scan_at_internal(bars, &bar_refs, index, ctx)
        }
    }

    /// Detect patterns in a range of bars.
    pub fn scan_range<T: OHLCV>(
        &self,
        bars: &[T],
        range: std::ops::Range<usize>,
        contexts: &[MarketContext],
    ) -> Vec<PatternMatch> {
        let mut results = Vec::new();

        if self.custom.is_empty() {
            for i in range {
                if let Some(ctx) = contexts.get(i) {
                    results.extend(self.scan_at_internal(bars, &[], i, ctx));
                }
            }
        } else {
            let bar_refs: Vec<&dyn OHLCV> = bars.iter().map(|b| b as &dyn OHLCV).collect();
            for i in range {
                if let Some(ctx) = contexts.get(i) {
                    results.extend(self.scan_at_internal(bars, &bar_refs, i, ctx));
                }
            }
        }

        results
    }

    // ===========================================
    // HIGH-LEVEL: Batch processing
    // ===========================================

    /// Scan all bars and return flat list of patterns.
    pub fn scan<T: OHLCV>(&self, bars: &[T]) -> Result<Vec<PatternMatch>> {
        if self.config.validate_data {
            self.validate_bars(bars)?;
        }

        let contexts = self.compute_contexts(bars);
        Ok(self.scan_range(bars, 0..bars.len(), &contexts))
    }

    /// Scan and return patterns grouped by bar index.
    pub fn scan_grouped<T: OHLCV>(&self, bars: &[T]) -> Result<Vec<Vec<PatternMatch>>> {
        if self.config.validate_data {
            self.validate_bars(bars)?;
        }

        let contexts = self.compute_contexts(bars);
        let mut grouped = vec![Vec::new(); bars.len()];

        if self.custom.is_empty() {
            for i in 0..bars.len() {
                grouped[i] = self.scan_at_internal(bars, &[], i, &contexts[i]);
            }
        } else {
            let bar_refs: Vec<&dyn OHLCV> = bars.iter().map(|b| b as &dyn OHLCV).collect();
            for i in 0..bars.len() {
                grouped[i] = self.scan_at_internal(bars, &bar_refs, i, &contexts[i]);
            }
        }

        Ok(grouped)
    }

    /// Create an iterator over bars with their patterns.
    pub fn iter<'a, T: OHLCV>(&'a self, bars: &'a [T]) -> PatternIterator<'a, T, C> {
        PatternIterator::new(self, bars)
    }

    // ===========================================
    // Internal helpers
    // ===========================================

    fn scan_at_internal<T: OHLCV>(
        &self,
        bars: &[T],
        bar_refs: &[&dyn OHLCV],
        index: usize,
        ctx: &MarketContext,
    ) -> Vec<PatternMatch> {
        let mut results = Vec::new();

        // Fast path: builtin detectors (enum dispatch, no vtable)
        for detector in &self.builtin {
            if index + 1 >= detector.min_bars() {
                if let Some(m) = detector.detect(bars, index, ctx) {
                    if self.should_include(&m) {
                        results.push(m);
                    }
                }
            }
        }

        // Slow path: custom detectors (vtable)
        if !self.custom.is_empty() && !bar_refs.is_empty() {
            for detector in &self.custom {
                if index + 1 >= detector.min_bars() {
                    if let Some(m) = detector.detect(bar_refs, index, ctx) {
                        if self.should_include(&m) {
                            results.push(m);
                        }
                    }
                }
            }
        }

        results
    }

    fn should_include(&self, m: &PatternMatch) -> bool {
        if let Some(min) = self.config.min_strength {
            if m.strength < min {
                return false;
            }
        }
        if let Some(ref filter) = self.config.pattern_filter {
            if !filter.contains(&m.pattern_id) {
                return false;
            }
        }
        true
    }

    fn validate_bars<T: OHLCV>(&self, bars: &[T]) -> Result<()> {
        for (i, bar) in bars.iter().enumerate() {
            bar.validate().map_err(|e| match e {
                PatternError::InvalidOHLCV { reason, .. } => {
                    PatternError::InvalidOHLCV { index: i, reason }
                }
                other => other,
            })?;
        }
        Ok(())
    }

    fn validate(&self) -> Result<()> {
        for d in &self.builtin {
            d.validate_config()?;
        }
        for d in &self.custom {
            d.validate_config()?;
        }
        Ok(())
    }
}

// ============================================================
// PATTERN ITERATOR
// ============================================================

/// Patterns found at a specific bar
#[derive(Debug, Clone)]
pub struct BarPatterns {
    pub index: usize,
    pub patterns: Vec<PatternMatch>,
}

/// Iterator over bars with their patterns
pub struct PatternIterator<'a, T: OHLCV, C: ContextProvider> {
    engine: &'a PatternEngine<C>,
    bars: &'a [T],
    bar_refs: Vec<&'a dyn OHLCV>,
    contexts: Vec<MarketContext>,
    current: usize,
}

impl<'a, T: OHLCV, C: ContextProvider> PatternIterator<'a, T, C> {
    fn new(engine: &'a PatternEngine<C>, bars: &'a [T]) -> Self {
        let bar_refs = if engine.custom.is_empty() {
            Vec::new()
        } else {
            bars.iter().map(|b| b as &dyn OHLCV).collect()
        };
        let contexts = engine.compute_contexts(bars);

        Self {
            engine,
            bars,
            bar_refs,
            contexts,
            current: 0,
        }
    }
}

impl<'a, T: OHLCV, C: ContextProvider> Iterator for PatternIterator<'a, T, C> {
    type Item = BarPatterns;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current >= self.bars.len() {
            return None;
        }

        let index = self.current;
        let ctx = &self.contexts[index];
        let patterns = self
            .engine
            .scan_at_internal(self.bars, &self.bar_refs, index, ctx);

        self.current += 1;

        Some(BarPatterns { index, patterns })
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = self.bars.len().saturating_sub(self.current);
        (remaining, Some(remaining))
    }
}

impl<'a, T: OHLCV, C: ContextProvider> ExactSizeIterator for PatternIterator<'a, T, C> {}

// ============================================================
// BUILDER
// ============================================================

/// Builder for creating PatternEngine instances
pub struct EngineBuilder<C: ContextProvider = DefaultContextProvider> {
    context_provider: C,
    builtin: Vec<BuiltinDetector>,
    custom: Vec<Box<dyn DynPatternDetector>>,
    config: EngineConfig,
}

impl Default for EngineBuilder<DefaultContextProvider> {
    fn default() -> Self {
        Self::new()
    }
}

impl EngineBuilder<DefaultContextProvider> {
    pub fn new() -> Self {
        Self {
            context_provider: DefaultContextProvider::default(),
            builtin: Vec::new(),
            custom: Vec::new(),
            config: EngineConfig::default(),
        }
    }
}

/// Generate an array of `BuiltinDetector` variants using `Default::default()` for each inner type.
macro_rules! builtin_defaults {
  ($($variant:ident),* $(,)?) => {
    [$(BuiltinDetector::$variant(Default::default())),*]
  };
}

impl<C: ContextProvider> EngineBuilder<C> {
    /// Change context provider
    pub fn context_provider<C2: ContextProvider>(self, provider: C2) -> EngineBuilder<C2> {
        EngineBuilder {
            context_provider: provider,
            builtin: self.builtin,
            custom: self.custom,
            config: self.config,
        }
    }

    /// Add all builtin patterns with default configurations
    pub fn with_all_defaults(self) -> Self {
        self.with_single_bar_defaults()
            .with_two_bar_defaults()
            .with_three_bar_defaults()
            .with_multi_bar_defaults()
            .with_extended_defaults()
    }

    /// Add only extended patterns with defaults
    pub fn with_extended_defaults(mut self) -> Self {
        self.builtin.extend(builtin_defaults![
            PriceLines,
            FallingWindow,
            RisingWindow,
            GappingDownDoji,
            GappingUpDoji,
            AboveTheStomach,
            BelowTheStomach,
            CollapsingDojiStar,
            Deliberation,
            LastEngulfingBottom,
            LastEngulfingTop,
            TwoBlackGapping,
            MeetingLinesBearish,
            MeetingLinesBullish,
            NorthernDoji,
            SouthernDoji,
            BlackMarubozu,
            WhiteMarubozu,
            OpeningBlackMarubozu,
            OpeningWhiteMarubozu,
            BlackCandle,
            WhiteCandle,
            ShortBlack,
            ShortWhite,
            LongBlackDay,
            LongWhiteDay,
            BlackSpinningTop,
            WhiteSpinningTop,
            ShootingStar2Lines,
            DownsideGapThreeMethods,
            UpsideGapThreeMethods,
            DownsideTasukiGap,
            UpsideTasukiGap,
        ]);
        self
    }

    /// Add only single-bar patterns with defaults (17)
    pub fn with_single_bar_defaults(mut self) -> Self {
        self.builtin.extend(builtin_defaults![
            Doji,
            DragonflyDoji,
            GravestoneDoji,
            LongLeggedDoji,
            RickshawMan,
            Hammer,
            HangingMan,
            InvertedHammer,
            ShootingStar,
            Takuri,
            Marubozu,
            ClosingMarubozu,
            LongLine,
            ShortLine,
            SpinningTop,
            HighWave,
            BeltHold,
        ]);
        self
    }

    /// Add two-bar patterns with defaults (18)
    pub fn with_two_bar_defaults(mut self) -> Self {
        self.builtin.extend(builtin_defaults![
            Engulfing,
            Harami,
            HaramiCross,
            Piercing,
            DarkCloudCover,
            DojiStar,
            Counterattack,
            InNeck,
            OnNeck,
            Thrusting,
            Kicking,
            KickingByLength,
            MatchingLow,
            HomingPigeon,
            SeparatingLines,
            GapSideSideWhite,
            TweezerTop,
            TweezerBottom,
        ]);
        self
    }

    /// Add three-bar patterns with defaults (20)
    pub fn with_three_bar_defaults(mut self) -> Self {
        self.builtin.extend(builtin_defaults![
            ThreeWhiteSoldiers,
            ThreeBlackCrows,
            ThreeInside,
            ThreeOutside,
            ThreeLineStrike,
            ThreeStarsInSouth,
            MorningStar,
            EveningStar,
            MorningDojiStar,
            EveningDojiStar,
            AbandonedBaby,
            TwoCrows,
            UpsideGapTwoCrows,
            IdenticalThreeCrows,
            AdvanceBlock,
            StalledPattern,
            StickSandwich,
            TasukiGap,
            Tristar,
            Unique3River,
        ]);
        self
    }

    /// Add multi-bar patterns with defaults (8)
    pub fn with_multi_bar_defaults(mut self) -> Self {
        self.builtin.extend(builtin_defaults![
            Breakaway,
            ConcealingBabySwallow,
            Hikkake,
            HikkakeMod,
            LadderBottom,
            MatHold,
            RiseFallThreeMethods,
            XSideGapThreeMethods,
        ]);
        self
    }

    /// Add a builtin detector
    #[allow(clippy::should_implement_trait)]
    pub fn add(mut self, detector: BuiltinDetector) -> Self {
        self.builtin.push(detector);
        self
    }

    /// Add with config validation
    pub fn add_checked(mut self, detector: BuiltinDetector) -> Result<Self> {
        detector.validate_config()?;
        self.builtin.push(detector);
        Ok(self)
    }

    /// Add a custom detector (slow path)
    pub fn add_custom<D: DynPatternDetector + 'static>(mut self, detector: D) -> Self {
        self.custom.push(Box::new(detector));
        self
    }

    /// Set minimum strength filter
    pub fn min_strength(mut self, strength: f64) -> Self {
        self.config.min_strength = Some(strength);
        self
    }

    /// Enable/disable data validation
    pub fn validate_data(mut self, enable: bool) -> Self {
        self.config.validate_data = enable;
        self
    }

    /// Filter to specific patterns only
    pub fn only_patterns(mut self, ids: impl IntoIterator<Item = PatternId>) -> Self {
        self.config.pattern_filter = Some(ids.into_iter().collect());
        self
    }

    /// Build the engine
    pub fn build(self) -> Result<PatternEngine<C>> {
        let engine = PatternEngine {
            builtin: self.builtin,
            custom: self.custom,
            context_provider: self.context_provider,
            config: self.config,
        };
        engine.validate()?;
        Ok(engine)
    }
}

// ============================================================
// PARALLEL SCANNING
// ============================================================

use rayon::prelude::*;

/// Result of scanning a single instrument
#[derive(Debug)]
pub struct ScanResult {
    pub symbol: String,
    pub patterns: Vec<PatternMatch>,
}

/// Error from scanning a single instrument
#[derive(Debug)]
pub struct ScanError {
    pub symbol: String,
    pub error: PatternError,
}

/// Parallel scanning of multiple instruments
pub fn scan_parallel<'a, T, I, C>(
    engine: &PatternEngine<C>,
    instruments: I,
) -> (Vec<ScanResult>, Vec<ScanError>)
where
    T: OHLCV + Sync + 'a,
    I: IntoParallelIterator<Item = (&'a str, &'a [T])>,
    C: ContextProvider + Sync,
{
    let results: Vec<_> = instruments
        .into_par_iter()
        .map(|(symbol, bars)| {
            engine
                .scan(bars)
                .map(|patterns| ScanResult {
                    symbol: symbol.to_string(),
                    patterns,
                })
                .map_err(|error| ScanError {
                    symbol: symbol.to_string(),
                    error,
                })
        })
        .collect();

    let mut successes = Vec::new();
    let mut errors = Vec::new();

    for result in results {
        match result {
            Ok(r) => successes.push(r),
            Err(e) => errors.push(e),
        }
    }

    (successes, errors)
}

// ============================================================
// TYPE ALIASES
// ============================================================

/// Default engine with DefaultContextProvider
pub type DefaultEngine = PatternEngine<DefaultContextProvider>;

// ============================================================
// TESTS
// ============================================================

#[cfg(test)]
mod tests {
    use super::*;

    /// Test OHLCV bar
    #[derive(Debug, Clone)]
    struct Bar {
        o: f64,
        h: f64,
        l: f64,
        c: f64,
        v: f64,
    }

    impl Bar {
        fn new(o: f64, h: f64, l: f64, c: f64) -> Self {
            Self {
                o,
                h,
                l,
                c,
                v: 1000.0,
            }
        }
    }

    impl OHLCV for Bar {
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
            self.v
        }
    }

    fn make_downtrend_bars() -> Vec<Bar> {
        (0..20)
            .map(|i| {
                let base = 100.0 - i as f64 * 2.0;
                Bar::new(base, base + 1.0, base - 1.0, base - 0.5)
            })
            .collect()
    }

    fn make_uptrend_bars() -> Vec<Bar> {
        (0..20)
            .map(|i| {
                let base = 100.0 + i as f64 * 2.0;
                Bar::new(base, base + 1.0, base - 1.0, base + 0.5)
            })
            .collect()
    }

    #[test]
    fn test_ratio_validation() {
        assert!(Ratio::new(0.0).is_ok());
        assert!(Ratio::new(1.0).is_ok());
        assert!(Ratio::new(0.5).is_ok());
        assert!(Ratio::new(-0.1).is_err());
        assert!(Ratio::new(1.1).is_err());
        assert!(Ratio::new(f64::NAN).is_err());
        assert!(Ratio::new(f64::INFINITY).is_err());
    }

    #[test]
    fn test_period_validation() {
        assert!(Period::new(1).is_ok());
        assert!(Period::new(100).is_ok());
        assert!(Period::new(0).is_err());
    }

    #[test]
    fn test_ohlcv_ext() {
        let bar = Bar::new(100.0, 110.0, 90.0, 105.0);
        assert_eq!(bar.body(), 5.0);
        assert_eq!(bar.range(), 20.0);
        assert!(bar.is_bullish());
        assert!(!bar.is_bearish());
        assert!((bar.body_ratio().unwrap() - 0.25).abs() < 0.001);
    }

    #[test]
    fn test_engine_builder() {
        let engine = EngineBuilder::new().with_all_defaults().build();
        assert!(engine.is_ok());
    }

    #[test]
    fn test_empty_scan() {
        let engine = EngineBuilder::new().with_all_defaults().build().unwrap();
        let bars: Vec<Bar> = vec![];
        let patterns = engine.scan(&bars).unwrap();
        assert!(patterns.is_empty());
    }

    #[test]
    fn test_doji_detection() {
        let engine = EngineBuilder::new()
            .add(BuiltinDetector::Doji(DojiDetector::with_defaults()))
            .build()
            .unwrap();

        let bars = vec![
            Bar::new(100.0, 110.0, 90.0, 100.5), // Doji
        ];

        let patterns = engine.scan(&bars).unwrap();
        assert!(!patterns.is_empty());
        assert_eq!(patterns[0].pattern_id, PatternId("CDL_DOJI"));
    }

    #[test]
    fn test_scan_grouped() {
        let engine = EngineBuilder::new()
            .add(BuiltinDetector::Doji(DojiDetector::with_defaults()))
            .add(BuiltinDetector::Marubozu(MarubozuDetector::with_defaults()))
            .build()
            .unwrap();

        let bars = vec![
            Bar::new(100.0, 110.0, 90.0, 100.5),  // Doji
            Bar::new(100.0, 105.0, 100.0, 105.0), // Marubozu (close = high)
            Bar::new(100.0, 100.0, 95.0, 95.0),   // Bearish Marubozu (close = low)
        ];

        let grouped = engine.scan_grouped(&bars).unwrap();
        assert_eq!(grouped.len(), bars.len());
    }

    #[test]
    fn test_iterator() {
        let engine = EngineBuilder::new()
            .add(BuiltinDetector::Doji(DojiDetector::with_defaults()))
            .build()
            .unwrap();

        let bars = vec![
            Bar::new(100.0, 110.0, 90.0, 100.5),
            Bar::new(100.0, 110.0, 90.0, 100.5),
        ];

        let results: Vec<_> = engine.iter(&bars).collect();
        assert_eq!(results.len(), bars.len());
    }

    #[test]
    fn test_iterator_exact_size() {
        let engine = EngineBuilder::new()
            .add(BuiltinDetector::Doji(DojiDetector::with_defaults()))
            .build()
            .unwrap();

        let bars = vec![
            Bar::new(100.0, 110.0, 90.0, 100.5),
            Bar::new(100.0, 110.0, 90.0, 100.5),
            Bar::new(100.0, 110.0, 90.0, 100.5),
        ];

        let iter = engine.iter(&bars);
        assert_eq!(iter.len(), 3);
    }

    #[test]
    fn test_min_strength_filter() {
        let engine = EngineBuilder::new()
            .add(BuiltinDetector::Doji(DojiDetector::with_defaults()))
            .min_strength(0.99)
            .build()
            .unwrap();

        let bars = vec![Bar::new(100.0, 110.0, 90.0, 100.5)];
        let patterns = engine.scan(&bars).unwrap();
        assert!(patterns.is_empty());
    }

    #[test]
    fn test_pattern_filter() {
        let engine = EngineBuilder::new()
            .add(BuiltinDetector::Doji(DojiDetector::with_defaults()))
            .add(BuiltinDetector::Marubozu(MarubozuDetector::with_defaults()))
            .only_patterns([PatternId("CDL_MARUBOZU")])
            .build()
            .unwrap();

        let bars = vec![Bar::new(100.0, 110.0, 90.0, 100.5)]; // Doji pattern
        let patterns = engine.scan(&bars).unwrap();
        assert!(patterns.is_empty()); // Doji filtered out
    }

    #[test]
    fn test_single_bar_defaults() {
        let engine = EngineBuilder::new()
            .with_single_bar_defaults()
            .build()
            .unwrap();
        assert!(engine.builtin.len() == 17);
    }

    #[test]
    fn test_two_bar_defaults() {
        let engine = EngineBuilder::new()
            .with_two_bar_defaults()
            .build()
            .unwrap();
        assert!(engine.builtin.len() == 18); // 17 + TweezerTop + TweezerBottom but 2 are in three_bar
    }

    #[test]
    fn test_three_bar_defaults() {
        let engine = EngineBuilder::new()
            .with_three_bar_defaults()
            .build()
            .unwrap();
        assert!(engine.builtin.len() == 20);
    }

    #[test]
    fn test_all_defaults_count() {
        let engine = EngineBuilder::new().with_all_defaults().build().unwrap();
        // Total: 17 + 18 + 20 + 8 = 63 (some overlap due to TweezerTop/Bottom in different counts)
        assert!(engine.builtin.len() >= 60);
    }

    #[test]
    fn test_hammer_detection() {
        let mut bars = make_downtrend_bars();
        // Add a hammer bar at the end
        bars.push(Bar::new(60.0, 60.1, 55.0, 60.05)); // Small body at top, long lower shadow

        let engine = EngineBuilder::new()
            .add(BuiltinDetector::Hammer(HammerDetector::with_defaults()))
            .build()
            .unwrap();

        let patterns = engine.scan(&bars).unwrap();
        let hammer = patterns
            .iter()
            .find(|p| p.pattern_id == PatternId("CDL_HAMMER"));
        assert!(hammer.is_some());
    }

    #[test]
    fn test_engulfing_detection() {
        let mut bars = make_downtrend_bars();
        // Add bearish then bullish engulfing pattern
        bars.push(Bar::new(60.0, 61.0, 59.0, 59.5)); // Bearish
        bars.push(Bar::new(59.0, 62.0, 58.0, 61.5)); // Bullish engulfing

        let engine = EngineBuilder::new()
            .add(BuiltinDetector::Engulfing(
                EngulfingDetector::with_defaults(),
            ))
            .build()
            .unwrap();

        let patterns = engine.scan(&bars).unwrap();
        let engulfing = patterns
            .iter()
            .find(|p| p.pattern_id == PatternId("CDL_ENGULFING"));
        assert!(engulfing.is_some());
    }

    #[test]
    fn test_parallel_scan() {
        let engine = EngineBuilder::new().with_all_defaults().build().unwrap();

        let bars1 = make_downtrend_bars();
        let bars2 = make_uptrend_bars();

        let instruments: Vec<(&str, &[Bar])> = vec![("AAPL", &bars1), ("GOOGL", &bars2)];

        let (results, errors) = scan_parallel(&engine, instruments);
        assert_eq!(results.len(), 2);
        assert!(errors.is_empty());
    }

    #[test]
    fn test_compute_contexts() {
        let engine = EngineBuilder::new().with_all_defaults().build().unwrap();

        let bars = make_downtrend_bars();
        let contexts = engine.compute_contexts(&bars);
        assert_eq!(contexts.len(), bars.len());
    }

    #[test]
    fn test_scan_at() {
        let engine = EngineBuilder::new()
            .add(BuiltinDetector::Doji(DojiDetector::with_defaults()))
            .build()
            .unwrap();

        let bars = vec![
            Bar::new(100.0, 110.0, 90.0, 100.5), // Doji
        ];

        let ctx = engine.compute_context_at(&bars, 0);
        let patterns = engine.scan_at(&bars, 0, &ctx);
        assert!(!patterns.is_empty());
    }

    #[test]
    fn test_scan_range() {
        let engine = EngineBuilder::new()
            .add(BuiltinDetector::Doji(DojiDetector::with_defaults()))
            .build()
            .unwrap();

        let bars: Vec<Bar> = (0..10)
            .map(|_| Bar::new(100.0, 110.0, 90.0, 100.5))
            .collect();
        let contexts = engine.compute_contexts(&bars);
        let patterns = engine.scan_range(&bars, 2..8, &contexts);
        assert!(!patterns.is_empty());
    }
}
