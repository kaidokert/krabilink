use super::super::element::Component;
use super::super::element::{
    Comparator, Constant, Gain, InitialCondition, Integrator, Saturate, Scope,
};

// The size of this is going to be whatever the largest component
// is. In no-alloc environment, we can construct needed component but
// pay for the overhead of the largest component.
pub enum AllComponents<'a> {
    Comparator(Comparator<'a>),
    Constant(Constant<'a>),
    Gain(Gain<'a>),
    InitialCondition(InitialCondition<'a>),
    Integrator(Integrator<'a>),
    Saturate(Saturate<'a>),
    Scope(Scope<'a>),
    Empty,
}
impl<'a> Default for AllComponents<'a> {
    fn default() -> Self {
        AllComponents::Empty
    }
}

impl<'a> AllComponents<'a> {
    pub fn from_json(s: &str, json: serde_json::Value) -> Result<Self, serde_json::Error> {
        match s {
            "Comparator" => Ok(AllComponents::Comparator(serde_json::from_value(json)?)),
            "Constant" => Ok(AllComponents::Constant(serde_json::from_value(json)?)),
            "Gain" => Ok(AllComponents::Gain(serde_json::from_value(json)?)),
            "InitialCondition" => Ok(AllComponents::InitialCondition(serde_json::from_value(
                json,
            )?)),
            "Integrator" => Ok(AllComponents::Integrator(serde_json::from_value(json)?)),
            "Saturate" => Ok(AllComponents::Saturate(serde_json::from_value(json)?)),
            "Scope" => Ok(AllComponents::Scope(serde_json::from_value(json)?)),
            _ => Err(serde::de::Error::custom("Invalid component")),
        }
    }
    pub fn from_string(s: &str) -> Self {
        match s {
            "Comparator" => AllComponents::Comparator(Comparator::default()),
            "Constant" => AllComponents::Constant(Constant::default()),
            "Gain" => AllComponents::Gain(Gain::default()),
            "InitialCondition" => AllComponents::InitialCondition(InitialCondition::default()),
            "Integrator" => AllComponents::Integrator(Integrator::default()),
            "Saturate" => AllComponents::Saturate(Saturate::default()),
            "Scope" => AllComponents::Scope(Scope::default()),
            _ => AllComponents::Empty,
        }
    }
}

impl<'a> core::ops::Deref for AllComponents<'a> {
    type Target = dyn Component<'a> + 'a;
    fn deref(&self) -> &Self::Target {
        self.as_component_ref()
    }
}
impl<'a> core::ops::DerefMut for AllComponents<'a> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.as_component_mut()
    }
}

impl<'a> AllComponents<'a> {
    fn as_component_ref(&self) -> &(dyn Component<'a> + 'a) {
        match self {
            AllComponents::Comparator(c) => c,
            AllComponents::Constant(c) => c,
            AllComponents::Gain(c) => c,
            AllComponents::InitialCondition(c) => c,
            AllComponents::Integrator(c) => c,
            AllComponents::Saturate(c) => c,
            AllComponents::Scope(c) => c,
            AllComponents::Empty => panic!("Not a component"),
        }
    }
    fn as_component_mut(&mut self) -> &mut (dyn Component<'a> + 'a) {
        match self {
            AllComponents::Comparator(c) => c,
            AllComponents::Constant(c) => c,
            AllComponents::Gain(c) => c,
            AllComponents::InitialCondition(c) => c,
            AllComponents::Integrator(c) => c,
            AllComponents::Saturate(c) => c,
            AllComponents::Scope(c) => c,
            AllComponents::Empty => panic!("Not a component"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::super::{Port, PortId};
    use super::*;

    #[test]
    fn test_make_array_of_components() {
        let some_ports = [Port::default(), Port::default()];

        let mut components = [
            AllComponents::from_string("Constant"),
            AllComponents::from_string("Gain"),
        ];
        (*components[0])
            .connect_output(PortId(0), &some_ports[0])
            .unwrap();
        (*components[1])
            .connect_input(PortId(0), &some_ports[0])
            .unwrap();
        (*components[1])
            .connect_output(PortId(0), &some_ports[1])
            .unwrap();

        for comp in components.iter_mut() {
            (*comp).action(0.0);
        }
    }
}
