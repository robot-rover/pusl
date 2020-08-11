#![feature(prelude_import)]
#[prelude_import]
use std::prelude::v1::*;
#[macro_use]
extern crate std;
#[macro_use]
extern crate bitflags;
extern crate serde;
pub mod backend {
    use std::cell::RefCell;
    use std::thread::LocalKey;
    use garbage::{GcPointer, ManagedPool};
    use crate::backend::linearize::{Function, OpCode, ByteCodeFile, resolve};
    use crate::backend::object::{Object, ObjectPtr, Value};
    use crate::parser::expression::Compare;
    use std::cmp::Ordering;
    use std::path::PathBuf;
    use std::io;
    pub mod linearize {
        use crate::lexer::token::Literal;
        use crate::parser::branch::{Branch, ConditionBody};
        use crate::parser::expression::AssignmentFlags;
        use crate::parser::expression::{Compare, Expression};
        use crate::parser::{Eval, ExpRef, Import, ParsedFile};
        use serde::de::Visitor;
        use serde::{Deserialize, Deserializer, Serialize, Serializer};
        use std::fmt;
        use std::fmt::{Debug, Formatter};
        use std::path::PathBuf;
        use crate::backend::RFunction;
        use crate::backend::object::ObjectPtr;
        pub enum OpCode {
            Modulus,
            Literal,
            PushReference,
            PushFunction,
            FunctionCall,
            MethodCall,
            FieldAccess,
            Addition,
            Subtraction,
            Negate,
            Multiply,
            Divide,
            AssignReference,
            AssignField,
            DivideTruncate,
            Exponent,
            Compare,
            And,
            Or,
            ScopeUp,
            ScopeDown,
            Return,
            ConditionalJump,
            ComparisonJump,
            Jump,
            Pop,
            IsNull,
            Duplicate,
        }
        #[automatically_derived]
        #[allow(unused_qualifications)]
        impl ::core::marker::Copy for OpCode {}
        #[automatically_derived]
        #[allow(unused_qualifications)]
        impl ::core::clone::Clone for OpCode {
            #[inline]
            fn clone(&self) -> OpCode {
                {
                    *self
                }
            }
        }
        #[automatically_derived]
        #[allow(unused_qualifications)]
        impl ::core::fmt::Debug for OpCode {
            fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                match (&*self,) {
                    (&OpCode::Modulus,) => {
                        let mut debug_trait_builder = f.debug_tuple("Modulus");
                        debug_trait_builder.finish()
                    }
                    (&OpCode::Literal,) => {
                        let mut debug_trait_builder = f.debug_tuple("Literal");
                        debug_trait_builder.finish()
                    }
                    (&OpCode::PushReference,) => {
                        let mut debug_trait_builder = f.debug_tuple("PushReference");
                        debug_trait_builder.finish()
                    }
                    (&OpCode::PushFunction,) => {
                        let mut debug_trait_builder = f.debug_tuple("PushFunction");
                        debug_trait_builder.finish()
                    }
                    (&OpCode::FunctionCall,) => {
                        let mut debug_trait_builder = f.debug_tuple("FunctionCall");
                        debug_trait_builder.finish()
                    }
                    (&OpCode::MethodCall,) => {
                        let mut debug_trait_builder = f.debug_tuple("MethodCall");
                        debug_trait_builder.finish()
                    }
                    (&OpCode::FieldAccess,) => {
                        let mut debug_trait_builder = f.debug_tuple("FieldAccess");
                        debug_trait_builder.finish()
                    }
                    (&OpCode::Addition,) => {
                        let mut debug_trait_builder = f.debug_tuple("Addition");
                        debug_trait_builder.finish()
                    }
                    (&OpCode::Subtraction,) => {
                        let mut debug_trait_builder = f.debug_tuple("Subtraction");
                        debug_trait_builder.finish()
                    }
                    (&OpCode::Negate,) => {
                        let mut debug_trait_builder = f.debug_tuple("Negate");
                        debug_trait_builder.finish()
                    }
                    (&OpCode::Multiply,) => {
                        let mut debug_trait_builder = f.debug_tuple("Multiply");
                        debug_trait_builder.finish()
                    }
                    (&OpCode::Divide,) => {
                        let mut debug_trait_builder = f.debug_tuple("Divide");
                        debug_trait_builder.finish()
                    }
                    (&OpCode::AssignReference,) => {
                        let mut debug_trait_builder = f.debug_tuple("AssignReference");
                        debug_trait_builder.finish()
                    }
                    (&OpCode::AssignField,) => {
                        let mut debug_trait_builder = f.debug_tuple("AssignField");
                        debug_trait_builder.finish()
                    }
                    (&OpCode::DivideTruncate,) => {
                        let mut debug_trait_builder = f.debug_tuple("DivideTruncate");
                        debug_trait_builder.finish()
                    }
                    (&OpCode::Exponent,) => {
                        let mut debug_trait_builder = f.debug_tuple("Exponent");
                        debug_trait_builder.finish()
                    }
                    (&OpCode::Compare,) => {
                        let mut debug_trait_builder = f.debug_tuple("Compare");
                        debug_trait_builder.finish()
                    }
                    (&OpCode::And,) => {
                        let mut debug_trait_builder = f.debug_tuple("And");
                        debug_trait_builder.finish()
                    }
                    (&OpCode::Or,) => {
                        let mut debug_trait_builder = f.debug_tuple("Or");
                        debug_trait_builder.finish()
                    }
                    (&OpCode::ScopeUp,) => {
                        let mut debug_trait_builder = f.debug_tuple("ScopeUp");
                        debug_trait_builder.finish()
                    }
                    (&OpCode::ScopeDown,) => {
                        let mut debug_trait_builder = f.debug_tuple("ScopeDown");
                        debug_trait_builder.finish()
                    }
                    (&OpCode::Return,) => {
                        let mut debug_trait_builder = f.debug_tuple("Return");
                        debug_trait_builder.finish()
                    }
                    (&OpCode::ConditionalJump,) => {
                        let mut debug_trait_builder = f.debug_tuple("ConditionalJump");
                        debug_trait_builder.finish()
                    }
                    (&OpCode::ComparisonJump,) => {
                        let mut debug_trait_builder = f.debug_tuple("ComparisonJump");
                        debug_trait_builder.finish()
                    }
                    (&OpCode::Jump,) => {
                        let mut debug_trait_builder = f.debug_tuple("Jump");
                        debug_trait_builder.finish()
                    }
                    (&OpCode::Pop,) => {
                        let mut debug_trait_builder = f.debug_tuple("Pop");
                        debug_trait_builder.finish()
                    }
                    (&OpCode::IsNull,) => {
                        let mut debug_trait_builder = f.debug_tuple("IsNull");
                        debug_trait_builder.finish()
                    }
                    (&OpCode::Duplicate,) => {
                        let mut debug_trait_builder = f.debug_tuple("Duplicate");
                        debug_trait_builder.finish()
                    }
                }
            }
        }
        #[repr(C)]
        pub union ByteCode {
            op_code: OpCode,
            value: u64,
            compare: Compare,
            let_assign: bool,
        }
        #[automatically_derived]
        #[allow(unused_qualifications)]
        impl ::core::marker::Copy for ByteCode {}
        #[automatically_derived]
        #[allow(unused_qualifications)]
        impl ::core::clone::Clone for ByteCode {
            #[inline]
            fn clone(&self) -> ByteCode {
                {
                    let _: ::core::clone::AssertParamIsCopy<Self>;
                    *self
                }
            }
        }
        impl Serialize for ByteCode {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: Serializer,
            {
                serializer.serialize_u64(unsafe { self.value })
            }
        }
        impl<'de> Deserialize<'de> for ByteCode {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: Deserializer<'de>,
            {
                deserializer.deserialize_u64(ByteCodeVisitor)
            }
        }
        struct ByteCodeVisitor;
        impl<'de> Visitor<'de> for ByteCodeVisitor {
            type Value = ByteCode;
            fn expecting(&self, f: &mut Formatter<'_>) -> fmt::Result {
                f.write_fmt(::core::fmt::Arguments::new_v1(
                    &["a pusl bytecode (8 bytes) representing an opcode or a u64"],
                    &match () {
                        () => [],
                    },
                ))
            }
            fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(ByteCode { value: v })
            }
        }
        impl ByteCode {
            fn op(op_code: OpCode) -> Self {
                ByteCode { op_code }
            }
            fn val(value: usize) -> Self {
                ByteCode {
                    value: value as u64,
                }
            }
            fn zero() -> Self {
                ByteCode { value: 0 }
            }
            pub fn as_op(self) -> OpCode {
                unsafe { self.op_code }
            }
            pub fn as_val(self) -> usize {
                unsafe { self.value as usize }
            }
            pub fn as_cmp(self) -> Compare {
                unsafe { self.compare }
            }
            pub fn as_bool(self) -> bool {
                unsafe { self.let_assign }
            }
        }
        pub struct ByteCodeFile {
            pub file: PathBuf,
            pub base_func: Function<()>,
            pub imports: Vec<Import>,
        }
        impl Debug for ByteCodeFile {
            fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
                if !f.alternate() {
                    f.write_fmt(::core::fmt::Arguments::new_v1(
                        &["ByteCode: ", ", Imports: ", ", "],
                        &match (&self.file.display(), &self.imports.len(), &self.base_func) {
                            (arg0, arg1, arg2) => [
                                ::core::fmt::ArgumentV1::new(arg0, ::core::fmt::Display::fmt),
                                ::core::fmt::ArgumentV1::new(arg1, ::core::fmt::Display::fmt),
                                ::core::fmt::ArgumentV1::new(arg2, ::core::fmt::Debug::fmt),
                            ],
                        },
                    ))?;
                } else {
                    f.write_fmt(::core::fmt::Arguments::new_v1(
                        &["ByteCode: ", "\n"],
                        &match (&self.file.display(),) {
                            (arg0,) => [::core::fmt::ArgumentV1::new(
                                arg0,
                                ::core::fmt::Display::fmt,
                            )],
                        },
                    ))?;
                    f.write_fmt(::core::fmt::Arguments::new_v1(
                        &["Imports:\n"],
                        &match () {
                            () => [],
                        },
                    ))?;
                    for (index, import) in self.imports.iter().enumerate() {
                        f.write_fmt(::core::fmt::Arguments::new_v1_formatted(
                            &["\t", ": ", " as ", "\n"],
                            &match (&index, &import.path.display(), &import.alias) {
                                (arg0, arg1, arg2) => [
                                    ::core::fmt::ArgumentV1::new(arg0, ::core::fmt::Display::fmt),
                                    ::core::fmt::ArgumentV1::new(arg1, ::core::fmt::Display::fmt),
                                    ::core::fmt::ArgumentV1::new(arg2, ::core::fmt::Display::fmt),
                                ],
                            },
                            &[
                                ::core::fmt::rt::v1::Argument {
                                    position: ::core::fmt::rt::v1::Position::At(0usize),
                                    format: ::core::fmt::rt::v1::FormatSpec {
                                        fill: ' ',
                                        align: ::core::fmt::rt::v1::Alignment::Unknown,
                                        flags: 0u32,
                                        precision: ::core::fmt::rt::v1::Count::Implied,
                                        width: ::core::fmt::rt::v1::Count::Is(3usize),
                                    },
                                },
                                ::core::fmt::rt::v1::Argument {
                                    position: ::core::fmt::rt::v1::Position::At(1usize),
                                    format: ::core::fmt::rt::v1::FormatSpec {
                                        fill: ' ',
                                        align: ::core::fmt::rt::v1::Alignment::Unknown,
                                        flags: 0u32,
                                        precision: ::core::fmt::rt::v1::Count::Implied,
                                        width: ::core::fmt::rt::v1::Count::Implied,
                                    },
                                },
                                ::core::fmt::rt::v1::Argument {
                                    position: ::core::fmt::rt::v1::Position::At(2usize),
                                    format: ::core::fmt::rt::v1::FormatSpec {
                                        fill: ' ',
                                        align: ::core::fmt::rt::v1::Alignment::Unknown,
                                        flags: 0u32,
                                        precision: ::core::fmt::rt::v1::Count::Implied,
                                        width: ::core::fmt::rt::v1::Count::Implied,
                                    },
                                },
                            ],
                        ))?;
                    }
                    f.write_fmt(::core::fmt::Arguments::new_v1_formatted(
                        &[""],
                        &match (&self.base_func,) {
                            (arg0,) => {
                                [::core::fmt::ArgumentV1::new(arg0, ::core::fmt::Debug::fmt)]
                            }
                        },
                        &[::core::fmt::rt::v1::Argument {
                            position: ::core::fmt::rt::v1::Position::At(0usize),
                            format: ::core::fmt::rt::v1::FormatSpec {
                                fill: ' ',
                                align: ::core::fmt::rt::v1::Alignment::Unknown,
                                flags: 4u32,
                                precision: ::core::fmt::rt::v1::Count::Implied,
                                width: ::core::fmt::rt::v1::Count::Implied,
                            },
                        }],
                    ))?;
                }
                Ok(())
            }
        }
        pub struct Function<T> {
            pub args: Vec<String>,
            literals: Vec<Literal>,
            references: Vec<String>,
            code: Vec<ByteCode>,
            pub sub_functions: Vec<Function<T>>,
            pub resolved: T,
        }
        #[automatically_derived]
        #[allow(unused_qualifications)]
        impl<T: ::core::clone::Clone> ::core::clone::Clone for Function<T> {
            #[inline]
            fn clone(&self) -> Function<T> {
                match *self {
                    Function {
                        args: ref __self_0_0,
                        literals: ref __self_0_1,
                        references: ref __self_0_2,
                        code: ref __self_0_3,
                        sub_functions: ref __self_0_4,
                        resolved: ref __self_0_5,
                    } => Function {
                        args: ::core::clone::Clone::clone(&(*__self_0_0)),
                        literals: ::core::clone::Clone::clone(&(*__self_0_1)),
                        references: ::core::clone::Clone::clone(&(*__self_0_2)),
                        code: ::core::clone::Clone::clone(&(*__self_0_3)),
                        sub_functions: ::core::clone::Clone::clone(&(*__self_0_4)),
                        resolved: ::core::clone::Clone::clone(&(*__self_0_5)),
                    },
                }
            }
        }
        #[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
        const _IMPL_SERIALIZE_FOR_Function: () = {
            #[allow(unknown_lints)]
            #[allow(rust_2018_idioms)]
            extern crate serde as _serde;
            #[automatically_derived]
            impl<T> _serde::Serialize for Function<T>
            where
                T: _serde::Serialize,
            {
                fn serialize<__S>(
                    &self,
                    __serializer: __S,
                ) -> _serde::export::Result<__S::Ok, __S::Error>
                where
                    __S: _serde::Serializer,
                {
                    let mut __serde_state = match _serde::Serializer::serialize_struct(
                        __serializer,
                        "Function",
                        false as usize + 1 + 1 + 1 + 1 + 1 + 1,
                    ) {
                        _serde::export::Ok(__val) => __val,
                        _serde::export::Err(__err) => {
                            return _serde::export::Err(__err);
                        }
                    };
                    match _serde::ser::SerializeStruct::serialize_field(
                        &mut __serde_state,
                        "args",
                        &self.args,
                    ) {
                        _serde::export::Ok(__val) => __val,
                        _serde::export::Err(__err) => {
                            return _serde::export::Err(__err);
                        }
                    };
                    match _serde::ser::SerializeStruct::serialize_field(
                        &mut __serde_state,
                        "literals",
                        &self.literals,
                    ) {
                        _serde::export::Ok(__val) => __val,
                        _serde::export::Err(__err) => {
                            return _serde::export::Err(__err);
                        }
                    };
                    match _serde::ser::SerializeStruct::serialize_field(
                        &mut __serde_state,
                        "references",
                        &self.references,
                    ) {
                        _serde::export::Ok(__val) => __val,
                        _serde::export::Err(__err) => {
                            return _serde::export::Err(__err);
                        }
                    };
                    match _serde::ser::SerializeStruct::serialize_field(
                        &mut __serde_state,
                        "code",
                        &self.code,
                    ) {
                        _serde::export::Ok(__val) => __val,
                        _serde::export::Err(__err) => {
                            return _serde::export::Err(__err);
                        }
                    };
                    match _serde::ser::SerializeStruct::serialize_field(
                        &mut __serde_state,
                        "sub_functions",
                        &self.sub_functions,
                    ) {
                        _serde::export::Ok(__val) => __val,
                        _serde::export::Err(__err) => {
                            return _serde::export::Err(__err);
                        }
                    };
                    match _serde::ser::SerializeStruct::serialize_field(
                        &mut __serde_state,
                        "resolved",
                        &self.resolved,
                    ) {
                        _serde::export::Ok(__val) => __val,
                        _serde::export::Err(__err) => {
                            return _serde::export::Err(__err);
                        }
                    };
                    _serde::ser::SerializeStruct::end(__serde_state)
                }
            }
        };
        #[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
        const _IMPL_DESERIALIZE_FOR_Function: () = {
            #[allow(unknown_lints)]
            #[allow(rust_2018_idioms)]
            extern crate serde as _serde;
            #[automatically_derived]
            impl<'de, T> _serde::Deserialize<'de> for Function<T>
            where
                T: _serde::Deserialize<'de>,
            {
                fn deserialize<__D>(__deserializer: __D) -> _serde::export::Result<Self, __D::Error>
                where
                    __D: _serde::Deserializer<'de>,
                {
                    #[allow(non_camel_case_types)]
                    enum __Field {
                        __field0,
                        __field1,
                        __field2,
                        __field3,
                        __field4,
                        __field5,
                        __ignore,
                    }
                    struct __FieldVisitor;
                    impl<'de> _serde::de::Visitor<'de> for __FieldVisitor {
                        type Value = __Field;
                        fn expecting(
                            &self,
                            __formatter: &mut _serde::export::Formatter,
                        ) -> _serde::export::fmt::Result {
                            _serde::export::Formatter::write_str(__formatter, "field identifier")
                        }
                        fn visit_u64<__E>(
                            self,
                            __value: u64,
                        ) -> _serde::export::Result<Self::Value, __E>
                        where
                            __E: _serde::de::Error,
                        {
                            match __value {
                                0u64 => _serde::export::Ok(__Field::__field0),
                                1u64 => _serde::export::Ok(__Field::__field1),
                                2u64 => _serde::export::Ok(__Field::__field2),
                                3u64 => _serde::export::Ok(__Field::__field3),
                                4u64 => _serde::export::Ok(__Field::__field4),
                                5u64 => _serde::export::Ok(__Field::__field5),
                                _ => _serde::export::Err(_serde::de::Error::invalid_value(
                                    _serde::de::Unexpected::Unsigned(__value),
                                    &"field index 0 <= i < 6",
                                )),
                            }
                        }
                        fn visit_str<__E>(
                            self,
                            __value: &str,
                        ) -> _serde::export::Result<Self::Value, __E>
                        where
                            __E: _serde::de::Error,
                        {
                            match __value {
                                "args" => _serde::export::Ok(__Field::__field0),
                                "literals" => _serde::export::Ok(__Field::__field1),
                                "references" => _serde::export::Ok(__Field::__field2),
                                "code" => _serde::export::Ok(__Field::__field3),
                                "sub_functions" => _serde::export::Ok(__Field::__field4),
                                "resolved" => _serde::export::Ok(__Field::__field5),
                                _ => _serde::export::Ok(__Field::__ignore),
                            }
                        }
                        fn visit_bytes<__E>(
                            self,
                            __value: &[u8],
                        ) -> _serde::export::Result<Self::Value, __E>
                        where
                            __E: _serde::de::Error,
                        {
                            match __value {
                                b"args" => _serde::export::Ok(__Field::__field0),
                                b"literals" => _serde::export::Ok(__Field::__field1),
                                b"references" => _serde::export::Ok(__Field::__field2),
                                b"code" => _serde::export::Ok(__Field::__field3),
                                b"sub_functions" => _serde::export::Ok(__Field::__field4),
                                b"resolved" => _serde::export::Ok(__Field::__field5),
                                _ => _serde::export::Ok(__Field::__ignore),
                            }
                        }
                    }
                    impl<'de> _serde::Deserialize<'de> for __Field {
                        #[inline]
                        fn deserialize<__D>(
                            __deserializer: __D,
                        ) -> _serde::export::Result<Self, __D::Error>
                        where
                            __D: _serde::Deserializer<'de>,
                        {
                            _serde::Deserializer::deserialize_identifier(
                                __deserializer,
                                __FieldVisitor,
                            )
                        }
                    }
                    struct __Visitor<'de, T>
                    where
                        T: _serde::Deserialize<'de>,
                    {
                        marker: _serde::export::PhantomData<Function<T>>,
                        lifetime: _serde::export::PhantomData<&'de ()>,
                    }
                    impl<'de, T> _serde::de::Visitor<'de> for __Visitor<'de, T>
                    where
                        T: _serde::Deserialize<'de>,
                    {
                        type Value = Function<T>;
                        fn expecting(
                            &self,
                            __formatter: &mut _serde::export::Formatter,
                        ) -> _serde::export::fmt::Result {
                            _serde::export::Formatter::write_str(__formatter, "struct Function")
                        }
                        #[inline]
                        fn visit_seq<__A>(
                            self,
                            mut __seq: __A,
                        ) -> _serde::export::Result<Self::Value, __A::Error>
                        where
                            __A: _serde::de::SeqAccess<'de>,
                        {
                            let __field0 = match match _serde::de::SeqAccess::next_element::<
                                Vec<String>,
                            >(&mut __seq)
                            {
                                _serde::export::Ok(__val) => __val,
                                _serde::export::Err(__err) => {
                                    return _serde::export::Err(__err);
                                }
                            } {
                                _serde::export::Some(__value) => __value,
                                _serde::export::None => {
                                    return _serde::export::Err(_serde::de::Error::invalid_length(
                                        0usize,
                                        &"struct Function with 6 elements",
                                    ));
                                }
                            };
                            let __field1 = match match _serde::de::SeqAccess::next_element::<
                                Vec<Literal>,
                            >(&mut __seq)
                            {
                                _serde::export::Ok(__val) => __val,
                                _serde::export::Err(__err) => {
                                    return _serde::export::Err(__err);
                                }
                            } {
                                _serde::export::Some(__value) => __value,
                                _serde::export::None => {
                                    return _serde::export::Err(_serde::de::Error::invalid_length(
                                        1usize,
                                        &"struct Function with 6 elements",
                                    ));
                                }
                            };
                            let __field2 = match match _serde::de::SeqAccess::next_element::<
                                Vec<String>,
                            >(&mut __seq)
                            {
                                _serde::export::Ok(__val) => __val,
                                _serde::export::Err(__err) => {
                                    return _serde::export::Err(__err);
                                }
                            } {
                                _serde::export::Some(__value) => __value,
                                _serde::export::None => {
                                    return _serde::export::Err(_serde::de::Error::invalid_length(
                                        2usize,
                                        &"struct Function with 6 elements",
                                    ));
                                }
                            };
                            let __field3 = match match _serde::de::SeqAccess::next_element::<
                                Vec<ByteCode>,
                            >(&mut __seq)
                            {
                                _serde::export::Ok(__val) => __val,
                                _serde::export::Err(__err) => {
                                    return _serde::export::Err(__err);
                                }
                            } {
                                _serde::export::Some(__value) => __value,
                                _serde::export::None => {
                                    return _serde::export::Err(_serde::de::Error::invalid_length(
                                        3usize,
                                        &"struct Function with 6 elements",
                                    ));
                                }
                            };
                            let __field4 = match match _serde::de::SeqAccess::next_element::<
                                Vec<Function<T>>,
                            >(&mut __seq)
                            {
                                _serde::export::Ok(__val) => __val,
                                _serde::export::Err(__err) => {
                                    return _serde::export::Err(__err);
                                }
                            } {
                                _serde::export::Some(__value) => __value,
                                _serde::export::None => {
                                    return _serde::export::Err(_serde::de::Error::invalid_length(
                                        4usize,
                                        &"struct Function with 6 elements",
                                    ));
                                }
                            };
                            let __field5 =
                                match match _serde::de::SeqAccess::next_element::<T>(&mut __seq) {
                                    _serde::export::Ok(__val) => __val,
                                    _serde::export::Err(__err) => {
                                        return _serde::export::Err(__err);
                                    }
                                } {
                                    _serde::export::Some(__value) => __value,
                                    _serde::export::None => {
                                        return _serde::export::Err(
                                            _serde::de::Error::invalid_length(
                                                5usize,
                                                &"struct Function with 6 elements",
                                            ),
                                        );
                                    }
                                };
                            _serde::export::Ok(Function {
                                args: __field0,
                                literals: __field1,
                                references: __field2,
                                code: __field3,
                                sub_functions: __field4,
                                resolved: __field5,
                            })
                        }
                        #[inline]
                        fn visit_map<__A>(
                            self,
                            mut __map: __A,
                        ) -> _serde::export::Result<Self::Value, __A::Error>
                        where
                            __A: _serde::de::MapAccess<'de>,
                        {
                            let mut __field0: _serde::export::Option<Vec<String>> =
                                _serde::export::None;
                            let mut __field1: _serde::export::Option<Vec<Literal>> =
                                _serde::export::None;
                            let mut __field2: _serde::export::Option<Vec<String>> =
                                _serde::export::None;
                            let mut __field3: _serde::export::Option<Vec<ByteCode>> =
                                _serde::export::None;
                            let mut __field4: _serde::export::Option<Vec<Function<T>>> =
                                _serde::export::None;
                            let mut __field5: _serde::export::Option<T> = _serde::export::None;
                            while let _serde::export::Some(__key) =
                                match _serde::de::MapAccess::next_key::<__Field>(&mut __map) {
                                    _serde::export::Ok(__val) => __val,
                                    _serde::export::Err(__err) => {
                                        return _serde::export::Err(__err);
                                    }
                                }
                            {
                                match __key {
                                    __Field::__field0 => {
                                        if _serde::export::Option::is_some(&__field0) {
                                            return _serde::export::Err(
                                                <__A::Error as _serde::de::Error>::duplicate_field(
                                                    "args",
                                                ),
                                            );
                                        }
                                        __field0 = _serde::export::Some(
                                            match _serde::de::MapAccess::next_value::<Vec<String>>(
                                                &mut __map,
                                            ) {
                                                _serde::export::Ok(__val) => __val,
                                                _serde::export::Err(__err) => {
                                                    return _serde::export::Err(__err);
                                                }
                                            },
                                        );
                                    }
                                    __Field::__field1 => {
                                        if _serde::export::Option::is_some(&__field1) {
                                            return _serde::export::Err(
                                                <__A::Error as _serde::de::Error>::duplicate_field(
                                                    "literals",
                                                ),
                                            );
                                        }
                                        __field1 = _serde::export::Some(
                                            match _serde::de::MapAccess::next_value::<Vec<Literal>>(
                                                &mut __map,
                                            ) {
                                                _serde::export::Ok(__val) => __val,
                                                _serde::export::Err(__err) => {
                                                    return _serde::export::Err(__err);
                                                }
                                            },
                                        );
                                    }
                                    __Field::__field2 => {
                                        if _serde::export::Option::is_some(&__field2) {
                                            return _serde::export::Err(
                                                <__A::Error as _serde::de::Error>::duplicate_field(
                                                    "references",
                                                ),
                                            );
                                        }
                                        __field2 = _serde::export::Some(
                                            match _serde::de::MapAccess::next_value::<Vec<String>>(
                                                &mut __map,
                                            ) {
                                                _serde::export::Ok(__val) => __val,
                                                _serde::export::Err(__err) => {
                                                    return _serde::export::Err(__err);
                                                }
                                            },
                                        );
                                    }
                                    __Field::__field3 => {
                                        if _serde::export::Option::is_some(&__field3) {
                                            return _serde::export::Err(
                                                <__A::Error as _serde::de::Error>::duplicate_field(
                                                    "code",
                                                ),
                                            );
                                        }
                                        __field3 = _serde::export::Some(
                                            match _serde::de::MapAccess::next_value::<Vec<ByteCode>>(
                                                &mut __map,
                                            ) {
                                                _serde::export::Ok(__val) => __val,
                                                _serde::export::Err(__err) => {
                                                    return _serde::export::Err(__err);
                                                }
                                            },
                                        );
                                    }
                                    __Field::__field4 => {
                                        if _serde::export::Option::is_some(&__field4) {
                                            return _serde::export::Err(
                                                <__A::Error as _serde::de::Error>::duplicate_field(
                                                    "sub_functions",
                                                ),
                                            );
                                        }
                                        __field4 = _serde::export::Some(
                                            match _serde::de::MapAccess::next_value::<
                                                Vec<Function<T>>,
                                            >(
                                                &mut __map
                                            ) {
                                                _serde::export::Ok(__val) => __val,
                                                _serde::export::Err(__err) => {
                                                    return _serde::export::Err(__err);
                                                }
                                            },
                                        );
                                    }
                                    __Field::__field5 => {
                                        if _serde::export::Option::is_some(&__field5) {
                                            return _serde::export::Err(
                                                <__A::Error as _serde::de::Error>::duplicate_field(
                                                    "resolved",
                                                ),
                                            );
                                        }
                                        __field5 = _serde::export::Some(
                                            match _serde::de::MapAccess::next_value::<T>(&mut __map)
                                            {
                                                _serde::export::Ok(__val) => __val,
                                                _serde::export::Err(__err) => {
                                                    return _serde::export::Err(__err);
                                                }
                                            },
                                        );
                                    }
                                    _ => {
                                        let _ = match _serde::de::MapAccess::next_value::<
                                            _serde::de::IgnoredAny,
                                        >(
                                            &mut __map
                                        ) {
                                            _serde::export::Ok(__val) => __val,
                                            _serde::export::Err(__err) => {
                                                return _serde::export::Err(__err);
                                            }
                                        };
                                    }
                                }
                            }
                            let __field0 = match __field0 {
                                _serde::export::Some(__field0) => __field0,
                                _serde::export::None => {
                                    match _serde::private::de::missing_field("args") {
                                        _serde::export::Ok(__val) => __val,
                                        _serde::export::Err(__err) => {
                                            return _serde::export::Err(__err);
                                        }
                                    }
                                }
                            };
                            let __field1 = match __field1 {
                                _serde::export::Some(__field1) => __field1,
                                _serde::export::None => {
                                    match _serde::private::de::missing_field("literals") {
                                        _serde::export::Ok(__val) => __val,
                                        _serde::export::Err(__err) => {
                                            return _serde::export::Err(__err);
                                        }
                                    }
                                }
                            };
                            let __field2 = match __field2 {
                                _serde::export::Some(__field2) => __field2,
                                _serde::export::None => {
                                    match _serde::private::de::missing_field("references") {
                                        _serde::export::Ok(__val) => __val,
                                        _serde::export::Err(__err) => {
                                            return _serde::export::Err(__err);
                                        }
                                    }
                                }
                            };
                            let __field3 = match __field3 {
                                _serde::export::Some(__field3) => __field3,
                                _serde::export::None => {
                                    match _serde::private::de::missing_field("code") {
                                        _serde::export::Ok(__val) => __val,
                                        _serde::export::Err(__err) => {
                                            return _serde::export::Err(__err);
                                        }
                                    }
                                }
                            };
                            let __field4 = match __field4 {
                                _serde::export::Some(__field4) => __field4,
                                _serde::export::None => {
                                    match _serde::private::de::missing_field("sub_functions") {
                                        _serde::export::Ok(__val) => __val,
                                        _serde::export::Err(__err) => {
                                            return _serde::export::Err(__err);
                                        }
                                    }
                                }
                            };
                            let __field5 = match __field5 {
                                _serde::export::Some(__field5) => __field5,
                                _serde::export::None => {
                                    match _serde::private::de::missing_field("resolved") {
                                        _serde::export::Ok(__val) => __val,
                                        _serde::export::Err(__err) => {
                                            return _serde::export::Err(__err);
                                        }
                                    }
                                }
                            };
                            _serde::export::Ok(Function {
                                args: __field0,
                                literals: __field1,
                                references: __field2,
                                code: __field3,
                                sub_functions: __field4,
                                resolved: __field5,
                            })
                        }
                    }
                    const FIELDS: &'static [&'static str] = &[
                        "args",
                        "literals",
                        "references",
                        "code",
                        "sub_functions",
                        "resolved",
                    ];
                    _serde::Deserializer::deserialize_struct(
                        __deserializer,
                        "Function",
                        FIELDS,
                        __Visitor {
                            marker: _serde::export::PhantomData::<Function<T>>,
                            lifetime: _serde::export::PhantomData,
                        },
                    )
                }
            }
        };
        pub fn resolve<'a, I>(
            function: Function<()>,
            global_imports: I,
            target_imports: Vec<Import>,
        ) -> &'static RFunction
        where
            I: IntoIterator<Item = &'a (PathBuf, ObjectPtr)>,
        {
            let Function {
                args,
                literals,
                references,
                code,
                sub_functions,
                ..
            } = function;
            let mut iter = global_imports.into_iter();
            let mut imports = Vec::new();
            for Import { path, alias } in target_imports {
                let import_object = iter
                    .by_ref()
                    .find(|i| &i.0 == &path)
                    .map(|i| i.1.clone())
                    .unwrap();
                imports.push((alias, import_object));
            }
            let imports: &Vec<_> = Box::leak(Box::new(imports));
            let sub_functions = sub_functions
                .into_iter()
                .map(|f| sub_resolve(f, imports))
                .collect();
            let result = RFunction {
                args,
                literals,
                references,
                code,
                sub_functions,
                resolved: imports,
            };
            Box::leak(Box::new(result))
        }
        fn sub_resolve(
            function: Function<()>,
            imports: &'static Vec<(String, ObjectPtr)>,
        ) -> RFunction {
            let Function {
                args,
                literals,
                references,
                code,
                sub_functions,
                ..
            } = function;
            let sub_functions = sub_functions
                .into_iter()
                .map(|f| sub_resolve(f, imports))
                .collect();
            RFunction {
                args,
                literals,
                references,
                code,
                sub_functions,
                resolved: imports,
            }
        }
        impl<T> Debug for Function<T> {
            fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
                f.write_fmt(::core::fmt::Arguments::new_v1(
                    &["Function("],
                    &match () {
                        () => [],
                    },
                ))?;
                let mut arg_iter = self.args.iter().peekable();
                while let Some(arg_name) = arg_iter.next() {
                    f.write_fmt(::core::fmt::Arguments::new_v1(
                        &[""],
                        &match (&arg_name,) {
                            (arg0,) => [::core::fmt::ArgumentV1::new(
                                arg0,
                                ::core::fmt::Display::fmt,
                            )],
                        },
                    ))?;
                    if arg_iter.peek().is_some() {
                        f.write_fmt(::core::fmt::Arguments::new_v1(
                            &[", "],
                            &match () {
                                () => [],
                            },
                        ))?;
                    }
                }
                f.write_fmt(::core::fmt::Arguments::new_v1(
                    &[")"],
                    &match () {
                        () => [],
                    },
                ))?;
                if !f.alternate() {
                    f.write_fmt(::core::fmt::Arguments::new_v1(
                        &[" - lits: ", ", refs: ", ", code: ", ", sub-funcs: "],
                        &match (
                            &self.literals.len(),
                            &self.references.len(),
                            &self.code.len(),
                            &self.sub_functions.len(),
                        ) {
                            (arg0, arg1, arg2, arg3) => [
                                ::core::fmt::ArgumentV1::new(arg0, ::core::fmt::Display::fmt),
                                ::core::fmt::ArgumentV1::new(arg1, ::core::fmt::Display::fmt),
                                ::core::fmt::ArgumentV1::new(arg2, ::core::fmt::Display::fmt),
                                ::core::fmt::ArgumentV1::new(arg3, ::core::fmt::Display::fmt),
                            ],
                        },
                    ))?;
                } else {
                    f.write_fmt(::core::fmt::Arguments::new_v1(
                        &["\nLiterals:\n"],
                        &match () {
                            () => [],
                        },
                    ))?;
                    for (index, literal) in self.literals.iter().enumerate() {
                        f.write_fmt(::core::fmt::Arguments::new_v1_formatted(
                            &["\t", ": ", "\n"],
                            &match (&index, &literal) {
                                (arg0, arg1) => [
                                    ::core::fmt::ArgumentV1::new(arg0, ::core::fmt::Display::fmt),
                                    ::core::fmt::ArgumentV1::new(arg1, ::core::fmt::Debug::fmt),
                                ],
                            },
                            &[
                                ::core::fmt::rt::v1::Argument {
                                    position: ::core::fmt::rt::v1::Position::At(0usize),
                                    format: ::core::fmt::rt::v1::FormatSpec {
                                        fill: ' ',
                                        align: ::core::fmt::rt::v1::Alignment::Unknown,
                                        flags: 0u32,
                                        precision: ::core::fmt::rt::v1::Count::Implied,
                                        width: ::core::fmt::rt::v1::Count::Is(3usize),
                                    },
                                },
                                ::core::fmt::rt::v1::Argument {
                                    position: ::core::fmt::rt::v1::Position::At(1usize),
                                    format: ::core::fmt::rt::v1::FormatSpec {
                                        fill: ' ',
                                        align: ::core::fmt::rt::v1::Alignment::Unknown,
                                        flags: 0u32,
                                        precision: ::core::fmt::rt::v1::Count::Implied,
                                        width: ::core::fmt::rt::v1::Count::Implied,
                                    },
                                },
                            ],
                        ))?;
                    }
                    f.write_fmt(::core::fmt::Arguments::new_v1(
                        &["References:\n"],
                        &match () {
                            () => [],
                        },
                    ))?;
                    for (index, reference) in self.references.iter().enumerate() {
                        f.write_fmt(::core::fmt::Arguments::new_v1_formatted(
                            &["\t", ": ", "\n"],
                            &match (&index, &reference) {
                                (arg0, arg1) => [
                                    ::core::fmt::ArgumentV1::new(arg0, ::core::fmt::Display::fmt),
                                    ::core::fmt::ArgumentV1::new(arg1, ::core::fmt::Display::fmt),
                                ],
                            },
                            &[
                                ::core::fmt::rt::v1::Argument {
                                    position: ::core::fmt::rt::v1::Position::At(0usize),
                                    format: ::core::fmt::rt::v1::FormatSpec {
                                        fill: ' ',
                                        align: ::core::fmt::rt::v1::Alignment::Unknown,
                                        flags: 0u32,
                                        precision: ::core::fmt::rt::v1::Count::Implied,
                                        width: ::core::fmt::rt::v1::Count::Is(3usize),
                                    },
                                },
                                ::core::fmt::rt::v1::Argument {
                                    position: ::core::fmt::rt::v1::Position::At(1usize),
                                    format: ::core::fmt::rt::v1::FormatSpec {
                                        fill: ' ',
                                        align: ::core::fmt::rt::v1::Alignment::Unknown,
                                        flags: 0u32,
                                        precision: ::core::fmt::rt::v1::Count::Implied,
                                        width: ::core::fmt::rt::v1::Count::Implied,
                                    },
                                },
                            ],
                        ))?;
                    }
                    f.write_fmt(::core::fmt::Arguments::new_v1(
                        &["Sub-Functions:\n"],
                        &match () {
                            () => [],
                        },
                    ))?;
                    for (index, sub_function) in self.sub_functions.iter().enumerate() {
                        f.write_fmt(::core::fmt::Arguments::new_v1_formatted(
                            &["\t", ": ", "\n"],
                            &match (&index, &sub_function) {
                                (arg0, arg1) => [
                                    ::core::fmt::ArgumentV1::new(arg0, ::core::fmt::Display::fmt),
                                    ::core::fmt::ArgumentV1::new(arg1, ::core::fmt::Debug::fmt),
                                ],
                            },
                            &[
                                ::core::fmt::rt::v1::Argument {
                                    position: ::core::fmt::rt::v1::Position::At(0usize),
                                    format: ::core::fmt::rt::v1::FormatSpec {
                                        fill: ' ',
                                        align: ::core::fmt::rt::v1::Alignment::Unknown,
                                        flags: 0u32,
                                        precision: ::core::fmt::rt::v1::Count::Implied,
                                        width: ::core::fmt::rt::v1::Count::Is(3usize),
                                    },
                                },
                                ::core::fmt::rt::v1::Argument {
                                    position: ::core::fmt::rt::v1::Position::At(1usize),
                                    format: ::core::fmt::rt::v1::FormatSpec {
                                        fill: ' ',
                                        align: ::core::fmt::rt::v1::Alignment::Unknown,
                                        flags: 0u32,
                                        precision: ::core::fmt::rt::v1::Count::Implied,
                                        width: ::core::fmt::rt::v1::Count::Implied,
                                    },
                                },
                            ],
                        ))?;
                    }
                    f.write_fmt(::core::fmt::Arguments::new_v1(
                        &["Code:\n"],
                        &match () {
                            () => [],
                        },
                    ))?;
                    let mut code_iter = self.code.iter().enumerate();
                    while let Some(tuple) = code_iter.next() {
                        write_bytecode_line(tuple, f, &mut code_iter, &self)?;
                    }
                }
                Ok(())
            }
        }
        fn write_bytecode_line<'a, T, F>(
            line: (usize, &ByteCode),
            f: &mut Formatter<'_>,
            code_iter: &mut T,
            func: &Function<F>,
        ) -> fmt::Result
        where
            T: Iterator<Item = (usize, &'a ByteCode)>,
        {
            let (index, bytecode) = line;
            let op_code = bytecode.as_op();
            f.write_fmt(::core::fmt::Arguments::new_v1_formatted(
                &["\t", ": "],
                &match (&index,) {
                    (arg0,) => [::core::fmt::ArgumentV1::new(
                        arg0,
                        ::core::fmt::Display::fmt,
                    )],
                },
                &[::core::fmt::rt::v1::Argument {
                    position: ::core::fmt::rt::v1::Position::At(0usize),
                    format: ::core::fmt::rt::v1::FormatSpec {
                        fill: ' ',
                        align: ::core::fmt::rt::v1::Alignment::Unknown,
                        flags: 0u32,
                        precision: ::core::fmt::rt::v1::Count::Implied,
                        width: ::core::fmt::rt::v1::Count::Is(3usize),
                    },
                }],
            ))?;
            match op_code {
                OpCode::Modulus => f.write_fmt(::core::fmt::Arguments::new_v1(
                    &["Modulus\n"],
                    &match () {
                        () => [],
                    },
                ))?,
                OpCode::Literal => {
                    let pool_index = code_iter.next().unwrap().1.as_val();
                    let pool_value = &func.literals[pool_index];
                    f.write_fmt(::core::fmt::Arguments::new_v1(
                        &["Literal ", "[", "]\n"],
                        &match (&pool_value, &pool_index) {
                            (arg0, arg1) => [
                                ::core::fmt::ArgumentV1::new(arg0, ::core::fmt::Debug::fmt),
                                ::core::fmt::ArgumentV1::new(arg1, ::core::fmt::Display::fmt),
                            ],
                        },
                    ))?;
                }
                OpCode::PushReference => {
                    let pool_index = code_iter.next().unwrap().1.as_val();
                    let pool_value = &func.references[pool_index];
                    f.write_fmt(::core::fmt::Arguments::new_v1(
                        &["PushRef \"", "\"[", "]\n"],
                        &match (&pool_value, &pool_index) {
                            (arg0, arg1) => [
                                ::core::fmt::ArgumentV1::new(arg0, ::core::fmt::Display::fmt),
                                ::core::fmt::ArgumentV1::new(arg1, ::core::fmt::Display::fmt),
                            ],
                        },
                    ))?;
                }
                OpCode::PushFunction => {
                    let pool_index = code_iter.next().unwrap().1.as_val();
                    let pool_value = &func.sub_functions[pool_index];
                    f.write_fmt(::core::fmt::Arguments::new_v1(
                        &["PushFunc ", " | [", "]\n"],
                        &match (&pool_value, &pool_index) {
                            (arg0, arg1) => [
                                ::core::fmt::ArgumentV1::new(arg0, ::core::fmt::Debug::fmt),
                                ::core::fmt::ArgumentV1::new(arg1, ::core::fmt::Display::fmt),
                            ],
                        },
                    ))?;
                }
                OpCode::FunctionCall => {
                    let pool_index = code_iter.next().unwrap().1.as_val();
                    f.write_fmt(::core::fmt::Arguments::new_v1(
                        &["FnCall ", "\n"],
                        &match (&pool_index,) {
                            (arg0,) => [::core::fmt::ArgumentV1::new(
                                arg0,
                                ::core::fmt::Display::fmt,
                            )],
                        },
                    ))?;
                }
                OpCode::MethodCall => {
                    let pool_index = code_iter.next().unwrap().1.as_val();
                    f.write_fmt(::core::fmt::Arguments::new_v1(
                        &["ObjCall ", "\n"],
                        &match (&pool_index,) {
                            (arg0,) => [::core::fmt::ArgumentV1::new(
                                arg0,
                                ::core::fmt::Display::fmt,
                            )],
                        },
                    ))?;
                }
                OpCode::FieldAccess => {
                    let pool_index = code_iter.next().unwrap().1.as_val();
                    let pool_value = &func.references[pool_index];
                    f.write_fmt(::core::fmt::Arguments::new_v1(
                        &["Field ", "[", "]\n"],
                        &match (&pool_value, &pool_index) {
                            (arg0, arg1) => [
                                ::core::fmt::ArgumentV1::new(arg0, ::core::fmt::Display::fmt),
                                ::core::fmt::ArgumentV1::new(arg1, ::core::fmt::Display::fmt),
                            ],
                        },
                    ))?;
                }
                OpCode::Addition => f.write_fmt(::core::fmt::Arguments::new_v1(
                    &["Addition\n"],
                    &match () {
                        () => [],
                    },
                ))?,
                OpCode::Subtraction => f.write_fmt(::core::fmt::Arguments::new_v1(
                    &["Subtraction\n"],
                    &match () {
                        () => [],
                    },
                ))?,
                OpCode::Negate => f.write_fmt(::core::fmt::Arguments::new_v1(
                    &["Negate\n"],
                    &match () {
                        () => [],
                    },
                ))?,
                OpCode::Multiply => f.write_fmt(::core::fmt::Arguments::new_v1(
                    &["Multiply\n"],
                    &match () {
                        () => [],
                    },
                ))?,
                OpCode::Divide => f.write_fmt(::core::fmt::Arguments::new_v1(
                    &["Divide\n"],
                    &match () {
                        () => [],
                    },
                ))?,
                OpCode::DivideTruncate => f.write_fmt(::core::fmt::Arguments::new_v1(
                    &["DivTrunc\n"],
                    &match () {
                        () => [],
                    },
                ))?,
                OpCode::Exponent => f.write_fmt(::core::fmt::Arguments::new_v1(
                    &["Exponent\n"],
                    &match () {
                        () => [],
                    },
                ))?,
                OpCode::Compare => {
                    let compare = unsafe { code_iter.next().unwrap().1.compare };
                    f.write_fmt(::core::fmt::Arguments::new_v1(
                        &["Compare ", "\n"],
                        &match (&compare,) {
                            (arg0,) => {
                                [::core::fmt::ArgumentV1::new(arg0, ::core::fmt::Debug::fmt)]
                            }
                        },
                    ))?;
                }
                OpCode::And => f.write_fmt(::core::fmt::Arguments::new_v1(
                    &["And\n"],
                    &match () {
                        () => [],
                    },
                ))?,
                OpCode::Or => f.write_fmt(::core::fmt::Arguments::new_v1(
                    &["Or\n"],
                    &match () {
                        () => [],
                    },
                ))?,
                OpCode::ScopeUp => f.write_fmt(::core::fmt::Arguments::new_v1(
                    &["ScopeUp\n"],
                    &match () {
                        () => [],
                    },
                ))?,
                OpCode::ScopeDown => f.write_fmt(::core::fmt::Arguments::new_v1(
                    &["ScopeDown\n"],
                    &match () {
                        () => [],
                    },
                ))?,
                OpCode::Return => f.write_fmt(::core::fmt::Arguments::new_v1(
                    &["Return\n"],
                    &match () {
                        () => [],
                    },
                ))?,
                OpCode::ConditionalJump => {
                    let jump_index = code_iter.next().unwrap().1.as_val();
                    f.write_fmt(::core::fmt::Arguments::new_v1(
                        &["CndJmp ", "\n"],
                        &match (&jump_index,) {
                            (arg0,) => [::core::fmt::ArgumentV1::new(
                                arg0,
                                ::core::fmt::Display::fmt,
                            )],
                        },
                    ))?;
                }
                OpCode::ComparisonJump => {
                    let greater_jump_index = code_iter.next().unwrap().1.as_val();
                    let less_jump_index = code_iter.next().unwrap().1.as_val();
                    let equal_jump_index = code_iter.next().unwrap().1.as_val();
                    f.write_fmt(::core::fmt::Arguments::new_v1(
                        &["CmpJmp G:", " L:", " E:", "\n"],
                        &match (&greater_jump_index, &less_jump_index, &equal_jump_index) {
                            (arg0, arg1, arg2) => [
                                ::core::fmt::ArgumentV1::new(arg0, ::core::fmt::Display::fmt),
                                ::core::fmt::ArgumentV1::new(arg1, ::core::fmt::Display::fmt),
                                ::core::fmt::ArgumentV1::new(arg2, ::core::fmt::Display::fmt),
                            ],
                        },
                    ))?;
                }
                OpCode::Jump => {
                    let jump_index = code_iter.next().unwrap().1.as_val();
                    f.write_fmt(::core::fmt::Arguments::new_v1(
                        &["Jmp ", "\n"],
                        &match (&jump_index,) {
                            (arg0,) => [::core::fmt::ArgumentV1::new(
                                arg0,
                                ::core::fmt::Display::fmt,
                            )],
                        },
                    ))?;
                }
                OpCode::Pop => f.write_fmt(::core::fmt::Arguments::new_v1(
                    &["Pop\n"],
                    &match () {
                        () => [],
                    },
                ))?,
                OpCode::IsNull => f.write_fmt(::core::fmt::Arguments::new_v1(
                    &["IsNull\n"],
                    &match () {
                        () => [],
                    },
                ))?,
                OpCode::Duplicate => f.write_fmt(::core::fmt::Arguments::new_v1(
                    &["Duplicate\n"],
                    &match () {
                        () => [],
                    },
                ))?,
                OpCode::AssignReference => {
                    let is_let = unsafe { code_iter.next().unwrap().1.let_assign };
                    let pool_index = code_iter.next().unwrap().1.as_val();
                    let pool_value = &func.references[pool_index];
                    f.write_fmt(::core::fmt::Arguments::new_v1(
                        &["AssignRef let:", " \"", "\"[", "]\n"],
                        &match (&is_let, &pool_value, &pool_index) {
                            (arg0, arg1, arg2) => [
                                ::core::fmt::ArgumentV1::new(arg0, ::core::fmt::Display::fmt),
                                ::core::fmt::ArgumentV1::new(arg1, ::core::fmt::Display::fmt),
                                ::core::fmt::ArgumentV1::new(arg2, ::core::fmt::Display::fmt),
                            ],
                        },
                    ))?;
                }
                OpCode::AssignField => {
                    let is_let = unsafe { code_iter.next().unwrap().1.let_assign };
                    let pool_index = code_iter.next().unwrap().1.as_val();
                    let pool_value = &func.references[pool_index];
                    f.write_fmt(::core::fmt::Arguments::new_v1(
                        &["AssignField let:", " \"", "\"[", "]\n"],
                        &match (&is_let, &pool_value, &pool_index) {
                            (arg0, arg1, arg2) => [
                                ::core::fmt::ArgumentV1::new(arg0, ::core::fmt::Display::fmt),
                                ::core::fmt::ArgumentV1::new(arg1, ::core::fmt::Display::fmt),
                                ::core::fmt::ArgumentV1::new(arg2, ::core::fmt::Display::fmt),
                            ],
                        },
                    ))?;
                }
            }
            Ok(())
        }
        impl<T> Function<T> {
            pub fn get_code(&self, index: usize) -> Option<OpCode> {
                self.code.get(index).map(|b| b.as_op())
            }
            pub fn get_val(&self, index: usize) -> usize {
                self.code[index].as_val()
            }
            pub fn get_cmp(&self, index: usize) -> Compare {
                self.code[index].as_cmp()
            }
            pub fn get_assign_type(&self, index: usize) -> bool {
                self.code[index].as_bool()
            }
            pub fn get_literal(&self, index: usize) -> Literal {
                self.literals[index].clone()
            }
            pub fn get_reference(&self, index: usize) -> String {
                self.references[index].clone()
            }
            pub fn get_function(&self, index: usize) -> &Function<T> {
                &self.sub_functions[index]
            }
            fn add_literal(&mut self, literal: Literal) -> usize {
                let exists = self
                    .literals
                    .iter()
                    .enumerate()
                    .find(|(_, existing)| &&literal == existing)
                    .map(|(index, _)| index);
                exists.unwrap_or_else(|| {
                    let index = self.literals.len();
                    self.literals.push(literal);
                    index
                })
            }
            fn add_reference(&mut self, reference: String) -> usize {
                let exists = self
                    .references
                    .iter()
                    .enumerate()
                    .find(|(_, existing)| &&reference == existing)
                    .map(|(index, _)| index);
                exists.unwrap_or_else(|| {
                    let index = self.references.len();
                    self.references.push(reference);
                    index
                })
            }
            fn set_jump(&mut self, index: usize, jump_to: usize) {
                self.code[index].value = jump_to as u64;
            }
            fn place_jump(&mut self, conditional: bool) -> usize {
                let op = if conditional {
                    OpCode::ConditionalJump
                } else {
                    OpCode::Jump
                };
                self.code.push(ByteCode::op(op));
                let index = self.current_index();
                self.code.push(ByteCode::zero());
                index
            }
            fn place_jump_to(&mut self, conditional: bool, jump_to: usize) {
                let op = if conditional {
                    OpCode::ConditionalJump
                } else {
                    OpCode::Jump
                };
                self.code.push(ByteCode::op(op));
                self.code.push(ByteCode::val(jump_to));
            }
            fn current_index(&self) -> usize {
                self.code.len()
            }
            fn with_args(args: Vec<String>, resolved: T) -> Function<T> {
                Function {
                    args,
                    literals: <[_]>::into_vec(box []),
                    references: <[_]>::into_vec(box []),
                    code: <[_]>::into_vec(box []),
                    sub_functions: <[_]>::into_vec(box []),
                    resolved,
                }
            }
        }
        pub fn linearize_file(file: ParsedFile, path: PathBuf) -> ByteCodeFile {
            let ParsedFile { expr, imports } = file;
            let func = linearize(expr, <[_]>::into_vec(box []));
            ByteCodeFile {
                file: path,
                base_func: func,
                imports,
            }
        }
        fn linearize(expr: ExpRef, args: Vec<String>) -> Function<()> {
            let mut code = Function::<()>::with_args(args, ());
            linearize_exp_ref(expr, &mut code, false);
            code
        }
        fn linearize_exp_ref(exp_ref: ExpRef, func: &mut Function<()>, expand_stack: bool) {
            match *exp_ref {
                Eval::Branch(branch) => {
                    if !!expand_stack {
                        {
                            ::std::rt::begin_panic(
                                "assertion failed: !expand_stack",
                                &("pusl_lang/src/backend/linearize.rs", 462u32, 13u32),
                            )
                        }
                    };
                    linearize_branch(branch, func)
                }
                Eval::Expression(expr) => linearize_expr(expr, func, expand_stack),
            }
        }
        fn linearize_expr(expr: Expression, func: &mut Function<()>, expand_stack: bool) {
            let created_value = match expr {
                Expression::Modulus { lhs, rhs } => {
                    linearize_exp_ref(lhs, func, true);
                    linearize_exp_ref(rhs, func, true);
                    func.code.push(ByteCode::op(OpCode::Modulus));
                    true
                }
                Expression::Literal { value } => {
                    let literal_index = func.add_literal(value);
                    func.code.push(ByteCode::op(OpCode::Literal));
                    func.code.push(ByteCode::val(literal_index));
                    true
                }
                Expression::Reference { target } => {
                    let reference_index = func.add_reference(target);
                    func.code.push(ByteCode::op(OpCode::PushReference));
                    func.code.push(ByteCode::val(reference_index));
                    true
                }
                Expression::Joiner { expressions } => {
                    if !!expand_stack {
                        {
                            ::std::rt::begin_panic(
                                "assertion failed: !expand_stack",
                                &("pusl_lang/src/backend/linearize.rs", 490u32, 13u32),
                            )
                        }
                    };
                    expressions
                        .into_iter()
                        .for_each(|expr| linearize_exp_ref(expr, func, false));
                    false
                }
                Expression::FunctionCall { target, arguments } => {
                    func.code.push(ByteCode::op(OpCode::PushReference));
                    let pool_index = func.add_reference(target);
                    func.code.push(ByteCode::val(pool_index));
                    let num_args = arguments.len();
                    arguments
                        .into_iter()
                        .for_each(|argument| linearize_exp_ref(argument, func, true));
                    func.code.push(ByteCode::op(OpCode::FunctionCall));
                    func.code.push(ByteCode::val(num_args));
                    true
                }
                Expression::MethodCall {
                    target,
                    field,
                    arguments,
                } => {
                    linearize_exp_ref(target, func, true);
                    func.code.push(ByteCode::op(OpCode::Duplicate));
                    func.code.push(ByteCode::op(OpCode::FieldAccess));
                    let pool_index = func.add_reference(field);
                    func.code.push(ByteCode::val(pool_index));
                    let num_args = arguments.len();
                    arguments
                        .into_iter()
                        .for_each(|argument| linearize_exp_ref(argument, func, true));
                    func.code.push(ByteCode::op(OpCode::MethodCall));
                    func.code.push(ByteCode::val(num_args));
                    true
                }
                Expression::FieldAccess { target, name } => {
                    linearize_exp_ref(target, func, true);
                    let reference_index = func.add_reference(name);
                    func.code.push(ByteCode::op(OpCode::FieldAccess));
                    func.code.push(ByteCode::val(reference_index));
                    true
                }
                Expression::Addition { lhs, rhs } => {
                    linearize_exp_ref(lhs, func, true);
                    linearize_exp_ref(rhs, func, true);
                    func.code.push(ByteCode::op(OpCode::Addition));
                    true
                }
                Expression::Subtract { lhs, rhs } => {
                    linearize_exp_ref(lhs, func, true);
                    linearize_exp_ref(rhs, func, true);
                    func.code.push(ByteCode::op(OpCode::Subtraction));
                    true
                }
                Expression::Negate { operand } => {
                    linearize_exp_ref(operand, func, true);
                    func.code.push(ByteCode::op(OpCode::Negate));
                    true
                }
                Expression::Multiply { lhs, rhs } => {
                    linearize_exp_ref(lhs, func, true);
                    linearize_exp_ref(rhs, func, true);
                    func.code.push(ByteCode::op(OpCode::Multiply));
                    true
                }
                Expression::Divide { lhs, rhs } => {
                    linearize_exp_ref(lhs, func, true);
                    linearize_exp_ref(rhs, func, true);
                    func.code.push(ByteCode::op(OpCode::Multiply));
                    true
                }
                Expression::Elvis { lhs, rhs } => {
                    linearize_exp_ref(lhs, func, true);
                    func.code.push(ByteCode::op(OpCode::Duplicate));
                    func.code.push(ByteCode::op(OpCode::IsNull));
                    func.code.push(ByteCode::op(OpCode::Negate));
                    let use_first_index = func.place_jump(true);
                    func.code.push(ByteCode::op(OpCode::Pop));
                    linearize_exp_ref(rhs, func, true);
                    let current_index = func.current_index();
                    func.set_jump(use_first_index, current_index);
                    true
                }
                Expression::ReferenceAssigment {
                    target,
                    expression,
                    flags,
                } => {
                    let target_index = func.add_reference(target);
                    let skip_index_option = if flags.intersects(AssignmentFlags::CONDITIONAL) {
                        func.code.push(ByteCode::op(OpCode::PushReference));
                        func.code.push(ByteCode::val(target_index));
                        func.code.push(ByteCode::op(OpCode::IsNull));
                        func.code.push(ByteCode::op(OpCode::Negate));
                        Some(func.place_jump(true))
                    } else {
                        None
                    };
                    linearize_exp_ref(expression, func, true);
                    func.code.push(ByteCode::op(OpCode::AssignReference));
                    func.code.push(ByteCode {
                        let_assign: flags.intersects(AssignmentFlags::LET),
                    });
                    func.code.push(ByteCode::val(target_index));
                    if let Some(jump_instruction) = skip_index_option {
                        func.set_jump(jump_instruction, func.current_index());
                    }
                    false
                }
                Expression::FieldAssignment {
                    target,
                    field,
                    expression,
                    flags,
                } => {
                    linearize_exp_ref(target, func, true);
                    let target_index = func.add_reference(field);
                    let skip_index_option = if flags.intersects(AssignmentFlags::CONDITIONAL) {
                        func.code.push(ByteCode::op(OpCode::Duplicate));
                        func.code.push(ByteCode::op(OpCode::FieldAccess));
                        func.code.push(ByteCode::val(target_index));
                        func.code.push(ByteCode::op(OpCode::IsNull));
                        func.code.push(ByteCode::op(OpCode::Negate));
                        Some(func.place_jump(true))
                    } else {
                        None
                    };
                    linearize_exp_ref(expression, func, true);
                    func.code.push(ByteCode::op(OpCode::AssignField));
                    func.code.push(ByteCode {
                        let_assign: flags.intersects(AssignmentFlags::LET),
                    });
                    func.code.push(ByteCode::val(target_index));
                    if let Some(jump_instruction) = skip_index_option {
                        func.set_jump(jump_instruction, func.current_index());
                    }
                    false
                }
                Expression::DivideTruncate { lhs, rhs } => {
                    linearize_exp_ref(lhs, func, true);
                    linearize_exp_ref(rhs, func, true);
                    func.code.push(ByteCode::op(OpCode::DivideTruncate));
                    true
                }
                Expression::Exponent { lhs, rhs } => {
                    linearize_exp_ref(lhs, func, true);
                    linearize_exp_ref(rhs, func, true);
                    func.code.push(ByteCode::op(OpCode::Exponent));
                    true
                }
                Expression::Compare {
                    lhs,
                    rhs,
                    operation,
                } => {
                    linearize_exp_ref(lhs, func, true);
                    linearize_exp_ref(rhs, func, true);
                    func.code.push(ByteCode::op(OpCode::Compare));
                    func.code.push(ByteCode { compare: operation });
                    true
                }
                Expression::And { lhs, rhs } => {
                    linearize_exp_ref(lhs, func, true);
                    linearize_exp_ref(rhs, func, true);
                    func.code.push(ByteCode::op(OpCode::And));
                    true
                }
                Expression::Or { lhs, rhs } => {
                    linearize_exp_ref(lhs, func, true);
                    linearize_exp_ref(rhs, func, true);
                    func.code.push(ByteCode::op(OpCode::Or));
                    true
                }
                Expression::FunctionDeclaration { params, body } => {
                    let new_func = linearize(body, params);
                    let index = func.sub_functions.len();
                    func.sub_functions.push(new_func);
                    func.code.push(ByteCode::op(OpCode::PushFunction));
                    func.code.push(ByteCode::val(index));
                    true
                }
                Expression::Return { value } => {
                    linearize_exp_ref(value, func, true);
                    func.code.push(ByteCode::op(OpCode::Return));
                    false
                }
            };
            match (expand_stack, created_value) {
                (true, false) => ::std::rt::begin_panic(
                    "explicit panic",
                    &("pusl_lang/src/backend/linearize.rs", 681u32, 26u32),
                ),
                (false, true) => func.code.push(ByteCode::op(OpCode::Pop)),
                _ => {}
            }
        }
        fn linearize_branch(branch: Branch, func: &mut Function<()>) {
            match branch {
                Branch::WhileLoop { condition, body } => linearize_while(condition, body, func),
                Branch::IfElseBlock { conditions, last } => {
                    linearize_if_else(conditions, last, func)
                }
                Branch::CompareBlock {
                    lhs,
                    rhs,
                    greater,
                    equal,
                    less,
                    body,
                } => linearize_compare(lhs, rhs, greater, equal, less, body, func),
                _ => ::std::rt::begin_panic(
                    "explicit panic",
                    &("pusl_lang/src/backend/linearize.rs", 699u32, 14u32),
                ),
            }
        }
        fn linearize_compare(
            lhs: ExpRef,
            rhs: ExpRef,
            greater: u8,
            equal: u8,
            less: u8,
            body: Vec<ExpRef>,
            func: &mut Function<()>,
        ) {
            linearize_exp_ref(lhs, func, true);
            linearize_exp_ref(rhs, func, true);
            func.code.push(ByteCode::op(OpCode::ComparisonJump));
            let jump_table = func.current_index();
            func.code.push(ByteCode::zero());
            func.code.push(ByteCode::zero());
            func.code.push(ByteCode::zero());
            let indexes = body
                .into_iter()
                .map(|expr| {
                    let start_index = func.current_index();
                    func.code.push(ByteCode::op(OpCode::ScopeUp));
                    linearize_exp_ref(expr, func, false);
                    func.code.push(ByteCode::op(OpCode::ScopeDown));
                    let jump_out_index = func.place_jump(false);
                    (start_index, jump_out_index)
                })
                .collect::<Vec<_>>();
            func.code[jump_table + 0].value = indexes[greater as usize].0 as u64;
            func.code[jump_table + 1].value = indexes[less as usize].0 as u64;
            func.code[jump_table + 2].value = indexes[equal as usize].0 as u64;
            let jump_out_to = func.current_index();
            indexes.into_iter().for_each(|(_, jump_out_index)| {
                func.code[jump_out_index].value = jump_out_to as u64
            });
        }
        fn linearize_if_else(
            conditions: Vec<ConditionBody>,
            last: Option<ExpRef>,
            func: &mut Function<()>,
        ) {
            let place_conditions = conditions
                .into_iter()
                .map(|ConditionBody { condition, body }| {
                    linearize_exp_ref(condition, func, true);
                    let jump_index = func.place_jump(true);
                    (jump_index, body)
                })
                .collect::<Vec<_>>();
            if let Some(else_expr) = last {
                func.code.push(ByteCode::op(OpCode::ScopeUp));
                linearize_exp_ref(else_expr, func, false);
                func.code.push(ByteCode::op(OpCode::ScopeDown));
            }
            let jump_to_end_index = func.place_jump(false);
            let place_bodies = place_conditions
                .into_iter()
                .map(|(jump_index, body)| {
                    let jump_to = func.current_index();
                    func.set_jump(jump_index, jump_to);
                    func.code.push(ByteCode::op(OpCode::ScopeUp));
                    linearize_exp_ref(body, func, false);
                    func.code.push(ByteCode::op(OpCode::ScopeDown));
                    let jump_to_end_index = func.place_jump(false);
                    jump_to_end_index
                })
                .collect::<Vec<_>>();
            let jump_to = func.current_index();
            place_bodies.into_iter().for_each(|jump_index| {
                func.set_jump(jump_index, jump_to);
            });
            func.set_jump(jump_to_end_index, jump_to);
        }
        fn linearize_while(condition: ExpRef, body: ExpRef, func: &mut Function<()>) {
            let begin_index = func.current_index();
            linearize_exp_ref(condition, func, true);
            func.code.push(ByteCode::op(OpCode::Negate));
            let condition_jump_index = func.place_jump(true);
            func.code.push(ByteCode::op(OpCode::ScopeUp));
            linearize_exp_ref(body, func, false);
            func.code.push(ByteCode::op(OpCode::ScopeDown));
            func.place_jump_to(false, begin_index);
            let end_index = func.current_index();
            func.set_jump(condition_jump_index, end_index);
        }
    }
    pub mod object {
        use bitflags::_core::cell::RefCell;
        use bitflags::_core::fmt::Formatter;
        use garbage::{GcPointer, MarkTrace};
        use std::collections::HashMap;
        use std::fmt;
        use std::fmt::Display;
        use crate::backend::RFunction;
        pub type ObjectPtr = GcPointer<RefCell<Object>>;
        pub enum Value {
            Null,
            Boolean(bool),
            Integer(i64),
            Float(f64),
            String(GcPointer<String>),
            Function(&'static RFunction),
            Native(fn(Vec<Value>, Option<ObjectPtr>) -> Value),
            Object(ObjectPtr),
        }
        #[automatically_derived]
        #[allow(unused_qualifications)]
        impl ::core::clone::Clone for Value {
            #[inline]
            fn clone(&self) -> Value {
                match (&*self,) {
                    (&Value::Null,) => Value::Null,
                    (&Value::Boolean(ref __self_0),) => {
                        Value::Boolean(::core::clone::Clone::clone(&(*__self_0)))
                    }
                    (&Value::Integer(ref __self_0),) => {
                        Value::Integer(::core::clone::Clone::clone(&(*__self_0)))
                    }
                    (&Value::Float(ref __self_0),) => {
                        Value::Float(::core::clone::Clone::clone(&(*__self_0)))
                    }
                    (&Value::String(ref __self_0),) => {
                        Value::String(::core::clone::Clone::clone(&(*__self_0)))
                    }
                    (&Value::Function(ref __self_0),) => {
                        Value::Function(::core::clone::Clone::clone(&(*__self_0)))
                    }
                    (&Value::Native(ref __self_0),) => {
                        Value::Native(::core::clone::Clone::clone(&(*__self_0)))
                    }
                    (&Value::Object(ref __self_0),) => {
                        Value::Object(::core::clone::Clone::clone(&(*__self_0)))
                    }
                }
            }
        }
        #[automatically_derived]
        #[allow(unused_qualifications)]
        impl ::core::fmt::Debug for Value {
            fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                match (&*self,) {
                    (&Value::Null,) => {
                        let mut debug_trait_builder = f.debug_tuple("Null");
                        debug_trait_builder.finish()
                    }
                    (&Value::Boolean(ref __self_0),) => {
                        let mut debug_trait_builder = f.debug_tuple("Boolean");
                        let _ = debug_trait_builder.field(&&(*__self_0));
                        debug_trait_builder.finish()
                    }
                    (&Value::Integer(ref __self_0),) => {
                        let mut debug_trait_builder = f.debug_tuple("Integer");
                        let _ = debug_trait_builder.field(&&(*__self_0));
                        debug_trait_builder.finish()
                    }
                    (&Value::Float(ref __self_0),) => {
                        let mut debug_trait_builder = f.debug_tuple("Float");
                        let _ = debug_trait_builder.field(&&(*__self_0));
                        debug_trait_builder.finish()
                    }
                    (&Value::String(ref __self_0),) => {
                        let mut debug_trait_builder = f.debug_tuple("String");
                        let _ = debug_trait_builder.field(&&(*__self_0));
                        debug_trait_builder.finish()
                    }
                    (&Value::Function(ref __self_0),) => {
                        let mut debug_trait_builder = f.debug_tuple("Function");
                        let _ = debug_trait_builder.field(&&(*__self_0));
                        debug_trait_builder.finish()
                    }
                    (&Value::Native(ref __self_0),) => {
                        let mut debug_trait_builder = f.debug_tuple("Native");
                        let _ = debug_trait_builder.field(&&(*__self_0));
                        debug_trait_builder.finish()
                    }
                    (&Value::Object(ref __self_0),) => {
                        let mut debug_trait_builder = f.debug_tuple("Object");
                        let _ = debug_trait_builder.field(&&(*__self_0));
                        debug_trait_builder.finish()
                    }
                }
            }
        }
        impl Display for Value {
            fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
                match self {
                    Value::Null => f.write_fmt(::core::fmt::Arguments::new_v1(
                        &["null"],
                        &match () {
                            () => [],
                        },
                    ))?,
                    Value::Boolean(val) => f.write_fmt(::core::fmt::Arguments::new_v1(
                        &[""],
                        &match (&val,) {
                            (arg0,) => [::core::fmt::ArgumentV1::new(
                                arg0,
                                ::core::fmt::Display::fmt,
                            )],
                        },
                    ))?,
                    Value::Integer(val) => f.write_fmt(::core::fmt::Arguments::new_v1(
                        &[""],
                        &match (&val,) {
                            (arg0,) => [::core::fmt::ArgumentV1::new(
                                arg0,
                                ::core::fmt::Display::fmt,
                            )],
                        },
                    ))?,
                    Value::Float(val) => f.write_fmt(::core::fmt::Arguments::new_v1(
                        &[""],
                        &match (&val,) {
                            (arg0,) => [::core::fmt::ArgumentV1::new(
                                arg0,
                                ::core::fmt::Display::fmt,
                            )],
                        },
                    ))?,
                    Value::String(val) => f.write_fmt(::core::fmt::Arguments::new_v1(
                        &[""],
                        &match (&**val,) {
                            (arg0,) => [::core::fmt::ArgumentV1::new(
                                arg0,
                                ::core::fmt::Display::fmt,
                            )],
                        },
                    ))?,
                    Value::Function(val) => f.write_fmt(::core::fmt::Arguments::new_v1(
                        &["Function "],
                        &match (&((*val) as *const _),) {
                            (arg0,) => [::core::fmt::ArgumentV1::new(
                                arg0,
                                ::core::fmt::Pointer::fmt,
                            )],
                        },
                    ))?,
                    Value::Native(val) => f.write_fmt(::core::fmt::Arguments::new_v1(
                        &["NativeFunc "],
                        &match (&*val,) {
                            (arg0,) => [::core::fmt::ArgumentV1::new(
                                arg0,
                                ::core::fmt::Pointer::fmt,
                            )],
                        },
                    ))?,
                    Value::Object(val) => {
                        f.write_fmt(::core::fmt::Arguments::new_v1(
                            &["Object "],
                            &match () {
                                () => [],
                            },
                        ))?;
                        (*val).write_addr(f)?;
                    }
                }
                Ok(())
            }
        }
        impl Value {
            pub fn type_string(&self) -> &'static str {
                match self {
                    Value::Null => "Null",
                    Value::Boolean(_) => "Boolean",
                    Value::Integer(_) => "Integer",
                    Value::Float(_) => "Float",
                    Value::String(_) => "String",
                    Value::Function(_) => "Function",
                    Value::Object(_) => "Object",
                    Value::Native(_) => "Native Function",
                }
            }
        }
        impl MarkTrace for Value {
            fn mark_children(&self) {
                match self {
                    Value::Object(object) => object.mark_children(),
                    _ => {}
                }
            }
        }
        impl MarkTrace for Object {
            fn mark_children(&self) {
                if let Some(super_ptr) = &self.super_ptr {
                    super_ptr.mark_recurse();
                }
                self.fields.iter().for_each(|(_, v)| {
                    if let Value::Object(ptr) = v {
                        ptr.mark_recurse();
                    }
                })
            }
        }
        pub struct Object {
            super_ptr: Option<ObjectPtr>,
            fields: HashMap<String, Value>,
        }
        #[automatically_derived]
        #[allow(unused_qualifications)]
        impl ::core::fmt::Debug for Object {
            fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                match *self {
                    Object {
                        super_ptr: ref __self_0_0,
                        fields: ref __self_0_1,
                    } => {
                        let mut debug_trait_builder = f.debug_struct("Object");
                        let _ = debug_trait_builder.field("super_ptr", &&(*__self_0_0));
                        let _ = debug_trait_builder.field("fields", &&(*__self_0_1));
                        debug_trait_builder.finish()
                    }
                }
            }
        }
        impl Object {
            pub fn new() -> RefCell<Self> {
                let object = Object {
                    super_ptr: None,
                    fields: HashMap::new(),
                };
                RefCell::new(object)
            }
            pub fn new_with_parent(parent: ObjectPtr) -> RefCell<Self> {
                let object = Object {
                    super_ptr: Some(parent),
                    fields: HashMap::new(),
                };
                RefCell::new(object)
            }
            pub fn get_field(&self, name: &str) -> Value {
                self.fields
                    .get(name)
                    .map(|val| (*val).clone())
                    .unwrap_or(Value::Null)
            }
            pub fn let_field(&mut self, name: String, value: Value) {
                self.fields.insert(name, value);
            }
            pub fn assign_field(&mut self, name: &str, value: Value) {
                let entry = self.fields.get_mut(name);
                if let Some(old_value) = entry {
                    *old_value = value;
                } else {
                    {
                        ::std::rt::begin_panic(
                            "Cannot assign to non-existent field without let",
                            &("pusl_lang/src/backend/object.rs", 121u32, 13u32),
                        )
                    }
                }
            }
        }
    }
    pub type RFunction = Function<&'static Vec<(String, ObjectPtr)>>;
    enum VariableStack {
        Variable(Variable),
        ScopeBoundary,
    }
    struct Variable {
        value: Value,
        name: String,
    }
    struct StackFrame {
        this_obj: Option<GcPointer<RefCell<Object>>>,
        function: &'static RFunction,
        variables: Vec<VariableStack>,
        op_stack: Vec<Value>,
        index: usize,
    }
    impl StackFrame {
        fn from_function(function: &'static RFunction) -> Self {
            StackFrame {
                this_obj: None,
                function,
                variables: <[_]>::into_vec(box []),
                op_stack: <[_]>::into_vec(box []),
                index: 0,
            }
        }
        fn from_method(function: &'static RFunction, this_obj: ObjectPtr) -> Self {
            StackFrame {
                this_obj: Some(this_obj),
                function,
                variables: <[_]>::into_vec(box []),
                op_stack: <[_]>::into_vec(box []),
                index: 0,
            }
        }
        fn from_file(function: &'static RFunction) -> (Self, ObjectPtr) {
            let new_object = GC.with(|gc| gc.borrow_mut().place_in_heap(Object::new()));
            let frame = StackFrame {
                this_obj: Some(new_object.clone()),
                function,
                variables: <[_]>::into_vec(box []),
                op_stack: <[_]>::into_vec(box []),
                index: 0,
            };
            (frame, new_object)
        }
        pub fn get_code(&mut self) -> Option<OpCode> {
            let code = self.function.get_code(self.index);
            self.index += 1;
            code
        }
        pub fn get_val(&mut self) -> usize {
            let value = self.function.get_val(self.index);
            self.index += 1;
            value
        }
        pub fn get_cmp(&mut self) -> Compare {
            let value = self.function.get_cmp(self.index);
            self.index += 1;
            value
        }
        pub fn get_assign_type(&mut self) -> bool {
            let value = self.function.get_assign_type(self.index);
            self.index += 1;
            value
        }
    }
    pub const GC: ::std::thread::LocalKey<RefCell<ManagedPool>> = {
        #[inline]
        fn __init() -> RefCell<ManagedPool> {
            RefCell::new(ManagedPool::new())
        }
        unsafe fn __getit() -> ::std::option::Option<&'static RefCell<ManagedPool>> {
            #[thread_local]
            #[cfg(all(
                target_thread_local,
                not(all(target_arch = "wasm32", not(target_feature = "atomics"))),
            ))]
            static __KEY: ::std::thread::__FastLocalKeyInner<RefCell<ManagedPool>> =
                ::std::thread::__FastLocalKeyInner::new();
            __KEY.get(__init)
        }
        unsafe { ::std::thread::LocalKey::new(__getit) }
    };
    pub const STDOUT: ::std::thread::LocalKey<RefCell<Box<dyn io::Write>>> = {
        #[inline]
        fn __init() -> RefCell<Box<dyn io::Write>> {
            RefCell::new(Box::new(io::stdout()))
        }
        unsafe fn __getit() -> ::std::option::Option<&'static RefCell<Box<dyn io::Write>>> {
            #[thread_local]
            #[cfg(all(
                target_thread_local,
                not(all(target_arch = "wasm32", not(target_feature = "atomics"))),
            ))]
            static __KEY: ::std::thread::__FastLocalKeyInner<RefCell<Box<dyn io::Write>>> =
                ::std::thread::__FastLocalKeyInner::new();
            __KEY.get(__init)
        }
        unsafe { ::std::thread::LocalKey::new(__getit) }
    };
    pub type GcPoolRef = &'static LocalKey<RefCell<ManagedPool>>;
    pub struct ExecContext {
        pub stdout: Option<Box<dyn io::Write>>,
        pub resolve: fn(PathBuf) -> Option<ByteCodeFile>,
    }
    impl Default for ExecContext {
        fn default() -> Self {
            ExecContext {
                stdout: None,
                resolve: |_| None,
            }
        }
    }
    fn process_bcf(
        bcf: ByteCodeFile,
        resolved_imports: &mut Vec<(PathBuf, ObjectPtr)>,
    ) -> StackFrame {
        let ByteCodeFile {
            file,
            base_func,
            imports,
        } = bcf;
        let base_func = resolve(base_func, resolved_imports as &_, imports);
        let (current_frame, import_object) = StackFrame::from_file(base_func);
        resolved_imports.push((file, import_object));
        current_frame
    }
    pub fn execute(main: ByteCodeFile, ctx: ExecContext) {
        let ExecContext { stdout, resolve } = ctx;
        if let Some(new_out) = stdout {
            STDOUT.with(|stdout| *stdout.borrow_mut() = new_out);
        }
        let mut resolved_imports = Vec::<(PathBuf, ObjectPtr)>::new();
        let mut resolve_stack = <[_]>::into_vec(box [main]);
        let mut index = 0;
        while index < resolve_stack.len() {
            let mut append = Vec::new();
            for import in &resolve_stack[index].imports {
                if !resolve_stack.iter().any(|bcf| bcf.file == import.path) {
                    let new_bcf = resolve(import.path.clone()).expect(
                        {
                            let res = ::alloc::fmt::format(::core::fmt::Arguments::new_v1(
                                &["Unable to resolve import "],
                                &match (&import.path.display(),) {
                                    (arg0,) => [::core::fmt::ArgumentV1::new(
                                        arg0,
                                        ::core::fmt::Display::fmt,
                                    )],
                                },
                            ));
                            res
                        }
                        .as_str(),
                    );
                    append.push(new_bcf);
                }
            }
            resolve_stack.append(&mut append);
            index += 1;
        }
        let mut ex_stack = Vec::<StackFrame>::new();
        let top = resolve_stack.pop().unwrap();
        let mut current_frame = process_bcf(top, &mut resolved_imports);
        loop {
            let current_op = if let Some(op) = current_frame.get_code() {
                op
            } else {
                if let Some(mut parent_frame) = ex_stack.pop() {
                    parent_frame.op_stack.push(Value::Null);
                    current_frame = parent_frame;
                    continue;
                } else if let Some(parent_frame) = resolve_stack.pop() {
                    current_frame = process_bcf(parent_frame, &mut resolved_imports);
                    continue;
                } else {
                    return;
                }
            };
            match current_op {
                OpCode::Modulus => {
                    let rhs = current_frame.op_stack.pop().unwrap();
                    let lhs = current_frame.op_stack.pop().unwrap();
                    current_frame.op_stack.push(modulus(lhs, rhs));
                }
                OpCode::Literal => {
                    let pool_index = current_frame.get_val();
                    current_frame.op_stack.push(
                        current_frame
                            .function
                            .get_literal(pool_index)
                            .into_value(&GC),
                    )
                }
                OpCode::PushReference => {
                    let pool_index = current_frame.get_val();
                    let reference_name = current_frame.function.get_reference(pool_index);
                    let value = current_frame
                        .variables
                        .iter_mut()
                        .rev()
                        .filter_map(|var_stack| {
                            if let VariableStack::Variable(var) = var_stack {
                                Some(var)
                            } else {
                                None
                            }
                        })
                        .find(|var| var.name == reference_name)
                        .map(|var| var.value.clone())
                        .or_else(|| {
                            if reference_name.as_str() == "typeof" {
                                Some(Value::Native(type_of))
                            } else {
                                None
                            }
                        })
                        .or_else(|| {
                            if reference_name.as_str() == "self" {
                                current_frame
                                    .this_obj
                                    .clone()
                                    .map(|ptr| Value::Object(ptr))
                                    .or(Some(Value::Null))
                            } else {
                                None
                            }
                        })
                        .or_else(|| {
                            if reference_name.as_str() == "print" {
                                Some(Value::Native(print))
                            } else {
                                None
                            }
                        })
                        .or_else(|| {
                            if reference_name.as_str() == "Object" {
                                Some(Value::Native(new_object))
                            } else {
                                None
                            }
                        })
                        .expect(
                            {
                                let res = ::alloc::fmt::format(::core::fmt::Arguments::new_v1(
                                    &["Undeclared Variable \"", "\""],
                                    &match (&reference_name,) {
                                        (arg0,) => [::core::fmt::ArgumentV1::new(
                                            arg0,
                                            ::core::fmt::Display::fmt,
                                        )],
                                    },
                                ));
                                res
                            }
                            .as_str(),
                        );
                    current_frame.op_stack.push(value);
                }
                OpCode::PushFunction => {
                    let pool_index = current_frame.get_val();
                    current_frame.op_stack.push(Value::Function(
                        current_frame.function.get_function(pool_index),
                    ));
                }
                OpCode::FunctionCall => {
                    let num_args = current_frame.get_val();
                    let mut args = Vec::with_capacity(num_args);
                    for _ in 0..num_args {
                        args.push(current_frame.op_stack.pop().unwrap());
                    }
                    let function = current_frame.op_stack.pop().unwrap();
                    match function {
                        Value::Function(reference) => {
                            {
                                match (&reference.args.len(), &args.len()) {
                                    (left_val, right_val) => {
                                        if !(*left_val == *right_val) {
                                            {
                                                :: std :: rt :: begin_panic_fmt ( & :: core :: fmt :: Arguments :: new_v1 ( & [ "assertion failed: `(left == right)`\n  left: `" , "`,\n right: `" , "`" ] , & match ( & & * left_val , & & * right_val ) { ( arg0 , arg1 ) => [ :: core :: fmt :: ArgumentV1 :: new ( arg0 , :: core :: fmt :: Debug :: fmt ) , :: core :: fmt :: ArgumentV1 :: new ( arg1 , :: core :: fmt :: Debug :: fmt ) ] , } ) , & ( "pusl_lang/src/backend/mod.rs" , 241u32 , 25u32 ) )
                                            }
                                        }
                                    }
                                }
                            };
                            let mut new_frame = StackFrame::from_function(reference);
                            for name in reference.args.iter().cloned() {
                                let value =
                                    args.pop().expect("Wrong Number of arguments for function");
                                new_frame
                                    .variables
                                    .push(VariableStack::Variable(Variable { value, name }));
                            }
                            if !args.is_empty() {
                                {
                                    ::std::rt::begin_panic(
                                        "Wrong number of arguments for function",
                                        &("pusl_lang/src/backend/mod.rs", 249u32, 25u32),
                                    )
                                }
                            };
                            let old_frame = std::mem::replace(&mut current_frame, new_frame);
                            ex_stack.push(old_frame);
                        }
                        Value::Native(ptr) => {
                            let result = ptr(args, None);
                            current_frame.op_stack.push(result);
                        }
                        _ => ::std::rt::begin_panic(
                            "Value must be a function to call",
                            &("pusl_lang/src/backend/mod.rs", 257u32, 26u32),
                        ),
                    };
                }
                OpCode::MethodCall => {
                    let num_args = current_frame.get_val();
                    let mut args = Vec::with_capacity(num_args);
                    for _ in 0..num_args {
                        args.push(current_frame.op_stack.pop().unwrap());
                    }
                    let function = current_frame.op_stack.pop().unwrap();
                    let this_obj = if let Value::Object(ptr) = current_frame.op_stack.pop().unwrap()
                    {
                        ptr
                    } else {
                        {
                            ::std::rt::begin_panic(
                                "Cannot call method on Non Object",
                                &("pusl_lang/src/backend/mod.rs", 270u32, 21u32),
                            )
                        }
                    };
                    match function {
                        Value::Function(reference) => {
                            {
                                match (&reference.args.len(), &args.len()) {
                                    (left_val, right_val) => {
                                        if !(*left_val == *right_val) {
                                            {
                                                :: std :: rt :: begin_panic_fmt ( & :: core :: fmt :: Arguments :: new_v1 ( & [ "assertion failed: `(left == right)`\n  left: `" , "`,\n right: `" , "`" ] , & match ( & & * left_val , & & * right_val ) { ( arg0 , arg1 ) => [ :: core :: fmt :: ArgumentV1 :: new ( arg0 , :: core :: fmt :: Debug :: fmt ) , :: core :: fmt :: ArgumentV1 :: new ( arg1 , :: core :: fmt :: Debug :: fmt ) ] , } ) , & ( "pusl_lang/src/backend/mod.rs" , 274u32 , 25u32 ) )
                                            }
                                        }
                                    }
                                }
                            };
                            let mut new_frame = StackFrame::from_method(reference, this_obj);
                            for name in reference.args.iter().cloned() {
                                let value =
                                    args.pop().expect("Wrong Number of arguments for function");
                                new_frame
                                    .variables
                                    .push(VariableStack::Variable(Variable { value, name }));
                            }
                            if !args.is_empty() {
                                {
                                    ::std::rt::begin_panic(
                                        "Wrong number of arguments for function",
                                        &("pusl_lang/src/backend/mod.rs", 282u32, 25u32),
                                    )
                                }
                            };
                            let old_frame = std::mem::replace(&mut current_frame, new_frame);
                            ex_stack.push(old_frame);
                        }
                        Value::Native(ptr) => {
                            let result = ptr(args, Some(this_obj));
                            current_frame.op_stack.push(result);
                        }
                        _ => ::std::rt::begin_panic(
                            "Value must be a function to call",
                            &("pusl_lang/src/backend/mod.rs", 290u32, 26u32),
                        ),
                    };
                }
                OpCode::FieldAccess => {
                    let object = if let Value::Object(ptr) = current_frame.op_stack.pop().unwrap() {
                        ptr
                    } else {
                        {
                            ::std::rt::begin_panic(
                                "Cannot Access Field of non-object value",
                                &("pusl_lang/src/backend/mod.rs", 297u32, 21u32),
                            )
                        }
                    };
                    let name_index = current_frame.get_val();
                    let name = current_frame.function.get_reference(name_index);
                    current_frame
                        .op_stack
                        .push((*object).borrow_mut().get_field(name.as_str()));
                }
                OpCode::Addition => {
                    let rhs = current_frame.op_stack.pop().unwrap();
                    let lhs = current_frame.op_stack.pop().unwrap();
                    current_frame.op_stack.push(addition(lhs, rhs));
                }
                OpCode::Subtraction => {
                    let rhs = current_frame.op_stack.pop().unwrap();
                    let lhs = current_frame.op_stack.pop().unwrap();
                    current_frame.op_stack.push(subtraction(lhs, rhs));
                }
                OpCode::Negate => {
                    let operand = current_frame.op_stack.pop().unwrap();
                    current_frame.op_stack.push(negate(operand));
                }
                OpCode::Multiply => {
                    let rhs = current_frame.op_stack.pop().unwrap();
                    let lhs = current_frame.op_stack.pop().unwrap();
                    current_frame.op_stack.push(multiplication(lhs, rhs));
                }
                OpCode::Divide => {
                    let rhs = current_frame.op_stack.pop().unwrap();
                    let lhs = current_frame.op_stack.pop().unwrap();
                    current_frame.op_stack.push(division(lhs, rhs));
                }
                OpCode::DivideTruncate => {
                    let rhs = current_frame.op_stack.pop().unwrap();
                    let lhs = current_frame.op_stack.pop().unwrap();
                    current_frame.op_stack.push(truncate_division(lhs, rhs));
                }
                OpCode::Exponent => {
                    let rhs = current_frame.op_stack.pop().unwrap();
                    let lhs = current_frame.op_stack.pop().unwrap();
                    current_frame.op_stack.push(exponent(lhs, rhs));
                }
                OpCode::Compare => {
                    let op = current_frame.get_cmp();
                    let rhs = current_frame.op_stack.pop().unwrap();
                    let lhs = current_frame.op_stack.pop().unwrap();
                    current_frame.op_stack.push(compare(lhs, rhs, op));
                }
                OpCode::And => {
                    let rhs = current_frame.op_stack.pop().unwrap();
                    let lhs = current_frame.op_stack.pop().unwrap();
                    current_frame.op_stack.push(logic(lhs, rhs, true));
                }
                OpCode::Or => {
                    let rhs = current_frame.op_stack.pop().unwrap();
                    let lhs = current_frame.op_stack.pop().unwrap();
                    current_frame.op_stack.push(logic(lhs, rhs, false));
                }
                OpCode::ScopeUp => {
                    current_frame.variables.push(VariableStack::ScopeBoundary);
                }
                OpCode::ScopeDown => {
                    while let Some(VariableStack::Variable(_)) = current_frame.variables.pop() {}
                }
                OpCode::Return => {
                    let return_value = current_frame.op_stack.pop().unwrap();
                    if let Some(mut parent_frame) = ex_stack.pop() {
                        parent_frame.op_stack.push(return_value);
                        current_frame = parent_frame;
                        continue;
                    } else if let Some(parent_frame) = resolve_stack.pop() {
                        current_frame = process_bcf(parent_frame, &mut resolved_imports);
                        continue;
                    } else {
                        return;
                    }
                }
                OpCode::ConditionalJump => {
                    let jump_index = current_frame.get_val();
                    let condition =
                        if let Value::Boolean(val) = current_frame.op_stack.pop().unwrap() {
                            val
                        } else {
                            {
                                ::std::rt::begin_panic(
                                    "ConditionalJump expects boolean",
                                    &("pusl_lang/src/backend/mod.rs", 379u32, 21u32),
                                )
                            };
                        };
                    if condition {
                        current_frame.index = jump_index;
                    }
                }
                OpCode::ComparisonJump => {
                    let greater_index = current_frame.get_val();
                    let less_index = current_frame.get_val();
                    let equal_index = current_frame.get_val();
                    let rhs = current_frame.op_stack.pop().unwrap();
                    let lhs = current_frame.op_stack.pop().unwrap();
                    let ordering = compare_numerical(lhs, rhs);
                    let index = match ordering {
                        Ordering::Less => less_index,
                        Ordering::Equal => equal_index,
                        Ordering::Greater => greater_index,
                    };
                    current_frame.index = index;
                }
                OpCode::Jump => {
                    let jump_index = current_frame.get_val();
                    current_frame.index = jump_index;
                }
                OpCode::Pop => {
                    current_frame.op_stack.pop().unwrap();
                }
                OpCode::IsNull => {
                    let value = current_frame.op_stack.pop().unwrap();
                    let is_null = if let Value::Null = value { true } else { false };
                    current_frame.op_stack.push(Value::Boolean(is_null));
                }
                OpCode::Duplicate => {
                    let value = (*current_frame.op_stack.last().unwrap()).clone();
                    current_frame.op_stack.push(value);
                }
                OpCode::AssignReference => {
                    let is_let = current_frame.get_assign_type();
                    let pool_index = current_frame.get_val();
                    let reference_name = current_frame.function.get_reference(pool_index);
                    let value = current_frame.op_stack.pop().unwrap();
                    if is_let {
                        current_frame
                            .variables
                            .push(VariableStack::Variable(Variable {
                                value,
                                name: reference_name,
                            }))
                    } else {
                        let variable = current_frame
                            .variables
                            .iter_mut()
                            .rev()
                            .filter_map(|var_stack| {
                                if let VariableStack::Variable(var) = var_stack {
                                    Some(var)
                                } else {
                                    None
                                }
                            })
                            .find(|var| var.name == reference_name)
                            .expect("Non-Let Assignment on undeclared variable");
                        variable.value = value;
                    }
                }
                OpCode::AssignField => {
                    let is_let = current_frame.get_assign_type();
                    let pool_index = current_frame.get_val();
                    let reference_name = current_frame.function.get_reference(pool_index);
                    let value = current_frame.op_stack.pop().unwrap();
                    let object = if let Value::Object(ptr) = current_frame.op_stack.pop().unwrap() {
                        ptr
                    } else {
                        {
                            ::std::rt::begin_panic(
                                "Cannot Assign to non-object",
                                &("pusl_lang/src/backend/mod.rs", 452u32, 21u32),
                            )
                        }
                    };
                    if is_let {
                        (*object).borrow_mut().let_field(reference_name, value);
                    } else {
                        (*object)
                            .borrow_mut()
                            .assign_field(reference_name.as_str(), value);
                    }
                }
            }
        }
    }
    fn logic(lhs: Value, rhs: Value, is_and: bool) -> Value {
        match lhs {
            Value::Boolean(lhs) => {
                if let Value::Boolean(rhs) = rhs {
                    let result = if is_and { lhs & rhs } else { lhs | rhs };
                    Value::Boolean(result)
                } else {
                    {
                        ::std::rt::begin_panic(
                            "Use Logical Operator with Boolean or Integer",
                            &("pusl_lang/src/backend/mod.rs", 474u32, 17u32),
                        )
                    }
                }
            }
            Value::Integer(lhs) => {
                if let Value::Integer(rhs) = rhs {
                    let result = if is_and { lhs & rhs } else { lhs | rhs };
                    Value::Integer(result)
                } else {
                    {
                        ::std::rt::begin_panic(
                            "Use Logical Operator with Boolean or Integer",
                            &("pusl_lang/src/backend/mod.rs", 482u32, 17u32),
                        )
                    }
                }
            }
            _ => ::std::rt::begin_panic(
                "Use Logical Operator with Boolean or Integer",
                &("pusl_lang/src/backend/mod.rs", 485u32, 14u32),
            ),
        }
    }
    fn type_of(mut args: Vec<Value>, _: Option<ObjectPtr>) -> Value {
        if let Some(value) = args.pop() {
            if !args.is_empty() {
                {
                    {
                        ::std::rt::begin_panic(
                            "explicit panic",
                            &("pusl_lang/src/backend/mod.rs", 492u32, 13u32),
                        )
                    }
                }
            }
            let type_string = value.type_string();
            let gc_ptr = GC.with(|gc| gc.borrow_mut().place_in_heap(type_string.to_owned()));
            Value::String(gc_ptr)
        } else {
            {
                {
                    ::std::rt::begin_panic(
                        "explicit panic",
                        &("pusl_lang/src/backend/mod.rs", 498u32, 9u32),
                    )
                }
            }
        }
    }
    fn new_object(mut args: Vec<Value>, _: Option<ObjectPtr>) -> Value {
        if args.len() > 1 {
            {
                {
                    ::std::rt::begin_panic(
                        "explicit panic",
                        &("pusl_lang/src/backend/mod.rs", 504u32, 9u32),
                    )
                }
            }
        }
        let object = if let Some(super_obj) = args.pop() {
            let super_obj = if let Value::Object(ptr) = super_obj {
                ptr
            } else {
                {
                    {
                        ::std::rt::begin_panic(
                            "explicit panic",
                            &("pusl_lang/src/backend/mod.rs", 510u32, 13u32),
                        )
                    }
                }
            };
            Object::new_with_parent(super_obj)
        } else {
            Object::new()
        };
        let gc_ptr = GC.with(|gc| gc.borrow_mut().place_in_heap(object));
        Value::Object(gc_ptr)
    }
    fn print(args: Vec<Value>, _: Option<ObjectPtr>) -> Value {
        for value in args.into_iter().rev() {
            ::std::io::_print(::core::fmt::Arguments::new_v1(
                &[""],
                &match (&value,) {
                    (arg0,) => [::core::fmt::ArgumentV1::new(
                        arg0,
                        ::core::fmt::Display::fmt,
                    )],
                },
            ));
        }
        Value::Null
    }
    #[inline]
    fn modulus(lhs: Value, rhs: Value) -> Value {
        let lhs = if let Value::Integer(value) = lhs {
            value
        } else {
            {
                ::std::rt::begin_panic(
                    "Modulus only works with Integral operands",
                    &("pusl_lang/src/backend/mod.rs", 533u32, 9u32),
                )
            }
        };
        let rhs = if let Value::Integer(value) = rhs {
            value
        } else {
            {
                ::std::rt::begin_panic(
                    "Modulus only works with Integral operands",
                    &("pusl_lang/src/backend/mod.rs", 539u32, 9u32),
                )
            }
        };
        return Value::Integer(lhs % rhs);
    }
    #[inline]
    fn addition(lhs: Value, rhs: Value) -> Value {
        match lhs {
            Value::Integer(lhs) => match rhs {
                Value::Integer(rhs) => Value::Integer(lhs + rhs),
                Value::Float(rhs) => Value::Float(lhs as f64 + rhs),
                _ => ::std::rt::begin_panic(
                    "Invalid Operand for Addition",
                    &("pusl_lang/src/backend/mod.rs", 550u32, 18u32),
                ),
            },
            Value::Float(lhs) => match rhs {
                Value::Integer(rhs) => Value::Float(lhs + rhs as f64),
                Value::Float(rhs) => Value::Float(lhs + rhs),
                _ => ::std::rt::begin_panic(
                    "Invalid Operand for Addition",
                    &("pusl_lang/src/backend/mod.rs", 555u32, 18u32),
                ),
            },
            _ => ::std::rt::begin_panic(
                "Invalid Operand for Addition",
                &("pusl_lang/src/backend/mod.rs", 557u32, 14u32),
            ),
        }
    }
    #[inline]
    fn subtraction(lhs: Value, rhs: Value) -> Value {
        match lhs {
            Value::Integer(lhs) => match rhs {
                Value::Integer(rhs) => Value::Integer(lhs - rhs),
                Value::Float(rhs) => Value::Float(lhs as f64 - rhs),
                _ => ::std::rt::begin_panic(
                    "Invalid Operand for Subtraction",
                    &("pusl_lang/src/backend/mod.rs", 567u32, 18u32),
                ),
            },
            Value::Float(lhs) => match rhs {
                Value::Integer(rhs) => Value::Float(lhs - rhs as f64),
                Value::Float(rhs) => Value::Float(lhs - rhs),
                _ => ::std::rt::begin_panic(
                    "Invalid Operand for Subtraction",
                    &("pusl_lang/src/backend/mod.rs", 572u32, 18u32),
                ),
            },
            _ => ::std::rt::begin_panic(
                "Invalid Operand for Subtraction",
                &("pusl_lang/src/backend/mod.rs", 574u32, 14u32),
            ),
        }
    }
    #[inline]
    fn multiplication(lhs: Value, rhs: Value) -> Value {
        match lhs {
            Value::Integer(lhs) => match rhs {
                Value::Integer(rhs) => Value::Integer(lhs * rhs),
                Value::Float(rhs) => Value::Float(lhs as f64 * rhs),
                _ => ::std::rt::begin_panic(
                    "Invalid Operand for Multiplication",
                    &("pusl_lang/src/backend/mod.rs", 584u32, 18u32),
                ),
            },
            Value::Float(lhs) => match rhs {
                Value::Integer(rhs) => Value::Float(lhs * rhs as f64),
                Value::Float(rhs) => Value::Float(lhs * rhs),
                _ => ::std::rt::begin_panic(
                    "Invalid Operand for Multiplication",
                    &("pusl_lang/src/backend/mod.rs", 589u32, 18u32),
                ),
            },
            _ => ::std::rt::begin_panic(
                "Invalid Operand for Multiplication",
                &("pusl_lang/src/backend/mod.rs", 591u32, 14u32),
            ),
        }
    }
    #[inline]
    fn division(lhs: Value, rhs: Value) -> Value {
        match lhs {
            Value::Integer(lhs) => match rhs {
                Value::Integer(rhs) => Value::Float(lhs as f64 / rhs as f64),
                Value::Float(rhs) => Value::Float(lhs as f64 / rhs),
                _ => ::std::rt::begin_panic(
                    "Invalid Operand for Division",
                    &("pusl_lang/src/backend/mod.rs", 601u32, 18u32),
                ),
            },
            Value::Float(lhs) => match rhs {
                Value::Integer(rhs) => Value::Float(lhs / rhs as f64),
                Value::Float(rhs) => Value::Float(lhs / rhs),
                _ => ::std::rt::begin_panic(
                    "Invalid Operand for Division",
                    &("pusl_lang/src/backend/mod.rs", 606u32, 18u32),
                ),
            },
            _ => ::std::rt::begin_panic(
                "Invalid Operand for Division",
                &("pusl_lang/src/backend/mod.rs", 608u32, 14u32),
            ),
        }
    }
    #[inline]
    fn truncate_division(lhs: Value, rhs: Value) -> Value {
        match lhs {
            Value::Integer(lhs) => match rhs {
                Value::Integer(rhs) => Value::Integer(lhs / rhs),
                Value::Float(rhs) => Value::Integer((lhs as f64 / rhs) as i64),
                _ => ::std::rt::begin_panic(
                    "Invalid Operand for TruncDivision",
                    &("pusl_lang/src/backend/mod.rs", 618u32, 18u32),
                ),
            },
            Value::Float(lhs) => match rhs {
                Value::Integer(rhs) => Value::Integer((lhs / rhs as f64) as i64),
                Value::Float(rhs) => Value::Integer((lhs / rhs) as i64),
                _ => ::std::rt::begin_panic(
                    "Invalid Operand for TruncDivision",
                    &("pusl_lang/src/backend/mod.rs", 623u32, 18u32),
                ),
            },
            _ => ::std::rt::begin_panic(
                "Invalid Operand for TruncDivision",
                &("pusl_lang/src/backend/mod.rs", 625u32, 14u32),
            ),
        }
    }
    #[inline]
    fn exponent(lhs: Value, rhs: Value) -> Value {
        match lhs {
            Value::Integer(lhs) => match rhs {
                Value::Integer(rhs) => Value::Float((lhs as f64).powi(rhs as i32)),
                Value::Float(rhs) => Value::Float((lhs as f64).powf(rhs)),
                _ => ::std::rt::begin_panic(
                    "Invalid Operand for Exponent",
                    &("pusl_lang/src/backend/mod.rs", 635u32, 18u32),
                ),
            },
            Value::Float(lhs) => match rhs {
                Value::Integer(rhs) => Value::Float(lhs.powi(rhs as i32)),
                Value::Float(rhs) => Value::Float(lhs.powf(rhs)),
                _ => ::std::rt::begin_panic(
                    "Invalid Operand for Exponent",
                    &("pusl_lang/src/backend/mod.rs", 640u32, 18u32),
                ),
            },
            _ => ::std::rt::begin_panic(
                "Invalid Operand for Exponent",
                &("pusl_lang/src/backend/mod.rs", 642u32, 14u32),
            ),
        }
    }
    #[inline]
    fn negate(operand: Value) -> Value {
        match operand {
            Value::Boolean(val) => Value::Boolean(!val),
            Value::Integer(val) => Value::Integer(-val),
            Value::Float(val) => Value::Float(-val),
            _ => ::std::rt::begin_panic(
                "Invalid Operand for Negation",
                &("pusl_lang/src/backend/mod.rs", 652u32, 14u32),
            ),
        }
    }
    #[inline]
    fn compare(lhs: Value, rhs: Value, compare: Compare) -> Value {
        let equality = match compare {
            Compare::Equal => Some(false),
            Compare::NotEqual => Some(true),
            _ => None,
        };
        let result = if let Some(invert) = equality {
            let is_equal = match lhs {
                Value::Null => {
                    if let Value::Null = rhs {
                        true
                    } else {
                        false
                    }
                }
                Value::Boolean(lhs) => {
                    if let Value::Boolean(rhs) = rhs {
                        lhs == rhs
                    } else {
                        false
                    }
                }
                Value::Integer(lhs) => match rhs {
                    Value::Integer(rhs) => lhs == rhs,
                    Value::Float(rhs) => lhs as f64 == rhs,
                    _ => false,
                },
                Value::Float(lhs) => match rhs {
                    Value::Integer(rhs) => lhs == rhs as f64,
                    Value::Float(rhs) => lhs == rhs,
                    _ => false,
                },
                Value::String(lhs) => {
                    if let Value::String(rhs) = rhs {
                        *lhs == *rhs
                    } else {
                        false
                    }
                }
                Value::Function(lhs) => {
                    if let Value::Function(rhs) = rhs {
                        lhs as *const _ == rhs as *const _
                    } else {
                        false
                    }
                }
                Value::Object(lhs) => {
                    if let Value::Object(rhs) = rhs {
                        lhs == rhs
                    } else {
                        false
                    }
                }
                Value::Native(lhs) => {
                    if let Value::Native(rhs) = rhs {
                        lhs == rhs
                    } else {
                        false
                    }
                }
            };
            is_equal ^ invert
        } else {
            let cmp = compare_numerical(lhs, rhs);
            match compare {
                Compare::Less => match cmp {
                    Ordering::Less => true,
                    Ordering::Equal => false,
                    Ordering::Greater => false,
                },
                Compare::LessEqual => match cmp {
                    Ordering::Less => true,
                    Ordering::Equal => true,
                    Ordering::Greater => false,
                },
                Compare::Greater => match cmp {
                    Ordering::Less => false,
                    Ordering::Equal => false,
                    Ordering::Greater => true,
                },
                Compare::GreaterEqual => match cmp {
                    Ordering::Less => false,
                    Ordering::Equal => true,
                    Ordering::Greater => true,
                },
                _ => ::std::rt::begin_panic(
                    "Invariant",
                    &("pusl_lang/src/backend/mod.rs", 744u32, 18u32),
                ),
            }
        };
        Value::Boolean(result)
    }
    fn compare_numerical(lhs: Value, rhs: Value) -> Ordering {
        match lhs {
            Value::Integer(lhs) => match rhs {
                Value::Integer(rhs) => lhs.cmp(&rhs),
                Value::Float(rhs) => (lhs as f64).partial_cmp(&rhs).expect("Comparison Failed!"),
                _ => ::std::rt::begin_panic(
                    "Cannot Compare non-numeric types",
                    &("pusl_lang/src/backend/mod.rs", 756u32, 18u32),
                ),
            },
            Value::Float(lhs) => match rhs {
                Value::Integer(rhs) => lhs.partial_cmp(&(rhs as f64)).expect("Comparison Failed!"),
                Value::Float(rhs) => lhs.partial_cmp(&rhs).expect("Comparison Failed!"),
                _ => ::std::rt::begin_panic(
                    "Cannot Compare non-numeric types",
                    &("pusl_lang/src/backend/mod.rs", 761u32, 18u32),
                ),
            },
            _ => ::std::rt::begin_panic(
                "Cannot Compare non-numeric types",
                &("pusl_lang/src/backend/mod.rs", 763u32, 14u32),
            ),
        }
    }
}
pub mod lexer {
    //! The lexer takes the raw source code and changes each line into a list of tokens.
    //! Then, the lexer uses the indentation data and changes it into a hierarchy of tokens.
    //! This hierarchy is taken in by the parser which assembles it into logical units.
    //! This module finds syntactical errors.
    use crate::lexer::peek_while::peek_while;
    use crate::lexer::token::Symbol::*;
    use crate::lexer::token::{Block, BlockType, Keyword, LexUnit, Literal, Symbol, Token};
    use std::iter::Peekable;
    use std::str::Chars;
    pub mod peek_while {
        pub struct PeekWhile<'a, I, F>
        where
            I: Iterator + 'a,
        {
            iter: &'a mut std::iter::Peekable<I>,
            f: F,
        }
        impl<'a, I, F> Iterator for PeekWhile<'a, I, F>
        where
            I: Iterator + 'a,
            F: for<'b> FnMut(&'b <I as Iterator>::Item) -> bool,
        {
            type Item = <I as Iterator>::Item;
            fn next(&mut self) -> Option<<Self as Iterator>::Item> {
                let &mut PeekWhile {
                    ref mut iter,
                    ref mut f,
                } = self;
                if iter.peek().map(f).unwrap_or(false) {
                    iter.next()
                } else {
                    None
                }
            }
        }
        pub fn peek_while<'a, I, F>(
            iter: &'a mut std::iter::Peekable<I>,
            f: F,
        ) -> PeekWhile<'a, I, F>
        where
            I: Iterator + 'a,
            F: for<'b> FnMut(&'b <I as Iterator>::Item) -> bool,
        {
            PeekWhile { iter, f }
        }
    }
    pub mod token {
        use crate::backend::object::Value;
        use crate::backend::GcPoolRef;
        use serde::{Deserialize, Serialize};
        use std::fmt;
        use std::fmt::{Debug, Formatter};
        pub enum Literal {
            Boolean(bool),
            Integer(i64),
            Float(f64),
            String(String),
            Null,
        }
        #[automatically_derived]
        #[allow(unused_qualifications)]
        impl ::core::fmt::Debug for Literal {
            fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                match (&*self,) {
                    (&Literal::Boolean(ref __self_0),) => {
                        let mut debug_trait_builder = f.debug_tuple("Boolean");
                        let _ = debug_trait_builder.field(&&(*__self_0));
                        debug_trait_builder.finish()
                    }
                    (&Literal::Integer(ref __self_0),) => {
                        let mut debug_trait_builder = f.debug_tuple("Integer");
                        let _ = debug_trait_builder.field(&&(*__self_0));
                        debug_trait_builder.finish()
                    }
                    (&Literal::Float(ref __self_0),) => {
                        let mut debug_trait_builder = f.debug_tuple("Float");
                        let _ = debug_trait_builder.field(&&(*__self_0));
                        debug_trait_builder.finish()
                    }
                    (&Literal::String(ref __self_0),) => {
                        let mut debug_trait_builder = f.debug_tuple("String");
                        let _ = debug_trait_builder.field(&&(*__self_0));
                        debug_trait_builder.finish()
                    }
                    (&Literal::Null,) => {
                        let mut debug_trait_builder = f.debug_tuple("Null");
                        debug_trait_builder.finish()
                    }
                }
            }
        }
        impl ::core::marker::StructuralPartialEq for Literal {}
        #[automatically_derived]
        #[allow(unused_qualifications)]
        impl ::core::cmp::PartialEq for Literal {
            #[inline]
            fn eq(&self, other: &Literal) -> bool {
                {
                    let __self_vi =
                        unsafe { ::core::intrinsics::discriminant_value(&*self) } as isize;
                    let __arg_1_vi =
                        unsafe { ::core::intrinsics::discriminant_value(&*other) } as isize;
                    if true && __self_vi == __arg_1_vi {
                        match (&*self, &*other) {
                            (&Literal::Boolean(ref __self_0), &Literal::Boolean(ref __arg_1_0)) => {
                                (*__self_0) == (*__arg_1_0)
                            }
                            (&Literal::Integer(ref __self_0), &Literal::Integer(ref __arg_1_0)) => {
                                (*__self_0) == (*__arg_1_0)
                            }
                            (&Literal::Float(ref __self_0), &Literal::Float(ref __arg_1_0)) => {
                                (*__self_0) == (*__arg_1_0)
                            }
                            (&Literal::String(ref __self_0), &Literal::String(ref __arg_1_0)) => {
                                (*__self_0) == (*__arg_1_0)
                            }
                            _ => true,
                        }
                    } else {
                        false
                    }
                }
            }
            #[inline]
            fn ne(&self, other: &Literal) -> bool {
                {
                    let __self_vi =
                        unsafe { ::core::intrinsics::discriminant_value(&*self) } as isize;
                    let __arg_1_vi =
                        unsafe { ::core::intrinsics::discriminant_value(&*other) } as isize;
                    if true && __self_vi == __arg_1_vi {
                        match (&*self, &*other) {
                            (&Literal::Boolean(ref __self_0), &Literal::Boolean(ref __arg_1_0)) => {
                                (*__self_0) != (*__arg_1_0)
                            }
                            (&Literal::Integer(ref __self_0), &Literal::Integer(ref __arg_1_0)) => {
                                (*__self_0) != (*__arg_1_0)
                            }
                            (&Literal::Float(ref __self_0), &Literal::Float(ref __arg_1_0)) => {
                                (*__self_0) != (*__arg_1_0)
                            }
                            (&Literal::String(ref __self_0), &Literal::String(ref __arg_1_0)) => {
                                (*__self_0) != (*__arg_1_0)
                            }
                            _ => false,
                        }
                    } else {
                        true
                    }
                }
            }
        }
        #[automatically_derived]
        #[allow(unused_qualifications)]
        impl ::core::clone::Clone for Literal {
            #[inline]
            fn clone(&self) -> Literal {
                match (&*self,) {
                    (&Literal::Boolean(ref __self_0),) => {
                        Literal::Boolean(::core::clone::Clone::clone(&(*__self_0)))
                    }
                    (&Literal::Integer(ref __self_0),) => {
                        Literal::Integer(::core::clone::Clone::clone(&(*__self_0)))
                    }
                    (&Literal::Float(ref __self_0),) => {
                        Literal::Float(::core::clone::Clone::clone(&(*__self_0)))
                    }
                    (&Literal::String(ref __self_0),) => {
                        Literal::String(::core::clone::Clone::clone(&(*__self_0)))
                    }
                    (&Literal::Null,) => Literal::Null,
                }
            }
        }
        #[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
        const _IMPL_SERIALIZE_FOR_Literal: () = {
            #[allow(unknown_lints)]
            #[allow(rust_2018_idioms)]
            extern crate serde as _serde;
            #[automatically_derived]
            impl _serde::Serialize for Literal {
                fn serialize<__S>(
                    &self,
                    __serializer: __S,
                ) -> _serde::export::Result<__S::Ok, __S::Error>
                where
                    __S: _serde::Serializer,
                {
                    match *self {
                        Literal::Boolean(ref __field0) => {
                            _serde::Serializer::serialize_newtype_variant(
                                __serializer,
                                "Literal",
                                0u32,
                                "Boolean",
                                __field0,
                            )
                        }
                        Literal::Integer(ref __field0) => {
                            _serde::Serializer::serialize_newtype_variant(
                                __serializer,
                                "Literal",
                                1u32,
                                "Integer",
                                __field0,
                            )
                        }
                        Literal::Float(ref __field0) => {
                            _serde::Serializer::serialize_newtype_variant(
                                __serializer,
                                "Literal",
                                2u32,
                                "Float",
                                __field0,
                            )
                        }
                        Literal::String(ref __field0) => {
                            _serde::Serializer::serialize_newtype_variant(
                                __serializer,
                                "Literal",
                                3u32,
                                "String",
                                __field0,
                            )
                        }
                        Literal::Null => _serde::Serializer::serialize_unit_variant(
                            __serializer,
                            "Literal",
                            4u32,
                            "Null",
                        ),
                    }
                }
            }
        };
        #[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
        const _IMPL_DESERIALIZE_FOR_Literal: () = {
            #[allow(unknown_lints)]
            #[allow(rust_2018_idioms)]
            extern crate serde as _serde;
            #[automatically_derived]
            impl<'de> _serde::Deserialize<'de> for Literal {
                fn deserialize<__D>(__deserializer: __D) -> _serde::export::Result<Self, __D::Error>
                where
                    __D: _serde::Deserializer<'de>,
                {
                    #[allow(non_camel_case_types)]
                    enum __Field {
                        __field0,
                        __field1,
                        __field2,
                        __field3,
                        __field4,
                    }
                    struct __FieldVisitor;
                    impl<'de> _serde::de::Visitor<'de> for __FieldVisitor {
                        type Value = __Field;
                        fn expecting(
                            &self,
                            __formatter: &mut _serde::export::Formatter,
                        ) -> _serde::export::fmt::Result {
                            _serde::export::Formatter::write_str(__formatter, "variant identifier")
                        }
                        fn visit_u64<__E>(
                            self,
                            __value: u64,
                        ) -> _serde::export::Result<Self::Value, __E>
                        where
                            __E: _serde::de::Error,
                        {
                            match __value {
                                0u64 => _serde::export::Ok(__Field::__field0),
                                1u64 => _serde::export::Ok(__Field::__field1),
                                2u64 => _serde::export::Ok(__Field::__field2),
                                3u64 => _serde::export::Ok(__Field::__field3),
                                4u64 => _serde::export::Ok(__Field::__field4),
                                _ => _serde::export::Err(_serde::de::Error::invalid_value(
                                    _serde::de::Unexpected::Unsigned(__value),
                                    &"variant index 0 <= i < 5",
                                )),
                            }
                        }
                        fn visit_str<__E>(
                            self,
                            __value: &str,
                        ) -> _serde::export::Result<Self::Value, __E>
                        where
                            __E: _serde::de::Error,
                        {
                            match __value {
                                "Boolean" => _serde::export::Ok(__Field::__field0),
                                "Integer" => _serde::export::Ok(__Field::__field1),
                                "Float" => _serde::export::Ok(__Field::__field2),
                                "String" => _serde::export::Ok(__Field::__field3),
                                "Null" => _serde::export::Ok(__Field::__field4),
                                _ => _serde::export::Err(_serde::de::Error::unknown_variant(
                                    __value, VARIANTS,
                                )),
                            }
                        }
                        fn visit_bytes<__E>(
                            self,
                            __value: &[u8],
                        ) -> _serde::export::Result<Self::Value, __E>
                        where
                            __E: _serde::de::Error,
                        {
                            match __value {
                                b"Boolean" => _serde::export::Ok(__Field::__field0),
                                b"Integer" => _serde::export::Ok(__Field::__field1),
                                b"Float" => _serde::export::Ok(__Field::__field2),
                                b"String" => _serde::export::Ok(__Field::__field3),
                                b"Null" => _serde::export::Ok(__Field::__field4),
                                _ => {
                                    let __value = &_serde::export::from_utf8_lossy(__value);
                                    _serde::export::Err(_serde::de::Error::unknown_variant(
                                        __value, VARIANTS,
                                    ))
                                }
                            }
                        }
                    }
                    impl<'de> _serde::Deserialize<'de> for __Field {
                        #[inline]
                        fn deserialize<__D>(
                            __deserializer: __D,
                        ) -> _serde::export::Result<Self, __D::Error>
                        where
                            __D: _serde::Deserializer<'de>,
                        {
                            _serde::Deserializer::deserialize_identifier(
                                __deserializer,
                                __FieldVisitor,
                            )
                        }
                    }
                    struct __Visitor<'de> {
                        marker: _serde::export::PhantomData<Literal>,
                        lifetime: _serde::export::PhantomData<&'de ()>,
                    }
                    impl<'de> _serde::de::Visitor<'de> for __Visitor<'de> {
                        type Value = Literal;
                        fn expecting(
                            &self,
                            __formatter: &mut _serde::export::Formatter,
                        ) -> _serde::export::fmt::Result {
                            _serde::export::Formatter::write_str(__formatter, "enum Literal")
                        }
                        fn visit_enum<__A>(
                            self,
                            __data: __A,
                        ) -> _serde::export::Result<Self::Value, __A::Error>
                        where
                            __A: _serde::de::EnumAccess<'de>,
                        {
                            match match _serde::de::EnumAccess::variant(__data) {
                                _serde::export::Ok(__val) => __val,
                                _serde::export::Err(__err) => {
                                    return _serde::export::Err(__err);
                                }
                            } {
                                (__Field::__field0, __variant) => _serde::export::Result::map(
                                    _serde::de::VariantAccess::newtype_variant::<bool>(__variant),
                                    Literal::Boolean,
                                ),
                                (__Field::__field1, __variant) => _serde::export::Result::map(
                                    _serde::de::VariantAccess::newtype_variant::<i64>(__variant),
                                    Literal::Integer,
                                ),
                                (__Field::__field2, __variant) => _serde::export::Result::map(
                                    _serde::de::VariantAccess::newtype_variant::<f64>(__variant),
                                    Literal::Float,
                                ),
                                (__Field::__field3, __variant) => _serde::export::Result::map(
                                    _serde::de::VariantAccess::newtype_variant::<String>(__variant),
                                    Literal::String,
                                ),
                                (__Field::__field4, __variant) => {
                                    match _serde::de::VariantAccess::unit_variant(__variant) {
                                        _serde::export::Ok(__val) => __val,
                                        _serde::export::Err(__err) => {
                                            return _serde::export::Err(__err);
                                        }
                                    };
                                    _serde::export::Ok(Literal::Null)
                                }
                            }
                        }
                    }
                    const VARIANTS: &'static [&'static str] =
                        &["Boolean", "Integer", "Float", "String", "Null"];
                    _serde::Deserializer::deserialize_enum(
                        __deserializer,
                        "Literal",
                        VARIANTS,
                        __Visitor {
                            marker: _serde::export::PhantomData::<Literal>,
                            lifetime: _serde::export::PhantomData,
                        },
                    )
                }
            }
        };
        impl Literal {
            pub fn into_value(self, gc: GcPoolRef) -> Value {
                match self {
                    Literal::Boolean(value) => Value::Boolean(value),
                    Literal::Integer(value) => Value::Integer(value),
                    Literal::Float(value) => Value::Float(value),
                    Literal::String(value) => {
                        let gc_ptr = gc.with(|gc| gc.borrow_mut().place_in_heap(value));
                        Value::String(gc_ptr)
                    }
                    Literal::Null => Value::Null,
                }
            }
        }
        pub enum Symbol {
            OpenParenthesis,
            CloseParenthesis,
            Percent,
            Comma,
            ExclamationPoint,
            Period,
            Colon,
            SemiColon,
            Elvis,
            ConditionalAssignment,
            Plus,
            Minus,
            Star,
            DoubleStar,
            Slash,
            DoubleSlash,
            Equals,
            DoubleEquals,
            NotEquals,
            Greater,
            Less,
            GreaterEquals,
            LessEquals,
            Or,
            And,
        }
        #[automatically_derived]
        #[allow(unused_qualifications)]
        impl ::core::fmt::Debug for Symbol {
            fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                match (&*self,) {
                    (&Symbol::OpenParenthesis,) => {
                        let mut debug_trait_builder = f.debug_tuple("OpenParenthesis");
                        debug_trait_builder.finish()
                    }
                    (&Symbol::CloseParenthesis,) => {
                        let mut debug_trait_builder = f.debug_tuple("CloseParenthesis");
                        debug_trait_builder.finish()
                    }
                    (&Symbol::Percent,) => {
                        let mut debug_trait_builder = f.debug_tuple("Percent");
                        debug_trait_builder.finish()
                    }
                    (&Symbol::Comma,) => {
                        let mut debug_trait_builder = f.debug_tuple("Comma");
                        debug_trait_builder.finish()
                    }
                    (&Symbol::ExclamationPoint,) => {
                        let mut debug_trait_builder = f.debug_tuple("ExclamationPoint");
                        debug_trait_builder.finish()
                    }
                    (&Symbol::Period,) => {
                        let mut debug_trait_builder = f.debug_tuple("Period");
                        debug_trait_builder.finish()
                    }
                    (&Symbol::Colon,) => {
                        let mut debug_trait_builder = f.debug_tuple("Colon");
                        debug_trait_builder.finish()
                    }
                    (&Symbol::SemiColon,) => {
                        let mut debug_trait_builder = f.debug_tuple("SemiColon");
                        debug_trait_builder.finish()
                    }
                    (&Symbol::Elvis,) => {
                        let mut debug_trait_builder = f.debug_tuple("Elvis");
                        debug_trait_builder.finish()
                    }
                    (&Symbol::ConditionalAssignment,) => {
                        let mut debug_trait_builder = f.debug_tuple("ConditionalAssignment");
                        debug_trait_builder.finish()
                    }
                    (&Symbol::Plus,) => {
                        let mut debug_trait_builder = f.debug_tuple("Plus");
                        debug_trait_builder.finish()
                    }
                    (&Symbol::Minus,) => {
                        let mut debug_trait_builder = f.debug_tuple("Minus");
                        debug_trait_builder.finish()
                    }
                    (&Symbol::Star,) => {
                        let mut debug_trait_builder = f.debug_tuple("Star");
                        debug_trait_builder.finish()
                    }
                    (&Symbol::DoubleStar,) => {
                        let mut debug_trait_builder = f.debug_tuple("DoubleStar");
                        debug_trait_builder.finish()
                    }
                    (&Symbol::Slash,) => {
                        let mut debug_trait_builder = f.debug_tuple("Slash");
                        debug_trait_builder.finish()
                    }
                    (&Symbol::DoubleSlash,) => {
                        let mut debug_trait_builder = f.debug_tuple("DoubleSlash");
                        debug_trait_builder.finish()
                    }
                    (&Symbol::Equals,) => {
                        let mut debug_trait_builder = f.debug_tuple("Equals");
                        debug_trait_builder.finish()
                    }
                    (&Symbol::DoubleEquals,) => {
                        let mut debug_trait_builder = f.debug_tuple("DoubleEquals");
                        debug_trait_builder.finish()
                    }
                    (&Symbol::NotEquals,) => {
                        let mut debug_trait_builder = f.debug_tuple("NotEquals");
                        debug_trait_builder.finish()
                    }
                    (&Symbol::Greater,) => {
                        let mut debug_trait_builder = f.debug_tuple("Greater");
                        debug_trait_builder.finish()
                    }
                    (&Symbol::Less,) => {
                        let mut debug_trait_builder = f.debug_tuple("Less");
                        debug_trait_builder.finish()
                    }
                    (&Symbol::GreaterEquals,) => {
                        let mut debug_trait_builder = f.debug_tuple("GreaterEquals");
                        debug_trait_builder.finish()
                    }
                    (&Symbol::LessEquals,) => {
                        let mut debug_trait_builder = f.debug_tuple("LessEquals");
                        debug_trait_builder.finish()
                    }
                    (&Symbol::Or,) => {
                        let mut debug_trait_builder = f.debug_tuple("Or");
                        debug_trait_builder.finish()
                    }
                    (&Symbol::And,) => {
                        let mut debug_trait_builder = f.debug_tuple("And");
                        debug_trait_builder.finish()
                    }
                }
            }
        }
        impl ::core::marker::StructuralEq for Symbol {}
        #[automatically_derived]
        #[allow(unused_qualifications)]
        impl ::core::cmp::Eq for Symbol {
            #[inline]
            #[doc(hidden)]
            fn assert_receiver_is_total_eq(&self) -> () {
                {}
            }
        }
        impl ::core::marker::StructuralPartialEq for Symbol {}
        #[automatically_derived]
        #[allow(unused_qualifications)]
        impl ::core::cmp::PartialEq for Symbol {
            #[inline]
            fn eq(&self, other: &Symbol) -> bool {
                {
                    let __self_vi =
                        unsafe { ::core::intrinsics::discriminant_value(&*self) } as isize;
                    let __arg_1_vi =
                        unsafe { ::core::intrinsics::discriminant_value(&*other) } as isize;
                    if true && __self_vi == __arg_1_vi {
                        match (&*self, &*other) {
                            _ => true,
                        }
                    } else {
                        false
                    }
                }
            }
        }
        #[automatically_derived]
        #[allow(unused_qualifications)]
        impl ::core::marker::Copy for Symbol {}
        #[automatically_derived]
        #[allow(unused_qualifications)]
        impl ::core::clone::Clone for Symbol {
            #[inline]
            fn clone(&self) -> Symbol {
                {
                    *self
                }
            }
        }
        pub enum BlockType {
            If,
            Else,
            ElseIf,
            While,
            For,
            Cmp,
            Function,
        }
        #[automatically_derived]
        #[allow(unused_qualifications)]
        impl ::core::fmt::Debug for BlockType {
            fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                match (&*self,) {
                    (&BlockType::If,) => {
                        let mut debug_trait_builder = f.debug_tuple("If");
                        debug_trait_builder.finish()
                    }
                    (&BlockType::Else,) => {
                        let mut debug_trait_builder = f.debug_tuple("Else");
                        debug_trait_builder.finish()
                    }
                    (&BlockType::ElseIf,) => {
                        let mut debug_trait_builder = f.debug_tuple("ElseIf");
                        debug_trait_builder.finish()
                    }
                    (&BlockType::While,) => {
                        let mut debug_trait_builder = f.debug_tuple("While");
                        debug_trait_builder.finish()
                    }
                    (&BlockType::For,) => {
                        let mut debug_trait_builder = f.debug_tuple("For");
                        debug_trait_builder.finish()
                    }
                    (&BlockType::Cmp,) => {
                        let mut debug_trait_builder = f.debug_tuple("Cmp");
                        debug_trait_builder.finish()
                    }
                    (&BlockType::Function,) => {
                        let mut debug_trait_builder = f.debug_tuple("Function");
                        debug_trait_builder.finish()
                    }
                }
            }
        }
        #[automatically_derived]
        #[allow(unused_qualifications)]
        impl ::core::marker::Copy for BlockType {}
        #[automatically_derived]
        #[allow(unused_qualifications)]
        impl ::core::clone::Clone for BlockType {
            #[inline]
            fn clone(&self) -> BlockType {
                {
                    *self
                }
            }
        }
        impl ::core::marker::StructuralEq for BlockType {}
        #[automatically_derived]
        #[allow(unused_qualifications)]
        impl ::core::cmp::Eq for BlockType {
            #[inline]
            #[doc(hidden)]
            fn assert_receiver_is_total_eq(&self) -> () {
                {}
            }
        }
        impl ::core::marker::StructuralPartialEq for BlockType {}
        #[automatically_derived]
        #[allow(unused_qualifications)]
        impl ::core::cmp::PartialEq for BlockType {
            #[inline]
            fn eq(&self, other: &BlockType) -> bool {
                {
                    let __self_vi =
                        unsafe { ::core::intrinsics::discriminant_value(&*self) } as isize;
                    let __arg_1_vi =
                        unsafe { ::core::intrinsics::discriminant_value(&*other) } as isize;
                    if true && __self_vi == __arg_1_vi {
                        match (&*self, &*other) {
                            _ => true,
                        }
                    } else {
                        false
                    }
                }
            }
        }
        pub enum LexUnit {
            Statement(Vec<Token>),
            Block(Block),
        }
        impl Debug for LexUnit {
            fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
                self.fmt_indent(f, 0)
            }
        }
        pub struct Block {
            pub kind: BlockType,
            pub line: Vec<Token>,
            pub children: Vec<LexUnit>,
        }
        impl LexUnit {
            pub fn get_tokens(&self) -> &Vec<Token> {
                match self {
                    LexUnit::Statement(tokens) => tokens,
                    LexUnit::Block(block) => block.get_tokens(),
                }
            }
            pub fn fmt_indent(&self, f: &mut Formatter<'_>, indent: usize) -> fmt::Result {
                match self {
                    LexUnit::Statement(tokens) => {
                        for _ in 0..indent {
                            f.write_fmt(::core::fmt::Arguments::new_v1(
                                &["\t"],
                                &match () {
                                    () => [],
                                },
                            ))?;
                        }
                        f.write_fmt(::core::fmt::Arguments::new_v1(
                            &["", "\n"],
                            &match (&tokens,) {
                                (arg0,) => {
                                    [::core::fmt::ArgumentV1::new(arg0, ::core::fmt::Debug::fmt)]
                                }
                            },
                        ))
                    }
                    LexUnit::Block(block) => block.fmt_indent(f, indent),
                }
            }
        }
        impl Block {
            pub fn fmt_indent(&self, f: &mut Formatter<'_>, indent: usize) -> fmt::Result {
                for _ in 0..indent {
                    f.write_fmt(::core::fmt::Arguments::new_v1(
                        &["\t"],
                        &match () {
                            () => [],
                        },
                    ))?;
                }
                f.write_fmt(::core::fmt::Arguments::new_v1(
                    &["", "\n"],
                    &match (&self.line,) {
                        (arg0,) => [::core::fmt::ArgumentV1::new(arg0, ::core::fmt::Debug::fmt)],
                    },
                ))?;
                for child in &self.children {
                    child.fmt_indent(f, indent + 1)?
                }
                Ok(())
            }
            pub fn get_tokens(&self) -> &Vec<Token> {
                &self.line
            }
            pub fn get_children(&self) -> &Vec<LexUnit> {
                &self.children
            }
        }
        impl Debug for Block {
            fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
                self.fmt_indent(f, 0)
            }
        }
        pub enum Keyword {
            Let,
            In,
            To,
            This,
            Return,
            Fn,
            Import,
            As,
        }
        #[automatically_derived]
        #[allow(unused_qualifications)]
        impl ::core::fmt::Debug for Keyword {
            fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                match (&*self,) {
                    (&Keyword::Let,) => {
                        let mut debug_trait_builder = f.debug_tuple("Let");
                        debug_trait_builder.finish()
                    }
                    (&Keyword::In,) => {
                        let mut debug_trait_builder = f.debug_tuple("In");
                        debug_trait_builder.finish()
                    }
                    (&Keyword::To,) => {
                        let mut debug_trait_builder = f.debug_tuple("To");
                        debug_trait_builder.finish()
                    }
                    (&Keyword::This,) => {
                        let mut debug_trait_builder = f.debug_tuple("This");
                        debug_trait_builder.finish()
                    }
                    (&Keyword::Return,) => {
                        let mut debug_trait_builder = f.debug_tuple("Return");
                        debug_trait_builder.finish()
                    }
                    (&Keyword::Fn,) => {
                        let mut debug_trait_builder = f.debug_tuple("Fn");
                        debug_trait_builder.finish()
                    }
                    (&Keyword::Import,) => {
                        let mut debug_trait_builder = f.debug_tuple("Import");
                        debug_trait_builder.finish()
                    }
                    (&Keyword::As,) => {
                        let mut debug_trait_builder = f.debug_tuple("As");
                        debug_trait_builder.finish()
                    }
                }
            }
        }
        #[automatically_derived]
        #[allow(unused_qualifications)]
        impl ::core::marker::Copy for Keyword {}
        #[automatically_derived]
        #[allow(unused_qualifications)]
        impl ::core::clone::Clone for Keyword {
            #[inline]
            fn clone(&self) -> Keyword {
                {
                    *self
                }
            }
        }
        impl ::core::marker::StructuralPartialEq for Keyword {}
        #[automatically_derived]
        #[allow(unused_qualifications)]
        impl ::core::cmp::PartialEq for Keyword {
            #[inline]
            fn eq(&self, other: &Keyword) -> bool {
                {
                    let __self_vi =
                        unsafe { ::core::intrinsics::discriminant_value(&*self) } as isize;
                    let __arg_1_vi =
                        unsafe { ::core::intrinsics::discriminant_value(&*other) } as isize;
                    if true && __self_vi == __arg_1_vi {
                        match (&*self, &*other) {
                            _ => true,
                        }
                    } else {
                        false
                    }
                }
            }
        }
        pub enum Token {
            Literal(Literal),
            Block(BlockType),
            Reference(String),
            Symbol(Symbol),
            Keyword(Keyword),
        }
        #[automatically_derived]
        #[allow(unused_qualifications)]
        impl ::core::fmt::Debug for Token {
            fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                match (&*self,) {
                    (&Token::Literal(ref __self_0),) => {
                        let mut debug_trait_builder = f.debug_tuple("Literal");
                        let _ = debug_trait_builder.field(&&(*__self_0));
                        debug_trait_builder.finish()
                    }
                    (&Token::Block(ref __self_0),) => {
                        let mut debug_trait_builder = f.debug_tuple("Block");
                        let _ = debug_trait_builder.field(&&(*__self_0));
                        debug_trait_builder.finish()
                    }
                    (&Token::Reference(ref __self_0),) => {
                        let mut debug_trait_builder = f.debug_tuple("Reference");
                        let _ = debug_trait_builder.field(&&(*__self_0));
                        debug_trait_builder.finish()
                    }
                    (&Token::Symbol(ref __self_0),) => {
                        let mut debug_trait_builder = f.debug_tuple("Symbol");
                        let _ = debug_trait_builder.field(&&(*__self_0));
                        debug_trait_builder.finish()
                    }
                    (&Token::Keyword(ref __self_0),) => {
                        let mut debug_trait_builder = f.debug_tuple("Keyword");
                        let _ = debug_trait_builder.field(&&(*__self_0));
                        debug_trait_builder.finish()
                    }
                }
            }
        }
        impl ::core::marker::StructuralPartialEq for Token {}
        #[automatically_derived]
        #[allow(unused_qualifications)]
        impl ::core::cmp::PartialEq for Token {
            #[inline]
            fn eq(&self, other: &Token) -> bool {
                {
                    let __self_vi =
                        unsafe { ::core::intrinsics::discriminant_value(&*self) } as isize;
                    let __arg_1_vi =
                        unsafe { ::core::intrinsics::discriminant_value(&*other) } as isize;
                    if true && __self_vi == __arg_1_vi {
                        match (&*self, &*other) {
                            (&Token::Literal(ref __self_0), &Token::Literal(ref __arg_1_0)) => {
                                (*__self_0) == (*__arg_1_0)
                            }
                            (&Token::Block(ref __self_0), &Token::Block(ref __arg_1_0)) => {
                                (*__self_0) == (*__arg_1_0)
                            }
                            (&Token::Reference(ref __self_0), &Token::Reference(ref __arg_1_0)) => {
                                (*__self_0) == (*__arg_1_0)
                            }
                            (&Token::Symbol(ref __self_0), &Token::Symbol(ref __arg_1_0)) => {
                                (*__self_0) == (*__arg_1_0)
                            }
                            (&Token::Keyword(ref __self_0), &Token::Keyword(ref __arg_1_0)) => {
                                (*__self_0) == (*__arg_1_0)
                            }
                            _ => unsafe { ::core::intrinsics::unreachable() },
                        }
                    } else {
                        false
                    }
                }
            }
            #[inline]
            fn ne(&self, other: &Token) -> bool {
                {
                    let __self_vi =
                        unsafe { ::core::intrinsics::discriminant_value(&*self) } as isize;
                    let __arg_1_vi =
                        unsafe { ::core::intrinsics::discriminant_value(&*other) } as isize;
                    if true && __self_vi == __arg_1_vi {
                        match (&*self, &*other) {
                            (&Token::Literal(ref __self_0), &Token::Literal(ref __arg_1_0)) => {
                                (*__self_0) != (*__arg_1_0)
                            }
                            (&Token::Block(ref __self_0), &Token::Block(ref __arg_1_0)) => {
                                (*__self_0) != (*__arg_1_0)
                            }
                            (&Token::Reference(ref __self_0), &Token::Reference(ref __arg_1_0)) => {
                                (*__self_0) != (*__arg_1_0)
                            }
                            (&Token::Symbol(ref __self_0), &Token::Symbol(ref __arg_1_0)) => {
                                (*__self_0) != (*__arg_1_0)
                            }
                            (&Token::Keyword(ref __self_0), &Token::Keyword(ref __arg_1_0)) => {
                                (*__self_0) != (*__arg_1_0)
                            }
                            _ => unsafe { ::core::intrinsics::unreachable() },
                        }
                    } else {
                        true
                    }
                }
            }
        }
        #[automatically_derived]
        #[allow(unused_qualifications)]
        impl ::core::clone::Clone for Token {
            #[inline]
            fn clone(&self) -> Token {
                match (&*self,) {
                    (&Token::Literal(ref __self_0),) => {
                        Token::Literal(::core::clone::Clone::clone(&(*__self_0)))
                    }
                    (&Token::Block(ref __self_0),) => {
                        Token::Block(::core::clone::Clone::clone(&(*__self_0)))
                    }
                    (&Token::Reference(ref __self_0),) => {
                        Token::Reference(::core::clone::Clone::clone(&(*__self_0)))
                    }
                    (&Token::Symbol(ref __self_0),) => {
                        Token::Symbol(::core::clone::Clone::clone(&(*__self_0)))
                    }
                    (&Token::Keyword(ref __self_0),) => {
                        Token::Keyword(::core::clone::Clone::clone(&(*__self_0)))
                    }
                }
            }
        }
    }
    type Source<'a> = Peekable<Chars<'a>>;
    pub fn lex<'a, I>(lines: I) -> Vec<LexUnit>
    where
        I: IntoIterator<Item = &'a str>,
    {
        let iter = lines.into_iter();
        let mut iter = iter
            .map(lex_line)
            .filter(|(line, _)| !line.is_empty())
            .peekable();
        let mut roots = Vec::new();
        while let Some(root) = lex_internal(&mut iter, 0) {
            roots.push(root);
        }
        roots
    }
    fn lex_internal<I>(stream: &mut Peekable<I>, indent: usize) -> Option<LexUnit>
    where
        I: Iterator<Item = (Vec<Token>, usize)>,
    {
        if let Some((tokens, indentation)) = stream.next() {
            {
                match (&indent, &indentation) {
                    (left_val, right_val) => {
                        if !(*left_val == *right_val) {
                            {
                                ::std::rt::begin_panic_fmt(
                                    &::core::fmt::Arguments::new_v1(
                                        &[
                                            "assertion failed: `(left == right)`\n  left: `",
                                            "`,\n right: `",
                                            "`",
                                        ],
                                        &match (&&*left_val, &&*right_val) {
                                            (arg0, arg1) => [
                                                ::core::fmt::ArgumentV1::new(
                                                    arg0,
                                                    ::core::fmt::Debug::fmt,
                                                ),
                                                ::core::fmt::ArgumentV1::new(
                                                    arg1,
                                                    ::core::fmt::Debug::fmt,
                                                ),
                                            ],
                                        },
                                    ),
                                    &("pusl_lang/src/lexer/mod.rs", 41u32, 9u32),
                                )
                            }
                        }
                    }
                }
            };
            let mut children = Vec::new();
            while stream.peek().map_or(false, |e| e.1 == indent + 1) {
                if let Some(child) = lex_internal(stream, indent + 1) {
                    children.push(child)
                } else {
                    {
                        {
                            ::std::rt::begin_panic(
                                "explicit panic",
                                &("pusl_lang/src/lexer/mod.rs", 47u32, 17u32),
                            )
                        }
                    };
                }
            }
            if !children.is_empty() {
                {
                    match (&tokens.last(), &Some(&Token::Symbol(Symbol::Colon))) {
                        (left_val, right_val) => {
                            if !(*left_val == *right_val) {
                                {
                                    ::std::rt::begin_panic_fmt(
                                        &::core::fmt::Arguments::new_v1(
                                            &[
                                                "assertion failed: `(left == right)`\n  left: `",
                                                "`,\n right: `",
                                                "`",
                                            ],
                                            &match (&&*left_val, &&*right_val) {
                                                (arg0, arg1) => [
                                                    ::core::fmt::ArgumentV1::new(
                                                        arg0,
                                                        ::core::fmt::Debug::fmt,
                                                    ),
                                                    ::core::fmt::ArgumentV1::new(
                                                        arg1,
                                                        ::core::fmt::Debug::fmt,
                                                    ),
                                                ],
                                            },
                                        ),
                                        &("pusl_lang/src/lexer/mod.rs", 52u32, 13u32),
                                    )
                                }
                            }
                        }
                    }
                };
                let first = tokens.first();
                let second = tokens.get(1);
                if let Some(&Token::Block(block_type)) = first {
                    let mut return_type = block_type;
                    if let BlockType::Else = block_type {
                        if let Some(&Token::Block(BlockType::If)) = second {
                            return_type = BlockType::ElseIf;
                        }
                    }
                    Some(LexUnit::Block(Block {
                        kind: return_type,
                        line: tokens,
                        children,
                    }))
                } else {
                    Some(LexUnit::Block(Block {
                        kind: BlockType::Function,
                        line: tokens,
                        children,
                    }))
                }
            } else {
                Some(LexUnit::Statement(tokens))
            }
        } else {
            None
        }
    }
    fn read_identifier(line: &mut Source) -> String {
        peek_while(line, |&c| c.is_ascii_alphanumeric() || c == '_').collect::<String>()
    }
    fn read_numeric_literal(line: &mut Source) -> Literal {
        let result = peek_while(line, |&c| c.is_digit(10) || c == '.').collect::<String>();
        if result.contains(".") {
            Literal::Float(result.parse().unwrap())
        } else {
            Literal::Integer(result.parse().unwrap())
        }
    }
    fn read_symbol(line: &mut Source) -> Symbol {
        let c = line.next().unwrap();
        match c {
            '(' => OpenParenthesis,
            ')' => CloseParenthesis,
            ',' => Comma,
            '.' => Period,
            ':' => Colon,
            ';' => SemiColon,
            '+' => Plus,
            '-' => Minus,
            '*' => {
                if line.peek().map_or(false, |&c| c == '*') {
                    line.next();
                    DoubleStar
                } else {
                    Star
                }
            }
            '/' => {
                if line.peek().map_or(false, |&c| c == '/') {
                    line.next();
                    DoubleSlash
                } else {
                    Slash
                }
            }
            '=' => {
                if line.peek().map_or(false, |&c| c == '=') {
                    line.next();
                    DoubleEquals
                } else {
                    Equals
                }
            }
            '<' => {
                if line.peek().map_or(false, |&c| c == '=') {
                    line.next();
                    LessEquals
                } else {
                    Less
                }
            }
            '>' => {
                if line.peek().map_or(false, |&c| c == '=') {
                    line.next();
                    GreaterEquals
                } else {
                    Greater
                }
            }
            '!' => {
                if line.peek().map_or(false, |&c| c == '=') {
                    line.next();
                    NotEquals
                } else {
                    ExclamationPoint
                }
            }
            '?' => match line.next() {
                Some(':') => Elvis,
                Some('=') => ConditionalAssignment,
                _ => ::std::rt::begin_panic(
                    "Unrecognized Symbol",
                    &("pusl_lang/src/lexer/mod.rs", 158u32, 18u32),
                ),
            },
            _ => ::std::rt::begin_panic(
                "Unrecognized Symbol",
                &("pusl_lang/src/lexer/mod.rs", 161u32, 14u32),
            ),
        }
    }
    fn read_string_literal(line: &mut Source) -> String {
        let quote = line.next().unwrap();
        {
            match (&quote, &'"') {
                (left_val, right_val) => {
                    if !(*left_val == *right_val) {
                        {
                            ::std::rt::begin_panic_fmt(
                                &::core::fmt::Arguments::new_v1(
                                    &[
                                        "assertion failed: `(left == right)`\n  left: `",
                                        "`,\n right: `",
                                        "`",
                                    ],
                                    &match (&&*left_val, &&*right_val) {
                                        (arg0, arg1) => [
                                            ::core::fmt::ArgumentV1::new(
                                                arg0,
                                                ::core::fmt::Debug::fmt,
                                            ),
                                            ::core::fmt::ArgumentV1::new(
                                                arg1,
                                                ::core::fmt::Debug::fmt,
                                            ),
                                        ],
                                    },
                                ),
                                &("pusl_lang/src/lexer/mod.rs", 167u32, 5u32),
                            )
                        }
                    }
                }
            }
        };
        let mut string = String::new();
        while let Some(c) = line.next() {
            if c == '"' {
                break;
            } else if c == '\\' {
                let escaped = match line.next().expect("expected character after backslash") {
                    'n' => '\n',
                    't' => '\t',
                    _ => ::std::rt::begin_panic(
                        "Illegal Character after backslash",
                        &("pusl_lang/src/lexer/mod.rs", 176u32, 22u32),
                    ),
                };
                string.push(escaped);
            } else {
                string.push(c);
            }
        }
        string
    }
    fn lex_line(line: &str) -> (Vec<Token>, usize) {
        let mut cursor: Source = line.chars().peekable();
        let indentation = peek_while(&mut cursor, |&c| c == ' ').count();
        let mut tokens = Vec::new();
        while let Some(&c) = cursor.peek() {
            if c.is_ascii_alphabetic() {
                let ident = read_identifier(&mut cursor);
                let token = match ident.as_str() {
                    "for" => Some(Token::Block(BlockType::For)),
                    "if" => Some(Token::Block(BlockType::If)),
                    "else" => Some(Token::Block(BlockType::Else)),
                    "in" => Some(Token::Keyword(Keyword::In)),
                    "while" => Some(Token::Block(BlockType::While)),
                    "compare" => Some(Token::Block(BlockType::Cmp)),
                    "to" => Some(Token::Keyword(Keyword::To)),
                    "true" => Some(Token::Literal(Literal::Boolean(true))),
                    "false" => Some(Token::Literal(Literal::Boolean(false))),
                    "let" => Some(Token::Keyword(Keyword::Let)),
                    "self" => Some(Token::Keyword(Keyword::This)),
                    "return" => Some(Token::Keyword(Keyword::Return)),
                    "null" => Some(Token::Literal(Literal::Null)),
                    "fn" => Some(Token::Keyword(Keyword::Fn)),
                    "import" => Some(Token::Keyword(Keyword::Import)),
                    "as" => Some(Token::Keyword(Keyword::As)),
                    _ => None,
                }
                .unwrap_or_else(|| Token::Reference(ident));
                tokens.push(token);
            } else if c.is_digit(10) {
                tokens.push(Token::Literal(read_numeric_literal(&mut cursor)));
            } else if c == '"' {
                tokens.push(Token::Literal(Literal::String(read_string_literal(
                    &mut cursor,
                ))));
            } else if c == ' ' {
                peek_while(&mut cursor, |&c| c == ' ').count();
            } else {
                tokens.push(Token::Symbol(read_symbol(&mut cursor)));
            }
        }
        (tokens, indentation)
    }
}
pub mod parser {
    //! The parser takes the token hierarchy produced by the lexer and creates an abstract syntax tree.
    //! This is where grammatical errors are caught (lexer catches syntax errors).
    //! This data is taken in by the linearization engine before being executed.
    use crate::lexer::token::{Block, BlockType, Keyword, LexUnit, Symbol, Token};
    use crate::parser::branch::{Branch, ConditionBody};
    use crate::parser::expression::AssignmentFlags;
    use crate::parser::expression::Compare;
    use crate::parser::expression::Expression;
    use crate::parser::InBetween::{Lexeme, Parsed};
    use std::iter::Peekable;
    use std::path::PathBuf;
    pub mod branch {
        use crate::parser::ExpRef;
        pub struct ConditionBody {
            pub condition: ExpRef,
            pub body: ExpRef,
        }
        #[automatically_derived]
        #[allow(unused_qualifications)]
        impl ::core::fmt::Debug for ConditionBody {
            fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                match *self {
                    ConditionBody {
                        condition: ref __self_0_0,
                        body: ref __self_0_1,
                    } => {
                        let mut debug_trait_builder = f.debug_struct("ConditionBody");
                        let _ = debug_trait_builder.field("condition", &&(*__self_0_0));
                        let _ = debug_trait_builder.field("body", &&(*__self_0_1));
                        debug_trait_builder.finish()
                    }
                }
            }
        }
        /// Syntax Blocks which branch execution flow
        pub enum Branch {
            IfElseBlock {
                conditions: Vec<ConditionBody>,
                last: Option<ExpRef>,
            },
            WhileLoop {
                condition: ExpRef,
                body: ExpRef,
            },
            ForLoop {
                iterable: ExpRef,
                body: ExpRef,
            },
            CompareBlock {
                lhs: ExpRef,
                rhs: ExpRef,
                greater: u8,
                equal: u8,
                less: u8,
                body: Vec<ExpRef>,
            },
            Joiner {
                expressions: Vec<ExpRef>,
            },
        }
        #[automatically_derived]
        #[allow(unused_qualifications)]
        impl ::core::fmt::Debug for Branch {
            fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                match (&*self,) {
                    (&Branch::IfElseBlock {
                        conditions: ref __self_0,
                        last: ref __self_1,
                    },) => {
                        let mut debug_trait_builder = f.debug_struct("IfElseBlock");
                        let _ = debug_trait_builder.field("conditions", &&(*__self_0));
                        let _ = debug_trait_builder.field("last", &&(*__self_1));
                        debug_trait_builder.finish()
                    }
                    (&Branch::WhileLoop {
                        condition: ref __self_0,
                        body: ref __self_1,
                    },) => {
                        let mut debug_trait_builder = f.debug_struct("WhileLoop");
                        let _ = debug_trait_builder.field("condition", &&(*__self_0));
                        let _ = debug_trait_builder.field("body", &&(*__self_1));
                        debug_trait_builder.finish()
                    }
                    (&Branch::ForLoop {
                        iterable: ref __self_0,
                        body: ref __self_1,
                    },) => {
                        let mut debug_trait_builder = f.debug_struct("ForLoop");
                        let _ = debug_trait_builder.field("iterable", &&(*__self_0));
                        let _ = debug_trait_builder.field("body", &&(*__self_1));
                        debug_trait_builder.finish()
                    }
                    (&Branch::CompareBlock {
                        lhs: ref __self_0,
                        rhs: ref __self_1,
                        greater: ref __self_2,
                        equal: ref __self_3,
                        less: ref __self_4,
                        body: ref __self_5,
                    },) => {
                        let mut debug_trait_builder = f.debug_struct("CompareBlock");
                        let _ = debug_trait_builder.field("lhs", &&(*__self_0));
                        let _ = debug_trait_builder.field("rhs", &&(*__self_1));
                        let _ = debug_trait_builder.field("greater", &&(*__self_2));
                        let _ = debug_trait_builder.field("equal", &&(*__self_3));
                        let _ = debug_trait_builder.field("less", &&(*__self_4));
                        let _ = debug_trait_builder.field("body", &&(*__self_5));
                        debug_trait_builder.finish()
                    }
                    (&Branch::Joiner {
                        expressions: ref __self_0,
                    },) => {
                        let mut debug_trait_builder = f.debug_struct("Joiner");
                        let _ = debug_trait_builder.field("expressions", &&(*__self_0));
                        debug_trait_builder.finish()
                    }
                }
            }
        }
    }
    pub mod expression {
        use crate::lexer::token::Literal;
        use crate::parser::ExpRef;
        /// Syntax Blocks which are linear
        /// i.e. they will never branch
        pub enum Expression {
            Modulus {
                lhs: ExpRef,
                rhs: ExpRef,
            },
            Literal {
                value: Literal,
            },
            Reference {
                target: String,
            },
            Joiner {
                expressions: Vec<ExpRef>,
            },
            FunctionCall {
                target: String,
                arguments: Vec<ExpRef>,
            },
            MethodCall {
                target: ExpRef,
                field: String,
                arguments: Vec<ExpRef>,
            },
            FieldAccess {
                target: ExpRef,
                name: String,
            },
            Addition {
                lhs: ExpRef,
                rhs: ExpRef,
            },
            Subtract {
                lhs: ExpRef,
                rhs: ExpRef,
            },
            /// Double Duty, negate numbers and binary not
            Negate {
                operand: ExpRef,
            },
            Multiply {
                lhs: ExpRef,
                rhs: ExpRef,
            },
            Divide {
                lhs: ExpRef,
                rhs: ExpRef,
            },
            Elvis {
                lhs: ExpRef,
                rhs: ExpRef,
            },
            ReferenceAssigment {
                target: String,
                expression: ExpRef,
                flags: AssignmentFlags,
            },
            FieldAssignment {
                target: ExpRef,
                field: String,
                expression: ExpRef,
                flags: AssignmentFlags,
            },
            DivideTruncate {
                lhs: ExpRef,
                rhs: ExpRef,
            },
            Exponent {
                lhs: ExpRef,
                rhs: ExpRef,
            },
            Compare {
                lhs: ExpRef,
                rhs: ExpRef,
                operation: Compare,
            },
            And {
                lhs: ExpRef,
                rhs: ExpRef,
            },
            Or {
                lhs: ExpRef,
                rhs: ExpRef,
            },
            FunctionDeclaration {
                params: Vec<String>,
                body: ExpRef,
            },
            Return {
                value: ExpRef,
            },
        }
        #[automatically_derived]
        #[allow(unused_qualifications)]
        impl ::core::fmt::Debug for Expression {
            fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                match (&*self,) {
                    (&Expression::Modulus {
                        lhs: ref __self_0,
                        rhs: ref __self_1,
                    },) => {
                        let mut debug_trait_builder = f.debug_struct("Modulus");
                        let _ = debug_trait_builder.field("lhs", &&(*__self_0));
                        let _ = debug_trait_builder.field("rhs", &&(*__self_1));
                        debug_trait_builder.finish()
                    }
                    (&Expression::Literal {
                        value: ref __self_0,
                    },) => {
                        let mut debug_trait_builder = f.debug_struct("Literal");
                        let _ = debug_trait_builder.field("value", &&(*__self_0));
                        debug_trait_builder.finish()
                    }
                    (&Expression::Reference {
                        target: ref __self_0,
                    },) => {
                        let mut debug_trait_builder = f.debug_struct("Reference");
                        let _ = debug_trait_builder.field("target", &&(*__self_0));
                        debug_trait_builder.finish()
                    }
                    (&Expression::Joiner {
                        expressions: ref __self_0,
                    },) => {
                        let mut debug_trait_builder = f.debug_struct("Joiner");
                        let _ = debug_trait_builder.field("expressions", &&(*__self_0));
                        debug_trait_builder.finish()
                    }
                    (&Expression::FunctionCall {
                        target: ref __self_0,
                        arguments: ref __self_1,
                    },) => {
                        let mut debug_trait_builder = f.debug_struct("FunctionCall");
                        let _ = debug_trait_builder.field("target", &&(*__self_0));
                        let _ = debug_trait_builder.field("arguments", &&(*__self_1));
                        debug_trait_builder.finish()
                    }
                    (&Expression::MethodCall {
                        target: ref __self_0,
                        field: ref __self_1,
                        arguments: ref __self_2,
                    },) => {
                        let mut debug_trait_builder = f.debug_struct("MethodCall");
                        let _ = debug_trait_builder.field("target", &&(*__self_0));
                        let _ = debug_trait_builder.field("field", &&(*__self_1));
                        let _ = debug_trait_builder.field("arguments", &&(*__self_2));
                        debug_trait_builder.finish()
                    }
                    (&Expression::FieldAccess {
                        target: ref __self_0,
                        name: ref __self_1,
                    },) => {
                        let mut debug_trait_builder = f.debug_struct("FieldAccess");
                        let _ = debug_trait_builder.field("target", &&(*__self_0));
                        let _ = debug_trait_builder.field("name", &&(*__self_1));
                        debug_trait_builder.finish()
                    }
                    (&Expression::Addition {
                        lhs: ref __self_0,
                        rhs: ref __self_1,
                    },) => {
                        let mut debug_trait_builder = f.debug_struct("Addition");
                        let _ = debug_trait_builder.field("lhs", &&(*__self_0));
                        let _ = debug_trait_builder.field("rhs", &&(*__self_1));
                        debug_trait_builder.finish()
                    }
                    (&Expression::Subtract {
                        lhs: ref __self_0,
                        rhs: ref __self_1,
                    },) => {
                        let mut debug_trait_builder = f.debug_struct("Subtract");
                        let _ = debug_trait_builder.field("lhs", &&(*__self_0));
                        let _ = debug_trait_builder.field("rhs", &&(*__self_1));
                        debug_trait_builder.finish()
                    }
                    (&Expression::Negate {
                        operand: ref __self_0,
                    },) => {
                        let mut debug_trait_builder = f.debug_struct("Negate");
                        let _ = debug_trait_builder.field("operand", &&(*__self_0));
                        debug_trait_builder.finish()
                    }
                    (&Expression::Multiply {
                        lhs: ref __self_0,
                        rhs: ref __self_1,
                    },) => {
                        let mut debug_trait_builder = f.debug_struct("Multiply");
                        let _ = debug_trait_builder.field("lhs", &&(*__self_0));
                        let _ = debug_trait_builder.field("rhs", &&(*__self_1));
                        debug_trait_builder.finish()
                    }
                    (&Expression::Divide {
                        lhs: ref __self_0,
                        rhs: ref __self_1,
                    },) => {
                        let mut debug_trait_builder = f.debug_struct("Divide");
                        let _ = debug_trait_builder.field("lhs", &&(*__self_0));
                        let _ = debug_trait_builder.field("rhs", &&(*__self_1));
                        debug_trait_builder.finish()
                    }
                    (&Expression::Elvis {
                        lhs: ref __self_0,
                        rhs: ref __self_1,
                    },) => {
                        let mut debug_trait_builder = f.debug_struct("Elvis");
                        let _ = debug_trait_builder.field("lhs", &&(*__self_0));
                        let _ = debug_trait_builder.field("rhs", &&(*__self_1));
                        debug_trait_builder.finish()
                    }
                    (&Expression::ReferenceAssigment {
                        target: ref __self_0,
                        expression: ref __self_1,
                        flags: ref __self_2,
                    },) => {
                        let mut debug_trait_builder = f.debug_struct("ReferenceAssigment");
                        let _ = debug_trait_builder.field("target", &&(*__self_0));
                        let _ = debug_trait_builder.field("expression", &&(*__self_1));
                        let _ = debug_trait_builder.field("flags", &&(*__self_2));
                        debug_trait_builder.finish()
                    }
                    (&Expression::FieldAssignment {
                        target: ref __self_0,
                        field: ref __self_1,
                        expression: ref __self_2,
                        flags: ref __self_3,
                    },) => {
                        let mut debug_trait_builder = f.debug_struct("FieldAssignment");
                        let _ = debug_trait_builder.field("target", &&(*__self_0));
                        let _ = debug_trait_builder.field("field", &&(*__self_1));
                        let _ = debug_trait_builder.field("expression", &&(*__self_2));
                        let _ = debug_trait_builder.field("flags", &&(*__self_3));
                        debug_trait_builder.finish()
                    }
                    (&Expression::DivideTruncate {
                        lhs: ref __self_0,
                        rhs: ref __self_1,
                    },) => {
                        let mut debug_trait_builder = f.debug_struct("DivideTruncate");
                        let _ = debug_trait_builder.field("lhs", &&(*__self_0));
                        let _ = debug_trait_builder.field("rhs", &&(*__self_1));
                        debug_trait_builder.finish()
                    }
                    (&Expression::Exponent {
                        lhs: ref __self_0,
                        rhs: ref __self_1,
                    },) => {
                        let mut debug_trait_builder = f.debug_struct("Exponent");
                        let _ = debug_trait_builder.field("lhs", &&(*__self_0));
                        let _ = debug_trait_builder.field("rhs", &&(*__self_1));
                        debug_trait_builder.finish()
                    }
                    (&Expression::Compare {
                        lhs: ref __self_0,
                        rhs: ref __self_1,
                        operation: ref __self_2,
                    },) => {
                        let mut debug_trait_builder = f.debug_struct("Compare");
                        let _ = debug_trait_builder.field("lhs", &&(*__self_0));
                        let _ = debug_trait_builder.field("rhs", &&(*__self_1));
                        let _ = debug_trait_builder.field("operation", &&(*__self_2));
                        debug_trait_builder.finish()
                    }
                    (&Expression::And {
                        lhs: ref __self_0,
                        rhs: ref __self_1,
                    },) => {
                        let mut debug_trait_builder = f.debug_struct("And");
                        let _ = debug_trait_builder.field("lhs", &&(*__self_0));
                        let _ = debug_trait_builder.field("rhs", &&(*__self_1));
                        debug_trait_builder.finish()
                    }
                    (&Expression::Or {
                        lhs: ref __self_0,
                        rhs: ref __self_1,
                    },) => {
                        let mut debug_trait_builder = f.debug_struct("Or");
                        let _ = debug_trait_builder.field("lhs", &&(*__self_0));
                        let _ = debug_trait_builder.field("rhs", &&(*__self_1));
                        debug_trait_builder.finish()
                    }
                    (&Expression::FunctionDeclaration {
                        params: ref __self_0,
                        body: ref __self_1,
                    },) => {
                        let mut debug_trait_builder = f.debug_struct("FunctionDeclaration");
                        let _ = debug_trait_builder.field("params", &&(*__self_0));
                        let _ = debug_trait_builder.field("body", &&(*__self_1));
                        debug_trait_builder.finish()
                    }
                    (&Expression::Return {
                        value: ref __self_0,
                    },) => {
                        let mut debug_trait_builder = f.debug_struct("Return");
                        let _ = debug_trait_builder.field("value", &&(*__self_0));
                        debug_trait_builder.finish()
                    }
                }
            }
        }
        pub struct AssignmentFlags {
            bits: u8,
        }
        #[automatically_derived]
        #[allow(unused_qualifications)]
        impl ::core::marker::Copy for AssignmentFlags {}
        impl ::core::marker::StructuralPartialEq for AssignmentFlags {}
        #[automatically_derived]
        #[allow(unused_qualifications)]
        impl ::core::cmp::PartialEq for AssignmentFlags {
            #[inline]
            fn eq(&self, other: &AssignmentFlags) -> bool {
                match *other {
                    AssignmentFlags {
                        bits: ref __self_1_0,
                    } => match *self {
                        AssignmentFlags {
                            bits: ref __self_0_0,
                        } => (*__self_0_0) == (*__self_1_0),
                    },
                }
            }
            #[inline]
            fn ne(&self, other: &AssignmentFlags) -> bool {
                match *other {
                    AssignmentFlags {
                        bits: ref __self_1_0,
                    } => match *self {
                        AssignmentFlags {
                            bits: ref __self_0_0,
                        } => (*__self_0_0) != (*__self_1_0),
                    },
                }
            }
        }
        impl ::core::marker::StructuralEq for AssignmentFlags {}
        #[automatically_derived]
        #[allow(unused_qualifications)]
        impl ::core::cmp::Eq for AssignmentFlags {
            #[inline]
            #[doc(hidden)]
            fn assert_receiver_is_total_eq(&self) -> () {
                {
                    let _: ::core::cmp::AssertParamIsEq<u8>;
                }
            }
        }
        #[automatically_derived]
        #[allow(unused_qualifications)]
        impl ::core::clone::Clone for AssignmentFlags {
            #[inline]
            fn clone(&self) -> AssignmentFlags {
                {
                    let _: ::core::clone::AssertParamIsClone<u8>;
                    *self
                }
            }
        }
        #[automatically_derived]
        #[allow(unused_qualifications)]
        impl ::core::cmp::PartialOrd for AssignmentFlags {
            #[inline]
            fn partial_cmp(
                &self,
                other: &AssignmentFlags,
            ) -> ::core::option::Option<::core::cmp::Ordering> {
                match *other {
                    AssignmentFlags {
                        bits: ref __self_1_0,
                    } => match *self {
                        AssignmentFlags {
                            bits: ref __self_0_0,
                        } => match ::core::cmp::PartialOrd::partial_cmp(
                            &(*__self_0_0),
                            &(*__self_1_0),
                        ) {
                            ::core::option::Option::Some(::core::cmp::Ordering::Equal) => {
                                ::core::option::Option::Some(::core::cmp::Ordering::Equal)
                            }
                            cmp => cmp,
                        },
                    },
                }
            }
            #[inline]
            fn lt(&self, other: &AssignmentFlags) -> bool {
                match *other {
                    AssignmentFlags {
                        bits: ref __self_1_0,
                    } => match *self {
                        AssignmentFlags {
                            bits: ref __self_0_0,
                        } => {
                            ::core::option::Option::unwrap_or(
                                ::core::cmp::PartialOrd::partial_cmp(
                                    &(*__self_0_0),
                                    &(*__self_1_0),
                                ),
                                ::core::cmp::Ordering::Greater,
                            ) == ::core::cmp::Ordering::Less
                        }
                    },
                }
            }
            #[inline]
            fn le(&self, other: &AssignmentFlags) -> bool {
                match *other {
                    AssignmentFlags {
                        bits: ref __self_1_0,
                    } => match *self {
                        AssignmentFlags {
                            bits: ref __self_0_0,
                        } => {
                            ::core::option::Option::unwrap_or(
                                ::core::cmp::PartialOrd::partial_cmp(
                                    &(*__self_0_0),
                                    &(*__self_1_0),
                                ),
                                ::core::cmp::Ordering::Greater,
                            ) != ::core::cmp::Ordering::Greater
                        }
                    },
                }
            }
            #[inline]
            fn gt(&self, other: &AssignmentFlags) -> bool {
                match *other {
                    AssignmentFlags {
                        bits: ref __self_1_0,
                    } => match *self {
                        AssignmentFlags {
                            bits: ref __self_0_0,
                        } => {
                            ::core::option::Option::unwrap_or(
                                ::core::cmp::PartialOrd::partial_cmp(
                                    &(*__self_0_0),
                                    &(*__self_1_0),
                                ),
                                ::core::cmp::Ordering::Less,
                            ) == ::core::cmp::Ordering::Greater
                        }
                    },
                }
            }
            #[inline]
            fn ge(&self, other: &AssignmentFlags) -> bool {
                match *other {
                    AssignmentFlags {
                        bits: ref __self_1_0,
                    } => match *self {
                        AssignmentFlags {
                            bits: ref __self_0_0,
                        } => {
                            ::core::option::Option::unwrap_or(
                                ::core::cmp::PartialOrd::partial_cmp(
                                    &(*__self_0_0),
                                    &(*__self_1_0),
                                ),
                                ::core::cmp::Ordering::Less,
                            ) != ::core::cmp::Ordering::Less
                        }
                    },
                }
            }
        }
        #[automatically_derived]
        #[allow(unused_qualifications)]
        impl ::core::cmp::Ord for AssignmentFlags {
            #[inline]
            fn cmp(&self, other: &AssignmentFlags) -> ::core::cmp::Ordering {
                match *other {
                    AssignmentFlags {
                        bits: ref __self_1_0,
                    } => match *self {
                        AssignmentFlags {
                            bits: ref __self_0_0,
                        } => match ::core::cmp::Ord::cmp(&(*__self_0_0), &(*__self_1_0)) {
                            ::core::cmp::Ordering::Equal => ::core::cmp::Ordering::Equal,
                            cmp => cmp,
                        },
                    },
                }
            }
        }
        #[automatically_derived]
        #[allow(unused_qualifications)]
        impl ::core::hash::Hash for AssignmentFlags {
            fn hash<__H: ::core::hash::Hasher>(&self, state: &mut __H) -> () {
                match *self {
                    AssignmentFlags {
                        bits: ref __self_0_0,
                    } => ::core::hash::Hash::hash(&(*__self_0_0), state),
                }
            }
        }
        impl ::bitflags::_core::fmt::Debug for AssignmentFlags {
            fn fmt(
                &self,
                f: &mut ::bitflags::_core::fmt::Formatter,
            ) -> ::bitflags::_core::fmt::Result {
                #[allow(non_snake_case)]
                trait __BitFlags {
                    #[inline]
                    fn LET(&self) -> bool {
                        false
                    }
                    #[inline]
                    fn CONDITIONAL(&self) -> bool {
                        false
                    }
                }
                impl __BitFlags for AssignmentFlags {
                    #[allow(deprecated)]
                    #[inline]
                    fn LET(&self) -> bool {
                        if Self::LET.bits == 0 && self.bits != 0 {
                            false
                        } else {
                            self.bits & Self::LET.bits == Self::LET.bits
                        }
                    }
                    #[allow(deprecated)]
                    #[inline]
                    fn CONDITIONAL(&self) -> bool {
                        if Self::CONDITIONAL.bits == 0 && self.bits != 0 {
                            false
                        } else {
                            self.bits & Self::CONDITIONAL.bits == Self::CONDITIONAL.bits
                        }
                    }
                }
                let mut first = true;
                if <AssignmentFlags as __BitFlags>::LET(self) {
                    if !first {
                        f.write_str(" | ")?;
                    }
                    first = false;
                    f.write_str("LET")?;
                }
                if <AssignmentFlags as __BitFlags>::CONDITIONAL(self) {
                    if !first {
                        f.write_str(" | ")?;
                    }
                    first = false;
                    f.write_str("CONDITIONAL")?;
                }
                let extra_bits = self.bits & !AssignmentFlags::all().bits();
                if extra_bits != 0 {
                    if !first {
                        f.write_str(" | ")?;
                    }
                    first = false;
                    f.write_str("0x")?;
                    ::bitflags::_core::fmt::LowerHex::fmt(&extra_bits, f)?;
                }
                if first {
                    f.write_str("(empty)")?;
                }
                Ok(())
            }
        }
        impl ::bitflags::_core::fmt::Binary for AssignmentFlags {
            fn fmt(
                &self,
                f: &mut ::bitflags::_core::fmt::Formatter,
            ) -> ::bitflags::_core::fmt::Result {
                ::bitflags::_core::fmt::Binary::fmt(&self.bits, f)
            }
        }
        impl ::bitflags::_core::fmt::Octal for AssignmentFlags {
            fn fmt(
                &self,
                f: &mut ::bitflags::_core::fmt::Formatter,
            ) -> ::bitflags::_core::fmt::Result {
                ::bitflags::_core::fmt::Octal::fmt(&self.bits, f)
            }
        }
        impl ::bitflags::_core::fmt::LowerHex for AssignmentFlags {
            fn fmt(
                &self,
                f: &mut ::bitflags::_core::fmt::Formatter,
            ) -> ::bitflags::_core::fmt::Result {
                ::bitflags::_core::fmt::LowerHex::fmt(&self.bits, f)
            }
        }
        impl ::bitflags::_core::fmt::UpperHex for AssignmentFlags {
            fn fmt(
                &self,
                f: &mut ::bitflags::_core::fmt::Formatter,
            ) -> ::bitflags::_core::fmt::Result {
                ::bitflags::_core::fmt::UpperHex::fmt(&self.bits, f)
            }
        }
        #[allow(dead_code)]
        impl AssignmentFlags {
            pub const LET: AssignmentFlags = AssignmentFlags { bits: 0b00000001 };
            pub const CONDITIONAL: AssignmentFlags = AssignmentFlags { bits: 0b00000010 };
            /// Returns an empty set of flags
            #[inline]
            pub const fn empty() -> AssignmentFlags {
                AssignmentFlags { bits: 0 }
            }
            /// Returns the set containing all flags.
            #[inline]
            pub const fn all() -> AssignmentFlags {
                #[allow(non_snake_case)]
                trait __BitFlags {
                    const LET: u8 = 0;
                    const CONDITIONAL: u8 = 0;
                }
                impl __BitFlags for AssignmentFlags {
                    #[allow(deprecated)]
                    const LET: u8 = Self::LET.bits;
                    #[allow(deprecated)]
                    const CONDITIONAL: u8 = Self::CONDITIONAL.bits;
                }
                AssignmentFlags {
                    bits: <AssignmentFlags as __BitFlags>::LET
                        | <AssignmentFlags as __BitFlags>::CONDITIONAL,
                }
            }
            /// Returns the raw value of the flags currently stored.
            #[inline]
            pub const fn bits(&self) -> u8 {
                self.bits
            }
            /// Convert from underlying bit representation, unless that
            /// representation contains bits that do not correspond to a flag.
            #[inline]
            pub fn from_bits(bits: u8) -> ::bitflags::_core::option::Option<AssignmentFlags> {
                if (bits & !AssignmentFlags::all().bits()) == 0 {
                    ::bitflags::_core::option::Option::Some(AssignmentFlags { bits })
                } else {
                    ::bitflags::_core::option::Option::None
                }
            }
            /// Convert from underlying bit representation, dropping any bits
            /// that do not correspond to flags.
            #[inline]
            pub const fn from_bits_truncate(bits: u8) -> AssignmentFlags {
                AssignmentFlags {
                    bits: bits & AssignmentFlags::all().bits,
                }
            }
            /// Convert from underlying bit representation, preserving all
            /// bits (even those not corresponding to a defined flag).
            #[inline]
            pub const unsafe fn from_bits_unchecked(bits: u8) -> AssignmentFlags {
                AssignmentFlags { bits }
            }
            /// Returns `true` if no flags are currently stored.
            #[inline]
            pub const fn is_empty(&self) -> bool {
                self.bits() == AssignmentFlags::empty().bits()
            }
            /// Returns `true` if all flags are currently set.
            #[inline]
            pub const fn is_all(&self) -> bool {
                self.bits == AssignmentFlags::all().bits
            }
            /// Returns `true` if there are flags common to both `self` and `other`.
            #[inline]
            pub const fn intersects(&self, other: AssignmentFlags) -> bool {
                !AssignmentFlags {
                    bits: self.bits & other.bits,
                }
                .is_empty()
            }
            /// Returns `true` all of the flags in `other` are contained within `self`.
            #[inline]
            pub const fn contains(&self, other: AssignmentFlags) -> bool {
                (self.bits & other.bits) == other.bits
            }
            /// Inserts the specified flags in-place.
            #[inline]
            pub fn insert(&mut self, other: AssignmentFlags) {
                self.bits |= other.bits;
            }
            /// Removes the specified flags in-place.
            #[inline]
            pub fn remove(&mut self, other: AssignmentFlags) {
                self.bits &= !other.bits;
            }
            /// Toggles the specified flags in-place.
            #[inline]
            pub fn toggle(&mut self, other: AssignmentFlags) {
                self.bits ^= other.bits;
            }
            /// Inserts or removes the specified flags depending on the passed value.
            #[inline]
            pub fn set(&mut self, other: AssignmentFlags, value: bool) {
                if value {
                    self.insert(other);
                } else {
                    self.remove(other);
                }
            }
        }
        impl ::bitflags::_core::ops::BitOr for AssignmentFlags {
            type Output = AssignmentFlags;
            /// Returns the union of the two sets of flags.
            #[inline]
            fn bitor(self, other: AssignmentFlags) -> AssignmentFlags {
                AssignmentFlags {
                    bits: self.bits | other.bits,
                }
            }
        }
        impl ::bitflags::_core::ops::BitOrAssign for AssignmentFlags {
            /// Adds the set of flags.
            #[inline]
            fn bitor_assign(&mut self, other: AssignmentFlags) {
                self.bits |= other.bits;
            }
        }
        impl ::bitflags::_core::ops::BitXor for AssignmentFlags {
            type Output = AssignmentFlags;
            /// Returns the left flags, but with all the right flags toggled.
            #[inline]
            fn bitxor(self, other: AssignmentFlags) -> AssignmentFlags {
                AssignmentFlags {
                    bits: self.bits ^ other.bits,
                }
            }
        }
        impl ::bitflags::_core::ops::BitXorAssign for AssignmentFlags {
            /// Toggles the set of flags.
            #[inline]
            fn bitxor_assign(&mut self, other: AssignmentFlags) {
                self.bits ^= other.bits;
            }
        }
        impl ::bitflags::_core::ops::BitAnd for AssignmentFlags {
            type Output = AssignmentFlags;
            /// Returns the intersection between the two sets of flags.
            #[inline]
            fn bitand(self, other: AssignmentFlags) -> AssignmentFlags {
                AssignmentFlags {
                    bits: self.bits & other.bits,
                }
            }
        }
        impl ::bitflags::_core::ops::BitAndAssign for AssignmentFlags {
            /// Disables all flags disabled in the set.
            #[inline]
            fn bitand_assign(&mut self, other: AssignmentFlags) {
                self.bits &= other.bits;
            }
        }
        impl ::bitflags::_core::ops::Sub for AssignmentFlags {
            type Output = AssignmentFlags;
            /// Returns the set difference of the two sets of flags.
            #[inline]
            fn sub(self, other: AssignmentFlags) -> AssignmentFlags {
                AssignmentFlags {
                    bits: self.bits & !other.bits,
                }
            }
        }
        impl ::bitflags::_core::ops::SubAssign for AssignmentFlags {
            /// Disables all flags enabled in the set.
            #[inline]
            fn sub_assign(&mut self, other: AssignmentFlags) {
                self.bits &= !other.bits;
            }
        }
        impl ::bitflags::_core::ops::Not for AssignmentFlags {
            type Output = AssignmentFlags;
            /// Returns the complement of this set of flags.
            #[inline]
            fn not(self) -> AssignmentFlags {
                AssignmentFlags { bits: !self.bits } & AssignmentFlags::all()
            }
        }
        impl ::bitflags::_core::iter::Extend<AssignmentFlags> for AssignmentFlags {
            fn extend<T: ::bitflags::_core::iter::IntoIterator<Item = AssignmentFlags>>(
                &mut self,
                iterator: T,
            ) {
                for item in iterator {
                    self.insert(item)
                }
            }
        }
        impl ::bitflags::_core::iter::FromIterator<AssignmentFlags> for AssignmentFlags {
            fn from_iter<T: ::bitflags::_core::iter::IntoIterator<Item = AssignmentFlags>>(
                iterator: T,
            ) -> AssignmentFlags {
                let mut result = Self::empty();
                result.extend(iterator);
                result
            }
        }
        pub enum Compare {
            Less,
            LessEqual,
            Greater,
            GreaterEqual,
            Equal,
            NotEqual,
        }
        #[automatically_derived]
        #[allow(unused_qualifications)]
        impl ::core::marker::Copy for Compare {}
        #[automatically_derived]
        #[allow(unused_qualifications)]
        impl ::core::clone::Clone for Compare {
            #[inline]
            fn clone(&self) -> Compare {
                {
                    *self
                }
            }
        }
        #[automatically_derived]
        #[allow(unused_qualifications)]
        impl ::core::fmt::Debug for Compare {
            fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                match (&*self,) {
                    (&Compare::Less,) => {
                        let mut debug_trait_builder = f.debug_tuple("Less");
                        debug_trait_builder.finish()
                    }
                    (&Compare::LessEqual,) => {
                        let mut debug_trait_builder = f.debug_tuple("LessEqual");
                        debug_trait_builder.finish()
                    }
                    (&Compare::Greater,) => {
                        let mut debug_trait_builder = f.debug_tuple("Greater");
                        debug_trait_builder.finish()
                    }
                    (&Compare::GreaterEqual,) => {
                        let mut debug_trait_builder = f.debug_tuple("GreaterEqual");
                        debug_trait_builder.finish()
                    }
                    (&Compare::Equal,) => {
                        let mut debug_trait_builder = f.debug_tuple("Equal");
                        debug_trait_builder.finish()
                    }
                    (&Compare::NotEqual,) => {
                        let mut debug_trait_builder = f.debug_tuple("NotEqual");
                        debug_trait_builder.finish()
                    }
                }
            }
        }
    }
    /// All structures in the Abstract Syntax Tree
    /// Branch and Expression or different because Branch will affect execution flow
    /// Expression executes in a deterministic order
    pub enum Eval {
        Expression(Expression),
        Branch(Branch),
    }
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::fmt::Debug for Eval {
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            match (&*self,) {
                (&Eval::Expression(ref __self_0),) => {
                    let mut debug_trait_builder = f.debug_tuple("Expression");
                    let _ = debug_trait_builder.field(&&(*__self_0));
                    debug_trait_builder.finish()
                }
                (&Eval::Branch(ref __self_0),) => {
                    let mut debug_trait_builder = f.debug_tuple("Branch");
                    let _ = debug_trait_builder.field(&&(*__self_0));
                    debug_trait_builder.finish()
                }
            }
        }
    }
    pub struct ParsedFile {
        pub expr: ExpRef,
        pub imports: Vec<Import>,
    }
    pub struct Import {
        pub path: PathBuf,
        pub alias: String,
    }
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::fmt::Debug for Import {
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            match *self {
                Import {
                    path: ref __self_0_0,
                    alias: ref __self_0_1,
                } => {
                    let mut debug_trait_builder = f.debug_struct("Import");
                    let _ = debug_trait_builder.field("path", &&(*__self_0_0));
                    let _ = debug_trait_builder.field("alias", &&(*__self_0_1));
                    debug_trait_builder.finish()
                }
            }
        }
    }
    pub type ExpRef = Box<Eval>;
    pub fn parse<I>(source: I) -> ParsedFile
    where
        I: IntoIterator<Item = LexUnit>,
    {
        let mut iter = source.into_iter().peekable();
        let mut imports = Vec::new();
        while let Some(LexUnit::Statement(tokens)) = iter.peek() {
            if let Some(&Token::Keyword(Keyword::Import)) = tokens.first() {
                if let Some(LexUnit::Statement(tokens)) = iter.next() {
                    let import = parse_import(tokens);
                    imports.push(import);
                } else {
                    {
                        ::std::rt::begin_panic(
                            "Invariant",
                            &("pusl_lang/src/parser/mod.rs", 51u32, 17u32),
                        )
                    };
                }
            } else {
                break;
            }
        }
        let mut expr_list = Vec::new();
        while let Some(unit) = iter.next() {
            let expr = parse_lex_unit(unit, &mut iter);
            expr_list.push(expr);
        }
        let expr = Box::new(Eval::Expression(Expression::Joiner {
            expressions: expr_list,
        }));
        ParsedFile { expr, imports }
    }
    fn parse_import<I>(tokens: I) -> Import
    where
        I: IntoIterator<Item = Token>,
    {
        let mut iter = tokens.into_iter().peekable();
        {
            match (&Some(Token::Keyword(Keyword::Import)), &iter.next()) {
                (left_val, right_val) => {
                    if !(*left_val == *right_val) {
                        {
                            ::std::rt::begin_panic_fmt(
                                &::core::fmt::Arguments::new_v1(
                                    &[
                                        "assertion failed: `(left == right)`\n  left: `",
                                        "`,\n right: `",
                                        "`",
                                    ],
                                    &match (&&*left_val, &&*right_val) {
                                        (arg0, arg1) => [
                                            ::core::fmt::ArgumentV1::new(
                                                arg0,
                                                ::core::fmt::Debug::fmt,
                                            ),
                                            ::core::fmt::ArgumentV1::new(
                                                arg1,
                                                ::core::fmt::Debug::fmt,
                                            ),
                                        ],
                                    },
                                ),
                                &("pusl_lang/src/parser/mod.rs", 73u32, 5u32),
                            )
                        }
                    }
                }
            }
        };
        let mut path = PathBuf::new();
        while let Some(token) = iter.next() {
            if let Token::Reference(name) = token {
                path.push(name);
                match iter.next() {
                    Some(Token::Symbol(Symbol::Period)) => {}
                    Some(Token::Keyword(Keyword::As)) => {
                        break;
                    }
                    _ => ::std::rt::begin_panic(
                        "explicit panic",
                        &("pusl_lang/src/parser/mod.rs", 83u32, 22u32),
                    ),
                }
            } else {
                {
                    ::std::rt::begin_panic(
                        "Invalid Import",
                        &("pusl_lang/src/parser/mod.rs", 86u32, 13u32),
                    )
                }
            }
        }
        let alias = if let Some(Token::Reference(alias)) = iter.next() {
            alias
        } else {
            {
                {
                    ::std::rt::begin_panic(
                        "explicit panic",
                        &("pusl_lang/src/parser/mod.rs", 93u32, 9u32),
                    )
                }
            }
        };
        {
            match (&iter.next(), &None) {
                (left_val, right_val) => {
                    if !(*left_val == *right_val) {
                        {
                            ::std::rt::begin_panic_fmt(
                                &::core::fmt::Arguments::new_v1(
                                    &[
                                        "assertion failed: `(left == right)`\n  left: `",
                                        "`,\n right: `",
                                        "`",
                                    ],
                                    &match (&&*left_val, &&*right_val) {
                                        (arg0, arg1) => [
                                            ::core::fmt::ArgumentV1::new(
                                                arg0,
                                                ::core::fmt::Debug::fmt,
                                            ),
                                            ::core::fmt::ArgumentV1::new(
                                                arg1,
                                                ::core::fmt::Debug::fmt,
                                            ),
                                        ],
                                    },
                                ),
                                &("pusl_lang/src/parser/mod.rs", 96u32, 5u32),
                            )
                        }
                    }
                }
            }
        };
        Import { path, alias }
    }
    fn parse_lex_unit<I>(unit: LexUnit, stream: &mut Peekable<I>) -> ExpRef
    where
        I: Iterator<Item = LexUnit>,
    {
        match unit {
            LexUnit::Block(block) => parse_branch(block, stream),
            LexUnit::Statement(tokens) => parse_statement(tokens),
        }
    }
    /// Parse a while loop
    fn parse_while(block: Block) -> Branch {
        {
            match (&(BlockType::While), &(block.kind)) {
                (left_val, right_val) => {
                    if !(*left_val == *right_val) {
                        {
                            :: std :: rt :: begin_panic_fmt ( & :: core :: fmt :: Arguments :: new_v1 ( & [ "assertion failed: `(left == right)`\n  left: `" , "`,\n right: `" , "`: " ] , & match ( & & * left_val , & & * right_val , & :: core :: fmt :: Arguments :: new_v1 ( & [ "If the function is called, should be parsing a while loop" ] , & match ( ) { ( ) => [ ] , } ) ) { ( arg0 , arg1 , arg2 ) => [ :: core :: fmt :: ArgumentV1 :: new ( arg0 , :: core :: fmt :: Debug :: fmt ) , :: core :: fmt :: ArgumentV1 :: new ( arg1 , :: core :: fmt :: Debug :: fmt ) , :: core :: fmt :: ArgumentV1 :: new ( arg2 , :: core :: fmt :: Display :: fmt ) ] , } ) , & ( "pusl_lang/src/parser/mod.rs" , 114u32 , 5u32 ) )
                        }
                    }
                }
            }
        };
        let mut condition_func = |it: &mut dyn Iterator<Item = Token>| {
            {
                match (&Some(Token::Block(BlockType::While)), &it.next()) {
                    (left_val, right_val) => {
                        if !(*left_val == *right_val) {
                            {
                                ::std::rt::begin_panic_fmt(
                                    &::core::fmt::Arguments::new_v1(
                                        &[
                                            "assertion failed: `(left == right)`\n  left: `",
                                            "`,\n right: `",
                                            "`",
                                        ],
                                        &match (&&*left_val, &&*right_val) {
                                            (arg0, arg1) => [
                                                ::core::fmt::ArgumentV1::new(
                                                    arg0,
                                                    ::core::fmt::Debug::fmt,
                                                ),
                                                ::core::fmt::ArgumentV1::new(
                                                    arg1,
                                                    ::core::fmt::Debug::fmt,
                                                ),
                                            ],
                                        },
                                    ),
                                    &("pusl_lang/src/parser/mod.rs", 120u32, 9u32),
                                )
                            }
                        }
                    }
                }
            };
            parse_expression(it)
        };
        let (condition, body) = parse_condition_body(block, &mut condition_func);
        Branch::WhileLoop { condition, body }
    }
    /// Parse a group of if, else if, ..., else blocks
    fn parse_if_else<I>(if_block: Block, block_stream: &mut Peekable<I>) -> Branch
    where
        I: Iterator<Item = LexUnit>,
    {
        let mut conditions = Vec::<ConditionBody>::new();
        let mut if_func = |it: &mut dyn Iterator<Item = Token>| {
            {
                match (&(Some(Token::Block(BlockType::If))), &(it.next())) {
                    (left_val, right_val) => {
                        if !(*left_val == *right_val) {
                            {
                                :: std :: rt :: begin_panic_fmt ( & :: core :: fmt :: Arguments :: new_v1 ( & [ "assertion failed: `(left == right)`\n  left: `" , "`,\n right: `" , "`: " ] , & match ( & & * left_val , & & * right_val , & :: core :: fmt :: Arguments :: new_v1 ( & [ "If the function is called, should be parsing an if block" ] , & match ( ) { ( ) => [ ] , } ) ) { ( arg0 , arg1 , arg2 ) => [ :: core :: fmt :: ArgumentV1 :: new ( arg0 , :: core :: fmt :: Debug :: fmt ) , :: core :: fmt :: ArgumentV1 :: new ( arg1 , :: core :: fmt :: Debug :: fmt ) , :: core :: fmt :: ArgumentV1 :: new ( arg2 , :: core :: fmt :: Display :: fmt ) ] , } ) , & ( "pusl_lang/src/parser/mod.rs" , 135u32 , 9u32 ) )
                            }
                        }
                    }
                }
            };
            parse_expression(it)
        };
        let (if_condition, if_body) = parse_condition_body(if_block, &mut if_func);
        conditions.push(ConditionBody {
            condition: if_condition,
            body: if_body,
        });
        let mut elif_func = |it: &mut dyn Iterator<Item = Token>| {
            {
                match (&(Some(Token::Block(BlockType::Else))), &(it.next())) {
                    (left_val, right_val) => {
                        if !(*left_val == *right_val) {
                            {
                                :: std :: rt :: begin_panic_fmt ( & :: core :: fmt :: Arguments :: new_v1 ( & [ "assertion failed: `(left == right)`\n  left: `" , "`,\n right: `" , "`: " ] , & match ( & & * left_val , & & * right_val , & :: core :: fmt :: Arguments :: new_v1 ( & [ "If the function is called, should be parsing an if else block" ] , & match ( ) { ( ) => [ ] , } ) ) { ( arg0 , arg1 , arg2 ) => [ :: core :: fmt :: ArgumentV1 :: new ( arg0 , :: core :: fmt :: Debug :: fmt ) , :: core :: fmt :: ArgumentV1 :: new ( arg1 , :: core :: fmt :: Debug :: fmt ) , :: core :: fmt :: ArgumentV1 :: new ( arg2 , :: core :: fmt :: Display :: fmt ) ] , } ) , & ( "pusl_lang/src/parser/mod.rs" , 149u32 , 9u32 ) )
                            }
                        }
                    }
                }
            };
            {
                match (&(Some(Token::Block(BlockType::If))), &(it.next())) {
                    (left_val, right_val) => {
                        if !(*left_val == *right_val) {
                            {
                                :: std :: rt :: begin_panic_fmt ( & :: core :: fmt :: Arguments :: new_v1 ( & [ "assertion failed: `(left == right)`\n  left: `" , "`,\n right: `" , "`: " ] , & match ( & & * left_val , & & * right_val , & :: core :: fmt :: Arguments :: new_v1 ( & [ "If the function is called, should be parsing an if else block" ] , & match ( ) { ( ) => [ ] , } ) ) { ( arg0 , arg1 , arg2 ) => [ :: core :: fmt :: ArgumentV1 :: new ( arg0 , :: core :: fmt :: Debug :: fmt ) , :: core :: fmt :: ArgumentV1 :: new ( arg1 , :: core :: fmt :: Debug :: fmt ) , :: core :: fmt :: ArgumentV1 :: new ( arg2 , :: core :: fmt :: Display :: fmt ) ] , } ) , & ( "pusl_lang/src/parser/mod.rs" , 154u32 , 9u32 ) )
                            }
                        }
                    }
                }
            };
            parse_expression(it)
        };
        while block_stream.peek().map_or(false, |lex_unit| {
            if let LexUnit::Block(block) = lex_unit {
                block.kind == BlockType::ElseIf
            } else {
                false
            }
        }) {
            if let Some(LexUnit::Block(elif_block)) = block_stream.next() {
                let (elif_condition, elif_body) = parse_condition_body(elif_block, &mut elif_func);
                conditions.push(ConditionBody {
                    condition: elif_condition,
                    body: elif_body,
                })
            } else {
                {
                    ::std::rt::begin_panic(
                        "Invariant Violated",
                        &("pusl_lang/src/parser/mod.rs", 175u32, 13u32),
                    )
                }
            }
        }
        let mut else_func = |it: &mut dyn Iterator<Item = Token>| {
            {
                match (&(Some(Token::Block(BlockType::Else))), &(it.next())) {
                    (left_val, right_val) => {
                        if !(*left_val == *right_val) {
                            {
                                :: std :: rt :: begin_panic_fmt ( & :: core :: fmt :: Arguments :: new_v1 ( & [ "assertion failed: `(left == right)`\n  left: `" , "`,\n right: `" , "`: " ] , & match ( & & * left_val , & & * right_val , & :: core :: fmt :: Arguments :: new_v1 ( & [ "If the function is called, should be parsing an else block" ] , & match ( ) { ( ) => [ ] , } ) ) { ( arg0 , arg1 , arg2 ) => [ :: core :: fmt :: ArgumentV1 :: new ( arg0 , :: core :: fmt :: Debug :: fmt ) , :: core :: fmt :: ArgumentV1 :: new ( arg1 , :: core :: fmt :: Debug :: fmt ) , :: core :: fmt :: ArgumentV1 :: new ( arg2 , :: core :: fmt :: Display :: fmt ) ] , } ) , & ( "pusl_lang/src/parser/mod.rs" , 180u32 , 9u32 ) )
                            }
                        }
                    }
                }
            };
            {
                match (&(None), &(it.next())) {
                    (left_val, right_val) => {
                        if !(*left_val == *right_val) {
                            {
                                ::std::rt::begin_panic_fmt(
                                    &::core::fmt::Arguments::new_v1(
                                        &[
                                            "assertion failed: `(left == right)`\n  left: `",
                                            "`,\n right: `",
                                            "`: ",
                                        ],
                                        &match (
                                            &&*left_val,
                                            &&*right_val,
                                            &::core::fmt::Arguments::new_v1(
                                                &["An else block shouldn\'t have a condition"],
                                                &match () {
                                                    () => [],
                                                },
                                            ),
                                        ) {
                                            (arg0, arg1, arg2) => [
                                                ::core::fmt::ArgumentV1::new(
                                                    arg0,
                                                    ::core::fmt::Debug::fmt,
                                                ),
                                                ::core::fmt::ArgumentV1::new(
                                                    arg1,
                                                    ::core::fmt::Debug::fmt,
                                                ),
                                                ::core::fmt::ArgumentV1::new(
                                                    arg2,
                                                    ::core::fmt::Display::fmt,
                                                ),
                                            ],
                                        },
                                    ),
                                    &("pusl_lang/src/parser/mod.rs", 185u32, 9u32),
                                )
                            }
                        }
                    }
                }
            };
        };
        let else_body = if block_stream.peek().map_or(false, |lex_unit| {
            if let LexUnit::Block(block) = lex_unit {
                block.kind == BlockType::Else
            } else {
                false
            }
        }) {
            if let Some(LexUnit::Block(else_block)) = block_stream.next() {
                let ((), else_body) = parse_condition_body(else_block, &mut else_func);
                Some(else_body)
            } else {
                {
                    ::std::rt::begin_panic(
                        "Invariant Violated",
                        &("pusl_lang/src/parser/mod.rs", 198u32, 13u32),
                    )
                }
            }
        } else {
            None
        };
        Branch::IfElseBlock {
            conditions,
            last: else_body,
        }
    }
    /// Parse a line and its connected blocks
    fn parse_condition_body<F, R>(block: Block, condition_parse: &mut F) -> (R, ExpRef)
    where
        F: FnMut(&mut dyn Iterator<Item = Token>) -> R,
    {
        let Block { line, children, .. } = block;
        let (condition, body) = if children.is_empty() {
            {
                ::std::rt::begin_panic(
                    "Block has no children",
                    &("pusl_lang/src/parser/mod.rs", 217u32, 9u32),
                )
            }
        } else {
            let mut found_colon = false;
            let condition = condition_parse(&mut line.into_iter().take_while(|token| {
                found_colon = token == &Token::Symbol(Symbol::Colon);
                !found_colon
            }));
            if !found_colon {
                {
                    ::std::rt::begin_panic(
                        "Parsing a while loop with a body, colon should be end of my line",
                        &("pusl_lang/src/parser/mod.rs", 224u32, 9u32),
                    )
                }
            };
            let mut child_iter = children.into_iter().peekable();
            let mut body_pieces = Vec::new();
            while let Some(next) = child_iter.next() {
                body_pieces.push(parse_lex_unit(next, &mut child_iter));
            }
            let body = Expression::Joiner {
                expressions: body_pieces,
            };
            (condition, Box::new(Eval::Expression(body)))
        };
        (condition, body)
    }
    fn parse_for(block: Block) -> Branch {
        {
            match (&(BlockType::For), &(block.kind)) {
                (left_val, right_val) => {
                    if !(*left_val == *right_val) {
                        {
                            :: std :: rt :: begin_panic_fmt ( & :: core :: fmt :: Arguments :: new_v1 ( & [ "assertion failed: `(left == right)`\n  left: `" , "`,\n right: `" , "`: " ] , & match ( & & * left_val , & & * right_val , & :: core :: fmt :: Arguments :: new_v1 ( & [ "If the function is called, should be parsing a for loop" ] , & match ( ) { ( ) => [ ] , } ) ) { ( arg0 , arg1 , arg2 ) => [ :: core :: fmt :: ArgumentV1 :: new ( arg0 , :: core :: fmt :: Debug :: fmt ) , :: core :: fmt :: ArgumentV1 :: new ( arg1 , :: core :: fmt :: Debug :: fmt ) , :: core :: fmt :: ArgumentV1 :: new ( arg2 , :: core :: fmt :: Display :: fmt ) ] , } ) , & ( "pusl_lang/src/parser/mod.rs" , 242u32 , 5u32 ) )
                        }
                    }
                }
            }
        };
        let mut condition_func = |it: &mut dyn Iterator<Item = Token>| {
            {
                match (&Some(Token::Block(BlockType::For)), &it.next()) {
                    (left_val, right_val) => {
                        if !(*left_val == *right_val) {
                            {
                                ::std::rt::begin_panic_fmt(
                                    &::core::fmt::Arguments::new_v1(
                                        &[
                                            "assertion failed: `(left == right)`\n  left: `",
                                            "`,\n right: `",
                                            "`",
                                        ],
                                        &match (&&*left_val, &&*right_val) {
                                            (arg0, arg1) => [
                                                ::core::fmt::ArgumentV1::new(
                                                    arg0,
                                                    ::core::fmt::Debug::fmt,
                                                ),
                                                ::core::fmt::ArgumentV1::new(
                                                    arg1,
                                                    ::core::fmt::Debug::fmt,
                                                ),
                                            ],
                                        },
                                    ),
                                    &("pusl_lang/src/parser/mod.rs", 248u32, 9u32),
                                )
                            }
                        }
                    }
                }
            };
            {
                ::std::rt::begin_panic(
                    "not yet implemented",
                    &("pusl_lang/src/parser/mod.rs", 249u32, 9u32),
                )
            }
        };
        let (_, _) = parse_condition_body(block, &mut condition_func);
        {
            ::std::rt::begin_panic(
                "not yet implemented",
                &("pusl_lang/src/parser/mod.rs", 253u32, 5u32),
            )
        }
    }
    fn parse_compare(block: Block) -> Branch {
        {
            match (&(BlockType::Cmp), &(block.kind)) {
                (left_val, right_val) => {
                    if !(*left_val == *right_val) {
                        {
                            :: std :: rt :: begin_panic_fmt ( & :: core :: fmt :: Arguments :: new_v1 ( & [ "assertion failed: `(left == right)`\n  left: `" , "`,\n right: `" , "`: " ] , & match ( & & * left_val , & & * right_val , & :: core :: fmt :: Arguments :: new_v1 ( & [ "If the function is called, should be parsing a for loop" ] , & match ( ) { ( ) => [ ] , } ) ) { ( arg0 , arg1 , arg2 ) => [ :: core :: fmt :: ArgumentV1 :: new ( arg0 , :: core :: fmt :: Debug :: fmt ) , :: core :: fmt :: ArgumentV1 :: new ( arg1 , :: core :: fmt :: Debug :: fmt ) , :: core :: fmt :: ArgumentV1 :: new ( arg2 , :: core :: fmt :: Display :: fmt ) ] , } ) , & ( "pusl_lang/src/parser/mod.rs" , 257u32 , 5u32 ) )
                        }
                    }
                }
            }
        };
        let mut condition_func = |it: &mut dyn Iterator<Item = Token>| {
            {
                match (&Some(Token::Block(BlockType::Cmp)), &it.next()) {
                    (left_val, right_val) => {
                        if !(*left_val == *right_val) {
                            {
                                ::std::rt::begin_panic_fmt(
                                    &::core::fmt::Arguments::new_v1(
                                        &[
                                            "assertion failed: `(left == right)`\n  left: `",
                                            "`,\n right: `",
                                            "`",
                                        ],
                                        &match (&&*left_val, &&*right_val) {
                                            (arg0, arg1) => [
                                                ::core::fmt::ArgumentV1::new(
                                                    arg0,
                                                    ::core::fmt::Debug::fmt,
                                                ),
                                                ::core::fmt::ArgumentV1::new(
                                                    arg1,
                                                    ::core::fmt::Debug::fmt,
                                                ),
                                            ],
                                        },
                                    ),
                                    &("pusl_lang/src/parser/mod.rs", 263u32, 9u32),
                                )
                            }
                        }
                    }
                }
            };
            {
                ::std::rt::begin_panic(
                    "not yet implemented",
                    &("pusl_lang/src/parser/mod.rs", 264u32, 9u32),
                )
            }
        };
        let (_, _) = parse_condition_body(block, &mut condition_func);
        {
            ::std::rt::begin_panic(
                "not yet implemented",
                &("pusl_lang/src/parser/mod.rs", 268u32, 5u32),
            )
        }
    }
    /// Parse a branching block (type of [Branch](crate::parser::branch::Branch))
    fn parse_branch<I>(block: Block, block_stream: &mut Peekable<I>) -> ExpRef
    where
        I: Iterator<Item = LexUnit>,
    {
        let block = match block.kind {
            BlockType::If => Eval::Branch(parse_if_else(block, block_stream)),
            BlockType::While => Eval::Branch(parse_while(block)),
            BlockType::For => Eval::Branch(parse_for(block)),
            BlockType::Cmp => Eval::Branch(parse_compare(block)),
            BlockType::Function => Eval::Expression(parse_function_declaration(block)),
            BlockType::Else | BlockType::ElseIf => ::std::rt::begin_panic(
                "Parsed else without if",
                &("pusl_lang/src/parser/mod.rs", 282u32, 48u32),
            ),
        };
        Box::new(block)
    }
    fn parse_function_declaration(block: Block) -> Expression {
        let mut declaration_func = |it: &mut dyn Iterator<Item = Token>| {
            let line = it.collect::<Vec<_>>();
            let is_assignment = line
                .iter()
                .enumerate()
                .filter_map(|(index, token)| {
                    if let Token::Symbol(symbol) = token {
                        Some((index, *symbol))
                    } else {
                        None
                    }
                })
                .find(|&(_, token)| {
                    token == Symbol::Equals || token == Symbol::ConditionalAssignment
                });
            if let Some((index, kind)) = is_assignment {
                let mut is_let = false;
                let mut tokens = line.into_boxed_slice();
                let (mut lhs, mut rhs) = tokens.split_at_mut(index);
                rhs = &mut rhs[1..];
                if let Some(Token::Keyword(Keyword::Let)) = lhs.first() {
                    lhs = &mut lhs[1..];
                    is_let = true;
                }
                let mut lhs_iter = lhs.iter().cloned();
                let target = parse_identifier(&mut lhs_iter);
                let mut rhs_iter = rhs.iter().cloned();
                {
                    match (&Some(Token::Keyword(Keyword::Fn)), &rhs_iter.next()) {
                        (left_val, right_val) => {
                            if !(*left_val == *right_val) {
                                {
                                    ::std::rt::begin_panic_fmt(
                                        &::core::fmt::Arguments::new_v1(
                                            &[
                                                "assertion failed: `(left == right)`\n  left: `",
                                                "`,\n right: `",
                                                "`",
                                            ],
                                            &match (&&*left_val, &&*right_val) {
                                                (arg0, arg1) => [
                                                    ::core::fmt::ArgumentV1::new(
                                                        arg0,
                                                        ::core::fmt::Debug::fmt,
                                                    ),
                                                    ::core::fmt::ArgumentV1::new(
                                                        arg1,
                                                        ::core::fmt::Debug::fmt,
                                                    ),
                                                ],
                                            },
                                        ),
                                        &("pusl_lang/src/parser/mod.rs", 319u32, 13u32),
                                    )
                                }
                            }
                        }
                    }
                };
                {
                    match (
                        &Some(Token::Symbol(Symbol::OpenParenthesis)),
                        &rhs_iter.next(),
                    ) {
                        (left_val, right_val) => {
                            if !(*left_val == *right_val) {
                                {
                                    ::std::rt::begin_panic_fmt(
                                        &::core::fmt::Arguments::new_v1(
                                            &[
                                                "assertion failed: `(left == right)`\n  left: `",
                                                "`,\n right: `",
                                                "`",
                                            ],
                                            &match (&&*left_val, &&*right_val) {
                                                (arg0, arg1) => [
                                                    ::core::fmt::ArgumentV1::new(
                                                        arg0,
                                                        ::core::fmt::Debug::fmt,
                                                    ),
                                                    ::core::fmt::ArgumentV1::new(
                                                        arg1,
                                                        ::core::fmt::Debug::fmt,
                                                    ),
                                                ],
                                            },
                                        ),
                                        &("pusl_lang/src/parser/mod.rs", 320u32, 13u32),
                                    )
                                }
                            }
                        }
                    }
                };
                let parameters = parse_function_parameters(&mut rhs_iter);
                {
                    match (&None, &rhs_iter.next()) {
                        (left_val, right_val) => {
                            if !(*left_val == *right_val) {
                                {
                                    ::std::rt::begin_panic_fmt(
                                        &::core::fmt::Arguments::new_v1(
                                            &[
                                                "assertion failed: `(left == right)`\n  left: `",
                                                "`,\n right: `",
                                                "`",
                                            ],
                                            &match (&&*left_val, &&*right_val) {
                                                (arg0, arg1) => [
                                                    ::core::fmt::ArgumentV1::new(
                                                        arg0,
                                                        ::core::fmt::Debug::fmt,
                                                    ),
                                                    ::core::fmt::ArgumentV1::new(
                                                        arg1,
                                                        ::core::fmt::Debug::fmt,
                                                    ),
                                                ],
                                            },
                                        ),
                                        &("pusl_lang/src/parser/mod.rs", 325u32, 13u32),
                                    )
                                }
                            }
                        }
                    }
                };
                let mut flags = AssignmentFlags::empty();
                if is_let {
                    flags |= AssignmentFlags::LET;
                }
                if kind == Symbol::ConditionalAssignment {
                    flags |= AssignmentFlags::CONDITIONAL;
                }
                (target, flags, parameters)
            } else {
                {
                    ::std::rt::begin_panic(
                        "Function declaration without assignment",
                        &("pusl_lang/src/parser/mod.rs", 336u32, 13u32),
                    )
                }
            }
        };
        let ((target, flags, params), body) = parse_condition_body(block, &mut declaration_func);
        let decl_expr = Expression::FunctionDeclaration { params, body };
        let decl_expr = Box::new(Eval::Expression(decl_expr));
        match target {
            Identifier::Reference(name) => Expression::ReferenceAssigment {
                target: name,
                expression: decl_expr,
                flags,
            },
            Identifier::Field(target, field) => Expression::FieldAssignment {
                target,
                field,
                expression: decl_expr,
                flags,
            },
        }
    }
    enum Identifier {
        Reference(String),
        Field(ExpRef, String),
    }
    fn parse_identifier<I>(tokens: &mut I) -> Identifier
    where
        I: Iterator<Item = Token>,
    {
        let mut tokens = tokens.collect::<Vec<_>>();
        let name = if let Some(Token::Reference(name)) = tokens.pop() {
            name
        } else {
            {
                {
                    ::std::rt::begin_panic(
                        "explicit panic",
                        &("pusl_lang/src/parser/mod.rs", 371u32, 9u32),
                    )
                }
            }
        };
        let ident = if tokens.is_empty() {
            Identifier::Reference(name)
        } else {
            let mut name_stack = Vec::new();
            while let Some(Token::Symbol(Symbol::Period)) = tokens.pop() {
                if let Some(Token::Reference(name)) = tokens.pop() {
                    name_stack.push(name);
                } else {
                    {
                        ::std::rt::begin_panic(
                            "Invalid Identifier",
                            &("pusl_lang/src/parser/mod.rs", 382u32, 17u32),
                        )
                    };
                }
            }
            let mut target = Box::new(Eval::Expression(Expression::Reference {
                target: name_stack.pop().unwrap(),
            }));
            while let Some(name) = name_stack.pop() {
                target = Box::new(Eval::Expression(Expression::FieldAccess { target, name }))
            }
            Identifier::Field(target, name)
        };
        if !tokens.is_empty() {
            {
                ::std::rt::begin_panic(
                    "assertion failed: tokens.is_empty()",
                    &("pusl_lang/src/parser/mod.rs", 393u32, 5u32),
                )
            }
        };
        ident
    }
    fn parse_statement(tokens: Vec<Token>) -> ExpRef {
        let is_assignment = tokens
            .iter()
            .enumerate()
            .filter_map(|(index, token)| {
                if let Token::Symbol(symbol) = token {
                    Some((index, *symbol))
                } else {
                    None
                }
            })
            .find(|&(_, token)| token == Symbol::Equals || token == Symbol::ConditionalAssignment);
        if let Some((index, kind)) = is_assignment {
            let mut is_let = false;
            let mut tokens = tokens.into_boxed_slice();
            let (mut lhs, mut rhs) = tokens.split_at_mut(index);
            rhs = &mut rhs[1..];
            if let Some(Token::Keyword(Keyword::Let)) = lhs.first() {
                lhs = &mut lhs[1..];
                is_let = true;
            }
            let mut lhs_iter = lhs.iter().cloned();
            let target = parse_identifier(&mut lhs_iter);
            let mut rhs_iter = rhs.iter().cloned();
            let expression = parse_expression(&mut rhs_iter);
            let mut flags = AssignmentFlags::empty();
            if is_let {
                flags |= AssignmentFlags::LET;
            }
            if kind == Symbol::ConditionalAssignment {
                flags |= AssignmentFlags::CONDITIONAL;
            }
            let expr = match target {
                Identifier::Reference(name) => Expression::ReferenceAssigment {
                    target: name,
                    expression,
                    flags,
                },
                Identifier::Field(target, field) => Expression::FieldAssignment {
                    target,
                    field,
                    expression,
                    flags,
                },
            };
            Box::new(Eval::Expression(expr))
        } else {
            parse_expression(&mut tokens.into_iter())
        }
    }
    enum InBetween {
        Lexeme(Token),
        Parsed(ExpRef),
    }
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::fmt::Debug for InBetween {
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            match (&*self,) {
                (&InBetween::Lexeme(ref __self_0),) => {
                    let mut debug_trait_builder = f.debug_tuple("Lexeme");
                    let _ = debug_trait_builder.field(&&(*__self_0));
                    debug_trait_builder.finish()
                }
                (&InBetween::Parsed(ref __self_0),) => {
                    let mut debug_trait_builder = f.debug_tuple("Parsed");
                    let _ = debug_trait_builder.field(&&(*__self_0));
                    debug_trait_builder.finish()
                }
            }
        }
    }
    fn parse_inside_parenthesis(tokens: &mut dyn Iterator<Item = Token>) -> ExpRef {
        let mut level = 1;
        let mut take_while = tokens.take_while(|token| {
            match token {
                Token::Symbol(Symbol::OpenParenthesis) => level += 1,
                Token::Symbol(Symbol::CloseParenthesis) => level -= 1,
                _ => {}
            };
            level > 0
        });
        parse_expression(&mut take_while)
    }
    fn parse_function_parameters(tokens: &mut dyn Iterator<Item = Token>) -> Vec<String> {
        let mut parameters = Vec::new();
        while let Some(parameter) = tokens.next() {
            if let Token::Reference(name) = parameter {
                parameters.push(name);
            } else {
                {
                    ::std::rt::begin_panic(
                        "Expected Function Parameter Name",
                        &("pusl_lang/src/parser/mod.rs", 483u32, 13u32),
                    )
                }
            }
            match tokens.next() {
                Some(Token::Symbol(Symbol::Comma)) => {}
                Some(Token::Symbol(Symbol::CloseParenthesis)) => break,
                Some(_) => ::std::rt::begin_panic(
                    "Expected Comma or Closing Parenthesis",
                    &("pusl_lang/src/parser/mod.rs", 488u32, 24u32),
                ),
                None => ::std::rt::begin_panic(
                    "Unexpected End of Line",
                    &("pusl_lang/src/parser/mod.rs", 489u32, 21u32),
                ),
            }
        }
        parameters
    }
    fn parse_function_arguments(tokens: &mut dyn Iterator<Item = Token>) -> Expression {
        let mut next = true;
        let mut arguments = Vec::new();
        while next {
            let mut level = 0;
            let mut take_while = tokens
                .take_while(|token| {
                    match token {
                        Token::Symbol(Symbol::OpenParenthesis) => level += 1,
                        Token::Symbol(Symbol::CloseParenthesis) => {
                            level -= 1;
                            if level < 0 {
                                next = false;
                                return false;
                            }
                        }
                        Token::Symbol(Symbol::Comma) => {
                            if level == 0 {
                                return false;
                            }
                        }
                        _ => {}
                    };
                    true
                })
                .peekable();
            if take_while.peek().is_none() {
                if !!next {
                    {
                        ::std::rt::begin_panic(
                            "assertion failed: !next",
                            &("pusl_lang/src/parser/mod.rs", 524u32, 13u32),
                        )
                    }
                };
                break;
            }
            let expr = parse_expression(&mut take_while);
            arguments.push(expr);
        }
        Expression::FunctionCall {
            target: String::new(),
            arguments,
        }
    }
    fn parser_pass<I>(
        progress: I,
        targets: Vec<(Token, Box<dyn Fn(ExpRef, ExpRef) -> Expression>)>,
    ) -> Vec<InBetween>
    where
        I: IntoIterator<Item = InBetween>,
    {
        let mut result = Vec::new();
        let mut iter = progress.into_iter();
        while let Some(next) = iter.next() {
            let next_between = if let Lexeme(token) = next {
                if let Some((_, func)) = targets.iter().find(|(target, _)| target == &token) {
                    let lhs_exp = if let Some(Parsed(exp_ref)) = result.pop() {
                        exp_ref
                    } else {
                        {
                            {
                                ::std::rt::begin_panic(
                                    "explicit panic",
                                    &("pusl_lang/src/parser/mod.rs", 552u32, 21u32),
                                )
                            }
                        }
                    };
                    let rhs_exp = if let Some(Parsed(exp_ref)) = iter.next() {
                        exp_ref
                    } else {
                        {
                            {
                                ::std::rt::begin_panic(
                                    "explicit panic",
                                    &("pusl_lang/src/parser/mod.rs", 557u32, 21u32),
                                )
                            }
                        }
                    };
                    let expr = func(lhs_exp, rhs_exp);
                    Parsed(Box::new(Eval::Expression(expr)))
                } else {
                    Lexeme(token)
                }
            } else {
                next
            };
            result.push(next_between)
        }
        result
    }
    fn parser_pass_function_call<I>(progress: I) -> Vec<InBetween>
    where
        I: IntoIterator<Item = InBetween>,
    {
        let mut result = Vec::new();
        let mut iter = progress.into_iter();
        while let Some(next) = iter.next() {
            let next_between = if let Parsed(exp_ref) = next {
                if let Eval::Expression(Expression::FunctionCall { arguments, .. }) = *exp_ref {
                    let call = if let Some(Parsed(exp_ref)) = result.pop() {
                        match *exp_ref {
                            Eval::Expression(Expression::Reference { target }) => {
                                Expression::FunctionCall { target, arguments }
                            }
                            Eval::Expression(Expression::FieldAccess { target, name }) => {
                                Expression::MethodCall {
                                    target,
                                    field: name,
                                    arguments,
                                }
                            }
                            _ => ::std::rt::begin_panic(
                                "explicit panic",
                                &("pusl_lang/src/parser/mod.rs", 596u32, 30u32),
                            ),
                        }
                    } else {
                        {
                            {
                                ::std::rt::begin_panic(
                                    "explicit panic",
                                    &("pusl_lang/src/parser/mod.rs", 599u32, 21u32),
                                )
                            }
                        }
                    };
                    Parsed(Box::new(Eval::Expression(call)))
                } else {
                    Parsed(exp_ref)
                }
            } else {
                next
            };
            result.push(next_between);
        }
        result
    }
    fn parser_pass_unary<I>(
        progress: I,
        targets: Vec<(Token, Box<dyn Fn(ExpRef) -> Expression>)>,
    ) -> Vec<InBetween>
    where
        I: IntoIterator<Item = InBetween>,
    {
        let mut result = Vec::new();
        let mut iter = progress.into_iter();
        while let Some(next) = iter.next() {
            let next_between = if let Lexeme(token) = next {
                if let Some((_, func)) = targets.iter().find(|(kind, _)| &token == kind) {
                    let exp_ref = if let Some(Parsed(exp_ref)) = iter.next() {
                        exp_ref
                    } else {
                        {
                            {
                                ::std::rt::begin_panic(
                                    "explicit panic",
                                    &("pusl_lang/src/parser/mod.rs", 630u32, 21u32),
                                )
                            }
                        }
                    };
                    let expr = func(exp_ref);
                    Parsed(Box::new(Eval::Expression(expr)))
                } else {
                    Lexeme(token)
                }
            } else {
                next
            };
            result.push(next_between);
        }
        result
    }
    /// Parse an expression from tokens until a specified token is reached (consumes said token)
    fn parse_expression(tokens: &mut dyn Iterator<Item = Token>) -> ExpRef {
        let mut between = Vec::new();
        while let Some(token) = tokens.next() {
            let next = match token {
                Token::Literal(literal) => {
                    Parsed(Box::new(Eval::Expression(Expression::Literal {
                        value: literal,
                    })))
                }
                Token::Reference(name) => {
                    Parsed(Box::new(Eval::Expression(Expression::Reference {
                        target: name,
                    })))
                }
                Token::Symbol(Symbol::OpenParenthesis) => {
                    if let Some(Parsed(_)) = between.last() {
                        let expr = parse_function_arguments(tokens);
                        Parsed(Box::new(Eval::Expression(expr)))
                    } else {
                        Parsed(parse_inside_parenthesis(tokens))
                    }
                }
                other_token => Lexeme(other_token),
            };
            between.push(next);
        }
        between = parser_pass(
            between,
            <[_]>::into_vec(box [(
                Token::Symbol(Symbol::Period),
                Box::new(|lhs, rhs| {
                    let reference = if let Eval::Expression(Expression::Reference { target }) = *rhs
                    {
                        target
                    } else {
                        {
                            ::std::rt::begin_panic(
                                "Cannot access field without reference",
                                &("pusl_lang/src/parser/mod.rs", 679u32, 21u32),
                            )
                        }
                    };
                    Expression::FieldAccess {
                        target: lhs,
                        name: reference,
                    }
                }),
            )]),
        );
        between = parser_pass_function_call(between);
        between = parser_pass_unary(
            between,
            <[_]>::into_vec(box [(
                Token::Symbol(Symbol::ExclamationPoint),
                Box::new(|target| Expression::Negate { operand: target }),
            )]),
        );
        between = parser_pass(
            between,
            <[_]>::into_vec(box [(
                Token::Symbol(Symbol::DoubleStar),
                Box::new(|lhs, rhs| Expression::Exponent { lhs, rhs }),
            )]),
        );
        between = parser_pass(
            between,
            <[_]>::into_vec(box [
                (
                    Token::Symbol(Symbol::Star),
                    Box::new(|lhs, rhs| Expression::Multiply { lhs, rhs }),
                ),
                (
                    Token::Symbol(Symbol::Slash),
                    Box::new(|lhs, rhs| Expression::Divide { lhs, rhs }),
                ),
                (
                    Token::Symbol(Symbol::DoubleSlash),
                    Box::new(|lhs, rhs| Expression::DivideTruncate { lhs, rhs }),
                ),
                (
                    Token::Symbol(Symbol::Percent),
                    Box::new(|lhs, rhs| Expression::Modulus { lhs, rhs }),
                ),
            ]),
        );
        between = parser_pass(
            between,
            <[_]>::into_vec(box [
                (
                    Token::Symbol(Symbol::Plus),
                    Box::new(|lhs, rhs| Expression::Addition { lhs, rhs }),
                ),
                (
                    Token::Symbol(Symbol::Minus),
                    Box::new(|lhs, rhs| Expression::Subtract { lhs, rhs }),
                ),
            ]),
        );
        between = parser_pass(
            between,
            <[_]>::into_vec(box [(
                Token::Symbol(Symbol::And),
                Box::new(|lhs, rhs| Expression::And { lhs, rhs }),
            )]),
        );
        between = parser_pass(
            between,
            <[_]>::into_vec(box [(
                Token::Symbol(Symbol::Or),
                Box::new(|lhs, rhs| Expression::Or { lhs, rhs }),
            )]),
        );
        between = parser_pass(
            between,
            <[_]>::into_vec(box [
                (
                    Token::Symbol(Symbol::DoubleEquals),
                    Box::new(|lhs, rhs| Expression::Compare {
                        lhs,
                        rhs,
                        operation: Compare::Equal,
                    }),
                ),
                (
                    Token::Symbol(Symbol::Less),
                    Box::new(|lhs, rhs| Expression::Compare {
                        lhs,
                        rhs,
                        operation: Compare::Less,
                    }),
                ),
                (
                    Token::Symbol(Symbol::LessEquals),
                    Box::new(|lhs, rhs| Expression::Compare {
                        lhs,
                        rhs,
                        operation: Compare::LessEqual,
                    }),
                ),
                (
                    Token::Symbol(Symbol::Greater),
                    Box::new(|lhs, rhs| Expression::Compare {
                        lhs,
                        rhs,
                        operation: Compare::Greater,
                    }),
                ),
                (
                    Token::Symbol(Symbol::GreaterEquals),
                    Box::new(|lhs, rhs| Expression::Compare {
                        lhs,
                        rhs,
                        operation: Compare::GreaterEqual,
                    }),
                ),
                (
                    Token::Symbol(Symbol::NotEquals),
                    Box::new(|lhs, rhs| Expression::Compare {
                        lhs,
                        rhs,
                        operation: Compare::NotEqual,
                    }),
                ),
            ]),
        );
        between = parser_pass(
            between,
            <[_]>::into_vec(box [(
                Token::Symbol(Symbol::Elvis),
                Box::new(|lhs, rhs| Expression::Elvis { lhs, rhs }),
            )]),
        );
        between = parser_pass_unary(
            between,
            <[_]>::into_vec(box [(
                Token::Keyword(Keyword::Return),
                Box::new(|target| Expression::Return { value: target }),
            )]),
        );
        let expr = if let Some(Parsed(exp_ref)) = between.pop() {
            exp_ref
        } else {
            {
                {
                    ::std::rt::begin_panic(
                        "explicit panic",
                        &("pusl_lang/src/parser/mod.rs", 821u32, 9u32),
                    )
                }
            }
        };
        if !between.is_empty() {
            {
                ::std::rt::begin_panic(
                    "assertion failed: between.is_empty()",
                    &("pusl_lang/src/parser/mod.rs", 823u32, 5u32),
                )
            }
        };
        expr
    }
}
