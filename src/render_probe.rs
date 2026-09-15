//! Headless VDom regression tests for the tag multiselect.
//!
//! The `GenericMultiSelect` keeps an internal `HashSet<String>` widget signal and syncs it to an
//! externally-owned `Signal<HashSet<T>>` in BOTH directions (the app prunes/updates the external
//! signal on tag refetch; the widget updates it on clicks). These tests pin the convergence
//! behavior so an app-side write to the external signal can never revert a widget click and vice
//! versa.

#![cfg(test)]

use std::cell::RefCell;
use std::collections::HashSet;
use std::rc::Rc;

use dioxus::core::{ElementId, Mutation, Mutations};
use dioxus::html::SerializedHtmlEventConverter;
use dioxus::prelude::*;

use crate::backend::data::Tag;
use crate::components::multiselect::GenericMultiSelect;

fn sample_tags() -> HashSet<Tag> {
    HashSet::from([Tag::new("alpha"), Tag::new("beta")])
}

thread_local! {
    static SELECTED: std::cell::RefCell<Option<Signal<HashSet<Tag>>>> =
        std::cell::RefCell::new(None);
}

fn probe() -> Element {
    let mut selected = use_signal(|| HashSet::<Tag>::new());
    SELECTED.with(|slot| *slot.borrow_mut() = Some(selected));
    rsx! {
        GenericMultiSelect::<Tag> {
            list: sample_tags(),
            placeholder: "Select tags".to_string(),
            selected,
        }
    }
}

fn take_selected() -> Signal<HashSet<Tag>> {
    SELECTED.with(|slot| slot.borrow_mut().take().expect("probe mounted"))
}

struct Dom {
    dom: VirtualDom,
    selected: Signal<HashSet<Tag>>,
    alpha_option: ElementId,
}

fn mount() -> Dom {
    let mut dom = VirtualDom::new(probe);
    let initial = dom.rebuild_to_vec();
    let selected = take_selected();
    let mut dom = Dom {
        dom,
        selected,
        alpha_option: first_option_id(&initial).expect("alpha option rendered"),
    };
    // Let the mount-time effects run so subscriptions are established before the test acts.
    dom.flush(4);
    dom
}

impl Dom {
    fn flush(&mut self, times: usize) -> Vec<String> {
        let mut texts = Vec::new();
        for _ in 0..times {
            let mutations = self.dom.render_immediate_to_vec();
            for m in mutations.edits {
                match m {
                    Mutation::CreateTextNode { value, .. } => texts.push(value.clone()),
                    Mutation::SetText { value, .. } => texts.push(value.clone()),
                    _ => {}
                }
            }
        }
        texts
    }
}

/// Element id of the FIRST option button (deterministically "alpha": options are emitted in
/// sorted order and `role="option"` is a static attribute baked into a template, never an edit;
/// the dynamic `aria-selected` SetAttribute whose element IS the button is what identifies it).
fn first_option_id(initial: &Mutations) -> Option<ElementId> {
    initial
        .edits
        .iter()
        .find_map(|m| match m {
            Mutation::SetAttribute {
                name: "aria-selected",
                id,
                ..
            } => Some(*id),
            _ => None,
        })
}

/// The app refetches tags on window focus and prunes selections that no longer exist. That write
/// to the external signal must be reflected in the widget AND must never be undone by the widget's
/// own sync effect.
#[test]
fn app_side_update_survives_and_reaches_widget() {
    let mut dom = mount();

    dom.selected.set(HashSet::from([Tag::new("alpha")]));
    let texts = dom.flush(8);

    assert_eq!(*dom.selected.peek(), HashSet::from([Tag::new("alpha")]));
    assert!(
        texts.iter().any(|t| t.contains("alpha")),
        "widget should now display 'alpha', got {texts:?}"
    );
}

/// A click on an option (widget -> external) must land in the external signal and stay there
/// across later renders, even though both signals stay in sync.
#[test]
fn clicking_option_updates_external_selection_stably() {
    dioxus::html::set_event_converter(Box::new(SerializedHtmlEventConverter));

    let mut dom = mount();
    let alpha_button = dom.alpha_option;

    let event = dioxus::core::Event::new(
        Rc::new(dioxus::html::PlatformEventData::new(Box::new(
            dioxus::html::SerializedMouseData::default(),
        ))) as Rc<dyn std::any::Any>,
        true,
    );
    dom.dom.runtime().handle_event("click", event, alpha_button);
    let texts = dom.flush(8);

    assert_eq!(
        *dom.selected.peek(),
        HashSet::from([Tag::new("alpha")]),
        "clicked option must reach the external signal"
    );
    assert!(
        texts.iter().any(|t| t.contains("alpha")),
        "widget should now display 'alpha', got {texts:?}"
    );

    // Stability: extra renders / effect flushes must not revert the selection.
    let _ = dom.flush(8);
    assert_eq!(*dom.selected.peek(), HashSet::from([Tag::new("alpha")]));
}

/// Selecting, then having the app clear (e.g. all tags deleted), then selecting again must each
/// converge.
#[test]
fn repeated_selection_and_clears_converge() {
    let mut dom = mount();

    dom.selected.set(HashSet::from([Tag::new("beta")]));
    let _ = dom.flush(6);
    assert_eq!(*dom.selected.peek(), HashSet::from([Tag::new("beta")]));

    dom.selected.set(HashSet::new());
    let _ = dom.flush(6);
    assert_eq!(*dom.selected.peek(), HashSet::new());

    dom.selected.set(HashSet::from([Tag::new("alpha"), Tag::new("beta")]));
    let _ = dom.flush(6);
    assert_eq!(
        *dom.selected.peek(),
        HashSet::from([Tag::new("alpha"), Tag::new("beta")])
    );
}