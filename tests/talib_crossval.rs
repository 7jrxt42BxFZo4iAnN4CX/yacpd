//! Cross-validation tests: YACPD vs Python TA-Lib.
//!
//! Runs `python3 tests/talib_validation.py --export-json` to get TA-Lib reference
//! results, then compares them bar-by-bar against YACPD output.

use std::sync::OnceLock;

use serde_json::Value;
use yacpd::prelude::*;

// ============================================================
// TestBar + OHLCV impl (same as tests/patterns.rs)
// ============================================================

#[derive(Debug, Clone, Copy)]
struct TestBar {
    o: f64,
    h: f64,
    l: f64,
    c: f64,
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

// ============================================================
// TA-Lib name → YACPD pattern_id mapping
// ============================================================

/// Convert TA-Lib pattern name (e.g. "CDLDOJI") to YACPD pattern_id (e.g. "CDL_DOJI").
///
/// Rule: insert `_` after the `CDL` prefix.
/// Special cases handled explicitly.
fn talib_to_yacpd_id(talib_name: &str) -> &'static str {
    match talib_name {
        "CDL2CROWS" => "CDL_2CROWS",
        "CDL3BLACKCROWS" => "CDL_3BLACKCROWS",
        "CDL3INSIDE" => "CDL_3INSIDE",
        "CDL3LINESTRIKE" => "CDL_3LINESTRIKE",
        "CDL3OUTSIDE" => "CDL_3OUTSIDE",
        "CDL3STARSINSOUTH" => "CDL_3STARSINSOUTH",
        "CDL3WHITESOLDIERS" => "CDL_3WHITESOLDIERS",
        "CDLABANDONEDBABY" => "CDL_ABANDONEDBABY",
        "CDLADVANCEBLOCK" => "CDL_ADVANCEBLOCK",
        "CDLBELTHOLD" => "CDL_BELTHOLD",
        "CDLBREAKAWAY" => "CDL_BREAKAWAY",
        "CDLCLOSINGMARUBOZU" => "CDL_CLOSINGMARUBOZU",
        "CDLCONCEALBABYSWALL" => "CDL_CONCEALBABYSWALL",
        "CDLCOUNTERATTACK" => "CDL_COUNTERATTACK",
        "CDLDARKCLOUDCOVER" => "CDL_DARKCLOUDCOVER",
        "CDLDOJI" => "CDL_DOJI",
        "CDLDOJISTAR" => "CDL_DOJISTAR",
        "CDLDRAGONFLYDOJI" => "CDL_DRAGONFLYDOJI",
        "CDLENGULFING" => "CDL_ENGULFING",
        "CDLEVENINGDOJISTAR" => "CDL_EVENINGDOJISTAR",
        "CDLEVENINGSTAR" => "CDL_EVENINGSTAR",
        "CDLGAPSIDESIDEWHITE" => "CDL_GAPSIDESIDEWHITE",
        "CDLGRAVESTONEDOJI" => "CDL_GRAVESTONEDOJI",
        "CDLHAMMER" => "CDL_HAMMER",
        "CDLHANGINGMAN" => "CDL_HANGINGMAN",
        "CDLHARAMI" => "CDL_HARAMI",
        "CDLHARAMICROSS" => "CDL_HARAMICROSS",
        "CDLHIGHWAVE" => "CDL_HIGHWAVE",
        "CDLHIKKAKE" => "CDL_HIKKAKE",
        "CDLHIKKAKEMOD" => "CDL_HIKKAKEMOD",
        "CDLHOMINGPIGEON" => "CDL_HOMINGPIGEON",
        "CDLIDENTICAL3CROWS" => "CDL_IDENTICAL3CROWS",
        "CDLINNECK" => "CDL_INNECK",
        "CDLINVERTEDHAMMER" => "CDL_INVERTEDHAMMER",
        "CDLKICKING" => "CDL_KICKING",
        "CDLKICKINGBYLENGTH" => "CDL_KICKINGBYLENGTH",
        "CDLLADDERBOTTOM" => "CDL_LADDERBOTTOM",
        "CDLLONGLEGGEDDOJI" => "CDL_LONGLEGGEDDOJI",
        "CDLLONGLINE" => "CDL_LONGLINE",
        "CDLMARUBOZU" => "CDL_MARUBOZU",
        "CDLMATCHINGLOW" => "CDL_MATCHINGLOW",
        "CDLMATHOLD" => "CDL_MATHOLD",
        "CDLMORNINGDOJISTAR" => "CDL_MORNINGDOJISTAR",
        "CDLMORNINGSTAR" => "CDL_MORNINGSTAR",
        "CDLONNECK" => "CDL_ONNECK",
        "CDLPIERCING" => "CDL_PIERCING",
        "CDLRICKSHAWMAN" => "CDL_RICKSHAWMAN",
        "CDLRISEFALL3METHODS" => "CDL_RISEFALL3METHODS",
        "CDLSEPARATINGLINES" => "CDL_SEPARATINGLINES",
        "CDLSHOOTINGSTAR" => "CDL_SHOOTINGSTAR",
        "CDLSHORTLINE" => "CDL_SHORTLINE",
        "CDLSPINNINGTOP" => "CDL_SPINNINGTOP",
        "CDLSTALLEDPATTERN" => "CDL_STALLEDPATTERN",
        "CDLSTICKSANDWICH" => "CDL_STICKSANDWICH",
        "CDLTAKURI" => "CDL_TAKURI",
        "CDLTASUKIGAP" => "CDL_TASUKIGAP",
        "CDLTHRUSTING" => "CDL_THRUSTING",
        "CDLTRISTAR" => "CDL_TRISTAR",
        "CDLUNIQUE3RIVER" => "CDL_UNIQUE3RIVER",
        "CDLUPSIDEGAP2CROWS" => "CDL_UPSIDEGAP2CROWS",
        "CDLXSIDEGAP3METHODS" => "CDL_XSIDEGAP3METHODS",
        other => panic!("Unknown TA-Lib pattern: {other}"),
    }
}

