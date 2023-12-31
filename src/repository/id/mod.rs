//! Repository identifier parsing, handling and validation primitives.

use std::{
    path::{self, Path, PathBuf},
    str::FromStr,
};

use super::SOURCE_REPOSITORY_NAME;

mod error;
pub use error::Error;

mod name;
pub use name::{Name, REPOSITORY_NAME_EXT};

mod base;
pub use base::Base;

/// A repository [`Id`] is defined as a path without a leading `/`
/// that does not contain any other component than [`path::Component::Normal`]
/// that are parsed as a [`Base`] and a [`Name`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Id {
    namespace: Option<Base>,
    repository: Name,
}

impl Id {
    /// Get the [`Id`] of the origin source's repository.
    pub fn origin() -> Self {
        Self {
            namespace: None,
            repository: SOURCE_REPOSITORY_NAME,
        }
    }

    pub fn namespace(&self) -> Option<&str> {
        self.namespace.as_deref()
    }

    pub fn repository(&self) -> &Name {
        &self.repository
    }

    pub fn as_type(&self) -> Type<'_> {
        match &self.namespace {
            None if self.is_source() => Type::OriginSource(self),
            Some(_) if self.is_source() => Type::NamespaceSource(self),
            _ => Type::Plain(self),
        }
    }

    pub fn is_source(&self) -> bool {
        self.repository == SOURCE_REPOSITORY_NAME
    }

    /// Constructs the source repository [`Id`]
    /// from the current `namespace`:`repository` couple.
    pub fn to_source(&self) -> Self {
        Self {
            namespace: self.namespace.clone(),
            repository: SOURCE_REPOSITORY_NAME,
        }
    }

    /// Converts the current [`Id`] to a [`PathBuf`], in the `storage` path.
    pub fn to_path(&self, storage: &Path) -> PathBuf {
        storage.join(self.to_string())
    }
}

impl FromStr for Id {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // Encapsulate in a path and strip any leading `/`
        let path = Path::new(s);
        let path = path.strip_prefix("/").unwrap_or(path);

        let components: Vec<_> = path.components().collect();

        let (namespace, repository) = match components[..] {
            [path::Component::Normal(repository)] => (None, repository.to_str().unwrap().parse()?),
            [path::Component::Normal(namespace), path::Component::Normal(repository)] => (
                Some(namespace.to_str().unwrap().parse()?),
                repository.to_str().unwrap().parse()?,
            ),
            _ => Err(Error::MisformattedPath)?,
        };

        Ok(Self {
            namespace,
            repository,
        })
    }
}

/// The repository type regarding it's [`Id`].
pub enum Type<'i> {
    OriginSource(&'i Id),
    NamespaceSource(&'i Id),
    Plain(&'i Id),
}

impl std::fmt::Display for Id {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(namespace) = &self.namespace {
            f.write_str(namespace)?;
            f.write_str(std::path::MAIN_SEPARATOR_STR)?;
        }
        write!(f, "{}", &self.repository)
    }
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::*;

    #[rstest]
    #[case("/user/repo.git",
        Id { namespace: Some(Base("user".into())), repository: Name(Base("repo".into())) })]
    #[case("user/repo.git",
        Id { namespace: Some(Base("user".into())), repository: Name(Base("repo".into())) })]
    #[case("//user/repo.git",
        Id { namespace: Some(Base("user".into())), repository: Name(Base("repo".into())) })]
    #[case("?.git",
            Id { namespace: None, repository: Name(Base("?".into())) })]
    fn it_allows_valid_repositories(#[case] path: &str, #[case] expected: Id) {
        let path = Id::from_str(path).expect(path);

        assert_eq!(path, expected);
    }

    #[rstest]
    #[case("")]
    #[case("/")]
    #[case("..")]
    #[case(".git")]
    #[case("/.git")]
    #[case("~/user/repo.git")]
    #[case("./repo.git")]
    #[case("user/../repo.git")]
    #[case("/user/repo")]
    #[case("/repo")]
    #[case("..git")]
    #[case("toto/..git")]
    #[case(".toto.git")]
    #[case("toto..git")]
    fn it_denies_sketchy_repositories(#[case] path: &str) {
        let _ = Id::from_str(path).unwrap_err();
    }
}
