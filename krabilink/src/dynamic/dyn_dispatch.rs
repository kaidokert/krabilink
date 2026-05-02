use super::super::element::Component;
use super::super::element::{
    Comparator, Constant, Gain, InitialCondition, Integrator, Saturate, Scope,
};

pub fn from_json_gen<'a, T>(
    json: serde_json::Value,
) -> Result<Box<dyn Component<'a> + 'a>, serde_json::Error>
where
    T: serde::de::DeserializeOwned + Component<'a> + 'a,
{
    let me: T = serde_json::from_value(json)?;
    Ok(Box::new(me))
}

type FactoryFn<'a> =
    fn(serde_json::Value) -> Result<Box<dyn Component<'a> + 'a>, serde_json::Error>;

pub fn initialize_component_factories<'a>() -> Vec<(String, FactoryFn<'a>)> {
    vec![
        ("Comparator".to_string(), from_json_gen::<Comparator>),
        ("Constant".to_string(), from_json_gen::<Constant>),
        ("Gain".to_string(), from_json_gen::<Gain>),
        (
            "InitialCondition".to_string(),
            from_json_gen::<InitialCondition>,
        ),
        ("Integrator".to_string(), from_json_gen::<Integrator>),
        ("Saturate".to_string(), from_json_gen::<Saturate>),
        ("Scope".to_string(), from_json_gen::<Scope>),
    ]
}

#[cfg(test)]
mod tests {
    use super::super::{Port, PortId};
    use super::*;

    #[test]
    fn test_gain() {
        let ports: [Port; 2] = core::array::from_fn(|_| Port::default());

        let mut constant = Constant::default();
        constant.set_value(1.5);
        constant.connect_output(PortId(0), &ports[0]).unwrap();

        let mut gain = Gain::default();
        gain.set_value(2.0);
        gain.connect_input(PortId(0), &ports[0]).unwrap();
        gain.connect_output(PortId(0), &ports[1]).unwrap();

        // move into array
        let mut components: [&mut dyn Component; 2] = [&mut constant, &mut gain];

        for component in &mut components {
            component.action(1.0);
        }
        assert_eq!(ports[1].get(), 3.0);
    }
}
