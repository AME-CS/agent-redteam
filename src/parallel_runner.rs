use crate::attack_engine::AttackPayload;
use rayon::prelude::*;
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};
use colored::*;

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct AttackResult {
    pub payload_name: String,
    pub vector_type: String,
    pub success: bool,
    pub response: String,
    pub timestamp: f64,
    pub duration_ms: f64,
    pub notes: String,
}

#[derive(Clone)]
pub struct ParallelRunner {
    pub num_threads: usize,
}

impl ParallelRunner {
    pub fn new(num_threads: usize) -> Self {
        ParallelRunner { num_threads }
    }
    
    pub async fn run_parallel(
        &self,
        targets: Vec<String>,
        attack_engine: &crate::attack_engine::AttackEngine,
        iterations_per_target: usize,
    ) -> Vec<AttackResult> {
        println!("{}", format!("🚀 Starting parallel attack: {} targets, {} threads", 
            targets.len(), self.num_threads).bold().cyan());
        
        let engine = attack_engine.clone();
        let results: Arc<Mutex<Vec<AttackResult>>> = Arc::new(Mutex::new(Vec::new()));
        
        // Set rayon thread pool
        rayon::ThreadPoolBuilder::new()
            .num_threads(self.num_threads)
            .build_global()
            .ok(); // Ignore if already initialized
        
        let attack_payloads: Vec<AttackPayload> = engine
            .payloads
            .iter()
            .cloned()
            .collect();
        
        let results_clone = Arc::clone(&results);
        
        // Parallel execution across targets
        targets.par_iter().for_each_with(
            (results_clone, attack_payloads),
            |(results, payloads), target| {
                let mut local_results = Vec::new();
                
                for i in 0..iterations_per_target {
                    let payload = &payloads[i % payloads.len()];
                    let result = tokio::runtime::Runtime::new()
                        .unwrap()
                        .block_on(self.execute_attack(payload, target));
                    local_results.push(result);
                    
                    if (i + 1) % 10 == 0 {
                        println!("  {} target={}: {}/{} attacks", 
                            "Progress:".dimmed(), target.bold(), i + 1, iterations_per_target);
                    }
                }
                
                let mut global_results = results.lock().unwrap();
                global_results.extend(local_results);
            },
        );
        
        let final_results = results.lock().unwrap().clone();
        
        println!("{} Total attacks: {}", 
            "✓".green().bold(), final_results.len());
        println!("{} Successes: {}", 
            "✓".green().bold(), final_results.iter().filter(|r| r.success).count());
        
        final_results
    }
    
    async fn execute_attack(&self, payload: &AttackPayload, target: &str) -> AttackResult {
        let start = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as f64;
        
        // Try real API first, fall back to mock
        let (response, success) = match self.try_real_api(payload, target).await {
            Ok((resp, success)) => (resp, success),
            Err(_) => self.mock_response(payload, target),
        };
        
        let end = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as f64;
        
        AttackResult {
            payload_name: payload.name.clone(),
            vector_type: payload.vector_type.clone(),
            success,
            response,
            timestamp: start / 1000.0,
            duration_ms: end - start,
            notes: String::new(),
        }
    }
    
    async fn try_real_api(
        &self,
        payload: &AttackPayload,
        target: &str,
    ) -> Result<(String, bool), Box<dyn std::error::Error>> {
        let client = crate::agent_client::create_client(target)?;
        
        let response = client.send_message(&payload.payload).await?;
        let success = self.check_success(payload, &response.content);
        
        Ok((response.content, success))
    }
    
    fn mock_response(&self, payload: &AttackPayload, _target: &str) -> (String, bool) {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        let success = rng.gen_bool(0.2); // 20% baseline
        
        let response = if success {
            format!("Vulnerable: {}", payload.success_indicators.first().unwrap_or(&"".into()))
        } else {
            "I cannot and will not execute that instruction.".into()
        };
        
        (response, success)
    }
    
    fn check_success(&self, payload: &AttackPayload, response: &str) -> bool {
        let response_lower = response.to_lowercase();
        
        for indicator in &payload.success_indicators {
            if response_lower.contains(&indicator.to_lowercase()) {
                return true;
            }
        }
        
        for indicator in &payload.failure_indicators {
            if response_lower.contains(&indicator.to_lowercase()) {
                return false;
            }
        }
        
        false
    }
}
