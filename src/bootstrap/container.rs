use std::any::{Any, TypeId};
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex, RwLock};
use std::fmt::Debug;
use std::marker::PhantomData;

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Errors that can occur during dependency injection operations
#[derive(Error, Debug)]
pub enum ContainerError {
    #[error("Service not found: {0}")]
    ServiceNotFound(String),
    
    #[error("Circular dependency detected: {0}")]
    CircularDependency(String),
    
    #[error("Failed to resolve service: {0}")]
    ResolutionError(String),
    
    #[error("Service already registered: {0}")]
    ServiceAlreadyRegistered(String),
    
    #[error("Failed to build service: {0}")]
    BuildError(String),
    
    #[error("Type mismatch when resolving service")]
    TypeMismatch,
}

pub type Result<T> = std::result::Result<T, ContainerError>;

/// Service lifetime configuration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ServiceLifetime {
    /// Service is created once and shared across all requests
    Singleton,
    /// New instance is created for each request
    Transient,
}

/// Trait for services that can be registered in the container
pub trait Service: Any + Send + Sync {
    fn as_any(&self) -> &dyn Any;
    fn as_any_arc(self: Arc<Self>) -> Arc<dyn Any + Send + Sync>;
}

impl<T: Any + Send + Sync> Service for T {
    fn as_any(&self) -> &dyn Any {
        self
    }
    
    fn as_any_arc(self: Arc<Self>) -> Arc<dyn Any + Send + Sync> {
        self
    }
}

/// Factory function for creating service instances
pub type ServiceFactory = Box<dyn Fn(&Container) -> Result<Arc<dyn Service>> + Send + Sync>;

/// Service registration information
struct ServiceRegistration {
    lifetime: ServiceLifetime,
    factory: ServiceFactory,
    dependencies: Vec<TypeId>,
}

/// Dependency injection container
pub struct Container {
    registrations: RwLock<HashMap<TypeId, ServiceRegistration>>,
    singletons: Mutex<HashMap<TypeId, Arc<dyn Service>>>,
    resolving: Mutex<HashSet<TypeId>>,
}

impl Container {
    /// Create a new empty container
    pub fn new() -> Self {
        Self {
            registrations: RwLock::new(HashMap::new()),
            singletons: Mutex::new(HashMap::new()),
            resolving: Mutex::new(HashSet::new()),
        }
    }
    
    /// Create a new container builder
    pub fn builder() -> ContainerBuilder {
        ContainerBuilder::new()
    }
    
    /// Register a service with the container
    pub fn register<T, F>(&self, lifetime: ServiceLifetime, factory: F) -> Result<()>
    where
        T: Service + 'static,
        F: Fn(&Container) -> Result<Arc<T>> + Send + Sync + 'static,
    {
        let type_id = TypeId::of::<T>();
        let mut registrations = self.registrations.write().unwrap();
        
        if registrations.contains_key(&type_id) {
            return Err(ContainerError::ServiceAlreadyRegistered(
                std::any::type_name::<T>().to_string()
            ));
        }
        
        let service_factory: ServiceFactory = Box::new(move |container| {
            factory(container).map(|service| service as Arc<dyn Service>)
        });
        
        registrations.insert(type_id, ServiceRegistration {
            lifetime,
            factory: service_factory,
            dependencies: Vec::new(),
        });
        
        Ok(())
    }
    
    /// Register a service with dependencies
    pub fn register_with_deps<T, F>(
        &self,
        lifetime: ServiceLifetime,
        dependencies: Vec<TypeId>,
        factory: F,
    ) -> Result<()>
    where
        T: Service + 'static,
        F: Fn(&Container) -> Result<Arc<T>> + Send + Sync + 'static,
    {
        let type_id = TypeId::of::<T>();
        let mut registrations = self.registrations.write().unwrap();
        
        if registrations.contains_key(&type_id) {
            return Err(ContainerError::ServiceAlreadyRegistered(
                std::any::type_name::<T>().to_string()
            ));
        }
        
        let service_factory: ServiceFactory = Box::new(move |container| {
            factory(container).map(|service| service as Arc<dyn Service>)
        });
        
        registrations.insert(type_id, ServiceRegistration {
            lifetime,
            factory: service_factory,
            dependencies,
        });
        
        Ok(())
    }
    
