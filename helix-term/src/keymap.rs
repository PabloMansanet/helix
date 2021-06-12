use crate::commands;
pub use crate::commands::Command;
use anyhow::{anyhow, Error, Result};
use helix_core::hashmap;
use helix_view::document::Mode;
use std::{collections::HashMap, fmt::Display, str::FromStr};

// Kakoune-inspired:
// mode = {
//      normal = {
//          q = record_macro
//          w = (next) word
//          W = next WORD
//          e = end of word
//          E = end of WORD
//          r = replace
//          R = replace with yanked
//          t = 'till char
//          y = yank
//          u = undo
//          U = redo
//          i = insert
//          I = INSERT (start of line)
//          o = open below (insert on new line below)
//          O = open above (insert on new line above)
//          p = paste (before cursor)
//          P = PASTE (after cursor)
//          ` =
//          [ = select to text object start (alt = select whole object)
//          ] = select to text object end
//          { = extend to inner object start
//          } = extend to inner object end
//          a = append
//          A = APPEND (end of line)
//          s = split
//          S = select
//          d = delete()
//          f = find_char()
//          g = goto (gg, G, gc, gd, etc)
//
//          h = move_char_left(n)   || arrow-left  = move_char_left(n)
//          j = move_line_down(n)   || arrow-down  = move_line_down(n)
//          k = move_line_up(n)     || arrow_up    = move_line_up(n)
//          l = move_char_right(n)  || arrow-right = move_char_right(n)
//          : = command line
//          ; = collapse selection to cursor
//          " = use register
//          ` = convert case? (to lower) (alt = swap case)
//          ~ = convert to upper case
//          . = repeat last command
//          \ = disable hook?
//          / = search
//          > = indent
//          < = deindent
//          % = select whole buffer (in vim = jump to matching bracket)
//          * = search pattern in selection
//          ( = rotate main selection backward
//          ) = rotate main selection forward
//          - = trim selections? (alt = merge contiguous sel together)
//          @ = convert tabs to spaces
//          & = align cursor
//          ? = extend to next given regex match (alt = to prev)
//
//          in kakoune these are alt-h alt-l / gh gl
//                              select from curs to begin end / move curs to begin end
//          0 = start of line
//          ^ = start of line(first non blank char) || Home  = start of line(first non blank char)
//          $ = end of line                         || End   = end of line
//
//          z = save selections
//          Z = restore selections
//          x = select line
//          X = extend line
//          c = change selected text
//          C = copy selection?
//          v = view menu (viewport manipulation)
//          b = select to previous word start
//          B = select to previous WORD start
//
//
//
//
//
//
//          = = align?
//          + =
//      }
//
//      gd = goto definition
//      gr = goto reference
//      [d = previous diagnostic
//      d] = next diagnostic
//      [D = first diagnostic
//      D] = last diagnostic
// }

// #[cfg(feature = "term")]
pub use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

pub type Keymap = HashMap<KeyEvent, Command>;
pub type Keymaps = HashMap<Mode, Keymap>;

pub type Remap = HashMap<KeyEvent, KeyEvent>;
pub type Remaps = HashMap<Mode, Remap>;

#[macro_export]
macro_rules! key {
    ($($ch:tt)*) => {
        KeyEvent {
            code: KeyCode::Char($($ch)*),
            modifiers: KeyModifiers::NONE,
        }
    };
}

macro_rules! ctrl {
    ($($ch:tt)*) => {
        KeyEvent {
            code: KeyCode::Char($($ch)*),
            modifiers: KeyModifiers::CONTROL,
        }
    };
}

macro_rules! alt {
    ($($ch:tt)*) => {
        KeyEvent {
            code: KeyCode::Char($($ch)*),
            modifiers: KeyModifiers::ALT,
        }
    };
}

