use std::collections::{BTreeMap, HashSet};
use std::cmp;
use std::io::Write;

use gimli;

use {Options, Result, Sort};
use file::FileHash;
use function::{Function, FunctionOffset};
use print::{DiffState, Print, PrintState, SortList};
use range::{Range, RangeList};
use types::{Type, TypeKind, TypeOffset};
use variable::{Variable, VariableOffset};

#[derive(Debug, Default)]
pub(crate) struct Unit<'input> {
    pub dir: Option<&'input [u8]>,
    pub name: Option<&'input [u8]>,
    pub language: Option<gimli::DwLang>,
    pub address_size: Option<u64>,
    pub low_pc: Option<u64>,
    pub ranges: RangeList,
    pub types: BTreeMap<TypeOffset, Type<'input>>,
    pub functions: BTreeMap<FunctionOffset, Function<'input>>,
    pub variables: BTreeMap<VariableOffset, Variable<'input>>,
}

impl<'input> Unit<'input> {
    // Does not include unknown ranges.
    pub fn ranges(&self, hash: &FileHash) -> RangeList {
        let mut ranges = RangeList::default();
        for function in self.functions.values() {
            if let Some(range) = function.address() {
                ranges.push(range);
            }
        }
        for variable in self.variables.values() {
            if let Some(range) = variable.address(hash) {
                ranges.push(range);
            }
        }
        ranges.sort();
        ranges
    }

    pub fn unknown_ranges(&self, hash: &FileHash) -> RangeList {
        let mut ranges = RangeList::default();
        for range in self.ranges.list() {
            ranges.push(*range);
        }
        ranges.sort();
        ranges.subtract(&self.ranges(hash))
    }

    fn size(&self, hash: &FileHash) -> u64 {
        // TODO: account for padding and overlap between functions and variables?
        self.function_size() + self.variable_size(hash)
    }

    pub fn function_size(&self) -> u64 {
        let mut ranges = RangeList::default();
        for function in self.functions.values() {
            if let Some(range) = function.address() {
                ranges.push(range);
            }
        }
        ranges.sort();
        ranges.size()
    }

    pub fn variable_size(&self, hash: &FileHash) -> u64 {
        let mut ranges = RangeList::default();
        for variable in self.variables.values() {
            if let Some(range) = variable.address(hash) {
                ranges.push(range);
            }
        }
        ranges.sort();
        ranges.size()
    }

    /// The offsets of types that should be printed inline.
    fn inline_types(&self, hash: &FileHash) -> HashSet<usize> {
        let mut inline_types = HashSet::new();
        for ty in self.types.values() {
            // Assume all anonymous types are inline. We don't actually check
            // that they will be inline, but in future we could (eg for TypeDefs).
            // TODO: is this a valid assumption?
            if ty.is_anon() {
                inline_types.insert(ty.offset.0);
            }

            // Find all inline members.
            ty.visit_members(&mut |t| {
                if t.is_inline(hash) {
                    if let Some(offset) = t.ty {
                        inline_types.insert(offset.0);
                    }
                }
            });
        }
        inline_types
    }

    fn print_ref(&self, w: &mut Write) -> Result<()> {
        match self.name {
            Some(name) => write!(w, "{}", String::from_utf8_lossy(name))?,
            None => write!(w, "<anon>")?,
        }
        Ok(())
    }

