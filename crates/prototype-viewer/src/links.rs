//! Project links shown on the login screen. Empty = not configured yet: the
//! button is shown disabled and nothing is fetched. The README header carries
//! the same Discord and website links: change them there too.

/// Source code repository (GitHub page).
pub const GITHUB: &str = "https://github.com/deadlyxreturn/Prototype";
/// Issue template chooser, opened in the browser by "Report a bug" (login
/// screen, top bar, Help menu) and by the crash window.
pub const NEW_ISSUE: &str = "https://github.com/deadlyxreturn/Prototype/issues/new/choose";
/// The project's wiki, generated from the guides of `docs/`
/// (`scripts/build-wiki.ps1`): Help › Prototype wiki.
pub const WIKI: &str = "https://github.com/deadlyxreturn/Prototype/wiki";
/// Community Discord invite.
/// Prototype's website: news, downloads and support.
pub const PROTOTYPE_SITE: &str = "https://www.prototypesl.com";
pub const DISCORD: &str = "https://www.prototypesl.com";
/// Project website.
pub const WEBSITE: &str = "https://prototypesl.com";
/// The privacy policy on the website (Preferences › Privacy › Your data).
pub const PRIVACY_POLICY: &str = "https://prototypesl.com/privacy/";
/// News feed: this folder of `main`, hand-edited Markdown files, one
/// `<language>.md` per UI language (`news/` at the repository root). The
/// viewer keeps what it read in its cache as a JSON array of
/// `{"date", "title", "summary", "url"}`.
pub const NEWS_FEED: &str = "https://raw.githubusercontent.com/deadlyxreturn/Prototype/main/news/";
/// Files of the source repository as GitHub serves them unrendered; a branch
/// name and a path follow.
pub const RAW_FILES: &str = "https://raw.githubusercontent.com/deadlyxreturn/Prototype/";
