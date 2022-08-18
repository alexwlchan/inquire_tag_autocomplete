use std::collections::HashSet;

use inquire::{
    autocompletion::{AutoComplete, Completion},
    error::CustomUserError,
    Text,
};

// The list of tags is fetched as a function so it can be created
// at runtime, e.g. by querying a database -- it doesn't have to be
// compiled into the binary.
fn get_tags<'a>() -> Vec<&'a str> {
    vec!["adventure", "action", "mystery", "romance", "scifi"]
}

#[derive(Clone)]
pub struct FilePathCompleter<'a> {
    tags: Vec<&'a str>,
    suggestions: Vec<&'a str>,
    prefix: String,
}

impl<'a> FilePathCompleter<'a> {
    pub fn new(tags: Vec<&'a str>) -> Self {
        Self {
            tags: tags.clone(),
            suggestions: tags,
            prefix: "".to_owned(),
        }
    }
}

impl<'a> AutoComplete for FilePathCompleter<'a> {
    fn update_input(&mut self, input: &str) -> Result<(), CustomUserError> {
        if input.is_empty() {
            self.prefix.clear();
            self.suggestions = self.tags.to_vec();
            return Ok(());
        }

        // What tags have already been used?  Tags can only be selected
        // once, so we don't want to suggest a tag already in the input.
        let input_tags = input.split_whitespace();

        let used_tags: HashSet<&str> = HashSet::from_iter(input_tags);

        // What's the latest tag the user is typing?  i.e. what are we trying
        // to autocomplete on this tag.
        let last_char_is_space = input.chars().last().unwrap().is_whitespace();
        let this_tag = if last_char_is_space {
            None
        } else {
            input.split_whitespace().last()
        };

        self.prefix = if last_char_is_space {
            input.to_string()
        } else if let Some(tag) = this_tag {
            input[..(input.len() - tag.len())].to_string()
        } else {
            unreachable!();
            // input.to_string()
        };

        self.suggestions = self
            .tags
            .iter()
            .filter(|s| !used_tags.contains(*s))
            // Note: this will filter to all the matching tags if the user
            // is midway through matching a tag (e.g. "adventure ac" -> "action"),
            // but will also display *all* the tags on the initial prompt.
            //
            // If there are lots of tags, that might be unwieldy.
            .filter(|s| match this_tag {
                None => true,
                Some(t) => s.contains(&t),
            })
            .take(15)
            .copied()
            .collect();

        Ok(())
    }

    fn get_suggestions(&self) -> Result<Vec<String>, CustomUserError> {
        Ok(self.suggestions.iter().map(|s| s.to_string()).collect())
    }

    fn get_completion(
        &self,
        selected_suggestion: Option<(usize, &str)>,
    ) -> Result<inquire::autocompletion::Completion, CustomUserError> {
        let completion = match selected_suggestion {
            None => self.suggestions.first().copied(),
            Some(suggestion) => Some(suggestion.1),
        };

        let completion = match completion {
            Some(c) => match self.prefix.is_empty() {
                true => Completion::Replace(format!("{} ", c)),
                false => {
                    let separator = if self.prefix.chars().last().unwrap().is_whitespace() {
                        ""
                    } else {
                        " "
                    };
                    Completion::Replace(format!("{}{}{} ", self.prefix, separator, c))
                }
            },
            None => Completion::None,
        };

        Ok(completion)
    }
}

#[cfg(test)]
mod tests {
    use inquire::autocompletion::AutoComplete;

    use crate::{get_tags, FilePathCompleter};

    #[test]
    fn it_offers_all_options_initially() {
        let mut ac = FilePathCompleter::new(get_tags());
        ac.update_input("").unwrap();

        let suggestions = ac.get_suggestions().unwrap();

        assert_eq!(
            suggestions,
            vec!["adventure", "action", "mystery", "romance", "scifi"]
        );
    }

    #[test]
    fn it_offers_all_options_with_a_matching_substring() {
        let mut ac = FilePathCompleter::new(get_tags());
        ac.update_input("s").unwrap();

        let suggestions = ac.get_suggestions().unwrap();

        assert_eq!(suggestions, vec!["mystery", "scifi"]);
    }

    #[test]
    fn it_only_offers_unused_options() {
        let mut ac = FilePathCompleter::new(get_tags());
        ac.update_input("scifi s").unwrap();

        let suggestions = ac.get_suggestions().unwrap();

        assert_eq!(suggestions, vec!["mystery"]);
    }

    #[test]
    fn it_offers_no_options_if_no_matches() {
        let mut ac = FilePathCompleter::new(get_tags());
        ac.update_input("scifi z").unwrap();
        let suggestions = ac.get_suggestions().unwrap();

        assert_eq!(suggestions.len(), 0);

        let mut ac = FilePathCompleter::new(get_tags());
        ac.update_input("z").unwrap();
        let suggestions = ac.get_suggestions().unwrap();

        assert_eq!(suggestions.len(), 0);
    }
}

fn main() {
    let answer = Text::new("What are the tags?")
        .with_auto_completion(FilePathCompleter::new(get_tags()))
        .prompt()
        .unwrap();

    let tags: Vec<&str> = answer.split_whitespace().collect();

    println!("The tags are {:?}", tags);
}
