//! GraphQL Security Analysis
//!
//! Provides security analysis and threat detection for GraphQL queries

use std::collections::HashMap;
use serde::{Serialize, Deserialize};

/// Security level classification for queries
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SecurityLevel {
    Safe,
    Low,
    Medium,
    High,
    Critical,
}

/// Detailed threat analysis for a GraphQL query
#[derive(Debug, Clone)]
pub struct ThreatAnalysis {
    pub level: SecurityLevel,
    pub threats: Vec<ThreatType>,
    pub recommendations: Vec<String>,
    pub risk_score: u32,
}

/// Types of security threats that can be detected
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ThreatType {
    DoSAttack { reason: String },
    DataExfiltration { fields: Vec<String> },
    InjectionAttempt { payload: String },
    AuthorizationEvasion { method: String },
    ResourceExhaustion { resource_type: String },
    InformationDisclosure { sensitive_data: String },
}

/// GraphQL query security analyzer
pub struct QuerySecurityAnalyzer {
    threat_patterns: HashMap<String, ThreatType>,
}

impl QuerySecurityAnalyzer {
    pub fn new() -> Self {
        let mut threat_patterns = HashMap::new();
        
        // Initialize known threat patterns
        threat_patterns.insert(
            "union.*select".to_string(),
            ThreatType::InjectionAttempt { payload: "SQL injection pattern".to_string() }
        );
        
        threat_patterns.insert(
            "script.*alert".to_string(),
            ThreatType::InjectionAttempt { payload: "XSS injection pattern".to_string() }
        );
        
        Self { threat_patterns }
    }
    
    pub fn analyze_query(&self, query: &str) -> ThreatAnalysis {
        let mut threats = Vec::new();
        let mut recommendations = Vec::new();
        let mut risk_score = 0;
        
        // Analyze for various threat types
        self.analyze_dos_patterns(query, &mut threats, &mut risk_score);
        self.analyze_injection_patterns(query, &mut threats, &mut risk_score);
        self.analyze_data_exfiltration(query, &mut threats, &mut risk_score);
        self.analyze_authorization_evasion(query, &mut threats, &mut risk_score);
        
        // Generate recommendations based on threats
        for threat in &threats {
            match threat {
                ThreatType::DoSAttack { .. } => {
                    recommendations.push("Implement rate limiting and query complexity analysis".to_string());
                },
                ThreatType::InjectionAttempt { .. } => {
                    recommendations.push("Validate and sanitize all input parameters".to_string());
                },
                ThreatType::DataExfiltration { .. } => {
                    recommendations.push("Implement field-level authorization checks".to_string());
                },
                ThreatType::AuthorizationEvasion { .. } => {
                    recommendations.push("Verify user permissions for all requested fields".to_string());
                },
                _ => {},
            }
        }
        
        let level = self.calculate_security_level(risk_score);
        
        ThreatAnalysis {
            level,
            threats,
            recommendations,
            risk_score,
        }
    }
    
    fn analyze_dos_patterns(&self, query: &str, threats: &mut Vec<ThreatType>, risk_score: &mut u32) {
        // Check for patterns that could lead to DoS
        if query.matches('{').count() > 20 {
            threats.push(ThreatType::DoSAttack {
                reason: "Deeply nested query structure".to_string(),
            });
            *risk_score += 15;
        }
        
        if query.contains("limit: 1000") || query.contains("first: 1000") {
            threats.push(ThreatType::DoSAttack {
                reason: "Large result set request".to_string(),
            });
            *risk_score += 10;
        }
        
        if query.len() > 10000 {
            threats.push(ThreatType::DoSAttack {
                reason: "Excessively large query".to_string(),
            });
            *risk_score += 20;
        }
    }
    
    fn analyze_injection_patterns(&self, query: &str, threats: &mut Vec<ThreatType>, risk_score: &mut u32) {
        let injection_patterns = vec![
            ("DROP TABLE", "SQL injection attempt"),
            ("UNION SELECT", "SQL injection attempt"),
            ("' OR '1'='1", "SQL injection attempt"),
            ("<script>", "XSS injection attempt"),
            ("javascript:", "JavaScript injection attempt"),
            ("${", "Template injection attempt"),
            ("{{", "Template injection attempt"),
        ];
        
        for (pattern, description) in injection_patterns {
            if query.to_uppercase().contains(pattern) {
                threats.push(ThreatType::InjectionAttempt {
                    payload: format!("{}: {}", description, pattern),
                });
                *risk_score += 25;
            }
        }
    }
    
    fn analyze_data_exfiltration(&self, query: &str, threats: &mut Vec<ThreatType>, risk_score: &mut u32) {
        let sensitive_fields = vec![
            "password", "token", "secret", "key", "credential",
            "email", "phone", "ssn", "credit_card", "bank_account"
        ];
        
        let mut detected_fields = Vec::new();
        
        for field in sensitive_fields {
            if query.contains(field) {
                detected_fields.push(field.to_string());
                *risk_score += 10;
            }
        }
        
        if !detected_fields.is_empty() {
            threats.push(ThreatType::DataExfiltration {
                fields: detected_fields,
            });
        }
        
        // Check for introspection queries (information disclosure)
        if query.contains("__schema") || query.contains("__type") {
            threats.push(ThreatType::InformationDisclosure {
                sensitive_data: "Schema information".to_string(),
            });
            *risk_score += 15;
        }
    }
    
    fn analyze_authorization_evasion(&self, query: &str, threats: &mut Vec<ThreatType>, risk_score: &mut u32) {
        // Check for attempts to access administrative functions
        let admin_patterns = vec![
            "admin", "root", "system", "internal", "debug",
            "deleteAll", "resetDatabase", "clearCache"
        ];
        
        for pattern in admin_patterns {
            if query.contains(pattern) {
                threats.push(ThreatType::AuthorizationEvasion {
                    method: format!("Potential admin function access: {}", pattern),
                });
                *risk_score += 20;
            }
        }
        
        // Check for user enumeration attempts
        if query.contains("users(") && (query.contains("limit: ") || query.contains("first: ")) {
            threats.push(ThreatType::AuthorizationEvasion {
                method: "Potential user enumeration".to_string(),
            });
            *risk_score += 12;
        }
    }
    
    fn calculate_security_level(&self, risk_score: u32) -> SecurityLevel {
        match risk_score {
            0..=5 => SecurityLevel::Safe,
            6..=15 => SecurityLevel::Low,
            16..=30 => SecurityLevel::Medium,
            31..=50 => SecurityLevel::High,
            _ => SecurityLevel::Critical,
        }
    }
}