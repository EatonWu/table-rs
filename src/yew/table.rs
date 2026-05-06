use gloo_timers::callback::Timeout;
use std::collections::HashMap;
use yew::prelude::*;
use yew_bootstrap::component::form::{FormControl, FormControlType, SelectOption};
use yew_bootstrap::component::{Button, ButtonSize};
use yew_bootstrap::util::Color;
use yew_icons::{Icon, IconData};

use crate::yew::body::TableBody;
use crate::yew::controls::PaginationControls;
use crate::yew::header::TableHeader;
use crate::yew::types::{FilterType, SortOrder, TableProps};

#[derive(Clone, PartialEq)]
struct ColumnFilter {
    column_id: &'static str,
    operator: FilterOperator,
    value: String,
}

#[derive(Clone)]
struct FilterColumnOption {
    id: &'static str,
    label: String,
    filter_type: FilterType,
}

#[derive(Clone, Copy, PartialEq)]
enum FilterOperator {
    Contains,
    Equals,
    NotEqual,
    StartsWith,
    EndsWith,
    GreaterThan,
    GreaterThanOrEqual,
    LessThan,
    LessThanOrEqual,
    IsEmpty,
    IsNotEmpty,
}

fn default_operator(filter_type: &FilterType) -> FilterOperator {
    match filter_type {
        FilterType::String => FilterOperator::Contains,
        FilterType::Number => FilterOperator::Equals,
        FilterType::Bool => FilterOperator::Equals,
        FilterType::Enum(_) => FilterOperator::Equals,
    }
}

fn operator_label(filter_type: &FilterType, operator: FilterOperator) -> &'static str {
    match filter_type {
        FilterType::String => match operator {
            FilterOperator::Contains => "contains",
            FilterOperator::Equals => "equals",
            FilterOperator::NotEqual => "not equals",
            FilterOperator::StartsWith => "starts with",
            FilterOperator::EndsWith => "ends with",
            FilterOperator::IsEmpty => "is empty",
            FilterOperator::IsNotEmpty => "is not empty",
            _ => "contains",
        },
        FilterType::Number => match operator {
            FilterOperator::Equals => "=",
            FilterOperator::NotEqual => "!=",
            FilterOperator::GreaterThan => ">",
            FilterOperator::GreaterThanOrEqual => ">=",
            FilterOperator::LessThan => "<",
            FilterOperator::LessThanOrEqual => "<=",
            FilterOperator::IsEmpty => "is empty",
            FilterOperator::IsNotEmpty => "is not empty",
            _ => "=",
        },
        FilterType::Bool => match operator {
            FilterOperator::Equals => "is",
            FilterOperator::NotEqual => "is not",
            FilterOperator::IsEmpty => "is empty",
            FilterOperator::IsNotEmpty => "is not empty",
            _ => "is",
        },
        FilterType::Enum(_) => match operator {
            FilterOperator::Equals => "is",
            FilterOperator::NotEqual => "is not",
            _ => "is",
        },
    }
}

