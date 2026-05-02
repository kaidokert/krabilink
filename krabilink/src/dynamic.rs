use super::component;
use super::connection;
use super::element::{Component, Port, PortId};

mod dyn_dispatch;

#[derive(Debug, serde::Deserialize)]
pub struct Chart {
    pub components: Vec<component::ComponentDeserializeHelper>,
    pub connections: Vec<connection::ConnectionConfig>,
}

pub struct ComponentList<'a> {
    pub components: Vec<(Box<dyn Component<'a> + 'a>, String)>,
}
impl<'a> ComponentList<'a> {
    pub fn new() -> Self {
        Self {
            components: Vec::new(),
        }
    }
}

#[derive(Debug)]
pub enum LoadErr {
    JsonError(serde_json::Error),
    LoadError(u8),
    UnknownComponent(String),
    PortTargetNotFound(PortId, String),
    PortConnectionError(PortId, String),
}

impl From<serde_json::Error> for LoadErr {
    fn from(error: serde_json::Error) -> Self {
        Self::JsonError(error)
    }
}

impl From<u8> for LoadErr {
    fn from(error: u8) -> Self {
        Self::LoadError(error)
    }
}

pub fn load_chart<'a>(json: &str, ports: &'a mut Vec<Port>) -> Result<ComponentList<'a>, LoadErr> {
    let chart: Chart = serde_json::from_str(json)?;
    let num_connections = chart.connections.len();
    log::info!("Number of connections: {}", num_connections);
    ports.clear();
    ports.extend(core::iter::repeat_with(Port::default).take(num_connections));
    let chart = load_and_resolve_chart(chart, ports)?;
    Ok(chart)
}

pub fn load_and_resolve_chart<'a>(
    chart: Chart,
    ports: &'a [Port], // Changed to immutable reference since we use Cell
) -> Result<ComponentList<'a>, LoadErr> {
    let factory_registry = dyn_dispatch::initialize_component_factories::<'a>();

    let mut component_list = ComponentList::<'a>::new();
    for component_def in chart.components {
        let id: String = component_def.config.component_id;
        log::debug!("Loading component {}", id);
        let factory = factory_registry
            .iter()
            .find(|(name, _)| name == &component_def.config.component_type)
            .ok_or(LoadErr::UnknownComponent(
                component_def.config.component_type,
            ))?;

        let param_def = component_def.params.unwrap_or_default();
        let component = (factory.1)(param_def).map_err(|_| 1)?;
        component_list.components.push((component, id));
    }

    // Iterate and call connect_input and connect_output
    for (i, connection) in chart.connections.iter().enumerate() {
        log::info!("Connection: {:?}", connection.name);
        let from_port_id = PortId(connection.from.out_port as usize);
        let to_port_id = PortId(connection.to.in_port as usize);
        let found_from = component_list
            .components
            .iter_mut()
            .find(|(_, id)| id == &connection.from.component);
        if let Some(from) = found_from {
            from.0
                .connect_output(from_port_id, &ports[i])
                .map_err(|_| {
                    LoadErr::PortConnectionError(from_port_id, connection.from.component.clone())
                })?;
        } else {
            return Err(LoadErr::PortTargetNotFound(
                from_port_id,
                connection.from.component.clone(),
            ));
        }
        let found_to = component_list
            .components
            .iter_mut()
            .find(|(_, id)| id == &connection.to.component);
        if let Some(to) = found_to {
            to.0.connect_input(to_port_id, &ports[i]).map_err(|_| {
                LoadErr::PortConnectionError(to_port_id, connection.to.component.clone())
            })?;
        } else {
            return Err(LoadErr::PortTargetNotFound(
                to_port_id,
                connection.to.component.clone(),
            ));
        }
    }

    Ok(component_list)
}

pub fn run_simulation(chart: &mut ComponentList, dt: f32, num_steps: usize) {
    for i in 0..num_steps {
        log::warn!("\n---- Step: {} time: {} --- \n", i, i as f32 * dt);
        for (component, comp_id) in &mut chart.components {
            log::info!("Component: {}", comp_id);
            component.action(dt);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::element::{Component, Constant, Gain, Port, PortId};
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
