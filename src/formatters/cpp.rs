use std::collections::HashSet;

use crate::{
    bytecode::{
        address_index::AddressIndex,
        expr::{Expr, ExprKind, TextLiteral},
        refs::{ClassRef, FunctionRef, PropertyRef, StructRef},
        types::{Address, BytecodeOffset},
    },
    formatters::theme::Theme,
};

pub struct CppFormatter<'a> {
    indent_level: usize,
    address_index: &'a AddressIndex<'a>,
    referenced_offsets: HashSet<BytecodeOffset>,
    statement_prefix: String,
}

/// Context for formatting expressions - tracks the current object being operated on
#[derive(Clone)]
pub enum FormatContext {
    /// Implicit 'this' context
    This,
    /// Explicit object context
    Object(String),
}

impl<'a> CppFormatter<'a> {
    pub fn new(
        address_index: &'a AddressIndex<'a>,
        referenced_offsets: HashSet<BytecodeOffset>,
    ) -> Self {
        Self {
            indent_level: 0,
            address_index,
            referenced_offsets,
            statement_prefix: String::new(),
        }
    }

    /// Check if a function is a KismetMathLibrary operator and format it accordingly
    fn try_format_as_operator(&self, full_path: &str, params: &[String]) -> Option<String> {
        // Unary operators
        if params.len() == 1 {
            let operand = &params[0];
            return match full_path {
                "/Script/Engine.KismetMathLibrary:Not_PreBool" => Some(format!("!{}", operand)),
                "/Script/Engine.KismetMathLibrary:NegateFloat" => Some(format!("-{}", operand)),
                "/Script/Engine.KismetMathLibrary:NegateInt" => Some(format!("-{}", operand)),
                "/Script/Engine.KismetMathLibrary:NegateInt64" => Some(format!("-{}", operand)),
                _ => None,
            };
        }

        // Binary operators
        if params.len() == 2 {
            let left = &params[0];
            let right = &params[1];

            return match full_path {
                // Logical operators
                "/Script/Engine.KismetMathLibrary:BooleanAND" => {
                    Some(format!("({} && {})", left, right))
                }
                "/Script/Engine.KismetMathLibrary:BooleanOR" => {
                    Some(format!("({} || {})", left, right))
                }
                "/Script/Engine.KismetMathLibrary:BooleanXOR" => {
                    Some(format!("({} ^ {})", left, right))
                }

                // Integer arithmetic
                "/Script/Engine.KismetMathLibrary:Add_IntInt" => {
                    Some(format!("({} + {})", left, right))
                }
                "/Script/Engine.KismetMathLibrary:Subtract_IntInt" => {
                    Some(format!("({} - {})", left, right))
                }
                "/Script/Engine.KismetMathLibrary:Multiply_IntInt" => {
                    Some(format!("({} * {})", left, right))
                }
                "/Script/Engine.KismetMathLibrary:Divide_IntInt" => {
                    Some(format!("({} / {})", left, right))
                }
                "/Script/Engine.KismetMathLibrary:Percent_IntInt" => {
                    Some(format!("({} % {})", left, right))
                }

                // Float arithmetic
                "/Script/Engine.KismetMathLibrary:Add_FloatFloat" => {
                    Some(format!("({} + {})", left, right))
                }
                "/Script/Engine.KismetMathLibrary:Subtract_FloatFloat" => {
                    Some(format!("({} - {})", left, right))
                }
                "/Script/Engine.KismetMathLibrary:Multiply_FloatFloat" => {
                    Some(format!("({} * {})", left, right))
                }
                "/Script/Engine.KismetMathLibrary:Divide_FloatFloat" => {
                    Some(format!("({} / {})", left, right))
                }

                // Double arithmetic
                "/Script/Engine.KismetMathLibrary:Add_DoubleDouble" => {
                    Some(format!("({} + {})", left, right))
                }
                "/Script/Engine.KismetMathLibrary:Subtract_DoubleDouble" => {
                    Some(format!("({} - {})", left, right))
                }
                "/Script/Engine.KismetMathLibrary:Multiply_DoubleDouble" => {
                    Some(format!("({} * {})", left, right))
                }
                "/Script/Engine.KismetMathLibrary:Divide_DoubleDouble" => {
                    Some(format!("({} / {})", left, right))
                }

                // Integer comparisons
                "/Script/Engine.KismetMathLibrary:EqualEqual_IntInt" => {
                    Some(format!("({} == {})", left, right))
                }
                "/Script/Engine.KismetMathLibrary:NotEqual_IntInt" => {
                    Some(format!("({} != {})", left, right))
                }
                "/Script/Engine.KismetMathLibrary:Greater_IntInt" => {
                    Some(format!("({} > {})", left, right))
                }
                "/Script/Engine.KismetMathLibrary:GreaterEqual_IntInt" => {
                    Some(format!("({} >= {})", left, right))
                }
                "/Script/Engine.KismetMathLibrary:Less_IntInt" => {
                    Some(format!("({} < {})", left, right))
                }
                "/Script/Engine.KismetMathLibrary:LessEqual_IntInt" => {
                    Some(format!("({} <= {})", left, right))
                }

                // Byte comparisons
                "/Script/Engine.KismetMathLibrary:EqualEqual_ByteByte" => {
                    Some(format!("({} == {})", left, right))
                }
                "/Script/Engine.KismetMathLibrary:NotEqual_ByteByte" => {
                    Some(format!("({} != {})", left, right))
                }
                "/Script/Engine.KismetMathLibrary:Greater_ByteByte" => {
                    Some(format!("({} > {})", left, right))
                }
                "/Script/Engine.KismetMathLibrary:GreaterEqual_ByteByte" => {
                    Some(format!("({} >= {})", left, right))
                }
                "/Script/Engine.KismetMathLibrary:Less_ByteByte" => {
                    Some(format!("({} < {})", left, right))
                }
                "/Script/Engine.KismetMathLibrary:LessEqual_ByteByte" => {
                    Some(format!("({} <= {})", left, right))
                }

                // Float comparisons
                "/Script/Engine.KismetMathLibrary:EqualEqual_DoubleDouble" => {
                    Some(format!("({} == {})", left, right))
                }
                "/Script/Engine.KismetMathLibrary:NotEqual_DoubleDouble" => {
                    Some(format!("({} != {})", left, right))
                }
                "/Script/Engine.KismetMathLibrary:Greater_DoubleDouble" => {
                    Some(format!("({} > {})", left, right))
                }
                "/Script/Engine.KismetMathLibrary:GreaterEqual_DoubleDouble" => {
                    Some(format!("({} >= {})", left, right))
                }
                "/Script/Engine.KismetMathLibrary:Less_DoubleDouble" => {
                    Some(format!("({} < {})", left, right))
                }
                "/Script/Engine.KismetMathLibrary:LessEqual_DoubleDouble" => {
                    Some(format!("({} <= {})", left, right))
                }

                _ => None,
            };
        }

        None
    }

