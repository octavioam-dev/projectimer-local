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
}

fn mount() -> Dom {
    let mut dom = VirtualDom::new(probe);
    dom.rebuild_to_vec();
    let selected = take_selected();
    let mut dom = Dom { dom, selected };
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

    /// Element id of the option button whose rendered text equals `label`.
    fn option_button_id(&mut self, label: &str) -> Option<ElementId> {
        let initial = self.dom.rebuild_to_vec();
        // Buttons appear in sorted order; find the one whose text we're after via the id that the
        // CreateTextNode for `label` shares with its sibling button in template order. Since the
        // template emits CreateTextNode with the *text* id (not the button id), instead resolve by
        // the SetAttribute "role"=option order and match against the CreateTextNode sequence.
        let mut role_ids = Vec::new();
        let mut text_labels = Vec::new();
        for m in &initial.edits {
            match m {
                Mutation::SetAttribute {
                    name: "role",
                    value: dioxus::core::AttributeValue::Text(v),
                    id,
                    ns: _,
                } if v == "option" => role_ids.push(*id),
                Mutation::CreateTextNode { value, .. } => text_labels.push(value.clone()),
                _ => {}
            }
        }
        // `label`'s position among created text nodes matches `role` position among options.
        let label_pos = text_labels.iter().position(|t| t == label)?;
        role_ids.get(label_pos).copied()
    }
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
        texts.iter().any(|t| t.contains("1 selected")),
        "widget should now display '1 selected', got {texts:?}"
    );
}

/// A click on an option (widget -> external) must land in the external signal and stay there
/// across later renders, even though both signals stay in sync.
#[test]
fn clicking_option_updates_external_selection_stably() {
    dioxus::html::set_event_converter(Box::new(SerializedHtmlEventConverter));

    let mut dom = mount();
    let alpha_button = dom.option_button_id("alpha").expect("alpha option rendered");

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
        texts.iter().any(|t| t.contains("1 selected")),
        "widget should now display '1 selected', got {texts:?}"
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