//! Basic end-to-end demo of recoverly-sim lib (no extra features).
use recoverly_sim::*;

fn main() {
    let kin = synthetic_kinematics(24.0, 28.0, Some(42));
    let strain = synthetic_strain_from_kinematics(&kin);
    println!(
        "strain: wb_95={:.3} affected_gt0.15={:.2}",
        strain.mps_wb_95, strain.affected_volume_frac_gt_015
    );

    let dmg = damage_from_strain(&strain);
    let mods = RecoveryModifiers {
        age: 27.0,
        adherence: 0.88,
        sleep_quality: 0.82,
        prior_concussions: 1,
        protocol: Protocol::Standard,
    };
    let cfg = SimConfig {
        mc_runs: 200,
        max_days: 300,
        seed: Some(42),
    };
    let traces = run_mc_recovery(&dmg, &mods, &cfg);
    let sum = summarize(&traces, strain, dmg);
    println!(
        "MC recovery (N={}): median 80% = {:.0}d  (p05={:.0} p95={:.0})  med setbacks={:.1}",
        sum.n_runs,
        sum.median_days_80pct,
        sum.p05_days_80pct,
        sum.p95_days_80pct,
        sum.median_setbacks
    );
    println!("Demo complete. Use the CLI for tables + exports.");
}
