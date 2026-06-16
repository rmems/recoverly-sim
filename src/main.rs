//! recoverly-sim CLI binary
use anyhow::Result;
use clap::{Parser, Subcommand};
use recoverly_sim::*;
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "recoverly-sim", version, about = "Brain injury recovery simulation (Ji Lab CNN / custom models + stochastic recovery)")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Generate or load kinematics and estimate strain metrics (synthetic or Ji bridge stub)
    EstimateStrain {
        /// Use built-in synthetic pulse instead of file
        #[arg(long)]
        synthetic: bool,
        #[arg(long, default_value_t = 25.0)]
        peak_rot: f64,
        #[arg(long, default_value_t = 25.0)]
        duration_ms: f64,
        /// Optional path to Ji-style preprocessed .mat or JSON (future)
        #[arg(long)]
        from_ji: Option<PathBuf>,
        #[arg(long)]
        seed: Option<u64>,
    },
    /// Run Monte-Carlo recovery simulation from strain or synthetic impact
    Simulate {
        #[arg(long)]
        kinematics: Option<PathBuf>, // future: load json kinematics
        /// Generate synthetic with this peak rotational vel (rad/s)
        #[arg(long, default_value_t = 22.0)]
        peak_rot: f64,
        #[arg(long, default_value_t = 25.0)]
        duration_ms: f64,
        #[arg(long, default_value_t = 28.0)]
        age: f64,
        #[arg(long, default_value_t = 0.85)]
        adherence: f64,
        #[arg(long, default_value_t = 0.85)]
        sleep: f64,
        #[arg(long, default_value_t = 0)]
        prior: u32,
        #[arg(long, value_enum, default_value_t = ProtocolArg::Standard)]
        protocol: ProtocolArg,
        #[arg(long, default_value_t = 300)]
        mc_runs: usize,
        #[arg(long, default_value_t = 400)]
        max_days: u32,
        #[arg(long)]
        seed: Option<u64>,
        #[arg(long)]
        export: Option<PathBuf>,
    },
    /// List built-in / available "injury generators" (synthetic + Ji stubs)
    ListGenerators {},
}

#[derive(clap::ValueEnum, Clone, Copy)]
enum ProtocolArg {
    Conservative,
    Standard,
    Aggressive,
}

impl From<ProtocolArg> for Protocol {
    fn from(p: ProtocolArg) -> Self {
        match p {
            ProtocolArg::Conservative => Protocol::Conservative,
            ProtocolArg::Standard => Protocol::Standard,
            ProtocolArg::Aggressive => Protocol::Aggressive,
        }
    }
}

fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let cli = Cli::parse();

    match cli.command {
        Commands::EstimateStrain {
            synthetic,
            peak_rot,
            duration_ms,
            from_ji,
            seed,
        } => {
            let kin = if synthetic || from_ji.is_none() {
                synthetic_kinematics(peak_rot, duration_ms, seed)
            } else {
                // TODO: real Ji .mat loader
                eprintln!("Ji .mat loader not yet implemented — falling back to synthetic");
                synthetic_kinematics(peak_rot, duration_ms, seed)
            };
            let strain = if from_ji.is_some() {
                // stub: in real would call ji_bridge
                eprintln!("(stub) pretending to call Ji Lab CNN on provided input");
                synthetic_strain_from_kinematics(&kin)
            } else {
                synthetic_strain_from_kinematics(&kin)
            };
            println!("{}", serde_json::to_string_pretty(&strain)?);
        }
        Commands::Simulate {
            kinematics: _,
            peak_rot,
            duration_ms,
            age,
            adherence,
            sleep,
            prior,
            protocol,
            mc_runs,
            max_days,
            seed,
            export,
        } => {
            let kin = synthetic_kinematics(peak_rot, duration_ms, seed);
            let strain = synthetic_strain_from_kinematics(&kin);
            let dmg = damage_from_strain(&strain);
            let mods = RecoveryModifiers {
                age,
                adherence,
                sleep_quality: sleep,
                prior_concussions: prior,
                protocol: protocol.into(),
            };
            let cfg = SimConfig {
                mc_runs,
                max_days,
                seed,
            };
            let traces = run_mc_recovery(&dmg, &mods, &cfg);
            let summary = summarize(&traces, strain, dmg.clone());

            // pretty table
            use comfy_table::Table;
            let mut table = Table::new();
            table.set_header(vec!["metric", "median", "p05", "p95"]);
            table.add_row(vec![
                "days_to_80pct".into(),
                format!("{:.0}", summary.median_days_80pct),
                format!("{:.0}", summary.p05_days_80pct),
                format!("{:.0}", summary.p95_days_80pct),
            ]);
            table.add_row(vec![
                "median_setbacks".into(),
                format!("{:.1}", summary.median_setbacks),
                "-".into(),
                "-".into(),
            ]);
            println!("{table}");
            println!(
                "initial_deficit={:.3}  severity={}",
                dmg.initial_deficit, dmg.severity_tag
            );

            if let Some(dir) = export {
                std::fs::create_dir_all(&dir)?;
                let ts = chrono::Utc::now().format("%Y%m%dT%H%M%S").to_string();
                let base = dir.join(format!("recoverly_{}", ts));
                std::fs::write(
                    base.with_extension("summary.json"),
                    serde_json::to_string_pretty(&summary)?,
                )?;
                // write a few traces as csv for demo
                let mut wtr = csv::Writer::from_path(base.with_extension("traces_sample.csv"))?;
                wtr.write_record(["run_id", "day", "function"])?;
                for tr in traces.iter().take(5) {
                    for (d, f) in tr.daily_function.iter().enumerate() {
                        wtr.write_record(&[tr.run_id.to_string(), d.to_string(), f.to_string()])?;
                    }
                }
                wtr.flush()?;
                println!("Wrote artifacts under {}", dir.display());
            }
        }
        Commands::ListGenerators {} => {
            println!("synthetic (built-in pulse generator)");
            println!("ji-regional (stub — will call Jilab-biomechanics/CNN-brain-strains)");
            println!("ji-distrib (stub — will call Jilab-biomechanics/CNN-estimation-of-brain-strain-distribution)");
            println!("(custom PINN/GNN can supply equivalent StrainMetrics)");
        }
    }
    Ok(())
}
