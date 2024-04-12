use std::borrow::Cow;
use std::cmp;

use parser::{
    ArrayType, FileHash, FunctionType, PointerToMemberType, SubrangeType, Type, TypeKind,
    TypeModifier, TypeModifierKind, Unit, UnspecifiedType,
};

use crate::print::{self, DiffState, Print, PrintHeader, PrintState, SortList, ValuePrinter};
use crate::{Options, Result, Sort};

pub(crate) fn kind<'a>(ty: &'a Type) -> Result<&'a dyn PrintHeader> {
    Ok(match ty.kind() {
        TypeKind::Base(val) => val,
        TypeKind::Def(val) => val,
        TypeKind::Struct(val) => val,
        TypeKind::Union(val) => val,
        TypeKind::Enumeration(val) => val,
        TypeKind::Void
        | TypeKind::Array(..)
        | TypeKind::Function(..)
        | TypeKind::Unspecified(..)
        | TypeKind::PointerToMember(..)
        | TypeKind::Modifier(..)
        | TypeKind::Subrange(..) => return Err(format!("can't print {:?}", ty).into()),
    })
}

pub(crate) fn print(ty: &Type, state: &mut PrintState, unit: &Unit) -> Result<()> {
    let kind = kind(ty)?;
    state.id(
        ty.id(),
        |state| kind.print_header(state),
        |state| kind.print_body(state, unit),
    )?;
    state.line_break()?;
    Ok(())
}

pub(crate) fn print_ref(
    ty: Option<Cow<Type>>,
    w: &mut dyn ValuePrinter,
    hash: &FileHash,
) -> Result<()> {
    match ty {
        None => {
            write!(w, "<invalid-type>")?;
            Ok(())
        }
        Some(ty) => {
            let id = ty.id();
            match ty.kind() {
                TypeKind::Void => print_ref_void(w),
                TypeKind::Base(val) => print::base_type::print_ref(val, w, id),
                TypeKind::Def(val) => print::type_def::print_ref(val, w, id),
                TypeKind::Struct(val) => print::struct_type::print_ref(val, w, id),
                TypeKind::Union(val) => print::union_type::print_ref(val, w, id),
                TypeKind::Enumeration(val) => print::enumeration::print_ref(val, w, id),
                TypeKind::Array(val) => print_ref_array(val, w, hash),
                TypeKind::Function(val) => print_ref_function(val, w, hash),
                TypeKind::Unspecified(val) => print_ref_unspecified(val, w),
                TypeKind::PointerToMember(val) => print_ref_pointer_to_member(val, w, hash),
                TypeKind::Modifier(val) => print_ref_modifier(val, w, hash),
                TypeKind::Subrange(val) => print_ref_subrange(val, w, hash),
            }
        }
    }
}

fn ref_id(ty: Option<Cow<Type>>, hash: &FileHash) -> Option<usize> {
    let ty = ty?;
    let id = Some(ty.id());
    match ty.kind() {
        TypeKind::Void
        | TypeKind::Function(_)
        | TypeKind::Unspecified(_)
        | TypeKind::PointerToMember(_) => None,
        TypeKind::Base(_)
        | TypeKind::Def(_)
        | TypeKind::Struct(_)
        | TypeKind::Union(_)
        | TypeKind::Enumeration(_) => id,
        TypeKind::Array(val) => ref_id(val.element_type(hash), hash),
        TypeKind::Modifier(val) => ref_id(val.ty(hash), hash),
        TypeKind::Subrange(val) => ref_id(val.ty(hash), hash),
    }
}

fn print_ref_void(w: &mut dyn ValuePrinter) -> Result<()> {
    write!(w, "void")?;
    Ok(())
}

fn print_ref_array(ty: &ArrayType, w: &mut dyn ValuePrinter, hash: &FileHash) -> Result<()> {
    write!(w, "[")?;
    print_ref(ty.element_type(hash), w, hash)?;
    let mut counts = ty.counts();
    if let Some(count) = counts.next() {
        if let Some(count) = count {
            write!(w, "; {}", count)?;
        } else {
            write!(w, "; ??")?;
        }
    }
    for count in counts {
        if let Some(count) = count {
            write!(w, ", {}", count)?;
        } else {
            write!(w, ", ??")?;
        }
    }
    write!(w, "]")?;
    Ok(())
}

