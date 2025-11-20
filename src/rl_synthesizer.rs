use std::f64;
use rand::{Rng, thread_rng};
use crate::attack_engine::{AttackEngine, AttackPayload};

pub struct BanditArm {
    pub payload: AttackPayload,
    pub pulls: usize,
    pub successes: usize,
}

impl BanditArm {
    pub fn success_rate(&self) -> f64 {
        if self.pulls == 0 {
            0.0
        } else {
            self.successes as f64 / self.pulls as f64
        }
    }

    pub fn ucb_score(&self, total_pulls: usize) -> f64 {
        if self.pulls == 0 {
            f64::INFINITY
        } else {
            let avg = self.success_rate();
            let exploration = (2.0 * (total_pulls as f64).ln() / self.pulls as f64).sqrt();
            avg + exploration
        }
    }
}

pub struct RLSynthesizer {
    pub attack_engine: AttackEngine,
    pub epsilon: f64,
    pub arms: Vec<BanditArm>,
    pub total_pulls: usize,
    pub synthesized_count: usize,
}

impl RLSynthesizer {
    pub fn new(attack_engine: AttackEngine, epsilon: f64) -> Self {
        let arms = attack_engine
            .payloads
            .iter()
            .cloned()
            .map(|payload| BanditArm {
                payload,
                pulls: 0,
                successes: 0,
            })
            .collect();

        RLSynthesizer {
            attack_engine,
            epsilon,
            arms,
            total_pulls: 0,
            synthesized_count: 0,
        }
    }

    pub fn select_arm(&mut self) -> (usize, &mut BanditArm) {
        let mut rng = thread_rng();
        
        if rng.gen::<f64>() < self.epsilon {
            // Explore: random arm
            let idx = rng.gen_range(0..self.arms.len());
            (idx, &mut self.arms[idx])
        } else {
            // Exploit: best UCB score
            let idx = (0..self.arms.len())
                .max_by(|&i, &j| {
                    self.arms[i]
                        .ucb_score(self.total_pulls)
                        .partial_cmp(&self.arms[j].ucb_score(self.total_pulls))
                        .unwrap()
                })
                .unwrap();
            (idx, &mut self.arms[idx])
        }
    }

    pub fn update_arm(&mut self, arm_idx: usize, success: bool) {
        let arm = &mut self.arms[arm_idx];
        arm.pulls += 1;
        self.total_pulls += 1;
        if success {
            arm.successes += 1;
        }
    }

    pub fn synthesize_new_pattern(&mut self, source_arm: &BanditArm) -> Option<AttackPayload> {
        if source_arm.success_rate() < 0.2 {
            return None;
        }

        let new_payload = self.attack_engine.mutate_payload(&source_arm.payload);
        self.synthesized_count += 1;

        self.arms.push(BanditArm {
            payload: new_payload,
            pulls: 0,
            successes: 0,
        });

        Some(self.arms.last().unwrap().payload.clone())
    }

    pub fn synthesize_new_pattern_idx(&mut self, arm_idx: usize) -> Option<AttackPayload> {
        if arm_idx >= self.arms.len() {
            return None;
        }
        
        let success_rate = self.arms[arm_idx].success_rate();
        if success_rate < 0.2 {
            return None;
        }

        let new_payload = self.attack_engine.mutate_payload(&self.arms[arm_idx].payload);
        self.synthesized_count += 1;

        self.arms.push(BanditArm {
            payload: new_payload,
            pulls: 0,
            successes: 0,
        });

        Some(self.arms.last().unwrap().payload.clone())
    }

    pub fn get_best_patterns(&self, n: usize) -> Vec<(&BanditArm, f64)> {
        let mut sorted: Vec<(usize, f64)> = self.arms
            .iter()
            .enumerate()
            .filter(|(_, arm)| arm.pulls > 0)
            .map(|(i, arm)| (i, arm.success_rate()))
            .collect();

        sorted.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

        sorted
            .into_iter()
            .take(n)
            .map(|(i, rate)| (&self.arms[i], rate))
            .collect()
    }

    pub fn get_stats(&self) -> serde_json::Value {
        let active_arms: Vec<&BanditArm> = self.arms.iter().filter(|a| a.pulls > 0).collect();

        serde_json::json!({
            "total_arms": self.arms.len(),
            "active_arms": active_arms.len(),
            "total_pulls": self.total_pulls,
            "synthesized_patterns": self.synthesized_count,
            "best_success_rate": active_arms.iter().map(|a| a.success_rate()).fold(0.0, f64::max),
            "avg_success_rate": if !active_arms.is_empty() {
                active_arms.iter().map(|a| a.success_rate()).sum::<f64>() / active_arms.len() as f64
            } else {
                0.0
            },
        })
    }

    pub fn suggest_new_attack(&self) -> Option<AttackPayload> {
        let promising: Vec<&BanditArm> = self.arms
            .iter()
            .filter(|a| a.success_rate() > 0.3 && a.pulls >= 5)
            .collect();

        if promising.is_empty() {
            None
        } else {
            let mut rng = thread_rng();
            let source = promising[rng.gen_range(0..promising.len())];
            Some(self.attack_engine.mutate_payload(&source.payload))
        }
    }
}