    /// Resolve a service from the container
    pub fn resolve<T: Service + 'static>(&self) -> Result<Arc<T>> {
        let type_id = TypeId::of::<T>();
        
        // Check for circular dependencies
        {
            let mut resolving = self.resolving.lock().unwrap();
            if resolving.contains(&type_id) {
                return Err(ContainerError::CircularDependency(
                    std::any::type_name::<T>().to_string()
                ));
            }
            resolving.insert(type_id);
        }
        
        // Ensure we remove from resolving set when done
        let _guard = CircularDependencyGuard {
            type_id,
            resolving: &self.resolving,
        };
        
        // Check if singleton already exists
        let registration = {
            let registrations = self.registrations.read().unwrap();
            registrations.get(&type_id).ok_or_else(|| {
                ContainerError::ServiceNotFound(std::any::type_name::<T>().to_string())
            })?.lifetime
        };
        
        if registration == ServiceLifetime::Singleton {
            let singletons = self.singletons.lock().unwrap();
            if let Some(service) = singletons.get(&type_id) {
                return self.downcast_service::<T>(service);
            }
        }
        
        // Create new instance
        let service = self.create_instance(type_id)?;
        
        // Store singleton if needed
        if registration == ServiceLifetime::Singleton {
            let mut singletons = self.singletons.lock().unwrap();
            singletons.insert(type_id, service.clone());
        }
        
        self.downcast_service::<T>(&service)
    }
    
    /// Helper method to safely downcast a service
    fn downcast_service<T: Service + 'static>(&self, service: &Arc<dyn Service>) -> Result<Arc<T>> {
        // Clone and convert to Any
        let any_service = service.clone().as_any_arc();
        any_service.downcast::<T>()
            .map_err(|_| ContainerError::TypeMismatch)
    }
    
    /// Create a new instance of a service
    fn create_instance(&self, type_id: TypeId) -> Result<Arc<dyn Service>> {
        let registrations = self.registrations.read().unwrap();
        let registration = registrations.get(&type_id).ok_or_else(|| {
            ContainerError::ServiceNotFound(format!("TypeId: {:?}", type_id))
        })?;
        
        (registration.factory)(self)
    }
    
    /// Check if a service is registered
    pub fn is_registered<T: Service + 'static>(&self) -> bool {
        let type_id = TypeId::of::<T>();
        self.registrations.read().unwrap().contains_key(&type_id)
    }
    
    /// Get the number of registered services
    pub fn service_count(&self) -> usize {
        self.registrations.read().unwrap().len()
    }
    
    /// Clear all singleton instances
    pub fn clear_singletons(&self) {
        self.singletons.lock().unwrap().clear();
    }
    
    /// Validate the dependency graph for circular dependencies
    pub fn validate_dependencies(&self) -> Result<()> {
        let registrations = self.registrations.read().unwrap();
        let mut visited = HashSet::new();
        let mut path = Vec::new();
        
        for &type_id in registrations.keys() {
            if !visited.contains(&type_id) {
                self.check_circular_deps(&registrations, type_id, &mut visited, &mut path)?;
            }
        }
        
        Ok(())
    }
    
    fn check_circular_deps(
        &self,
        registrations: &HashMap<TypeId, ServiceRegistration>,
        type_id: TypeId,
        visited: &mut HashSet<TypeId>,
        path: &mut Vec<TypeId>,
    ) -> Result<()> {
        if path.contains(&type_id) {
            let cycle = path.iter()
                .skip_while(|&&id| id != type_id)
                .map(|id| format!("{:?}", id))
                .collect::<Vec<_>>()
                .join(" -> ");
            return Err(ContainerError::CircularDependency(cycle));
        }
        
        path.push(type_id);
        
        if let Some(registration) = registrations.get(&type_id) {
            for &dep_id in &registration.dependencies {
                if !visited.contains(&dep_id) {
                    self.check_circular_deps(registrations, dep_id, visited, path)?;
                }
            }
        }
        
        path.pop();
        visited.insert(type_id);
        Ok(())
    }
}