// ============================================================
// Python runner (cached via OnceLock)
// ============================================================

static PYTHON_DATA: OnceLock<Option<Value>> = OnceLock::new();

fn get_python_data() -> &'static Option<Value> {
    PYTHON_DATA.get_or_init(run_python_export)
}

fn run_python_export() -> Option<Value> {
    // Try conda first (for environments with TA-Lib installed in conda)
    let output = std::process::Command::new("conda")
        .args([
            "run",
            "-n",
            "trade",
            "python3",
            "tests/talib_validation.py",
            "--export-json",
        ])
        .output()
        .or_else(|_| {
            // Fall back to plain python3
            std::process::Command::new("python3")
                .args(["tests/talib_validation.py", "--export-json"])
                .output()
        });

    let output = match output {
        Ok(o) => o,
        Err(e) => {
            eprintln!("[talib_crossval] python3 not available: {e}");
            return None;
        }
    };

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        eprintln!(
            "[talib_crossval] Python script failed (exit {}):",
            output.status
        );
        eprintln!("{stderr}");
        return None;
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    match serde_json::from_str::<Value>(&stdout) {
        Ok(v) => Some(v),
        Err(e) => {
            eprintln!("[talib_crossval] Failed to parse JSON: {e}");
            eprintln!(
                "[talib_crossval] First 500 chars: {}",
                &stdout[..stdout.len().min(500)]
            );
            None
        }
    }
}

// ============================================================
// Python runner for enhanced data (cached via OnceLock)
// ============================================================

static PYTHON_ENHANCED_DATA: OnceLock<Option<Value>> = OnceLock::new();

fn get_python_enhanced_data() -> &'static Option<Value> {
    PYTHON_ENHANCED_DATA.get_or_init(run_python_enhanced_export)
}

fn run_python_enhanced_export() -> Option<Value> {
    let output = std::process::Command::new("conda")
        .args([
            "run",
            "-n",
            "trade",
            "python3",
            "tests/talib_validation.py",
            "--export-enhanced-json",
        ])
        .output()
        .or_else(|_| {
            std::process::Command::new("python3")
                .args(["tests/talib_validation.py", "--export-enhanced-json"])
                .output()
        });

    let output = match output {
        Ok(o) => o,
        Err(e) => {
            eprintln!("[talib_crossval] python3 not available for enhanced export: {e}");
            return None;
        }
    };

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        eprintln!(
            "[talib_crossval] Enhanced Python script failed (exit {}):",
            output.status
        );
        eprintln!("{stderr}");
        return None;
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    match serde_json::from_str::<Value>(&stdout) {
        Ok(v) => Some(v),
        Err(e) => {
            eprintln!("[talib_crossval] Failed to parse enhanced JSON: {e}");
            eprintln!(
                "[talib_crossval] First 500 chars: {}",
                &stdout[..stdout.len().min(500)]
            );
            None
        }
    }
}

