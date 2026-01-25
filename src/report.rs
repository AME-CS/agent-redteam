use crate::scoring::RiskAssessment;

pub struct ReportGenerator {
    version: String,
}

impl ReportGenerator {
    pub fn new(version: &str) -> Self {
        ReportGenerator {
            version: version.into(),
        }
    }

    fn risk_class(tier: &str) -> &'static str {
        match tier {
            "LOW" => "risk-LOW",
            "MODERATE" => "risk-MODERATE",
            "HIGH" => "risk-HIGH",
            "CRITICAL" => "risk-CRITICAL",
            "EMERGENCY" => "risk-EMERGENCY",
            _ => "risk-LOW",
        }
    }

    fn generate_vector_rows(assessment: &RiskAssessment) -> String {
        let mut rows = String::new();
        
        let combined = assessment.to_dict();
        let by_vector = combined["by_vector"].as_object().unwrap();
        
        for (vt, score) in &assessment.vector_scores {
            let rate = by_vector.get(vt)
                .and_then(|v| v.as_object())
                .and_then(|o| o.get("rate"))
                .and_then(|r| r.as_f64())
                .unwrap_or(0.0);
            
            let bar_width = (score * 10.0) as usize;
            
            rows.push_str(&format!(
                "<tr><td>{}</td><td>{:.1}/10</td><td>{:.1}%</td>\
                <td><div class=\"score-bar\"><div class=\"score-fill\" style=\"width: {}%\"></div></div></td></tr>",
                vt, score, rate * 100.0, bar_width
            ));
        }
        
        rows
    }

    fn generate_recommendation_items(assessment: &RiskAssessment) -> String {
        assessment.recommendations
            .iter()
            .map(|rec| format!("<li>{}</li>", rec))
            .collect::<Vec<_>>()
            .join("\n")
    }

    pub fn generate_html(&self, assessment: &RiskAssessment) -> String {
        let risk_class = Self::risk_class(&assessment.risk_tier);
        let vector_rows = Self::generate_vector_rows(assessment);
        let recommendation_items = Self::generate_recommendation_items(assessment);
        let timestamp = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();
        
        // Read template at runtime
        let template = include_str!("../templates/report.html");
        
        template.replace("{{target}}", &assessment.target)
            .replace("{{timestamp}}", &timestamp)
            .replace("{{overall_score}}", &format!("{:.1}", assessment.overall_score))
            .replace("{{total_attacks}}", &assessment.total_attacks.to_string())
            .replace("{{total_vulns}}", &assessment.total_vulnerabilities.to_string())
            .replace("{{risk_tier}}", &assessment.risk_tier)
            .replace("{{risk_class}}", risk_class)
            .replace("{{vector_rows}}", &vector_rows)
            .replace("{{recommendation_items}}", &recommendation_items)
            .replace("{{version}}", &self.version)
            .replace("{{timestamp_full}}", &chrono::Utc::now().to_rfc3339())
    }

    pub fn generate_json(&self, assessment: &RiskAssessment) -> String {
        serde_json::to_string_pretty(&assessment.to_dict()).unwrap()
    }

    pub fn save_report(&self, assessment: &RiskAssessment, output_path: &str, fmt: &str) -> String {
        let content = match fmt {
            "html" => self.generate_html(assessment),
            "json" => self.generate_json(assessment),
            _ => panic!("Unknown format: {}", fmt),
        };
        
        std::fs::write(output_path, content).expect("Failed to write report");
        output_path.into()
    }
}