    fn resolve_property(&self, prop: &PropertyRef) -> &str {
        self.address_index
            .resolve_property(prop.address)
            .map(|p| p.property.name.as_str())
            .unwrap_or("<err resolving prop>")
    }

    fn resolve_object(&self, address: Address) -> &str {
        let obj_info = self.address_index.resolve_object(address).unwrap();
        obj_info.path.rsplit('/').next().unwrap_or(obj_info.path)
    }

    fn resolve_class(&self, class: &ClassRef) -> &str {
        self.resolve_object(class.address)
    }

    fn resolve_struct(&self, s: &StructRef) -> &str {
        self.resolve_object(s.address)
    }

    fn resolve_function<'b>(&'b self, func: &'b FunctionRef) -> &'b str {
        match func {
            FunctionRef::ByName(name) => name.as_str(),
            FunctionRef::ByAddress(addr) => self
                .address_index
                .resolve_object(*addr)
                .map(|o| o.path)
                .unwrap_or("<err resolving func>"),
        }
    }

    fn indent(&self) -> String {
        format!(
            "{}{}",
            "    ".repeat(self.indent_level),
            self.statement_prefix
        )
    }

    fn add_indent(&mut self) {
        self.indent_level += 1;
    }

    fn drop_indent(&mut self) {
        if self.indent_level > 0 {
            self.indent_level -= 1;
        }
    }

    fn format_label(&self, offset: BytecodeOffset) -> String {
        Theme::label(format!("Label_0x{:X}", offset.as_usize())).to_string()
    }

