#[derive(Debug, serde::Deserialize)]
pub struct ComponentConfig {
    pub component_id: String,
    pub component_type: String,
    pub name: String,
}

#[derive(Debug)]
pub struct ComponentDeserializeHelper {
    pub config: ComponentConfig,
    pub params: Option<serde_json::Value>,
}

impl<'de> serde::Deserialize<'de> for ComponentDeserializeHelper {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::de::Deserializer<'de>,
    {
        let value = serde_json::Value::deserialize(deserializer)?;
        let config: ComponentConfig =
            serde_json::from_value(value.clone()).map_err(serde::de::Error::custom)?;

        let parameters = value.get("parameters");

        Ok(ComponentDeserializeHelper {
            config,
            params: parameters.cloned(),
        })
    }
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
        assert_eq!(helper.params, None);
    }
}
