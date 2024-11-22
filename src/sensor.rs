use crate::profile::{Flow, Pressure, Temp, Weight};

#[derive(Default, Clone, Debug)]
pub struct SensorState {
    pub piston_position: f64,
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

#[derive(Default, Clone, Debug)]
pub struct Driver {
    sensors: SensorState,
}

impl Driver {
    pub fn sensor_data(&self) -> &SensorState {
        &self.sensors
    }

    /*
    pub fn sensor_data_mut(&mut self) -> &mut SensorState {
        &mut self.sensors
    }
     */

    pub fn get_button_gesture(&self, source: &str, gesture: &str) -> bool {
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