    pub fn set_indent_level(&mut self, level: usize) {
        self.indent_level = level;
    }

    pub fn set_statement_prefix(&mut self, prefix: String) {
        self.statement_prefix = prefix;
    }

    pub fn clear_statement_prefix(&mut self) {
        self.statement_prefix.clear();
    }

    pub fn format(&mut self, expressions: &[Expr]) {
        for expr in expressions {
            // Only print label if this offset is referenced
            if self.referenced_offsets.contains(&expr.offset) {
                println!("{}{}:", self.indent(), self.format_label(expr.offset));
            }
            self.add_indent();
            self.format_statement(expr);
            self.drop_indent();
        }
    }

    pub fn format_statement(&mut self, expr: &Expr) {
        match &expr.kind {
            // Assignments
            ExprKind::Let {
                property: _,
                variable,
                value,
            } => {
                let var = self.format_expr_inline(variable, &FormatContext::This);
                let val = self.format_expr_inline(value, &FormatContext::This);
                println!("{}{} = {};", self.indent(), var, val);
            }
            ExprKind::LetObj { variable, value }
            | ExprKind::LetWeakObjPtr { variable, value }
            | ExprKind::LetBool { variable, value }
            | ExprKind::LetDelegate { variable, value }
            | ExprKind::LetMulticastDelegate { variable, value } => {
                let var = self.format_expr_inline(variable, &FormatContext::This);
                let val = self.format_expr_inline(value, &FormatContext::This);
                println!("{}{} = {};", self.indent(), var, val);
            }
            ExprKind::LetValueOnPersistentFrame { property, value } => {
                let prop_name = self.resolve_property(property);
                println!(
                    "{}// PersistentFrame: {}",
                    self.indent(),
                    Theme::comment(prop_name)
                );
                let val = self.format_expr_inline(value, &FormatContext::This);
                println!("{}{} = {};", self.indent(), Theme::variable(prop_name), val);
            }

            // Control flow
            ExprKind::Return(ret_expr) => {
                let expr_str = self.format_expr_inline(ret_expr, &FormatContext::This);
                if expr_str == "<Nothing>" || expr_str.is_empty() {
                    println!("{}return;", self.indent());
                } else {
                    println!("{}return {};", self.indent(), expr_str);
                }
            }
            ExprKind::Jump { target } => {
                println!("{}goto {};", self.indent(), self.format_label(*target));
            }
            ExprKind::JumpIfNot { condition, target } => {
                let cond = self.format_expr_inline(condition, &FormatContext::This);
                println!(
                    "{}if (!{}) goto {};",
                    self.indent(),
                    cond,
                    self.format_label(*target)
                );
            }
            ExprKind::ComputedJump { offset_expr } => {
                let expr = self.format_expr_inline(offset_expr, &FormatContext::This);
                println!("{}goto {};", self.indent(), expr);
            }
            ExprKind::SwitchValue {
                index,
                cases,
                default,
                end_offset: _,
            } => {
                let index_expr = self.format_expr_inline(index, &FormatContext::This);
                println!("{}switch ({}) {{", self.indent(), index_expr);
                self.add_indent();

                for case in cases {
                    let case_val = self.format_expr_inline(&case.case_value, &FormatContext::This);
                    println!("{}case {}:", self.indent(), case_val);
                    self.add_indent();
                    let result = self.format_expr_inline(&case.result, &FormatContext::This);
                    if !result.is_empty() {
                        println!("{}{};", self.indent(), result);
                    }
                    println!("{}break;", self.indent());
                    self.drop_indent();
                }

                println!("{}default:", self.indent());
                self.add_indent();
                let default_result = self.format_expr_inline(default, &FormatContext::This);
                if !default_result.is_empty() {
                    println!("{}{};", self.indent(), default_result);
                }
                println!("{}break;", self.indent());
                self.drop_indent();

                self.drop_indent();
                println!("{}}}", self.indent());
            }

            // Delegates
            ExprKind::BindDelegate {
                func_name,
                delegate_expr,
                object_expr,
            } => {
                let delegate = self.format_expr_inline(delegate_expr, &FormatContext::This);
                let object = self.format_expr_inline(object_expr, &FormatContext::This);
                println!(
                    "{}{}.BindDynamic({}, &{}::{});",
                    self.indent(),
                    delegate,
                    object,
                    object,
                    func_name.as_str()
                );
            }
            ExprKind::AddMulticastDelegate {
                delegate_expr,
                to_add_expr,
            } => {
                let delegate = self.format_expr_inline(delegate_expr, &FormatContext::This);
                let to_add = self.format_expr_inline(to_add_expr, &FormatContext::This);
                println!("{}{}.AddDynamic({});", self.indent(), delegate, to_add);
            }
            ExprKind::RemoveMulticastDelegate {
                delegate_expr,
                to_remove_expr,
            } => {
                let delegate = self.format_expr_inline(delegate_expr, &FormatContext::This);
                let to_remove = self.format_expr_inline(to_remove_expr, &FormatContext::This);
                println!(
                    "{}{}.RemoveDynamic({});",
                    self.indent(),
                    delegate,
                    to_remove
                );
            }
            ExprKind::ClearMulticastDelegate(delegate_expr) => {
                let delegate = self.format_expr_inline(delegate_expr, &FormatContext::This);
                println!("{}{}.Clear();", self.indent(), delegate);
            }
            ExprKind::CallMulticastDelegate {
                stack_node: _,
                delegate_expr,
                params,
            } => {
                let delegate = self.format_expr_inline(delegate_expr, &FormatContext::This);
                let param_strs: Vec<String> = params
                    .iter()
                    .map(|p| self.format_expr_inline(p, &FormatContext::This))
                    .collect();
                println!(
                    "{}{}.Broadcast({});",
                    self.indent(),
                    delegate,
                    param_strs.join(", ")
                );
            }

            // Debug/instrumentation
            ExprKind::Assert {
                line,
                in_debug: _,
                condition,
            } => {
                let cond = self.format_expr_inline(condition, &FormatContext::This);
                println!("{}check({}); // line {}", self.indent(), cond, line);
            }
            ExprKind::PushExecutionFlow { push_offset } => {
                println!(
                    "{}PushExecutionFlow({});",
                    self.indent(),
                    self.format_label(*push_offset)
                );
            }
            ExprKind::PopExecutionFlow => {
                println!("{}PopExecutionFlow;", self.indent());
            }
            ExprKind::PopExecutionFlowIfNot { condition } => {
                let cond = self.format_expr_inline(condition, &FormatContext::This);
                println!("{}PopExecutionFlowIfNot({});", self.indent(), cond);
            }
            ExprKind::Breakpoint => {
                println!("{} <<< BREAKPOINT >>>", self.indent());
            }
            ExprKind::Tracepoint | ExprKind::WireTracepoint => {
                println!("{} <<< TRACEPOINT >>>", self.indent());
            }
            ExprKind::InstrumentationEvent { event_type } => {
                println!(
                    "{} <<< INSTRUMENTATION EVENT {} >>>",
                    self.indent(),
                    event_type
                );
            }
            ExprKind::EndOfScript => {
                println!("{}// End of script", self.indent());
            }

            // Everything else - try to format as expression statement
            _ => {
                let expr_str = self.format_expr_inline(expr, &FormatContext::This);
                if !expr_str.is_empty() {
                    println!("{}{};", self.indent(), expr_str);
                }
            }
        }
    }

