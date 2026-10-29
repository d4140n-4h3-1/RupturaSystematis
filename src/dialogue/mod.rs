//! Talking to the maze's inhabitants, as in Fallout 3: E to talk to a droid close by and in
//! front of you, and the camera closes in on its face. What it says runs along the bottom of the
//! screen, in its own System Latin (see `data/system_latin/system_latin_systemized.md`) with
//! what that means under it, and under that the replies to pick from - with the mouse, W and S or
//! the arrows and E, or the number keys. Tab walks away.
//!
//! What each droid says is in `data/dialogue/droids.json`, whose `about` says what everything in
//! it means, and which can be rewritten without a rebuild. Each kind of droid has a
//! conversation of its own: lines, and the replies to each that lead on to other lines or end
//! it. A reply can be a skill check, `[Speech 40%]`, that goes one way if it succeeds and another
//! if it fails; a check is tried only once. A reply already given is shown dimmed.
//!
//! A check can also be tried with a bribe (see [`Bribe`]), offered under it as a reply of its own,
//! `[Speech 75%] [40.00 CR]`: the same check, more likely to succeed, for credits paid whether it
//! does or not. It is one try with the check: trying either takes both away. Without the credits
//! it is shown dimmed, and cannot be picked. Only droids that know something worth paying for
//! are offered bribes - not maintenance units, which do not know where the exit is - and drones
//! cannot be talked to at all.
//!
//! Each line has a mood, which colours the whole panel, and the droid's eyes: green as usual, blue
//! for success, yellow for a warning or a question, orange for agitation, red for hostility. A
//! line says its own, or takes one from how the check that led to it went: blue if it succeeded,
//! orange if not.
//!
//! A line can have the droid turn on the player once the conversation is over (see
//! [`Conversation::attacks`]): a sentry that sees through the player goes after them.
//!
//! A droid on its way to tell security of a shooting it saw (see
//! [`crate::inhabitants::Inhabitants::witnesses`]) can be stopped and questioned: its
//! conversation opens at [`WITNESS_SAW`] if it saw who did it, or [`WITNESS_UNSURE`] if it saw
//! only the one shot, and a line can have it keep quiet about it once it is over
//! ([`Conversation::silences`]), or know the player for who did it after all
//! ([`Conversation::identifies`]). A kind of
//! droid can also have [`Bark`]s, said out loud with nobody talking to it, as it hunts the
//! player: when it spots them, when it loses them, when it hears them, and when it gives up
//! looking.
//!
//! Every kind of droid can say how it takes having the pistol pointed at it ([`Threatened`]):
//! how long it will stand for it before warning the player, and what it does once it has warned
//! them twice - a sentry goes after them, anyone else sounds the alarm for the sentries.
//!
//! Lines can name what is true where the conversation happens, in braces: `{code}`, the droid's
//! code, and `{exit_far}` and `{exit_way}`, how far off the exit is and which way. Each is put in
//! System Latin where the droid says it, and in English where the meaning is given.
//!
//! [`screen`] draws it all.

pub mod screen;

use crate::credits::Credits;
use serde::Deserialize;
use std::collections::{HashMap, HashSet};

/// Where the droids' conversations are.
pub const SCRIPT: &str = "data/dialogue/droids.json";
/// What the droids say in capture the flag, where they only call out, and are not talked to.
pub const CTF_SCRIPT: &str = "data/dialogue/ctf.json";
/// What the droids say in battle royale, where everyone is against everyone: they call out as
/// they fight, and taunt the player who tries to talk to them.
pub const ROYALE_SCRIPT: &str = "data/dialogue/royale.json";
/// What the guards say in a heist, as they hunt whoever breaks in: they are not talked to.
pub const HEIST_SCRIPT: &str = "data/dialogue/heist.json";
/// What the droids of a city say, in the story: about the city, its robberies, and the way to
/// Summa. There are many more of them, and some of its security droids walk about in pairs.
pub const URBS_SCRIPT: &str = "data/dialogue/urbs.json";

/// Where a witness's conversation opens: one that saw who did it, and one that did not.
pub const WITNESS_SAW: &str = "witness_saw";
pub const WITNESS_UNSURE: &str = "witness_unsure";

/// A skill check on a reply: which skill, and the chance it succeeds, in percent.
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct Check {
    pub skill: String,
    pub chance: u32,
}

/// Something the player can say back.
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct Reply {
    pub say: String,
    /// The line it leads to; none ends the conversation.
    #[serde(default)]
    pub to: Option<String>,
    #[serde(default)]
    pub check: Option<Check>,
    /// The line a failed check leads to instead.
    #[serde(default)]
    pub fail: Option<String>,
    /// The check tried with a bribe instead, if the droid takes one.
    #[serde(default)]
    pub bribe: Option<Bribe>,
}

/// A check tried with credits: what the player says instead, how many credits it costs, and the
/// chance the check succeeds with them, in percent - higher than without. It goes where the
/// check would, and the credits are paid either way.
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct Bribe {
    pub say: String,
    pub credits: Credits,
    pub chance: u32,
}

