use crate::yew::types::PaginationControlsProps;
use yew::prelude::*;

#[function_component(PaginationControls)]
pub fn pagination_controls(props: &PaginationControlsProps) -> Html {
    let PaginationControlsProps {
        page,
        total_pages,
        on_prev,
        on_next,
        on_jump,
        classes,
        texts,
    } = props;
    let page_val = *page;
    let page_input = use_state(|| {
        let has_pages = *total_pages > 0;
        if has_pages {
            (page_val + 1).to_string()
        } else {
            "0".to_string()
        }
    });

    {
        let page_input = page_input.clone();
        let page_val = page_val;
        let total_pages = *total_pages;
        use_effect_with((page_val, total_pages), move |(p, t)| {
            if *t > 0 {
                page_input.set((p + 1).to_string());
            } else {
                page_input.set("0".to_string());
            }
            || ()
        });
    }

    let on_prev = {
        let on_prev = on_prev.clone();
        Callback::from(move |_| on_prev.emit(()))
    };

    let on_next = {
        let on_next = on_next.clone();
        Callback::from(move |_| on_next.emit(()))
    };
    let on_page_input = {
        let page_input = page_input.clone();
        Callback::from(move |e: InputEvent| {
            let input: web_sys::HtmlInputElement = e.target_unchecked_into();
            page_input.set(input.value());
        })
    };
    let submit_jump = {
        let on_jump = on_jump.clone();
        let page_input = page_input.clone();
        let total_pages = *total_pages;
        Callback::from(move |_| {
            if total_pages == 0 {
                return;
            }
            let parsed = (*page_input).trim().parse::<usize>().ok();
            if let Some(page_number) = parsed {
                if page_number > 0 {
                    on_jump.emit(page_number - 1);
                }
            }
        })
    };
    let on_page_keydown = {
        let submit_jump = submit_jump.clone();
        Callback::from(move |e: KeyboardEvent| {
            if e.key() == "Enter" {
                e.prevent_default();
                submit_jump.emit(());
            }
        })
    };
    let has_pages = *total_pages > 0;

    html! {
        <div class={classes.pagination}>
            <button class={classes.pagination_button} onclick={on_prev} disabled={!has_pages || page_val == 0}>
                { texts.previous_button }
            </button>
            <span class="d-inline-flex align-items-center" style="gap: 0.35rem;">
                {"Page"}
                <input
                    type="number"
                    min="1"
                    max={if has_pages { total_pages.to_string() } else { "1".to_string() }}
                    value={(*page_input).clone()}
                    oninput={on_page_input}
                    onkeydown={on_page_keydown}
                    aria-label="Page number"
                    style="width: 4.5rem;"
                    class="form-control form-control-sm d-inline-block"
                    disabled={!has_pages}
                />
                {"of"}
                {total_pages.to_string()}
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
