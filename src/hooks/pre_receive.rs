use std::io;

use clap::Parser;
use color_eyre::eyre::{self, WrapErr};
use futures::{io::AllowStdIo, TryStreamExt};

use super::{Error, Params, RefUpdate};
use crate::repository::{
    authority::{Authority, Namespace, Origin},
    id::Type,
    Repository,
};

/// The first script to run when handling a push from a client is pre-receive.
/// It takes a list of references that are being pushed from stdin;
/// if it exits non-zero, none of them are accepted.
/// You can use this hook to do things like make sure none of the updated references are non-fast-forwards,
/// or to do access control for all the refs and files they’re modifying with the push.
///
/// see https://git-scm.com/book/en/v2/Customizing-Git-Git-Hooks#_pre_receive
#[derive(Debug, Parser)]
pub struct PreReceive {
    #[command(flatten)]
    params: Params,
}

impl PreReceive {
    pub async fn run(self) -> Result<(), Error<eyre::Error>> {
        RefUpdate::from_io(AllowStdIo::new(io::stdin()))
            .try_for_each(|refupdate| self.receive(refupdate))
            .await
    }

    async fn receive(
        &self,
        RefUpdate {
            oldrev: _,
            newrev,
            refname,
        }: RefUpdate,
    ) -> Result<(), Error<eyre::Error>> {
        let Params { storage, id } = &self.params;

        let repository = Repository::open_from_hook(storage, id)?;
        let is_head = refname
            == repository
                .find_reference("HEAD")?
                .symbolic_target()
                .expect("HEAD is not a symbolic reference");

        match id.as_type() {
            Type::OriginAuthority(_) => {
                match Origin::read_commit(&repository, &newrev).wrap_err("Authority parse failed") {
                    // If repository's head is updated, ensure authority integrity
                    Err(err) if is_head => Err(Error::Err(err)),
                    // If another branch is updated, simply spit out a warning
                    Err(err) => Err(Error::Warn(err)),

                    _ => Ok(()),
                }
            }
            Type::NamespaceAuthority(_) => {
                match Namespace::read_commit(&repository, &newrev)
                    .wrap_err("Authority parse failed")
                {
                    // If repository's head is updated, ensure authority integrity
                    Err(err) if is_head => Err(Error::Err(err)),
                    // If another branch is updated, simply spit out a warning
                    Err(err) => Err(Error::Warn(err)),

                    _ => Ok(()),
                }
            }
            Type::Plain(id) => {
                let authority = Repository::open(storage, &id.to_authority())?;

                let def = if id.namespace().is_none() {
                    Origin::read(&authority)?
                        .repository(id)
                        .expect("The repository is not defined in it's authority repository")
                        .clone()
                } else {
                    Namespace::read(&authority)?
                        .repository(id)
                        .expect("The repository is not defined in it's authority repository")
                        .clone()
                };

                Ok(())
            }
        }
    }
}