    pub fn print(&self, state: &mut PrintState) -> Result<()> {
        let options = state.options();
        if options.category_unit {
            state.line(|w, _state| {
                write!(w, "unit ")?;
                self.print_ref(w)
            })?;
            state.indent(|state| {
                let unknown_ranges = self.unknown_ranges(state.hash());

                if options.print_unit_address {
                    let ranges = self.ranges(state.hash());
                    if ranges.list().len() > 1 {
                        state.list("addresses", &(), ranges.list())?;
                    } else {
                        let range = ranges.list().first().cloned();
                        state.line_option(|w, _state| self.print_address(w, range))?;
                    }

                    state.list("unknown addresses", &(), unknown_ranges.list())?;
                }

                let fn_size = self.function_size();
                if fn_size != 0 {
                    state.line_u64("fn size", fn_size)?;
                }

                let var_size = self.variable_size(state.hash());
                if var_size != 0 {
                    state.line_u64("var size", var_size)?;
                }

                let unknown_size = unknown_ranges.size();
                if unknown_size != 0 {
                    state.line_u64("unknown size", unknown_size)?;
                }
                Ok(())
            })?;
            state.line_break()?;
        }

        if options.category_type {
            let mut types = self.filter_types(state.hash(), options, false);
            state.sort_list(self, &mut *types)?;
        }
        if options.category_function {
            state.sort_list(self, &mut *self.filter_functions(options))?;
        }
        if options.category_variable {
            state.sort_list(self, &mut *self.filter_variables(options))?;
        }
        Ok(())
    }

    pub fn diff(state: &mut DiffState, unit_a: &Unit, unit_b: &Unit) -> Result<()> {
        let options = state.options();
        if options.category_unit {
            state.line(unit_a, unit_b, |w, _state, unit| {
                write!(w, "unit ")?;
                unit.print_ref(w)
            })?;
            state.indent(|state| {
                let unknown_ranges_a = unit_a.unknown_ranges(state.hash_a());
                let unknown_ranges_b = unit_b.unknown_ranges(state.hash_b());

                if options.print_unit_address {
                    let ranges_a = unit_a.ranges(state.hash_a());
                    let ranges_b = unit_b.ranges(state.hash_b());
                    if ranges_a.list().len() > 1 || ranges_a.list().len() > 1 {
                        state.ord_list("addresses", &(), ranges_a.list(), &(), ranges_b.list())?;
                    } else {
                        let range_a = ranges_a.list().first().cloned();
                        let range_b = ranges_b.list().first().cloned();
                        state.line_option(
                            (unit_a, range_a),
                            (unit_b, range_b),
                            |w, _state, (unit, range)| unit.print_address(w, range),
                        )?;
                    }

                    state.ord_list(
                        "unknown addresses",
                        &(),
                        unknown_ranges_a.list(),
                        &(),
                        unknown_ranges_b.list(),
                    )?;
                }

                let fn_size_a = unit_a.function_size();
                let fn_size_b = unit_b.function_size();
                if fn_size_a != 0 || fn_size_b != 0 {
                    state.line_u64("fn size", fn_size_a, fn_size_b)?;
                }

                let var_size_a = unit_a.variable_size(state.hash_a());
                let var_size_b = unit_b.variable_size(state.hash_b());
                if var_size_a != 0 || var_size_b != 0 {
                    state.line_u64("var size", var_size_a, var_size_b)?;
                }

                let unknown_size_a = unknown_ranges_a.size();
                let unknown_size_b = unknown_ranges_b.size();
                if unknown_size_a != 0 || unknown_size_b != 0 {
                    state.line_u64("unknown size", unknown_size_a, unknown_size_b)?;
                }
                Ok(())
            })?;
            state.line_break()?;
        }

        if options.category_type {
            let mut types_a = unit_a.filter_types(state.hash_a(), options, true);
            let mut types_b = unit_b.filter_types(state.hash_b(), options, true);
            state.sort_list(unit_a, &mut *types_a, unit_b, &mut *types_b)?;
        }
        if options.category_function {
            state.sort_list(
                unit_a,
                &mut *unit_a.filter_functions(options),
                unit_b,
                &mut *unit_b.filter_functions(options),
            )?;
        }
        if options.category_variable {
            state.sort_list(
                unit_a,
                &mut *unit_a.filter_variables(options),
                unit_b,
                &mut *unit_b.filter_variables(options),
            )?;
        }
        Ok(())
    }

