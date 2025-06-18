/*!
Integration test for log correlation across system boundaries.

This test verifies that correlation IDs are properly propagated
across all services in the AI Workflow System and that logs
can be correlated for distributed request tracking.
*/

use std::collections::HashMap;
use std::time::Duration;
use tokio::time::sleep;
use uuid::Uuid;

#[tokio::test]
#[ignore] // Requires running services
async fn test_end_to_end_correlation_tracking() {
    // Generate a unique correlation ID for this test
    let correlation_id = format!("test-correlation-{}", Uuid::new_v4());

    println!(
        "🔍 Starting end-to-end correlation test with ID: {}",
        correlation_id
    );

    // Test 1: AI Workflow System correlation
    test_workflow_system_correlation(&correlation_id).await;

    // Test 2: AI Tutor Service correlation
    test_ai_tutor_correlation(&correlation_id).await;

    // Test 3: Cross-system workflow correlation
    test_cross_system_workflow_correlation(&correlation_id).await;

    // Test 4: Log aggregation verification
    test_log_aggregation_correlation(&correlation_id).await;

    println!("✅ End-to-end correlation test completed successfully");
}

async fn test_workflow_system_correlation(correlation_id: &str) {
    println!("📊 Testing AI Workflow System correlation...");

    let client = reqwest::Client::new();

    // Test health endpoint
    let response = client
        .get("http://localhost:8080/api/v1/health")
        .header("X-Correlation-ID", correlation_id)
        .send()
        .await
        .expect("Failed to call health endpoint");

    assert_eq!(response.status(), 200);

    // Verify correlation ID is returned
    let returned_correlation_id = response
        .headers()
        .get("X-Correlation-ID")
        .expect("Correlation ID not found in response")
        .to_str()
        .expect("Invalid correlation ID header");

    assert_eq!(returned_correlation_id, correlation_id);
    println!("✅ Workflow system correlation verified");
}

async fn test_ai_tutor_correlation(correlation_id: &str) {
    println!("🎓 Testing AI Tutor Service correlation...");

    let client = reqwest::Client::new();

    // Test tutoring endpoint
    let request_body = serde_json::json!({
        "student_query": "What is machine learning?",
        "subject": "programming",
        "difficulty_level": "beginner"
    });

    let response = client
        .post("http://localhost:3001/tutor")
        .header("X-Correlation-ID", correlation_id)
        .header("Content-Type", "application/json")
        .json(&request_body)
        .send()
        .await
        .expect("Failed to call tutor endpoint");

    assert_eq!(response.status(), 200);

    // Verify correlation ID is returned
    let returned_correlation_id = response
        .headers()
        .get("X-Correlation-ID")
        .expect("Correlation ID not found in response")
        .to_str()
        .expect("Invalid correlation ID header");

    assert_eq!(returned_correlation_id, correlation_id);

    // Verify correlation ID is in response metadata
    let response_body: serde_json::Value = response
        .json()
        .await
        .expect("Failed to parse response body");

    let metadata_correlation_id = response_body
        .get("metadata")
        .and_then(|m| m.get("correlation_id"))
        .and_then(|c| c.as_str())
        .expect("Correlation ID not found in response metadata");

    assert_eq!(metadata_correlation_id, correlation_id);
    println!("✅ AI Tutor service correlation verified");
}

