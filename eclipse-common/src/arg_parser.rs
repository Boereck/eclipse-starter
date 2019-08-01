/*******************************************************************************
 * Copyright (c) 2019 Fraunhofer FOKUS and others.
 *
 * This program and the accompanying materials
 * are made available under the terms of the Eclipse Public License 2.0
 * which accompanies this distribution, and is available at
 * https://www.eclipse.org/legal/epl-2.0/
 *
 * SPDX-License-Identifier: EPL-2.0
 *
 * Contributors:
 *     Max Bureck (Fraunhofer FOKUS)
 *******************************************************************************/

//! This module provides functionality to parse command line parameters.
//! First a `Parser` needs to be created using `Parser::new()`, then
//! Parsing rules need to be added using the `add_*` methods on the parser.
//! To start parsing, the `parse` method needs to be called with an iterator
//! over the parameters.
//! To get the results for parsing rules, the id values returned from the `add_*`
//! methods need to be passed to the `take_*` methods on the `ParseResult`.
//!
//! Example:
//! ```
//! use eclipse_common::arg_parser::{Parser, ListParseStyle};
//! 
//! // arguments to parse
//! let args = vec!["-application", "foo.bar", "-clean", "-vmargs", "-Dfoo=bar", "-Dfizz=buzz"];
//!
//! let mut parser = Parser::new();
//!
//! // define parsing rules
//! let clean = parser.add_flag("-clean");
//! let vmargs = parser.add_list("-vmargs", ListParseStyle::AllRemaining);
//! let application = parser.add_option("-application");
//!
//! let mut parse_result = parser.parse(args);
//!
//! // get parse results
//! let clean_value = parse_result.take_flag(clean);
//! assert!(clean_value);
//!
//! let vmargs_value = parse_result.take_list(vmargs);
//! let expected_vmargs = vec!["-Dfoo=bar".to_string(), "-Dfizz=buzz".to_string()];
//! assert_eq!(vmargs_value.unwrap(), expected_vmargs);
//!
//! let application_value = parse_result.take_option(application);
//! assert_eq!(application_value.unwrap(), "foo.bar");
//! ```

use std::collections::{HashMap, HashSet};

#[derive(Eq, PartialEq, Hash, Debug, Clone, Copy)]
pub struct OptionId(usize);

#[derive(Eq, PartialEq, Hash, Debug, Clone, Copy)]
pub struct OptionalOptionId(usize);

#[derive(Eq, PartialEq, Hash, Debug, Clone, Copy)]
pub struct FlagId(usize);

#[derive(Eq, PartialEq, Hash, Debug, Clone, Copy)]
pub struct ListId(usize);

/// This type represents two ways of parsing a list parameter.
pub enum ListParseStyle {
    /// This parsing variant aggregates all remaining parameters
    /// into the resulting list of values for a given argument.
    AllRemaining,

    /// This variant only takes arguments until one argument is found
    /// starting with a `-`. Parsing rules will be applied again to this
    /// argument.
    UntilDashPrefix,
}

/// This type represents the parsing result for an optional option.
/// Such an option can either not be set (`NotSet`) or is set, but
/// has no following associated value (`SetNoVal`), or has one additional
/// argument (`Set`).
#[derive(Eq, PartialEq, Debug)]
pub enum OptionalParam {
    NotSet,
    SetNoVal,
    Set(String),
}

impl OptionalParam {
    
    /// Returns `false` if self is `OptionalParam::NotSet`, `true` otherwise
    pub fn is_set(&self) -> bool {
        match self {
            OptionalParam::NotSet => false,
            _ => true,
        }
    }
}

impl Default for OptionalParam {
    fn default() -> Self {
        OptionalParam::NotSet
    }
}

enum ArgumentType {
    Option(OptionId),
    OptionalOption(OptionalOptionId),
    Flag(FlagId),
    List(ListId, ListParseStyle),
}

/// A simple parser for command line arguments. Four types of parsing
/// rules can be registered on the parser, before starting the parsing
/// using the `parse` function:
/// * Flags: call `add_flag` for parameters that do not have a following
///   value, the parse result will be a `bool` that is true if the flag was found
/// * Options: call `add_option` for parameters that have exactly one folloing
///   value after the parameter. The parse result will be an `Option<String>`
///   wich will hold a value if the parameter was found.
/// * Optional option
pub struct Parser {
    args_by_name: HashMap<&'static str, ArgumentType>,
    next_index: usize,
}

impl Parser {
    pub fn new() -> Self {
        Parser {
            next_index: 0,
            args_by_name: HashMap::new(),
        }
    }

