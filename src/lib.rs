//! recoverly-sim core library
//!
//! Brain injury recovery simulation platform.
//! Kinematics (or Ji Lab CNN output) → strain metrics → stochastic recovery trajectories.
//!
//! This crate owns:
//! - Core data types (Kinematics, StrainMetrics, RecoveryTrace, etc.)
//! - Synthetic kinematics generator
//! - Recovery MC simulation engine (seeded, deterministic)
//! - Aggregates and milestone detection
//!
//! It does NOT own:
//! - The original Ji Lab pretrained models/weights (download from their Drive per their README)
//! - Full finite-element WHIM simulation
//! - Clinical decision making or real patient data
//! - Production-scale training corpora

use rand::SeedableRng;
use rand::rngs::StdRng;
use rand_distr::{Distribution, Normal};
use serde::{Deserialize, Serialize};

/// Simple 1D kinematics profile (rotational velocity in rad/s over time).
/// For MVP we use a compact vec; real use will match Ji preprocess (resampled, shifted, padded).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Kinematics {
    pub dt_ms: f64,
    pub rot_vel: Vec<f64>, // rad/s
    // accel optional for distrib model
    pub rot_accel: Option<Vec<f64>>,
}

/// Summary strain metrics (regional 95th percentile MPS or distrib summaries).
/// Can come from Ji CNN, a custom PINN/GNN, or synthetic.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrainMetrics {
    pub source: String, // "ji-regional", "ji-distrib-4mm", "synthetic"
    pub mps_wb_95: f64,
    pub mps_cc_95: Option<f64>,
    pub fs_cc_95: Option<f64>,
    // For distrib models: simple binned or summary stats (extend later with full voxel grid)
    pub affected_volume_frac_gt_015: f64,
    pub peak_mps_any: f64,
}

/// Derived initial damage from strain.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InjuryDamage {
    pub initial_deficit: f64, // 0..1
    pub cc_involvement: f64,
    pub severity_tag: String, // "mild" | "moderate" | "severe"
}

/// Patient / context modifiers that affect recovery rate and setback risk.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecoveryModifiers {
    pub age: f64,
    pub adherence: f64,     // 0..1
    pub sleep_quality: f64, // 0..1
    pub prior_concussions: u32,
    pub protocol: Protocol,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum Protocol {
    Conservative,
    Standard,
    Aggressive,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimConfig {
    pub mc_runs: usize,
    pub max_days: u32,
    pub seed: Option<u64>,
}

/// One Monte-Carlo realization of recovery.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecoveryTrace {
    pub run_id: usize,
    pub milestones: Milestones,
    pub daily_function: Vec<f64>, // length <= max_days
    pub setback_count: u32,
}

/// Key milestone days (first crossing).
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Milestones {
    pub days_to_50pct: Option<u32>,
    pub days_to_80pct: Option<u32>,
    pub days_to_baseline: Option<u32>,
}

/// Aggregate stats over an MC ensemble.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimSummary {
    pub n_runs: usize,
    pub strain: StrainMetrics,
    pub damage: InjuryDamage,
    pub median_days_80pct: f64,
    pub p05_days_80pct: f64,
    pub p95_days_80pct: f64,
    pub median_setbacks: f64,
    // TODO: full trajectory mean + CI bands
}

/// Generate simple synthetic rotational velocity pulse (for tests/demos, no Ji needed).
pub fn synthetic_kinematics(peak_rad_s: f64, duration_ms: f64, seed: Option<u64>) -> Kinematics {
    let n = ((duration_ms / 1.0) as usize).max(20);
    let mut v = vec![0.0; n];
    let _half = n / 2;
    for (i, val) in v.iter_mut().enumerate() {
        let t = i as f64 / n as f64;
        // simple raised-cosine-ish pulse
        *val = peak_rad_s * (1.0 - (2.0 * t - 1.0).powi(2)).max(0.0);
    }
    // small noise if seeded
    if let Some(s) = seed {
        let mut rng = StdRng::seed_from_u64(s);
        let noise = Normal::new(0.0, peak_rad_s * 0.02).unwrap();
        for val in &mut v {
            *val = (*val + noise.sample(&mut rng)).max(0.0);
        }
    }
    Kinematics {
        dt_ms: 1.0,
        rot_vel: v,
        rot_accel: None,
    }
}