/// How a droid feels saying a line, which colours the panel.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Mood {
    /// Green.
    #[default]
    Normal,
    /// Blue.
    Success,
    /// Yellow: warning, or questioning.
    Warning,
    /// Orange: agitated, or a failed check.
    Agitated,
    /// Red.
    Hostile,
}

impl Mood {
    /// Its name, as the files have it: what a voice's pitch in it is kept under (see
    /// [`crate::formants::speech::Voices::mood_pitch`]).
    pub fn name(self) -> &'static str {
        match self {
            Mood::Normal => "normal",
            Mood::Success => "success",
            Mood::Warning => "warning",
            Mood::Agitated => "agitated",
            Mood::Hostile => "hostile",
        }
    }
}

/// Something a droid says, and what the player can say back.
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct Line {
    /// In System Latin.
    pub says: String,
    /// What that means, in English.
    #[serde(default)]
    pub means: String,
    /// None at all leaves only walking away.
    #[serde(default)]
    pub replies: Vec<Reply>,
    /// None takes the mood from how the check that led here went, if one did.
    #[serde(default)]
    pub mood: Option<Mood>,
    /// Whether the droid goes after the player once the conversation is over.
    #[serde(default)]
    pub attacks: bool,
    /// For a witness: whether it keeps quiet about what it saw once the conversation is over,
    /// telling no one.
    #[serde(default)]
    pub silenced: bool,
    /// For a witness that did not see who did it: whether it knows the player for who did after
    /// all, once the conversation is over.
    #[serde(default)]
    pub identifies: bool,
}

/// Something a droid says out loud by itself, with nobody talking to it.
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct Bark {
    /// In System Latin.
    pub says: String,
    /// What that means, in English.
    #[serde(default)]
    pub means: String,
}

/// What a droid does once it has had the pistol pointed at it for too long.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Provoked {
    /// It goes after the player.
    Attacks,
    /// It sounds the alarm, for the droids that go after the player to come and look.
    Alarm,
}

/// How a droid takes having the pistol pointed at it.
#[derive(Debug, Clone, Copy, PartialEq, Deserialize)]
pub struct Threatened {
    /// How long it stands for it, in seconds, before it warns the player; as long again before
    /// its last warning; and as long again before it does something about it.
    pub patience: f32,
    pub then: Provoked,
}

/// A kind of droid, and how a conversation with one goes.
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct Character {
    /// What it is called, before its code.
    pub name: String,
    /// The line it opens with, and its lines; none, for one that is not talked to.
    #[serde(default)]
    pub start: String,
    #[serde(default)]
    pub lines: HashMap<String, Line>,
    /// What it says by itself, by when: as it hunts the player, `spotted`, `lost`, `heard`,
    /// `alarmed` or `gave_up`; with the pistol pointed at it, `warned`, `warned_again`,
    /// `provoked` or `calmed`; as it goes after anyone, `engaged`; and in battle royale, `downed`
    /// when it has shot someone down, `back` when it comes back after losing a life, and `ring`
    /// when the closing ring catches it outside.
    #[serde(default)]
    pub barks: HashMap<String, Bark>,
    /// What it says when the player tries to talk to it, one at random each time, for one that
    /// has no conversation to have.
    #[serde(default)]
    pub chatter: Vec<Bark>,
    /// What those of a pair or group of its kind say among themselves, going about their
    /// business near the player: each a few lines, said one after the other by one of them and
    /// then the next, round them in turn.
    #[serde(default)]
    pub talks: Vec<Vec<Bark>>,
    /// How it takes having the pistol pointed at it; not at all, without.
    #[serde(default)]
    pub threatened: Option<Threatened>,
    /// The model it is made from; the player's droid's, without.
    #[serde(default)]
    pub model: Option<String>,
    /// Whether it changes into the hostile droid's colours while it is after the player. Sentries
    /// keep their own; the colours are for another kind of droid.
    #[serde(default)]
    pub hostile_colours: bool,
    /// What share of droids of its kind walk about in twos, one keeping beside the other, from 0
    /// to 1: none, without.
    #[serde(default)]
    pub in_pairs: f32,
    /// The most of its kind that walk about together, one leading and the rest about it: each
    /// group is from two to this many. Without, as `in_pairs` says.
    #[serde(default)]
    pub in_groups_of: usize,
    /// How often one of its kind - or a group of them - is put down, against the others' shares:
    /// as often as any other, without.
    #[serde(default = "one")]
    pub share: f32,
    /// How near the player, in meters, has to be for it to take it as the pistol pointed at it
    /// (see `threatened`): standing over it, it warns them, and if they stay, it does something
    /// about it. Nearness is nothing to it, without.
    #[serde(default)]
    pub crowded: Option<f32>,
    /// Whether its kind is a gang: it answers no alarm, and none of the others join its chases -
    /// its own group goes after the player with it.
    #[serde(default)]
    pub gang: bool,
    /// The colour of its eyes while it is hostile, red to blue from 0 to 255 each: the usual red,
    /// without.
    #[serde(default)]
    pub hostile_eyes: Option<[u8; 3]>,
    /// How many of its groups keep to a spot of their own rather than walk about, and how far
    /// round it is theirs (see [`Turf`]): none, without.
    #[serde(default)]
    pub turf: Option<Turf>,
}

