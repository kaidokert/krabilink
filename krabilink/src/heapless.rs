mod enum_dispatch;
use heapless::Vec;
use serde::Deserialize;

use crate::Port;

use super::component;
use super::connection;

use enum_dispatch::AllComponents;

#[derive(Debug, Deserialize)]
pub struct Chart<const N: usize, const M: usize> {
    pub components: Vec<component::ComponentDeserializeHelper, N>,
    pub connections: Vec<connection::ConnectionConfig, M>,
}

pub struct ComponentList<'a, const N: usize> {
    pub componentss: Vec<AllComponents<'a>, N>,
}
impl<'a, const N: usize> ComponentList<'a, N> {
    pub fn new() -> Self {
        Self {
            componentss: Vec::new(),
        }
    }
}

pub fn load_chart<'a, const N: usize, const M: usize>(
    json: &str,
    ports: &'a mut Vec<Port, M>,
) -> Result<ComponentList<'a, N>, u8> {
    let chart: Chart<N, M> = serde_json::from_str(json).unwrap();
    let num_connections = chart.connections.len();
    log::info!("Number of connections: {}", num_connections);
    ports.clear();
    ports.extend(core::iter::repeat_with(Port::default).take(num_connections));
    let chart = load_and_resolve_chart(chart, ports)?;
    Ok(chart)
}

pub fn load_and_resolve_chart<'a, const N: usize, const M: usize>(
    chart: Chart<N, M>,
    ports: &'a [Port],
) -> Result<ComponentList<'a, N>, u8> {
    let mut component_list = ComponentList::<'a>::new();
    for component_def in chart.components {
        let id = component_def.config.component_id;
        log::debug!("Loading component {}", id);
        let param_def = component_def.params.unwrap_or_default();
        let comp =
            AllComponents::from_json(&component_def.config.component_type, param_def).unwrap();
        component_list.componentss.push(comp);
    }

    for connection in chart.connections {}
    Ok(component_list)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_foo() {
        let json = r#"{
            "components": [
                {
                    "component_id" : "Constant01",
                    "component_type": "Constant",
                    "name" : "Constant01"
                },
                {
                    "component_id" : "Gain01",
                    "component_type": "Gain",
                    "name" : "Gain01"
                }
            ],

            "connections": [
                {
                    "name": "Connection01",
                    "from": {
                        "component": "Constant01",
                        "out_port": 0
                    },
                    "to": {
                        "component": "Gain01",
                        "in_port": 0
                    }
                }
            ]

        }"#;

        let chart: Chart<6, 5> = serde_json::from_str(json).unwrap();
        println!("{:?}", chart);
    }
}
