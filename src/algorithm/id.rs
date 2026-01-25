use uuid::Uuid;

#[derive(Clone, Debug)]
pub enum AlgorithmId {
    Invalid,
    Id(Uuid),
}

impl PartialEq<Self> for AlgorithmId {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (AlgorithmId::Id(a), AlgorithmId::Id(b)) => a == b,
            _ => false
        }
    }
}

impl From<Uuid> for AlgorithmId {
    fn from(value: Uuid) -> Self {
        AlgorithmId::Id(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn invalid_ids_are_not_equal() {
        let id_1 = AlgorithmId::Invalid;
        let id_2 = AlgorithmId::Invalid;

        assert_ne!(id_1, id_2);
    }

    #[test]
    fn valid_id_not_equal_invalid_id() {
        let id_1 = AlgorithmId::Id(Uuid::from_bytes([0x00; 16].into()));
        let id_2 = AlgorithmId::Invalid;

        assert_ne!(id_1, id_2);
    }

    #[test]
    fn valid_id_all_zeroes_not_equal_valid_id_all_ones() {
        let id_1 = AlgorithmId::Id(Uuid::from_bytes([0x00; 16].into()));
        let id_2 = AlgorithmId::Id(Uuid::from_bytes([0xff; 16].into()));

        assert_ne!(id_1, id_2);
    }

    #[test]
    fn valid_ids_same_bytes_equal() {
        let bytes = [0x13; 16];
        let id_1 = AlgorithmId::Id(Uuid::from_bytes(bytes.clone().into()));
        let id_2 = AlgorithmId::Id(Uuid::from_bytes(bytes.clone().into()));

        assert_eq!(id_1, id_2);
    }
}