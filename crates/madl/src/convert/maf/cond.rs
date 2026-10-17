//! Conditions on the ways between actions, as Power Fx text. A way is a list of conditions that
//! all hold; a target reached by several ways is reached when any of them holds.

/// `Not(c)`.
pub(super) fn not(c: &str) -> String {
    format!("Not({c})")
}

/// The ways, without ways that another way makes unnecessary: `(a) Or (a And b)` is `a`, and
/// `(a And b) Or (a And Not(b))` is `a`. An empty way means the target is reached always.
pub(super) fn simplify(mut ways: Vec<Vec<String>>) -> Vec<Vec<String>> {
    loop {
        ways = absorb(ways);
        let Some((i, j, keep)) = complement(&ways) else { return ways };
        ways[i] = keep;
        ways.remove(j);
    }
}

/// Drops each way that holds every condition of another way, and repeats.
fn absorb(ways: Vec<Vec<String>>) -> Vec<Vec<String>> {
    let within = |v: &[String], w: &[String]| v.iter().all(|c| w.contains(c));
    let mut out: Vec<Vec<String>> = Vec::new();
    for w in ways {
        if out.iter().any(|v| within(v, &w)) {
            continue;
        }
        out.retain(|v| !within(&w, v));
        out.push(w);
    }
    out
}

/// Two ways alike but for one condition, held in one and denied in the other, and what they
/// share.
fn complement(ways: &[Vec<String>]) -> Option<(usize, usize, Vec<String>)> {
    for i in 0..ways.len() {
        for j in i + 1..ways.len() {
            let (v, w) = (&ways[i], &ways[j]);
            let only_v: Vec<&String> = v.iter().filter(|c| !w.contains(c)).collect();
            let only_w: Vec<&String> = w.iter().filter(|c| !v.contains(c)).collect();
            if let ([a], [b]) = (only_v.as_slice(), only_w.as_slice())
                && (**a == not(b) || **b == not(a))
            {
                return Some((i, j, v.iter().filter(|c| c != a).cloned().collect()));
            }
        }
    }
    None
}

/// `a And (b < 4)`, or `(a) Or (b)`; `None` when some way has no condition.
pub(super) fn render(ways: &[Vec<String>]) -> Option<String> {
    if ways.iter().any(Vec::is_empty) {
        return None;
    }
    let all = |w: &Vec<String>| match w.as_slice() {
        [one] => one.clone(),
        _ => w.iter().map(|c| group(c)).collect::<Vec<_>>().join(" And "),
    };
    Some(match ways {
        [one] => all(one),
        _ => ways.iter().map(|w| format!("({})", all(w))).collect::<Vec<_>>().join(" Or "),
    })
}

/// `c` in parentheses when it has an operator outside parentheses and quotes.
fn group(c: &str) -> String {
    let (mut depth, mut quoted) = (0i32, false);
    let open = c.chars().any(|ch| {
        match ch {
            '"' => quoted = !quoted,
            '(' if !quoted => depth += 1,
            ')' if !quoted => depth -= 1,
            _ => {}
        }
        depth == 0 && !quoted && (ch.is_whitespace() || "<>=!&|+-*/".contains(ch))
    });
    if open { format!("({c})") } else { c.to_string() }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ways(list: &[&[&str]]) -> Vec<Vec<String>> {
        list.iter().map(|w| w.iter().map(|c| c.to_string()).collect()).collect()
    }

    #[test]
    fn branches_that_join_again_leave_no_condition() {
        let joined = simplify(ways(&[&["s", "l"], &["s", "Not(l)", "p"], &["s", "Not(l)", "Not(p)"]]));
        assert_eq!(joined, ways(&[&["s"]]));
        assert_eq!(render(&simplify(ways(&[&["a"], &["a", "b"]]))).as_deref(), Some("a"));
        assert_eq!(render(&ways(&[&[]])), None);
    }

    #[test]
    fn conditions_with_operators_are_grouped() {
        let text = render(&ways(&[&["Not(x)", "a || b"], &["T < 4"]])).unwrap();
        assert_eq!(text, "(Not(x) And (a || b)) Or (T < 4)");
        assert_eq!(render(&ways(&[&["Find(\"a b\", x)"]])).unwrap(), "Find(\"a b\", x)");
    }
}
