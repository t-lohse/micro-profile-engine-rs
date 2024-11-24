use crate::profile::{Flow, Pressure, Temp, Weight};

pub trait SensorState {
    fn piston_position(&self) -> f64;
    fn piston_speed(&self) -> f64;
    fn water_temp(&self) -> f64;
    fn cylinder_temperature(&self) -> f64;
    fn external_temperature_1(&self) -> f64;
    fn external_temperature_2(&self) -> f64;
    fn tube_temperature(&self) -> f64;
    fn plunger_temperature(&self) -> f64;
    fn water_flow(&self) -> f64;
    fn water_pressure(&self) -> f64;
    fn predictive_temperature(&self) -> f64;
    fn weight(&self) -> f64;
    fn temperature_up(&self) -> f64;
    fn temperature_middle_up(&self) -> f64;
    fn temperature_middle_down(&self) -> f64;
    fn temperature_down(&self) -> f64;
    fn output_position(&self) -> f64;
    fn motor_encoder(&self) -> f64; //DEPRECATED
    fn stable_temperature(&self) -> f64; //DEPRECATED
    fn has_water(&self) -> bool;
}

#[derive(Default, Clone, Debug)]
pub struct Driver<T: SensorState> {
    sensors: T,
}

impl<T: SensorState> Driver<T> {
    pub fn sensor_data(&self) -> &T {
        &self.sensors
    }

    pub fn get_button_gesture(&self, _source: &str, _gesture: &str) -> bool {
        false
    }

    // -------- HW ---------
    pub fn heating_finished(&self) -> bool {
        hardware_connection::heating_finished()
    }

    pub fn has_reached_final_weight(&self) -> bool {
        hardware_connection::has_reached_final_weight()
    }

    pub fn set_target_weight(&self, set_point: Weight) {
        hardware_connection::set_target_weight(set_point)
    }

    pub fn set_target_temperature(&self, set_point: Temp) {
        hardware_connection::set_target_temperature(set_point)
    }

    pub fn set_target_pressure(&self, set_point: Pressure) {
        hardware_connection::set_target_pressure(set_point)
    }

    pub fn set_limited_pressure(&self, set_point: Pressure) {
        hardware_connection::set_limited_pressure(set_point)
    }

    pub fn set_target_flow(&self, set_point: Flow) {
        hardware_connection::set_target_flow(set_point)
    }

    pub fn set_limited_flow(&self, set_point: Flow) {
        hardware_connection::set_limited_flow(set_point)
    }

    pub fn set_target_power(&self, set_point: f64) {
        hardware_connection::set_target_power(set_point)
    }

    pub fn set_target_piston_position(&self, set_point: f64) {
        hardware_connection::set_target_piston_position(set_point)
    }
}

#[derive(Clone, Debug)]
pub struct DummySensorState {
    pub piston_position: Box<f64>,
    pub piston_speed: f64,
    pub water_temp: f64,
    pub cylinder_temperature: f64,
    pub external_temperature_1: f64,
    pub external_temperature_2: f64,
    pub tube_temperature: f64,
    pub plunger_temperature: f64,
    pub water_flow: f64,
    pub water_pressure: f64,
    pub predictive_temperature: f64,
    pub weight: f64,
    pub temperature_up: f64,
    pub temperature_middle_up: f64,
    pub temperature_middle_down: f64,
    pub temperature_down: f64,
    pub output_position: f64,
    pub motor_encoder: f64,      //DEPRECATED
    pub stable_temperature: f64, //DEPRECATED
    pub has_water: bool,
}

impl Default for DummySensorState {
    fn default() -> Self {
        let v = Box::new(0.0);
        //let ptr = Box::leak(v) as *mut f64;
        Self {
            piston_position: v,
            piston_speed: 0.0,
            water_temp: 0.0,
            cylinder_temperature: 0.0,
            external_temperature_1: 0.0,
            external_temperature_2: 0.0,
            tube_temperature: 0.0,
            plunger_temperature: 0.0,
            water_flow: 0.0,
            water_pressure: 0.0,
            predictive_temperature: 0.0,
            weight: 0.0,
            temperature_up: 0.0,
            temperature_middle_up: 0.0,
            temperature_middle_down: 0.0,
            temperature_down: 0.0,
            output_position: 0.0,
            motor_encoder: 0.0,
            stable_temperature: 0.0,
            has_water: false,
        }
    }
}

impl SensorState for DummySensorState {
    fn piston_position(&self) -> f64 {
        println!("Piston pos: {}", *self.piston_position);
        *self.piston_position
    }

    fn piston_speed(&self) -> f64 {
        self.piston_speed
    }

    fn water_temp(&self) -> f64 {
        self.water_temp
    }

    fn cylinder_temperature(&self) -> f64 {
        self.cylinder_temperature
    }

    fn external_temperature_1(&self) -> f64 {
        self.external_temperature_1
    }

    fn external_temperature_2(&self) -> f64 {
        self.external_temperature_2
    }

    fn tube_temperature(&self) -> f64 {
        self.tube_temperature
    }

    fn plunger_temperature(&self) -> f64 {
        self.plunger_temperature
    }

    fn water_flow(&self) -> f64 {
        self.water_flow
    }

    fn water_pressure(&self) -> f64 {
        self.water_pressure
    }

    fn predictive_temperature(&self) -> f64 {
        self.predictive_temperature
    }

    fn weight(&self) -> f64 {
        self.weight
    }

    fn temperature_up(&self) -> f64 {
        self.temperature_up
    }

    fn temperature_middle_up(&self) -> f64 {
        self.temperature_middle_up
    }

    fn temperature_middle_down(&self) -> f64 {
        self.temperature_middle_down
    }

    fn temperature_down(&self) -> f64 {
        self.temperature_down
    }

    fn output_position(&self) -> f64 {
        self.output_position
    }

    fn motor_encoder(&self) -> f64 {
        self.motor_encoder
    }

    fn stable_temperature(&self) -> f64 {
        self.stable_temperature
    }

    fn has_water(&self) -> bool {
        self.has_water
    }
}

mod hardware_connection {
    use crate::profile::{Flow, Pressure, Temp, Weight};

    pub fn heating_finished() -> bool {
        true
    }

    pub fn has_reached_final_weight() -> bool {
        false
    }

    pub fn set_target_weight(set_point: Weight) {
        println!("Setting target weight to {}", Into::<f64>::into(set_point));
    }

    pub fn set_target_temperature(set_point: Temp) {
        println!(
            "Setting target temperature to {}",
            Into::<f64>::into(set_point)
        );
    }

    pub fn set_target_pressure(set_point: Pressure) {
        println!(
            "Setting target pressure to {}",
            Into::<f64>::into(set_point)
        );
    }

    pub fn set_limited_pressure(set_point: Pressure) {
        println!(
            "Setting target pressure limit to {}",
            Into::<f64>::into(set_point)
        );
    }

    pub fn set_target_flow(set_point: Flow) {
        println!("Setting target flow to {}", Into::<f64>::into(set_point));
    }

    pub fn set_limited_flow(set_point: Flow) {
        println!("Setting flow limit to {}", Into::<f64>::into(set_point));
    }

    pub fn set_target_power(set_point: f64) {
        println!("Setting target power to {}", set_point);
    }

    pub fn set_target_piston_position(set_point: f64) {
        println!("Setting target piston position to {}", set_point);
    }
}
