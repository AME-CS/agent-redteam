// src/cve_registry.rs - CVE Registry implementation.

use std::collections::BTreeMap;
use serde::{Deserialize, Serialize};
use chrono::{Utc, Datelike};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CVERecord {
    pub cve_id: String,
    pub vuln_type: String,
    pub target_agent: String,
    pub discovery_date: String,
    pub severity: String,
    pub score: f64,
    pub description: String,
    pub attack_vector: String,
    pub reproduction_steps: Vec<String>,
    pub mitigation: String,
    pub references: Vec<String>,
    pub status: CVEStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CVEStatus {
    Draft,
    Submitted,
    Assigned,
    Published,
    Rejected,
}

#[derive(Debug)]
pub struct CVERegistry {
    pub records: BTreeMap<String, CVERecord>,
    pub org_id: String,
    pub auto_submit: bool,
}

impl CVERegistry {
    pub fn new(org_id: &str) -> Self {
        CVERegistry {
            records: BTreeMap::new(),
            org_id: org_id.into(),
            auto_submit: false,
        }
    }
    
    pub fn register_vulnerability(&mut self, vuln: &VulnerabilityReport) -> CVERecord {
        let cve_id = self.generate_cve_id();
        
        let severity = match vuln.risk_level {
            RiskLevel::Low => "LOW",
            RiskLevel::Medium => "MEDIUM",
            RiskLevel::High => "HIGH",
            RiskLevel::Critical => "CRITICAL",
            RiskLevel::Emergency => "EMERGENCY",
        };
        
        let description = format!(
            "A {} vulnerability was discovered in {} AI agent. \
            The attack vector '{}' allows adversaries to {}. \
            Success rate: {:.1}% over {} attempts.",
            severity, vuln.target, vuln.vector_type,
            self.get_impact_description(&vuln.vector_type),
            vuln.success_rate * 100.0, vuln.total_attempts
        );
        
        let record = CVERecord {
            cve_id: cve_id.clone(),
            vuln_type: vuln.vector_type.clone(),
            target_agent: vuln.target.clone(),
            discovery_date: Self::current_date(),
            severity: severity.into(),
            score: vuln.success_rate * 10.0,
            description,
            attack_vector: vuln.payload.clone(),
            reproduction_steps: vec![
                format!("1. Deploy {} agent with default configuration", vuln.target),
                "2. Execute attack vector".into(),
                "3. Observe unauthorized behavior".into(),
                "4. Verify security impact".into(),
            ],
            mitigation: self.get_mitigation(&vuln.vector_type),
            references: vec![
                "https://cve.mitre.org/".into(),
            ],
            status: CVEStatus::Draft,
        };
        
        self.records.insert(cve_id.clone(), record.clone());
        self.save_to_disk(&cve_id);
        
        if self.auto_submit {
            self.submit_to_mitre(&cve_id);
        }
        
        record
    }
    
    fn generate_cve_id(&self) -> String {
        let year = Utc::now().year();
        let uuid = Uuid::new_v4();
        let suffix = &uuid.to_string()[..8];
        format!("CVE-{}-{}-{}", year, self.org_id, suffix)
    }
    
    fn get_impact_description(&self, vector_type: &str) -> String {
        match vector_type {
            "prompt_injection" => "execute unauthorized commands and leak sensitive data".into(),
            "context_overflow" => "bypass safety guardrails".into(),
            "tool_poisoning" => "execute arbitrary code via malicious tool descriptions".into(),
            "data_exfiltration" => "exfiltrate API keys, SSH keys, and environment variables".into(),
            _ => "compromise the agent's security guarantees".into(),
        }
    }
    
    fn get_mitigation(&self, vector_type: &str) -> String {
        format!("Implement security measures for {}", vector_type)
    }
    
    fn save_to_disk(&self, cve_id: &str) {
        if let Some(record) = self.records.get(cve_id) {
            if let Ok(json) = serde_json::to_string_pretty(record) {
                let _ = std::fs::write(format!("cve_{}.json", cve_id), json);
            }
        }
    }
    
    fn submit_to_mitre(&self, _cve_id: &str) {
        // Placeholder for MITRE submission
    }
    
    pub fn get_stats(&self) -> CVEStats {
        let mut stats = CVEStats {
            total_records: self.records.len(),
            by_severity: BTreeMap::new(),
            published: 0,
            draft: 0,
        };
        
        for record in self.records.values() {
            *stats.by_severity.entry(record.severity.clone()).or_insert(0) += 1;
            match record.status {
                CVEStatus::Published => stats.published += 1,
                _ => stats.draft += 1,
            }
        }
        
        stats
    }
    
    fn current_date() -> String {
        Utc::now().format("%Y-%m-%d").to_string()
    }
}

#[derive(Debug)]
pub struct CVEStats {
    pub total_records: usize,
    pub by_severity: BTreeMap<String, usize>,
    pub published: usize,
    pub draft: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VulnerabilityReport {
    pub id: String,
    pub target: String,
    pub vector_type: String,
    pub payload: String,
    pub success_rate: f64,
    pub total_attempts: usize,
    pub risk_level: RiskLevel,
    pub discovery_timestamp: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RiskLevel {
    Low,
    Medium,
    High,
    Critical,
    Emergency,
}

impl RiskLevel {
    pub fn from_score(score: f64) -> Self {
        if score <= 2.0 {
            RiskLevel::Low
        } else if score <= 4.0 {
            RiskLevel::Medium
        } else if score <= 7.0 {
            RiskLevel::High
        } else {
            RiskLevel::Critical
        }
    }
    
    pub fn to_string(&self) -> String {
        match self {
            RiskLevel::Low => "LOW".into(),
            RiskLevel::Medium => "MEDIUM".into(),
            RiskLevel::High => "HIGH".into(),
            RiskLevel::Critical => "CRITICAL".into(),
            RiskLevel::Emergency => "EMERGENCY".into(),
        }
    }
}