/// A group's turf: the spot it keeps to, standing about there, and the ground round it that it
/// takes as its own. The player on it is as the pistol pointed at it (see `threatened`), and the
/// others keep off it.
#[derive(Debug, Clone, Copy, PartialEq, Deserialize)]
pub struct Turf {
    /// The share of its kind's groups that keep to one, from 0 to 1.
    pub share: f32,
    /// How far round the spot is theirs, in meters.
    pub reach: f32,
}

fn one() -> f32 {
    1.0
}

/// Everyone's conversations, as the file has them.
#[derive(Debug, Clone, Default, PartialEq, Deserialize)]
pub struct Script {
    pub characters: Vec<Character>,
    /// How many droids live in a level they are in; the game's own number, without.
    #[serde(default)]
    pub inhabitants: Option<usize>,
    /// Whether they are a city's crowd: all about the player, those left far behind out of sight
    /// turning up again nearer by, as others (see [`crate::inhabitants::Inhabitants::recycle`]).
    #[serde(default)]
    pub crowd: bool,
}

impl Script {
    /// The conversations in the file at `path`, as long as every reply leads somewhere there is.
    pub fn load(path: &str) -> Result<Self, String> {
        let text = crate::platform::read_to_string(path)?;
        let script: Self =
            serde_json::from_str(&text).map_err(|error| format!("{path}: {error}"))?;
        match script.problems().first() {
            Some(problem) => Err(format!("{path}: {problem}")),
            None => Ok(script),
        }
    }

    /// Whatever in the conversations leads nowhere.
    pub fn problems(&self) -> Vec<String> {
        let mut problems = Vec::new();
        for character in &self.characters {
            let name = &character.name;
            let lines = &character.lines;
            if !lines.is_empty() && !lines.contains_key(&character.start) {
                problems.push(format!(
                    "{name} starts with {}, which it does not have",
                    character.start
                ));
            }
            for (key, line) in lines {
                for reply in &line.replies {
                    let leads = reply.to.iter().chain(&reply.fail);
                    for to in leads.filter(|to| !lines.contains_key(*to)) {
                        problems.push(format!(
                            "{name}'s {key}: \"{}\" leads to {to}, which it does not have",
                            reply.say
                        ));
                    }
                    if reply.check.is_some() && reply.fail.is_none() {
                        problems.push(format!(
                            "{name}'s {key}: \"{}\" is a check with no fail",
                            reply.say
                        ));
                    }
                    match (&reply.check, &reply.bribe) {
                        (None, Some(_)) => problems.push(format!(
                            "{name}'s {key}: \"{}\" has a bribe but no check",
                            reply.say
                        )),
                        (Some(check), Some(bribe)) if bribe.chance <= check.chance => {
                            problems.push(format!(
                                "{name}'s {key}: \"{}\" has a bribe no likelier to work",
                                reply.say
                            ))
                        }
                        _ => (),
                    }
                }
            }
        }
        problems
    }
}

/// What is true where a conversation happens, for lines to name: each `{key}`, in System Latin
/// and in English.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Facts(Vec<(&'static str, String, String)>);

impl Facts {
    pub fn with(
        mut self,
        key: &'static str,
        latin: impl Into<String>,
        english: impl Into<String>,
    ) -> Self {
        self.0.push((key, latin.into(), english.into()));
        self
    }

    /// `text` with every `{key}` it names filled in, in System Latin or in English.
    fn fill(&self, text: &str, latin: bool) -> String {
        self.0.iter().fold(text.to_string(), |text, (key, l, e)| {
            text.replace(&format!("{{{key}}}"), if latin { l } else { e })
        })
    }
}

/// A number as System Latin reads out a code: digit by digit (see section XXX of
/// `data/system_latin/system_latin_systemized.md`).
pub fn digits(number: u32) -> String {
    const DIGITS: [&str; 10] = [
        "zero", "unum", "duo", "tria", "quattuor", "quinque", "sex", "septem", "octo", "novem",
    ];
    number
        .to_string()
        .bytes()
        .map(|digit| DIGITS[(digit - b'0') as usize])
        .collect::<Vec<_>>()
        .join(" ")
}

/// What the player can say back, as shown.
#[derive(Debug, Clone, PartialEq)]
pub struct Choice {
    /// With its check, if it has one, in front.
    pub label: String,
    /// Whether it has been said before, and is dimmed.
    pub said: bool,
}

/// What is on screen for the line the conversation is at.
#[derive(Debug, Clone, PartialEq)]
pub struct View {
    pub says: String,
    pub means: String,
    /// How the last check went, if the last reply was one.
    pub note: Option<String>,
    pub choices: Vec<Choice>,
    pub mood: Mood,
}

/// Where a conversation with one droid has got to.
#[derive(Debug, Clone, PartialEq)]
pub struct Conversation {
    character: usize,
    line: String,
    /// The replies said so far, by the line they were said to and which of its replies they were.
    said: HashSet<(String, usize)>,
    /// How the last check went, if the last reply was one: what to say about it, and whether it
    /// succeeded.
    note: Option<(String, bool)>,
}

