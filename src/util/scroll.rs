pub fn compute_scroll_offset(
    current_offset: usize,
    selected: usize,
    total_items: usize,
    viewport_height: usize,
) -> usize {
    if viewport_height == 0 || total_items == 0 {
        return 0;
    }

    let max_offset = total_items.saturating_sub(viewport_height);
    let mut offset = current_offset.min(max_offset);
    let scrolloff = (viewport_height / 4).max(1);

    if selected < offset + scrolloff {
        offset = selected.saturating_sub(scrolloff);
    } else if selected + scrolloff + 1 > offset + viewport_height {
        offset = (selected + scrolloff + 1).saturating_sub(viewport_height);
    }

    offset.min(max_offset)
}
