use std::sync::Arc;
use workflow_engine_api::bootstrap::Container;

#[derive(Debug)]
struct DatabaseService {
    connection_string: String,
}

impl DatabaseService {
    fn new(connection_string: String) -> Self {
        Self { connection_string }
    }
    
    fn query(&self, sql: &str) -> String {
        format!("Executing '{}' on {}", sql, self.connection_string)
    }
}

#[derive(Debug)]
struct UserService {
    db: Arc<DatabaseService>,
}

impl UserService {
    fn new(db: Arc<DatabaseService>) -> Self {
        Self { db }
    }
    
    fn get_user_by_id(&self, id: u32) -> String {
        self.db.query(&format!("SELECT * FROM users WHERE id = {}", id))
    }
    
    fn create_user(&self, name: &str) -> String {
        self.db.query(&format!("INSERT INTO users (name) VALUES ('{}')", name))
    }
}

#[derive(Debug)]
struct ApiController {
    user_service: Arc<UserService>,
}

impl ApiController {
    fn new(user_service: Arc<UserService>) -> Self {
        Self { user_service }
    }
    
    fn handle_get_user(&self, id: u32) -> String {
        format!("API Response: {}", self.user_service.get_user_by_id(id))
    }
    
    fn handle_create_user(&self, name: &str) -> String {
        format!("API Response: {}", self.user_service.create_user(name))
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create container using the builder pattern
    let container = Container::builder()
        // Register DatabaseService as singleton
        .singleton(|_| {
            println!("Creating DatabaseService instance");
            Ok(Arc::new(DatabaseService::new("postgresql://localhost:5432/mydb".to_string())))
        })?
        // Register UserService as singleton with DatabaseService dependency
        .singleton(|container| {
            println!("Creating UserService instance");
            let db = container.resolve::<DatabaseService>()?;
            Ok(Arc::new(UserService::new(db)))
        })?
        // Register ApiController as transient with UserService dependency
        .transient(|container| {
            println!("Creating ApiController instance");
            let user_service = container.resolve::<UserService>()?;
            Ok(Arc::new(ApiController::new(user_service)))
        })?
        .build()?;
    
    println!("=== Dependency Injection Container Example ===\n");
    
    // Demonstrate singleton behavior
    println!("1. Testing singleton behavior:");
    let db1 = container.resolve::<DatabaseService>()?;
    let db2 = container.resolve::<DatabaseService>()?;
    println!("Database instances are same: {}", Arc::ptr_eq(&db1, &db2));
    
    // Demonstrate transient behavior
    println!("\n2. Testing transient behavior:");
    let controller1 = container.resolve::<ApiController>()?;
    let controller2 = container.resolve::<ApiController>()?;
    println!("Controller instances are same: {}", Arc::ptr_eq(&controller1, &controller2));
    
    // Demonstrate dependency injection
    println!("\n3. Testing dependency injection:");
    let controller = container.resolve::<ApiController>()?;
    println!("{}", controller.handle_get_user(123));
    println!("{}", controller.handle_create_user("Alice"));
    
    // Check registered services
    println!("\n4. Container information:");
    println!("Is DatabaseService registered: {}", container.is_registered::<DatabaseService>());
    println!("Is UserService registered: {}", container.is_registered::<UserService>());
    println!("Is ApiController registered: {}", container.is_registered::<ApiController>());
    println!("Total registered services: {}", container.service_count());
    
    Ok(())
}