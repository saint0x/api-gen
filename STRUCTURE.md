# TRONCH API Management System - Project Structure

## Directory Structure
```
api-gen/
├── crates/                    # Core functionality crates
│   ├── core/                 # Core types and traits
│   │   ├── src/
│   │   │   ├── types.rs     # Shared types
│   │   │   ├── error.rs     # Error types
│   │   │   └── lib.rs
│   │   └── Cargo.toml
│   │
│   ├── keys/                 # API Key management
│   │   ├── src/
│   │   │   ├── generation.rs
│   │   │   ├── validation.rs
│   │   │   ├── storage.rs
│   │   │   ├── rotation.rs
│   │   │   └── lib.rs
│   │   └── Cargo.toml
│   │
│   ├── security/            # Security features
│   │   ├── src/
│   │   │   ├── rate_limit.rs
│   │   │   ├── hashing.rs
│   │   │   └── lib.rs
│   │   └── Cargo.toml
│   │
│   ├── monitoring/          # Monitoring features
│   │   ├── src/
│   │   │   ├── health.rs
│   │   │   ├── metrics.rs
│   │   │   ├── logging.rs
│   │   │   └── lib.rs
│   │   └── Cargo.toml
│   │
│   └── audit/              # Audit logging
│       ├── src/
│       │   ├── logging.rs
│       │   ├── storage.rs
│       │   └── lib.rs
│       └── Cargo.toml
│
├── src/                    # Main API implementation
│   ├── api/
│   │   ├── health.rs      # Health API implementation
│   │   ├── keys.rs        # Key management API implementation
│   │   ├── metrics.rs     # Metrics API implementation
│   │   ├── audit.rs       # Audit API implementation
│   │   └── mod.rs
│   │
│   ├── server/
│   │   ├── router.rs      # Route definitions
│   │   ├── handlers.rs    # HTTP handlers
│   │   ├── middleware.rs  # Server middleware
│   │   └── mod.rs
│   │
│   ├── config.rs          # Configuration management
│   ├── lib.rs             # Main API entry point
│   └── main.rs            # Server binary entry point
│
├── examples/              # Usage examples
│   ├── http_server.rs
│   └── grpc_server.rs
│
└── Cargo.toml            # Workspace manifest
```

## Core API Implementation

### Main API Entry Point
```rust
// src/lib.rs
pub struct Tronch {
    config: Config,
    keys: keys::KeyManager,
    security: security::SecurityManager,
    monitoring: monitoring::MonitoringManager,
    audit: audit::AuditManager,
}

impl Tronch {
    pub fn builder() -> TronchBuilder {
        TronchBuilder::new()
    }

    // Key Management API
    pub fn create_key(&self, config: KeyConfig) -> Result<APIKey>;
    pub fn rotate_key(&self, key: &str) -> Result<APIKey>;
    pub fn validate_key(&self, key: &str) -> Result<KeyStatus>;

    // Health & Metrics API
    pub fn check_health(&self) -> Result<HealthStatus>;
    pub fn get_metrics(&self) -> Result<MetricsSnapshot>;

    // Rate Limiting API
    pub fn check_rate_limit(&self, key: &str) -> Result<RateLimit>;

    // Audit API
    pub fn get_audit_log(&self, filter: AuditFilter) -> Result<Vec<AuditEntry>>;
}
```

### Server Implementation
```rust
// src/server/mod.rs
pub struct TronchServer {
    api: Arc<Tronch>,
    router: Router,
}

impl TronchServer {
    pub fn new(config: ServerConfig) -> Self {
        let api = Arc::new(Tronch::builder()
            .with_config(config)
            .build()
            .expect("Failed to initialize API"));
            
        let router = build_router(api.clone());
        Self { api, router }
    }

    pub fn run(self) -> Result<()>;
}
```

## Component Integration

### Health Monitoring
```rust
// crates/monitoring/src/health.rs
pub struct HealthManager {
    state: Arc<AtomicHealthState>,
    checks: Vec<Box<dyn HealthCheck>>,
}

// src/api/health.rs
impl Tronch {
    pub fn check_health(&self) -> Result<HealthStatus> {
        self.monitoring.check_health()
    }
}
```

### Key Management
```rust
// crates/keys/src/lib.rs
pub struct KeyManager {
    storage: Box<dyn KeyStorage>,
    generator: KeyGenerator,
    validator: KeyValidator,
}

// src/api/keys.rs
impl Tronch {
    pub fn create_key(&self, config: KeyConfig) -> Result<APIKey> {
        self.keys.create(config)
    }
}
```

### Rate Limiting
```rust
// crates/security/src/rate_limit.rs
pub struct RateLimiter {
    storage: Box<dyn RateLimitStorage>,
    config: RateLimitConfig,
}

// src/server/middleware.rs
pub fn rate_limit_middleware(api: Arc<Tronch>) -> impl Middleware {
    move |req, next| {
        api.check_rate_limit(&req.key)?;
        next.handle(req)
    }
}
```

## Configuration Management

### Configuration Structure
```rust
// src/config.rs
pub struct Config {
    server: ServerConfig,
    keys: KeyConfig,
    security: SecurityConfig,
    monitoring: MonitoringConfig,
    audit: AuditConfig,
}

impl Config {
    pub fn from_env() -> Result<Self>;
    pub fn from_file(path: &Path) -> Result<Self>;
}
```

### Builder Pattern
```rust
pub struct TronchBuilder {
    config: Option<Config>,
    features: HashSet<Feature>,
}

impl TronchBuilder {
    pub fn with_config(mut self, config: Config) -> Self;
    pub fn with_key_management(mut self) -> Self;
    pub fn with_rate_limiting(mut self) -> Self;
    pub fn with_monitoring(mut self) -> Self;
    pub fn with_audit_logging(mut self) -> Self;
    pub fn build(self) -> Result<Tronch>;
}
```

## Implementation Benefits

1. **Single Entry Point**: All functionality exposed through `Tronch` struct
2. **Modular Components**: Each crate maintains independence while integrating cleanly
3. **Flexible Configuration**: Builder pattern allows optional features
4. **Clear API Surface**: Well-defined public interfaces
5. **Easy Testing**: Components can be tested independently
6. **Simple Integration**: Easy to add new features
7. **Type Safety**: Strong typing throughout the system
8. **Error Handling**: Consistent error types and handling

## Migration Strategy

1. Create new directory structure
2. Move existing crate functionality into appropriate new locations
3. Create core API implementation
4. Update tests to use new structure
5. Implement configuration management
6. Add server implementation
7. Create examples
8. Update documentation

## Next Steps

1. Implement core API structure
2. Move existing functionality into new structure
3. Add configuration management
4. Create HTTP server implementation
5. Add comprehensive tests
6. Create usage examples
7. Update documentation 