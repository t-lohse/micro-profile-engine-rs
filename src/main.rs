//#![allow(non_snake_case)]
//include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
//
//use arduino_json_rs::JsonString;
/*
#[cxx::bridge]
mod ffi {

    extern "Rust" {}

    unsafe extern "C++" {
        include!("cpp/ArduinoJson-v7.0.3.h");

        type JsonDocument;
        type JsonSHittyFailure;
        type JsonArray;
        fn put(self: &JsonArray);
        fn isNull(self: &JsonArray) -> bool;

    }
}
*/

//use crate::ProfileGenerator;

mod exit_trigger;
mod profile;
mod sampler;
mod sensor;
mod simplified_profile_engine;
use sensor::SensorState;

use crate::exit_trigger::*;
use crate::profile::ProfileGenerator;
use crate::sensor::Driver;
use crate::simplified_profile_engine::{ProfileState, SimplifiedProfileEngine};

static PROFILE_JSON: &str = r#"{
    "name": "E61 with dropping pressure",
    "id": "4cdc0015-07cd-4738-b198-c7d8742acd2b",
    "author": "Carlos",
    "author_id": "d9123a0a-d3d7-40fd-a548-b81376e43f23",
    "previous_authors": [
        {
            "name": "mimoja",
            "author_id": "d9123a0a-d3d7-40fd-a548-b81376e43f23",
            "profile_id": "0cdf18ca-d72e-4776-8e25-7b3279907dce"
        },
        {
            "name": "Alu",
            "author_id": "ee86a777-fdd6-46d6-8cf7-099a9fb609a1",
            "profile_id": "58036fd5-7d5b-4647-9ab6-2832014bb9be"
        }
    ],
    "temperature": 92.5,
    "final_weight": 80.0,
    "stages": [
        {
            "name": "stage 1",
            "type": "power",
            "dynamics": {
                "points": [
                    [0, 100],
                    [10, 50],
                    [20, 40]
                ],
                "over": "piston_position",
                "interpolation": "linear"
            },
            "exit_triggers": [
                {
                    "type": "time",
                    "value": 3
                },
                {
                    "type": "pressure",
                    "value": 4
                }
            ]
        },
        {
            "name": "stage 2",
            "type": "flow",
            "dynamics": {
                "points": [
                    [0, 8.5],
                    [30, 6.5]
                ],
                "over": "time",
                "interpolation": "linear"
            },
            "exit_triggers": [
                {
                    "type": "time",
                    "value": 2,
                    "relative": true
                }
            ],
            "limits": [
                {
                    "type": "flow",
                    "value": 3
                }
            ]
        }
    ]
}"#;

