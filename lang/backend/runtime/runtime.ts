const __fs = require("node:fs");

// primitives
type I64 = bigint;
type F64 = number;
type String_ = string;
type Char = number;

// data
type Unit = { tag: "MkUnit"; args: [] };
type Option<T> = { tag: "Some"; args: [T] } | { tag: "None"; args: [] };
type List<T> = { tag: "Cons"; args: [T, List<T>] } | { tag: "Nil"; args: [] };
function __arr_to_list<T>(arr: T[]): List<T> {
  let list: List<T> = { tag: "Nil", args: [] };
  for (const x of [...arr].reverse()) {
    list = { tag: "Cons", args: [x, list] };
  }
  return list;
}

// IO
type IO<T> = () => T;

function $add_i64(x: I64, y: I64): I64 {
  return BigInt.asIntN(64, x + y);
}

function $sub_i64(x: I64, y: I64): I64 {
  return BigInt.asIntN(64, x - y);
}

function $mul_i64(x: I64, y: I64): I64 {
  return BigInt.asIntN(64, x * y);
}

function $div_i64(x: I64, y: I64): I64 {
  return BigInt.asIntN(64, x / y);
}

function $add_f64(x: F64, y: F64): F64 {
  return x + y;
}

function $sub_f64(x: F64, y: F64): F64 {
  return x - y;
}

function $mul_f64(x: F64, y: F64): F64 {
  return x * y;
}

function $div_f64(x: F64, y: F64): F64 {
  return x / y;
}

function $concat_string(s1: String_, s2: String_): String_ {
  return s1.concat(s2);
}

function $append_char(c: Char, s: String_): String_ {
  return s.concat(String.fromCodePoint(c));
}

function $string_to_chars(s: String_): List<Char> {
  return __arr_to_list(Array.from(s, (c) => c.codePointAt(0)!));
}

function $return_io<T>(x: T): IO<T> {
  return function () {
    return x;
  };
}

function $print(s: String_): IO<Unit> {
  return function () {
    process.stdout.write(s);
    return {
      tag: "MkUnit",
      args: [],
    };
  };
}

function $read_file(path: String_): IO<Option<String_>> {
  return function () {
    try {
      const data: String_ = __fs.readFileSync(path, "utf8");
      return {
        tag: "Some",
        args: [data],
      };
    } catch (err) {
      return {
        tag: "None",
        args: [],
      };
    }
  };
}

function $args(): IO<List<String_>> {
  return function () {
    return __arr_to_list(process.argv.slice(2));
  };
}
