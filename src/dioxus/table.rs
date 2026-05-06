use dioxus::prelude::*;

use crate::dioxus::body::TableBody;
use crate::dioxus::controls::PaginationControls;
use crate::dioxus::header::TableHeader;
use crate::dioxus::types::{FilterType, SortOrder, TableProps};

#[derive(Clone, PartialEq)]
struct ColumnFilter {
    column_id: &'static str,
    operator: FilterOperator,
    value: String,
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
    }
}

/// A fully featured table component with sorting, pagination, and search functionality in Dioxus.
///
/// This component renders an interactive HTML `<table>` with customizable columns, data,
/// class names, and labels. It supports client-side sorting and search,
/// and pagination.
///
/// # Props
/// `TableProps` defines the configuration for this component:
/// - `data`: A `Vec<HashMap<&'static str, String>>` representing row data.
/// - `columns`: A `Vec<Column>` describing each column's ID, header text, and behavior.
/// - `page_size`: Number of rows to display per page (default: `10`).
/// - `loading`: When `true`, displays a loading indicator (default: `false`).
/// - `paginate`: Enables pagination controls (default: `false`).
/// - `search`: Enables a search input for client-side filtering (default: `false`).
/// - `texts`: Customizable text labels for UI strings (default: `TableTexts::default()`).
/// - `classes`: Customizable CSS class names for each table part (default: `TableClasses::default()`).
///
/// # Features
/// - **Search**: Filters rows client-side using a text input.
/// - **Sorting**: Clickable headers allow sorting columns ascending or descending.
/// - **Pagination**: Navigate between pages using prev/next buttons, with an indicator showing current page.
/// - **Custom Classes**: All elements are styled via `TableClasses` for full customization.
/// - **Text Overrides**: All UI strings (e.g., empty state, loading, buttons) can be customized using `TableTexts`.
///
/// # Returns
/// Returns a `Dioxus` `Element` that renders a complete table with the above features.
///
/// # Example
/// ```rust
/// use dioxus::prelude::*;
/// use maplit::hashmap;
/// use table_rs::dioxus::table::Table;
/// use table_rs::dioxus::types::Column;
///
///
/// fn App() -> Element {
///     let data = vec![
///         hashmap! { "name" => "ferris".to_string(), "email" => "ferris@opensass.org".to_string() },
///         hashmap! { "name" => "ferros".to_string(), "email" => "ferros@opensass.org".to_string() },
///     ];
///
///     let columns = vec![
///         Column { id: "name", header: "Name", sortable: true, ..Default::default() },
///         Column { id: "email", header: "Email", ..Default::default() },
///     ];
///
///     rsx! {
///         Table {
///             data: data,
///             columns: columns,
///             paginate: true,
///             search: true,
///         }
///     }
/// }
/// ```
///
/// # See Also
/// - [MDN `<table>` Element](https://developer.mozilla.org/en-US/docs/Web/HTML/Reference/Elements/table)
#[component]
pub fn Table(props: TableProps) -> Element {
    let TableProps {
        data,
        columns,
        page_size,
        loading,
        paginate,
        search,
        filterable_columns,
        texts,
        classes,
        row_end_component,
    } = props;

    let mut page = use_signal(|| 0_usize);
    let mut sort_column = use_signal(|| None::<&'static str>);
    let mut sort_order = use_signal(SortOrder::default);
    let mut search_query = use_signal(String::new);
    let mut filters = use_signal(|| Vec::<ColumnFilter>::new());

    let filtered_rows = {
        let mut rows = data.clone();
        if !filterable_columns.is_empty() {
            rows.retain(|row| {
                filters().iter().all(|filter| {
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
        if !search_query().is_empty() {
            rows.retain(|row| {
                columns.iter().any(|col| {
                    row.get(col.id)
                        .map(|v| v.to_lowercase().contains(&search_query().to_lowercase()))
                        .unwrap_or(false)
                })
            });
        }

        if let Some(col_id) = sort_column() {
            if let Some(col) = columns.iter().find(|c| c.id == col_id) {
                rows.sort_by(|a, b| {
                    let val = "".to_string();
                    let a_val = a.get(col.id).unwrap_or(&val);
                    let b_val = b.get(col.id).unwrap_or(&val);
                    match sort_order() {
                        SortOrder::Asc => a_val.cmp(b_val),
                        SortOrder::Desc => b_val.cmp(a_val),
                    }
                });
            }
        }

        rows
    };

    let total_pages = (filtered_rows.len() as f64 / page_size as f64).ceil() as usize;
    let start = page() * page_size;
    let end = ((page() + 1) * page_size).min(filtered_rows.len());
    let page_rows = &filtered_rows[start..end];

    let on_sort_column = move |id: &'static str| {
        if Some(id) == sort_column() {
            sort_order.set(match sort_order() {
                SortOrder::Asc => SortOrder::Desc,
                SortOrder::Desc => SortOrder::Asc,
            });
        } else {
            sort_column.set(Some(id));
            sort_order.set(SortOrder::Asc);
        }
    };

    let available_filter_columns: Vec<(&'static str, FilterType)> = columns
        .iter()
        .filter_map(|col| {
            filterable_columns
                .get(col.id)
                .map(|filter_type| (col.id, *filter_type))
        })
        .collect();

    let on_add_filter = {
        let available_filter_columns = available_filter_columns.clone();
        move |_| {
            if let Some((column_id, filter_type)) = available_filter_columns.first() {
                let mut next = filters();
                next.push(ColumnFilter {
                    column_id: *column_id,
                    operator: default_operator(filter_type),
                    value: String::new(),
                });
                filters.set(next);
                page.set(0);
            }
        }
    };

    let pagination_controls = if paginate {
        rsx! {
            PaginationControls {
                page: page,
                total_pages: total_pages,
                classes: classes.clone(),
                texts: texts.clone(),
            }
        }
    } else {
        rsx! {}
    };

    rsx! {
        div {
            class: "{classes.container}",
            if search || !filterable_columns.is_empty() {
                div {
                    style: "display: flex; gap: 8px; align-items: center;",
                    if search {
                        input {
                            class: "{classes.search_input}",
                            r#type: "text",
                            value: "{search_query()}",
                            placeholder: "{texts.search_placeholder}",
                            oninput: move |e| {
                                let val = e.value();
                                search_query.set(val.clone());
                                page.set(0);
                            }
                        }
                    }
                    if !filterable_columns.is_empty() {
                        button {
                            class: "{classes.filter_button}",
                            onclick: on_add_filter,
                            "+ Filter"
                        }
                    }
                    if !filterable_columns.is_empty() && !filters().is_empty() {
                        button {
                            class: "{classes.filter_remove_button}",
                            onclick: move |_| {
                                filters.set(Vec::new());
                                page.set(0);
                            },
                            "Clear"
                        }
                    }
                }
            }
            if !filterable_columns.is_empty() && !filters().is_empty() {
                div {
                    class: "{classes.filter_panel}",
                    for (index, filter) in filters().iter().enumerate() {
                        let filter_type = filterable_columns.get(filter.column_id);
                        let operator_value = filter.operator;
                        let input_type = match filter_type {
                            Some(FilterType::Number) => "number",
                            _ => "text",
                        };
                        div {
                            class: "{classes.filter_row}",
                            select {
                                class: "{classes.filter_select}",
                                onchange: move |e| {
                                    let value = e.value();
                                    let mut next = filters();
                                    if let Some(filter_entry) = next.get_mut(index) {
                                        if let Some((column_id, _)) = available_filter_columns
                                            .iter()
                                            .find(|(column_id, _)| *column_id == value.as_str())
                                        {
                                            filter_entry.column_id = *column_id;
                                            if let Some((_, filter_type)) = available_filter_columns
                                                .iter()
                                                .find(|(column_id, _)| *column_id == filter_entry.column_id)
                                            {
                                                filter_entry.operator = default_operator(filter_type);
                                            }
                                            filter_entry.value.clear();
                                        }
                                    }
                                    filters.set(next);
                                    page.set(0);
                                },
                                for (column_id, _) in available_filter_columns.iter() {
                                    option {
                                        value: "{column_id}",
                                        selected: *column_id == filter.column_id,
                                        "{column_id}"
                                    }
                                }
                            }
                            select {
                                class: "{classes.filter_operator}",
                                onchange: move |e| {
                                    let value = e.value();
                                    let mut next = filters();
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
                                    filters.set(next);
                                    page.set(0);
                                },
                                match filter_type {
                                    Some(FilterType::String) => rsx! {
                                        option { value: "contains", selected: operator_value == FilterOperator::Contains, "contains" }
                                        option { value: "equals", selected: operator_value == FilterOperator::Equals, "equals" }
                                        option { value: "not_equals", selected: operator_value == FilterOperator::NotEqual, "not equals" }
                                        option { value: "starts_with", selected: operator_value == FilterOperator::StartsWith, "starts with" }
                                        option { value: "ends_with", selected: operator_value == FilterOperator::EndsWith, "ends with" }
                                        option { value: "is_empty", selected: operator_value == FilterOperator::IsEmpty, "is empty" }
                                        option { value: "is_not_empty", selected: operator_value == FilterOperator::IsNotEmpty, "is not empty" }
                                    },
                                    Some(FilterType::Number) => rsx! {
                                        option { value: "equals", selected: operator_value == FilterOperator::Equals, "=" }
                                        option { value: "not_equals", selected: operator_value == FilterOperator::NotEqual, "!=" }
                                        option { value: "gt", selected: operator_value == FilterOperator::GreaterThan, ">" }
                                        option { value: "gte", selected: operator_value == FilterOperator::GreaterThanOrEqual, ">=" }
                                        option { value: "lt", selected: operator_value == FilterOperator::LessThan, "<" }
                                        option { value: "lte", selected: operator_value == FilterOperator::LessThanOrEqual, "<=" }
                                        option { value: "is_empty", selected: operator_value == FilterOperator::IsEmpty, "is empty" }
                                        option { value: "is_not_empty", selected: operator_value == FilterOperator::IsNotEmpty, "is not empty" }
                                    },
                                    Some(FilterType::Bool) => rsx! {
                                        option { value: "equals", selected: operator_value == FilterOperator::Equals, "is" }
                                        option { value: "not_equals", selected: operator_value == FilterOperator::NotEqual, "is not" }
                                        option { value: "is_empty", selected: operator_value == FilterOperator::IsEmpty, "is empty" }
                                        option { value: "is_not_empty", selected: operator_value == FilterOperator::IsNotEmpty, "is not empty" }
                                    },
                                    None => rsx! {},
                                }
                            }
                            if matches!(filter_type, Some(FilterType::Bool)) {
                                select {
                                    class: "{classes.filter_input}",
                                    value: "{filter.value}",
                                    disabled: matches!(operator_value, FilterOperator::IsEmpty | FilterOperator::IsNotEmpty),
                                    onchange: move |e| {
                                        let mut next = filters();
                                        if let Some(filter_entry) = next.get_mut(index) {
                                            filter_entry.value = e.value();
                                        }
                                        filters.set(next);
                                        page.set(0);
                                    },
                                    option { value: "", "Any" }
                                    option { value: "true", "true" }
                                    option { value: "false", "false" }
                                }
                            } else {
                                input {
                                    class: "{classes.filter_input}",
                                    r#type: "{input_type}",
                                    value: "{filter.value}",
                                    placeholder: "Filter...",
                                    disabled: matches!(operator_value, FilterOperator::IsEmpty | FilterOperator::IsNotEmpty),
                                    oninput: move |e| {
                                        let mut next = filters();
                                        if let Some(filter_entry) = next.get_mut(index) {
                                            filter_entry.value = e.value();
                                        }
                                        filters.set(next);
                                        page.set(0);
                                    }
                                }
                            }
                            button {
                                class: "{classes.filter_remove_button}",
                                onclick: move |_| {
                                    let mut next = filters();
                                    if index < next.len() {
                                        next.remove(index);
                                    }
                                    filters.set(next);
                                    page.set(0);
                                },
                                "Remove"
                            }
                        }
                    }
                }
            }
            table {
                class: "{classes.table}",
                TableHeader {
                    columns: columns.clone(),
                    sort_column: sort_column,
                    sort_order: sort_order,
                    on_sort_column: on_sort_column,
                    classes: classes.clone(),
                    has_row_end: row_end_component.is_some(),
                }
                TableBody {
                    columns: columns.clone(),
                    rows: page_rows.to_vec(),
                    loading: loading,
                    classes: classes.clone(),
                    texts: texts.clone(),
                    row_end_component: row_end_component.clone(),
                }
            }
            {pagination_controls}
        }
    }
}