fn print_ref_function(ty: &FunctionType, w: &mut dyn ValuePrinter, hash: &FileHash) -> Result<()> {
    let mut first = true;
    write!(w, "(")?;
    for parameter in ty.parameters() {
        if first {
            first = false;
        } else {
            write!(w, ", ")?;
        }
        if let Some(name) = parameter.name() {
            write!(w, "{}: ", name)?;
        }
        print_ref(parameter.ty(hash), w, hash)?;
    }
    write!(w, ")")?;

    if let Some(return_type) = ty.return_type(hash) {
        if !return_type.is_void() {
            write!(w, " -> ")?;
            print_ref(Some(return_type), w, hash)?;
        }
    }
    Ok(())
}

fn print_ref_unspecified(ty: &UnspecifiedType, w: &mut dyn ValuePrinter) -> Result<()> {
    if let Some(namespace) = ty.namespace() {
        print::namespace::print(namespace, w)?;
    }
    w.name(ty.name().unwrap_or("<void>"))?;
    Ok(())
}

fn print_ref_pointer_to_member(
    ty: &PointerToMemberType,
    w: &mut dyn ValuePrinter,
    hash: &FileHash,
) -> Result<()> {
    print_ref(ty.containing_type(hash), w, hash)?;
    write!(w, "::* ")?;
    print_ref(ty.member_type(hash), w, hash)?;
    Ok(())
}

fn print_ref_modifier(ty: &TypeModifier, w: &mut dyn ValuePrinter, hash: &FileHash) -> Result<()> {
    if let Some(name) = ty.name() {
        if let Some(id) = ref_id(ty.ty(hash), hash) {
            w.link(id, &mut |w| w.name(name))?;
        } else {
            w.name(name)?;
        }
    } else {
        match ty.kind() {
            TypeModifierKind::Pointer => write!(w, "* ")?,
            TypeModifierKind::Reference | TypeModifierKind::RvalueReference => write!(w, "& ")?,
            TypeModifierKind::Const => write!(w, "const ")?,
            TypeModifierKind::Volatile => write!(w, "volatile ")?,
            TypeModifierKind::Restrict => write!(w, "restrict ")?,
            TypeModifierKind::Packed
            | TypeModifierKind::Shared
            | TypeModifierKind::Atomic
            | TypeModifierKind::Other => {}
        }
        print_ref(ty.ty(hash), w, hash)?;
    }
    Ok(())
}

fn print_ref_subrange(ty: &SubrangeType, w: &mut dyn ValuePrinter, hash: &FileHash) -> Result<()> {
    if let Some(name) = ty.name() {
        if let Some(id) = ref_id(ty.ty(hash), hash) {
            w.link(id, &mut |w| w.name(name))?;
        } else {
            w.name(name)?;
        }
    } else {
        print_ref(ty.ty(hash), w, hash)?;
    }
    // TODO: display bounds use underlying type
    match (ty.lower(), ty.upper()) {
        (Some(lower), Some(upper)) => write!(w, " {}..{}", lower, upper)?,
        (Some(lower), None) => write!(w, " {}..", lower)?,
        (None, Some(upper)) => write!(w, " ..{}", upper)?,
        (None, None) => {}
    }
    Ok(())
}

pub(crate) fn diff_header(state: &mut DiffState, type_a: &Type, type_b: &Type) -> Result<()> {
    use self::TypeKind::*;
    match (type_a.kind(), type_b.kind()) {
        (Base(a), Base(b)) => PrintHeader::diff_header(state, a, b),
        (Def(a), Def(b)) => PrintHeader::diff_header(state, a, b),
        (Struct(a), Struct(b)) => PrintHeader::diff_header(state, a, b),
        (Union(a), Union(b)) => PrintHeader::diff_header(state, a, b),
        (Enumeration(a), Enumeration(b)) => PrintHeader::diff_header(state, a, b),
        _ => Err(format!("can't diff {:?}, {:?}", type_a, type_b).into()),
    }
}

