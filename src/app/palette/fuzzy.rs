pub(super) fn fuzzy_score(candidate: &str, query: &str) -> Option<i64> {
    let needle = query.trim().to_lowercase();
    if needle.is_empty() {
        return Some(0);
    }

    let haystack: Vec<char> = candidate.to_lowercase().chars().collect();
    let mut idx = 0usize;
    let mut score = 0i64;
    let mut prev_match = None;

    for ch in needle.chars() {
        let mut found = None;
        while idx < haystack.len() {
            if haystack[idx] == ch {
                found = Some(idx);
                idx += 1;
                break;
            }
            idx += 1;
        }

        let pos = found?;
        score += 10;

        if let Some(prev) = prev_match {
            if pos == prev + 1 {
                score += 7;
            }
        } else if pos == 0 {
            score += 8;
        }

        if pos > 0 {
            let prev = haystack[pos - 1];
            if prev == '/' || prev == '_' || prev == '-' || prev == '.' {
                score += 6;
            }
        }

        prev_match = Some(pos);
    }

    Some(score - haystack.len() as i64)
}
