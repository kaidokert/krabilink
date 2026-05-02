use super::ComponentPorts;
use super::{Component, Port};

#[derive(Debug)]
struct IntegratorInternalState {
    accumulator: f32,
    reset_state: bool,
    previous_reset_state: bool,
}
impl Default for IntegratorInternalState {
    fn default() -> Self {
        Self {
            accumulator: 0.0,
            reset_state: false,
            previous_reset_state: true,
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum ExternalResetTrigger {
    None,
    Rising,
    Falling,
    Either,
    Level,
}

impl Default for ExternalResetTrigger {
    fn default() -> Self {
        Self::None
    }
}

impl<'de> serde::Deserialize<'de> for ExternalResetTrigger {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        match s.as_str() {
            "none" => Ok(ExternalResetTrigger::None),
            "rising" => Ok(ExternalResetTrigger::Rising),
            "falling" => Ok(ExternalResetTrigger::Falling),
            "either" => Ok(ExternalResetTrigger::Either),
            "level" => Ok(ExternalResetTrigger::Level),
            _ => Err(serde::de::Error::custom("Invalid operand")),
        }
    }
}

#[derive(Debug, serde::Deserialize)]
pub struct Integrator<'a> {
    output_lower_saturation_limit: Option<f32>,
    output_upper_saturation_limit: Option<f32>,
    #[serde(default)]
    initial_condition: f32,
    #[serde(default)]
    external_reset: ExternalResetTrigger,

    #[serde(skip)]
    internal_state: IntegratorInternalState,

    #[serde(skip)]
    ports: ComponentPorts<'a, 3, 2>,
}

impl<'a> Default for Integrator<'a> {
    fn default() -> Self {
        Self {
            output_lower_saturation_limit: None,
            output_upper_saturation_limit: None,
            initial_condition: 0.0,
            external_reset: ExternalResetTrigger::default(),
            internal_state: IntegratorInternalState::default(),
            ports: ComponentPorts::default(),
        }
    }
}

#[cfg(test)]
impl<'a> Integrator<'a> {
    fn set_initial_condition(&mut self, value: f32) {
        self.initial_condition = value;
    }
}

const FLOAT_ZERO_THRESHOLD: f32 = 0.01;

impl<'a> Component<'a> for Integrator<'a> {
    fn get_input_ports(&mut self) -> &mut [Option<&'a Port>] {
        &mut self.ports.inputs
    }
    fn get_output_ports(&mut self) -> &mut [Option<&'a Port>] {
        &mut self.ports.outputs
    }

    fn action(&mut self, delta_time: f32) {
        if let Some(input_external_reset) = self.ports.inputs[1] {
            let bool_reset = input_external_reset.get() > FLOAT_ZERO_THRESHOLD;
            let prev = self.internal_state.previous_reset_state;
            let should_reset = match self.external_reset {
                ExternalResetTrigger::None => false,
                ExternalResetTrigger::Rising => bool_reset && !prev,
                ExternalResetTrigger::Falling => !bool_reset && prev,
                ExternalResetTrigger::Either => bool_reset != prev,
                ExternalResetTrigger::Level => bool_reset,
            };
            self.internal_state.reset_state = bool_reset;
            if should_reset {
                log::warn!("Reset state changed");
                if let Some(input_initial_condition) = self.ports.inputs[2] {
                    self.internal_state.accumulator = input_initial_condition.get();
                } else {
                    self.internal_state.accumulator = self.initial_condition;
                }
            }
        } else {
            // we are using internal reset
            if self.internal_state.reset_state != self.internal_state.previous_reset_state {
                self.internal_state.accumulator = self.initial_condition;
            }
        }
        self.internal_state.previous_reset_state = self.internal_state.reset_state;

        let input_value = self.ports.inputs[0].map(|p| p.get()).unwrap_or(0.0);
        self.internal_state.accumulator += input_value * delta_time;
        if let Some(non_reset_output) = self.ports.outputs[1] {
            non_reset_output.set(self.internal_state.accumulator);
        }

        if let Some(lower_limit) = self.output_lower_saturation_limit {
            if self.internal_state.accumulator < lower_limit {
                self.internal_state.accumulator = lower_limit;
            }
        }
        if let Some(upper_limit) = self.output_upper_saturation_limit {
            if self.internal_state.accumulator > upper_limit {
                self.internal_state.accumulator = upper_limit;
            }
        }
        if let Some(output) = self.ports.outputs[0] {
            output.set(self.internal_state.accumulator);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::PortId;
    use super::*;

    #[test]
    fn test_deserialize_from_json() {
        let json = r#"{}"#;
        let integrator = serde_json::from_str::<Integrator>(json);
        assert_eq!(integrator.is_ok(), true);

        let json =
            r#"{"output_lower_saturation_limit": 0.0, "output_upper_saturation_limit": 1.0}"#;
        let integrator: Integrator = serde_json::from_str(json).unwrap();
        assert_eq!(integrator.output_lower_saturation_limit, Some(0.0));
        assert_eq!(integrator.output_upper_saturation_limit, Some(1.0));
    }

    #[test]
    fn test_deserialize_from_json_with_external_reset() {
        let json = r#"{"external_reset": "rising"}"#;
        let integrator: Integrator = serde_json::from_str(json).unwrap();
        assert_eq!(integrator.external_reset, ExternalResetTrigger::Rising);
        let integrator = serde_json::from_str::<Integrator>(
            r#"{"external_reset": "falling", "initial_condition": 5.0}"#,
        )
        .unwrap();
        assert_eq!(integrator.external_reset, ExternalResetTrigger::Falling);
        assert_eq!(integrator.initial_condition, 5.0);
    }

    // This test is without external resets or initial conditions
    #[test]
    fn test_simple_integrator() {
        let mut integrator = Integrator::default();
        integrator.set_initial_condition(0.0);
        let ports = [Port::new(0.0), Port::new(0.0)];
        integrator.connect_input(PortId(0), &ports[0]).unwrap();
        integrator.connect_output(PortId(0), &ports[1]).unwrap();
        integrator.action(1.0);
        assert_eq!(ports[1].get(), 0.0);
        integrator.action(1.0);
        assert_eq!(ports[1].get(), 0.0);
        ports[0].set(2.0);
        integrator.action(1.0);
        assert_eq!(ports[1].get(), 2.0);
        integrator.action(1.0);
        assert_eq!(ports[1].get(), 4.0);
        integrator.action(0.5);
        assert_eq!(ports[1].get(), 5.0);
    }
    #[test]
    fn test_initial_condition() {
        let mut integrator = Integrator::default();
        integrator.set_initial_condition(1.0);
        let ports = [Port::new(0.0), Port::new(0.0)];
        integrator.connect_input(PortId(0), &ports[0]).unwrap();
        integrator.connect_output(PortId(0), &ports[1]).unwrap();
        integrator.action(1.0);
        assert_eq!(ports[1].get(), 1.0);
    }

    #[test]
    fn test_external_reset() {
        let mut integrator = Integrator::default();
        integrator.set_initial_condition(1.0);
        let ports = [Port::new(0.0), Port::new(0.0), Port::new(0.0)];
        integrator.connect_input(PortId(0), &ports[0]).unwrap();
        integrator.connect_input(PortId(0), &ports[1]).unwrap();
        integrator.connect_output(PortId(0), &ports[2]).unwrap();
        integrator.action(1.0);
        assert_eq!(ports[1].get(), 0.0);
    }
}