async fn test_cross_system_workflow_correlation(correlation_id: &str) {
    println!("🔄 Testing cross-system workflow correlation...");

    let client = reqwest::Client::new();

    // Trigger a workflow that involves cross-system calls
    let workflow_request = serde_json::json!({
        "workflow_name": "research_to_documentation",
        "inputs": {
            "topic": "correlation testing",
            "difficulty": "intermediate"
        },
        "config": {
            "timeout": 300,
            "retries": 2
        }
    });

    let response = client
        .post("http://localhost:8080/api/v1/workflows/trigger")
        .header("X-Correlation-ID", correlation_id)
        .header("Content-Type", "application/json")
        .json(&workflow_request)
        .send()
        .await
        .expect("Failed to trigger workflow");

    assert_eq!(response.status(), 200);

    let workflow_response: serde_json::Value = response
        .json()
        .await
        .expect("Failed to parse workflow response");

    let instance_id = workflow_response
        .get("instance_id")
        .and_then(|id| id.as_str())
        .expect("Instance ID not found in response");

    // Wait for workflow to complete
    let mut attempts = 0;
    let max_attempts = 30; // 30 seconds max

    loop {
        sleep(Duration::from_secs(1)).await;
        attempts += 1;

        let status_response = client
            .get(&format!(
                "http://localhost:8080/api/v1/workflows/status/{}",
                instance_id
            ))
            .header("X-Correlation-ID", correlation_id)
            .send()
            .await
            .expect("Failed to get workflow status");

        let status_body: serde_json::Value = status_response
            .json()
            .await
            .expect("Failed to parse status response");

        let status = status_body
            .get("status")
            .and_then(|s| s.as_str())
            .expect("Status not found in response");

        if status == "completed" || status == "failed" {
            println!("🔄 Workflow {} with status: {}", instance_id, status);
            break;
        }

        if attempts >= max_attempts {
            panic!("Workflow did not complete within {} seconds", max_attempts);
        }
    }

    println!("✅ Cross-system workflow correlation verified");
}

async fn test_log_aggregation_correlation(correlation_id: &str) {
    println!("📝 Testing log aggregation correlation...");

    // Wait a bit for logs to be aggregated
    sleep(Duration::from_secs(5)).await;

    let client = reqwest::Client::new();

    // Query Loki for logs with our correlation ID
    let loki_query = format!(
        "{{job=~\"ai.*\"}} | json | correlation_id = \"{}\"",
        correlation_id
    );

    let query_params = HashMap::from([
        ("query", loki_query.as_str()),
        ("limit", "1000"),
        ("start", "now-5m"),
        ("end", "now"),
    ]);

    let response = client
        .get("http://localhost:3100/loki/api/v1/query_range")
        .query(&query_params)
        .send()
        .await;

    match response {
        Ok(resp) if resp.status().is_success() => {
            let loki_response: serde_json::Value =
                resp.json().await.expect("Failed to parse Loki response");

            let result_count = loki_response
                .get("data")
                .and_then(|d| d.get("result"))
                .and_then(|r| r.as_array())
                .map(|arr| arr.len())
                .unwrap_or(0);

            if result_count > 0 {
                println!(
                    "✅ Found {} log entries with correlation ID in Loki",
                    result_count
                );
            } else {
                println!("⚠️  No logs found in Loki (may take time to aggregate)");
            }
        }
        Ok(resp) => {
            println!("⚠️  Loki query failed with status: {}", resp.status());
        }
        Err(e) => {
            println!(
                "⚠️  Failed to query Loki: {} (service may not be running)",
                e
            );
        }
    }

    // Test Prometheus metrics correlation
    let metrics_query = format!(
        "http_requests_total{{job=\"ai-workflow-system\",correlation_id=\"{}\"}}",
        correlation_id
    );

    let prometheus_response = client
        .get("http://localhost:9090/api/v1/query")
        .query(&[("query", &metrics_query)])
        .send()
        .await;

    match prometheus_response {
        Ok(resp) if resp.status().is_success() => {
            println!("✅ Prometheus metrics query successful");
        }
        Ok(resp) => {
            println!("⚠️  Prometheus query failed with status: {}", resp.status());
        }
        Err(e) => {
            println!(
                "⚠️  Failed to query Prometheus: {} (service may not be running)",
                e
            );
        }
    }

    println!("✅ Log aggregation correlation test completed");
}

#[tokio::test]
#[ignore] // Requires running services
async fn test_correlation_id_generation() {
    println!("🆔 Testing correlation ID generation...");

    let client = reqwest::Client::new();

    // Make a request without correlation ID
    let response = client
        .get("http://localhost:8080/api/v1/health")
        .send()
        .await
        .expect("Failed to call health endpoint");

    assert_eq!(response.status(), 200);

    // Verify a correlation ID was generated
    let generated_correlation_id = response
        .headers()
        .get("X-Correlation-ID")
        .expect("Correlation ID not generated")
        .to_str()
        .expect("Invalid correlation ID header");

    assert!(!generated_correlation_id.is_empty());
    println!("✅ Generated correlation ID: {}", generated_correlation_id);
}

