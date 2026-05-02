use super::Port;
use super::{Error, PortId};

// Generic ports container
#[derive(Debug)]
pub struct ComponentPorts<'a, const IN: usize, const OUT: usize> {
    pub inputs: [Option<&'a Port>; IN],
    pub outputs: [Option<&'a Port>; OUT],
}
impl<'a, const IN: usize, const OUT: usize> Default for ComponentPorts<'a, IN, OUT> {
    fn default() -> Self {
        Self {
            inputs: [None; IN],
            outputs: [None; OUT],
        }
    }
}

pub trait Component<'a> {
    fn get_input_ports(&mut self) -> &mut [Option<&'a Port>] {
        &mut []
    }
    fn get_output_ports(&mut self) -> &mut [Option<&'a Port>] {
        &mut []
    }

    fn action(&mut self, _delta_time: f32) {}
    fn connect_input(&mut self, id: PortId, port: &'a Port) -> Result<(), Error> {
        let input_ports = self.get_input_ports();
        if id.0 >= input_ports.len() {
            return Err(Error::InvalidPortId(id));
        }
        input_ports[id.0] = Some(port);
        Ok(())
    }

    fn connect_output(&mut self, id: PortId, port: &'a Port) -> Result<(), Error> {
        let output_ports = self.get_output_ports();
        if id.0 >= output_ports.len() {
            return Err(Error::InvalidPortId(id));
        }
        output_ports[id.0] = Some(port);
        Ok(())
    }
}
