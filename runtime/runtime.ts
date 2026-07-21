const __fs = require("node:fs");

// primitives
type I64 = bigint
type F64 = number
type String_ = string
type Char = number
type Unit = undefined

// data
type Option<T> =
    | { tag: "Some", args: [T] }
    | { tag: "None", args: [] };

// IO
type IO<T> = () => T

function add_i64(x: I64, y: I64): I64 {
    return BigInt.asIntN(64, x + y);
}

function sub_i64(x: I64, y: I64): I64 {
    return BigInt.asIntN(64, x - y);
}

function mul_i64(x: I64, y: I64): I64 {
    return BigInt.asIntN(64, x * y);
}

function div_i64(x: I64, y: I64): I64 {
    return BigInt.asIntN(64, x / y);
}

function add_f64(x: F64, y: F64): F64 {
    return x + y;
}

function sub_f64(x: F64, y: F64): F64 {
    return x - y;
}

function mul_f64(x: F64, y: F64): F64 {
    return x * y;
}

function div_f64(x: F64, y: F64): F64 {
    return x / y;
}

function concat(s1: String_, s2: String_): String_ {
    return s1.concat(s2);
}

function append_char(c: Char, s: String_): String_ {
    return s.concat(String.fromCodePoint(c));
}

function unit(): Unit {
    return void 0;
}

function return_io<T>(x: T): IO<T> {
    return function() {
        return x;
    };
}

function println(s: String_): IO<Unit> {
    return function() {
        console.log(s);
        return void 0;
    };
}

function read_file(path: String_): IO<Option<String_>> {
    return function() {
        try {
            const data: String_ = __fs.readFileSync(path, "utf8");
            return {
                tag: "Some",
                args: [data]
            };
        } catch (err) {
            return {
                tag: "None",
                args: []
            };
        }
    };
}
