use dioxus::prelude::*;
use std::collections::HashSet;
use std::hash::Hash;
use crate::ui::multi_select::{
    MultiSelect, MultiSelectContent, MultiSelectGroup, MultiSelectItem, MultiSelectOption, MultiSelectTrigger,
    MultiSelectValue,
};

/// Bi-directional, updates when the signal itself updates
#[component]
pub fn GenericMultiSelect<T>(
    list: HashSet<T>,
    placeholder: String,
    mut selected: Signal<HashSet<T>>,
) -> Element
where
    T: AsRef<str> + Clone + PartialEq + Eq + Hash + 'static,
{
    let mut internal_values = use_signal({
        let initial: HashSet<String> = selected()
            .iter()
            .map(|item| item.as_ref().to_string())
            .collect();
        move || initial.clone()
    });

    // Widget -> external
    let list_for_effect = list.clone();
    use_effect(move || {
        let values = internal_values();
        let matched: HashSet<T> = list_for_effect
            .iter()
            .filter(|item| values.contains(item.as_ref()))
            .cloned()
            .collect();
        if matched != selected() {
            selected.set(matched);
        }
    });

    // External -> widget
    use_effect(move || {
        let ext_strings: HashSet<String> = selected()
            .iter()
            .map(|item| item.as_ref().to_string())
            .collect();
        if ext_strings != internal_values() {
            internal_values.set(ext_strings);
        }
    });

    // Sort once per render for stable display order.
    let mut sorted_list: Vec<&T> = list.iter().collect();
    sorted_list.sort_by(|a, b| a.as_ref().cmp(b.as_ref()));

    rsx! {
        div { class: "mx-auto",
            MultiSelect { values: internal_values,
                MultiSelectTrigger { class: "w-[250px]",
                    MultiSelectValue { placeholder: placeholder.clone() }
                }
                MultiSelectContent {
                    MultiSelectGroup {
                        for item in sorted_list.into_iter() {
                            {
                                let value: String = item.as_ref().to_string();
                                rsx! {
                                    MultiSelectItem {
                                        MultiSelectOption { value: value.clone(),
                                            "{value}"
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}