use super::ComponentPorts;
use super::{Component, Port};

#[derive(Debug, serde::Deserialize)]
pub struct Constant<'a> {
    value: f32,
    #[serde(skip)]
    ports: ComponentPorts<'a, 0, 1>,
}

impl<'a> Default for Constant<'a> {
    fn default() -> Self {
        Self {
            value: 0.0,
            ports: ComponentPorts::default(),
        }
    }
}

#[cfg(test)]
impl<'a> Constant<'a> {
    pub fn set_value(&mut self, value: f32) {
        self.value = value;
    }
}

impl<'a> Component<'a> for Constant<'a> {
    fn get_output_ports(&mut self) -> &mut [Option<&'a Port>] {
        &mut self.ports.outputs
    }
    fn action(&mut self, _delta_time: f32) {
        if let Some(output) = self.ports.outputs[0] {
            output.set(self.value)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::super::{Port, PortId};
    use super::*;

    #[test]
    fn test_constant() {
        let mut constant = Constant::default();
        constant.set_value(1.5);
        let port = [Port::default()];
        constant.connect_output(PortId(0), &port[0]).unwrap();
        constant.action(1.0);
        assert_eq!(port[0].get(), 1.5);
    }

    #[test]
    fn test_deserialize_from_json() {
        let json = r#"{"value": 1.5}"#;
        let constant: Constant = serde_json::from_str(json).unwrap();
        assert_eq!(constant.value, 1.5);
    }
}
