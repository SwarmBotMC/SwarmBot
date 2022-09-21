#[derive(Debug, PartialEq)]
pub struct Speed {
    multiplier: f64,
}

impl Default for Speed {
    fn default() -> Self {
        Speed::STOP
    }
}

impl Speed {
    const fn new(multiplier: f64) -> Self {
        Self { multiplier }
    }

    pub const SPRINT: Speed = Speed::new(1.3);
    pub const WALK: Speed = Speed::new(1.0);
    pub const SNEAK: Speed = Speed::new(0.3);
    pub const STOP: Speed = Speed::new(0.);

    pub fn multiplier(&self) -> f64 {
        self.multiplier * 0.98 // TODO: differnet at 45 degree angle
    }
}
