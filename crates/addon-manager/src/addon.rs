use std::{borrow::Cow, ops::{BitAnd, BitAndAssign}, str::FromStr};

use serde::{Deserialize, Serialize};
use url::Url;

use crate::Error;

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum Target {
    #[default]
    LuaCats,
    Github,
}

impl FromStr for Target {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Some(url) = s.strip_prefix("git:") {
            let url = Url::parse(url)?;
            match url.host_str() {
                Some("github.com") => Ok(Target::Github),
                Some(other) => Err(Error::UnsupportedSource(other.to_string())),
                _ => Err(Error::UnsupportedSource(s.to_string())),
            }
        } else {
            Ok(Target::LuaCats)
        }
    }
}

#[derive(Default, Debug, Clone, Deserialize, Serialize, PartialEq, Eq, PartialOrd, Ord)]
pub struct Addon {
    pub src: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub checksum: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub branch: Option<String>,
    pub target: Target,
}

impl Addon {
    pub fn cats(name: String, checksum: Option<String>, branch: Option<String>) -> Self {
        Self {
            src: name,
            checksum,
            branch,
            target: Target::LuaCats,
        }
    }

    pub fn name(&self) -> Cow<'static, str> {
        match self.target {
            Target::LuaCats => self.src.clone().into(),
            Target::Github => {
                let url = Url::parse(self.src.as_str()).unwrap();
                url.path_segments()
                    .unwrap()
                    .nth(1)
                    .unwrap()
                    .to_string()
                    .into()
            }
        }
    }

    pub fn clone_url(&self) -> String {
        match self.target {
            Target::LuaCats => format!("https://github.com/LuaCATS/{}.git", self.src),
            Target::Github => self.src.to_string(),
        }
    }
}

impl BitAnd for Addon {
    type Output = Self;
    fn bitand(self, rhs: Self) -> Self::Output {
        Self {
            src: rhs.src,
            target: rhs.target,
            branch: rhs.branch.or(self.branch),
            checksum: rhs.checksum.or(self.checksum),
        }
    }
}

impl BitAndAssign for Addon {
    fn bitand_assign(&mut self, rhs: Self) {
        self.src = rhs.src.clone();
        self.target = rhs.target;

        if let Some(branch) = rhs.branch.as_ref() {
            self.branch = Some(branch.to_string());
        }

        if let Some(checksum) = rhs.checksum.as_ref() {
            self.checksum = Some(checksum.to_string());
        }
    }
}

impl BitAndAssign<&Self> for Addon {
    fn bitand_assign(&mut self, rhs: &Self) {
        self.src = rhs.src.clone();
        self.target = rhs.target;

        if let Some(branch) = rhs.branch.as_ref() {
            self.branch = Some(branch.clone());
        }

        if let Some(checksum) = rhs.checksum.as_ref() {
            self.checksum = Some(checksum.clone());
        }
    }
}

impl<S: AsRef<str>> From<S> for Addon {
    fn from(s: S) -> Self {
        let mut source = s.as_ref();
        let mut checksum = None;

        if source.contains('@') {
            let (f, s) = source.split_once('@').unwrap();
            source = f;
            checksum = Some(s.to_string());
        }

        Self {
            target: Target::from_str(source).unwrap(),
            src: source.to_string(),
            checksum,
            branch: None,
        }
    }
}

impl std::fmt::Display for Addon {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.src)?;
        if let Some(checksum) = self.checksum.as_deref() {
            write!(f, "@{checksum}")?;
        }
        Ok(())
    }
}
