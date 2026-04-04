// src/launch/models.rs
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LaunchContext {
    pub version_id: String,      // such as "1.20.1-fabric"
    pub java_path: PathBuf,      // Path to the verified Java executable file
    pub main_class: String, // The final main class (provided by the original version or the Loader)
    pub classpath: Vec<PathBuf>, // Complete list of classpaths (original library + Fabric library + core JAR)
    pub game_args: Vec<String>,  // Game launch parameters
    pub jvm_args: Vec<String>,   // JVM Memory and Optimization Parameters
    pub work_dir: PathBuf,       // The game's installation directory (instance root directory)
}