    /// Adds the parsing rule for a parameter specified via `name` that is followed by a value.
    /// After calling `parse`, the value can be obtained via the `ParseResult::take_option`
    /// method by providing the `OptionId` returned by this method.
    pub fn add_option(&mut self, name: &'static str) -> OptionId {
        let result = OptionId(self.next_index);
        self.args_by_name.insert(name, ArgumentType::Option(result));
        self.next_index += 1;
        result
    }

    /// Adds the parsing rule for a parameter specified via `name` that is optionally followed by a value.
    /// After calling `parse`, the value can be obtained via the `ParseResult::take_optionaloption`
    /// method by providing the `OptionId` returned by this method.
    pub fn add_optional_option(&mut self, name: &'static str) -> OptionalOptionId {
        let result = OptionalOptionId(self.next_index);
        self.args_by_name
            .insert(name, ArgumentType::OptionalOption(result));
        self.next_index += 1;
        result
    }

    /// Adds the parsing rule for a parameter specified via `name` that is _not_ followed by a value.
    /// After calling `parse`, the method `ParseResult::take_flag` returns `true` if the flag was present.
    /// The returned `FlagId` has to be passed to `take_flag` for this matter.
    pub fn add_flag(&mut self, name: &'static str) -> FlagId {
        let result = FlagId(self.next_index);
        self.next_index += 1;
        self.args_by_name.insert(name, ArgumentType::Flag(result));
        result
    }

    /// Adds the parsing rule for a parameter specified via `name` that is followed by a list of values.
    /// Note that this parameter has to be the last, because all further parameters will be consumed by
    /// There are two flavors of parsing of the list rule:
    /// * When passing `ListParseStyle::AllRemaining` this rule will take all remainung arguments into
    ///   the list. After calling `parse`, the method `ParseResult::take_list` returns an `Option` which
    ///   holds the parsed list, if the parameter was present..
    /// * When passing `ListParseStyle::UntilDashPrefix` all following element that do not start with a
    ///   "-" prefix are added to the list. Different parameters may follow.
    ///
    /// The returned `ListId` has to be passed to `take_list` for retrieving the
    /// parsed elements.
    /// There can also just be one list rule, calling this method again will overwrite the
    /// previously added list rule.
    pub fn add_list(&mut self, name: &'static str, style: ListParseStyle) -> ListId {
        let result = ListId(self.next_index);
        self.args_by_name
            .insert(name, ArgumentType::List(result, style));
        self.next_index += 1;
        result
    }

    /// Parses the given `args` according to the rules previously registered on the
    /// parser via `add_*` functions. Returns the parsing results which can be queried
    /// for results using the `take_*` functions.
    pub fn parse<'a, 'b>(&'a self, args: impl IntoIterator<Item = &'b str>) -> ParseResult {
        let mut found_flags = HashSet::<FlagId>::new();
        let mut found_options = HashMap::<OptionId, String>::new();
        let mut found_optionaloptions = HashMap::<OptionalOptionId, OptionalParam>::new();
        let mut found_list: Option<(ListId, Vec<String>)> = None;
        let mut iter = args.into_iter().peekable();
        'parse_loop: while let Some(ref arg_name) = iter.next() {
            let key: &str = arg_name;
            // is this an argument to parse?
            let arg = match self.args_by_name.get(key) {
                Some(arg) => arg,
                None => continue,
            };
            use ArgumentType::*;
            // apply parsing rule
            match arg {
                Flag(id) => {
                    found_flags.insert(*id);
                }
                Option(id) => {
                    // TODO: flatten, this nests too deep
                    if let Some(arg_value) = iter.peek() {
                        let key: &str = arg_value;
                        // TODO: don't fail, just warn
                        if !self.args_by_name.contains_key(key) {
                            iter.next();
                            let opt_value = key.to_owned();
                            found_options.insert(*id, opt_value.to_owned());
                        } else {
                            // TODO: some kind of error!
                        }
                    }
                }
                OptionalOption(id) => {
                    // TODO: flatten, this nests too deep
                    if let Some(arg_value) = iter.peek() {
                        let key: &str = arg_value;
                        let opt_value = if self.args_by_name.contains_key(key) {
                            OptionalParam::SetNoVal
                        } else {
                            iter.next();
                            OptionalParam::Set(key.to_owned())
                        };
                        found_optionaloptions.insert(*id, opt_value);
                    }
                }
                List(id, style) => {
                    // TODO: maybe extract to function
                    match style {
                        ListParseStyle::AllRemaining => {
                            let list: Vec<String> = iter.map(str::to_owned).collect();
                            found_list = Some((*id, list));
                            break 'parse_loop;
                        }
                        ListParseStyle::UntilDashPrefix => {
                            let mut list = Vec::<String>::new();
                            while iter.peek().map_or(false, |s| !s.starts_with('-')) {
                                // unwrapping is safe, since peek already determined
                                // a next element exists
                                let s = iter.next().unwrap();
                                list.push(s.to_owned());
                            }
                            found_list = Some((*id, list));
                        }
                    }
                }
            }
        }
        ParseResult {
            flags: found_flags,
            options: found_options,
            optionaloptions: found_optionaloptions,
            list: found_list,
        }
    }
}

