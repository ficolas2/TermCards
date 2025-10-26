use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Card {
    pub volume_mounts: Vec<(String, String)>,
    pub expected_output: String,
    pub expected_input: String,
    pub command: Option<String>,
    pub docker_image: String,
    pub work_dir: Option<String>,
}