pub fn default() -> Keymaps {
    let normal = hashmap!(
        key!('h') => commands::move_char_left as Command,
        key!('j') => commands::move_line_down,
        key!('k') => commands::move_line_up,
        key!('l') => commands::move_char_right,

        KeyEvent {
            code: KeyCode::Left,
            modifiers: KeyModifiers::NONE
        } => commands::move_char_left,
        KeyEvent {
            code: KeyCode::Down,
            modifiers: KeyModifiers::NONE
        } => commands::move_line_down,
        KeyEvent {
            code: KeyCode::Up,
            modifiers: KeyModifiers::NONE
        } => commands::move_line_up,
        KeyEvent {
            code: KeyCode::Right,
            modifiers: KeyModifiers::NONE
        } => commands::move_char_right,

        key!('t') => commands::find_till_char,
        key!('f') => commands::find_next_char,
        key!('T') => commands::till_prev_char,
        key!('F') => commands::find_prev_char,
        // and matching set for select mode (extend)
        //
        key!('r') => commands::replace,
        key!('R') => commands::replace_with_yanked,

        KeyEvent {
            code: KeyCode::Home,
            modifiers: KeyModifiers::NONE
        } => commands::move_line_start,

        KeyEvent {
            code: KeyCode::End,
            modifiers: KeyModifiers::NONE
        } => commands::move_line_end,

        key!('w') => commands::move_next_word_start,
        key!('b') => commands::move_prev_word_start,
        key!('e') => commands::move_next_word_end,

        key!('v') => commands::select_mode,
        key!('g') => commands::goto_mode,
        key!(':') => commands::command_mode,

        key!('i') => commands::insert_mode,
        key!('I') => commands::prepend_to_line,
        key!('a') => commands::append_mode,
        key!('A') => commands::append_to_line,
        key!('o') => commands::open_below,
        key!('O') => commands::open_above,
        // [<space>  ]<space> equivalents too (add blank new line, no edit)


        key!('d') => commands::delete_selection,
        // TODO: also delete without yanking
        key!('c') => commands::change_selection,
        // TODO: also change delete without yanking

        // key!('r') => commands::replace_with_char,

        key!('s') => commands::select_regex,
        alt!('s') => commands::split_selection_on_newline,
        key!('S') => commands::split_selection,
        key!(';') => commands::collapse_selection,
        alt!(';') => commands::flip_selections,
        key!('%') => commands::select_all,
        key!('x') => commands::select_line,
        key!('X') => commands::extend_line,
        // or select mode X?
        // extend_to_whole_line, crop_to_whole_line


        key!('m') => commands::match_brackets,
        // TODO: refactor into
        // key!('m') => commands::select_to_matching,
        // key!('M') => commands::back_select_to_matching,
        // select mode extend equivalents

        // key!('.') => commands::repeat_insert,
        // repeat_select

        // TODO: figure out what key to use
        // key!('[') => commands::expand_selection, ??
        key!('[') => commands::left_bracket_mode,
        key!(']') => commands::right_bracket_mode,

        key!('/') => commands::search,
        // ? for search_reverse
        key!('n') => commands::search_next,
        key!('N') => commands::extend_search_next,
        // N for search_prev
        key!('*') => commands::search_selection,

        key!('u') => commands::undo,
        key!('U') => commands::redo,

        key!('y') => commands::yank,
        // yank_all
        key!('p') => commands::paste_after,
        // paste_all
        key!('P') => commands::paste_before,

        key!('>') => commands::indent,
        key!('<') => commands::unindent,
        key!('=') => commands::format_selections,
        key!('J') => commands::join_selections,
        // TODO: conflicts hover/doc
        key!('K') => commands::keep_selections,
        // TODO: and another method for inverse

        // TODO: clashes with space mode
        key!(' ') => commands::keep_primary_selection,

        // key!('q') => commands::record_macro,
        // key!('Q') => commands::replay_macro,

        // ~ / apostrophe => change case
        // & align selections
        // _ trim selections

        // C / altC = copy (repeat) selections on prev/next lines

        KeyEvent {
            code: KeyCode::Esc,
            modifiers: KeyModifiers::NONE
        } => commands::normal_mode,
        KeyEvent {
            code: KeyCode::PageUp,
            modifiers: KeyModifiers::NONE
        } => commands::page_up,
        ctrl!('b') => commands::page_up,
        KeyEvent {
            code: KeyCode::PageDown,
            modifiers: KeyModifiers::NONE
        } => commands::page_down,
        ctrl!('f') => commands::page_down,
        ctrl!('u') => commands::half_page_up,
        ctrl!('d') => commands::half_page_down,

        ctrl!('w') => commands::window_mode,

        // move under <space>c
        ctrl!('c') => commands::toggle_comments,
        key!('K') => commands::hover,

        // z family for save/restore/combine from/to sels from register

        KeyEvent { // supposedly ctrl!('i') but did not work
            code: KeyCode::Tab,
            modifiers: KeyModifiers::NONE,
        } => commands::jump_forward,
        ctrl!('o') => commands::jump_backward,
        // ctrl!('s') => commands::save_selection,

        key!(' ') => commands::space_mode,
        key!('z') => commands::view_mode,

        key!('"') => commands::select_register,
    );
    // TODO: decide whether we want normal mode to also be select mode (kakoune-like), or whether
    // we keep this separate select mode. More keys can fit into normal mode then, but it's weird
    // because some selection operations can now be done from normal mode, some from select mode.
    let mut select = normal.clone();
    select.extend(
        hashmap!(
            key!('h') => commands::extend_char_left as Command,
            key!('j') => commands::extend_line_down,
            key!('k') => commands::extend_line_up,
            key!('l') => commands::extend_char_right,

            KeyEvent {
                code: KeyCode::Left,
                modifiers: KeyModifiers::NONE
            } => commands::extend_char_left,
            KeyEvent {
                code: KeyCode::Down,
                modifiers: KeyModifiers::NONE
            } => commands::extend_line_down,
            KeyEvent {
                code: KeyCode::Up,
                modifiers: KeyModifiers::NONE
            } => commands::extend_line_up,
            KeyEvent {
                code: KeyCode::Right,
                modifiers: KeyModifiers::NONE
            } => commands::extend_char_right,

            key!('w') => commands::extend_next_word_start,
            key!('b') => commands::extend_prev_word_start,
            key!('e') => commands::extend_next_word_end,

            key!('t') => commands::extend_till_char,
            key!('f') => commands::extend_next_char,

            key!('T') => commands::extend_till_prev_char,
            key!('F') => commands::extend_prev_char,
            KeyEvent {
                code: KeyCode::Home,
                modifiers: KeyModifiers::NONE
            } => commands::extend_line_start,
            KeyEvent {
                code: KeyCode::End,
                modifiers: KeyModifiers::NONE
            } => commands::extend_line_end,
            KeyEvent {
                code: KeyCode::Esc,
                modifiers: KeyModifiers::NONE
            } => commands::exit_select_mode,
        )
        .into_iter(),
    );

    hashmap!(
        // as long as you cast the first item, rust is able to infer the other cases
        // TODO: select could be normal mode with some bindings merged over
        Mode::Normal => normal,
        Mode::Select => select,
        Mode::Insert => hashmap!(
            KeyEvent {
                code: KeyCode::Esc,
                modifiers: KeyModifiers::NONE
            } => commands::normal_mode as Command,
            KeyEvent {
                code: KeyCode::Backspace,
                modifiers: KeyModifiers::NONE
            } => commands::insert::delete_char_backward,
            KeyEvent {
                code: KeyCode::Delete,
                modifiers: KeyModifiers::NONE
            } => commands::insert::delete_char_forward,
            KeyEvent {
                code: KeyCode::Enter,
                modifiers: KeyModifiers::NONE
            } => commands::insert::insert_newline,
            KeyEvent {
                code: KeyCode::Tab,
                modifiers: KeyModifiers::NONE
            } => commands::insert::insert_tab,

            ctrl!('x') => commands::completion,
            ctrl!('w') => commands::insert::delete_word_backward,
        ),
    )
}