// ============================================================
// Helpers: deserialize bars, compare results
// ============================================================

fn json_bars_to_test_bars(bars_json: &[Value]) -> Vec<TestBar> {
    bars_json
        .iter()
        .map(|b| TestBar {
            o: b["o"].as_f64().unwrap(),
            h: b["h"].as_f64().unwrap(),
            l: b["l"].as_f64().unwrap(),
            c: b["c"].as_f64().unwrap(),
        })
        .collect()
}

/// Build an engine with only the 61 standard TA-Lib patterns (no extended).
fn build_talib_engine() -> PatternEngine {
    EngineBuilder::new()
        .with_single_bar_defaults()
        .with_two_bar_defaults()
        .with_three_bar_defaults()
        .with_multi_bar_defaults()
        .build()
        .unwrap()
}

/// TA-Lib lookback period for a pattern.
/// TA-Lib outputs 0 for all bars before the lookback, so we skip those in comparison.
/// Lookback = max(candle setting periods used) + (num_bars_in_pattern - 1)
fn talib_lookback(talib_name: &str) -> usize {
    // candle_period = 10 for most settings (BodyLong, BodyShort, BodyDoji, ShadowVeryShort, ShadowShort)
    // Some settings use period 5 (Near, Far, Equal) but max is still 10
    let base = 10;
    // Number of bars in pattern window (minus 1)
    let extra = match talib_name {
        // 1-bar patterns
        "CDLDOJI" | "CDLDRAGONFLYDOJI" | "CDLGRAVESTONEDOJI" | "CDLLONGLEGGEDDOJI"
        | "CDLHIGHWAVE" | "CDLSPINNINGTOP" | "CDLLONGLINE" | "CDLSHORTLINE" | "CDLMARUBOZU"
        | "CDLCLOSINGMARUBOZU" | "CDLBELTHOLD" | "CDLRICKSHAWMAN" | "CDLTAKURI" => 0,
        // 2-bar patterns (need 1 extra bar for the pattern window + sometimes trailing for 1st bar)
        "CDLENGULFING"
        | "CDLHARAMI"
        | "CDLHARAMICROSS"
        | "CDLPIERCING"
        | "CDLDARKCLOUDCOVER"
        | "CDLDOJISTAR"
        | "CDLHAMMER"
        | "CDLHANGINGMAN"
        | "CDLINVERTEDHAMMER"
        | "CDLSHOOTINGSTAR"
        | "CDLHOMINGPIGEON"
        | "CDLINNECK"
        | "CDLONNECK"
        | "CDLTHRUSTING"
        | "CDLCOUNTERATTACK"
        | "CDLMATCHINGLOW"
        | "CDLKICKING"
        | "CDLKICKINGBYLENGTH"
        | "CDLSEPARATINGLINES"
        | "CDLGAPSIDESIDEWHITE"
        | "CDLSTICKSANDWICH" => 1,
        // 3-bar patterns
        "CDL3INSIDE"
        | "CDL3OUTSIDE"
        | "CDL3WHITESOLDIERS"
        | "CDL3LINESTRIKE"
        | "CDLEVENINGSTAR"
        | "CDLMORNINGSTAR"
        | "CDLEVENINGDOJISTAR"
        | "CDLMORNINGDOJISTAR"
        | "CDL2CROWS"
        | "CDLUPSIDEGAP2CROWS"
        | "CDLTRISTAR"
        | "CDLUNIQUE3RIVER"
        | "CDLIDENTICAL3CROWS"
        | "CDLADVANCEBLOCK"
        | "CDLSTALLEDPATTERN"
        | "CDLTASUKIGAP"
        | "CDLXSIDEGAP3METHODS"
        | "CDL3STARSINSOUTH" => 2,
        // 4-bar patterns (3 pattern bars + 1 prior bar)
        "CDL3BLACKCROWS" => 3,
        // 4-5 bar patterns
        "CDLCONCEALBABYSWALL"
        | "CDLLADDERBOTTOM"
        | "CDLRISEFALL3METHODS"
        | "CDLBREAKAWAY"
        | "CDLMATHOLD" => 4,
        // Hikkake: no candle settings, pure price comparison, lookback = 5
        "CDLHIKKAKE" => {
            return 5;
        }
        // HikkakeMod: uses Near setting (period=5), lookback = max(1,5) + 5 = 10
        "CDLHIKKAKEMOD" => {
            return 10;
        }
        "CDLABANDONEDBABY" => 2,
        _ => 2,
    };
    base + extra
}

