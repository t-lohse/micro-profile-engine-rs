use crate::{ExitComparison, ExitReferenceType, ExitTrigger, ExitType};
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
    pub fn sensor_data_mut(&mut self) -> &mut SensorState {
        &mut self.sensors
    }

    fn get_exit_input(
        &self,
        trigger: &ExitTrigger,
        stage_timestamp: f64,
        profile_timestamp: f64,
    ) -> f64 {
        match trigger.exit_type() {
            ExitType::ExitPressure => self.sensors.water_pressure,
            ExitType::ExitFlow => self.sensors.water_flow,
            ExitType::ExitTime => {
                (if trigger.exit_ref().is_absolute() {
                    profile_timestamp as f64
                } else {
                    stage_timestamp as f64
                }) / 1_000.0
            }

            ExitType::ExitWeight => self.sensors.weight,
            ExitType::ExitPistonPosition => self.sensors.piston_position,
            ExitType::ExitTemperature => self.sensors.stable_temperature,
            ExitType::ExitPower => unimplemented!("Exit button not done yet"),
            ExitType::ExitButton => self
                .get_button_gesture("Encoder Button", "Single Tap")
                .into(),
        }
    }

    pub fn check_exit_cond(
        &self,
        trigger: &ExitTrigger,
        stage_timestamp: f64,
        profile_timestamp: f64,
    ) -> bool {
        let current_value = self.get_exit_input(trigger, stage_timestamp, profile_timestamp);
        let exit_value = trigger.value().into(); // as f64;

        match trigger.exit_comp() {
            ExitComparison::ExitCompSmaller => current_value <= exit_value,
            ExitComparison::ExitCompGreater => current_value >= exit_value,
        }
    }

    fn get_button_gesture(&self, source: &str, gesture: &str) -> bool {
        false
    }
}
