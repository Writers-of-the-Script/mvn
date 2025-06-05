use anyhow::{Result, anyhow};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[repr(i16)]
pub enum RouteAccess {
    Public = 0,
    Hidden = 1,
    Private = 2,
}

impl RouteAccess {
    pub fn value(&self) -> i16 {
        match self {
            Self::Public => 0,
            Self::Hidden => 1,
            Self::Private => 2,
        }
    }

    pub fn from_value(value: i16) -> Result<Self> {
        match value {
            0 => Ok(Self::Public),
            1 => Ok(Self::Hidden),
            2 => Ok(Self::Private),
            _ => Err(anyhow!("Unknown value: {value}")),
        }
    }
}

impl Into<i16> for RouteAccess {
    fn into(self) -> i16 {
        self.value()
    }
}

impl TryFrom<i16> for RouteAccess {
    type Error = anyhow::Error;

    fn try_from(value: i16) -> Result<Self, Self::Error> {
        Self::from_value(value)
    }
}
