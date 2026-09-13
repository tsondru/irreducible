//! SurrealDB persistence for cospan chains, via catgraph-surreal.
//!
//! [`EvolutionPersistence`] stores a chain of `Cospan<u32>` across two tiers:
//! each cospan goes to catgraph-surreal's content-addressed cospan tier, and
//! the ordered list of the addresses they landed at goes to the document tier
//! as a [`CospanChainRecord`]. Loading walks the record's addresses back
//! through the cospan tier, so a chain comes back in the order it was written.
//!
//! Both producers of a `Vec<Cospan<u32>>` in this crate's dependency surface
//! feed the same `&[Cospan<u32>]` entry point:
//! `HypergraphEvolution::to_cospan_chain` (wrapped as
//! [`EvolutionPersistence::persist_evolution`]) and
//! [`TemporalComplex::to_cospan_chain`](crate::TemporalComplex::to_cospan_chain),
//! which is passed to [`EvolutionPersistence::persist_cospan_chain`] directly.
//!
//! # A reloaded chain is equal up to `canonical_form`
//!
//! The cospan tier addresses a cospan by its *presentation* and keys it by its
//! morphism: the canonical key column is `UNIQUE`, so writing a second,
//! differently presented spelling of an already-stored morphism is refused with
//! `StoreError::Duplicate` rather than stored beside the first. This module
//! recovers from that refusal by asking `find_by_canon` for the address the
//! morphism is already at, which means a chain containing two presentations of
//! one morphism persists as two entries pointing at one row. Reloading such a
//! chain yields the *stored* presentation at both positions, so the round trip
//! is equality of morphisms — `Cospan::canonical_form` — not structural
//! equality of presentations. A chain whose entries are all distinct morphisms
//! reloads structurally identical.
//!
//! # Cospans only
//!
//! There is no span tier: catgraph-surreal stores cospans and does not yet
//! store spans (store issue sustia-llc/catgraph-surreal#9). Rewrite spans
//! reach this module only through the cospan chain an evolution derives from
//! them.

use catgraph::cospan::Cospan;
use catgraph_surreal::{CospanAddr, CospanStore, DocStore, Result, Store, StoreError};
use serde::{Deserialize, Serialize};

use super::HypergraphEvolution;

/// The document-tier `kind` every chain record is filed under.
const CHAIN_KIND: &str = "cospan_chain";

/// The document-tier record naming one stored chain.
///
/// `addrs` holds the cospan tier's addresses in chain order, in their stored
/// string form. They are strings rather than `CospanAddr` because the record
/// crosses serde, and `CospanAddr` is opaque: it is re-parsed on load, which
/// re-validates the format at the same time.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CospanChainRecord {
    /// The caller-supplied name the chain is stored under; also its record id.
    pub name: String,
    /// The chain's cospan addresses, in order.
    pub addrs: Vec<String>,
}

/// The two-tier handle a chain is stored and loaded through.
#[derive(Debug, Clone)]
pub struct EvolutionPersistence {
    cospans: CospanStore<u32>,
    chains: DocStore<CospanChainRecord>,
}

impl EvolutionPersistence {
    /// Open both tiers on one connection.
    ///
    /// Each tier bootstraps and verifies its own schema, so opening is where a
    /// drifted schema is reported rather than at the first write.
    ///
    /// # Errors
    ///
    /// Fails if either tier's schema cannot be defined or does not verify.
    pub async fn open(store: Store) -> Result<Self> {
        let cospans = CospanStore::open(store.clone()).await?;
        let chains = DocStore::open(store).await?;
        Ok(Self { cospans, chains })
    }

    /// Store a chain under `name`, returning the address of each entry in
    /// order.
    ///
    /// Entries are written in chain order. An entry the cospan tier refuses as
    /// a duplicate morphism resolves to the address that morphism is already
    /// stored at — see the [module documentation](self) for what that means for
    /// the round trip. Storing under a name that is already in use replaces the
    /// document-tier record; the cospans it pointed at stay where they are.
    ///
    /// # Errors
    ///
    /// Fails if a cospan cannot be encoded or written, if a refused duplicate
    /// cannot then be found by its canonical key (reported as
    /// [`StoreError::Corrupt`], since the index refused the write on the
    /// grounds that the morphism is there), or if the chain record cannot be
    /// written.
    pub async fn persist_cospan_chain(
        &self,
        name: &str,
        chain: &[Cospan<u32>],
    ) -> Result<Vec<CospanAddr>> {
        let mut addrs = Vec::with_capacity(chain.len());
        for (position, cospan) in chain.iter().enumerate() {
            let addr = match self.cospans.put(cospan).await {
                Ok(addr) => addr,
                Err(StoreError::Duplicate { .. }) => self
                    .cospans
                    .find_by_canon(cospan)
                    .await?
                    .ok_or_else(|| StoreError::Corrupt {
                        context: format!("cospan chain `{name}`"),
                        detail: format!(
                            "entry {position} was refused as an already-stored morphism, but no \
                             presentation of it is stored under its canonical key"
                        ),
                    })?,
                Err(e) => return Err(e),
            };
            addrs.push(addr);
        }

        let record = CospanChainRecord {
            name: name.to_owned(),
            addrs: addrs.iter().map(|addr| addr.as_str().to_owned()).collect(),
        };
        self.chains.put(name, CHAIN_KIND, &record).await?;
        Ok(addrs)
    }

    /// Store the cospan chain a hypergraph evolution derives from its
    /// deterministic path.
    ///
    /// # Errors
    ///
    /// As [`Self::persist_cospan_chain`].
    pub async fn persist_evolution(
        &self,
        name: &str,
        evolution: &HypergraphEvolution,
    ) -> Result<Vec<CospanAddr>> {
        self.persist_cospan_chain(name, &evolution.to_cospan_chain())
            .await
    }

    /// Load the chain stored under `name`.
    ///
    /// Returns `None` when no chain record is stored under that name. A record
    /// that *is* there but names an address the cospan tier does not hold is a
    /// failure rather than a shorter chain: dropping the entry would silently
    /// change which composite the chain describes.
    ///
    /// # Errors
    ///
    /// [`StoreError::Corrupt`] if the record names an address that is not
    /// well-formed or that the cospan tier does not hold; otherwise a read or
    /// revalidation failure from either tier.
    pub async fn load_cospan_chain(&self, name: &str) -> Result<Option<Vec<Cospan<u32>>>> {
        let Some(record) = self.chains.get(name).await? else {
            return Ok(None);
        };

        let mut chain = Vec::with_capacity(record.addrs.len());
        for (position, raw) in record.addrs.iter().enumerate() {
            let addr = CospanAddr::parse(raw).ok_or_else(|| StoreError::Corrupt {
                context: format!("cospan chain `{name}`"),
                detail: format!("entry {position} (`{raw}`) is not a cospan address"),
            })?;
            let cospan = self
                .cospans
                .get(&addr)
                .await?
                .ok_or_else(|| StoreError::Corrupt {
                    context: format!("cospan chain `{name}`"),
                    detail: format!("entry {position} (`{raw}`) is not stored in the cospan tier"),
                })?;
            chain.push(cospan);
        }
        Ok(Some(chain))
    }
}
