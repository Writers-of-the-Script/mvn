use anyhow::{Result, anyhow};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[repr(i16)]
pub enum MavenTokenPermissions {
    Read = 0,
    Write = 1,
    ReadWrite = 2,
}

impl MavenTokenPermissions {
    pub fn value(&self) -> i16 {
        match self {
            Self::Read => 0,
            Self::Write => 1,
            Self::ReadWrite => 2,
        }
    }

    pub fn from_value(value: i16) -> Result<Self> {
        match value {
            0 => Ok(Self::Read),
            1 => Ok(Self::Write),
            2 => Ok(Self::ReadWrite),
            _ => Err(anyhow!("Unknown value: {value}")),
        }
    }
}

impl Into<i16> for MavenTokenPermissions {
    fn into(self) -> i16 {
        self.value()
    }
}

impl TryFrom<i16> for MavenTokenPermissions {
    type Error = anyhow::Error;

    fn try_from(value: i16) -> Result<Self, Self::Error> {
        Self::from_value(value)
    }
}