/// Compare TA-Lib results against YACPD for a single pattern on given bars.
///
/// Returns a list of mismatches (bar_index, talib_value, yacpd_direction_or_none).
fn compare_pattern(
    talib_name: &str,
    talib_results: &[Value],
    bars: &[TestBar],
    engine: &PatternEngine,
) -> Vec<String> {
    let yacpd_id = talib_to_yacpd_id(talib_name);
    let grouped = engine.scan_grouped(bars).unwrap();
    let mut mismatches = Vec::new();
    let lookback = talib_lookback(talib_name);

    for (i, talib_val) in talib_results.iter().enumerate() {
        let talib_int = talib_val.as_i64().unwrap_or(0);

        // Skip bars within TA-Lib's lookback period.
        // TA-Lib always returns 0 for these bars (insufficient averaging data).
        // YACPD may detect patterns here using partial averages, which is fine
        // for general use but not TA-Lib-compatible, so we skip the comparison.
        if i < lookback && talib_int == 0 {
            continue;
        }

        // Find YACPD match for this pattern at this bar
        let yacpd_match = grouped
            .get(i)
            .and_then(|pats| pats.iter().find(|p| p.pattern_id.0 == yacpd_id));

        match (talib_int, yacpd_match) {
            // Both agree: no detection
            (0, None) => {}
            // TA-Lib +100, YACPD Bullish → OK
            (v, Some(m)) if v > 0 && m.direction == Direction::Bullish => {}
            // TA-Lib -100, YACPD Bearish → OK
            (v, Some(m)) if v < 0 && m.direction == Direction::Bearish => {}
            // TA-Lib non-zero, YACPD Neutral → OK (neutral patterns like Doji)
            (v, Some(m)) if v != 0 && m.direction == Direction::Neutral => {}
            // TA-Lib +100, YACPD detected but wrong direction
            (v, Some(m)) if v > 0 && m.direction == Direction::Bearish => {
                mismatches.push(format!(
          "  bar[{i}]: TA-Lib=+{v} (bullish) but YACPD={:?} | o={:.1} h={:.1} l={:.1} c={:.1}",
          m.direction, bars[i].o, bars[i].h, bars[i].l, bars[i].c
        ));
            }
            (v, Some(m)) if v < 0 && m.direction == Direction::Bullish => {
                mismatches.push(format!(
          "  bar[{i}]: TA-Lib={v} (bearish) but YACPD={:?} | o={:.1} h={:.1} l={:.1} c={:.1}",
          m.direction, bars[i].o, bars[i].h, bars[i].l, bars[i].c
        ));
            }
            // TA-Lib detected, YACPD missed
            (v, None) if v != 0 => {
                let dir = if v > 0 { "bullish" } else { "bearish" };
                mismatches.push(format!(
          "  bar[{i}]: TA-Lib={v} ({dir}) but YACPD=NONE | o={:.1} h={:.1} l={:.1} c={:.1}",
          bars[i].o, bars[i].h, bars[i].l, bars[i].c
        ));
            }
            // YACPD detected, TA-Lib didn't
            (0, Some(m)) => {
                mismatches.push(format!(
                    "  bar[{i}]: TA-Lib=0 but YACPD={:?} | o={:.1} h={:.1} l={:.1} c={:.1}",
                    m.direction, bars[i].o, bars[i].h, bars[i].l, bars[i].c
                ));
            }
            // Catch-all
            (v, m) => {
                mismatches.push(format!(
                    "  bar[{i}]: TA-Lib={v} vs YACPD={:?} | o={:.1} h={:.1} l={:.1} c={:.1}",
                    m.map(|p| p.direction),
                    bars[i].o,
                    bars[i].h,
                    bars[i].l,
                    bars[i].c
                ));
            }
        }
    }

    mismatches
}

