use super::ComponentPorts;
use super::{Component, Port};

#[derive(Debug, serde::Deserialize)]
pub struct Saturate<'a> {
    lower_limit: f32,
    upper_limit: f32,
    #[serde(skip)]
    ports: ComponentPorts<'a, 1, 1>,
}

impl<'a> Default for Saturate<'a> {
    fn default() -> Self {
        Self {
            lower_limit: 0.0,
            upper_limit: 1.0,
            ports: ComponentPorts::default(),
        }
    }
}

#[cfg(test)]
impl<'a> Saturate<'a> {
    pub fn set_limits(&mut self, lower_limit: f32, upper_limit: f32) {
        self.lower_limit = lower_limit;
        self.upper_limit = upper_limit;
    }
}

impl<'a> Component<'a> for Saturate<'a> {
    fn get_input_ports(&mut self) -> &mut [Option<&'a Port>] {
        &mut self.ports.inputs
    }
    fn get_output_ports(&mut self) -> &mut [Option<&'a Port>] {
        &mut self.ports.outputs
    }

    fn action(&mut self, _delta_time: f32) {
        let input_value = self.ports.inputs[0].unwrap().get();
        let output_value = input_value.clamp(self.lower_limit, self.upper_limit);
        self.ports.outputs[0].unwrap().set(output_value);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::PortId;

    #[test]
    fn test_deserialize_from_json() {
        let json = r#"{"lower_limit": 0.0, "upper_limit": 1.0}"#;
        let saturate: Saturate = serde_json::from_str(json).unwrap();
        assert_eq!(saturate.lower_limit, 0.0);
        assert_eq!(saturate.upper_limit, 1.0);
    }
    #[test]
    fn test_saturate() {
        let mut saturate = Saturate::default();
        saturate.set_limits(0.0, 1.0);
        let mut ports = [Port::default(), Port::default()];
        saturate.connect_input(PortId(0), &ports[0]).unwrap();
        saturate.connect_output(PortId(0), &ports[1]).unwrap();
        ports[0].set(0.5);
        saturate.action(1.0);
        assert_eq!(ports[1].get(), 0.5);
        ports[0].set(1.5);
        saturate.action(1.0);
        assert_eq!(ports[1].get(), 1.0);
        ports[0].set(-0.5);
        saturate.action(1.0);
        assert_eq!(ports[1].get(), 0.0);
    }
}
