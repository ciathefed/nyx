const fehler = @import("fehler");

const Span = @This();

filename: []const u8,
start: usize,
end: usize,

pub fn init(start: usize, end: usize, filename: []const u8) Span {
    return Span{ .start = start, .end = end, .filename = filename };
}

pub fn toSourceRange(self: *const Span, source: []const u8) fehler.SourceRange {
    var line: usize = 1;
    var column: usize = 1;
    var start_pos: ?fehler.Position = null;
    var end_pos: ?fehler.Position = null;

    for (source, 0..) |ch, i| {
        if (start_pos == null and i == self.start) {
            start_pos = fehler.Position{ .line = line, .column = column };
        }
        if (end_pos == null and i == self.end) {
            end_pos = fehler.Position{ .line = line, .column = column };
            break;
        }

        switch (ch) {
            '\n' => {
                line += 1;
                column = 1;
            },
            '\r' => {
                if (i + 1 < source.len and source[i + 1] != '\n') {
                    line += 1;
                    column = 1;
                } else {
                    column += 1;
                }
            },
            else => {
                column += 1;
            },
        }
    }

    if (end_pos == null) {
        end_pos = fehler.Position{ .line = line, .column = column };
    }
    if (start_pos == null) {
        start_pos = fehler.Position{ .line = 1, .column = 1 };
    }

    return fehler.SourceRange{
        .file = self.filename,
        .start = start_pos.?,
        .end = end_pos.?,
    };
}
