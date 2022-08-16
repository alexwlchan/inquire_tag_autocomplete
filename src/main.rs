use std::collections::HashSet;

use inquire::{error::CustomUserError, Text};

fn get_tags<'a>() -> Vec<&'a str> {
    vec!["adventure", "fiction", "mystery", "romance", "scifi"]
}

fn suggester(val: &str) -> Result<Vec<String>, CustomUserError> {
    let tags = HashSet::from_iter(get_tags());

    // What tags have already been used?
    let used_tags: HashSet<&str> = HashSet::from_iter(val.split_whitespace());

    // What tags are still available?
    let mut available_tags = tags.difference(&used_tags).cloned().collect::<Vec<&str>>();
    available_tags.sort();

    // What's the latest tag the user is typing?
    let latest_tag = val.split_whitespace().last();

    let prefix = match latest_tag {
        None => val,
        Some(t) => &val[..(val.len() - t.len())],
    };

    Ok(available_tags
        .iter()
        .filter(|s| match latest_tag {
            None => true,
            Some(t) => s.contains(&t),
        })
        .map(|s| {
            if prefix.is_empty() {
                format!("{} ", s)
            } else {
                format!("{} {} ", prefix.trim_end(), s)
            }
        })
        .collect())
}

fn completer(val: &str) -> Result<Option<String>, CustomUserError> {
    let suggestions = suggester(val)?;

    if suggestions.len() == 1 {
        Ok(Some(suggestions[0].clone()))
    } else {
        Ok(None)
    }
}

#[cfg(test)]
mod tests {
    use crate::suggester;

    #[test]
    fn it_offers_all_options_initially() {
        let result = suggester("");
        assert_eq!(
            result.unwrap(),
            vec!["adventure ", "fiction ", "mystery ", "romance ", "scifi "]
        );
    }

    #[test]
    fn it_offers_all_options_with_a_matching_substring() {
        let result = suggester("s");
        assert_eq!(result.unwrap(), vec!["mystery ", "scifi "]);
    }

    #[test]
    fn it_only_offers_unused_options() {
        let result = suggester("scifi s");
        assert_eq!(result.unwrap(), vec!["scifi mystery "]);
    }

    #[test]
    fn it_offers_no_options_if_no_matches() {
        let result = suggester("scifi z");
        assert_eq!(result.unwrap().len(), 0);

        let result = suggester("z");
        assert_eq!(result.unwrap().len(), 0);
    }
}

fn main() {
    let answer = Text::new("What are the tags?")
        .with_suggester(&suggester)
        .with_completer(&completer)
        .prompt()
        .unwrap();

    let tags: Vec<&str> = answer.split_whitespace().collect();

    println!("The tags are {:?}", tags);
}