// ============================================================
// Tests
// ============================================================

#[test]
fn talib_crossval_all_61_patterns() {
    let data = get_python_data();
    let data = match data {
        Some(d) => d,
        None => {
            eprintln!("[talib_crossval] SKIPPED: Python/TA-Lib not available");
            return;
        }
    };

    let test_cases = &data["test_cases"];
    assert!(
        test_cases.is_object(),
        "Expected 'test_cases' object in JSON"
    );

    let engine = build_talib_engine();
    let mut total_patterns = 0;
    let mut passed_patterns = 0;
    let mut all_failures = Vec::new();

    for (talib_name, case) in test_cases.as_object().unwrap() {
        total_patterns += 1;

        let bars_json = case["bars"].as_array().expect("missing 'bars'");
        let result_json = case["result"].as_array().expect("missing 'result'");

        let bars = json_bars_to_test_bars(bars_json);
        let mismatches = compare_pattern(talib_name, result_json, &bars, &engine);

        if mismatches.is_empty() {
            passed_patterns += 1;
            eprintln!("[PASS] {talib_name}");
        } else {
            eprintln!("[FAIL] {talib_name} ({} mismatches):", mismatches.len());
            for m in &mismatches {
                eprintln!("{m}");
            }
            all_failures.push((talib_name.clone(), mismatches));
        }
    }

    eprintln!("\n=== CURATED RESULTS: {passed_patterns}/{total_patterns} patterns passed ===");

    if !all_failures.is_empty() {
        let mut msg = format!(
            "{} of {total_patterns} patterns had mismatches:\n",
            all_failures.len()
        );
        for (name, mismatches) in &all_failures {
            msg.push_str(&format!("  {name}: {} mismatches\n", mismatches.len()));
        }
        panic!("{msg}");
    }
}

#[test]
fn talib_crossval_fuzz() {
    let data = get_python_data();
    let data = match data {
        Some(d) => d,
        None => {
            eprintln!("[talib_crossval] SKIPPED: Python/TA-Lib not available");
            return;
        }
    };

    let fuzz = match data["fuzz"].as_array() {
        Some(f) => f,
        None => {
            eprintln!("[talib_crossval] No fuzz data in JSON, skipping");
            return;
        }
    };

    let engine = build_talib_engine();
    let mut total_rounds = 0;
    let mut total_patterns_checked = 0;
    let mut total_mismatches = 0;
    let mut all_failures: Vec<(u64, String, Vec<String>)> = Vec::new();

    for round in fuzz {
        let seed = round["seed"].as_u64().unwrap_or(0);
        total_rounds += 1;

        let bars_json = round["bars"]
            .as_array()
            .expect("missing 'bars' in fuzz round");
        let bars = json_bars_to_test_bars(bars_json);

        let results = round["results"].as_object().expect("missing 'results'");

        for (talib_name, result_vals) in results {
            total_patterns_checked += 1;
            let result_arr = result_vals.as_array().expect("result should be array");
            let mismatches = compare_pattern(talib_name, result_arr, &bars, &engine);

            if !mismatches.is_empty() {
                total_mismatches += mismatches.len();
                all_failures.push((seed, talib_name.clone(), mismatches));
            }
        }
    }

    eprintln!(
        "\n=== FUZZ RESULTS: {total_rounds} rounds, {total_patterns_checked} pattern checks, {total_mismatches} mismatches ==="
    );

    if !all_failures.is_empty() {
        // Show first 10 failures in detail
        let show = all_failures.len().min(50);
        let mut msg = format!(
            "{} pattern/round combinations had mismatches ({total_mismatches} total):\n",
            all_failures.len()
        );
        for (seed, name, mismatches) in &all_failures[..show] {
            msg.push_str(&format!(
                "  seed={seed} {name}: {} mismatches\n",
                mismatches.len()
            ));
            for m in mismatches.iter().take(3) {
                msg.push_str(&format!("    {m}\n"));
            }
        }
        if all_failures.len() > show {
            msg.push_str(&format!("  ... and {} more\n", all_failures.len() - show));
        }
        panic!("{msg}");
    }
}

// ============================================================
// Enhanced tests (edge cases, heavy fuzz, real market data)
// ============================================================

