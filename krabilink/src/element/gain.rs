use super::ComponentPorts;
use super::{Component, Port};

#[derive(Debug, serde::Deserialize)]
pub struct Gain<'a> {
    value: f32,
    #[serde(skip)]
    ports: ComponentPorts<'a, 1, 1>,
}
impl<'a> Default for Gain<'a> {
    fn default() -> Self {
        Self {
            value: 1.0,
            ports: ComponentPorts::default(),
        }
    }
}

#[cfg(test)]
impl<'a> Gain<'a> {
    pub fn set_value(&mut self, value: f32) {
        self.value = value;
    }
}

impl<'a> Component<'a> for Gain<'a> {
    fn get_input_ports(&mut self) -> &mut [Option<&'a Port>] {
        &mut self.ports.inputs
    }
    fn get_output_ports(&mut self) -> &mut [Option<&'a Port>] {
        &mut self.ports.outputs
    }

    fn action(&mut self, _delta_time: f32) {
        if let Some(input) = self.ports.inputs[0] {
            if let Some(output) = self.ports.outputs[0] {
                output.set(input.get() * self.value);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::PortId;

    #[test]
    fn test_gain() {
        let mut gain = Gain::default();
        gain.set_value(2.0);
        let port = [Port::default(), Port::default()];
        port[0].set(2.5);
        gain.connect_input(PortId(0), &port[0]).unwrap();
        gain.connect_output(PortId(0), &port[1]).unwrap();
        gain.action(1.0);
        assert_eq!(port[1].get(), 5.0);
    }

    #[test]
    fn test_deserialize_from_json() {
        let json = r#"{"value": 1.5}"#;
        let gain: Gain = serde_json::from_str(json).unwrap();
        assert_eq!(gain.value, 1.5);
    }
}
