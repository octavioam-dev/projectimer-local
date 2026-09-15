use dioxus::prelude::*;
use tw_merge::tw_merge;
use crate::ui::combobox::*;
use crate::ui::popover::{Popover, PopoverContent, PopoverTrigger};
use crate::ui::skeleton::Skeleton;

pub trait ComboboxItem {
    fn id(&self) -> i64;
    fn label(&self) -> String;
}

/// Generic Combobox Component implemented by myself

#[component]
pub(crate) fn GenericCombobox<T>(
    data: Vec<T>,
    placeholder: String,
    selected_item: Signal<Option<T>>,
    #[props(into, optional)] trigger_class: Option<String>,
    #[props(default = false)] disabled: bool,
) -> Element
where
    T: ComboboxItem + Clone + PartialEq + 'static,
{
    let current_label = selected_item()
        .map(|item| item.label())
        .unwrap_or_else(|| placeholder.clone());

    rsx! {
        Popover {
            PopoverTrigger {
                class: tw_merge!("justify-between w-[200px]", trigger_class.as_deref().unwrap_or("")),
                disabled,
                span { class: "truncate",
                    "{current_label}"
                }
                icons::ChevronsUpDown { class: "ml-auto opacity-50 size-4" }
            }

            PopoverContent { class: "p-0 w-[200px]",
                Command {
                    div { class: "flex gap-2 items-center px-2 border-b",
                        icons::Search { class: "size-4 text-muted-foreground shrink-0" }
                        CommandInput { }
                    }
                    CommandList { class: "min-h-0",
                        CommandEmpty { "No results found." }
                        CommandGroup {
                            for item in data.iter() {
                                {
                                    let item_clone = item.clone();
                                    let is_selected = selected_item() == Some(item.clone());
                                    rsx! {
                                        CommandItem {
                                            value: item.label(),
                                            selected: is_selected,
                                            on_select: move |()| {
                                                selected_item.set(Some(item_clone.clone()));
                                            },
                                            "{item.label()}"
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