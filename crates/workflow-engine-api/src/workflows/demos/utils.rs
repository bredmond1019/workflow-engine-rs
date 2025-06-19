use std::time::Duration;
use tokio::time::sleep;
use crate::workflows::demos::timing;

pub struct NodeLogger {
    node_name: String,
}

impl NodeLogger {
    pub fn new(node_name: &str) -> Self {
        Self {
            node_name: node_name.to_string(),
        }
    }

    pub async fn starting(&self) {
        println!("🔄 Starting {}", self.node_name);
        sleep(timing::QUICK_PAUSE).await;
    }

    pub async fn working(&self, task_description: &str) {
        println!("⚙️  {} is doing {}", self.node_name, task_description);
        sleep(timing::MEDIUM_PAUSE).await;
    }

    pub async fn completed(&self) {
        println!("✅ {} is done", self.node_name);
        sleep(timing::QUICK_PAUSE).await;
    }

    pub async fn result(&self, result_description: &str) {
        println!("📊 {} result: {}", self.node_name, result_description);
        sleep(timing::MEDIUM_PAUSE).await;
    }

    pub async fn execute_with_logging<F, Fut, T>(&self, task_description: &str, operation: F) -> T
    where
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = T>,
    {
        self.starting().await;
        self.working(task_description).await;
        
        let result = operation().await;
        
        self.completed().await;
        result
    }

    pub async fn execute_with_result<F, Fut, T>(&self, task_description: &str, result_description: &str, operation: F) -> T
    where
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = T>,
    {
        self.starting().await;
        self.working(task_description).await;
        
        let result = operation().await;
        
        self.completed().await;
        self.result(result_description).await;
        result
    }
}

pub async fn section_break(title: &str) {
    println!("\n{}", "═".repeat(80));
    println!("  {}", title);
    println!("{}\n", "═".repeat(80));
    sleep(timing::SECTION_PAUSE).await;
}

pub async fn subsection_break(title: &str) {
    println!("\n{}", "─".repeat(60));
    println!("  {}", title);
    println!("{}", "─".repeat(60));
    sleep(timing::MEDIUM_PAUSE).await;
}

pub async fn demo_pause() {
    sleep(timing::DEMO_PAUSE).await;
}

pub async fn reading_pause() {
    sleep(timing::READING_PAUSE).await;
}

pub fn format_success(message: &str) -> String {
    format!("✅ {}", message)
}

pub fn format_info(message: &str) -> String {
    format!("ℹ️  {}", message)
}

pub fn format_warning(message: &str) -> String {
    format!("⚠️  {}", message)
}

pub fn format_error(message: &str) -> String {
    format!("❌ {}", message)
}

pub fn format_progress(message: &str) -> String {
    format!("🔄 {}", message)
}