use tracing::debug;

pub fn check_haiku(text: &str) -> Option<Vec<String>> {
    //use hypher::hyphenate;

    //let en_us = Standard::from_embedded(Language::EnglishUS).unwrap();

    let words = text.split_whitespace();
    let parsed = words.clone().map(|word| syllables(word));

    let mut parsed = words.zip(parsed);

    //dbg!(parsed.clone().map(|s| s.collect::<Vec<_>>()).collect::<Vec<_>>());

    let mut total_syllables = 0;

    let mut line_syllables = 0;
    let mut line_text = Vec::new();
    while let Some((word, syllables)) = parsed.next() {
        line_syllables += syllables;

        if line_syllables > 5 {
            return None;
        }

        line_text.push(word);

        if line_syllables == 5 {
            break;
        }
    }

    total_syllables += line_syllables;

    let first_line = line_text.join(" ");
    debug!(%first_line);

    let mut line_syllables = 0;
    let mut line_text = Vec::new();
    while let Some((word, syllables)) = parsed.next() {
        line_syllables += syllables;

        if line_syllables > 7 {
            return None;
        }

        line_text.push(word);

        if line_syllables == 7 {
            break;
        }
    }

    total_syllables += line_syllables;

    let second_line = line_text.join(" ");

    let mut line_syllables = 0;
    let mut line_text = Vec::new();
    while let Some((word, syllables)) = parsed.next() {
        line_syllables += syllables;

        if line_syllables > 5 {
            return None;
        }

        line_text.push(word);

        if line_syllables == 5 {
            break;
        }
    }

    total_syllables += line_syllables;

    let third_line = line_text.join(" ");

    debug!(total_syllables);

    if total_syllables == 17 {
        Some(vec![first_line, second_line, third_line])
    } else {
        None
    }
}

fn syllables(word: &str) -> usize {
    // check exceptions
    match word {
        "tumblr" => return 2,
        "cringe" => return 1,
        "alien" => return 3,
        "aliens" => return 3,
        _ => (),
    }

    let mut last_letter: Option<char> = None;

    let mut vowels = Vec::new();

    for letter in word.chars() {
        let last = last_letter.map(|ch| ch.to_lowercase().next().unwrap());

        let last_is_vowel = matches!(last, Some('a' | 'e' | 'i' | 'o' | 'u' | 'y'));

        if matches!(
            letter.to_lowercase().to_string().as_str(),
            "a" | "e" | "i" | "o" | "u" | "y"
        ) && !last_is_vowel
        {
            vowels.push(letter)
        }

        last_letter.replace(letter);
    }
    //vowels.dedup();

    debug!(%word, ?vowels);

    if word.ends_with("ses") || word.ends_with("ces") {
    } else if vowels.len() > 1 {
        if word.ends_with("es") || word.ends_with("ed") || word.ends_with("e") {
            if let Some('e') = vowels.last() {
                vowels.pop();
            }
        }
    }

    debug!(%word, ?vowels);

    vowels.len()
}

#[cfg(test)]
mod tests {
    mod haikus {
        macro_rules! test_haiku {
            ($(#[$m:meta])?
             $name:ident: $text:expr$(,)?) => {
                #[test]
                #[tracing_test::traced_test]
                $(#[$m])?
                fn $name() {
                    assert!(
                        super::super::check_haiku($text).is_some()
                    )
                }
            };

            ($name:ident: $text:expr, $($names:ident: $texts:expr),+$(,)?) => {
                test_haiku!($name: $text);
                test_haiku! { $($names: $texts),+ }
            };
        }

        test_haiku! {
            five: "five five five five five seven seven seven one five five five five five",
            olive: "a haze of olive encompassing points of white vibrantly muted",
            honey: "i am warm honey i am sweet cream and cherries lick me like candy",
            stew: "all the days blending together into a stew but not a good stew",
            tumblr: "anything that one haiku bot on tumblr posts turns out pretty good",
            bigfoot: "i got a picture with bigfoot and the ancient aliens dude slay",
            cool: "look at all the cool things that you find when you are trying to help people",
            a: "a a a a a a a a a a a a a a a a a",
        }

        macro_rules! test_not_haiku {
            ($name:ident: $text:expr$(,)?) => {
                test_haiku!(#[should_panic] $name: $text);
            };

            ($name:ident: $text:expr, $($names:ident: $texts:expr),+$(,)?) => {
                test_not_haiku!($name: $text);
                test_not_haiku! { $($names: $texts),+ }
            };
        }

        test_not_haiku! {
            what: "what",
        }
    }

    mod syllables {
        macro_rules! test_syllables {
            ($word:ident: $count:expr$(,)?) => {
                #[test]
                fn $word() {
                    pretty_assertions::assert_eq!(
                        super::super::syllables(stringify!($word)),
                        $count
                    )
                }
            };

            ($word:ident: $count:expr, $($words:ident: $counts:expr),+$(,)?) => {
                test_syllables!($word: $count);
                test_syllables! { $($words: $counts),+ }
            };
        }

        test_syllables! {
            five: 1,
            seven: 2,
            vibrantly: 3,
            cherries: 2,
            blending: 2,
            together: 3,
            the: 1,
            tumblr: 2,
        }
    }
}
