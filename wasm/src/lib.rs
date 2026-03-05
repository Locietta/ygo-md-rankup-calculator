use wasm_bindgen::prelude::*;

const EPS: f64 = 1e-12;

#[wasm_bindgen]
pub struct RankStats {
    pub expected_matches: f64,
    pub promotion_probability: f64,
}

#[wasm_bindgen]
pub struct RankProgressStats {
    pub leave_current_segment_expected: f64,
    pub current_segment_promotion_probability: f64,
    pub expected_to_current_tier_i: f64,
    pub expected_to_next_big_tier_v: f64,
}

#[wasm_bindgen]
pub fn calculate_rank_stats_formulas(p: f64, k_val: i32, current_net_wins_val: i32) -> RankStats {
    calculate_rank_stats_formulas_internal(p, k_val, current_net_wins_val)
}

#[wasm_bindgen]
pub fn calculate_rank_progress_stats(
    p: f64,
    k_val: i32,
    current_net_wins_val: i32,
    current_subtier_val: i32,
) -> RankProgressStats {
    if !(k_val == 4 || k_val == 5) {
        return RankProgressStats {
            leave_current_segment_expected: f64::INFINITY,
            current_segment_promotion_probability: f64::NAN,
            expected_to_current_tier_i: f64::INFINITY,
            expected_to_next_big_tier_v: f64::INFINITY,
        };
    }

    if !(0..=4).contains(&current_subtier_val) {
        return RankProgressStats {
            leave_current_segment_expected: f64::INFINITY,
            current_segment_promotion_probability: f64::NAN,
            expected_to_current_tier_i: f64::INFINITY,
            expected_to_next_big_tier_v: f64::INFINITY,
        };
    }

    let current_stats = if current_subtier_val == 0 {
        calculate_v_floor_segment_stats_internal(p, k_val, current_net_wins_val)
    } else {
        calculate_rank_stats_formulas_internal(p, k_val, current_net_wins_val)
    };
    let base_stats = calculate_rank_stats_formulas_internal(p, k_val, 0);
    let v_floor_base_stats = calculate_v_floor_segment_stats_internal(p, k_val, 0);
    let current_tier = current_subtier_val as usize;

    let expected_to_current_tier_i =
        compute_expected_to_current_tier_i(
            current_tier,
            &current_stats,
            &base_stats,
            &v_floor_base_stats,
        );
    let expected_to_next_big_tier_v =
        compute_expected_to_next_big_tier_v(
            current_tier,
            &current_stats,
            &base_stats,
            &v_floor_base_stats,
        );

    RankProgressStats {
        leave_current_segment_expected: current_stats.expected_matches,
        current_segment_promotion_probability: current_stats.promotion_probability,
        expected_to_current_tier_i,
        expected_to_next_big_tier_v,
    }
}

fn calculate_rank_stats_formulas_internal(p: f64, k_val: i32, current_net_wins_val: i32) -> RankStats {
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

    let expected_matches = calculate_non_floor_segment_expected_internal(p, k_val, current_net_wins_val);

    RankStats {
        expected_matches: expected_matches.max(0.0),
        promotion_probability: promotion_probability.max(0.0).min(1.0),
    }
}

fn calculate_non_floor_segment_expected_internal(p: f64, k_val: i32, current_net_wins_val: i32) -> f64 {
    let start = current_net_wins_val.clamp(-2, k_val - 1);
    let q = 1.0 - p;

    // Transient states are -2, -1, 0, ..., k-1.
    let mut states = Vec::with_capacity((k_val + 2) as usize);
    for s in -2..=k_val - 1 {
        states.push(s);
    }

    let size = states.len();
    let mut matrix = vec![vec![0.0; size]; size];
    let vector = vec![1.0; size];

    for (row, &state) in states.iter().enumerate() {
        matrix[row][row] = 1.0;

        let next_win = if state < 0 { 1 } else { state + 1 };
        if (-2..=k_val - 1).contains(&next_win) {
            let col = (next_win + 2) as usize;
            matrix[row][col] -= p;
        }

        let next_lose = state - 1;
        if (-2..=k_val - 1).contains(&next_lose) {
            let col = (next_lose + 2) as usize;
            matrix[row][col] -= q;
        }
    }

    solve_linear_system(&matrix, &vector)
        .and_then(|solved| solved.get((start + 2) as usize).copied())
        .unwrap_or(f64::INFINITY)
}