fn main() {
    // Profile maxProfile;
    // maxProfile.stages_len = 2;
    // maxProfile.temperature = writeProfileTemperature(88.4);
    // Stage* stage = &maxProfile.stages[0];
    // stage->dynamics.controlSelect = ControlType::CONTROL_PRESSURE;
    // stage->dynamics.inputSelect = InputType::INPUT_TIME;
    // stage->dynamics.interpolation = InterpolationType::INTERPOLATION_LINEAR;
    // stage->dynamics.points_len = 3;
    // Point *p1 = &stage->dynamics.points[0];
    // Point *p2 = &stage->dynamics.points[1];
    // Point *p3 = &stage->dynamics.points[2];

    // p1->x = 0.1 * 10;
    // p1->y.pressure = writeProfilePressure(4.0);
    // p2->x = 0.5 * 10;
    // p2->y.pressure = writeProfilePressure(1.0);
    // p3->x = 4.1 * 10;
    // p3->y.pressure = writeProfilePressure(7.0);

    // stage->exitTrigger_len = 1;
    // ExitTrigger *trigger = &stage->exitTrigger[0];
    // trigger->comparison = ExitComparison::EXIT_COMP_GREATER;
    // trigger->type = ExitType::EXIT_TIME;
    // trigger->target_stage = 1;
    // trigger->value = writeExitValue(5.0);

    // // Create stage 2
    // stage = &maxProfile.stages[1];
    // stage->dynamics.controlSelect = ControlType::CONTROL_PRESSURE;
    // stage->dynamics.inputSelect = InputType::INPUT_TIME;
    // stage->dynamics.interpolation = InterpolationType::INTERPOLATION_LINEAR;
    // stage->dynamics.points_len = 1;
    // p1 = &stage->dynamics.points[0];
    // p1->x = 0.5 * 10;
    // p1->y.pressure = writeProfilePressure(8.0);
    // stage->exitTrigger_len = 1;
    // trigger = &stage->exitTrigger[0];
    // trigger->comparison = ExitComparison::EXIT_COMP_GREATER;
    // trigger->type = ExitType::EXIT_TIME;
    // trigger->target_stage = 1;
    // trigger->value = writeExitValue(2.0);

    println!("{}", std::mem::size_of::<exit_trigger::ExitTrigger>());
    println!("{}", std::mem::size_of::<exit_trigger::ExitType>());
    println!("{}", std::mem::size_of::<exit_trigger::ExitComparison>());
    println!("{}", std::mem::size_of::<exit_trigger::ExitReferenceType>());
    let trigger = ExitTrigger::new(
        ExitType::ExitPressure,
        ExitComparison::ExitCompGreater,
        ExitReferenceType::ExitRefSelf,
        10,
        u32::MAX >> 6,
    );

    println!("{:?}", trigger);
    println!("{:?}", trigger.exit_type());
    println!("{:?}", trigger.exit_comp());
    println!("{:?}", trigger.exit_ref());
    println!("{}", trigger.value());

    //println!("{}", ExitTrigger::TYPE_OFFSET);
    //println!("{}", ExitTrigger::COMP_OFFSET);
    //println!("{}", ExitTrigger::REF_OFFSET);

    // TODO: Missing files (in order):
    // - ExitTrigger.[h|cpp] - DONE
    // - Sensor.h - DONE
    // - Sampler.[h|cpp] - DONE
    // - SimplifiedProfileEngine.[h|cpp] - DONE
    // - main.cpp

    let mut generator = ProfileGenerator::try_new(PROFILE_JSON).unwrap();
    //let mut max_profile = generator.profile();

    let mut driver = Driver::default();
    let sensor_data: *mut SensorState = driver.sensor_data_mut() as *mut SensorState;

    let mut engine = SimplifiedProfileEngine::try_new(generator.profile_mut(), &driver).unwrap();
    println!(
        "After creating the engine is in state: {:?}",
        engine.get_state()
    );

    //engine.step().unwrap();
    //println!(
    //    "After one step without starting the engine is in state: {:?}\n",
    //    engine.get_state()
    //);
    println!("Starting engine");
    engine.start();
    let mut state = engine.get_state();
    println!("The engine is in state: {:?}", engine.get_state());
    let mut i = 0;
    while (engine.get_state() != ProfileState::Done) {
        engine.step().unwrap_or_else(|e| {
            println!("No Stages in profile!!!");
            return;
        });
        std::thread::sleep(std::time::Duration::from_millis(50));
        // We fake the piston moving 1% each step to show the piston position samping capabilities
        println!("The engine is in state: {:?}", engine.get_state());
        if (engine.get_state() == ProfileState::Brewing) {
            unsafe {
                (*sensor_data).piston_position =
                    (driver.sensor_data().piston_position + 1.0).min(100.0);
                println!("Piston: {}", (*sensor_data).piston_position)
            }
        }
        println!("{i}");
        i += 1;
    }
    println!("Profile execution finished.");
    //println!("Profile allocated 0x{.2} bytes({} kB) of ram for all {} stages combined",
    //    generator.memoryUsed, generator.memoryUsed / 1024, max_profile.stages_len);
}