// Newtype wrapper over keys to allow toml serialization/parsing
#[derive(Debug, PartialEq, PartialOrd, Clone, Copy, Hash)]
pub struct RepresentableKeyEvent(pub KeyEvent);
impl Display for RepresentableKeyEvent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let Self(key) = self;
        f.write_fmt(format_args!(
            "{}{}{}",
            if key.modifiers.contains(KeyModifiers::SHIFT) {
                "S-"
            } else {
                ""
            },
            if key.modifiers.contains(KeyModifiers::ALT) {
                "A-"
            } else {
                ""
            },
            if key.modifiers.contains(KeyModifiers::CONTROL) {
                "C-"
            } else {
                ""
            },
        ))?;
        match key.code {
            KeyCode::Backspace => f.write_str("Bs")?,
            KeyCode::Enter => f.write_str("Enter")?,
            KeyCode::Left => f.write_str("Left")?,
            KeyCode::Right => f.write_str("Right")?,
            KeyCode::Up => f.write_str("Up")?,
            KeyCode::Down => f.write_str("Down")?,
            KeyCode::Home => f.write_str("Home")?,
            KeyCode::End => f.write_str("End")?,
            KeyCode::PageUp => f.write_str("PageUp")?,
            KeyCode::PageDown => f.write_str("PageDown")?,
            KeyCode::Tab => f.write_str("Tab")?,
            KeyCode::BackTab => f.write_str("BackTab")?,
            KeyCode::Delete => f.write_str("Del")?,
            KeyCode::Insert => f.write_str("Insert")?,
            KeyCode::F(i) => f.write_fmt(format_args!("F{}", i))?,
            KeyCode::Char(c) => f.write_fmt(format_args!("{}", c))?,
            KeyCode::Null => f.write_str("Null")?,
            KeyCode::Esc => f.write_str("Esc")?,
        };
        Ok(())
    }
}

