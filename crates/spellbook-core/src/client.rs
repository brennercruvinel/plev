//! Threaded wrapper around the operations so the UI never blocks.
//!
//! Same shape as plev's `GitClient`: commands flow through an mpsc channel
//! into a worker thread, results come back through a caller-provided
//! callback. The callback normally forwards into a `winit::EventLoopProxy`,
//! but this crate stays UI-agnostic on purpose - nothing here knows what a
//! window is.
//!
//! Why a thread at all: under render-on-demand a frame is only drawn when
//! something changed, so a query blocking the UI thread does not show a
//! spinner, it freezes the window. Every operation here can block - SQLite
//! on a cold page cache, the image cache on disk, Scryfall over the network.
//!
//! One `Command`/`Event` pair per operation. Reads answer with their payload,
//! writes answer with what the write produced (the new id, the removal
//! report), because the screen that sent the write is what shows the toast
//! and refreshes its list.

use std::sync::mpsc::{Sender, channel};
use std::thread::JoinHandle;

use crate::error::{Error, Result};
use crate::ops::{cards, collection, data, decks, games, wishlist};
use crate::types::{Activity, Card};
use crate::wizard::AutoBuildIn;

/// Decoded, already-downscaled artwork as the worker thread produces it.
/// The UI crate turns this into its own `Rgba` and feeds the art cache.
#[derive(Clone, Debug)]
pub struct ArtImage {
    pub width: u32,
    pub height: u32,
    pub pixels: Vec<u8>,
}

/// Everything the dashboard shows, in one round trip: the old page fired four
/// HTTP requests on load; here it is one command and one event.
#[derive(Clone, Debug)]
pub struct HomeData {
    pub totals: collection::CollectionTotals,
    pub wishlist: wishlist::WishlistTotals,
    pub decks: Vec<decks::DeckSummary>,
    pub activity: Vec<Activity>,
}

/// Work the UI asks for.
#[derive(Debug)]
pub enum Command {
    /// Stop the worker. Sent by `Drop`; the thread is joined after it.
    Shutdown,

    // -- Dashboard ------------------------------------------------------------
    LoadHome,

    // -- Card index -----------------------------------------------------------
    SearchCards {
        q: String,
        limit: i64,
    },
    GetCard {
        name: String,
        oracle_id: Option<String>,
    },
    CardPrintings {
        name: String,
    },
    CardVariants {
        name: String,
    },
    SearchSets {
        q: String,
        card: Option<String>,
        limit: i64,
    },

    // -- Collection -----------------------------------------------------------
    ListCollection {
        status: String,
        q: String,
    },
    CardCopies {
        name: String,
    },
    AddCollection(Box<collection::CollectionIn>),
    EditCollection {
        entry_id: i64,
        patch: Box<collection::CollectionEditIn>,
    },
    DeleteCollection {
        entry_id: i64,
    },
    AllocateCollection {
        entry_id: i64,
        deck_id: Option<i64>,
    },

    // -- Decks ----------------------------------------------------------------
    ListDecks,
    GetDeck {
        deck_id: i64,
    },
    DeckSynergy {
        deck_id: i64,
    },
    CreateDeck(Box<decks::DeckIn>),
    UpdateDeck {
        deck_id: i64,
        patch: Box<decks::DeckIn>,
    },
    DeleteDeck {
        deck_id: i64,
        mode: decks::DeleteMode,
    },
    AddDeckCard {
        deck_id: i64,
        card: Box<decks::DeckCardIn>,
    },
    RemoveDeckCard {
        deck_id: i64,
        card_id: i64,
    },
    ImportPreview {
        deck_id: i64,
        text: String,
    },
    ImportCommit {
        deck_id: i64,
        cards: Vec<decks::ImportCard>,
        mode: decks::ImportMode,
    },
    AutoBuild(Box<AutoBuildIn>),
    AutoBuildStatus,
    FetchDeckSynergy {
        deck_id: i64,
    },

    // -- Wishlist -------------------------------------------------------------
    ListWishlist {
        q: String,
    },
    AddWishlist(Box<wishlist::WishlistIn>),
    DeleteWishlist {
        entry_id: i64,
    },
    AcquireWishlist {
        entry_id: i64,
    },

    // -- Games ----------------------------------------------------------------
    ListGames,
    GamesStats,
    AddGame(Box<games::GameIn>),

    // -- Data / updater -------------------------------------------------------
    DataInfo,
    UpdateStatus,
    UpdateStart,
    UpdateCancel,
    ImagesInfo,

    // -- Card art -------------------------------------------------------------
    /// Load `rel` from the image cache, downscaled to `max_edge` device
    /// pixels. Batched by the shell: one command per frame's worth of misses,
    /// not one per tile.
    LoadArt {
        rels: Vec<String>,
        max_edge: u32,
    },
}