    fn print_address(&self, w: &mut Write, range: Option<Range>) -> Result<()> {
        if let Some(range) = range {
            write!(w, "address: ")?;
            range.print_address(w)?;
        } else if let Some(low_pc) = self.low_pc {
            write!(w, "address: 0x{:x}", low_pc)?;
        }
        Ok(())
    }

    /// Filter and the list of types using the options.
    /// Perform additional filtering when diffing.
    fn filter_types(&self, hash: &FileHash, options: &Options, diff: bool) -> Vec<&Type> {
        let inline_types = self.inline_types(hash);
        let filter_type = |t: &Type| {
            // Filter by user options.
            if !t.filter(options) {
                return false;
            }
            match t.kind {
                TypeKind::Struct(ref t) => {
                    // Hack for rust closures
                    // TODO: is there better way of identifying these, or a
                    // a way to match pairs for diffing?
                    if diff && t.name == Some(b"closure") {
                        return false;
                    }
                }
                TypeKind::Def(..) | TypeKind::Union(..) | TypeKind::Enumeration(..) => {}
                TypeKind::Base(..)
                | TypeKind::Array(..)
                | TypeKind::Function(..)
                | TypeKind::Unspecified(..)
                | TypeKind::PointerToMember(..)
                | TypeKind::Modifier(..) => return false,
            }
            // Filter out inline types.
            !inline_types.contains(&t.offset.0)
        };
        self.types.values().filter(|a| filter_type(a)).collect()
    }

    fn filter_functions(&self, options: &Options) -> Vec<&Function> {
        self.functions.values().filter(|a| a.filter(options)).collect()
    }

    fn filter_variables(&self, options: &Options) -> Vec<&Variable> {
        self.variables.values().filter(|a| a.filter(options)).collect()
    }

    fn prefix_map(&self, options: &Options<'input>) -> (&'input [u8], &'input [u8]) {
        let name = self.name.unwrap_or(&[]);
        for &(old, new) in &options.prefix_map {
            if name.starts_with(old.as_bytes()) {
                return (new.as_bytes(), &name[old.len()..]);
            }
        }
        (&[], name)
    }

    /// Return true if this unit matches the filter options.
    pub fn filter(&self, options: &Options) -> bool {
        if let Some(filter) = options.filter_unit {
            let (prefix, suffix) = self.prefix_map(options);
            let iter = prefix.iter().chain(suffix);
            iter.cmp(filter.as_bytes()) == cmp::Ordering::Equal
        } else {
            true
        }
    }
}

impl<'input> Print for Unit<'input> {
    type Arg = ();

    fn print(&self, state: &mut PrintState, _arg: &()) -> Result<()> {
        self.print(state)
    }

    fn diff(state: &mut DiffState, _arg_a: &(), a: &Self, _arg_b: &(), b: &Self) -> Result<()> {
        Self::diff(state, a, b)
    }
}

impl<'input> SortList for Unit<'input> {
    fn cmp_id(
        _hash_a: &FileHash,
        a: &Self,
        _hash_b: &FileHash,
        b: &Self,
        options: &Options,
    ) -> cmp::Ordering {
        let (prefix_a, suffix_a) = a.prefix_map(options);
        let (prefix_b, suffix_b) = b.prefix_map(options);
        let iter_a = prefix_a.iter().chain(suffix_a);
        let iter_b = prefix_b.iter().chain(suffix_b);
        iter_a.cmp(iter_b)
    }

    fn cmp_by(
        hash_a: &FileHash,
        a: &Self,
        hash_b: &FileHash,
        b: &Self,
        options: &Options,
    ) -> cmp::Ordering {
        match options.sort {
            // TODO: sort by offset?
            Sort::None => cmp::Ordering::Equal,
            Sort::Name => Self::cmp_id(hash_a, a, hash_b, b, options),
            Sort::Size => a.size(hash_a).cmp(&b.size(hash_b)),
        }
    }
}
