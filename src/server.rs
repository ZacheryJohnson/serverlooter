use std::collections::btree_map::Entry;
use std::collections::BTreeMap;
use crate::script::ScriptId;

#[derive(Clone)]
pub struct Server {
    pub name: String,

    /// Number of concurrent processes available for all running scripts
    pub threads: u32,

    /// Defined as an u64 to avoid float imprecision
    pub clock_speed_hz: u64,

    pub stats: ServerStatInstances,
}

impl Server {
    pub fn empty() -> Server {
        Server {
            name: "".to_string(),
            threads: 0,
            clock_speed_hz: 0,
            stats: ServerStatInstances::new(),
        }
    }
}

pub trait ServerStats {
    /// Returns the current value of a stat, considering all buffs/debuffs currently applied.
    /// For example, a stat with an innate value of 10, a buff of +3 and a debuff of -1 would
    /// return a value of +12.
    fn value_of(&self, stat_type: ServerStatType) -> i32;

    /// Returns the current modification of a stat, considering all buffs/debuffs currently applied.
    /// For example, a stat with an innate value of 10, a buff of +3 and a debuff of -1 would
    /// return a modification of +2.
    fn modification_of(&self, stat_type: ServerStatType) -> i32;

    /// Applies a server stat instance to this server, purging modifications that would sum to zero.
    /// For example, say the server only has a single +1 exfil resist stat applied from a script.
    /// ```
    /// let server_stat_instances = [
    ///     ServerStatInstance(Script(_), ExfilResist, 1),
    /// ];
    /// ```
    /// Then, `apply_and_purge` is called with `ServerStatInstance(Script(_), ExfilResist, -1)`.
    /// ```
    ///
    /// let server_stat_instances = [
    ///     ServerStatInstance(Script(_), ExfilResist, 1),
    ///     ServerStatInstance(Script(_), ExfilResist, -1),
    /// ];
    /// ```
    /// `apply_and_purge` will remove both modifications such that `server_stat_instances` is empty.
    /// The removed modifications are returned by the function.
    fn apply_and_purge(&mut self, stat_instance: ServerStatInstance) -> Vec<ServerStatInstance>;

    /// Returns the value of each stat type owned by this server.
    fn stat_values(&self) -> BTreeMap<ServerStatType, i32>;

    /// Returns each individual stat instance on this server.
    fn stat_instances(&self) -> BTreeMap<ServerStatType, Vec<ServerStatInstance>>;
}

#[derive(Clone)]
pub struct ServerStatInstances {
    stats: Vec<ServerStatInstance>,
}

impl ServerStatInstances {
    pub fn new() -> ServerStatInstances {
        ServerStatInstances { stats: vec![] }
    }

    pub fn from(stats: &[ServerStatInstance]) -> ServerStatInstances {
        ServerStatInstances { stats: stats.to_vec() }
    }
}

impl ServerStats for ServerStatInstances {
    fn value_of(&self, stat_type: ServerStatType) -> i32 {
        self
            .stats
            .iter()
            .filter(|stat| stat.stat_type() == &stat_type)
            .map(|stat| stat.value())
            .sum()
    }

    fn modification_of(&self, stat_type: ServerStatType) -> i32 {
        self
            .stats
            .iter()
            .filter(|stat| stat.stat_type() == &stat_type)
            .filter(|stat| stat.source().purgable())
            .map(|stat| stat.value())
            .sum()
    }

    fn apply_and_purge(
        &mut self,
        stat_instance: ServerStatInstance
    ) -> Vec<ServerStatInstance> {
        // ZJ-TODO: deprecate purging
        self.stats.push(stat_instance);
        vec![]
    }

    fn stat_values(&self) -> BTreeMap<ServerStatType, i32> {
        let mut stat_map = BTreeMap::new();
        for stat in &self.stats {
            match stat_map.entry(stat.stat_type().to_owned()) {
                Entry::Vacant(new) => {
                    new.insert(stat.value());
                }
                Entry::Occupied(mut existing) => {
                    *existing.get_mut() += stat.value();
                }
            }
        }

        stat_map
    }

    fn stat_instances(&self) -> BTreeMap<ServerStatType, Vec<ServerStatInstance>> {
        let mut stat_map: BTreeMap<ServerStatType, Vec<ServerStatInstance>> = BTreeMap::new();
        for stat in &self.stats {
            match stat_map.entry(stat.stat_type().to_owned()) {
                Entry::Vacant(new) => {
                    new.insert(vec![stat.clone()]);
                }
                Entry::Occupied(mut existing) => {
                    existing.get_mut().push(stat.clone());
                }
            }
        }

        stat_map
    }
}

#[derive(Hash, Eq, PartialEq, Clone, Debug)]
pub enum ServerStatSource {
    /// The stat is innate to the server.
    Innate,

    /// The stat is being modified by a script.
    Script(ScriptId),
}

impl ServerStatSource {
    /// Determines if the stat source is purgable by the `Purge` algorithm effect.
    pub fn purgable(&self) -> bool {
        matches!(self, ServerStatSource::Script(_))
    }
}

/// All stats that can exist on a server.
#[derive(Hash, Eq, PartialEq, Clone, Debug, Ord, PartialOrd)]
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