use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::sync::atomic::{AtomicUsize, Ordering};

use dioxus::prelude::*;

const PREFIX: &str = "rust_ui";

pub fn use_random_id() -> String {
    format!("_{PREFIX}_{}", generate_hash())
}

pub fn use_random_id_for(element: &str) -> String {
    format!("{}_{PREFIX}_{}", element, generate_hash())
}

/// Stable variant of [`use_random_id_for`]: the id is generated once and kept
/// the same across re-renders (unlike the plain function, which regenerates a
/// new id on every call). Use this inside components that wire DOM attributes
/// and injected `<script>` bodies to the id.
pub fn use_stable_random_id_for(element: &str) -> String {
    use_hook(|| use_random_id_for(element))
}

pub fn use_random_transition_name() -> String {
    let random_id = use_random_id();
    format!("view-transition-name: {random_id}")
}

static COUNTER: AtomicUsize = AtomicUsize::new(1);

fn generate_hash() -> u64 {
    let mut hasher = DefaultHasher::new();
    let counter = COUNTER.fetch_add(1, Ordering::SeqCst);
    counter.hash(&mut hasher);
    hasher.finish()
}