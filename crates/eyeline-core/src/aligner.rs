use crate::script::{normalize, Script};

const LOOKBACK: usize = 6;
const LOOKAHEAD: usize = 40;
const MAX_TAIL: usize = 8;
const MIN_SINGLE_LEN: usize = 4;

pub struct Aligner {
    norms: Vec<String>,
    word_index: Vec<usize>,
    total_words: usize,
    cursor: usize,
}

impl Aligner {
    pub fn new(script: &Script) -> Self {
        let mut norms = Vec::new();
        let mut word_index = Vec::new();
        for (i, w) in script.words.iter().enumerate() {
            if w.spoken {
                norms.push(w.norm.clone());
                word_index.push(i);
            }
        }
        Self {
            norms,
            word_index,
            total_words: script.words.len(),
            cursor: 0,
        }
    }

    pub fn cursor_word(&self) -> usize {
        self.word_index
            .get(self.cursor)
            .copied()
            .unwrap_or(self.total_words)
    }

    pub fn finished(&self) -> bool {
        self.cursor >= self.norms.len()
    }

    pub fn seek_word(&mut self, word: usize) {
        self.cursor = self.word_index.partition_point(|&i| i < word);
    }

    pub fn feed(&mut self, transcript: &str) -> Option<usize> {
        let heard: Vec<String> = transcript
            .split_whitespace()
            .map(normalize)
            .filter(|t| !t.is_empty())
            .collect();
        let tail = &heard[heard.len().saturating_sub(MAX_TAIL)..];
        if tail.is_empty() || self.norms.is_empty() {
            return None;
        }

        let lo = self.cursor.saturating_sub(LOOKBACK);
        let hi = (self.cursor + LOOKAHEAD).min(self.norms.len());

        for k in (1..=tail.len()).rev() {
            let needle = &tail[tail.len() - k..];
            let mut best: Option<(usize, usize)> = None;

            for landing in lo.max(k)..=hi {
                let p = landing - k;
                if !self.run_matches(needle, p) {
                    continue;
                }
                let ahead = landing as isize - self.cursor as isize;
                if !self.accepts(k, ahead, &needle[0]) {
                    continue;
                }
                let distance = ahead.unsigned_abs() * 2 + usize::from(ahead < 0);
                if best.is_none_or(|(d, _)| distance < d) {
                    best = Some((distance, landing));
                }
            }

            if let Some((_, landing)) = best {
                if landing == self.cursor {
                    return None;
                }
                self.cursor = landing;
                return Some(self.cursor_word());
            }
        }
        None
    }

    fn run_matches(&self, needle: &[String], at: usize) -> bool {
        needle
            .iter()
            .zip(&self.norms[at..at + needle.len()])
            .all(|(a, b)| same_word(a, b))
    }

    fn accepts(&self, k: usize, ahead: isize, first: &str) -> bool {
        match k {
            1 => ahead == 1 && first.chars().count() >= MIN_SINGLE_LEN,
            2 => (0..=4).contains(&ahead),
            _ => true,
        }
    }
}

fn same_word(a: &str, b: &str) -> bool {
    a == b || (a.chars().count() >= 5 && b.chars().count() >= 5 && within_one_edit(a, b))
}

fn within_one_edit(a: &str, b: &str) -> bool {
    let a: Vec<char> = a.chars().collect();
    let b: Vec<char> = b.chars().collect();
    if a.len().abs_diff(b.len()) > 1 {
        return false;
    }
    let (short, long) = if a.len() <= b.len() { (&a, &b) } else { (&b, &a) };
    let mut i = 0;
    while i < short.len() && short[i] == long[i] {
        i += 1;
    }
    if i == short.len() {
        return true;
    }
    if short.len() == long.len() {
        let substituted = short[i + 1..] == long[i + 1..];
        let swapped = i + 1 < short.len()
            && short[i] == long[i + 1]
            && short[i + 1] == long[i]
            && short[i + 2..] == long[i + 2..];
        substituted || swapped
    } else {
        short[i..] == long[i + 1..]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::script::parse_script;

    fn aligner(text: &str) -> Aligner {
        Aligner::new(&parse_script(text))
    }

    const SCRIPT: &str = "Welcome to the show everyone. Today we are talking about teleprompters and how \
        they keep your eyes on the lens. First, the notch. Second, the pricing model.";

    #[test]
    fn follows_exact_speech() {
        let mut a = aligner(SCRIPT);
        assert_eq!(a.feed("welcome to the show"), Some(4));
        assert_eq!(a.feed("welcome to the show everyone today we"), Some(7));
    }

    #[test]
    fn ignores_repeated_hypothesis() {
        let mut a = aligner(SCRIPT);
        a.feed("welcome to the show");
        assert_eq!(a.feed("welcome to the show"), None);
    }

    #[test]
    fn ad_lib_does_not_move_cursor() {
        let mut a = aligner(SCRIPT);
        a.feed("welcome to the show");
        let before = a.cursor_word();
        assert_eq!(a.feed("welcome to the show uh so basically yeah anyway"), None);
        assert_eq!(a.cursor_word(), before);
    }

    #[test]
    fn follows_after_ad_lib_when_speaker_returns() {
        let mut a = aligner(SCRIPT);
        a.feed("welcome to the show");
        a.feed("welcome to the show you know what");
        assert!(a.feed("you know what everyone today we are").is_some());
        assert_eq!(a.cursor_word(), 8);
    }

    #[test]
    fn skips_ahead_when_speaker_drops_a_sentence() {
        let mut a = aligner(SCRIPT);
        a.feed("welcome to the show everyone");
        let moved = a.feed("first the notch second the pricing");
        assert!(moved.is_some());
        assert!(a.cursor_word() > 15);
    }

    #[test]
    fn jumping_beyond_lookahead_needs_a_cue() {
        let long = format!("{} {}", "alpha beta gamma delta ".repeat(30), "needle haystack position marker");
        let mut a = aligner(&long);
        assert_eq!(a.feed("needle haystack position marker"), None);
        a.seek_word(120);
        assert_eq!(a.feed("needle haystack position marker"), Some(124));
    }

    #[test]
    fn repeating_a_line_moves_back() {
        let mut a = aligner(SCRIPT);
        a.feed("welcome to the show everyone today we are talking");
        let before = a.cursor_word();
        a.feed("today we are talking");
        assert_eq!(a.cursor_word(), before);
        a.feed("welcome to the show");
        assert_eq!(a.cursor_word(), 4);
    }

    #[test]
    fn tolerates_one_character_recognition_errors() {
        let mut a = aligner("We will receive the shipment tomorrow morning");
        assert!(a.feed("we will recieve the shipment").is_some());
        assert_eq!(a.cursor_word(), 5);
    }

    #[test]
    fn skips_stage_directions() {
        let mut a = aligner("Start here [pause] and then continue");
        assert_eq!(a.feed("start here and then"), Some(5));
    }

    #[test]
    fn reports_finished_at_the_end() {
        let mut a = aligner("The end is near");
        a.feed("the end is near");
        assert!(a.finished());
    }

    #[test]
    fn empty_input_is_safe() {
        let mut a = aligner("");
        assert_eq!(a.feed("anything at all"), None);
        assert!(a.finished());
    }

    #[test]
    fn single_short_word_never_moves_cursor() {
        let mut a = aligner("the cat sat on the mat today");
        assert_eq!(a.feed("the"), None);
    }
}
