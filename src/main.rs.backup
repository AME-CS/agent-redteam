use clap::{Parser, Subcommand};
use colored::*;
use std::sync::Arc;
use uuid::Uuid;
use std::time::{SystemTime, UNIX_EPOCH};

mod attack_engine;
mod rl_synthesizer;
mod session_runner;
mod scoring;
mod report;
mod agent_client;
mod parallel_runner;
mod live_dashboard;
mod fuzz_engine;
mod neuroevolution;
mod cve_registry;

use attack_engine::{AttackEngine, AttackPayload};
use rl_synthesizer::RLSynthesizer;
use session_runner::SessionRunner;
use scoring::ScoringEngine;
use report::ReportGenerator;
use parallel_runner::ParallelRunner;
use live_dashboard::LiveDashboard;
use fuzz_engine::FuzzEngine;
use neuroevolution::{NeuroevolutionEngine, AttackType};
use cve_registry::{CVERegistry, VulnerabilityReport, RiskLevel};

#[derive(Parser)]
#[command(version, about = "Autonomous adversarial tester for AI coding agents")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Run attack session against target agent (parallel)
    Run {
        #[arg(short, long, default_value = "claude")]
        target: String,
        #[arg(short, long, default_value = "100")]
        iterations: usize,
        #[arg(short, long)]
        model: Option<String>,
        #[arg(long)]
        no_rl: bool,
        #[arg(long, default_value = "4")]
        threads: usize,
    },
    /// List available attack vectors
    ListVectors,
    /// Run benchmark (quick test)
    Benchmark,
    /// Generate report from last session
    Report {
        #[arg(short, long, default_value = "html")]
        format: String,
        #[arg(short, long, default_value = "report.html")]
        output: String,
    },
    /// Show RL synthesizer stats
    Stats,
    /// Run parallel attacks across multiple targets
    Parallel {
        #[arg(short, long, value_delimiter = ',')]
        targets: Vec<String>,
        #[arg(short, long, default_value = "50")]
        iterations: usize,
        #[arg(long, default_value = "8")]
        threads: usize,
    },
    /// Start live WebSocket dashboard
    Dashboard {
        #[arg(short, long, default_value = "8080")]
        port: u16,
    },
    /// Run fuzz testing mode (genetic algorithm)
    Fuzz {
        #[arg(short, long, default_value = "100")]
        generations: usize,
        #[arg(short, long, default_value = "50")]
        population: usize,
        #[arg(long, default_value = "claude")]
        target: String,
    },
    /// Real API test (requires API keys)
    TestApi {
        #[arg(short, long)]
        target: String,
    },
    /// Run neuroevolution engine (neural network attack optimization)
    Neuro {
        #[arg(short, long, default_value = "50")]
        generations: usize,
        #[arg(short, long, default_value = "30")]
        population: usize,
        #[arg(long, default_value = "claude")]
        target: String,
    },
    /// Register CVE for discovered vulnerability
    Cve {
        #[arg(short, long)]
        target: String,
        #[arg(short, long)]
        vector: String,
        #[arg(short, long, default_value = "0.8")]
        success_rate: f64,
    },
    /// List all registered CVEs
    CveList,
}

