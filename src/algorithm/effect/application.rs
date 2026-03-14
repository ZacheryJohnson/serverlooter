use std::sync::{Arc, Mutex};
use crate::algorithm::effect::AlgorithmEffect;
use crate::script::Script;
use crate::server::Server;

pub struct AlgorithmEffectApplication {
    pub host_server: Arc<Mutex<Server>>,
    pub target_server: Arc<Mutex<Server>>,
    pub effect: AlgorithmEffect,
    pub script: Arc<Mutex<Script>>,
}