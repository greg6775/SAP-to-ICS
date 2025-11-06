pub fn fix_mojibake(s: &str) -> String {
    let latin1_bytes: Vec<u8> = s
        .chars()
        .filter_map(|c| {
            let code = c as u32;
            if code <= 0xFF {
                Some(code as u8)
            } else {
                None
            }
        })
        .collect();

    match String::from_utf8(latin1_bytes) {
        Ok(fixed) => {
            if fixed != s && seems_better(&fixed, s) {
                fixed
            } else {
                s.to_string()
            }
        }
        Err(_) => s.to_string(),
    }
}

fn seems_better(fixed: &str, original: &str) -> bool {
    let original_suspicious = count_suspicious(original);
    let fixed_suspicious = count_suspicious(fixed);

    fixed_suspicious < original_suspicious
}

fn count_suspicious(s: &str) -> usize {
    let mut count = 0;

    if s.contains("Ã¤") || s.contains("Ã¶") || s.contains("Ã¼") {
        count += 10;
    }
    if s.contains("Ã„") || s.contains("Ã–") || s.contains("Ãœ") {
        count += 10;
    }
    if s.contains("ÃŸ") {
        count += 10;
    }
    if s.contains("Ã©") || s.contains("Ã¨") || s.contains("Ã«") {
        count += 5;
    }

    count
}
