// Std
use std::collections::hash_map::{Entry, Iter, Keys};
use std::ffi::OsStr;
use std::mem;

// Internal
use {Arg, ArgSettings, ArgMatches, SubCommand};
// use parsing::AnyArg;
use matched::MatchedArg;

#[doc(hidden)]
#[allow(missing_debug_implementations)]
pub struct ArgMatcher<'a>(pub ArgMatches<'a>);

impl<'a> Default for ArgMatcher<'a> {
    fn default() -> Self { ArgMatcher(ArgMatches::default()) }
}

impl<'a> ArgMatcher<'a> {
    pub fn new() -> Self { ArgMatcher::default() }

    pub fn propagate(&mut self, arg: &'a str) {
        debugln!("ArgMatcher::propagate: arg={}", arg);
        let vals: Vec<_> = if let Some(ma) = self.get(arg) {
            ma.vals.clone()
        } else {
            debugln!("ArgMatcher::propagate: arg wasn't used");
            return;
        };
        if let Some(ref mut sc) = self.0.subcommand {
            {
                let sma = (*sc).matches.args.entry(arg).or_insert_with(|| {
                    let mut gma = MatchedArg::new();
                    gma.occurs += 1;
                    gma.vals = vals.clone();
                    gma
                });
                if sma.vals.is_empty() {
                    sma.vals = vals.clone();
                }
            }
            let mut am = ArgMatcher(mem::replace(&mut sc.matches, ArgMatches::new()));
            am.propagate(arg);
            mem::swap(&mut am.0, &mut sc.matches);
        } else {
            debugln!("ArgMatcher::propagate: Subcommand wasn't used");
        }
    }

    pub fn get_mut(&mut self, arg: &str) -> Option<&mut MatchedArg> { self.0.args.get_mut(arg) }

    pub fn get(&self, arg: &str) -> Option<&MatchedArg> { self.0.args.get(arg) }

    pub fn remove(&mut self, arg: &str) { self.0.args.remove(arg); }

    pub fn remove_all(&mut self, args: &[&str]) {
        for &arg in args {
            self.0.args.remove(arg);
        }
    }

    pub fn insert(&mut self, name: &'a str) { self.0.args.insert(name, MatchedArg::new()); }

    pub fn contains(&self, arg: &str) -> bool { self.0.args.contains_key(arg) }

    pub fn is_empty(&self) -> bool { self.0.args.is_empty() }

    pub fn usage(&mut self, usage: String) { self.0.usage = Some(usage); }

    pub fn arg_names(&self) -> Keys<&'a str, MatchedArg> { self.0.args.keys() }

    pub fn entry(&mut self, arg: &'a str) -> Entry<&'a str, MatchedArg> { self.0.args.entry(arg) }

    pub fn subcommand(&mut self, sc: SubCommand<'a>) { self.0.subcommand = Some(Box::new(sc)); }

    pub fn subcommand_name(&self) -> Option<&str> { self.0.subcommand_name() }

    pub fn iter(&self) -> Iter<&str, MatchedArg> { self.0.args.iter() }

    pub fn inc_occurrence_of(&mut self, arg: &'a str) {
        debugln!("ArgMatcher::inc_occurrence_of: arg={}", arg);
        if let Some(a) = self.get_mut(arg) {
            a.occurs += 1;
            return;
        }
        debugln!("ArgMatcher::inc_occurrence_of: first instance");
        self.insert(arg);
    }

    pub fn inc_occurrences_of(&mut self, args: &[&'a str]) {
        debugln!("ArgMatcher::inc_occurrences_of: args={:?}", args);
        for arg in args {
            self.inc_occurrence_of(arg);
        }
    }

    pub fn add_val_to(&mut self, arg: &'a str, val: &OsStr) {
        let ma = self.entry(arg).or_insert(MatchedArg {
            occurs: 0,
            vals: Vec::with_capacity(1),
        });
        // let len = ma.vals.len() + 1;
        ma.vals.push(val.to_owned());
    }

    pub fn needs_more_vals<'b>(&self, o: &Arg) -> bool
    {
        debugln!("ArgMatcher::needs_more_vals: o={}", o.name);
        if let Some(ma) = self.get(o.name) {
            if let Some(num) = o.number_of_values {
                debugln!("ArgMatcher::needs_more_vals: num_vals...{}", num);
                return if o.is_set(ArgSettings::Multiple) {
                    (ma.vals.len() % num) != 0
                } else {
                    num != ma.vals.len()
                };
            } else if let Some(num) = o.max_values {
                debugln!("ArgMatcher::needs_more_vals: max_vals...{}", num);
                return !(ma.vals.len() > num);
            } else if o.min_values.is_some() {
                debugln!("ArgMatcher::needs_more_vals: min_vals...true");
                return true;
            }
            return o.is_set(ArgSettings::Multiple);
        }
        true
    }
}

impl<'a> Into<ArgMatches<'a>> for ArgMatcher<'a> {
    fn into(self) -> ArgMatches<'a> { self.0 }
}