use super::ComponentPorts;
use super::{Component, Port};

#[derive(Debug, serde::Deserialize)]
pub struct InitialCondition<'a> {
    value: f32,
    #[serde(skip)]
    initial_value_sent: bool,
    #[serde(skip)]
    ports: ComponentPorts<'a, 1, 1>,
}

impl<'a> Default for InitialCondition<'a> {
    fn default() -> Self {
        Self {
            value: 0.0,
            initial_value_sent: false,
            ports: ComponentPorts::default(),
        }
    }
}

#[cfg(test)]
impl<'a> InitialCondition<'a> {
    pub fn set_value(&mut self, value: f32) {
        self.value = value;
    }
}

impl<'a> Component<'a> for InitialCondition<'a> {
    fn get_input_ports(&mut self) -> &mut [Option<&'a Port>] {
        &mut self.ports.inputs
    }
    fn get_output_ports(&mut self) -> &mut [Option<&'a Port>] {
        &mut self.ports.outputs
    }

    fn action(&mut self, _delta_time: f32) {
        let send_value = if self.initial_value_sent {
            self.ports.inputs[0].unwrap().get()
        } else {
            self.initial_value_sent = true;
            self.value
        };
        self.ports.outputs[0].unwrap().set(send_value);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::PortId;

    #[test]
    fn test_deserialize_from_json() {
        let json = r#"{"value": 1.0}"#;
        let initial_condition: InitialCondition = serde_json::from_str(json).unwrap();
        assert_eq!(initial_condition.value, 1.0);
    }

    #[test]
    fn test_initial_condition() {
        let mut initial_condition = InitialCondition::default();
        initial_condition.set_value(1.0);
        let ports: [Port; 2] = [Port::default(), Port::default()];
        ports[0].set(5.0);
        ports[1].set(6.0);
        initial_condition
            .connect_input(PortId(0), &ports[0])
            .unwrap();
        initial_condition
            .connect_output(PortId(0), &ports[1])
            .unwrap();
        initial_condition.action(0.0);
        assert_eq!(ports[1].get(), 1.0);
        initial_condition.action(0.0);
        assert_eq!(ports[1].get(), 5.0);
    }
}