impl FromStr for RepresentableKeyEvent {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut tokens: Vec<_> = s.split('-').collect();
        let code = match tokens.pop().ok_or_else(|| anyhow!("Missing key code"))? {
            "Bs" => KeyCode::Backspace,
            "Enter" => KeyCode::Enter,
            "Left" => KeyCode::Left,
            "Right" => KeyCode::Right,
            "Up" => KeyCode::Down,
            "Home" => KeyCode::Home,
            "End" => KeyCode::End,
            "PageUp" => KeyCode::PageUp,
            "PageDown" => KeyCode::PageDown,
            "Tab" => KeyCode::Tab,
            "BackTab" => KeyCode::BackTab,
            "Del" => KeyCode::Delete,
            "Insert" => KeyCode::Insert,
            "Null" => KeyCode::Null,
            "Esc" => KeyCode::Esc,
            single if single.len() == 1 => KeyCode::Char(single.chars().next().unwrap()),
            function if function.len() > 1 && &function[0..1] == "F" => {
                let function = str::parse::<u8>(&function[1..])?;
                (function > 0 && function < 13)
                    .then(|| KeyCode::F(function))
                    .ok_or_else(|| anyhow!("Invalid function key '{}'", function))?
            }
            invalid => return Err(anyhow!("Invalid key code '{}'", invalid)),
        };

        let mut modifiers = KeyModifiers::empty();
        for token in tokens {
            let flag = match token {
                "S" => KeyModifiers::SHIFT,
                "A" => KeyModifiers::ALT,
                "C" => KeyModifiers::CONTROL,
                _ => return Err(anyhow!("Invalid key modifier '{}-'", token)),
            };

            if modifiers.contains(flag) {
                return Err(anyhow!("Repeated key modifier '{}-'", token));
            }
            modifiers.insert(flag);
        }

        Ok(RepresentableKeyEvent(KeyEvent { code, modifiers }))
    }
}

