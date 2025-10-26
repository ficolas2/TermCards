use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Card {
    #[serde(default)]
    pub id: i64,
    pub volume_mounts: Vec<(String, String)>,
    pub expected_output: String,
    pub expected_input: String,
    pub command: Option<String>,
    pub docker_image: String,
    pub work_dir: Option<String>,
}