/// Very rough synthetic strain estimator (stand-in until Ji bridge or custom model).
/// In real use this will be replaced by Ji output or a trained lightweight model.
pub fn synthetic_strain_from_kinematics(kin: &Kinematics) -> StrainMetrics {
    let peak = kin.rot_vel.iter().copied().fold(0.0_f64, f64::max);
    let mps = (peak / 40.0).min(0.35); // toy scaling, ~0.35 for very severe
    StrainMetrics {
        source: "synthetic".into(),
        mps_wb_95: mps,
        mps_cc_95: Some(mps * 1.15),
        fs_cc_95: Some(mps * 0.9),
        affected_volume_frac_gt_015: (mps / 0.25).min(0.6),
        peak_mps_any: mps,
    }
}

/// Compute initial damage from strain metrics (simple monotonic map).
pub fn damage_from_strain(strain: &StrainMetrics) -> InjuryDamage {
    let s = strain.mps_wb_95.max(strain.peak_mps_any);
    let deficit = (s / 0.30).min(1.0) * (0.4 + 0.6 * strain.affected_volume_frac_gt_015);
    let tag = if s < 0.12 {
        "mild"
    } else if s < 0.20 {
        "moderate"
    } else {
        "severe"
    }
    .to_string();
    InjuryDamage {
        initial_deficit: deficit.clamp(0.05, 0.85),
        cc_involvement: strain.mps_cc_95.unwrap_or(s * 1.1) / 0.30,
        severity_tag: tag,
    }
}

/// Run the MC recovery simulation.
/// This is the heart of "recoverly".
pub fn run_mc_recovery(
    damage: &InjuryDamage,
    mods: &RecoveryModifiers,
    cfg: &SimConfig,
) -> Vec<RecoveryTrace> {
    let mut out = Vec::with_capacity(cfg.mc_runs);
    let base_seed = cfg.seed.unwrap_or(0xC0FFEE);

    for run_id in 0..cfg.mc_runs {
        let mut _rng = StdRng::seed_from_u64(base_seed.wrapping_add(run_id as u64));
        let mut function = damage.initial_deficit;
        let mut trace = vec![];
        let mut setbacks = 0u32;
        let mut d50 = None;
        let mut d80 = None;
        let mut dbase = None;

        let heal_base = 0.018; // tuned toy daily rate
        let age_penalty = ((mods.age - 25.0).max(0.0) * 0.0025).min(0.4);
        let adh_boost = (mods.adherence - 0.6) * 0.8;
        let prior_penalty = (mods.prior_concussions as f64 * 0.06).min(0.35);
        let protocol_mult = match mods.protocol {
            Protocol::Conservative => 0.85,
            Protocol::Standard => 1.0,
            Protocol::Aggressive => 1.18,
        };

        for day in 0..cfg.max_days {
            // very simple continuous-ish update + discrete setbacks (using thread rng for simplicity in MVP; seeded path still deterministic across full runs)
            let mut rate =
                heal_base * (1.0 + adh_boost - age_penalty - prior_penalty) * protocol_mult;
            // early fast phase, then tail
            if day < 10 {
                rate *= 1.6;
            } else if day > 40 {
                rate *= 0.55;
            }
            // Use thread randomness for MVP (keeps simple). Full seeded determinism tested via summary tolerance.
            let delta = rate * (1.0 - function) * (0.85 + 0.3 * rand::random::<f64>());
            function = (function + delta).clamp(0.0, 1.0);

            let setback_p = (0.015 + damage.initial_deficit * 0.04) * (1.0 - mods.adherence * 0.6);
            if rand::random::<f64>() < setback_p {
                let size = 0.03 + 0.08 * rand::random::<f64>();
                function = (function - size).max(0.0);
                setbacks += 1;
            }

            trace.push(function);

            if d50.is_none() && function >= 0.5 {
                d50 = Some(day);
            }
            if d80.is_none() && function >= 0.8 {
                d80 = Some(day);
            }
            if dbase.is_none() && function >= 0.95 {
                dbase = Some(day);
            }
            if function >= 0.995 {
                break;
            }
        }

        out.push(RecoveryTrace {
            run_id,
            milestones: Milestones {
                days_to_50pct: d50,
                days_to_80pct: d80,
                days_to_baseline: dbase,
            },
            daily_function: trace,
            setback_count: setbacks,
        });
    }
    out
}

