use crate::config::Config;
use oxidite_middleware::CorsLayer;
use tower_http::cors::Any;

/// Build a [`CorsLayer`] from the application configuration
pub fn cors_layer_from_config(config: &Config) -> CorsLayer {
    let mut layer = CorsLayer::new();

    if !config.security.cors_origins.is_empty() {
        if config.security.cors_origins.contains(&"*".to_string()) {
            layer = layer.allow_origin(Any);
        } else {
            let origins: Vec<_> = config.security.cors_origins.iter()
                .filter_map(|s| s.parse().ok())
                .collect();
            layer = layer.allow_origin(origins);
        }
    }

    if !config.security.cors_methods.is_empty() {
        let methods: Vec<_> = config.security.cors_methods.iter()
            .filter_map(|s| s.parse().ok())
            .collect();
        layer = layer.allow_methods(methods);
    }

    if !config.security.cors_headers.is_empty() {
        let headers: Vec<_> = config.security.cors_headers.iter()
            .filter_map(|s| s.parse().ok())
            .collect();
        layer = layer.allow_headers(headers);
    }

    layer
}
