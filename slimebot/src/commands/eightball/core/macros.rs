macro_rules! create_answer_consts {
    (
        affirmative {
            $($text:literal $($weight:literal)?),+
        }

        non_committal {
            $($text2:literal $($weight2:literal)?),+
        }

        negative {
            $($text3:literal $($weight3:literal)?),+
        }
    ) => {
        macro_rules! weight {
            ($value:literal) => { $value };
            () => { 1.0 }
        }

        pub const ANSWERS: Answers = Answers(&[
            $(Answer { tone: AnswerTone::Affirmative, text: $text, weight: weight!($($weight)?)},)+
            $(Answer { tone: AnswerTone::NonCommittal, text: $text2, weight: weight!($($weight2)?)} ,)+
            $(Answer { tone: AnswerTone::Negative, text: $text3, weight: weight!($($weight3)?)} ,)+
        ]);
    }
}
