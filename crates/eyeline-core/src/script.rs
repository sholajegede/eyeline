use serde::Serialize;

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct Word {
    pub text: String,
    pub norm: String,
    pub spoken: bool,
    pub cue: usize,
    pub paragraph_start: bool,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct Cue {
    pub title: String,
    pub start: usize,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq, Default)]
pub struct Script {
    pub words: Vec<Word>,
    pub cues: Vec<Cue>,
}

pub fn normalize(token: &str) -> String {
    token
        .chars()
        .filter(|c| c.is_alphanumeric())
        .flat_map(|c| c.to_lowercase())
        .collect()
}

fn heading_title(line: &str) -> Option<&str> {
    let trimmed = line.trim_start();
    let hashes = trimmed.chars().take_while(|c| *c == '#').count();
    if hashes == 0 || hashes > 3 {
        return None;
    }
    let rest = &trimmed[hashes..];
    rest.strip_prefix(' ').map(str::trim)
}

fn is_divider(line: &str) -> bool {
    let t = line.trim();
    t.len() >= 3 && t.chars().all(|c| c == '-')
}

pub fn parse_script(text: &str) -> Script {
    let mut script = Script::default();
    let mut paragraph_start = true;
    let mut in_direction = false;
    let mut pending_title: Option<String> = None;

    for line in text.lines() {
        if let Some(title) = heading_title(line) {
            pending_title = Some(if title.is_empty() {
                format!("Section {}", script.cues.len() + 1)
            } else {
                title.to_string()
            });
            paragraph_start = true;
            continue;
        }
        if is_divider(line) {
            paragraph_start = true;
            continue;
        }
        if line.trim().is_empty() {
            paragraph_start = true;
            continue;
        }

        for token in line.split_whitespace() {
            if script.cues.is_empty() || pending_title.is_some() {
                let title = pending_title.take().unwrap_or_else(|| "Script".to_string());
                script.cues.push(Cue {
                    title,
                    start: script.words.len(),
                });
            }

            let opens = token.starts_with('[');
            let closes = token.ends_with(']');
            if opens {
                in_direction = true;
            }
            let spoken = !in_direction;
            if closes {
                in_direction = false;
            }

            let norm = normalize(token);
            script.words.push(Word {
                text: token.to_string(),
                spoken: spoken && !norm.is_empty(),
                norm,
                cue: script.cues.len() - 1,
                paragraph_start,
            });
            paragraph_start = false;
        }
    }
    script
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn splits_cues_on_headings() {
        let s = parse_script("## Intro\nHello there.\n\n## Pricing\nIt costs money.");
        assert_eq!(s.cues.len(), 2);
        assert_eq!(s.cues[0].title, "Intro");
        assert_eq!(s.cues[1].title, "Pricing");
        assert_eq!(s.words[s.cues[1].start].text, "It");
    }

    #[test]
    fn text_before_first_heading_gets_default_cue() {
        let s = parse_script("Hello\n## Next\nWorld");
        assert_eq!(s.cues[0].title, "Script");
        assert_eq!(s.cues[1].title, "Next");
    }

    #[test]
    fn stage_directions_are_not_spoken() {
        let s = parse_script("Say this [pause for effect] then that");
        let spoken: Vec<_> = s.words.iter().filter(|w| w.spoken).map(|w| w.norm.as_str()).collect();
        assert_eq!(spoken, ["say", "this", "then", "that"]);
        assert_eq!(s.words.len(), 7);
    }

    #[test]
    fn normalizes_punctuation_and_case() {
        let s = parse_script("Don't STOP, believing!");
        let norms: Vec<_> = s.words.iter().map(|w| w.norm.as_str()).collect();
        assert_eq!(norms, ["dont", "stop", "believing"]);
    }

    #[test]
    fn marks_paragraph_starts() {
        let s = parse_script("One two\n\nThree");
        assert!(s.words[0].paragraph_start);
        assert!(!s.words[1].paragraph_start);
        assert!(s.words[2].paragraph_start);
    }

    #[test]
    fn empty_input_is_empty_script() {
        let s = parse_script("   \n\n");
        assert!(s.words.is_empty());
        assert!(s.cues.is_empty());
    }
}