impl Default for Parser {
    fn default() -> Parser {
        Parser::new()
    }
}

pub struct ParseResult {
    // TODO list of parse errors?
    flags: HashSet<FlagId>,
    options: HashMap<OptionId, String>,
    optionaloptions: HashMap<OptionalOptionId, OptionalParam>,
    list: Option<(ListId, Vec<String>)>,
}

impl ParseResult {
    pub fn take_option(&mut self, arg: OptionId) -> Option<String> {
        self.options.remove(&arg)
    }

    pub fn take_flag(&mut self, arg: FlagId) -> bool {
        self.flags.remove(&arg)
    }

    pub fn take_optional_option(&mut self, arg: OptionalOptionId) -> OptionalParam {
        self.optionaloptions
            .remove_entry(&arg)
            .map(|(_, v)| v)
            .unwrap_or_default()
    }

    pub fn take_list(&mut self, arg: ListId) -> Option<Vec<String>> {
        self.list
            .take()
            .filter(|(list_arg, _)| list_arg == &arg)
            .map(|(_, list)| list)
    }
}

#[cfg(test)]
mod parser_test {
    use super::{ListParseStyle, OptionalParam};
    #[test]
    fn test_replace() {
        let mut o = Some(4);
        o.get_or_insert(5);
        eprintln!("{:?}", o);
    }
    #[test]
    fn test_parsing_option() {
        let args = ["-application", "foo.bar"];
        let mut parser = super::Parser::new();
        let application = parser.add_option("-application");
        let mut parse_result = parser.parse(args.iter().copied());
        let application_value = parse_result.take_option(application);
        assert_eq!(application_value.unwrap(), "foo.bar");
    }

    #[test]
    fn test_parsing_list() {
        let args = ["bla", "-vmargs", "-Dfoo=bar", "-Dfizz=buzz"];
        let iter = args.iter().copied();
        let mut parser = super::Parser::new();
        let vmargs = parser.add_list("-vmargs", ListParseStyle::AllRemaining);
        let mut parse_result = parser.parse(iter);
        let vmargs_value = parse_result.take_list(vmargs);
        assert_eq!(
            vmargs_value.unwrap(),
            vec!["-Dfoo=bar".to_string(), "-Dfizz=buzz".to_string()]
        );
    }
    #[test]
    fn test_parsing_empty_list() {
        let args = ["bla", "-vmargs"];
        let iter = args.iter().copied();
        let mut parser = super::Parser::new();
        let vmargs = parser.add_list("-vmargs", ListParseStyle::AllRemaining);
        let mut parse_result = parser.parse(iter);
        let vmargs_value = parse_result.take_list(vmargs);
        assert!(vmargs_value.unwrap().is_empty());
    }

    #[test]
    fn test_parsing_flag() {
        let args = vec!["-clean", "-vmargs"];
        let iter = args.iter().copied();
        let mut parser = super::Parser::new();
        let clean = parser.add_flag("-clean");
        let mut parse_result = parser.parse(iter);
        let clean_value = parse_result.take_flag(clean);
        assert!(clean_value);
    }

    #[test]
    fn test_parsing_optional_option_set_no_val() {
        let args = vec!["-clean", "-console", "-application", "foo.bar"];
        let iter = args.iter().copied();
        let mut parser = super::Parser::new();
        let clean = parser.add_option("-application");
        let console = parser.add_optional_option("-console");
        let mut parse_result = parser.parse(iter);
        let console_value = parse_result.take_optional_option(console);
        assert_eq!(console_value, OptionalParam::SetNoVal);
    }

    #[test]
    fn test_parsing_optional_option_set_not_set() {
        let args = vec!["-clean", "-application", "foo.bar"];
        let iter = args.iter().copied();
        let mut parser = super::Parser::new();
        let clean = parser.add_option("-application");
        let console = parser.add_optional_option("-console");
        let mut parse_result = parser.parse(iter);
        let console_value = parse_result.take_optional_option(console);
        assert_eq!(console_value, OptionalParam::NotSet);
    }

    #[test]
    fn test_parsing_optional_option_set() {
        let args = vec!["-clean", "-console", "localhost:9090", "-application", "foo.bar"];
        let iter = args.iter().copied();
        let mut parser = super::Parser::new();
        let clean = parser.add_option("-application");
        let console = parser.add_optional_option("-console");
        let mut parse_result = parser.parse(iter);
        let console_value = parse_result.take_optional_option(console);
        assert_eq!(console_value, OptionalParam::Set("localhost:9090".into()));
    }
}
