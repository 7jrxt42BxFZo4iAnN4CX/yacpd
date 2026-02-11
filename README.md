# YACPD - Yet Another Candlestick Pattern Detector

[![CI](https://github.com/7jrxt42BxFZo4iAnN4CX/yacpd/actions/workflows/ci.yml/badge.svg)](https://github.com/7jrxt42BxFZo4iAnN4CX/yacpd/actions/workflows/ci.yml)
[![crates.io](https://img.shields.io/crates/v/yacpd.svg)](https://crates.io/crates/yacpd)
[![docs.rs](https://docs.rs/yacpd/badge.svg)](https://docs.rs/yacpd)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

High-performance candlestick pattern detection library for Rust.

## Features

- **97 builtin patterns**: 64 TA-Lib compatible + 33 extended patterns
- **High performance**: Fast path with enum dispatch, slow path for custom detectors
- **Multi-level API**: From low-level primitives to high-level batch processing
- **Parallel scanning**: Rayon-based multi-instrument scanning
- **Configurable**: Per-pattern configuration, strength filtering, trend requirements
- **Extensible**: Add custom detectors with full type safety

## Quick Start

```rust
use yacpd::prelude::*;

// Define your OHLCV data
struct Bar { o: f64, h: f64, l: f64, c: f64, v: f64 }

impl OHLCV for Bar {
    fn open(&self) -> f64 { self.o }
    fn high(&self) -> f64 { self.h }
    fn low(&self) -> f64 { self.l }
    fn close(&self) -> f64 { self.c }
    fn volume(&self) -> f64 { self.v }
}

// Create engine with default detectors
let engine = EngineBuilder::new()
    .with_all_defaults()
    .build()?;

// Scan your data
let patterns = engine.scan(&bars)?;

for p in &patterns {
    println!("{:?} at index {} (strength: {:.2})",
        p.pattern_id, p.end_index, p.strength);
}
```

## Supported Patterns

### Single-Bar (17 TA-Lib)
| Pattern | ID | Direction | Description |
|---------|-----|-----------|-------------|
| Doji | `CDL_DOJI` | Neutral | Open ≈ close, indecision |
| Dragonfly Doji | `CDL_DRAGONFLY_DOJI` | Bullish | Doji with long lower shadow |
| Gravestone Doji | `CDL_GRAVESTONE_DOJI` | Bearish | Doji with long upper shadow |
| Long-Legged Doji | `CDL_LONGLEGGED_DOJI` | Neutral | Doji with long shadows both sides |
| Rickshaw Man | `CDL_RICKSHAWMAN` | Neutral | Long-legged doji with equal shadows |
| Hammer | `CDL_HAMMER` | Bullish | Small body at top, long lower shadow |
| Hanging Man | `CDL_HANGINGMAN` | Bearish | Same as hammer but in uptrend |
| Inverted Hammer | `CDL_INVERTEDHAMMER` | Bullish | Small body at bottom, long upper shadow |
| Shooting Star | `CDL_SHOOTINGSTAR` | Bearish | Same as inverted hammer but in uptrend |
| Takuri | `CDL_TAKURI` | Bullish | Dragonfly doji with very long lower shadow |
| Marubozu | `CDL_MARUBOZU` | Both | No shadows, strong momentum |
| Closing Marubozu | `CDL_CLOSINGMARUBOZU` | Both | No shadow on close side |
| Long Line | `CDL_LONGLINE` | Both | Large body, small shadows |
| Short Line | `CDL_SHORTLINE` | Both | Small body |
| Spinning Top | `CDL_SPINNINGTOP` | Neutral | Small body, shadows both sides |
| High Wave | `CDL_HIGHWAVE` | Neutral | Small body, very long shadows |
| Belt Hold | `CDL_BELTHOLD` | Both | Opening marubozu |

### Two-Bar (18 TA-Lib)
| Pattern | ID | Direction | Description |
|---------|-----|-----------|-------------|
| Engulfing | `CDL_ENGULFING` | Both | Second candle engulfs first |
| Harami | `CDL_HARAMI` | Both | Small body inside previous large body |
| Harami Cross | `CDL_HARAMICROSS` | Both | Harami with doji |
| Piercing | `CDL_PIERCING` | Bullish | Opens below, closes above midpoint |
| Dark Cloud Cover | `CDL_DARKCLOUDCOVER` | Bearish | Opens above, closes below midpoint |
| Doji Star | `CDL_DOJISTAR` | Both | Long candle followed by gapped doji |
| Counterattack | `CDL_COUNTERATTACK` | Both | Opposite colors, same close |
| In-Neck | `CDL_INNECK` | Bearish | Continuation, closes near previous close |
| On-Neck | `CDL_ONNECK` | Bearish | Continuation, closes at previous low |
| Thrusting | `CDL_THRUSTING` | Bearish | Continuation, closes below midpoint |
| Kicking | `CDL_KICKING` | Both | Two marubozu with gap |
| Kicking by Length | `CDL_KICKINGBYLENGTH` | Both | Kicking determined by longer candle |
| Matching Low | `CDL_MATCHINGLOW` | Bullish | Two bearish candles with same close |
| Homing Pigeon | `CDL_HOMINGPIGEON` | Bullish | Two bearish, second inside first |
| Separating Lines | `CDL_SEPARATINGLINES` | Both | Opposite colors, same open |
| Gap Side White | `CDL_GAPSIDESIDEWHITE` | Both | Gap followed by two similar candles |
| Tweezer Top | `CDL_TWEEZER_TOP` | Bearish | Two candles with similar highs |
| Tweezer Bottom | `CDL_TWEEZER_BOTTOM` | Bullish | Two candles with similar lows |

### Three-Bar (20 TA-Lib)
| Pattern | ID | Direction | Description |
|---------|-----|-----------|-------------|
| Three White Soldiers | `CDL_3WHITESOLDIERS` | Bullish | Three consecutive bullish candles |
| Three Black Crows | `CDL_3BLACKCROWS` | Bearish | Three consecutive bearish candles |
| Three Inside | `CDL_3INSIDE` | Both | Harami + confirmation |
| Three Outside | `CDL_3OUTSIDE` | Both | Engulfing + confirmation |
| Three Line Strike | `CDL_3LINESTRIKE` | Both | Three + opposite engulfing fourth |
| Three Stars in South | `CDL_3STARSINSOUTH` | Bullish | Descending stars pattern |
| Morning Star | `CDL_MORNINGSTAR` | Bullish | Bearish + small star + bullish |
| Evening Star | `CDL_EVENINGSTAR` | Bearish | Bullish + small star + bearish |
| Morning Doji Star | `CDL_MORNINGDOJISTAR` | Bullish | Morning star with doji |
| Evening Doji Star | `CDL_EVENINGDOJISTAR` | Bearish | Evening star with doji |
| Abandoned Baby | `CDL_ABANDONEDBABY` | Both | Star with gaps both sides |
| Two Crows | `CDL_2CROWS` | Bearish | Bullish + two gapped bearish |
| Upside Gap Two Crows | `CDL_UPSIDEGAP2CROWS` | Bearish | Gap maintained in crows |
| Identical Three Crows | `CDL_IDENTICAL3CROWS` | Bearish | Three crows opening at previous close |
| Advance Block | `CDL_ADVANCEBLOCK` | Bearish | Weakening bullish pattern |
| Stalled Pattern | `CDL_STALLEDPATTERN` | Bearish | Deliberation pattern |
| Stick Sandwich | `CDL_STICKSANDWICH` | Bullish | Support at same level |
| Tasuki Gap | `CDL_TASUKIGAP` | Both | Gap + partial retracement |
| Tristar | `CDL_TRISTAR` | Both | Three doji pattern |
| Unique 3 River | `CDL_UNIQUE3RIVER` | Bullish | Rare bullish reversal |

### Multi-Bar (8 TA-Lib)
| Pattern | ID | Bars | Description |
|---------|-----|------|-------------|
| Breakaway | `CDL_BREAKAWAY` | 5 | Gap + continuation + reversal |
| Concealing Baby Swallow | `CDL_CONCEALBABYSWALL` | 4 | Two marubozu + concealed doji |
| Hikkake | `CDL_HIKKAKE` | 5 | Inside bar fake breakout |
| Hikkake Modified | `CDL_HIKKAKEMOD` | 5 | Tighter hikkake variant |
| Ladder Bottom | `CDL_LADDERBOTTOM` | 5 | Descending + reversal |
| Mat Hold | `CDL_MATHOLD` | 5 | Gap + retracement + continuation |
| Rising/Falling Three | `CDL_RISEFALL3METHODS` | 5 | Continuation pattern |
| X-Side Gap Three | `CDL_XSIDEGAP3METHODS` | 4 | Gap side-by-side + fill |

---

### Extended Patterns (33)

#### Price Lines
| Pattern | ID | Parameters | Description |
|---------|-----|------------|-------------|
| Price Lines | `PRICE_LINES` | `count: usize` (8) | N consecutive candles of same direction |

```rust
// Create specific count variants
let detector = PriceLinesDetector::eight();   // 8 lines
let detector = PriceLinesDetector::ten();     // 10 lines
let detector = PriceLinesDetector::twelve();  // 12 lines
let detector = PriceLinesDetector::thirteen(); // 13 lines
```

#### Windows (Gaps)
| Pattern | ID | Parameters | Description |
|---------|-----|------------|-------------|
| Falling Window | `FALLING_WINDOW` | — | Gap down (curr.high < prev.low) |
| Rising Window | `RISING_WINDOW` | — | Gap up (curr.low > prev.high) |
| Gapping Down Doji | `GAPPING_DOWN_DOJI` | `body_pct` (0.1) | Doji with gap down |
| Gapping Up Doji | `GAPPING_UP_DOJI` | `body_pct` (0.1) | Doji with gap up |

#### Reversal Patterns
| Pattern | ID | Parameters | Description |
|---------|-----|------------|-------------|
| Above the Stomach | `ABOVE_THE_STOMACH` | `penetration` (0.0) | Bullish: opens above prev midpoint |
| Below the Stomach | `BELOW_THE_STOMACH` | `penetration` (0.0) | Bearish: opens below prev midpoint |
| Collapsing Doji Star | `COLLAPSING_DOJI_STAR` | `body_pct` (0.1), `gap_pct` (0.3) | Doji star with gap + reversal |
| Deliberation | `DELIBERATION` | `body_pct` (0.3) | Three white, third small body |
| Last Engulfing Bottom | `LAST_ENGULFING_BOTTOM` | `trend_period` (14) | Final engulfing at bottom |
| Last Engulfing Top | `LAST_ENGULFING_TOP` | `trend_period` (14) | Final engulfing at top |
| Two Black Gapping | `TWO_BLACK_GAPPING` | — | Two bearish with gap down |
| Meeting Lines Bearish | `MEETING_LINES_BEARISH` | `tolerance` (0.001) | White + black, same close |
| Meeting Lines Bullish | `MEETING_LINES_BULLISH` | `tolerance` (0.001) | Black + white, same close |

#### Doji Variants
| Pattern | ID | Parameters | Description |
|---------|-----|------------|-------------|
| Northern Doji | `NORTHERN_DOJI` | `body_pct` (0.1), `trend_period` (14) | Doji after advance |
| Southern Doji | `SOUTHERN_DOJI` | `body_pct` (0.1), `trend_period` (14) | Doji after decline |

#### Marubozu Variants
| Pattern | ID | Parameters | Description |
|---------|-----|------------|-------------|
| Black Marubozu | `BLACK_MARUBOZU` | `shadow_tolerance` (0.01) | Bearish, no shadows |
| White Marubozu | `WHITE_MARUBOZU` | `shadow_tolerance` (0.01) | Bullish, no shadows |
| Opening Black Marubozu | `OPENING_BLACK_MARUBOZU` | `shadow_tolerance` (0.01) | No upper shadow |
| Opening White Marubozu | `OPENING_WHITE_MARUBOZU` | `shadow_tolerance` (0.01) | No lower shadow |

#### Basic Candles
| Pattern | ID | Parameters | Description |
|---------|-----|------------|-------------|
| Black Candle | `BLACK_CANDLE` | — | Simple bearish candle |
| White Candle | `WHITE_CANDLE` | — | Simple bullish candle |
| Short Black | `SHORT_BLACK` | `body_pct` (0.3) | Bearish with short body |
| Short White | `SHORT_WHITE` | `body_pct` (0.3) | Bullish with short body |
| Long Black Day | `LONG_BLACK_DAY` | `body_pct` (0.7) | Bearish with long body |
| Long White Day | `LONG_WHITE_DAY` | `body_pct` (0.7) | Bullish with long body |
| Black Spinning Top | `BLACK_SPINNING_TOP` | `body_pct` (0.3), `shadow_ratio` (0.5) | Bearish spinning top |
| White Spinning Top | `WHITE_SPINNING_TOP` | `body_pct` (0.3), `shadow_ratio` (0.5) | Bullish spinning top |

#### Additional Patterns
| Pattern | ID | Parameters | Description |
|---------|-----|------------|-------------|
| Shooting Star 2-Lines | `SHOOTING_STAR_2_LINES` | `body_pct` (0.3), `shadow_ratio` (2.0) | Shooting star with context |
| Downside Gap Three Methods | `DOWNSIDE_GAP_THREE_METHODS` | — | Gap down + fill (bearish continuation) |
| Upside Gap Three Methods | `UPSIDE_GAP_THREE_METHODS` | — | Gap up + fill (bullish continuation) |
| Downside Tasuki Gap | `DOWNSIDE_TASUKI_GAP` | `gap_fill_pct` (0.7) | Gap down + partial fill |
| Upside Tasuki Gap | `UPSIDE_TASUKI_GAP` | `gap_fill_pct` (0.7) | Gap up + partial fill |

---

## API Levels

```
┌─────────────────────────────────────────────┐
│         High-level (convenience)            │
│  scan(), scan_grouped(), iter()             │
├─────────────────────────────────────────────┤
│         Mid-level (control)                 │
│  scan_at(), scan_range()                    │
├─────────────────────────────────────────────┤
│         Low-level (primitives)              │
│  compute_contexts(), compute_context_at()   │
└─────────────────────────────────────────────┘
```

### High-Level: Batch Processing

```rust
// Flat list of all patterns
let patterns = engine.scan(&bars)?;

// Grouped by bar index (for backtesting)
let grouped = engine.scan_grouped(&bars)?;
for (i, patterns) in grouped.iter().enumerate() {
    strategy.on_bar(&bars[i], patterns);
}

// Iterator (streaming, low memory)
for bp in engine.iter(&bars) {
    process_bar(bp.index, &bp.patterns);
}
```

### Mid-Level: Fine Control

```rust
// Single bar detection
let ctx = engine.compute_context_at(&bars, index);
let patterns = engine.scan_at(&bars, index, &ctx);

// Range detection
let contexts = engine.compute_contexts(&bars);
let patterns = engine.scan_range(&bars, 100..200, &contexts);
```

### Low-Level: Primitives

```rust
// Precompute and reuse contexts
let contexts = engine.compute_contexts(&bars);

// Use contexts multiple times
let p1 = engine.scan_range(&bars, 0..100, &contexts);
let p2 = engine.scan_range(&bars, 100..200, &contexts);
```

## Configuration

### Pattern Groups

```rust
// All 97 patterns (TA-Lib + Extended)
let engine = EngineBuilder::new().with_all_defaults().build()?;

// Only TA-Lib single-bar patterns (17)
let engine = EngineBuilder::new().with_single_bar_defaults().build()?;

// Only TA-Lib two-bar patterns (18)
let engine = EngineBuilder::new().with_two_bar_defaults().build()?;

// Only TA-Lib three-bar patterns (20)
let engine = EngineBuilder::new().with_three_bar_defaults().build()?;

// Only extended patterns (33)
let engine = EngineBuilder::new().with_extended_defaults().build()?;
```

### Custom Pattern Parameters

```rust
let engine = EngineBuilder::new()
    .add(BuiltinDetector::Hammer(HammerDetector {
        body_max_ratio: Ratio::new(0.4)?,
        lower_shadow_min_ratio: Ratio::new(0.5)?,
        ..Default::default()
    }))
    .add(BuiltinDetector::PriceLines(PriceLinesDetector::ten()))
    .build()?;
```

### Filtering

```rust
let engine = EngineBuilder::new()
    .with_all_defaults()
    .min_strength(0.6)  // Only patterns with strength >= 0.6
    .only_patterns([    // Only specific patterns
        PatternId("CDL_HAMMER"),
        PatternId("CDL_DOJI"),
        PatternId("FALLING_WINDOW"),
    ])
    .build()?;
```

## Custom Detectors

```rust
struct MyPattern;

impl PatternDetector for MyPattern {
    fn id(&self) -> PatternId { PatternId("my_pattern") }
    fn min_bars(&self) -> usize { 2 }

    fn detect<T: OHLCV>(
        &self,
        bars: &[T],
        index: usize,
        ctx: &MarketContext,
    ) -> Option<PatternMatch> {
        let curr = bars.get(index)?;
        let prev = bars.get(index.checked_sub(1)?)?;

        // Your detection logic
        (curr.close() > prev.high()).then(|| PatternMatch {
            pattern_id: PatternDetector::id(self),
            direction: Direction::Bullish,
            strength: 0.7,
            start_index: index - 1,
            end_index: index,
        })
    }
}

let engine = EngineBuilder::new()
    .with_all_defaults()
    .add_custom(MyPattern)
    .build()?;
```

## Parallel Scanning

```rust
let instruments: Vec<(&str, &[Bar])> = vec![
    ("AAPL", &aapl_bars),
    ("GOOGL", &googl_bars),
    ("MSFT", &msft_bars),
];

let (results, errors) = scan_parallel(&engine, instruments);

for result in results {
    println!("{}: {} patterns found", result.symbol, result.patterns.len());
}
```

## Realtime Usage

```rust
let engine = EngineBuilder::new().with_all_defaults().build()?;
let mut bars = Vec::new();

loop {
    let new_bar = receive_bar();
    bars.push(new_bar);

    // Compute context for new bar
    let ctx = engine.compute_context_at(&bars, bars.len() - 1);

    // Detect patterns at new bar
    let patterns = engine.scan_at(&bars, bars.len() - 1, &ctx);

    for p in patterns {
        handle_signal(p);
    }
}
```

## Market Context

The library computes market context (trend, volatility, average volume) for better pattern detection:

```rust
// Custom context provider
struct MyContextProvider;

impl ContextProvider for MyContextProvider {
    fn compute_all<T: OHLCV>(&self, bars: &[T]) -> Vec<MarketContext> {
        // Your context computation
    }
}

let engine = EngineBuilder::new()
    .context_provider(MyContextProvider)
    .with_all_defaults()
    .build()?;
```

## Performance

- **Fast path**: Builtin detectors use enum dispatch (no vtable)
- **Context caching**: Compute once, reuse for all detectors
- **Lazy allocation**: `bar_refs` only created when custom detectors exist
- **Zero-copy**: `PatternMatch` is `Copy`, no allocations per match

## Benchmarks

```bash
cargo bench
```

| Benchmark | Time |
|-----------|------|
| Doji detection (1000 bars) | ~50µs |
| All patterns (1000 bars) | ~2ms |
| Parallel scan (4 instruments × 1000 bars) | ~3ms |

## Support

- [Boosty](https://boosty.to/nalofc/donate)
- ETH: `0x8DE690f1B58F31451b30862A18bae0845Da4026f`

## License

MIT
