use std::fmt;
use std::hash::Hash;

#[derive(Clone)]
pub struct HistoryKey {
    pub id: u32,
    pub set: String,
    pub plan: String,
}

impl fmt::Display for HistoryKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}({})", self.set, self.plan, self.id)
    }
}

impl PartialEq for HistoryKey {
    fn eq(&self, other: &Self) -> bool{
        if self.id == other.id {
            return true;
        }
        return false;
    }
}

impl PartialEq<u32> for HistoryKey {
    fn eq(&self, other: &u32) -> bool {
        if self.id == *other {
            return true;
        }
        return false;
    }
}

impl Eq for HistoryKey {}

impl Hash for HistoryKey {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}