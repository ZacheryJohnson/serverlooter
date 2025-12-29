#[derive(Clone)]
pub struct Server {
    pub name: String,

    /// Number of concurrent processes available for all running scripts
    pub threads: u32,

    /// Defined as an u32 to avoid float imprecision
    pub clock_speed_hz: u64,

    pub stats: Vec<ServerStatInstance>,
}

impl Server {
    pub fn empty() -> Server {
        Server {
            name: "".to_string(),
            threads: 0,
            clock_speed_hz: 0,
            stats: vec![],
        }
    }
}

#[derive(Hash, Eq, PartialEq, Clone, Debug)]
pub enum ServerStatSource {

}

#[derive(Hash, Eq, PartialEq, Clone, Debug)]
pub enum ServerStatType {
    ReconResistance,
    VulnerabilityResistance,
    ExtractionResistance,
}

#[derive(Hash, Eq, PartialEq, Clone, Debug)]
pub struct ServerStatInstance(ServerStatSource, ServerStatType, i32);
impl ServerStatInstance {
    pub fn new(source: ServerStatSource, stat_type: ServerStatType, value: i32) -> ServerStatInstance {
        ServerStatInstance(source, stat_type, value)
    }

    pub fn source(&self) -> &ServerStatSource {
        &self.0
    }

    pub fn stat_type(&self) -> &ServerStatType {
        &self.1
    }

    pub fn value(&self) -> i32 {
        self.2
    }
}