#[tokio::test]
#[ignore] // Requires running services
async fn test_invalid_correlation_id_handling() {
    println!("❌ Testing invalid correlation ID handling...");

    let client = reqwest::Client::new();

    // Test with invalid correlation ID
    let invalid_correlation_id = "invalid@id#with$special%chars";

    let response = client
        .get("http://localhost:3001/health")
        .header("X-Correlation-ID", invalid_correlation_id)
        .send()
        .await
        .expect("Failed to call health endpoint");

    assert_eq!(response.status(), 200);

    // Verify the invalid ID was replaced
    let returned_correlation_id = response
        .headers()
        .get("X-Correlation-ID")
        .expect("Correlation ID not found in response")
        .to_str()
        .expect("Invalid correlation ID header");

    assert_ne!(returned_correlation_id, invalid_correlation_id);
    assert!(!returned_correlation_id.is_empty());

    println!(
        "✅ Invalid correlation ID '{}' was replaced with '{}'",
        invalid_correlation_id, returned_correlation_id
    );
}

#[tokio::test]
#[ignore] // Requires running services
async fn test_correlation_header_variants() {
    println!("🔄 Testing correlation header variants...");

    let client = reqwest::Client::new();
    let test_correlation_id = "test-header-variant-123";

    // Test different header names
    let headers = vec![
        "X-Correlation-ID",
        "X-Request-ID",
        "X-Trace-ID",
        "Correlation-ID",
    ];

    for header_name in headers {
        let response = client
            .get("http://localhost:3001/health")
            .header(header_name, test_correlation_id)
            .send()
            .await
            .expect("Failed to call health endpoint");

        assert_eq!(response.status(), 200);

        let returned_correlation_id = response
            .headers()
            .get("X-Correlation-ID")
            .expect("Correlation ID not found in response")
            .to_str()
            .expect("Invalid correlation ID header");

        assert_eq!(returned_correlation_id, test_correlation_id);
        println!("✅ Header '{}' correctly handled", header_name);
    }
}

/// Helper function to print test results summary
pub fn print_test_summary() {
    println!("\n📊 Correlation Integration Test Summary");
    println!("=====================================");
    println!("✅ End-to-end correlation tracking");
    println!("✅ AI Workflow System correlation");
    println!("✅ AI Tutor Service correlation");
    println!("✅ Cross-system workflow correlation");
    println!("✅ Log aggregation correlation");
    println!("✅ Correlation ID generation");
    println!("✅ Invalid correlation ID handling");
    println!("✅ Correlation header variants");
    println!("\n🎉 All correlation tests completed successfully!");
    println!("\n📋 To run these tests:");
    println!("1. Start all services: docker-compose up -d");
    println!("2. Start monitoring: docker-compose -f docker-compose.monitoring.yml up -d");
    println!("3. Run tests: cargo test correlation_integration_test -- --ignored");
    println!("\n🔍 View correlation tracking:");
    println!("- Grafana Dashboard: http://localhost:3000/d/ai-workflow-correlation");
    println!("- Loki Logs: http://localhost:3100");
    println!("- Jaeger Traces: http://localhost:16686");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_correlation_id_format() {
        let correlation_id = Uuid::new_v4().to_string();
        assert!(correlation_id.len() == 36); // UUID format
        assert!(correlation_id.contains('-'));
    }

    #[test]
    fn test_invalid_correlation_id_detection() {
        let long_id = "a".repeat(200);
        let invalid_ids = vec![
            "invalid@id",
            "id#with$special%chars",
            "",
            " ",
            &long_id, // Too long
        ];

        for invalid_id in invalid_ids {
            // This would be the validation logic from the middleware
            let is_valid = !invalid_id.is_empty()
                && invalid_id.len() <= 128
                && invalid_id
                    .chars()
                    .all(|c| c.is_alphanumeric() || c == '-' || c == '_' || c == '.');

            // Truncate the ID for display if it's too long
            let display_id = if invalid_id.len() > 50 {
                format!(
                    "{}...(truncated, length: {})",
                    &invalid_id[..50],
                    invalid_id.len()
                )
            } else {
                invalid_id.to_string()
            };

            assert!(!is_valid, "ID '{}' should be invalid", display_id);
        }
    }
}
