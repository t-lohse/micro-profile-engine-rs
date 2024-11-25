macro_rules! json_val_to_obj_tryfrom {
    ($x:expr) => {
        match $x {
            json::JsonValue::Object(o) => Self::try_from(o),
            _ => Err(ProfileError::Type("Expected object, got other".to_string())),
        }
    };
}

mod profile;
//mod sampler;
mod engine;
mod sensor;

use crate::engine::*;
use crate::profile::{FromJson, Profile};
use crate::sensor::{Driver, DummySensorState};

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
    let doc = json::parse(PROFILE_JSON).unwrap();
    let mut profile = Profile::parse_value(&doc).unwrap();
    println!("{profile:?}");

    let driver = Driver::<DummySensorState>::default();

    let sensor_data = driver.sensor_data() as *const DummySensorState as *mut DummySensorState;
    //let mut curr_piston_pos
    let engine_idle = ProfileEngineIdle::try_new(&mut profile, driver).unwrap();

    println!("Starting engine");
    let mut engine = engine_idle.start();
    println!("The engine is in state: {:?}", engine.get_state());
    loop {
        engine = match engine.step() {
            EngineStepResult::Next(e) => {
                if e.get_state() == ProfileState::Done {
                    break;
                } else {
                    e
                }
            }
            EngineStepResult::Finished(_) => break,
            EngineStepResult::Error(e) => {
                println!("No Stages in profile!!! Error: `{e}`");
                return;
            }
        };
        std::thread::sleep(std::time::Duration::from_millis(50));
        // We fake the piston moving 1% each step to show the piston position sampling capabilities
        println!("The engine is in state: {:?}", engine.get_state());
        if engine.get_state() == ProfileState::Brewing {
            unsafe {
                let cur_pos = *(*sensor_data).piston_position;
                *(*sensor_data).piston_position = (cur_pos + 1.0).min(100.0);
                println!("Piston: {}", (*sensor_data).piston_position)
            }
        }
    }
    println!("Profile execution finished.");
    //println!("Profile allocated 0x{.2} bytes({} kB) of ram for all {} stages combined",
    //    generator.memoryUsed, generator.memoryUsed / 1024, max_profile.stages_len);
}
