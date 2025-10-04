const Flags = @This();

eq: bool,
lt: bool,

pub fn init() Flags {
    return Flags{
        .eq = false,
        .lt = false,
    };
}