/// Results delivered to the event callback, one per command.
#[derive(Debug)]
pub enum Event {
    /// The worker thread is up and the data directories exist.
    Ready,
    /// A command failed in a way no specific variant covers.
    Failed(Error),

    HomeLoaded(Result<Box<HomeData>>),

    CardsFound(Vec<Card>),
    CardLoaded(Result<Box<cards::CardDetail>>),
    PrintingsLoaded {
        name: String,
        printings: Vec<cards::Printing>,
    },
    VariantsLoaded {
        name: String,
        variants: Vec<Card>,
    },
    SetsFound(Vec<cards::SetInfo>),

    CollectionListed(Vec<collection::CollectionEntry>),
    CardCopiesLoaded {
        name: String,
        copies: Box<collection::CardCopies>,
    },
    CollectionAdded(Result<i64>),
    CollectionEdited(Result<()>),
    CollectionDeleted(Result<collection::CopyRemoved>),
    CollectionAllocated(Result<()>),

    DecksListed(Vec<decks::DeckSummary>),
    DeckLoaded {
        deck_id: i64,
        result: Result<Box<decks::DeckDetail>>,
    },
    SynergyLoaded {
        deck_id: i64,
        synergy: Box<decks::Synergy>,
    },
    DeckCreated(Result<i64>),
    DeckUpdated(Result<()>),
    DeckDeleted(Result<usize>),
    DeckCardAdded {
        deck_id: i64,
        result: Result<()>,
    },
    DeckCardRemoved {
        deck_id: i64,
        result: Result<()>,
    },
    ImportPreviewed {
        deck_id: i64,
        result: Result<Box<decks::ImportPreview>>,
    },
    ImportCommitted {
        deck_id: i64,
        result: Result<i64>,
    },
    AutoBuildFinished(Result<()>),
    AutoBuildStatus(decks::AutoBuildStatus),

    WishlistListed(Vec<wishlist::WishlistGroup>),
    WishlistAdded(Result<i64>),
    WishlistDeleted(Result<wishlist::WishlistRemoval>),
    WishlistAcquired(Result<()>),

    GamesListed(Vec<games::Game>),
    GamesStatsLoaded(Box<games::GamesStats>),
    GameAdded(Result<i64>),

    DataInfoLoaded(Box<data::DataInfo>),
    UpdateStatusLoaded(data::UpdateStatus),
    UpdateStarted(Result<()>),
    ImagesInfoLoaded(data::ImagesInfo),

    /// One batch of art answers, in the same order the paths were asked.
    ArtLoaded {
        images: Vec<(String, Option<ArtImage>)>,
    },
}

/// Handle to the worker thread. Dropping it shuts the worker down and waits
/// for the in-flight command to finish, so a half-written row never outlives
/// the process.
pub struct SpellbookClient {
    tx: Sender<Command>,
    handle: Option<JoinHandle<()>>,
}

impl SpellbookClient {
    /// Spawns the worker. `on_event` is called from the worker thread for
    /// every finished command.
    pub fn spawn(on_event: impl Fn(Event) + Send + 'static) -> Self {
        let (tx, rx) = channel::<Command>();
        let handle = std::thread::Builder::new()
            .name("spellbook-data".into())
            .spawn(move || {
                on_event(Event::Ready);
                while let Ok(command) = rx.recv() {
                    if matches!(command, Command::Shutdown) {
                        break;
                    }
                    run(command, &on_event);
                }
            })
            .expect("spawn spellbook worker thread");
        Self {
            tx,
            handle: Some(handle),
        }
    }

    /// Queues a command. Silently ignored once the worker has stopped -
    /// which only happens during shutdown, where there is no one left to
    /// tell.
    pub fn send(&self, command: Command) {
        let _ = self.tx.send(command);
    }

    /// A second sender for the view tree: screens queue their own commands
    /// without the shell forwarding for them.
    pub fn sender(&self) -> Sender<Command> {
        self.tx.clone()
    }
}

impl Drop for SpellbookClient {
    fn drop(&mut self) {
        let _ = self.tx.send(Command::Shutdown);
        if let Some(h) = self.handle.take() {
            let _ = h.join();
        }
    }
}

