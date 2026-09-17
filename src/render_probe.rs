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
use crate::clock_window::{LAP_SECONDS, ProgressRing};
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

/// Replicates the ClockWindow timer + ring progress arc and drives it with REAL tokio time to
/// pin down whether `elapsed_seconds` advances and `stroke-dashoffset` actually shrinks each
/// second (i.e. the outer ring fills up and would reset every lap).
mod timer_probe {
    use super::*;

    thread_local! {
        static ELAPSED: std::cell::RefCell<Option<Signal<u64>>> = std::cell::RefCell::new(None);
    }

    #[component]
    fn TimerRingProbe() -> Element {
        let mut is_running = use_signal(|| true);
        let mut elapsed = use_signal(|| 0u64);
        ELAPSED.with(|slot| *slot.borrow_mut() = Some(elapsed));

        use_future(move || async move {
            loop {
                tokio::time::sleep(std::time::Duration::from_millis(400)).await;
                if is_running() {
                    elapsed.set(elapsed() + 1);
                }
            }
        });

        let circumference = 2.0 * std::f64::consts::PI * 92.0;
        let lap_seconds = 1800u64;
        let progress_fraction = (elapsed() % lap_seconds) as f64 / lap_seconds as f64;
        let dash_offset = circumference * (1.0 - progress_fraction);

        rsx! {
            circle {
                "stroke-dasharray": "{circumference}",
                "stroke-dashoffset": "{dash_offset}",
            }
        }
    }

    #[component]
    fn RingFillProbe() -> Element {
        let mut seconds = use_signal(|| 0u64);
        ELAPSED.with(|slot| *slot.borrow_mut() = Some(seconds));
        rsx! { ProgressRing { elapsed_seconds: seconds } }
    }

fn latest_dashoffset(mutations: &Mutations) -> f64 {
    mutations
        .edits
        .iter()
        .filter_map(|m| {
            if let Mutation::SetAttribute {
                name: "stroke-dashoffset",
                value: dioxus::core::AttributeValue::Text(v),
                ..
            } = m
            {
                v.parse().ok()
            } else {
                None
            }
        })
        .last()
        .expect("no stroke-dashoffset edit emitted")
}

    fn latest_style(mutations: &Mutations) -> String {
        mutations
            .edits
            .iter()
            .filter_map(|m| {
                if let Mutation::SetAttribute {
                    name: "style",
                    value: dioxus::core::AttributeValue::Text(v),
                    ..
                } = m
                {
                    Some(v.clone())
                } else {
                    None
                }
            })
            .last()
            .unwrap_or_default()
    }

    #[test]
    fn real_progress_ring_fills_up_and_resets_every_lap() {
        let mut dom = VirtualDom::new(RingFillProbe);
        let base_text = dom.rebuild_to_vec();

        let mut seconds = ELAPSED.with(|slot| slot.borrow_mut().take().unwrap());
        let circumf = 2.0 * std::f64::consts::PI * 92.0;

        let d0 = latest_dashoffset(&base_text);
        assert!((d0 - circumf).abs() < 0.01, "lap start should be empty (offset={d0})");

        let rt = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .unwrap();
        let (offsets, styles): (Vec<f64>, Vec<String>) = rt.block_on(async {
            let mut offsets = Vec::new();
            let mut styles = Vec::new();
            for target in [1u64, LAP_SECONDS / 4, LAP_SECONDS - 1, LAP_SECONDS] {
                seconds.set(target);
                tokio::time::sleep(std::time::Duration::from_millis(15)).await;
                let frame = dom.render_immediate_to_vec();
                offsets.push(latest_dashoffset(&frame));
                styles.push(latest_style(&frame));
            }
            (offsets, styles)
        });

        assert!(
            offsets.len() == 4,
            "expected one offset per frame, got {offsets:?}"
        );
        assert!(
            offsets[0] < d0 && offsets[0] > d0 * 0.5,
            "first tick should start filling (got {offsets:?})"
        );
        assert!(
            offsets[1] < offsets[0] && offsets[2] < offsets[1],
            "offset should keep shrinking while counting (got {offsets:?})"
        );
        assert!(
            styles[0].contains("transition"),
            "while progressing the ring should keep its 1s fill transition (got {styles:?})"
        );
        let last = offsets.last().unwrap();
        assert!(
            (last - d0).abs() < 0.01,
            "ring should RESET to a fresh lap at LAP_SECONDS (got {last} vs {d0})"
        );
        assert_eq!(
            styles.last().unwrap(),
            "",
            "reset to a new cycle must be instant (no backwards transition)"
        );
    }

    #[test]
    fn timer_ticks_and_ring_fills() {
        let mut dom = VirtualDom::new(TimerRingProbe);
        dom.rebuild_to_vec();
        let elapsed = ELAPSED.with(|slot| slot.borrow_mut().take().unwrap());

        let rt = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .unwrap();
        let offsets: Vec<String> = rt.block_on(async {
            let mut offsets = Vec::new();
            for _ in 0..6 {
                tokio::time::sleep(std::time::Duration::from_millis(450)).await;
                let mutations = dom.render_immediate_to_vec();
                for m in mutations.edits {
                    if let Mutation::SetAttribute {
                        name: "stroke-dashoffset",
                        value: dioxus::core::AttributeValue::Text(v),
                        ..
                    } = m
                    {
                        offsets.push(v.clone());
                    }
                }
            }
            offsets
        });

        assert!(
            *elapsed.peek() >= 2,
            "timer should have ticked ~several times, got elapsed={}",
            *elapsed.peek()
        );
        assert!(
            offsets.len() >= 2,
            "ring should re-render with a new stroke-dashoffset each tick, got {offsets:?}"
        );
        let parsed: Vec<f64> = offsets
            .iter()
            .filter_map(|s| s.parse().ok())
            .collect();
        assert!(
            parsed.len() >= 2 && parsed.first().unwrap() > parsed.last().unwrap(),
            "stroke-dashoffset should DECREASE as the ring fills, got {offsets:?}"
        );
    }
}
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