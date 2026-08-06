import * as __fs from "node:fs";

// primitives
type I64 = bigint;
type F64 = number;
type String_ = string;
type Char = number;

// data
type Unit = { tag: "MkUnit"; args: [] };
type Bool = { tag: "T"; args: [] } | { tag: "F"; args: [] };
const __true: Bool = { tag: "T", args: [] };
const __false: Bool = { tag: "F", args: [] };
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
type IO<T> = () => Promise<T>;

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

function $eq_i64(x: I64, y: I64): Bool {
  return x === y ? __true : __false;
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

function $eq_f64(x: F64, y: F64): Bool {
  return x === y ? __true : __false;
}

function $eq_char(x: Char, y: Char): Bool {
  return x === y ? __true : __false;
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

function $eq_string(x: String_, y: String_): Bool {
  return x === y ? __true : __false;
}

function $return_io<T>(x: T): IO<T> {
  return async function () {
    return x;
  };
}

function $print(s: String_): IO<Unit> {
  return async function () {
    process.stdout.write(s);
    return {
      tag: "MkUnit",
      args: [],
    };
  };
}

function $read_file(path: String_): IO<Option<String_>> {
  return async function () {
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
  return async function () {
    return __arr_to_list(process.argv.slice(2));
  };
}
