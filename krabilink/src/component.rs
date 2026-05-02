#[derive(Debug, serde::Deserialize)]
pub struct ComponentConfig {
    pub component_id: String,
    pub component_type: String,
    pub name: String,
}

#[derive(Debug, serde::Deserialize)]
pub struct ComponentDeserializeHelper {
    #[serde(flatten)]
    pub config: ComponentConfig,
    pub parameters: Option<serde_json::Value>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deserialize() {
        let json = r#"{"component_id": "id1", "component_type": "type1", "name": "name1"}"#;
        let config: ComponentConfig = serde_json::from_str(json).unwrap();
        assert_eq!(config.component_id, "id1");
        assert_eq!(config.component_type, "type1");
        assert_eq!(config.name, "name1");
    }

    #[test]
    fn test_deserialize_helper() {
        let json = r#"{"component_id": "id1", "component_type": "type1", "name": "name1"}"#;
        let helper: ComponentDeserializeHelper = serde_json::from_str(json).unwrap();
        assert_eq!(helper.config.component_id, "id1");
        assert_eq!(helper.config.component_type, "type1");
        assert_eq!(helper.parameters, None);
    }
}
