use crate::script::ScriptId;

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

    pub fn stats(&'_ self) -> ServerStats<'_> {
        ServerStats::from(&self.stats)
    }

    pub fn stats_mut(&'_ mut self) -> ServerStatsMut<'_> {
        ServerStatsMut::from(&mut self.stats)
    }
}

pub struct ServerStats<'stats> {
    stats: &'stats Vec<ServerStatInstance>,
}

pub struct ServerStatsMut<'stats> {
    stats: &'stats mut Vec<ServerStatInstance>,
}

impl<'s> ServerStats<'s> {
    pub fn from(stats: &'s Vec<ServerStatInstance>) -> ServerStats<'s> {
        ServerStats { stats }
    }

    pub fn value_of(&self, stat_type: ServerStatType) -> i32 {
        self
            .stats
            .iter()
            .filter(|stat| stat.stat_type() == &stat_type)
            .map(|stat| stat.value())
            .sum()
    }
}

impl <'s> ServerStatsMut<'s> {
    pub fn from(stats: &'s mut Vec<ServerStatInstance>) -> ServerStatsMut<'s> {
        ServerStatsMut { stats }
    }

    pub fn apply(&mut self, server_stat_instance: ServerStatInstance) {
        self.stats.push(server_stat_instance);
    }
}

#[derive(Hash, Eq, PartialEq, Clone, Debug)]
pub enum ServerStatSource {
    /// The stat is innate to the server.
    Innate,

    /// The stat is being modified by a script.
    Script(ScriptId),
}

/// All stats that can exist on a server.
#[derive(Hash, Eq, PartialEq, Clone, Debug)]
pub enum ServerStatType {
    SiphonResist,
    ExfilResist,
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