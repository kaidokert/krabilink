mod enum_dispatch;
use heapless::Vec;
use serde::Deserialize;

use crate::element::PortId;
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
    pub components: Vec<(AllComponents<'a>, String), N>,
}
impl<'a, const N: usize> ComponentList<'a, N> {
    pub fn new() -> Self {
        Self {
            components: Vec::new(),
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
        let id = component_def.config.component_id.clone();
        log::debug!("Loading component {}", id);
        let param_def = component_def
            .params
            .unwrap_or_else(|| serde_json::Value::Object(Default::default()));
        let comp =
            AllComponents::from_json(&component_def.config.component_type, param_def).unwrap();
        component_list
            .components
            .push((comp, id))
            .map_err(|_| 2u8)?;
    }

    for (i, connection) in chart.connections.iter().enumerate() {
        log::info!("Connection: {:?}", connection.name);
        let from_port_id = PortId(connection.from.out_port as usize);
        let to_port_id = PortId(connection.to.in_port as usize);

        let found_from = component_list
            .components
            .iter_mut()
            .find(|(_, id)| id == &connection.from.component);
        if let Some((ref mut comp, _)) = found_from {
            comp.connect_output(from_port_id, &ports[i])
                .map_err(|_| 3u8)?;
        } else {
            return Err(4);
        }

        let found_to = component_list
            .components
            .iter_mut()
            .find(|(_, id)| id == &connection.to.component);
        if let Some((ref mut comp, _)) = found_to {
            comp.connect_input(to_port_id, &ports[i])
                .map_err(|_| 5u8)?;
        } else {
            return Err(6);
        }
    }

    Ok(component_list)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::element::Component;

    const CHART_JSON: &str = r#"{
        "components": [
            {
                "component_id" : "Constant01",
                "component_type": "Constant",
                "name" : "Constant01",
                "parameters": { "value": 1.5 }
            },
            {
                "component_id" : "Gain01",
                "component_type": "Gain",
                "name" : "Gain01",
                "parameters": { "value": 2.0 }
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

    #[test]
    fn test_deserialize_chart() {
        let chart: Chart<6, 5> = serde_json::from_str(CHART_JSON).unwrap();
        assert_eq!(chart.components.len(), 2);
        assert_eq!(chart.connections.len(), 1);
    }

    #[test]
    fn test_load_and_resolve_chart() {
        let chart: Chart<6, 5> = serde_json::from_str(CHART_JSON).unwrap();
        let ports: Vec<Port, 5> = Vec::new();
        // Need one port per connection
        let mut ports = ports;
        ports.push(Port::default()).unwrap();

        let result = load_and_resolve_chart(chart, &ports);
        assert!(result.is_ok());
        let component_list = result.unwrap();
        assert_eq!(component_list.components.len(), 2);
        assert_eq!(component_list.components[0].1, "Constant01");
        assert_eq!(component_list.components[1].1, "Gain01");
    }

    #[test]
    fn test_load_and_resolve_chart_runs_simulation() {
        let chart: Chart<6, 5> = serde_json::from_str(CHART_JSON).unwrap();
        let mut ports: Vec<Port, 5> = Vec::new();
        ports.push(Port::default()).unwrap();

        let mut component_list = load_and_resolve_chart(chart, &ports).unwrap();

        // Run one simulation step:
        // Constant(1.5) -> port[0] -> Gain(2.0)
        for (comp, _) in component_list.components.iter_mut() {
            comp.action(1.0);
        }
        // The connection port carries the value 1.5 from Constant to Gain input
        assert_eq!(ports[0].get(), 1.5);
    }

    #[test]
    fn test_load_chart_full() {
        let mut ports: Vec<Port, 5> = Vec::new();
        let result = load_chart::<6, 5>(CHART_JSON, &mut ports);
        assert!(result.is_ok());
        let component_list = result.unwrap();
        assert_eq!(component_list.components.len(), 2);
        // load_chart should have sized ports to match connections
        drop(component_list);
        assert_eq!(ports.len(), 1);
    }

    #[test]
    fn test_load_and_resolve_chart_missing_component() {
        let json = r#"{
            "components": [
                {
                    "component_id" : "Constant01",
                    "component_type": "Constant",
                    "name" : "Constant01",
                    "parameters": { "value": 1.0 }
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
                        "component": "Missing",
                        "in_port": 0
                    }
                }
            ]
        }"#;
        let chart: Chart<6, 5> = serde_json::from_str(json).unwrap();
        let mut ports: Vec<Port, 5> = Vec::new();
        ports.push(Port::default()).unwrap();
        let result = load_and_resolve_chart(chart, &ports);
        assert!(result.is_err());
    }
}