impl Conversation {
    /// A conversation with a droid that is `character` of `script`, from its line `start` if it
    /// has it - a witness's (see [`WITNESS_SAW`]) - or else its opening line.
    pub fn from(script: &Script, character: usize, start: Option<&str>) -> Option<Self> {
        let kind = script.characters.get(character)?;
        let line = start.filter(|start| kind.lines.contains_key(*start)).unwrap_or(&kind.start).to_string();
        Some(Self {
            character,
            line,
            said: HashSet::new(),
            note: None,
        })
    }

    fn line<'a>(&self, script: &'a Script) -> Option<&'a Line> {
        script.characters.get(self.character)?.lines.get(&self.line)
    }

    /// The replies there are to pick from, as indices into the line's replies, each with whether
    /// it is the reply's bribe: all but the checks already tried, each check's bribe after it.
    /// None for the line's end, when it has no replies.
    fn offered(&self, line: &Line) -> Vec<Option<(usize, bool)>> {
        let offered: Vec<Option<(usize, bool)>> = (0..line.replies.len())
            .filter(|&i| {
                line.replies[i].check.is_none() || !self.said.contains(&(self.line.clone(), i))
            })
            .flat_map(|i| {
                let bribe = line.replies[i].check.is_some() && line.replies[i].bribe.is_some();
                std::iter::once(Some((i, false))).chain(bribe.then_some(Some((i, true))))
            })
            .collect();
        if offered.is_empty() {
            vec![None]
        } else {
            offered
        }
    }

    /// What there is to show, with `facts` filled in, for a player with `credits`.
    pub fn view(&self, script: &Script, facts: &Facts, credits: Credits) -> View {
        let Some(line) = self.line(script) else {
            return View {
                says: String::new(),
                means: String::new(),
                note: None,
                choices: vec![leave()],
                mood: Mood::Normal,
            };
        };
        let choices = self
            .offered(line)
            .into_iter()
            .map(|offered| {
                let Some((index, bribed)) = offered else {
                    return leave();
                };
                let reply = &line.replies[index];
                match (&reply.check, reply.bribe.as_ref().filter(|_| bribed)) {
                    // Dimmed while the player cannot pay.
                    (Some(check), Some(bribe)) => Choice {
                        label: format!(
                            "[{} {}%] [{}] {}",
                            check.skill,
                            bribe.chance,
                            bribe.credits,
                            facts.fill(&bribe.say, false)
                        ),
                        said: credits < bribe.credits,
                    },
                    (check, _) => {
                        let say = facts.fill(&reply.say, false);
                        Choice {
                            label: match check {
                                Some(check) => format!("[{} {}%] {say}", check.skill, check.chance),
                                None => say,
                            },
                            said: self.said.contains(&(self.line.clone(), index)),
                        }
                    }
                }
            })
            .collect();
        let mood = line.mood.unwrap_or(match self.note {
            Some((_, true)) => Mood::Success,
            Some((_, false)) => Mood::Agitated,
            None => Mood::Normal,
        });
        View {
            says: facts.fill(&line.says, true),
            means: facts.fill(&line.means, false),
            note: self.note.as_ref().map(|(note, _)| note.clone()),
            choices,
            mood,
        }
    }

    /// Whether the line the conversation is at has the droid go after the player once it is
    /// over.
    pub fn attacks(&self, script: &Script) -> bool {
        self.line(script).is_some_and(|line| line.attacks)
    }

    /// Whether the line the conversation is at has a witness keep quiet about what it saw once it
    /// is over.
    pub fn silences(&self, script: &Script) -> bool {
        self.line(script).is_some_and(|line| line.silenced)
    }

    /// Whether the line the conversation is at has a witness know the player for who did it once
    /// it is over.
    pub fn identifies(&self, script: &Script) -> bool {
        self.line(script).is_some_and(|line| line.identifies)
    }

    /// What the `choice`th of the replies on offer costs, if it is a bribe.
    pub fn price(&self, script: &Script, choice: usize) -> Option<Credits> {
        let line = self.line(script)?;
        let (index, true) = (*self.offered(line).get(choice)?)? else {
            return None;
        };
        Some(line.replies[index].bribe.as_ref()?.credits)
    }

    /// Says the `choice`th of the replies on offer, rolling `roll` - from 0 to 99 - for its check
    /// if it has one, and paying for its bribe from `credits` if it is one; a bribe the player
    /// cannot pay for is not said. False once the conversation is over.
    pub fn choose(&mut self, script: &Script, choice: usize, roll: u32, credits: &mut Credits) -> bool {
        let Some(line) = self.line(script) else {
            return false;
        };
        let Some(&Some((index, bribed))) = self.offered(line).get(choice) else {
            // Past the end of what is on offer is nothing; walking away from a line with no
            // replies ends it.
            return choice >= self.offered(line).len();
        };
        let reply = &line.replies[index];
        let bribe = reply.bribe.as_ref().filter(|_| bribed);
        if bribe.is_some_and(|bribe| !credits.spend(bribe.credits)) {
            return true;
        }
        self.said.insert((self.line.clone(), index));
        self.note = None;
        let next = match &reply.check {
            Some(check) => {
                let chance = bribe.map_or(check.chance, |bribe| bribe.chance);
                let passed = roll < chance;
                let paid = bribe.map_or(String::new(), |bribe| format!(", {} paid", bribe.credits));
                self.note = Some((
                    format!(
                        "[{} {}%] {}{paid}",
                        check.skill,
                        chance,
                        if passed { "Succeeded" } else { "Failed" }
                    ),
                    passed,
                ));
                if passed {
                    &reply.to
                } else {
                    &reply.fail
                }
            }
            None => &reply.to,
        };
        match next {
            Some(next) => {
                self.line = next.clone();
                true
            }
            None => false,
        }
    }
}

