use tracing::debug;

pub fn check_haiku(text: &str) -> Option<Vec<String>> {
    //use hypher::hyphenate;

    //let en_us = Standard::from_embedded(Language::EnglishUS).unwrap();

    let words = text.split_whitespace();
    let parsed = words.clone().map(|word| {
        syllables(word)
    });

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
        _ => ()
    }

    let mut last_letter: Option<char> = None;

    let mut vowels = Vec::new();
    
    for letter in word.chars() {
        let last = last_letter.map(|ch| ch.to_lowercase().next().unwrap());

        let last_is_vowel = matches!(last, Some('a' | 'e' | 'i' | 'o' | 'u' | 'y'));

        if matches!(letter.to_lowercase().to_string().as_str(), "a" | "e" | "i" | "o" | "u" | "y")
            && !last_is_vowel {
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
            ($name:ident, $text:expr) => {
                #[test]
                #[tracing_test::traced_test]
                fn $name() {
                    assert!(
                        super::super::check_haiku($text).is_some()
                    )
                }
            };
        }

        test_haiku!(five, "five five five five five seven seven seven one five five five five five");
        test_haiku!(olive, "a haze of olive encompassing points of white vibrantly muted");
        test_haiku!(honey, "i am warm honey i am sweet cream and cherries lick me like candy");
        test_haiku!(stew, "all the days blending together into a stew but not a good stew");
        test_haiku!(tumblr, "anything that one haiku bot on tumblr posts turns out pretty good");
        test_haiku!(bigfoot, "i got a picture with bigfoot and the ancient aliens dude slay");
        test_haiku!(cool, "look at all the cool things that you find when you are trying to help people");
        //test_haiku!(cringe, "don't kill the part of you that is cringe. kill the part of you that cringes.");
        test_haiku!(a, "a a a a a a a a a a a a a a a a a");

        macro_rules! test_not_haiku {
            ($name:ident, $text:expr) => {
                #[test]
                #[tracing_test::traced_test]
                #[should_panic]
                fn $name() {
                    assert!(
                        super::super::check_haiku($text).is_some()
                    )
                }
            };
        }

        test_not_haiku!(what, "what");
    }
    
    mod syllables {
        macro_rules! test_syllables {
            ($word:ident, $count:expr) => {
                #[test]
                #[tracing_test::traced_test]
                fn $word() {
                    pretty_assertions::assert_eq!(
                        super::super::syllables(stringify!($word)),
                        $count
                    )
                }
            };
        }

        test_syllables!(five, 1);
        test_syllables!(seven, 2);
        test_syllables!(vibrantly, 3);
        test_syllables!(cherries, 2);
        test_syllables!(blending, 2);
        test_syllables!(together, 3);
        test_syllables!(the, 1);
        test_syllables!(tumblr, 2);
    }
}