fn calculate_v_floor_segment_stats_internal(p: f64, k_val: i32, current_net_wins_val: i32) -> RankStats {
    let start = current_net_wins_val.clamp(0, k_val - 1) as usize;

    if p <= 0.0 {
        return RankStats {
            expected_matches: f64::INFINITY,
            promotion_probability: 0.0,
        };
    }

    if p >= 1.0 {
        return RankStats {
            expected_matches: (k_val - start as i32) as f64,
            promotion_probability: 1.0,
        };
    }

    let q = 1.0 - p;
    let size = k_val as usize;
    let mut matrix = vec![vec![0.0; size]; size];
    let vector = vec![1.0; size];

    for i in 0..size {
        if i == 0 {
            matrix[i][0] = p;
            if size > 1 {
                matrix[i][1] = -p;
            }
            continue;
        }

        matrix[i][i] = 1.0;
        matrix[i][i - 1] = -q;
        if i + 1 < size {
            matrix[i][i + 1] = -p;
        }
    }

    let expected_matches = solve_linear_system(&matrix, &vector)
        .and_then(|solved| solved.get(start).copied())
        .unwrap_or(f64::INFINITY);

    RankStats {
        expected_matches,
        promotion_probability: 1.0,
    }
}

fn solve_linear_system(matrix: &[Vec<f64>], vector: &[f64]) -> Option<Vec<f64>> {
    let n = matrix.len();
    if n == 0 || vector.len() != n {
        return None;
    }

    let mut a = matrix.to_vec();
    let mut b = vector.to_vec();

    for col in 0..n {
        let mut pivot = col;
        for row in (col + 1)..n {
            if a[row][col].abs() > a[pivot][col].abs() {
                pivot = row;
            }
        }

        if a[pivot][col].abs() < EPS {
            return None;
        }

        a.swap(col, pivot);
        b.swap(col, pivot);

        let pivot_value = a[col][col];
        for j in col..n {
            a[col][j] /= pivot_value;
        }
        b[col] /= pivot_value;

        for row in 0..n {
            if row == col {
                continue;
            }

            let factor = a[row][col];
            if factor.abs() < EPS {
                continue;
            }

            for j in col..n {
                a[row][j] -= factor * a[col][j];
            }
            b[row] -= factor * b[col];
        }
    }

    Some(b)
}

fn compute_expected_to_current_tier_i(
    current_tier: usize,
    current_stats: &RankStats,
    base_stats: &RankStats,
    v_floor_base_stats: &RankStats,
) -> f64 {
    if current_tier >= 4 {
        return 0.0;
    }

    if base_stats.promotion_probability <= EPS {
        return f64::INFINITY;
    }

    // State order: V, IV, III, II. I is absorbing target.
    let size = 4;
    let mut matrix = vec![vec![0.0; size]; size];
    let mut vector = vec![0.0; size];

    for tier in 0..size {
        let stats = if tier == current_tier {
            current_stats
        } else if tier == 0 {
            v_floor_base_stats
        } else {
            base_stats
        };
        let up = stats.promotion_probability;
        let down = 1.0 - up;

        vector[tier] = stats.expected_matches;

        if tier == 0 {
            matrix[tier][tier] = up;
            matrix[tier][tier + 1] = -up;
            continue;
        }

        matrix[tier][tier] = 1.0;
        matrix[tier][tier - 1] = -down;
        if tier + 1 < size {
            matrix[tier][tier + 1] = -up;
        }
    }

    solve_linear_system(&matrix, &vector)
        .and_then(|solved| solved.get(current_tier).copied())
        .unwrap_or(f64::INFINITY)
}