/// Walking away, for a line nobody can reply to.
fn leave() -> Choice {
    Choice {
        label: "[Leave]".to_string(),
        said: false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn script() -> Script {
        serde_json::from_str(
            r#"{ "characters": [ { "name": "Explorator", "start": "hello", "lines": {
                "hello": { "says": "Explorator codex {code}.", "means": "Scout {code}.",
                    "replies": [
                        { "say": "Where is the exit?", "check": { "skill": "Speech", "chance": 50 },
                          "to": "exit", "fail": "no" },
                        { "say": "Tell me again.", "to": "hello" },
                        { "say": "Goodbye." } ] },
                "exit": { "says": "Progreda {exit_way}.", "means": "Go {exit_way}." },
                "no": { "says": "Negativum.", "means": "No.",
                    "replies": [ { "say": "Back.", "to": "hello" } ] },
                "cross": { "says": "Sta.", "means": "Stop.", "mood": "hostile",
                    "attacks": true } } } ] }"#,
        )
        .unwrap()
    }

    fn facts() -> Facts {
        Facts::default().with("code", "quattuor septem", "47").with(
            "exit_way",
            "ad sinistrum",
            "to your left",
        )
    }

    #[test]
    fn it_opens_with_the_first_line_and_fills_in_what_is_known() {
        let script = script();
        let view = Conversation::from(&script, 0, None)
            .unwrap()
            .view(&script, &facts(), Credits::default());
        assert_eq!(view.says, "Explorator codex quattuor septem.");
        assert_eq!(view.means, "Scout 47.");
        let labels: Vec<_> = view.choices.iter().map(|c| c.label.as_str()).collect();
        assert_eq!(
            labels,
            [
                "[Speech 50%] Where is the exit?",
                "Tell me again.",
                "Goodbye."
            ]
        );
    }

    #[test]
    fn a_check_goes_one_way_or_the_other_and_is_tried_only_once() {
        let script = script();
        let mut talk = Conversation::from(&script, 0, None).unwrap();
        assert!(
            talk.choose(&script, 0, 50, &mut Credits::default()),
            "a roll of 50 fails a 50% check"
        );
        let view = talk.view(&script, &facts(), Credits::default());
        assert_eq!(view.says, "Negativum.");
        assert_eq!(view.note.as_deref(), Some("[Speech 50%] Failed"));
        assert!(talk.choose(&script, 0, 0, &mut Credits::default()));
        let view = talk.view(&script, &facts(), Credits::default());
        assert_eq!(view.note, None);
        assert_eq!(view.choices.len(), 2, "the check is gone");

        let mut talk = Conversation::from(&script, 0, None).unwrap();
        assert!(talk.choose(&script, 0, 49, &mut Credits::default()));
        assert_eq!(talk.view(&script, &facts(), Credits::default()).says, "Progreda ad sinistrum.");
    }

    #[test]
    fn a_line_has_its_own_mood_or_the_one_its_check_gives_it() {
        let script = script();
        let mut talk = Conversation::from(&script, 0, None).unwrap();
        assert_eq!(talk.view(&script, &facts(), Credits::default()).mood, Mood::Normal);
        talk.choose(&script, 0, 99, &mut Credits::default());
        assert_eq!(talk.view(&script, &facts(), Credits::default()).mood, Mood::Agitated, "failed");
        talk.choose(&script, 0, 0, &mut Credits::default());
        assert_eq!(
            talk.view(&script, &facts(), Credits::default()).mood,
            Mood::Normal,
            "no check this time"
        );
        let mut talk = Conversation::from(&script, 0, None).unwrap();
        talk.choose(&script, 0, 0, &mut Credits::default());
        assert_eq!(
            talk.view(&script, &facts(), Credits::default()).mood,
            Mood::Success,
            "succeeded"
        );
        talk.line = "cross".into();
        assert_eq!(talk.view(&script, &facts(), Credits::default()).mood, Mood::Hostile, "its own");
    }

    #[test]
    fn only_a_line_that_says_so_ends_in_an_attack() {
        let script = script();
        let mut talk = Conversation::from(&script, 0, None).unwrap();
        assert!(!talk.attacks(&script));
        talk.line = "cross".into();
        assert!(talk.attacks(&script));
    }

    #[test]
    fn what_has_been_said_is_dimmed() {
        let script = script();
        let mut talk = Conversation::from(&script, 0, None).unwrap();
        talk.choose(&script, 1, 0, &mut Credits::default());
        let view = talk.view(&script, &facts(), Credits::default());
        assert!(view.choices[1].said && !view.choices[2].said);
    }

    #[test]
    fn goodbye_and_a_line_with_no_replies_end_it() {
        let script = script();
        let mut talk = Conversation::from(&script, 0, None).unwrap();
        assert!(!talk.choose(&script, 2, 0, &mut Credits::default()), "goodbye");
        let mut talk = Conversation::from(&script, 0, None).unwrap();
        talk.choose(&script, 0, 0, &mut Credits::default());
        let view = talk.view(&script, &facts(), Credits::default());
        assert_eq!(view.choices, [leave()]);
        assert!(!talk.choose(&script, 0, 0, &mut Credits::default()), "leaving");
    }

    #[test]
    fn a_reply_that_leads_nowhere_is_found() {
        let mut script = script();
        script.characters[0].lines.get_mut("no").unwrap().replies[0].to = Some("gone".into());
        assert_eq!(script.problems().len(), 1);
    }

    #[test]
    fn a_witness_is_questioned_from_its_own_line_and_can_be_silenced_or_identify_the_player() {
        let s: Script = serde_json::from_str(
            r#"{ "characters": [ { "name": "Explorator", "start": "hello", "lines": {
                "hello": { "says": "Salve.", "means": "Hello." },
                "witness_saw": { "says": "Tu.", "means": "You!", "replies": [
                    { "say": "Forget it.", "check": { "skill": "Intimidate", "chance": 40 },
                      "to": "silenced", "fail": "defiant" } ] },
                "silenced": { "says": "Silentium.", "means": "Quiet.", "silenced": true },
                "defiant": { "says": "Negativum.", "means": "No." },
                "witness_unsure": { "says": "Quis?", "means": "Who?", "replies": [
                    { "say": "It was me.", "to": "confessed" } ] },
                "confessed": { "says": "Tu.", "means": "You!", "identifies": true } } } ] }"#,
        )
        .unwrap();
        assert!(s.problems().is_empty(), "{:?}", s.problems());
        let mut credits = Credits::default();
        let mut saw = Conversation::from(&s, 0, Some(WITNESS_SAW)).unwrap();
        assert_eq!(saw.view(&s, &Facts::default(), credits).means, "You!");
        assert!(!saw.silences(&s), "not yet");
        saw.choose(&s, 0, 0, &mut credits);
        assert!(saw.silences(&s) && !saw.identifies(&s), "intimidated into keeping quiet");
        let mut defiant = Conversation::from(&s, 0, Some(WITNESS_SAW)).unwrap();
        defiant.choose(&s, 0, 99, &mut credits);
        assert!(!defiant.silences(&s), "not frightened");
        let mut unsure = Conversation::from(&s, 0, Some(WITNESS_UNSURE)).unwrap();
        unsure.choose(&s, 0, 0, &mut credits);
        assert!(unsure.identifies(&s) && !unsure.silences(&s), "told who did it");
        let other = Conversation::from(&s, 0, Some("nowhere")).unwrap();
        assert_eq!(other.view(&s, &Facts::default(), credits).means, "Hello.", "its opening line, without");
    }

    #[test]
    fn urbs_witnesses_can_be_questioned_by_the_player_and_by_security() {
        let script = Script::load(URBS_SCRIPT).unwrap();
        let mut witnesses = 0;
        for character in &script.characters {
            match character.threatened.map(|t| t.then) {
                Some(Provoked::Alarm) => {
                    witnesses += 1;
                    for line in [WITNESS_SAW, WITNESS_UNSURE] {
                        assert!(character.lines.contains_key(line), "{} opens at {line}", character.name);
                    }
                    let lines = || character.lines.values();
                    assert!(lines().any(|line| line.silenced), "{} can be silenced", character.name);
                    assert!(lines().any(|line| line.identifies), "{} can know who did it after all", character.name);
                    for bark in ["saw_who", "saw_none"] {
                        assert!(character.barks.contains_key(bark), "{} answers {bark}", character.name);
                    }
                }
                Some(Provoked::Attacks) if !character.gang => {
                    for bark in ["question", "identified", "unidentified", "heard"] {
                        assert!(character.barks.contains_key(bark), "{} says {bark}", character.name);
                    }
                }
                _ => (),
            }
        }
        assert!(witnesses >= 2, "kinds that run to tell security");
    }

    #[test]
    fn codes_are_read_digit_by_digit() {
        assert_eq!(digits(47), "quattuor septem");
        assert_eq!(digits(90), "novem zero");
    }

    #[test]
    fn the_droids_conversations_load() {
        let script = Script::load(SCRIPT).unwrap();
        assert!(!script.characters.is_empty());
    }

    #[test]
    fn urbs_is_peopled_some_of_its_security_in_pairs_and_each_kind_talks_of_summa() {
        let script = Script::load(URBS_SCRIPT).unwrap();
        let voices = <crate::formants::speech::Voices as crate::formants::Load>::load(crate::formants::VOICES).unwrap();
        assert!(script.inhabitants.is_some_and(|n| n > 6), "more than a maze has");
        let sentries = script.characters.iter().filter(|c| !c.gang && c.threatened.is_some_and(|t| t.then == Provoked::Attacks));
        assert!(sentries.clone().count() > 0 && sentries.clone().all(|c| (0.0..1.0).contains(&c.in_pairs) && c.in_pairs > 0.0), "some of security in pairs, not all");
        assert!(script.characters.iter().filter(|c| c.threatened.is_some_and(|t| t.then != Provoked::Attacks)).all(|c| c.in_pairs == 0.0));
        for character in script.characters.iter().filter(|c| !c.gang) {
            assert!(voices.voices.contains_key(&character.name), "{} has a voice", character.name);
            let summa = character.lines.values().any(|line| line.says.contains("Summa"));
            assert!(summa, "{} talks of Summa", character.name);
        }
    }

    #[test]
    fn urbs_has_gangs_in_groups_of_up_to_six_that_mind_the_player_standing_over_them() {
        let script = Script::load(URBS_SCRIPT).unwrap();
        let voices = <crate::formants::speech::Voices as crate::formants::Load>::load(crate::formants::VOICES).unwrap();
        let gangs: Vec<&Character> = script.characters.iter().filter(|c| c.gang).collect();
        assert!(!gangs.is_empty(), "a gang");
        for gang in gangs {
            assert_eq!(gang.in_groups_of, 6, "{} in groups of up to six", gang.name);
            assert!(gang.share > 0.0 && gang.share < 1.0, "{} rarer than the others", gang.name);
            assert!(gang.crowded.is_some_and(|reach| reach > 0.0), "{} minds the player near", gang.name);
            assert!(gang.threatened.is_some_and(|t| t.then == Provoked::Attacks), "{} goes for them", gang.name);
            assert!(gang.hostile_eyes.is_some_and(|eyes| eyes != [255, 0, 0]), "{} hostile in its own colour", gang.name);
            assert!(gang.lines.is_empty() && gang.chatter.len() >= 2, "{} only chatters", gang.name);
            assert!(gang.talks.len() >= 4, "{} talks among itself", gang.name);
            for talk in &gang.talks {
                assert!(talk.len() >= 2 && talk.iter().all(|line| !line.means.is_empty()), "{} answers itself", gang.name);
            }
            assert!(
                gang.turf.is_some_and(|turf| (0.0..=1.0).contains(&turf.share) && turf.reach > gang.crowded.unwrap_or(0.0)),
                "{} keeps to turf of its own, wider than it minds the player standing over it",
                gang.name
            );
            for bark in ["warned", "warned_again", "provoked", "calmed", "spotted", "lost", "heard", "gave_up", "flee", "caught", "engaged", "won"] {
                assert!(gang.barks.contains_key(bark), "{} says {bark}", gang.name);
            }
            assert!(voices.voices.contains_key(&gang.name), "{} has a voice", gang.name);
        }
        // Its security calls out as it goes after one of a gang in a foot chase, and in a
        // shootout with a gang.
        for sentry in script.characters.iter().filter(|c| !c.gang && c.threatened.is_some_and(|t| t.then == Provoked::Attacks)) {
            for bark in ["chase", "caught", "escaped", "shootout", "engaged", "won"] {
                assert!(sentry.barks.contains_key(bark), "{} says {bark}", sentry.name);
            }
            // And those walking about in pairs talk between themselves.
            assert!(sentry.in_pairs > 0.0 && sentry.talks.len() >= 4, "{} talks between its pairs", sentry.name);
            for talk in &sentry.talks {
                assert!(talk.len() >= 2 && talk.iter().all(|line| !line.means.is_empty()), "{} answers itself", sentry.name);
            }
        }
    }

    /// What a competitor in battle royale says by itself: hunting the player, going after anyone,
    /// having shot someone down, coming back after losing a life, and caught outside the ring.
    const ROYALE_BARKS: [&str; 8] = ["spotted", "lost", "heard", "gave_up", "engaged", "downed", "back", "ring"];

    #[test]
    fn battle_royales_droids_call_out_and_taunt_each_in_a_voice_of_its_own() {
        let script = Script::load(ROYALE_SCRIPT).unwrap();
        let voices = <crate::formants::speech::Voices as crate::formants::Load>::load(crate::formants::VOICES).unwrap();
        assert!(script.characters.len() >= 2, "more than one kind of competitor");
        let mut pitches = Vec::new();
        for character in &script.characters {
            assert!(character.lines.is_empty() && character.start.is_empty(), "{} is not talked to", character.name);
            for bark in ROYALE_BARKS {
                let said = character.barks.get(bark);
                assert!(said.is_some_and(|b| !b.says.is_empty() && !b.means.is_empty()), "{} says {bark}", character.name);
            }
            assert!(character.chatter.len() >= 2, "{} taunts", character.name);
            assert!(character.model.is_some(), "{} looks like itself", character.name);
            let voice = voices.voices.get(&character.name).expect("a voice of its own");
            pitches.push(voice.pitch);
        }
        pitches.sort_by(f32::total_cmp);
        pitches.dedup();
        assert_eq!(pitches.len(), script.characters.len(), "every voice its own pitch");
    }

    #[test]
    fn heist_guards_call_out_each_in_a_voice_of_its_own() {
        let script = Script::load(HEIST_SCRIPT).unwrap();
        let voices = <crate::formants::speech::Voices as crate::formants::Load>::load(crate::formants::VOICES).unwrap();
        assert!(!script.characters.is_empty());
        for character in &script.characters {
            assert!(character.lines.is_empty(), "{} is not talked to", character.name);
            for bark in ["spotted", "lost", "heard", "gave_up", "alarmed"] {
                assert!(character.barks.contains_key(bark), "{} says {bark}", character.name);
            }
            assert!(character.model.is_some());
            assert!(voices.voices.contains_key(&character.name), "{} has a voice", character.name);
        }
    }

    #[test]
    fn capture_the_flags_droids_only_call_out_each_side_in_a_voice_of_its_own() {
        let script = Script::load(CTF_SCRIPT).unwrap();
        let voices = <crate::formants::speech::Voices as crate::formants::Load>::load(crate::formants::VOICES).unwrap();
        let [ally, enemy] = &script.characters[..] else {
            panic!("a kind of droid for each side");
        };
        let mut pitches = Vec::new();
        for character in [ally, enemy] {
            assert!(character.lines.is_empty(), "{} is not talked to", character.name);
            assert!(character.barks.contains_key("engaged"), "{} calls out", character.name);
            let voice = voices.voices.get(&character.name).expect("a voice of its own");
            pitches.push(voice.pitch);
        }
        assert!(pitches[0] > 2.0 * pitches[1], "told apart by ear");
        assert!(ally.chatter.len() > 1, "the player's own say something when talked to");
        assert!(enemy.chatter.is_empty(), "the enemy's are not talked to");
    }

    fn bribable() -> Script {
        serde_json::from_str(
            r#"{ "characters": [ { "name": "Defendator", "start": "hello", "lines": {
                "hello": { "says": "Sta.", "means": "Halt.",
                    "replies": [
                        { "say": "I'm maintenance.", "check": { "skill": "Speech", "chance": 40 },
                          "bribe": { "say": "I'm maintenance. Here.", "credits": 40, "chance": 75 },
                          "to": "cleared", "fail": "suspect" },
                        { "say": "Goodbye." } ] },
                "cleared": { "says": "Progreda.", "means": "Proceed." },
                "suspect": { "says": "Suspecto tu.", "means": "I suspect you.",
                    "replies": [ { "say": "Back.", "to": "hello" } ] } } } ] }"#,
        )
        .unwrap()
    }

    #[test]
    fn a_bribe_is_offered_under_its_check_and_dimmed_without_the_credits() {
        let script = bribable();
        let talk = Conversation::from(&script, 0, None).unwrap();
        let view = talk.view(&script, &facts(), Credits::new(39, 99));
        let labels: Vec<_> = view.choices.iter().map(|c| c.label.as_str()).collect();
        assert_eq!(
            labels,
            [
                "[Speech 40%] I'm maintenance.",
                "[Speech 75%] [40.00 CR] I'm maintenance. Here.",
                "Goodbye."
            ]
        );
        assert!(view.choices[1].said, "dimmed: too poor");
        assert!(!talk.view(&script, &facts(), Credits::new(40, 0)).choices[1].said);
        assert_eq!(talk.price(&script, 1), Some(Credits::new(40, 0)));
        assert_eq!(talk.price(&script, 0), None);
    }

    #[test]
    fn a_bribe_is_a_likelier_check_paid_for_either_way_and_tried_only_once() {
        let script = bribable();
        // Too poor: nothing happens, and nothing is paid.
        let mut talk = Conversation::from(&script, 0, None).unwrap();
        let mut poor = Credits::new(10, 0);
        assert!(talk.choose(&script, 1, 0, &mut poor));
        assert_eq!(poor, Credits::new(10, 0));
        assert_eq!(talk.view(&script, &facts(), poor).says, "Sta.");
        // A roll the check alone would fail, and the bribe passes.
        let mut wallet = Credits::new(50, 25);
        assert!(talk.choose(&script, 1, 60, &mut wallet));
        assert_eq!(wallet, Credits::new(10, 25));
        let view = talk.view(&script, &facts(), wallet);
        assert_eq!(view.says, "Progreda.");
        assert_eq!(view.note.as_deref(), Some("[Speech 75%] Succeeded, 40.00 CR paid"));
        // Failing, it is paid all the same; and the check and its bribe are both gone after.
        let mut talk = Conversation::from(&script, 0, None).unwrap();
        let mut wallet = Credits::new(40, 0);
        assert!(talk.choose(&script, 1, 75, &mut wallet));
        assert_eq!(wallet, Credits::default());
        assert_eq!(talk.view(&script, &facts(), wallet).says, "Suspecto tu.");
        talk.choose(&script, 0, 0, &mut wallet);
        let labels: Vec<_> = talk
            .view(&script, &facts(), wallet)
            .choices
            .into_iter()
            .map(|c| c.label)
            .collect();
        assert_eq!(labels, ["Goodbye."]);
    }

    #[test]
    fn a_bribe_needs_a_check_and_better_odds_than_it() {
        let mut script = bribable();
        let reply = &mut script.characters[0].lines.get_mut("hello").unwrap().replies[0];
        reply.bribe.as_mut().unwrap().chance = 40;
        assert_eq!(script.problems().len(), 1, "no likelier");
        let reply = &mut script.characters[0].lines.get_mut("hello").unwrap().replies[0];
        reply.check = None;
        reply.fail = None;
        assert_eq!(script.problems().len(), 1, "no check");
    }
}
