use crate::profile::*;
use json;
use json::object::Object;
use json::Array;

use std::str::FromStr;

const MAX_STAGES: usize = 128;

pub struct ProfileGenerator {
    //json: String,
    profile: Profile,
    size: usize,
}

impl ProfileGenerator {
    pub fn try_new(json: &str) -> Result<Self, json::Error> {
        // TODO: Return new
        //JsonDocument doc;
        let doc = json::parse(json)?;
        /*
            profile.startTime = 0;
            profile.stages_len = 0;
            profile.temperature = writeProfileTemperature(doc["temperature"].as<double>());
            profile.finalWeight = writeProfileWeight(doc["final_weight"].as<double>());
            profile.wait_after_heating = doc["wait_after_heating"].as<bool>();
            profile.auto_purge = doc["auto_purge"].as<bool>();
        */

        let temp = doc["temperature"]
            .as_f64()
            .map(Into::into)
            .ok_or(type_err("float"))?;
        let final_weight = doc["final_weight"]
            .as_f64()
            .map(Into::into)
            .ok_or(type_err("float"))?;
        let wait = doc["wait_after_heating"]
            .as_bool()
            .ok_or(type_err("bool"))?;
        let purge = doc["aute_purge"].as_bool().ok_or(type_err("bool"))?;

        let json::JsonValue::Array(json_stages) = doc["stages"] else {
            return Err(type_err("array"));
        };
        let num_stages = std::cmp::min(json_stages.len(), MAX_STAGES);
        println!("Profile sategs len: {num_stages}");

        //let mut stages = Vec::with_capacity(num_stages);
        let mut stages = json_stages
            .into_iter()
            .map(Stage::try_from)
            .collect::<Vec<_>>()?;
        let mut stages_log = Vec::with_capacity(num_stages);

        /*
            JsonArray json_stages = doc["stages"].as<JsonArray>();
            auto num_stages = std::min(json_stages.size(), static_cast<size_t>(MAX_STAGES));
            printf("Profile stages len= %d\n", profile.stages_len);

            Stage *stages = static_cast<Stage *>(calloc(sizeof(Stage), num_stages));
            if (stages == nullptr)
                throw new std::length_error("cannot allocate enough memory for all stages");

            profile.stages = stages;
            profile.stages_len = num_stages;
            this->memoryUsed += sizeof(Stage) * profile.stages_len;
            for (int i = 0; i < profile.stages_len; ++i)
            {
                JsonObject stageJson = json_stages[i].as<JsonObject>();
                Stage &stage = profile.stages[i];
                this->memoryUsed +=
                    parseStage(stageJson, stage, i == (profile.stages_len - 1) ? i : i + 1);
            }
            StageLog *logs = static_cast<StageLog *>(calloc(sizeof(StageLog), num_stages));
            if (logs == nullptr)
                throw new std::length_error("cannot allocate enough memory for all stage logs");
            //this->memoryUsed += sizeof(StageLog) * profile.stages_len;

            profile.stage_log = logs;
        */
        let mut profile = Profile::new(0, 0, temp, final_weight, wait, purge, stages, stages_log);
        Ok(ProfileGenerator {
            profile,
            size: num_stages,
        })
    }
}

#[derive(Debug)]
struct InvalidJson {}

