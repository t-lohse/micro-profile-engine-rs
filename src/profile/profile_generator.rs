use crate::profile::{Profile, ProfileError, Stage};
use json;

const MAX_STAGES: usize = 128;

pub struct ProfileGenerator {
    //json: String,
    profile: Profile,
    //size: usize,
}

impl ProfileGenerator {
    pub fn try_new(json: &str) -> Result<Self, ProfileError> {
        let doc = json::parse(json)?;

        let temp = doc["temperature"]
            .as_f64()
            .map(Into::into)
            .ok_or(ProfileError::unexpected_type("float"))?;
        let final_weight = doc["final_weight"]
            .as_f64()
            .map(Into::into)
            .ok_or(ProfileError::unexpected_type("float"))?;
        let wait = doc["wait_after_heating"].as_bool().unwrap_or_default();
        //.ok_or(ProfileError::unexpected_type("bool"))?;
        let purge = doc["aute_purge"].as_bool().unwrap_or_default();

        let json::JsonValue::Array(ref json_stages) = doc["stages"] else {
            return Err(ProfileError::unexpected_type("array"));
        };
        let num_stages = std::cmp::min(json_stages.len(), MAX_STAGES);
        println!("Profile staegs len: {num_stages}");

        //let mut stages = Vec::with_capacity(num_stages);
        let mut stages = json_stages
            .into_iter()
            .map(Stage::try_from)
            .collect::<Result<Vec<_>, _>>()?;
        let mut stages_log = Vec::with_capacity(num_stages);

        let mut profile = Profile::new(0, 0, temp, final_weight, wait, purge, stages, stages_log);
        Ok(ProfileGenerator {
            profile,
            //size: num_stages,
        })
    }

    pub fn profile(&self) -> &Profile {
        &self.profile
    }
    pub fn profile_mut(&mut self) -> &mut Profile {
        &mut self.profile
    }
}