impl Default for Container {
    fn default() -> Self {
        Self::new()
    }
}

/// Guard to ensure type_id is removed from resolving set
struct CircularDependencyGuard<'a> {
    type_id: TypeId,
    resolving: &'a Mutex<HashSet<TypeId>>,
}

impl<'a> Drop for CircularDependencyGuard<'a> {
    fn drop(&mut self) {
        let mut resolving = self.resolving.lock().unwrap();
        resolving.remove(&self.type_id);
    }
}

/// Builder for configuring a dependency injection container
pub struct ContainerBuilder {
    container: Container,
}

impl ContainerBuilder {
    /// Create a new container builder
    pub fn new() -> Self {
        Self {
            container: Container::new(),
        }
    }
    
    /// Register a singleton service
    pub fn singleton<T, F>(self, factory: F) -> Result<Self>
    where
        T: Service + 'static,
        F: Fn(&Container) -> Result<Arc<T>> + Send + Sync + 'static,
    {
        self.container.register(ServiceLifetime::Singleton, factory)?;
        Ok(self)
    }
    
    /// Register a transient service
    pub fn transient<T, F>(self, factory: F) -> Result<Self>
    where
        T: Service + 'static,
        F: Fn(&Container) -> Result<Arc<T>> + Send + Sync + 'static,
    {
        self.container.register(ServiceLifetime::Transient, factory)?;
        Ok(self)
    }
    
    /// Register a singleton service with dependencies
    pub fn singleton_with_deps<T, F>(
        self,
        dependencies: Vec<TypeId>,
        factory: F,
    ) -> Result<Self>
    where
        T: Service + 'static,
        F: Fn(&Container) -> Result<Arc<T>> + Send + Sync + 'static,
    {
        self.container.register_with_deps(ServiceLifetime::Singleton, dependencies, factory)?;
        Ok(self)
    }
    
    /// Register a transient service with dependencies
    pub fn transient_with_deps<T, F>(
        self,
        dependencies: Vec<TypeId>,
        factory: F,
    ) -> Result<Self>
    where
        T: Service + 'static,
        F: Fn(&Container) -> Result<Arc<T>> + Send + Sync + 'static,
    {
        self.container.register_with_deps(ServiceLifetime::Transient, dependencies, factory)?;
        Ok(self)
    }
    
    /// Build the configured container
    pub fn build(self) -> Result<Container> {
        // Validate dependency graph before returning
        self.container.validate_dependencies()?;
        Ok(self.container)
    }
}

