use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct RankStats {
    pub expected_matches: f64,
    pub promotion_probability: f64,
}

#[wasm_bindgen]
pub fn calculate_rank_stats_formulas(p: f64, k_val: i32, current_net_wins_val: i32) -> RankStats {
    // Early validation and edge cases first
    if k_val != 4 && k_val != 5 {
        return RankStats {
            expected_matches: f64::INFINITY,
            promotion_probability: f64::NAN,
        };
    }

    // Handle edge cases for p=0 and p=1 first (most intuitive and avoids polynomial calculations)
    // also treat any negative p as 0, and any p > 1 as 1
    if p <= 0.0 {
        let expected = match current_net_wins_val {
            c_wins if c_wins >= -2 => 3.0 + c_wins as f64, // 3 losses from 0, + c_wins to reach 0
            _ => 0.0, // Should be caught by input validation or already demoted
        };
        return RankStats {
            expected_matches: expected,
            promotion_probability: 0.0,
        };
    }

    if p >= 1.0 {
        let expected = if current_net_wins_val < k_val {
            if current_net_wins_val < 0 {
                // 1 win to get to +1 state, then (K-1) more wins
                1.0 + (k_val - 1) as f64
            } else {
                (k_val - current_net_wins_val) as f64
            }
        } else {
            0.0 // Already at or above K
        };
        return RankStats {
            expected_matches: expected,
            promotion_probability: 1.0,
        };
    }

    let martingale = create_martingale_fn(p);

    let promotion_probability =
        (martingale(current_net_wins_val) - martingale(-3)) / (martingale(k_val) - martingale(-3));

    // Pre-calculate powers of p only once
    let p2 = p * p;
    let p3 = p2 * p;
    let p4 = p3 * p;
    let p5 = p4 * p;
    let p6 = p5 * p;

    let expected_matches = if k_val == 4 {
        calculate_k4_stats(p, p2, p3, p4, p5, p6, current_net_wins_val)
    } else {
        calculate_k5_stats(p, p2, p3, p4, p5, p6, current_net_wins_val)
    };

    RankStats {
        expected_matches: expected_matches.max(0.0),
        promotion_probability: promotion_probability.max(0.0).min(1.0),
    }
}

#[inline]
fn calculate_k4_stats(
    p: f64,
    p2: f64,
    p3: f64,
    p4: f64,
    p5: f64,
    p6: f64,
    current_net_wins_val: i32,
) -> f64 {
    let denom = p6 - 5.0 * p5 + 11.0 * p4 - 13.0 * p3 + 11.0 * p2 - 5.0 * p + 1.0;

    let expected = match current_net_wins_val {
        -2 => (2.0 * p5 - 4.0 * p4 - p3 + 10.0 * p2 - 6.0 * p + 3.0) / denom,
        -1 => (-p5 + 5.0 * p4 - 11.0 * p3 + 15.0 * p2 - 9.0 * p + 4.0) / denom,
        0 => (p5 - 2.0 * p4 - 3.0 * p3 + 13.0 * p2 - 12.0 * p + 5.0) / denom,
        1 => (-2.0 * p4 + 2.0 * p3 + 5.0 * p2 - 3.0 * p + 2.0) / denom,
        2 => (2.0 * p4 + 2.0 * p3 + p2 - p + 1.0) / denom,
        3 => (-2.0 * p5 + 12.0 * p4 - 29.0 * p3 + 36.0 * p2 - 22.0 * p + 6.0) / denom,
        _ => f64::INFINITY,
    };

    expected
}

#[inline]
fn calculate_k5_stats(
    p: f64,
    p2: f64,
    p3: f64,
    p4: f64,
    p5: f64,
    p6: f64,
    current_net_wins_val: i32,
) -> f64 {
    let denom = 2.0 * p6 - 10.0 * p5 + 22.0 * p4 - 24.0 * p3 + 16.0 * p2 - 6.0 * p + 1.0;
    let expected = match current_net_wins_val {
        -2 => (3.0 * p4 + 3.0 * p3 - 2.0 * p + 1.0) / denom,
        -1 => (3.0 * p6 - 6.0 * p4 + 3.0 * p3 - 8.0 * p2 + 5.0 * p - 2.0) / denom,
        0 => (3.0 * p6 - 9.0 * p5 + 12.0 * p4 - 11.0 * p3 + 16.0 * p2 - 9.0 * p + 3.0) / denom,
        1 => (-2.0 * p5 + 10.0 * p4 - 19.0 * p3 + 24.0 * p2 - 13.0 * p + 4.0) / denom,
        2 => (3.0 * p6 - 14.0 * p5 + 29.0 * p4 - 35.0 * p3 + 32.0 * p2 - 17.0 * p + 5.0) / denom,
        3 => (-p5 + 8.0 * p4 - 22.0 * p3 + 32.0 * p2 - 21.0 * p + 6.0) / denom,
        4 => (3.0 * p6 - 19.0 * p5 + 52.0 * p4 - 78.0 * p3 + 69.0 * p2 - 33.0 * p + 7.0) / denom,
        _ => f64::INFINITY,
    };

    expected
}

#[inline]
fn create_martingale_fn(p: f64) -> impl Fn(i32) -> f64 {
    move |n: i32| -> f64 {
        if n >= 0 {
            if p == 0.5 {
                n as f64
            } else {
                let factor = p / (2.0 * p - 1.0);
                let ratio = (1.0 - p) / p;
                factor * (1.0 - ratio.powi(n))
            }
        } else {
            1.0 - (1.0 - p).powi(n)
        }
    }
}