fn matches_filter(
    filter_type: &FilterType,
    operator: FilterOperator,
    row_value: &str,
    filter_value: &str,
) -> bool {
    let row_trimmed = row_value.trim();
    let filter_trimmed = filter_value.trim();

    match operator {
        FilterOperator::IsEmpty => return row_trimmed.is_empty(),
        FilterOperator::IsNotEmpty => return !row_trimmed.is_empty(),
        _ => {}
    }

    if filter_trimmed.is_empty() {
        return true;
    }

    match filter_type {
        FilterType::String => {
            let row_norm = row_trimmed.to_lowercase();
            let filter_norm = filter_trimmed.to_lowercase();
            match operator {
                FilterOperator::Contains => row_norm.contains(&filter_norm),
                FilterOperator::Equals => row_norm == filter_norm,
                FilterOperator::NotEqual => row_norm != filter_norm,
                FilterOperator::StartsWith => row_norm.starts_with(&filter_norm),
                FilterOperator::EndsWith => row_norm.ends_with(&filter_norm),
                _ => false,
            }
        }
        FilterType::Number => {
            let Ok(filter_number) = filter_trimmed.parse::<f64>() else {
                return false;
            };
            let Ok(row_number) = row_trimmed.parse::<f64>() else {
                return false;
            };
            match operator {
                FilterOperator::Equals => row_number == filter_number,
                FilterOperator::NotEqual => row_number != filter_number,
                FilterOperator::GreaterThan => row_number > filter_number,
                FilterOperator::GreaterThanOrEqual => row_number >= filter_number,
                FilterOperator::LessThan => row_number < filter_number,
                FilterOperator::LessThanOrEqual => row_number <= filter_number,
                _ => false,
            }
        }
        FilterType::Bool => {
            let row_norm = row_trimmed.to_lowercase();
            let filter_norm = filter_trimmed.to_lowercase();
            match operator {
                FilterOperator::Equals => row_norm == filter_norm,
                FilterOperator::NotEqual => row_norm != filter_norm,
                _ => false,
            }
        }
        FilterType::Enum(_) => match operator {
            FilterOperator::Equals => row_trimmed == filter_trimmed,
            FilterOperator::NotEqual => row_trimmed != filter_trimmed,
            _ => false,
        },
    }
}