pub(crate) fn diff_body(
    state: &mut DiffState,
    unit_a: &Unit,
    type_a: &Type,
    unit_b: &Unit,
    type_b: &Type,
) -> Result<()> {
    use self::TypeKind::*;
    match (type_a.kind(), type_b.kind()) {
        (Base(a), Base(b)) => PrintHeader::diff_body(state, unit_a, a, unit_b, b),
        (Def(a), Def(b)) => PrintHeader::diff_body(state, unit_a, a, unit_b, b),
        (Struct(a), Struct(b)) => PrintHeader::diff_body(state, unit_a, a, unit_b, b),
        (Union(a), Union(b)) => PrintHeader::diff_body(state, unit_a, a, unit_b, b),
        (Enumeration(a), Enumeration(b)) => PrintHeader::diff_body(state, unit_a, a, unit_b, b),
        _ => Err(format!("can't diff {:?}, {:?}", type_a, type_b).into()),
    }
}

pub(crate) fn diff(
    state: &mut DiffState,
    unit_a: &Unit,
    a: &Type,
    unit_b: &Unit,
    b: &Type,
) -> Result<()> {
    state.id(
        a.id(),
        |state| diff_header(state, a, b),
        |state| diff_body(state, unit_a, a, unit_b, b),
    )?;
    state.line_break()?;
    Ok(())
}

pub(crate) fn print_members(state: &mut PrintState, unit: &Unit, ty: Option<&Type>) -> Result<()> {
    if let Some(ty) = ty {
        match ty.kind() {
            TypeKind::Struct(t) => return print::struct_type::print_members(t, state, unit),
            TypeKind::Union(t) => return print::union_type::print_members(t, state, unit),
            _ => {}
        }
    }
    Ok(())
}

pub(crate) fn diff_members(
    state: &mut DiffState,
    unit_a: &Unit,
    type_a: Option<&Type>,
    unit_b: &Unit,
    type_b: Option<&Type>,
) -> Result<()> {
    if let (Some(type_a), Some(type_b)) = (type_a, type_b) {
        match (type_a.kind(), type_b.kind()) {
            (TypeKind::Struct(a), TypeKind::Struct(b)) => {
                return print::struct_type::diff_members(state, unit_a, a, unit_b, b);
            }
            (TypeKind::Union(a), TypeKind::Union(b)) => {
                return print::union_type::diff_members(state, unit_a, a, unit_b, b);
            }
            _ => {}
        }
    }

    // Different types, so don't try to diff the members.
    state.block((unit_a, type_a), (unit_b, type_b), |state, (unit, x)| {
        print_members(state, unit, x)
    })
}

impl<'input> Print for Type<'input> {
    type Arg = Unit<'input>;

    fn print(&self, state: &mut PrintState, unit: &Self::Arg) -> Result<()> {
        print(self, state, unit)
    }

    fn diff(
        state: &mut DiffState,
        unit_a: &Self::Arg,
        a: &Self,
        unit_b: &Self::Arg,
        b: &Self,
    ) -> Result<()> {
        diff(state, unit_a, a, unit_b, b)
    }
}

impl<'input> SortList for Type<'input> {
    /// This must only be called for types that have identifiers.
    fn cmp_id(
        hash_a: &FileHash,
        type_a: &Type,
        hash_b: &FileHash,
        type_b: &Type,
        _options: &Options,
    ) -> cmp::Ordering {
        Type::cmp_id(hash_a, type_a, hash_b, type_b)
    }

    fn cmp_by(
        hash_a: &FileHash,
        a: &Self,
        hash_b: &FileHash,
        b: &Self,
        options: &Options,
    ) -> cmp::Ordering {
        match options.sort {
            Sort::None => a.offset().cmp(&b.offset()),
            Sort::Name => Type::cmp_id(hash_a, a, hash_b, b),
            Sort::Size => a.byte_size(hash_a).cmp(&b.byte_size(hash_b)),
        }
    }
}
