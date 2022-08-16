use std::collections::HashSet;

use inquire::{error::CustomUserError, Text};

// The list of tags is fetched as a function so it can be created
// at runtime, e.g. by querying a database -- it doesn't have to be
// compiled into the binary.
fn get_tags<'a>() -> Vec<&'a str> {
    vec!["adventure", "action", "mystery", "romance", "scifi"]
}

fn suggester(val: &str) -> Result<Vec<String>, CustomUserError> {
    let tags = HashSet::from_iter(get_tags());

    // What tags have already been used?  Tags can only be selected
    // once, so we don't want to suggest a tag already in the input.
    let used_tags: HashSet<&str> = HashSet::from_iter(val.split_whitespace());
    let mut available_tags = tags.difference(&used_tags).cloned().collect::<Vec<&str>>();
    available_tags.sort();

    // What's the latest tag the user is typing?  i.e. what are we trying
    // to autocomplete on this tag.
    let this_tag = val.split_whitespace().last();

    let prefix = match this_tag {
        None => val,
        Some(t) => &val[..(val.len() - t.len())],
    };

    Ok(available_tags
        .iter()
        // Note: this will filter to all the matching tags if the user
        // is midway through matching a tag (e.g. "adventure ac" -> "action"),
        // but will also display *all* the tags on the initial prompt.
        //
        // If there are lots of tags, that might be unwieldy.
        .filter(|s| match this_tag {
            None => true,
            Some(t) => s.contains(&t),
        })
        // Note: the prefix may be empty if the user hasn't typed
        // anything yet.
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