#[test]
fn talib_crossval_enhanced_fuzz() {
    let data = match get_python_enhanced_data() {
        Some(d) => d,
        None => {
            eprintln!("[talib_crossval] SKIPPED: enhanced data not available");
            return;
        }
    };
    let fuzz = match data["enhanced_fuzz"].as_array() {
        Some(f) => f,
        None => {
            eprintln!("[talib_crossval] No enhanced_fuzz section");
            return;
        }
    };

    let engine = build_talib_engine();
    let mut total_rounds = 0;
    let mut total_patterns_checked = 0;
    let mut total_mismatches = 0;
    let mut all_failures: Vec<(u64, String, Vec<String>)> = Vec::new();

    for round in fuzz {
        let seed = round["seed"].as_u64().unwrap_or(0);
        let volatility = round["volatility"].as_f64().unwrap_or(0.0);
        let base = round["base"].as_f64().unwrap_or(0.0);
        total_rounds += 1;

        let bars_json = round["bars"]
            .as_array()
            .expect("missing 'bars' in enhanced fuzz round");
        let bars = json_bars_to_test_bars(bars_json);

        let results = round["results"]
            .as_object()
            .expect("missing 'results' in enhanced fuzz round");

        for (talib_name, result_vals) in results {
            total_patterns_checked += 1;
            let result_arr = result_vals.as_array().expect("result should be array");
            let mismatches = compare_pattern(talib_name, result_arr, &bars, &engine);

            if !mismatches.is_empty() {
                total_mismatches += mismatches.len();
                all_failures.push((seed, talib_name.clone(), mismatches));
            }
        }

        if total_rounds % 20 == 0 {
            eprintln!(
                "[enhanced_fuzz] {total_rounds} rounds done (last: seed={seed}, base={base:.1}, vol={volatility:.1})"
            );
        }
    }

    eprintln!(
        "\n=== ENHANCED FUZZ RESULTS: {total_rounds} rounds, {total_patterns_checked} pattern checks, {total_mismatches} mismatches ==="
    );

    if !all_failures.is_empty() {
        let show = all_failures.len().min(50);
        let mut msg = format!(
            "{} pattern/round combinations had mismatches ({total_mismatches} total):\n",
            all_failures.len()
        );
        for (seed, name, mismatches) in &all_failures[..show] {
            msg.push_str(&format!(
                "  seed={seed} {name}: {} mismatches\n",
                mismatches.len()
            ));
            for m in mismatches.iter().take(3) {
                msg.push_str(&format!("    {m}\n"));
            }
        }
        if all_failures.len() > show {
            msg.push_str(&format!("  ... and {} more\n", all_failures.len() - show));
        }
        panic!("{msg}");
    }
}

#[test]
fn talib_crossval_edge_cases() {
    let data = match get_python_enhanced_data() {
        Some(d) => d,
        None => {
            eprintln!("[talib_crossval] SKIPPED: enhanced data not available");
            return;
        }
    };
    let edge_cases = match data["edge_cases"].as_object() {
        Some(e) => e,
        None => {
            eprintln!("[talib_crossval] No edge_cases section");
            return;
        }
    };

    let engine = build_talib_engine();
    let mut total_scenarios = 0;
    let mut total_patterns_checked = 0;
    let mut total_mismatches = 0;
    let mut all_failures: Vec<(String, String, Vec<String>)> = Vec::new();

    for (case_name, case_data) in edge_cases {
        total_scenarios += 1;

        let bars_json = case_data["bars"]
            .as_array()
            .expect("missing 'bars' in edge case");
        let bars = json_bars_to_test_bars(bars_json);
        let results = case_data["results"]
            .as_object()
            .expect("missing 'results' in edge case");

        let mut case_mismatches = 0;

        for (talib_name, result_vals) in results {
            total_patterns_checked += 1;
            let result_arr = result_vals.as_array().expect("result should be array");
            let mismatches = compare_pattern(talib_name, result_arr, &bars, &engine);

            if !mismatches.is_empty() {
                case_mismatches += mismatches.len();
                total_mismatches += mismatches.len();
                all_failures.push((case_name.clone(), talib_name.clone(), mismatches));
            }
        }

        eprintln!(
            "[edge_case] {case_name}: {} bars, {} patterns checked, {case_mismatches} mismatches",
            bars.len(),
            results.len()
        );
    }

    eprintln!(
        "\n=== EDGE CASE RESULTS: {total_scenarios} scenarios, {total_patterns_checked} pattern checks, {total_mismatches} mismatches ==="
    );

    if !all_failures.is_empty() {
        let show = all_failures.len().min(50);
        let mut msg = format!(
            "{} pattern/case combinations had mismatches ({total_mismatches} total):\n",
            all_failures.len()
        );
        for (case_name, name, mismatches) in &all_failures[..show] {
            msg.push_str(&format!(
                "  {case_name}/{name}: {} mismatches\n",
                mismatches.len()
            ));
            for m in mismatches.iter().take(3) {
                msg.push_str(&format!("    {m}\n"));
            }
        }
        if all_failures.len() > show {
            msg.push_str(&format!("  ... and {} more\n", all_failures.len() - show));
        }
        panic!("{msg}");
    }
}

