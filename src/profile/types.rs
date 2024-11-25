#[derive(Debug, Default, Clone, Copy, PartialEq, PartialOrd)]
#[repr(transparent)]
pub struct Flow(pub f64);
impl Flow {
    pub fn new(v: f64) -> Self {
        Self(v)
    }
}
impl From<f64> for Flow {
    fn from(value: f64) -> Self {
        Self(value * 10.0)
    }
}

impl From<Flow> for f64 {
    fn from(val: Flow) -> Self {
        val.0 / 10.0
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, PartialOrd)]
#[repr(transparent)]
pub struct Pressure(f64);
impl Pressure {
    pub fn new(v: f64) -> Self {
        Self(v)
    }
}
impl From<f64> for Pressure {
    fn from(value: f64) -> Self {
        Self(value * 10.0)
    }
}
impl From<Pressure> for f64 {
    fn from(val: Pressure) -> Self {
        val.0 / 10.0
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, PartialOrd)]
#[repr(transparent)]
pub struct Percent(f64);
impl Percent {
    pub fn new(v: f64) -> Self {
        Self(v)
    }
}
impl From<f64> for Percent {
    fn from(value: f64) -> Self {
        Self(value)
    }
}
impl From<Percent> for f64 {
    fn from(val: Percent) -> Self {
        val.0
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, PartialOrd)]
#[repr(transparent)]
pub struct Temp(f64);
impl Temp {
    pub fn new(v: f64) -> Self {
        Self(v)
    }
}
impl From<f64> for Temp {
    fn from(value: f64) -> Self {
        Self(value * 10.0)
    }
}

impl From<Temp> for f64 {
    fn from(val: Temp) -> Self {
        val.0 / 10.0
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, PartialOrd)]
#[repr(transparent)]
pub struct Weight(f64);
impl Weight {
    pub fn new(v: f64) -> Self {
        Self(v)
    }
}
impl From<f64> for Weight {
    fn from(value: f64) -> Self {
        Self(value * 10.0)
    }
}

impl From<Weight> for f64 {
    fn from(val: Weight) -> Self {
        val.0 / 10.0
    }
}
