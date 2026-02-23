pub(super) fn subsequence_score(query: &str, candidate: &str) -> Option<(usize, usize)> {
    let mut q = query.chars();
    let mut current = q.next()?;
    let mut last_match = None;
    let mut gaps = 0usize;
    let mut matched = 0usize;

    for (idx, ch) in candidate.chars().enumerate() {
        if ch != current {
            continue;
        }
        if let Some(prev) = last_match {
            gaps += idx.saturating_sub(prev + 1);
        }
        matched += 1;
        last_match = Some(idx);
        if let Some(next) = q.next() {
            current = next;
        } else {
            let len_delta = candidate.chars().count().saturating_sub(matched);
            return Some((gaps, len_delta));
        }
    }

    None
}

pub(super) fn best_edit_distance(query: &str, candidate: &str) -> Option<u8> {
    candidate
        .split(|ch: char| !ch.is_alphanumeric() && ch != '_')
        .filter(|token| !token.is_empty())
        .filter_map(|token| {
            let prefix: String = token.chars().take(query.chars().count() + 1).collect();
            (!prefix.is_empty()).then_some(levenshtein(query, &prefix))
        })
        .min()
}

fn levenshtein(left: &str, right: &str) -> u8 {
    let left: Vec<char> = left.chars().collect();
    let right: Vec<char> = right.chars().collect();
    if left.is_empty() {
        return right.len().min(u8::MAX as usize) as u8;
    }
    if right.is_empty() {
        return left.len().min(u8::MAX as usize) as u8;
    }

    let mut prev: Vec<usize> = (0..=right.len()).collect();
    let mut curr = vec![0; right.len() + 1];
    for (i, lc) in left.iter().enumerate() {
        curr[0] = i + 1;
        for (j, rc) in right.iter().enumerate() {
            let cost = usize::from(lc != rc);
            curr[j + 1] = (prev[j + 1] + 1).min(curr[j] + 1).min(prev[j] + cost);
        }
        std::mem::swap(&mut prev, &mut curr);
    }

    prev[right.len()].min(u8::MAX as usize) as u8
}