pub fn parse_remaps(remaps: &str) -> Result<Remaps> {
    type TomlCompatibleRemaps = HashMap<String, HashMap<String, String>>;
    let toml_remaps: TomlCompatibleRemaps = toml::from_str(remaps)?;
    let mut remaps = Remaps::new();

    for (mode, map) in toml_remaps {
        let mode = Mode::from_str(&mode)?;
        let mut remap = Remap::new();

        for (source_key, target_key) in map {
            let source_key = str::parse::<RepresentableKeyEvent>(&source_key)?;
            let target_key = str::parse::<RepresentableKeyEvent>(&target_key)?;
            remap.insert(source_key.0, target_key.0);
        }
        remaps.insert(mode, remap);
    }
    Ok(remaps)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn parsing_remaps_file() {
        let sample_remaps = "\
            [Insert]\n\
            y = \"x\"\n\
            S-C-a = \"F12\"\n\

            [Normal]
            A-F12 = \"S-C-w\"\n\
        ";

        let parsed = parse_remaps(sample_remaps).unwrap();
        assert_eq!(
            parsed,
            hashmap!(
                Mode::Insert => hashmap!(
                    KeyEvent { code: KeyCode::Char('y'), modifiers: KeyModifiers::NONE }
                        => KeyEvent { code: KeyCode::Char('x'), modifiers: KeyModifiers::NONE },
                    KeyEvent { code: KeyCode::Char('a'), modifiers: KeyModifiers::SHIFT | KeyModifiers::CONTROL }
                        => KeyEvent { code: KeyCode::F(12), modifiers: KeyModifiers::NONE },
                ),
                Mode::Normal => hashmap!(
                    KeyEvent { code: KeyCode::F(12), modifiers: KeyModifiers::ALT }
                        => KeyEvent { code: KeyCode::Char('w'), modifiers: KeyModifiers::SHIFT | KeyModifiers::CONTROL },
                )
            )
        )
    }

    #[test]
    fn parsing_unmodified_keys() {
        assert_eq!(
            str::parse::<RepresentableKeyEvent>("Bs").unwrap(),
            RepresentableKeyEvent(KeyEvent {
                code: KeyCode::Backspace,
                modifiers: KeyModifiers::NONE
            })
        );

        assert_eq!(
            str::parse::<RepresentableKeyEvent>("Left").unwrap(),
            RepresentableKeyEvent(KeyEvent {
                code: KeyCode::Left,
                modifiers: KeyModifiers::NONE
            })
        );

        assert_eq!(
            str::parse::<RepresentableKeyEvent>(",").unwrap(),
            RepresentableKeyEvent(KeyEvent {
                code: KeyCode::Char(','),
                modifiers: KeyModifiers::NONE
            })
        );

        assert_eq!(
            str::parse::<RepresentableKeyEvent>("w").unwrap(),
            RepresentableKeyEvent(KeyEvent {
                code: KeyCode::Char('w'),
                modifiers: KeyModifiers::NONE
            })
        );

        assert_eq!(
            str::parse::<RepresentableKeyEvent>("F12").unwrap(),
            RepresentableKeyEvent(KeyEvent {
                code: KeyCode::F(12),
                modifiers: KeyModifiers::NONE
            })
        );
    }

    fn parsing_modified_keys() {
        assert_eq!(
            str::parse::<RepresentableKeyEvent>("S-Bs").unwrap(),
            RepresentableKeyEvent(KeyEvent {
                code: KeyCode::Backspace,
                modifiers: KeyModifiers::SHIFT
            })
        );

        assert_eq!(
            str::parse::<RepresentableKeyEvent>("C-A-S-F12").unwrap(),
            RepresentableKeyEvent(KeyEvent {
                code: KeyCode::F(12),
                modifiers: KeyModifiers::SHIFT | KeyModifiers::CONTROL | KeyModifiers::ALT
            })
        );
        assert_eq!(
            str::parse::<RepresentableKeyEvent>("S-C-2").unwrap(),
            RepresentableKeyEvent(KeyEvent {
                code: KeyCode::F(2),
                modifiers: KeyModifiers::SHIFT | KeyModifiers::CONTROL
            })
        );
    }

    #[test]
    fn parsing_nonsensical_keys_fails() {
        assert!(str::parse::<RepresentableKeyEvent>("F13").is_err());
        assert!(str::parse::<RepresentableKeyEvent>("F0").is_err());
        assert!(str::parse::<RepresentableKeyEvent>("aaa").is_err());
        assert!(str::parse::<RepresentableKeyEvent>("S-S-a").is_err());
        assert!(str::parse::<RepresentableKeyEvent>("C-A-S-C-1").is_err());
        assert!(str::parse::<RepresentableKeyEvent>("FU").is_err());
        assert!(str::parse::<RepresentableKeyEvent>("123").is_err());
    }
}
