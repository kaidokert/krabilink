use core::cell::Cell;

#[derive(Debug, PartialEq)]
pub struct Port {
    data: Cell<f32>,
}

impl Port {
    pub fn new(value: f32) -> Self {
        Self {
            data: Cell::new(value),
        }
    }
    pub fn set(&self, value: f32) {
        self.data.set(value);
    }
    pub fn get(&self) -> f32 {
        self.data.get()
    }
}
impl Default for Port {
    fn default() -> Self {
        Self {
            data: Cell::new(0.0),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_port() {
        let port = Port::new(1.0);
        assert_eq!(port.get(), 1.0);
    }

    #[test]
    fn test_default_port() {
        let port = Port::default();
        assert_eq!(port.get(), 0.0);
    }

    #[test]
    fn test_set_get() {
        let port = Port::default();
        port.set(2.5);
        assert_eq!(port.get(), 2.5);

        port.set(-1.0);
        assert_eq!(port.get(), -1.0);
    }

    #[test]
    fn test_multiple_ports() {
        let port1 = Port::new(1.0);
        let port2 = Port::new(2.0);

        assert_eq!(port1.get(), 1.0);
        assert_eq!(port2.get(), 2.0);

        port1.set(3.0);
        port2.set(4.0);

        assert_eq!(port1.get(), 3.0);
        assert_eq!(port2.get(), 4.0);
    }
}