/// A fully featured table component with pagination, sorting, and search support.
///
/// This component renders a complete `<table>` element, including headers (`<thead>`), body (`<tbody>`),
/// and optional features such as client-side sorting, pagination, and search input.
/// It is built using Yew and supports flexible styling and customization.
///
/// # Arguments
/// * `props` - The properties passed to the component.
///   - `data` - A `Vec<HashMap<&'static str, String>>` representing the table's row data.
///   - `columns` - A `Vec<Column>` defining the structure and behavior of each column.
///   - `page_size` - A `usize` defining how many rows to show per page.
///   - `loading` - A `bool` indicating whether the table is in a loading state.
///   - `classes` - A `TableClasses` struct for customizing class names of elements.
///   - `styles` - A `HashMap<&'static str, &'static str>` for inline style overrides.
///   - `paginate` - A `bool` controlling whether pagination controls are displayed.
///   - `search` - A `bool` enabling a search input above the table.
///   - `texts` - A `TableTexts` struct for customizing placeholder and fallback texts.
///
/// # Features
/// - **Client-side search**
/// - **Column sorting** (ascending/descending toggle)
/// - **Pagination controls**
/// - **Custom class and inline style support**
/// - Displays a loading row or empty state message when appropriate
///
/// # Returns
/// (Html): A complete, styled and interactive table component rendered in Yew.
///
/// # Examples
/// ```rust
/// use yew::prelude::*;
/// use maplit::hashmap;
/// use table_rs::yew::table::Table;
/// use table_rs::yew::types::{Column, TableClasses, TableTexts};
///
/// #[function_component(App)]
/// pub fn app() -> Html {
///     let data = vec![
///         hashmap! { "name" => "Ferris".into(), "email" => "ferris@opensass.org".into() },
///         hashmap! { "name" => "Ferros".into(), "email" => "ferros@opensass.org".into() },
///     ];
///
///     let columns = vec![
///         Column { id: "name", header: "Name", sortable: true, ..Default::default() },
///         Column { id: "email", header: "Email", sortable: false, ..Default::default() },
///     ];
///
///     html! {
///         <Table
///             data={data}
///             columns={columns}
///             page_size={10}
///             loading={false}
///             paginate={true}
///             search={true}
///             classes={TableClasses::default()}
///             texts={TableTexts::default()}
///         />
///     }
/// }
/// ```
///
/// # See Also
/// - [MDN table Element](https://developer.mozilla.org/en-US/docs/Web/HTML/Reference/Elements/table)
#[function_component(Table)]
pub fn table(props: &TableProps) -> Html {
    let TableProps {
        data,
        columns,
        page_size,
        loading,
        classes,
        styles,
        paginate,
        online_paginated,
        online_page,
        online_total_pages,
        on_online_page_change,
        search,
        filterable_columns,
        texts,
        row_end_component,
        default_sort_column,
        default_sort_order,
    } = props;

    let page = use_state(|| 0);
    let sort_column = use_state(|| default_sort_column.clone());
    let sort_order = use_state(|| default_sort_order.clone());
    let search_query = use_state(|| String::new());
    let debounced_search = use_state(|| None::<Timeout>);
    let filters = use_state(|| Vec::<ColumnFilter>::new());
    let active_filter_dropdown = use_state(|| None::<usize>);
    let show_add_filter_menu = use_state(|| false);

    let on_search_change = {
        let debounced_search = debounced_search.clone();
        let search_query = search_query.clone();
        let page = page.clone();
        Callback::from(move |e: InputEvent| {
            let search_query = search_query.clone();
            let page = page.clone();
            // TODO: Add debounce
            // let debounced_search_ref = debounced_search.clone();
            let input: web_sys::HtmlInputElement = e.target_unchecked_into();
            let value = input.value();

            // let prev_timeout = {
            //     debounced_search_ref.take()
            // };

            // if let Some(prev) = prev_timeout {
            //     prev.cancel();
            // }

            let timeout = Timeout::new(50, move || {
                search_query.set(value.clone());
                page.set(0);
            });

            debounced_search.set(Some(timeout));
        })
    };

    let mut filtered_rows = data.clone();

    if !filterable_columns.is_empty() {
        filtered_rows.retain(|row| {
            filters.iter().all(|filter| {
                let Some(filter_type) = filterable_columns.get(filter.column_id) else {
                    return true;
                };

                let row_value = row
                    .get(filter.column_id)
                    .map(|val| val.as_str())
                    .unwrap_or("");
                matches_filter(filter_type, filter.operator, row_value, &filter.value)
            })
        });
    }

    if !search_query.is_empty() {
        filtered_rows.retain(|row| {
            columns.iter().any(|col| {
                row.get(col.id)
                    .map(|v| v.to_lowercase().contains(&search_query.to_lowercase()))
                    .unwrap_or(false)
            })
        });
    }

    if let Some(col_id) = *sort_column {
        if let Some(col) = columns.iter().find(|c| c.id == col_id) {
            filtered_rows.sort_by(|a, b| {
                let val = "".to_string();
                let a_val = a.get(col.id).unwrap_or(&val);
                let b_val = b.get(col.id).unwrap_or(&val);
                match *sort_order {
                    SortOrder::Asc => a_val.cmp(b_val),
                    SortOrder::Desc => b_val.cmp(a_val),
                }
            });
        }
    }

    let total_pages = if *online_paginated {
        (*online_total_pages).max(1)
    } else {
        (filtered_rows.len() as f64 / *page_size as f64)
            .ceil()
            .max(1.0) as usize
    };
    let current_page = if *online_paginated {
        (*online_page).min(total_pages.saturating_sub(1))
    } else {
        (*page).min(total_pages.saturating_sub(1))
    };
    let page_rows: Vec<HashMap<&'static str, String>> = if *online_paginated {
        filtered_rows
    } else {
        let start = current_page * page_size;
        let end = ((current_page + 1) * page_size).min(filtered_rows.len());
        filtered_rows[start..end].to_vec()
    };

    let on_page_change = {
        let page = page.clone();
        let on_online_page_change = on_online_page_change.clone();
        let online_paginated = *online_paginated;
        Callback::from(move |next_page: usize| {
            if online_paginated {
                on_online_page_change.emit(next_page);
            } else {
                page.set(next_page);
            }
        })
    };

    let on_sort_column = {
        let sort_column = sort_column.clone();
        let sort_order = sort_order.clone();
        Callback::from(move |id: &'static str| {
            if Some(id) == *sort_column {
                sort_order.set(match *sort_order {
                    SortOrder::Asc => SortOrder::Desc,
                    SortOrder::Desc => SortOrder::Asc,
                });
            } else {
                sort_column.set(Some(id));
                sort_order.set(SortOrder::Asc);
            }
        })
    };

    let available_filter_columns: Vec<FilterColumnOption> = columns
        .iter()
        .filter_map(|col| {
            filterable_columns
                .get(col.id)
                .map(|filter_type| FilterColumnOption {
                    id: col.id,
                    label: if col.header.is_empty() {
                        col.id.to_string()
                    } else {
                        col.header.to_string()
                    },
                    filter_type: filter_type.clone(),
                })
        })
        .collect();

    let on_toggle_add_menu = {
        let show_add_filter_menu = show_add_filter_menu.clone();
        Callback::from(move |_| {
            show_add_filter_menu.set(!*show_add_filter_menu);
        })
    };

    let on_add_filter = {
        let filters = filters.clone();
        let available_filter_columns = available_filter_columns.clone();
        let page = page.clone();
        let active_filter_dropdown = active_filter_dropdown.clone();
        let show_add_filter_menu = show_add_filter_menu.clone();
        Callback::from(move |id: String| {
            if id.is_empty() {
                return;
            }
            let mut next = (*filters).clone();
            if let Some(existing_index) = next
                .iter()
                .position(|filter| filter.column_id == id.as_str())
            {
                active_filter_dropdown.set(Some(existing_index));
                show_add_filter_menu.set(false);
                return;
            }
            if let Some(option) = available_filter_columns
                .iter()
                .find(|option| option.id == id.as_str())
            {
                next.push(ColumnFilter {
                    column_id: option.id,
                    operator: default_operator(&option.filter_type),
                    value: String::new(),
                });
                let new_index = next.len().saturating_sub(1);
                filters.set(next);
                page.set(0);
                active_filter_dropdown.set(Some(new_index));
                show_add_filter_menu.set(false);
            }
        })
    };

    let container_style = format!(
        "display: flex; flex-direction: column; height: 100%; overflow: auto; {}",
        styles.get("container").unwrap_or(&"")
    );
    let table_style = format!(
        "flex: 1 1 auto; min-height: 0; border-collapse: separate; border-spacing: 0; {}",
        styles.get("table").unwrap_or(&"")
    );

    let search_style = format!(
        "position: sticky; top: 0; z-index: 2; background-color: var(--bs-body-bg, white); display: flex; gap: 8px; align-items: center; {}",
        styles.get("search").unwrap_or(&"")
    );

    let filter_panel_style = format!(
        "margin-top: 8px; display: flex; flex-direction: column; gap: 6px; {}",
        styles.get("filter_panel").unwrap_or(&"")
    );
    let filter_bar_style = format!(
        "display: flex; flex-wrap: wrap; gap: 8px; align-items: center; {}",
        styles.get("filter_bar").unwrap_or(&"")
    );
    let filter_dropdown_style = format!(
        "position: absolute; top: 100%; left: 0; margin-top: 0.25rem; z-index: 1080; min-width: 240px; padding: 10px; display: flex; flex-wrap: wrap; gap: 8px; align-items: center; {}",
        styles.get("filter_dropdown").unwrap_or(&"")
    );

    let pagination_style = format!(
        "margin-top: auto; position: sticky; bottom: 0; z-index: 2; background-color: var(--bs-body-bg, white); {}",
        styles.get("pagination").unwrap_or(&"")
    );

    html! {
        <div class={classes.container} style={container_style}>
            { if *search || !filterable_columns.is_empty() {
                let filters_enabled = !filterable_columns.is_empty();
                let filters = filters.clone();
                let on_add_filter = on_add_filter.clone();
                let on_clear_filters = {
                    let filters = filters.clone();
                    let page = page.clone();
                    let active_filter_dropdown = active_filter_dropdown.clone();
                    let show_add_filter_menu = show_add_filter_menu.clone();
                    Callback::from(move |_| {
                        filters.set(Vec::new());
                        page.set(0);
                        active_filter_dropdown.set(None);
                        show_add_filter_menu.set(false);
                    })
                };
                html! {
                    <>
                        { if *search {
                            html! {
                                <div style={search_style}>
                                    <input
                                        class={classes.search_input}
                                        type="text"
                                        value={(*search_query).clone()}
                                        placeholder={texts.search_placeholder}
                                        aria-label="Search table"
                                        oninput={on_search_change}
                                    />
                                </div>
                            }
                        } else {
                            html! {}
                        } }
                        { if filters_enabled {
                            let filters_for_add = filters.clone();
                            let page = page.clone();
                            let active_filter_dropdown = active_filter_dropdown.clone();
                            let available_for_add: Vec<FilterColumnOption> = available_filter_columns
                                .iter()
                                .filter(|option| !filters_for_add.iter().any(|filter| filter.column_id == option.id))
                                .cloned()
                                .collect();
                            html! {
                                <div class={classes.filter_panel} style={filter_panel_style}>
                                    <div class={classes!(classes.filter_row, "d-flex", "gap-2", "align-items-center", "flex-wrap")} style={filter_bar_style}>
                                        { for filters.iter().enumerate().map(|(index, filter)| {
                                            let active_filter_dropdown = active_filter_dropdown.clone();
                                            let active_filter_for_toggle = active_filter_dropdown.clone();
                                            let on_toggle = Callback::from(move |_| {
                                                if *active_filter_for_toggle == Some(index) {
                                                    active_filter_for_toggle.set(None);
                                                } else {
                                                    active_filter_for_toggle.set(Some(index));
                                                }
                                            });
                                            let filter_type = filterable_columns.get(filter.column_id).cloned();
                                            let operator_text = filter_type
                                                .as_ref()
                                                .map(|filter_type| operator_label(filter_type, filter.operator))
                                                .unwrap_or("contains");
                                            let label = filter.column_id.to_string();
                                            let display_label = available_filter_columns
                                                .iter()
                                                .find(|option| option.id == filter.column_id)
                                                .map(|option| option.label.clone())
                                                .unwrap_or(label);
                                            let value_preview = if filter.value.is_empty() { "Any".to_string() } else { filter.value.clone() };
                                            let button_text = format!("{} {} {}", display_label, operator_text, value_preview);
                                            let is_active = *active_filter_dropdown == Some(index);
                                            let filters_for_operator = filters.clone();
                                            let page_for_operator = page.clone();
                                            let on_operator_change = Callback::from(move |e: Event| {
                                                let input: web_sys::HtmlSelectElement = e.target_unchecked_into();
                                                let value = input.value();
                                                let mut next = (*filters_for_operator).clone();
                                                if let Some(filter_entry) = next.get_mut(index) {
                                                    filter_entry.operator = match value.as_str() {
                                                        "contains" => FilterOperator::Contains,
                                                        "equals" => FilterOperator::Equals,
                                                        "not_equals" => FilterOperator::NotEqual,
                                                        "starts_with" => FilterOperator::StartsWith,
                                                        "ends_with" => FilterOperator::EndsWith,
                                                        "gt" => FilterOperator::GreaterThan,
                                                        "gte" => FilterOperator::GreaterThanOrEqual,
                                                        "lt" => FilterOperator::LessThan,
                                                        "lte" => FilterOperator::LessThanOrEqual,
                                                        "is_empty" => FilterOperator::IsEmpty,
                                                        "is_not_empty" => FilterOperator::IsNotEmpty,
                                                        _ => FilterOperator::Contains,
                                                    };
                                                }
                                                filters_for_operator.set(next);
                                                page_for_operator.set(0);
                                            });

                                            let filters_for_input = filters.clone();
                                            let page_for_input = page.clone();
                                            let on_value_change = Callback::from(move |e: InputEvent| {
                                                let input: web_sys::HtmlInputElement = e.target_unchecked_into();
                                                let value = input.value();
                                                let mut next = (*filters_for_input).clone();
                                                if let Some(filter_entry) = next.get_mut(index) {
                                                    filter_entry.value = value;
                                                }
                                                filters_for_input.set(next);
                                                page_for_input.set(0);
                                            });

                                            let filters_for_remove = filters.clone();
                                            let page_for_remove = page.clone();
                                            let active_filter_dropdown = active_filter_dropdown.clone();
                                            let on_remove = Callback::from(move |_| {
                                                let mut next = (*filters_for_remove).clone();
                                                if index < next.len() {
                                                    next.remove(index);
                                                }
                                                filters_for_remove.set(next);
                                                page_for_remove.set(0);
                                                active_filter_dropdown.set(None);
                                            });
                                            html! {
                                                <div class="dropdown position-relative">
                                                    <Button
                                                        class={classes.filter_button.to_string()}
                                                        onclick={on_toggle}
                                                        style={Color::Secondary}
                                                        size={ButtonSize::Small}
                                                        outline={!is_active}
                                                    >
                                                        <span>{ button_text }</span>
                                                    </Button>
                                                    { if is_active {
                                                        let filter_type = filter_type.clone();
                                                        let operator_value = filter.operator;
                                                        html! {
                                                            <div
                                                                class={classes!("dropdown-menu", "show")}
                                                                style={filter_dropdown_style.clone()}
                                                                data-bs-popper="static"
                                                            >
                                                                { if !matches!(&filter_type, Some(FilterType::Enum(_))) {
                                                                    html! {
                                                                        <FormControl
                                                                            id={format!("filter-operator-{}", index)}
                                                                            ctype={FormControlType::Select}
                                                                            class={classes!(classes.filter_operator)}
                                                                            onchange={on_operator_change}
                                                                        >
                                                                            { match &filter_type {
                                                                                Some(FilterType::String) => html! {
                                                                                    <>
                                                                                        <SelectOption label="contains" value="contains" selected={operator_value == FilterOperator::Contains} />
                                                                                        <SelectOption label="equals" value="equals" selected={operator_value == FilterOperator::Equals} />
                                                                                        <SelectOption label="not equals" value="not_equals" selected={operator_value == FilterOperator::NotEqual} />
                                                                                        <SelectOption label="starts with" value="starts_with" selected={operator_value == FilterOperator::StartsWith} />
                                                                                        <SelectOption label="ends with" value="ends_with" selected={operator_value == FilterOperator::EndsWith} />
                                                                                        <SelectOption label="is empty" value="is_empty" selected={operator_value == FilterOperator::IsEmpty} />
                                                                                        <SelectOption label="is not empty" value="is_not_empty" selected={operator_value == FilterOperator::IsNotEmpty} />
                                                                                    </>
                                                                                },
                                                                                Some(FilterType::Number) => html! {
                                                                                    <>
                                                                                        <SelectOption label="=" value="equals" selected={operator_value == FilterOperator::Equals} />
                                                                                        <SelectOption label="!=" value="not_equals" selected={operator_value == FilterOperator::NotEqual} />
                                                                                        <SelectOption label=">" value="gt" selected={operator_value == FilterOperator::GreaterThan} />
                                                                                        <SelectOption label=">=" value="gte" selected={operator_value == FilterOperator::GreaterThanOrEqual} />
                                                                                        <SelectOption label="<" value="lt" selected={operator_value == FilterOperator::LessThan} />
                                                                                        <SelectOption label="<=" value="lte" selected={operator_value == FilterOperator::LessThanOrEqual} />
                                                                                        <SelectOption label="is empty" value="is_empty" selected={operator_value == FilterOperator::IsEmpty} />
                                                                                        <SelectOption label="is not empty" value="is_not_empty" selected={operator_value == FilterOperator::IsNotEmpty} />
                                                                                    </>
                                                                                },
                                                                                Some(FilterType::Bool) => html! {
                                                                                    <>
                                                                                        <SelectOption label="is" value="equals" selected={operator_value == FilterOperator::Equals} />
                                                                                        <SelectOption label="is not" value="not_equals" selected={operator_value == FilterOperator::NotEqual} />
                                                                                        <SelectOption label="is empty" value="is_empty" selected={operator_value == FilterOperator::IsEmpty} />
                                                                                        <SelectOption label="is not empty" value="is_not_empty" selected={operator_value == FilterOperator::IsNotEmpty} />
                                                                                    </>
                                                                                },
                                                                                _ => html! {},
                                                                            } }
                                                                        </FormControl>
                                                                    }
                                                                } else {
                                                                    html! {}
                                                                } }
                                                                { match &filter_type {
                                                                    Some(FilterType::Bool) => {
                                                                        let on_bool_change = {
                                                                            let filters_for_bool = filters.clone();
                                                                            let page_for_bool = page.clone();
                                                                            Callback::from(move |e: Event| {
                                                                                let input: web_sys::HtmlSelectElement = e.target_unchecked_into();
                                                                                let value = input.value();
                                                                                let mut next = (*filters_for_bool).clone();
                                                                                if let Some(filter_entry) = next.get_mut(index) {
                                                                                    filter_entry.value = value;
                                                                                }
                                                                                filters_for_bool.set(next);
                                                                                page_for_bool.set(0);
                                                                            })
                                                                        };
                                                                        html! {
                                                                            <FormControl
                                                                                id={format!("filter-value-bool-{}", index)}
                                                                                ctype={FormControlType::Select}
                                                                                class={classes!(classes.filter_input)}
                                                                                onchange={on_bool_change}
                                                                                disabled={matches!(operator_value, FilterOperator::IsEmpty | FilterOperator::IsNotEmpty)}
                                                                            >
                                                                                <SelectOption label="Any" value="" selected={filter.value.is_empty()} />
                                                                                <SelectOption label="true" value="true" selected={filter.value == "true"} />
                                                                                <SelectOption label="false" value="false" selected={filter.value == "false"} />
                                                                            </FormControl>
                                                                        }
                                                                    }
                                                                    Some(FilterType::Enum(options)) => {
                                                                        let on_enum_change = {
                                                                            let filters_for_enum = filters.clone();
                                                                            let page_for_enum = page.clone();
                                                                            Callback::from(move |e: Event| {
                                                                                let input: web_sys::HtmlSelectElement = e.target_unchecked_into();
                                                                                let value = input.value();
                                                                                let mut next = (*filters_for_enum).clone();
                                                                                if let Some(filter_entry) = next.get_mut(index) {
                                                                                    filter_entry.value = value;
                                                                                }
                                                                                filters_for_enum.set(next);
                                                                                page_for_enum.set(0);
                                                                            })
                                                                        };
                                                                        html! {
                                                                            <FormControl
                                                                                id={format!("filter-value-enum-{}", index)}
                                                                                ctype={FormControlType::Select}
                                                                                class={classes!(classes.filter_input)}
                                                                                onchange={on_enum_change}
                                                                            >
                                                                                <SelectOption label="Any" value="" selected={filter.value.is_empty()} />
                                                                                { for options.iter().map(|option| {
                                                                                    html! {
                                                                                        <SelectOption
                                                                                            label={option.clone()}
                                                                                            value={option.clone()}
                                                                                            selected={filter.value == *option}
                                                                                        />
                                                                                    }
                                                                                }) }
                                                                            </FormControl>
                                                                        }
                                                                    }
                                                                    _ => {
                                                                        let control_type = match &filter_type {
                                                                            Some(FilterType::Number) => FormControlType::Number { min: None, max: None },
                                                                            _ => FormControlType::Text,
                                                                        };
                                                                        html! {
                                                                            <FormControl
                                                                                id={format!("filter-value-{}", index)}
                                                                                ctype={control_type}
                                                                                class={classes!(classes.filter_input)}
                                                                                value={filter.value.clone()}
                                                                                placeholder={"Filter..."}
                                                                                disabled={matches!(operator_value, FilterOperator::IsEmpty | FilterOperator::IsNotEmpty)}
                                                                                oninput={on_value_change}
                                                                            />
                                                                        }
                                                                    }
                                                                } }
                                                                <Button
                                                                    class={classes.filter_remove_button.to_string()}
                                                                    onclick={on_remove}
                                                                    style={Color::Danger}
                                                                    size={ButtonSize::Small}
                                                                    outline={true}
                                                                >
                                                                    { "Remove" }
                                                                </Button>
                                                            </div>
                                                        }
                                                    } else {
                                                        html! {}
                                                    } }
                                                </div>
                                            }
                                        }) }
                                    </div>
                                    <div class={classes!("d-flex", "gap-2", "flex-wrap", "align-items-center")}>
                                        <Button
                                            class={classes.filter_button.to_string()}
                                            onclick={on_toggle_add_menu.clone()}
                                            style={Color::Secondary}
                                            size={ButtonSize::Small}
                                            outline={true}
                                        >
                                            <Icon data={IconData::BOOTSTRAP_FILTER} width={"1em".to_owned()} />
                                            <span class="ms-1">{ "+ Filter" }</span>
                                        </Button>
                                        { if *show_add_filter_menu {
                                            let on_add_filter = on_add_filter.clone();
                                            html! {
                                                <FormControl
                                                    id="filter-add-column"
                                                    ctype={FormControlType::Select}
                                                    class={classes!(classes.filter_select)}
                                                    onchange={Callback::from(move |e: Event| {
                                                        let input: web_sys::HtmlSelectElement = e.target_unchecked_into();
                                                        on_add_filter.emit(input.value());
                                                    })}
                                                >
                                                    <SelectOption label="Select column..." value="" selected={true} />
                                                    { for available_for_add.iter().map(|option| {
                                                        html! {
                                                            <SelectOption
                                                                label={option.label.clone()}
                                                                value={option.id.to_string()}
                                                                selected={false}
                                                            />
                                                        }
                                                    }) }
                                                </FormControl>
                                            }
                                        } else {
                                            html! {}
                                        } }
                                        { if !filters.is_empty() {
                                            html! {
                                                <Button
                                                    class={classes.filter_remove_button.to_string()}
                                                    onclick={on_clear_filters}
                                                    style={Color::Secondary}
                                                    size={ButtonSize::Small}
                                                    outline={true}
                                                >
                                                    { "Clear" }
                                                </Button>
                                            }
                                        } else {
                                            html! {}
                                        } }
                                    </div>
                                </div>
                            }
                        } else {
                            html! {}
                        } }
                    </>
                }
            } else {
                html! {}
            } }
            <table class={classes.table} style={table_style} role="table">
                <TableHeader
                    columns={columns.clone()}
                    {sort_column}
                    {sort_order}
                    {on_sort_column}
                    classes={classes.clone()}
                    has_row_end={row_end_component.is_some()}
                />
                <TableBody
                    columns={columns.clone()}
                    rows={page_rows.clone()}
                    loading={loading}
                    classes={classes.clone()}
                    texts={texts.clone()}
                    row_end_component={row_end_component.clone()}
                />
            </table>
            { if *paginate {
                    html! {
                        <div style={pagination_style}>
                            <PaginationControls page={current_page} {total_pages} on_page_change={on_page_change} classes={classes.clone()} texts={texts.clone()}/>
                        </div>
                    }
                } else {
                    html! {}
                } }
        </div>
    }
}
