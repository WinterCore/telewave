use std::{
    hash::{DefaultHasher, Hash, Hasher},
    sync::atomic::{AtomicUsize, Ordering},
    time::SystemTime,
};

/// Length of the random suffix appended to every slug.
const SUFFIX_LEN: usize = 5;

/// Slugify `s` and append a short random suffix: "Hello World!" -> "hello-world-xbkqz".
///
/// The core keeps Unicode alphanumerics (Cyrillic titles stay readable), collapses
/// every other run into a single dash, and never leaves a leading/trailing dash.
/// The suffix guarantees a non-empty, collision-unlikely result even for a title
/// that's entirely emoji.
pub fn slugify(s: &str) -> String {
    let core = slug_core(s);

    if core.is_empty() {
        random_suffix(SUFFIX_LEN)
    } else {
        format!("{core}-{}", random_suffix(SUFFIX_LEN))
    }
}

fn slug_core(s: &str) -> String {
    let mut out = String::new();
    // Pending separator: set by non-alphanumerics, only materialized when
    // another alphanumeric actually follows. Collapses runs and makes
    // leading/trailing separators disappear without a trim pass.
    let mut pending_dash = false;

    for c in s.to_lowercase().chars() {
        if c.is_alphanumeric() {
            if pending_dash && !out.is_empty() {
                out.push('-');
            }
            pending_dash = false;
            out.push(c);
        } else {
            pending_dash = true;
        }
    }

    out
}

/// Random-looking lowercase letters, no `rand` dependency.
///
/// Seeds a DefaultHasher with the clock plus a per-process counter (two calls
/// in the same nanosecond still differ), then expands it with an LCG. Uses the
/// high bits of each step — the low bits of an LCG are weak.
fn random_suffix(len: usize) -> String {
    static COUNTER: AtomicUsize = AtomicUsize::new(0);

    let mut hasher = DefaultHasher::new();
    SystemTime::now().hash(&mut hasher);
    COUNTER.fetch_add(1, Ordering::Relaxed).hash(&mut hasher);

    let mut state = hasher.finish();

    (0..len)
        .map(|_| {
            state = state
                .wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            (b'a' + ((state >> 33) % 26) as u8) as char
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn slug_core_basics() {
        assert_eq!(slug_core("Hello World!"), "hello-world");
        assert_eq!(slug_core("  leading and trailing  "), "leading-and-trailing");
        assert_eq!(slug_core("a  --  b"), "a-b"); // runs collapse
        assert_eq!(slug_core("__"), ""); // never a bare dash
    }

    #[test]
    fn slug_core_unicode() {
        assert_eq!(slug_core("Радио Канал"), "радио-канал");
        assert_eq!(slug_core("my_radio_channel"), "my-radio-channel"); // _ is a separator
        assert_eq!(slug_core("🎶🎧"), ""); // emoji-only titles fall back to the suffix
    }

    #[test]
    fn slugify_appends_suffix() {
        let slug = slugify("Hello World!");
        assert!(slug.starts_with("hello-world-"));
        assert_eq!(slug.len(), "hello-world-".len() + SUFFIX_LEN);
        assert!(slug[SUFFIX_LEN + "hello-world-".len()..].chars().all(|c| c.is_ascii_lowercase()));
    }

    #[test]
    fn suffixes_differ_between_calls() {
        assert_ne!(random_suffix(SUFFIX_LEN), random_suffix(SUFFIX_LEN));
    }
}
