// ConnectionEndpoint Struct
#[derive(Debug, serde::Deserialize)]
pub struct ConnectionEndpointOut {
    pub component: String,
    pub out_port: i32, // Changed to i32 to match JSON data
}

// ConnectionEndpoint Struct
#[derive(Debug, serde::Deserialize)]
pub struct ConnectionEndpointIn {
    pub component: String,
    pub in_port: i32, // Changed to i32 to match JSON data
}

// ConnectionConfig Struct
#[derive(Debug, serde::Deserialize)]
pub struct ConnectionConfig {
    pub name: String,
    pub from: ConnectionEndpointOut,
    pub to: ConnectionEndpointIn,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deserialize_connection_endpoint_out() {
        let json = r#"{"component": "comp1", "out_port": 1}"#;
        let endpoint: ConnectionEndpointOut = serde_json::from_str(json).unwrap();
        assert_eq!(endpoint.component, "comp1");
        assert_eq!(endpoint.out_port, 1);
    }

    #[test]
    fn test_deserialize_connection_endpoint_in() {
        let json = r#"{"component": "comp1", "in_port": 1}"#;
        let endpoint: ConnectionEndpointIn = serde_json::from_str(json).unwrap();
        assert_eq!(endpoint.component, "comp1");
        assert_eq!(endpoint.in_port, 1);
    }

    #[test]
    fn test_deserialize_connection_config() {
        let json = r#"{"name": "conn1", "from": {"component": "comp1", "out_port": 1}, "to": {"component": "comp2", "in_port": 2}}"#;
        let config: ConnectionConfig = serde_json::from_str(json).unwrap();
        assert_eq!(config.name, "conn1");
        assert_eq!(config.from.component, "comp1");
        assert_eq!(config.from.out_port, 1);
        assert_eq!(config.to.component, "comp2");
        assert_eq!(config.to.in_port, 2);
    }
}
