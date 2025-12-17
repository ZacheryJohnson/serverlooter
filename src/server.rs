pub struct Server {
    pub name: String,

    /// Number of concurrent processes available for all running scripts
    pub threads: u32,

    /// Defined as an u32 to avoid float imprecision
    pub clock_speed_hz: u64,

    pub stats: Vec<ServerStatInstance>,
}

pub enum ServerStatSource {

}

#[derive(Hash, Eq, PartialEq, Clone, Debug)]
pub enum ServerStatType {
    ReconResistance,
    VulnerabilityResistance,
    ExtractionResistance,
}

pub struct ServerStatInstance(ServerStatSource, ServerStatType, u32);
impl ServerStatInstance {
    pub fn new(source: ServerStatSource, stat_type: ServerStatType, value: u32) -> ServerStatInstance {
        ServerStatInstance(source, stat_type, value)
    }

    pub fn source(&self) -> &ServerStatSource {
        &self.0
    }

    pub fn stat_type(&self) -> &ServerStatType {
        &self.1
    }

    pub fn value(&self) -> u32 {
        self.2
    }
}