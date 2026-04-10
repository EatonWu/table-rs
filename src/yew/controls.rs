use crate::yew::types::PaginationControlsProps;
use yew::prelude::*;

#[function_component(PaginationControls)]
pub fn pagination_controls(props: &PaginationControlsProps) -> Html {
    let PaginationControlsProps {
        page,
        total_pages,
        on_prev,
        on_next,
        classes,
        texts,
    } = props;
    let page_val = *page;

    let on_prev = {
        let on_prev = on_prev.clone();
        Callback::from(move |_| on_prev.emit(()))
    };

    let on_next = {
        let on_next = on_next.clone();
        Callback::from(move |_| on_next.emit(()))
    };
    let has_pages = *total_pages > 0;
    let current_page_display = if has_pages { page_val + 1 } else { 0 };

    html! {
        <div class={classes.pagination}>
            <button class={classes.pagination_button} onclick={on_prev} disabled={!has_pages || page_val == 0}>
                { texts.previous_button }
            </button>
            <span>
                { texts.page_indicator.replace("{current}", &current_page_display.to_string()).replace("{total}", &total_pages.to_string()) }
            </span>
            <button
                class={classes.pagination_button}
                onclick={on_next}
                disabled={!has_pages || page_val + 1 >= *total_pages}
            >
                { texts.next_button }
            </button>
        </div>
    }
}