//impl std::error::Error for InvalidJson {}
/*
fn parseControlType(type: &str) -> ControlType
{
    if (type == "pressure"){
        return ControlType::CONTROL_PRESSURE;
    }if (type == "flow"){
        return ControlType::CONTROL_FLOW;
    }if (type == "power"){
        return ControlType::CONTROL_POWER;
    }if (type == "piston_position"){
        return ControlType::CONTROL_PISTON_POSITION;
    }
    //throw new InvalidJson(type);
}

fn parseInputType(type: &str) -> InputType
{
    if (type == "time"){
        return InputType::INPUT_TIME;
}if (type == "piston_position"){
        return InputType::INPUT_PISTON_POSITION;
    }if (type == "weight"){
        return InputType::INPUT_WEIGHT;
    }
    //throw new InvalidJson(type);
}


fn parseInterpolationType(type: &str) -> InterpolationType
{
    if (type == "linear"){
        return InterpolationType::INTERPOLATION_LINEAR;
}if (type == "catmull"){
        return InterpolationType::INTERPOLATION_CATMULL;
    }if (type == "bezier"){
        return InterpolationType::INTERPOLATION_BEZIER;
    }
    //throw new InvalidJson(type);
}

fn parseExitType(type: &str)-> ExitType
{
    if (type == "pressure"){
        return ExitType::EXIT_PRESSURE;
}if (type == "flow"){
        return ExitType::EXIT_FLOW;
    }if (type == "time"){
        return ExitType::EXIT_TIME;
    }if (type == "weight"){
        return ExitType::EXIT_WEIGHT;
    }if (type == "piston_position"){
        return ExitType::EXIT_PISTON_POSITION;
    }if (type == "power"){
        return ExitType::EXIT_POWER;
    }if (type == "temperature"){
        return ExitType::EXIT_TEMPERATURE;
    }if (type == "button"){
        return ExitType::EXIT_BUTTON;
    }
    //throw new InvalidJson(type);
}

fn parseExitComparison(comparison: &str)->ExitComparison
{
    if (comparison == "smaller") {
        return ExitComparison::EXIT_COMP_SMALLER;
}
    if (comparison == "greater"){
        return ExitComparison::EXIT_COMP_GREATER;
}
    //throw new InvalidJson(comparison);
}

fn parseExitReferenceType(is_relative: bool)->ExitReferenceType
{
    if is_relative {ExitReferenceType::EXIT_REF_SELF} else { ExitReferenceType::EXIT_REF_ABSOLUTE}
}

fn parseStage(stageJson: &Object &stageJson, stage: &Stage, default_stage_exit: u16) -> usize
{
    let bytes_allocated = 0;

    stage.dynamics.controlSelect = parseControlType(stageJson["type"].as::<String>());

    // Allocate memory for points and parse them
    if (stageJson.containsKey("exit_triggers"))
    {

        let jsonPoints: Array = stageJson["dynamics"]["points"].as::<Array>();
        let num_points = std::cmp::min(jsonPoints.size(), static_cast<size_t>(100));
        Point *points = static_cast<Point *>(calloc(sizeof(Point), num_points));
        if (points == nullptr) {
            //throw new std::length_error("cannot allocate enough memory for all stages");
}

        bytes_allocated += sizeof::<Point>() * num_points;
        stage.dynamics.points = points;
        stage.dynamics.points_len = num_points;
        for (size_t i = 0; i < stage.dynamics.points_len; ++i)
        {
            bool is_percent = stage.dynamics.controlSelect == ControlType::CONTROL_POWER ||
                              stage.dynamics.controlSelect == ControlType::CONTROL_PISTON_POSITION;
            stage.dynamics.points[i].x =
                static_cast<int16_t>(jsonPoints[i][0].as<float>() * 10.0f);
            stage.dynamics.points[i].y.val =
                static_cast<int16_t>(
                    jsonPoints[i][1].as<float>() * (is_percent ? 1 : 10));
        }
    }

    stage.dynamics.interpolation = parseInterpolationType(stageJson["dynamics"]["interpolation"].as<std::string>());
    stage.dynamics.inputSelect = parseInputType(stageJson["dynamics"]["over"].as<std::string>());

    // Allocate memory for the exit triggers
    if (stageJson.containsKey("exit_triggers"))
    {
        Array jsonExitTriggers = stageJson["exit_triggers"].as<Array>();

        auto num_exit_triggers = std::min(jsonExitTriggers.size(), static_cast<size_t>(100));
        ExitTrigger *exitTriggers = static_cast<ExitTrigger *>(calloc(sizeof(ExitTrigger), num_exit_triggers));
        if (exitTriggers == nullptr)
            throw new std::length_error("cannot allocate enough memory for all stages");

        bytes_allocated += sizeof(ExitTrigger) * num_exit_triggers;

        stage.exitTrigger = exitTriggers;
        stage.exitTrigger_len = num_exit_triggers;
        for (size_t i = 0; i < stage.exitTrigger_len; ++i)
        {
            Object exitTriggerJson = jsonExitTriggers[i].as<Object>();
            stage.exitTrigger[i].type = parseExitType(exitTriggerJson["type"].as<std::string>());
            stage.exitTrigger[i].value = writeExitValue(exitTriggerJson["value"].as<double>());
            stage.exitTrigger[i].comparison = parseExitComparison(exitTriggerJson["comparison"] | "greater");
            stage.exitTrigger[i].reference = parseExitReferenceType(exitTriggerJson["relative"] | true);
            stage.exitTrigger[i].target_stage = exitTriggerJson["target_stage"] | (default_stage_exit);
        }
    }

    if (stageJson.containsKey("limits"))
    {
        Array limits = stageJson["limits"].as<Array>();
        for (JsonObject limit : limits)
        {
            auto limit_type = limit["type"].as<std::string>();
            if (limit_type == "pressure")
            {
                stage.dynamics.limits.pressure = writeProfilePressure(limit["value"].as<double>());
                continue;
            }
            else if (limit_type == "flow")
            {
                stage.dynamics.limits.pressure = writeProfileFlow(limit["value"].as<double>());
                continue;
            }
            else
            {
                throw new InvalidJson(limit_type);
            }
        }
    }
    return bytes_allocated;
}
*/
