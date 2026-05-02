mod port;
pub use port::Port;

mod component;
pub use component::Component;
pub use component::ComponentPorts;

pub mod constant;
pub use constant::Constant;

pub mod gain;
pub use gain::Gain;

pub mod comparator;
pub use comparator::Comparator;

pub mod initial_condition;
pub use initial_condition::InitialCondition;

pub mod saturate;
pub use saturate::Saturate;

pub mod scope;
pub use scope::Scope;

pub mod integrator;
pub use integrator::Integrator;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PortId(pub usize);

#[derive(Debug)]
pub enum Error {
    InvalidPortId(PortId),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn components_can_be_put_in_a_vector() {
        let mut components: Vec<Box<dyn Component>> = Vec::new();
        components.push(Box::new(Constant::default()));
        components.push(Box::new(Gain::default()));
    }
}
