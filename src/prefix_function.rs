pub fn prefix_function(s: &String) -> Vec<usize> {
    let n = s.chars().size_hint().0;
    let mut pi: Vec<usize> = Vec::with_capacity(n);
    pi.push(0);
    for i in 1..n {
        let mut j = pi[i - 1];
        while s.chars().nth(j) != s.chars().nth(i) && j != 0 {
            j = pi[j - 1];
        }
        if j == 0 && s.chars().nth(j) != s.chars().nth(i) {
            pi.push(0)
        } else {
            pi.push(j + 1);
        }
    }

    pi
}

pub fn period(s: &String) -> usize {
    s.chars().size_hint().0 - prefix_function(s).last().unwrap_or(&0)
}
