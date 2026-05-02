use super::ComponentPorts;
use super::{Component, Port};

#[derive(Debug)]
pub enum Operand {
    Equal,
    NotEqual,
    LessThan,
    GreaterThan,
    LessThanOrEqual,
    GreaterThanOrEqual,
}

impl<'de> serde::Deserialize<'de> for Operand {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        match s.as_str() {
            "==" => Ok(Operand::Equal),
            "~=" => Ok(Operand::NotEqual),
            "<" => Ok(Operand::LessThan),
            ">" => Ok(Operand::GreaterThan),
            "<=" => Ok(Operand::LessThanOrEqual),
            ">=" => Ok(Operand::GreaterThanOrEqual),
            _ => Err(serde::de::Error::custom("Invalid operand")),
        }
    }
}

#[derive(Debug, serde::Deserialize)]
pub struct Comparator<'a> {
    operand: Operand,
    #[serde(skip)]
    ports: ComponentPorts<'a, 2, 1>,
}

impl<'a> Default for Comparator<'a> {
    fn default() -> Self {
        Self {
            operand: Operand::Equal,
            ports: ComponentPorts::default(),
        }
    }
}

#[cfg(test)]
impl<'a> Comparator<'a> {
    pub fn set_operand(&mut self, operand: Operand) {
        self.operand = operand;
    }
}

impl<'a> Component<'a> for Comparator<'a> {
    fn get_input_ports(&mut self) -> &mut [Option<&'a Port>] {
        &mut self.ports.inputs
    }
    fn get_output_ports(&mut self) -> &mut [Option<&'a Port>] {
        &mut self.ports.outputs
    }

    fn action(&mut self, _delta_time: f32) {
        fn bool_to_f32(b: bool) -> f32 {
            if b {
                1.0
            } else {
                0.0
            }
        }

        let a = self.ports.inputs[0].map(|p| p.get()).unwrap_or(0.0);
        let b = self.ports.inputs[1].map(|p| p.get()).unwrap_or(0.0);
        let bool_res = match self.operand {
            Operand::Equal => a == b,
            Operand::NotEqual => a != b,
            Operand::LessThan => a < b,
            Operand::GreaterThan => a > b,
            Operand::LessThanOrEqual => a <= b,
            Operand::GreaterThanOrEqual => a >= b,
        };
        if let Some(output) = self.ports.outputs[0] {
            output.set(bool_to_f32(bool_res));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::PortId;

    #[test]
    fn test_deserialize_from_json() {
        let json = r#"{"operand": "=="}"#;
        let comparator: Comparator = serde_json::from_str(json).unwrap();
        match comparator.operand {
            Operand::Equal => (),
            _ => panic!("Expected Equal operand"),
        }
    }

    #[test]
    fn test_comparator() {
        let ports = [Port::default(), Port::default(), Port::default()];
        let mut comparator = Comparator::default();
        comparator.set_operand(Operand::Equal);
        comparator.connect_input(PortId(0), &ports[0]).unwrap();
        comparator.connect_input(PortId(1), &ports[1]).unwrap();
        comparator.connect_output(PortId(0), &ports[2]).unwrap();
        comparator.action(1.0);
        assert_eq!(ports[2].get(), 1.0);
        comparator.set_operand(Operand::NotEqual);
        comparator.action(1.0);
        assert_eq!(ports[2].get(), 0.0);
        ports[0].set(1.0);
        ports[1].set(2.0);
        comparator.set_operand(Operand::LessThan);
        comparator.action(1.0);
        assert_eq!(ports[2].get(), 1.0);
    }
}