    pub fn format_expr_inline(&self, expr: &Expr, context: &FormatContext) -> String {
        match &expr.kind {
            // Variables
            ExprKind::LocalVariable(prop)
            | ExprKind::LocalOutVariable(prop)
            | ExprKind::ClassSparseDataVariable(prop) => {
                let name = self.resolve_property(prop);
                Theme::variable(name).to_string()
            }
            ExprKind::InstanceVariable(prop) => {
                let name = self.resolve_property(prop);
                let obj = match context {
                    FormatContext::This => Theme::object_ref("this").to_string(),
                    FormatContext::Object(obj) => obj.clone(),
                };
                format!("{}.{}", obj, Theme::variable(name))
            }
            ExprKind::DefaultVariable(prop) => {
                let name = self.resolve_property(prop);
                format!(
                    "{}.{}",
                    Theme::object_ref("GetDefaultObject()"),
                    Theme::variable(name)
                )
            }

            // Constants - integers
            ExprKind::IntZero => Theme::numeric("0").to_string(),
            ExprKind::IntOne => Theme::numeric("1").to_string(),
            ExprKind::IntConst(val) => Theme::numeric(val).to_string(),
            ExprKind::Int64Const(val) => Theme::numeric(format!("{}LL", val)).to_string(),
            ExprKind::UInt64Const(val) => Theme::numeric(format!("{}ULL", val)).to_string(),
            ExprKind::ByteConst(val) | ExprKind::IntConstByte(val) => {
                Theme::numeric(val).to_string()
            }

            // Constants - floating point
            ExprKind::FloatConst(val) => Theme::numeric(format!("{}f", val)).to_string(),

            // Constants - strings
            ExprKind::StringConst(val) => crate::formatters::theme::quoted_string(val).to_string(),
            ExprKind::UnicodeStringConst(val) => {
                Theme::string(format!("TEXT(\"{}\")", val)).to_string()
            }
            ExprKind::NameConst(name) => {
                Theme::string(format!("FName(\"{}\")", name.as_str())).to_string()
            }

            // Constants - vectors and transforms
            ExprKind::VectorConst { x, y, z } => {
                Theme::type_name(format!("FVector({}, {}, {})", x, y, z)).to_string()
            }
            ExprKind::RotationConst { pitch, yaw, roll } => {
                Theme::type_name(format!("FRotator({}, {}, {})", pitch, yaw, roll)).to_string()
            }
            ExprKind::TransformConst {
                rot_x,
                rot_y,
                rot_z,
                rot_w,
                trans_x,
                trans_y,
                trans_z,
                scale_x,
                scale_y,
                scale_z,
            } => Theme::type_name(format!(
                "FTransform(FQuat({}, {}, {}, {}), FVector({}, {}, {}), FVector({}, {}, {}))",
                rot_x, rot_y, rot_z, rot_w, trans_x, trans_y, trans_z, scale_x, scale_y, scale_z
            ))
            .to_string(),

            // Constants - special values
            ExprKind::True => Theme::keyword("true").to_string(),
            ExprKind::False => Theme::keyword("false").to_string(),
            ExprKind::NoObject | ExprKind::NoInterface => Theme::null_value("nullptr").to_string(),
            ExprKind::Self_ => Theme::object_ref("this").to_string(),
            ExprKind::Nothing | ExprKind::NothingInt32 => {
                Theme::null_value("<Nothing>").to_string()
            }

            // Function calls
            ExprKind::VirtualFunction { func, params }
            | ExprKind::FinalFunction { func, params } => {
                let func_name = self.resolve_function(func);
                let param_strs: Vec<String> = params
                    .iter()
                    .map(|p| self.format_expr_inline(p, &FormatContext::This))
                    .collect();
                // These can be called on an object context
                match context {
                    FormatContext::This => {
                        format!("{}({})", Theme::function(func_name), param_strs.join(", "))
                    }
                    FormatContext::Object(obj) => {
                        format!(
                            "{}.{}({})",
                            obj,
                            Theme::function(func_name),
                            param_strs.join(", ")
                        )
                    }
                }
            }
            ExprKind::CallMath { func, params } => {
                // Get the full function path for operator matching
                let full_path = match func {
                    FunctionRef::ByAddress(addr) => self
                        .address_index
                        .resolve_object(*addr)
                        .unwrap()
                        .path
                        .to_string(),
                    FunctionRef::ByName(name) => name.as_str().to_string(),
                };

                let param_strs: Vec<String> = params
                    .iter()
                    .map(|p| self.format_expr_inline(p, &FormatContext::This))
                    .collect();

                // Try to format as an operator first
                if let Some(operator_form) = self.try_format_as_operator(&full_path, &param_strs) {
                    return operator_form;
                }

                // Otherwise, format as a function call
                let func_name = self.resolve_function(func);
                format!("{}({})", Theme::function(func_name), param_strs.join(", "))
            }
            ExprKind::LocalVirtualFunction { func, params }
            | ExprKind::LocalFinalFunction { func, params } => {
                let func_name = self.resolve_function(func);
                let param_strs: Vec<String> = params
                    .iter()
                    .map(|p| self.format_expr_inline(p, &FormatContext::This))
                    .collect();
                let obj = match context {
                    FormatContext::This => Theme::object_ref("this").to_string(),
                    FormatContext::Object(obj) => obj.clone(),
                };
                format!(
                    "{}.{}({})",
                    obj,
                    Theme::function(func_name),
                    param_strs.join(", ")
                )
            }

            // Context/member access
            ExprKind::Context {
                object,
                field: _,
                context,
                skip_offset: _,
                fail_silent: _,
            }
            | ExprKind::ClassContext {
                object,
                field: _,
                context,
                skip_offset: _,
            } => {
                // The object expression determines the new context
                let obj_expr = self.format_expr_inline(object, &FormatContext::This);
                // Format the context expression with the new object context
                let new_context = FormatContext::Object(obj_expr.clone());
                self.format_expr_inline(context, &new_context)
            }
            ExprKind::StructMemberContext {
                struct_expr,
                member,
            } => {
                let expr = self.format_expr_inline(struct_expr, &FormatContext::This);
                let member_name = self.resolve_property(member);
                format!("{}.{}", expr, Theme::variable(member_name))
            }
            ExprKind::InterfaceContext(expr) => {
                let inner = self.format_expr_inline(expr, &FormatContext::This);
                format!("<InterfaceContext>({})", inner)
            }

            // Casts
            ExprKind::DynamicCast { target_class, expr } => {
                let class_name = self.resolve_class(target_class);
                let expr_str = self.format_expr_inline(expr, &FormatContext::This);
                format!("Cast<{}>({})", Theme::type_name(class_name), expr_str)
            }
            ExprKind::MetaCast { target_class, expr } => {
                let class_name = self.resolve_class(target_class);
                let expr_str = self.format_expr_inline(expr, &FormatContext::This);
                format!("MetaCast<{}>({})", Theme::type_name(class_name), expr_str)
            }
            ExprKind::PrimitiveCast {
                conversion_type,
                expr,
            } => {
                let expr_str = self.format_expr_inline(expr, &FormatContext::This);
                format!("({}<{}>)", expr_str, conversion_type)
            }
            ExprKind::ObjToInterfaceCast {
                target_interface,
                expr,
            }
            | ExprKind::InterfaceToObjCast {
                target_class: target_interface,
                expr,
            }
            | ExprKind::CrossInterfaceCast {
                target_interface,
                expr,
            } => {
                let class_name = self.resolve_class(target_interface);
                let expr_str = self.format_expr_inline(expr, &FormatContext::This);
                format!("Cast<{}>({})", Theme::type_name(class_name), expr_str)
            }

            // Collections
            ExprKind::ArrayConst {
                element_type,
                num_elements: _,
                elements,
            } => {
                let type_name = self.resolve_property(element_type);
                let elem_strs: Vec<String> = elements
                    .iter()
                    .map(|e| self.format_expr_inline(e, &FormatContext::This))
                    .collect();
                format!(
                    "TArray<{}>{{ {} }}",
                    Theme::type_name(type_name),
                    elem_strs.join(", ")
                )
            }
            ExprKind::StructConst {
                struct_type,
                serialized_size: _,
                elements,
            } => {
                let struct_name = self.resolve_struct(struct_type);
                let elem_strs: Vec<String> = elements
                    .iter()
                    .map(|e| self.format_expr_inline(e, &FormatContext::This))
                    .collect();
                format!(
                    "{}{{ {} }}",
                    Theme::type_name(struct_name),
                    elem_strs.join(", ")
                )
            }
            ExprKind::SetConst {
                element_type,
                num_elements: _,
                elements,
            } => {
                let type_name = self.resolve_property(element_type);
                let elem_strs: Vec<String> = elements
                    .iter()
                    .map(|e| self.format_expr_inline(e, &FormatContext::This))
                    .collect();
                format!(
                    "TSet<{}>{{ {} }}",
                    Theme::type_name(type_name),
                    elem_strs.join(", ")
                )
            }
            ExprKind::MapConst {
                key_type,
                value_type,
                num_elements: _,
                elements,
            } => {
                let key_type_name = self.resolve_property(key_type);
                let val_type_name = self.resolve_property(value_type);
                let elem_strs: Vec<String> = elements
                    .iter()
                    .map(|e| self.format_expr_inline(e, &FormatContext::This))
                    .collect();
                format!(
                    "TMap<{}, {}>{{ {} }}",
                    Theme::type_name(key_type_name),
                    Theme::type_name(val_type_name),
                    elem_strs.join(", ")
                )
            }

            // Array/set/map operations
            ExprKind::SetArray {
                array_expr,
                elements,
            } => {
                let array = self.format_expr_inline(array_expr, context);
                let elem_strs: Vec<String> = elements
                    .iter()
                    .map(|e| self.format_expr_inline(e, &FormatContext::This))
                    .collect();
                format!("{} = {{ {} }}", array, elem_strs.join(", "))
            }
            ExprKind::SetSet {
                set_expr,
                num: _,
                elements,
            } => {
                let set = self.format_expr_inline(set_expr, context);
                let elem_strs: Vec<String> = elements
                    .iter()
                    .map(|e| self.format_expr_inline(e, &FormatContext::This))
                    .collect();
                format!("{} = {{ {} }}", set, elem_strs.join(", "))
            }
            ExprKind::SetMap {
                map_expr,
                num: _,
                elements,
            } => {
                let map = self.format_expr_inline(map_expr, context);
                let elem_strs: Vec<String> = elements
                    .iter()
                    .map(|e| self.format_expr_inline(e, &FormatContext::This))
                    .collect();
                format!("{} = {{ {} }}", map, elem_strs.join(", "))
            }
            ExprKind::ArrayGetByRef {
                array_expr,
                index_expr,
            } => {
                let array = self.format_expr_inline(array_expr, context);
                let index = self.format_expr_inline(index_expr, &FormatContext::This);
                format!("{}[{}]", array, index)
            }

            // Text constants
            ExprKind::TextConst(text_lit) => match text_lit {
                TextLiteral::Empty => "FText::GetEmpty()".to_string(),
                TextLiteral::LiteralString { source } => {
                    let src = self.format_expr_inline(source, &FormatContext::This);
                    format!("FText::FromString({})", src)
                }
                TextLiteral::InvariantText { source } => {
                    let src = self.format_expr_inline(source, &FormatContext::This);
                    format!("FText::AsCultureInvariant({})", src)
                }
                TextLiteral::LocalizedText {
                    source,
                    key,
                    namespace,
                } => {
                    let src = self.format_expr_inline(source, &FormatContext::This);
                    let k = self.format_expr_inline(key, &FormatContext::This);
                    let ns = self.format_expr_inline(namespace, &FormatContext::This);
                    format!("NSLOCTEXT({}, {}, {})", ns, k, src)
                }
                TextLiteral::StringTableEntry { table_id, key } => {
                    let tid = self.format_expr_inline(table_id, &FormatContext::This);
                    let k = self.format_expr_inline(key, &FormatContext::This);
                    format!("FText::FromStringTable({}, {})", tid, k)
                }
            },

            // Object references
            ExprKind::ObjectConst(obj) => {
                let path = self
                    .address_index
                    .resolve_object(obj.address)
                    .map(|o| o.path)
                    .unwrap_or("<err resolving object>");
                Theme::object_ref(path).to_string()
            }
            ExprKind::PropertyConst(prop) => {
                let name = self.resolve_property(prop);
                Theme::variable(name).to_string()
            }
            ExprKind::SkipOffsetConst(offset) => self.format_label(*offset),

            // Control flow as expressions
            ExprKind::SwitchValue {
                index,
                cases,
                default,
                end_offset: _,
            } => {
                // Format as custom switch expression syntax
                let index_str = self.format_expr_inline(index, context);
                let mut case_strs = Vec::new();

                for case in cases {
                    let case_val = self.format_expr_inline(&case.case_value, &FormatContext::This);
                    let case_result = self.format_expr_inline(&case.result, &FormatContext::This);
                    case_strs.push(format!("{} => {}", case_val, case_result));
                }

                // Add default case
                let default_result = self.format_expr_inline(default, &FormatContext::This);
                case_strs.push(format!("default => {}", default_result));

                format!("switch({}) {{ {} }}", index_str, case_strs.join(", "))
            }

            // Delegates (as expressions)
            ExprKind::InstanceDelegate(name) => {
                format!("InstanceDelegate({})", name.as_str())
            }

            // Other
            _ => Theme::comment(format!("<{:?}>", expr.kind)).to_string(),
        }
    }
}
