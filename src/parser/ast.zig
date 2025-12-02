// TODO: all expressions have a span for better error reporting

const std = @import("std");
const Span = @import("../Span.zig");
const StringInterner = @import("../StringInterner.zig");
const StringId = StringInterner.StringId;
const DataSize = @import("immediate.zig").DataSize;
const Register = @import("../vm/register.zig").Register;

pub const Statement = union(enum) {
    label: Label,
    @"error": Expr1,
    define: Define,
    include: Expr1,
    ifdef: Expr1,
    ifndef: Expr1,
    @"else": Span,
    endif: Span,
    section: Section,
    entry: Expr1,
    ascii: Expr1,
    asciz: Expr1,
    @"extern": Expr1,
    nop: Span,
    mov: Expr2,
    ldr: Expr2,
    str: Expr2,
    sti: Expr3,
    push: PushPop,
    pop: PushPop,
    add: Expr3,
    sub: Expr3,
    mul: Expr3,
    div: Expr3,
    @"and": Expr3,
    @"or": Expr3,
    xor: Expr3,
    shl: Expr3,
    shr: Expr3,
    cmp: Expr2,
    jmp: Expr1,
    jne: Expr1,
    jeq: Expr1,
    jlt: Expr1,
    jgt: Expr1,
    jle: Expr1,
    jge: Expr1,
    call: Expr1,
    ret: Span,
    inc: Expr1,
    dec: Expr1,
    neg: Expr1,
    syscall: Span,
    hlt: Span,
    db: Db,
    dw: Db,
    dd: Db,
    dq: Db,
    resb: Expr1,
    resw: Expr1,
    resd: Expr1,
    resq: Expr1,

    pub const Expr1 = struct {
        expr: *Expression,
        span: Span,
    };

    pub const Expr2 = struct {
        expr1: *Expression,
        expr2: *Expression,
        span: Span,
    };

    pub const Expr3 = struct {
        expr1: *Expression,
        expr2: *Expression,
        expr3: *Expression,
        span: Span,
    };

    pub const Label = struct {
        name: StringId,
        span: Span,
    };

    pub const Define = struct {
        name: *Expression,
        expr: ?*Expression,
        span: Span,
    };

    pub const Section = struct {
        type: Type,
        span: Span,
        pub const Type = enum { text, data };
    };

    pub const PushPop = struct {
        data_size: ?*Expression,
        expr: *Expression,
        span: Span,
    };

    // TODO: each expr should have its own span
    pub const Db = struct {
        exprs: []*Expression,
        span: Span,
    };

    pub fn span(self: Statement) Span {
        return switch (self) {
            .label => |v| v.span,
            .@"error" => |v| v.span,
            .define => |v| v.span,
            .include => |v| v.span,
            .ifdef => |v| v.span,
            .ifndef => |v| v.span,
            .@"else" => |v| v,
            .endif => |v| v,
            .section => |v| v.span,
            .entry => |v| v.span,
            .ascii => |v| v.span,
            .asciz => |v| v.span,
            .@"extern" => |v| v.span,
            .nop => |v| v,
            .mov => |v| v.span,
            .ldr => |v| v.span,
            .str => |v| v.span,
            .sti => |v| v.span,
            .push => |v| v.span,
            .pop => |v| v.span,
            .add => |v| v.span,
            .sub => |v| v.span,
            .mul => |v| v.span,
            .div => |v| v.span,
            .@"and" => |v| v.span,
            .@"or" => |v| v.span,
            .xor => |v| v.span,
            .shl => |v| v.span,
            .shr => |v| v.span,
            .cmp => |v| v.span,
            .jmp => |v| v.span,
            .jne => |v| v.span,
            .jeq => |v| v.span,
            .jlt => |v| v.span,
            .jgt => |v| v.span,
            .jle => |v| v.span,
            .jge => |v| v.span,
            .call => |v| v.span,
            .ret => |v| v,
            .inc => |v| v.span,
            .dec => |v| v.span,
            .neg => |v| v.span,
            .syscall => |v| v,
            .hlt => |v| v,
            .db => |v| v.span,
            .dw => |v| v.span,
            .dd => |v| v.span,
            .dq => |v| v.span,
            .resb => |v| v.span,
            .resw => |v| v.span,
            .resd => |v| v.span,
            .resq => |v| v.span,
        };
    }
};

pub const Expression = union(enum) {
    identifier: StringId,
    register: Register,
    integer_literal: i64,
    float_literal: f64,
    string_literal: StringId,
    data_size: DataSize,
    address: Address,
    unary_op: UnaryOp,
    binary_op: BinaryOp,

    pub const Address = struct {
        base: *Expression,
        offset: ?*Expression,
    };

    pub const UnaryOp = struct {
        expr: *Expression,
        op: Op,
        span: Span,

        pub const Op = enum {
            neg,
        };
    };

    pub const BinaryOp = struct {
        lhs: *Expression,
        op: Op,
        rhs: *Expression,
        span: Span,

        pub const Op = enum {
            add, // +
            sub, // -
            mul, // *
            div, // /
            bit_or, // |
            bit_and, // &
            bit_xor, // ^
        };
    };
};