/// Runs one command on the worker thread and reports the result.
fn run(command: Command, on_event: &(impl Fn(Event) + ?Sized)) {
    use Command as C;
    use Event as E;
    match command {
        // Handled by the loop before it gets here.
        C::Shutdown => {}

        C::LoadHome => on_event(E::HomeLoaded(load_home())),

        C::SearchCards { q, limit } => on_event(E::CardsFound(cards::search(&q, limit))),
        C::GetCard { name, oracle_id } => on_event(E::CardLoaded(
            cards::get(&name, oracle_id.as_deref()).map(Box::new),
        )),
        C::CardPrintings { name } => {
            let printings = cards::printings(&name);
            on_event(E::PrintingsLoaded { name, printings });
        }
        C::CardVariants { name } => {
            let variants = cards::variants(&name);
            on_event(E::VariantsLoaded { name, variants });
        }
        C::SearchSets { q, card, limit } => {
            on_event(E::SetsFound(cards::sets(&q, card.as_deref(), limit)))
        }

        C::ListCollection { status, q } => on_event(E::CollectionListed(
            collection::list_collection(&status, &q),
        )),
        C::CardCopies { name } => {
            let copies = collection::card_copies(&name);
            on_event(E::CardCopiesLoaded {
                name,
                copies: Box::new(copies),
            });
        }
        C::AddCollection(p) => on_event(E::CollectionAdded(collection::add_collection(*p))),
        C::EditCollection { entry_id, patch } => on_event(E::CollectionEdited(
            collection::edit_collection_entry(entry_id, *patch),
        )),
        C::DeleteCollection { entry_id } => on_event(E::CollectionDeleted(
            collection::delete_collection_entry(entry_id),
        )),
        C::AllocateCollection { entry_id, deck_id } => on_event(E::CollectionAllocated(
            collection::allocate_collection(entry_id, deck_id),
        )),

        C::ListDecks => on_event(E::DecksListed(decks::list_decks())),
        C::GetDeck { deck_id } => on_event(E::DeckLoaded {
            deck_id,
            result: decks::get_deck(deck_id).map(Box::new),
        }),
        C::DeckSynergy { deck_id } => {
            let synergy = decks::deck_synergy(deck_id);
            on_event(E::SynergyLoaded {
                deck_id,
                synergy: Box::new(synergy),
            });
        }
        C::CreateDeck(p) => on_event(E::DeckCreated(decks::create_deck(*p))),
        C::UpdateDeck { deck_id, patch } => {
            on_event(E::DeckUpdated(decks::update_deck(deck_id, *patch)))
        }
        C::DeleteDeck { deck_id, mode } => {
            on_event(E::DeckDeleted(decks::delete_deck(deck_id, mode)))
        }
        C::AddDeckCard { deck_id, card } => on_event(E::DeckCardAdded {
            deck_id,
            result: decks::add_deck_card(deck_id, *card),
        }),
        C::RemoveDeckCard { deck_id, card_id } => on_event(E::DeckCardRemoved {
            deck_id,
            result: decks::remove_deck_card(deck_id, card_id),
        }),
        C::ImportPreview { deck_id, text } => on_event(E::ImportPreviewed {
            deck_id,
            result: decks::import_preview(deck_id, &text).map(Box::new),
        }),
        C::ImportCommit {
            deck_id,
            cards: list,
            mode,
        } => on_event(E::ImportCommitted {
            deck_id,
            result: decks::import_commit(deck_id, &list, mode),
        }),
        C::AutoBuild(p) => on_event(E::AutoBuildFinished(decks::auto_build(*p))),
        C::AutoBuildStatus => on_event(E::AutoBuildStatus(decks::auto_build_status())),
        C::FetchDeckSynergy { deck_id } => {
            if let Err(e) = decks::fetch_deck_synergy(deck_id) {
                on_event(E::Failed(e));
            }
        }

        C::ListWishlist { q } => on_event(E::WishlistListed(wishlist::list_wishlist(&q))),
        C::AddWishlist(p) => on_event(E::WishlistAdded(wishlist::add_wishlist(*p))),
        C::DeleteWishlist { entry_id } => on_event(E::WishlistDeleted(
            wishlist::delete_wishlist_entry(entry_id),
        )),
        C::AcquireWishlist { entry_id } => on_event(E::WishlistAcquired(
            wishlist::acquire_wishlist_entry(entry_id),
        )),

        C::ListGames => on_event(E::GamesListed(games::list_games())),
        C::GamesStats => on_event(E::GamesStatsLoaded(Box::new(games::games_stats()))),
        C::AddGame(p) => on_event(E::GameAdded(games::add_game(*p))),

        C::DataInfo => on_event(E::DataInfoLoaded(Box::new(data::info()))),
        C::UpdateStatus => on_event(E::UpdateStatusLoaded(data::update_status())),
        C::UpdateStart => on_event(E::UpdateStarted(data::update_start())),
        C::UpdateCancel => data::update_cancel(),
        C::ImagesInfo => on_event(E::ImagesInfoLoaded(data::images_info())),

        C::LoadArt { rels, max_edge } => {
            let images = rels
                .into_iter()
                .map(|rel| {
                    let art =
                        crate::images::load_scaled(&rel, max_edge).map(|(w, h, pixels)| ArtImage {
                            width: w,
                            height: h,
                            pixels,
                        });
                    (rel, art)
                })
                .collect();
            on_event(E::ArtLoaded { images });
        }
    }
}

fn load_home() -> Result<Box<HomeData>> {
    Ok(Box::new(HomeData {
        totals: collection::collection_total(),
        wishlist: wishlist::wishlist_total(),
        decks: decks::list_decks(),
        activity: games::list_activity(20),
    }))
}
