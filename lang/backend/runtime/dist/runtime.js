"use strict";
const __fs = require("node:fs");
function add_i64(x, y) {
    return BigInt.asIntN(64, x + y);
}
function sub_i64(x, y) {
    return BigInt.asIntN(64, x - y);
}
function mul_i64(x, y) {
    return BigInt.asIntN(64, x * y);
}
function div_i64(x, y) {
    return BigInt.asIntN(64, x / y);
}
function add_f64(x, y) {
    return x + y;
}
function sub_f64(x, y) {
    return x - y;
}
function mul_f64(x, y) {
    return x * y;
}
function div_f64(x, y) {
    return x / y;
}
function concat(s1, s2) {
    return s1.concat(s2);
}
function append_char(c, s) {
    return s.concat(String.fromCodePoint(c));
}
function return_io(x) {
    return function () {
        return x;
    };
}
function print(s) {
    return function () {
        process.stdout.write(s);
        return {
            tag: "MkUnit",
            args: [],
        };
    };
}
function read_file(path) {
    return function () {
        try {
            const data = __fs.readFileSync(path, "utf8");
            return {
                tag: "Some",
                args: [data],
            };
        }
        catch (err) {
            return {
                tag: "None",
                args: [],
            };
        }
    };
}
function args() {
    return function () {
        let args = {
            tag: "Nil",
            args: [],
        };
        for (const arg of process.argv.slice(2).reverse()) {
            args = {
                tag: "Cons",
                args: [arg, args],
            };
        }
        return args;
    };
}