fn compute_expected_to_next_big_tier_v(
    current_tier: usize,
    current_stats: &RankStats,
    base_stats: &RankStats,
    v_floor_base_stats: &RankStats,
) -> f64 {
    if base_stats.promotion_probability <= EPS {
        return f64::INFINITY;
    }

    // State order: V, IV, III, II, I. Next big-tier V is absorbing target.
    let size = 5;
    let mut matrix = vec![vec![0.0; size]; size];
    let mut vector = vec![0.0; size];

    for tier in 0..size {
        let stats = if tier == current_tier {
            current_stats
        } else if tier == 0 {
            v_floor_base_stats
        } else {
            base_stats
        };
        let up = stats.promotion_probability;
        let down = 1.0 - up;

        vector[tier] = stats.expected_matches;

        if tier == 0 {
            matrix[tier][tier] = up;
            matrix[tier][tier + 1] = -up;
            continue;
        }

        matrix[tier][tier] = 1.0;
        matrix[tier][tier - 1] = -down;
        if tier < 4 {
            matrix[tier][tier + 1] = -up;
        }
    }

    solve_linear_system(&matrix, &vector)
        .and_then(|solved| solved.get(current_tier).copied())
        .unwrap_or(f64::INFINITY)
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

#[cfg(all(test, not(target_arch = "wasm32")))]
mod tests {
    use super::*;
    use rand::{Rng, SeedableRng};
    use rand_chacha::ChaCha8Rng;

    const TRIALS: usize = 1_000_000;
    const EXPECTED_REL_TOLERANCE: f64 = 0.02;
    const PROBABILITY_REL_TOLERANCE: f64 = 0.01;
    const EXPECTED_ABS_FLOOR: f64 = 1e-6;
    const PROBABILITY_ABS_FLOOR: f64 = 1e-6;
    const MAX_STEPS: usize = 1_000_000;

    #[derive(Clone, Copy)]
    struct Case {
        p: f64,
        k: i32,
        net: i32,
        tier: i32,
        seed: u64,
    }

    fn assert_relative_close(
        metric: &str,
        mc: f64,
        exact: f64,
        rel_tolerance: f64,
        abs_floor: f64,
    ) {
        let diff = (mc - exact).abs();
        let allowed = (exact.abs() * rel_tolerance).max(abs_floor);
        assert!(
            diff <= allowed,
            "{} mismatch: mc={:.6}, exact={:.6}, diff={:.6}, allowed={:.6}",
            metric,
            mc,
            exact,
            diff,
            allowed
        );
    }

    fn simulate_segment_once<R: Rng>(
        rng: &mut R,
        p: f64,
        k: i32,
        start_net: i32,
        v_floor: bool,
    ) -> (usize, bool) {
        let mut matches = 0usize;

        if v_floor {
            let mut state = start_net.clamp(0, k - 1);
            for _ in 0..MAX_STEPS {
                if state >= k {
                    return (matches, true);
                }

                matches += 1;
                let win = rng.gen_bool(p);
                if win {
                    if state == 0 {
                        state = 1;
                    } else {
                        state += 1;
                    }
                } else if state > 0 {
                    state -= 1;
                }
            }

            panic!("segment simulation exceeded max-steps in v-floor mode");
        }

        let mut state = start_net.clamp(-2, k - 1);
        for _ in 0..MAX_STEPS {
            if state >= k {
                return (matches, true);
            }
            if state <= -3 {
                return (matches, false);
            }

            matches += 1;
            let win = rng.gen_bool(p);
            if win {
                if state < 0 {
                    state = 1;
                } else {
                    state += 1;
                }
            } else {
                state -= 1;
            }
        }

        panic!("segment simulation exceeded max-steps in non-floor mode");
    }

    fn simulate_to_current_tier_i_once<R: Rng>(rng: &mut R, case: Case) -> usize {
        if case.tier >= 4 {
            return 0;
        }

        let mut tier = case.tier;
        let mut first = true;
        let mut total = 0usize;

        for _ in 0..MAX_STEPS {
            if tier >= 4 {
                return total;
            }

            let start_net = if first { case.net } else { 0 };
            first = false;

            let (m, promoted) =
                simulate_segment_once(rng, case.p, case.k, start_net, tier == 0);
            total += m;

            if promoted {
                tier += 1;
            } else if tier > 0 {
                tier -= 1;
            }
        }

        panic!("simulate_to_current_tier_i_once exceeded max-steps");
    }

    fn simulate_to_next_big_tier_v_once<R: Rng>(rng: &mut R, case: Case) -> usize {
        let mut tier = case.tier;
        let mut first = true;
        let mut total = 0usize;

        for _ in 0..MAX_STEPS {
            let start_net = if first { case.net } else { 0 };
            first = false;

            let (m, promoted) =
                simulate_segment_once(rng, case.p, case.k, start_net, tier == 0);
            total += m;

            if promoted {
                if tier >= 4 {
                    return total;
                }
                tier += 1;
            } else if tier > 0 {
                tier -= 1;
            }
        }

        panic!("simulate_to_next_big_tier_v_once exceeded max-steps");
    }

    fn monte_carlo_validate_case(case: Case) {
        let exact = calculate_rank_progress_stats(case.p, case.k, case.net, case.tier);
        let mut rng = ChaCha8Rng::seed_from_u64(case.seed);

        let mut leave_sum = 0.0;
        let mut promote_count = 0.0;
        let mut to_i_sum = 0.0;
        let mut to_next_v_sum = 0.0;

        for _ in 0..TRIALS {
            let (leave_m, promoted) =
                simulate_segment_once(&mut rng, case.p, case.k, case.net, case.tier == 0);
            leave_sum += leave_m as f64;
            if promoted {
                promote_count += 1.0;
            }

            to_i_sum += simulate_to_current_tier_i_once(&mut rng, case) as f64;
            to_next_v_sum += simulate_to_next_big_tier_v_once(&mut rng, case) as f64;
        }

        let leave_mc = leave_sum / TRIALS as f64;
        let promote_mc = promote_count / TRIALS as f64;
        let to_i_mc = to_i_sum / TRIALS as f64;
        let to_next_v_mc = to_next_v_sum / TRIALS as f64;

        assert_relative_close(
            "leave_expected",
            leave_mc,
            exact.leave_current_segment_expected,
            EXPECTED_REL_TOLERANCE,
            EXPECTED_ABS_FLOOR,
        );
        assert_relative_close(
            "promotion_probability",
            promote_mc,
            exact.current_segment_promotion_probability,
            PROBABILITY_REL_TOLERANCE,
            PROBABILITY_ABS_FLOOR,
        );
        assert_relative_close(
            "to_current_tier_i",
            to_i_mc,
            exact.expected_to_current_tier_i,
            EXPECTED_REL_TOLERANCE,
            EXPECTED_ABS_FLOOR,
        );
        assert_relative_close(
            "to_next_big_tier_v",
            to_next_v_mc,
            exact.expected_to_next_big_tier_v,
            EXPECTED_REL_TOLERANCE,
            EXPECTED_ABS_FLOOR,
        );
    }

    #[test]
    fn monte_carlo_matches_formulas_k4() {
        monte_carlo_validate_case(Case {
            p: 0.6,
            k: 4,
            net: 0,
            tier: 0,
            seed: 114_514,
        });
    }

    #[test]
    fn monte_carlo_matches_formulas_k5() {
        monte_carlo_validate_case(Case {
            p: 0.52,
            k: 5,
            net: 0,
            tier: 4,
            seed: 9,
        });
    }
}