impl Default for ContainerBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Helper macro for registering services with automatic dependency tracking
#[macro_export]
macro_rules! register_service {
    ($container:expr, $service_type:ty, $lifetime:expr, |$c:ident| $body:expr) => {{
        $container.register::<$service_type, _>($lifetime, |$c| $body)
    }};
    
    ($container:expr, $service_type:ty, $lifetime:expr, [$($dep:ty),*], |$c:ident| $body:expr) => {{
        let deps = vec![$(std::any::TypeId::of::<$dep>()),*];
        $container.register_with_deps::<$service_type, _>($lifetime, deps, |$c| $body)
    }};
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};
    
    #[derive(Debug)]
    struct TestService {
        id: usize,
    }
    
    #[derive(Debug)]
    struct DependentService {
        test_service: Arc<TestService>,
    }
    
    #[derive(Debug)]
    struct CircularServiceA {
        b: Option<Arc<CircularServiceB>>,
    }
    
    #[derive(Debug)]
    struct CircularServiceB {
        a: Option<Arc<CircularServiceA>>,
    }
    
    static COUNTER: AtomicUsize = AtomicUsize::new(0);
    
    #[test]
    fn test_singleton_registration() {
        let container = Container::new();
        
        container.register(ServiceLifetime::Singleton, |_| {
            Ok(Arc::new(TestService {
                id: COUNTER.fetch_add(1, Ordering::SeqCst),
            }))
        }).unwrap();
        
        let service1 = container.resolve::<TestService>().unwrap();
        let service2 = container.resolve::<TestService>().unwrap();
        
        assert_eq!(service1.id, service2.id);
    }
    
    #[test]
    fn test_transient_registration() {
        let container = Container::new();
        
        container.register(ServiceLifetime::Transient, |_| {
            Ok(Arc::new(TestService {
                id: COUNTER.fetch_add(1, Ordering::SeqCst),
            }))
        }).unwrap();
        
        let service1 = container.resolve::<TestService>().unwrap();
        let service2 = container.resolve::<TestService>().unwrap();
        
        assert_ne!(service1.id, service2.id);
    }
    
    #[test]
    fn test_dependency_resolution() {
        let container = Container::new();
        
        container.register(ServiceLifetime::Singleton, |_| {
            Ok(Arc::new(TestService { id: 42 }))
        }).unwrap();
        
        container.register_with_deps(
            ServiceLifetime::Transient,
            vec![TypeId::of::<TestService>()],
            |c| {
                let test_service = c.resolve::<TestService>()?;
                Ok(Arc::new(DependentService { test_service }))
            }
        ).unwrap();
        
        let dependent = container.resolve::<DependentService>().unwrap();
        assert_eq!(dependent.test_service.id, 42);
    }
    
    #[test]
    fn test_circular_dependency_detection() {
        let container = Container::new();
        
        container.register_with_deps(
            ServiceLifetime::Singleton,
            vec![TypeId::of::<CircularServiceB>()],
            |c| {
                let b = c.resolve::<CircularServiceB>()?;
                Ok(Arc::new(CircularServiceA { b: Some(b) }))
            }
        ).unwrap();
        
        container.register_with_deps(
            ServiceLifetime::Singleton,
            vec![TypeId::of::<CircularServiceA>()],
            |c| {
                let a = c.resolve::<CircularServiceA>()?;
                Ok(Arc::new(CircularServiceB { a: Some(a) }))
            }
        ).unwrap();
        
        let result = container.validate_dependencies();
        assert!(result.is_err());
        assert!(matches!(result, Err(ContainerError::CircularDependency(_))));
    }
    
    #[test]
    fn test_service_not_found() {
        let container = Container::new();
        let result = container.resolve::<TestService>();
        assert!(matches!(result, Err(ContainerError::ServiceNotFound(_))));
    }
    
    #[test]
    fn test_builder_pattern() {
        let container = Container::builder()
            .singleton(|_| Ok(Arc::new(TestService { id: 100 })))
            .unwrap()
            .transient(|c| {
                let test_service = c.resolve::<TestService>()?;
                Ok(Arc::new(DependentService { test_service }))
            })
            .unwrap()
            .build()
            .unwrap();
        
        let service = container.resolve::<TestService>().unwrap();
        assert_eq!(service.id, 100);
        
        let dependent = container.resolve::<DependentService>().unwrap();
        assert_eq!(dependent.test_service.id, 100);
    }
    
    #[test]
    fn test_clear_singletons() {
        let container = Container::new();
        
        container.register(ServiceLifetime::Singleton, |_| {
            Ok(Arc::new(TestService {
                id: COUNTER.fetch_add(1, Ordering::SeqCst),
            }))
        }).unwrap();
        
        let service1 = container.resolve::<TestService>().unwrap();
        container.clear_singletons();
        let service2 = container.resolve::<TestService>().unwrap();
        
        assert_ne!(service1.id, service2.id);
    }
    
    #[test]
    fn test_is_registered() {
        let container = Container::new();
        
        assert!(!container.is_registered::<TestService>());
        
        container.register(ServiceLifetime::Singleton, |_| {
            Ok(Arc::new(TestService { id: 0 }))
        }).unwrap();
        
        assert!(container.is_registered::<TestService>());
    }
    
    #[test]
    fn test_service_count() {
        let container = Container::new();
        
        assert_eq!(container.service_count(), 0);
        
        container.register(ServiceLifetime::Singleton, |_| {
            Ok(Arc::new(TestService { id: 0 }))
        }).unwrap();
        
        assert_eq!(container.service_count(), 1);
    }
}