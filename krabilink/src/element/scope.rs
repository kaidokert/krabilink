use super::ComponentPorts;
use super::{Component, Port};

#[derive(Debug, serde::Deserialize)]
pub struct Scope<'a> {
    #[serde(skip)]
    ports: ComponentPorts<'a, 1, 0>,
}

impl<'a> Default for Scope<'a> {
    fn default() -> Self {
        Self {
            ports: ComponentPorts::default(),
        }
    }
}

impl<'a> Component<'a> for Scope<'a> {
    fn get_input_ports(&mut self) -> &mut [Option<&'a Port>] {
        &mut self.ports.inputs
    }

    fn action(&mut self, _delta_time: f32) {
        let input_value = self.ports.inputs[0].map(|p| p.get()).unwrap_or(0.0);
        log::info!("Scope: {}", input_value);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::PortId;

    #[test]
    fn test_deserialize_from_json() {
        let json = r#"{}"#;
        let scope = serde_json::from_str::<Scope>(json);
        assert_eq!(scope.is_ok(), true);
    }
    #[test]
    fn test_connect_input() {
        let mut scope = Scope::default();
        let port = Port::new(0.0);
        scope.connect_input(PortId(0), &port).unwrap();
        assert_eq!(scope.ports.inputs[0], Some(&port));
    }
}