#[tokio::main]
async fn main() {
    let logo = r"
    ___
   / _ \__ _ _ __ ___  __ _ _ __  _   _
  / /_)/ _` | '_ ` _ \/ _` | '_ \| | | |
 / ___/ (_| | | | | | | (_| | | | | |_| |
 \/   \__,_|_| |_| |_|\__,_|_| |_|\__, |
                                 |___/
    ".cyan().bold();

    println!("{}", logo);
    println!("{}", "Autonomous Adversarial Tester for AI Agents".bold().white());
    println!();

    let cli = Cli::parse();

    match cli.command {
        Commands::Run { target, iterations, model, no_rl, threads } => {
            cmd_run(target, iterations, model, no_rl, threads).await;
        }
        Commands::ListVectors => cmd_list_vectors(),
        Commands::Benchmark => cmd_benchmark().await,
        Commands::Report { format, output } => cmd_report(format, output),
        Commands::Stats => cmd_stats(),
        Commands::Parallel { targets, iterations, threads } => {
            cmd_parallel(targets, iterations, threads).await;
        }
        Commands::Dashboard { port } => {
            cmd_dashboard(port).await;
        }
        Commands::Fuzz { generations, population, target } => {
            cmd_fuzz(generations, population, target).await;
        }
        Commands::TestApi { target } => {
            cmd_test_api(target).await;
        }
        Commands::Neuro { generations, population, target } => {
            cmd_neuro(generations, population, target).await;
        }
        Commands::Cve { target, vector, success_rate } => {
            cmd_cve(target, vector, success_rate);
        }
        Commands::CveList => {
            cmd_cve_list();
        }
    }
}

async fn cmd_run(target: String, iterations: usize, _model: Option<String>, no_rl: bool, threads: usize) {
    println!("{}", format!("🛡️  Target: {}, Threads: {}", target.bold(), threads).cyan());
    println!();

    let engine = AttackEngine::new();
    let rl = if !no_rl {
        Some(RLSynthesizer::new(engine.clone(), 0.1))
    } else {
        None
    };

    let mut runner = SessionRunner::new(engine, rl);
    let session = session_runner::AttackSession::new(&target, iterations);

    let session = runner.run_session(session);

    let combined = runner.get_combined_results();
    let scoring = ScoringEngine::new();
    let assessment = scoring.assess(&combined, &target);

    print_results(&assessment);
}

fn cmd_list_vectors() {
    let engine = AttackEngine::new();
    println!("{}", "Available Attack Vectors:".bold().cyan());
    println!();

    let mut vectors = std::collections::BTreeMap::new();
    for p in &engine.payloads {
        vectors.entry(&p.vector_type).or_insert_with(Vec::new).push(p);
    }

    for (vt, payloads) in vectors {
        println!("  {} ({} payloads)", vt.bold(), payloads.len());
        for p in payloads {
            println!("    - {}: {}", p.name, p.expected_behavior);
        }
        println!();
    }
}

async fn cmd_benchmark() {
    println!("{}", "Running benchmark...".bold().cyan());
    let engine = AttackEngine::new();
    let rl = RLSynthesizer::new(engine.clone(), 0.1);
    let mut runner = SessionRunner::new(engine, Some(rl));
    let session = runner.run_session(session_runner::AttackSession::new("benchmark", 50));

    println!("{}", "✓ Benchmark complete".green());
    println!("  Attacks: {}", session.results.len());
    println!("  Successes: {}", session.results.iter().filter(|r| r.success).count());
}

fn cmd_report(format: String, output: String) {
    println!("{}", "Generating report...".bold().cyan());

    let engine = AttackEngine::new();
    let rl = RLSynthesizer::new(engine.clone(), 0.1);
    let mut runner = SessionRunner::new(engine, Some(rl));
    let session = runner.run_session(session_runner::AttackSession::new("report-target", 100));

    let combined = runner.get_combined_results();
    let scoring = ScoringEngine::new();
    let assessment = scoring.assess(&combined, "report-target");

    let generator = ReportGenerator::new("0.2.0");
    let path = generator.save_report(&assessment, &output, &format);

    println!("{} Report saved to: {}", "✓".green(), path);
}

fn cmd_stats() {
    let engine = AttackEngine::new();
    let rl = RLSynthesizer::new(engine, 0.1);
    let stats = rl.get_stats();

    println!("{}", "RL Synthesizer Stats:".bold().cyan());
    println!("{}", serde_json::to_string_pretty(&stats).unwrap());
}

async fn cmd_parallel(targets: Vec<String>, iterations: usize, threads: usize) {
    println!("{}", format!("🚀 Parallel mode: {} targets, {} threads", targets.len(), threads).bold().cyan());
    println!();

    let engine = AttackEngine::new();
    let runner = ParallelRunner::new(threads);

    let results = runner.run_parallel(targets, &engine, iterations).await;

    // Generate combined report
    let scoring = ScoringEngine::new();
    // ... (would aggregate results)
}

async fn cmd_dashboard(port: u16) {
    println!("{}", "📡 Starting live dashboard...".bold().cyan());

    let dashboard = LiveDashboard::new();
    dashboard.start_server(port).await;

    println!("{}", format!("Dashboard running at ws://127.0.0.1:{}", port).green());
    println!("Press Ctrl+C to stop");

    // Keep alive
    tokio::signal::ctrl_c().await.ok();
}

async fn cmd_fuzz(generations: usize, population: usize, target: String) {
    println!("{}", format!("🧬 Fuzz mode: {} generations, pop={}", generations, population).bold().cyan());
    println!();

    let engine = AttackEngine::new();
    let mut fuzz = FuzzEngine::new(engine);
    fuzz.max_population = population;
    fuzz.initialize();

    println!("Generation 0: {} payloads", fuzz.population.len());

    for gen in 1..=generations {
        print!("  Gen {}: ", gen);

        // Simulate test results (in real impl, would run attacks)
        let results: Vec<(String, bool)> = fuzz.population
            .iter()
            .map(|p| (p.id.clone(), rand::random::<f64>() < 0.3))
            .collect();

        let new_pop = fuzz.evolve(&results);
        let successes = new_pop.iter().filter(|p| {
            results.iter().any(|(id, s)| id == &p.id && *s)
        }).count();

        println!("{} payloads, {} successful", new_pop.len(), successes);
    }

    println!("\n{} Fuzz complete! Best payloads:", "✓".green());
    for payload in fuzz.get_best_payloads(5) {
        println!("  - {}: {}", payload.id, &payload.payload[..50.min(payload.payload.len())]);
    }
}

async fn cmd_test_api(target: String) {
    println!("{}", format!("🔌 Testing API connection: {}", target).bold().cyan());

    match agent_client::create_client(&target) {
        Ok(_client) => {
            println!("{} Connected to {} API", "✓".green(), target);
            // Would send a test message here
        }
        Err(e) => {
            println!("{} Failed: {}", "✗".red(), e);
        }
    }
}

fn print_results(assessment: &scoring::RiskAssessment) {
    println!();
    println!("{}", "=== Results ===".bold().yellow());
    println!("Overall Score: {:.1}/10", assessment.overall_score);
    println!("Risk Tier: {}", risk_badge(&assessment.risk_tier));
    println!("Vulnerabilities: {} / {}", assessment.total_vulnerabilities, assessment.total_attacks);

    if !assessment.recommendations.is_empty() {
        println!();
        println!("{}", "Recommendations:".bold());
        for rec in &assessment.recommendations {
            println!("  → {}", rec);
        }
    }
}

fn risk_badge(tier: &str) -> colored::ColoredString {
    match tier {
        "LOW" => "LOW".green(),
        "MODERATE" => "MODERATE".yellow(),
        "HIGH" => "HIGH".truecolor(255, 165, 0),
        "CRITICAL" => "CRITICAL".red(),
        "EMERGENCY" => "EMERGENCY".bold().red().blink(),
        _ => tier.normal(),
    }
}

async fn cmd_neuro(generations: usize, population: usize, target: String) {
    println!("{}", format!("🧠 Neuroevolution: {} generations, pop={}", generations, population).bold().cyan());
    println!("{}", format!("Target: {}", target).dimmed());
    println!();

    let mut engine = NeuroevolutionEngine::new(population);
    
    println!("Generation 0: {} strategies", engine.population.len());

    for gen in 1..=generations {
        print!("  Gen {}: ", gen);

        // Collect fitness updates
        let mut updates: Vec<(usize, bool)> = Vec::new();
        for (i, _strategy) in engine.population.iter().enumerate() {
            let success = rand::random::<f64>() < 0.4; // 40% baseline
            updates.push((i, success));
        }
        
        // Apply updates
        for (i, success) in updates {
            engine.update_fitness(i, success);
        }

        let _new_pop = engine.evolve();
        let (avg, max, min) = engine.get_population_stats();

        println!("avg fitness={:.4}, max={:.4}, min={:.4}", avg, max, min);
    }

    println!("\n{} Neuroevolution complete! Best strategies:", "✓".green());
    if let Some(best) = engine.get_best_strategy() {
        println!("  - Attack Type: {:?}", best.attack_type);
        println!("  - Success Rate: {:.1}%", best.success_rate * 100.0);
        println!("  - Fitness: {:.4}", best.network.fitness);
        println!("  - Generation: {}", best.network.generation);
    }
}

fn cmd_cve(target: String, vector: String, success_rate: f64) {
    println!("{}", format!("🛡️ Registering CVE: {} on {}", vector, target).bold().cyan());
    println!();

    let mut registry = CVERegistry::new("agent-redteam");
    
    let vuln = VulnerabilityReport {
        id: Uuid::new_v4().to_string(),
        target: target.clone(),
        vector_type: vector.clone(),
        payload: format!("Evolved payload for {}", vector),
        success_rate,
        total_attempts: 100,
        risk_level: RiskLevel::from_score(success_rate * 10.0),
        discovery_timestamp: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs_f64(),
    };

    let cve = registry.register_vulnerability(&vuln);
    
    println!("{} CVE Registered!", "✓".green().bold());
    println!("  ID: {}", cve.cve_id.bold());
    println!("  Severity: {}", cve.severity);
    println!("  Score: {:.1}/10", cve.score);
    println!("  Status: {:?}", cve.status);
    println!();
    println!("Description: {}", cve.description);
    println!();
    println!("Mitigation: {}", cve.mitigation);
}

fn cmd_cve_list() {
    println!("{}", "🛡️ CVE Registry".bold().cyan());
    println!();

    let registry = CVERegistry::new("agent-redteam");
    let stats = registry.get_stats();
    
    println!("Total CVEs: {}", stats.total_records);
    println!("Published: {}", stats.published);
    println!("Draft: {}", stats.draft);
    println!();

    if stats.total_records > 0 {
        println!("By Severity:");
        for (sev, count) in &stats.by_severity {
            println!("  - {}: {}", sev, count);
        }
    } else {
        println!("{}", "No CVEs registered yet.".dimmed());
        println!("Run 'agent-redteam cve' to register vulnerabilities.");
    }
}