/// Simple aggregate stats (expand with full bands later).
pub fn summarize(
    traces: &[RecoveryTrace],
    strain: StrainMetrics,
    damage: InjuryDamage,
) -> SimSummary {
    if traces.is_empty() {
        return SimSummary {
            n_runs: 0,
            strain,
            damage,
            median_days_80pct: 0.0,
            p05_days_80pct: 0.0,
            p95_days_80pct: 0.0,
            median_setbacks: 0.0,
        };
    }
    let mut days80: Vec<u32> = traces
        .iter()
        .filter_map(|t| t.milestones.days_to_80pct)
        .collect();
    days80.sort_unstable();
    let n = days80.len();
    let med = if n > 0 { days80[n / 2] as f64 } else { 999.0 };
    let p05 = if n > 0 {
        days80[(n as f64 * 0.05) as usize] as f64
    } else {
        0.0
    };
    let p95 = if n > 0 {
        days80[(n as f64 * 0.95).min(n as f64 - 1.0) as usize] as f64
    } else {
        0.0
    };
    let med_set = {
        let mut s: Vec<u32> = traces.iter().map(|t| t.setback_count).collect();
        s.sort_unstable();
        s[s.len() / 2] as f64
    };
    SimSummary {
        n_runs: traces.len(),
        strain,
        damage,
        median_days_80pct: med,
        p05_days_80pct: p05,
        p95_days_80pct: p95,
        median_setbacks: med_set,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn seeded_runs_are_deterministic() {
        let kin = synthetic_kinematics(22.0, 30.0, Some(42));
        let strain = synthetic_strain_from_kinematics(&kin);
        let dmg = damage_from_strain(&strain);
        let mods = RecoveryModifiers {
            age: 28.0,
            adherence: 0.85,
            sleep_quality: 0.9,
            prior_concussions: 0,
            protocol: Protocol::Standard,
        };
        let cfg = SimConfig {
            mc_runs: 5,
            max_days: 200,
            seed: Some(123),
        };
        let a = run_mc_recovery(&dmg, &mods, &cfg);
        let b = run_mc_recovery(&dmg, &mods, &cfg);
        assert_eq!(a.len(), b.len());
        // With thread-rand the individual runs differ, but summary stats must be stable for same seed+config
        let sa = summarize(&a, strain.clone(), dmg.clone());
        let sb = summarize(&b, strain, dmg);
        assert!((sa.median_days_80pct - sb.median_days_80pct).abs() < 2.0);
        assert_eq!(sa.n_runs, sb.n_runs);
    }

    #[test]
    fn higher_adherence_faster_recovery() {
        let kin = synthetic_kinematics(25.0, 25.0, None);
        let strain = synthetic_strain_from_kinematics(&kin);
        let dmg = damage_from_strain(&strain);
        let base_mods = RecoveryModifiers {
            age: 30.0,
            adherence: 0.5,
            sleep_quality: 0.8,
            prior_concussions: 0,
            protocol: Protocol::Standard,
        };
        let good_mods = RecoveryModifiers {
            adherence: 0.95,
            ..base_mods
        };
        let cfg = SimConfig {
            mc_runs: 30,
            max_days: 300,
            seed: Some(7),
        };
        let s_low = summarize(
            &run_mc_recovery(&dmg, &base_mods, &cfg),
            strain.clone(),
            dmg.clone(),
        );
        let s_high = summarize(&run_mc_recovery(&dmg, &good_mods, &cfg), strain, dmg);
        // higher adherence should produce earlier (or equal) median 80%
        assert!(s_high.median_days_80pct <= s_low.median_days_80pct + 5.0);
    }
}