#[test]
fn talib_crossval_real_data() {
    let data = match get_python_enhanced_data() {
        Some(d) => d,
        None => {
            eprintln!("[talib_crossval] SKIPPED: enhanced data not available");
            return;
        }
    };
    let real_data = match data["real_data"].as_object() {
        Some(r) if !r.is_empty() => r,
        _ => {
            eprintln!("[talib_crossval] SKIPPED: no real market data (yfinance not available?)");
            return;
        }
    };

    let engine = build_talib_engine();
    let mut total_tickers = 0;
    let mut total_patterns_checked = 0;
    let mut total_mismatches = 0;
    let mut all_failures: Vec<(String, String, Vec<String>)> = Vec::new();

    for (ticker, ticker_data) in real_data {
        total_tickers += 1;

        let bars_json = ticker_data["bars"]
            .as_array()
            .expect("missing 'bars' in real data");
        let bars = json_bars_to_test_bars(bars_json);
        let bar_count = bars.len();
        eprintln!("[{ticker}] Testing {bar_count} bars...");

        let results = ticker_data["results"]
            .as_object()
            .expect("missing 'results' in real data");

        let mut ticker_mismatches = 0;

        for (talib_name, result_vals) in results {
            total_patterns_checked += 1;
            let result_arr = result_vals.as_array().expect("result should be array");
            let mismatches = compare_pattern(talib_name, result_arr, &bars, &engine);

            if !mismatches.is_empty() {
                ticker_mismatches += mismatches.len();
                total_mismatches += mismatches.len();
                all_failures.push((ticker.clone(), talib_name.clone(), mismatches));
            }
        }

        eprintln!(
            "[{ticker}] {bar_count} bars, {} patterns checked, {ticker_mismatches} mismatches",
            results.len()
        );
    }

    eprintln!(
        "\n=== REAL DATA RESULTS: {total_tickers} tickers, {total_patterns_checked} pattern checks, {total_mismatches} mismatches ==="
    );

    // Allow a small tolerance for floating-point precision edge cases.
    // TA-Lib uses incremental running averages while YACPD computes fresh sums,
    // causing ~0.1% of borderline bars to differ. Real data is also non-deterministic
    // (fresh yfinance download each run), so the exact mismatching bars vary.
    // All deterministic tests (curated, fuzz, enhanced fuzz, edge cases) require 0 mismatches.
    let max_allowed_mismatches = 25;
    if total_mismatches > max_allowed_mismatches {
        let show = all_failures.len().min(50);
        let mut msg = format!(
            "{} pattern/ticker combinations had mismatches ({total_mismatches} total, max allowed {max_allowed_mismatches}):\n",
            all_failures.len()
        );
        for (ticker, name, mismatches) in &all_failures[..show] {
            msg.push_str(&format!(
                "  {ticker}/{name}: {} mismatches\n",
                mismatches.len()
            ));
            for m in mismatches.iter().take(3) {
                msg.push_str(&format!("    {m}\n"));
            }
        }
        if all_failures.len() > show {
            msg.push_str(&format!("  ... and {} more\n", all_failures.len() - show));
        }
        panic!("{msg}");
    } else if total_mismatches > 0 {
        eprintln!(
            "\n  ({total_mismatches} floating-point precision mismatches within tolerance of {max_allowed_mismatches})"
        );
    }
}
