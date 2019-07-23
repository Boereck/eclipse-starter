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
//! let args = vec!["-application", "foo.bar", "-clean", "-vmargs", "-Dfoo=bar", "-Dfizz=buzz"];
//! let mut parser = eclipse_common::arg_parser::Parser::new();
//! 
//! let clean = parser.add_flag("-clean");
//! let vmargs = parser.add_list("-vmargs");
//! let application = parser.add_option("-application");
//! 
//! let mut parse_result = parser.parse(args);
//! 
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

#[derive(Eq,PartialEq,Hash,Debug,Clone,Copy)]
pub struct OptionId(usize);

#[derive(Eq,PartialEq,Hash,Debug,Clone,Copy)]
pub struct OptionalOptionId(usize);

#[derive(Eq,PartialEq,Hash,Debug,Clone,Copy)]
pub struct FlagId(usize);

#[derive(Eq,PartialEq,Hash,Debug,Clone,Copy)]
pub struct ListId(usize);


enum ArgumentType {
    Option(OptionId),
    OptionalOption(OptionalOptionId),
    Flag(FlagId),
    List(ListId),
}

pub struct Parser {
    args_by_name: HashMap<&'static str, ArgumentType>,
    next_index : usize
}

impl Parser {
    
    pub fn new() -> Self {
        Parser {
            next_index: 0,
            args_by_name : HashMap::new()
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
    pub fn add_optionaloption(&mut self, name: &'static str) -> OptionalOptionId {
        let result = OptionalOptionId(self.next_index);
        self.args_by_name.insert(name, ArgumentType::OptionalOption(result));
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
    /// this rule. There can also just be one list rule, calling this method again will overwrite the 
    /// previously added list rule. 
    /// After calling `parse`, the method `ParseResult::take_list` returns an `Option` which holds the
    /// parsed list, if the parameter was present..
    /// The returned `ListId` has to be passed to `take_list` for this matter.
    pub fn add_list(&mut self, name: &'static str) -> ListId {
        let result = ListId(self.next_index);
        self.args_by_name.insert(name, ArgumentType::List(result));
        self.next_index += 1;
        result
    }

    pub fn parse<'a,'b>(&'a self, args: impl IntoIterator<Item = &'b str>) -> ParseResult {
        let mut found_flags = HashSet::<FlagId>::new();
        let mut found_options = HashMap::<OptionId,String>::new();
        let mut found_optionaloptions = HashMap::<OptionalOptionId,Option<String>>::new();
        let mut found_list: Option<(ListId,Vec<String>)> = None;
        let mut iter = args.into_iter().peekable();
        
        while let Some(ref arg_name) = iter.next() {
            let key : &str = arg_name;
            if let Some(arg) = self.args_by_name.get(key) {
                use ArgumentType::*;
                match arg {
                    Flag(id) => {
                        found_flags.insert(*id);
                    },
                    Option(id) => {
                        if let Some(arg_value) = iter.peek() {
                            let key : &str = arg_value;
                            if !self.args_by_name.contains_key(key) {
                                let opt_value = iter.next().unwrap();
                                found_options.insert(*id, opt_value.to_owned());
                            } else {
                                // TODO: some kind of error!
                            }
                        }
                    },
                    OptionalOption(id) => {
                        if let Some(arg_value) = iter.peek() {
                            let key : &str = arg_value;
                            let opt_value = if self.args_by_name.contains_key(key) {
                                None
                            } else {
                                Some(iter.next().unwrap().to_owned())
                            };
                            found_optionaloptions.insert(*id, opt_value);
                        }
                    },
                    List(id) => {
                        let mut list = Vec::new();
                        while let Some(list_entry) = iter.next() {
                            list.push(list_entry.to_owned())
                        }
                        found_list = Some((*id, list));
                    }
                }
            }
        }
        ParseResult {
            flags : found_flags,
            options: found_options,
            optionaloptions: found_optionaloptions,
            list: found_list,
        }
    }
}

pub struct ParseResult {
    // TODO list of parse errors?
    flags: HashSet<FlagId>,
    options: HashMap<OptionId,String>,
    optionaloptions: HashMap<OptionalOptionId,Option<String>>,
    list: Option<(ListId,Vec<String>)>,
}

impl ParseResult {

    pub fn take_option(&mut self, arg : OptionId)-> Option<String> {
        self.options.remove(&arg)
    }

    pub fn take_flag(&mut self, arg : FlagId) -> bool {
        self.flags.remove(&arg)
    }

    pub fn take_optional_option(&mut self, arg : OptionalOptionId) -> Option<Option<String>> {
        self.optionaloptions.remove_entry(&arg).map(|(_,v)| v)
    }

    
    pub fn take_list(&mut self, arg : ListId) -> Option<Vec<String>> {
        self.list.take()
            .filter(|(list_arg,_)| list_arg == &arg)
            .map(|(_, list)| list)
    }
}



#[cfg(test)]
mod parser_test {
    
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
        let args = ["bla","-vmargs", "-Dfoo=bar", "-Dfizz=buzz"];
        let iter = args.iter().copied();
        
        let mut parser = super::Parser::new();
        let vmargs = parser.add_list("-vmargs");
        let mut parse_result = parser.parse(iter);
        
        let vmargs_value = parse_result.take_list(vmargs);
        assert_eq!(vmargs_value.unwrap(), vec!["-Dfoo=bar".to_string(), "-Dfizz=buzz".to_string()]);
    }
    #[test]
    fn test_parsing_empty_list() {
        let args = ["bla","-vmargs"];
        let iter = args.iter().copied();
        
        let mut parser = super::Parser::new();
        let vmargs = parser.add_list("-vmargs");
        let mut parse_result = parser.parse(iter);
        
        let vmargs_value = parse_result.take_list(vmargs);
        assert!(vmargs_value.unwrap().is_empty());
    }

    #[test]
    fn test_parsing_flag() {
        let args = vec!["-clean"];
        let iter = args.iter().copied();
        
        let mut parser = super::Parser::new();
        let clean = parser.add_flag("-clean");
        let mut parse_result = parser.parse(iter);
        
        let clean_value = parse_result.take_flag(clean);
        assert!(clean_value);
    }
    
}
