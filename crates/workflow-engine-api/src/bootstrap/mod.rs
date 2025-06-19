pub mod config;
pub mod container;
pub mod discovery;
pub mod health;
pub mod lifecycle;
pub mod manager;
pub mod registry;
pub mod service;

pub use config::{ConfigurationManager, ServiceConfiguration, ServiceDependency};
pub use container::{Container, ContainerBuilder, ContainerError, Service, ServiceLifetime};
pub use discovery::{ServiceDiscovery, DiscoveryClient, RegistryDiscovery};
pub use health::{HealthMonitor, HealthCheckStrategy, HttpHealthCheck};
pub use lifecycle::{ServiceLifecycleManager, ServiceState, ServiceLifecycleHooks};
pub use manager::{ServiceBootstrapManager, ServiceBootstrapManagerBuilder};
pub use registry::{ServiceRegistry, ServiceInstance, LoadBalancingStrategy};
pub use service::{bootstrap_service, ServiceConfig, BootstrapError};