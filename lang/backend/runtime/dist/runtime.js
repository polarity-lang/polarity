import * as __fs from "node:fs";
import * as __readline from "node:readline/promises";
const __true = { tag: "T", args: [] };
const __false = { tag: "F", args: [] };
function $add_i64(x, y) {
    return BigInt.asIntN(64, x + y);
}
function $sub_i64(x, y) {
    return BigInt.asIntN(64, x - y);
}
function $mul_i64(x, y) {
    return BigInt.asIntN(64, x * y);
}
function $div_i64(x, y) {
    return BigInt.asIntN(64, x / y);
}
function $eq_i64(x, y) {
    return x === y ? __true : __false;
}
function $add_f64(x, y) {
    return x + y;
}
function $sub_f64(x, y) {
    return x - y;
}
function $mul_f64(x, y) {
    return x * y;
}
function $div_f64(x, y) {
    return x / y;
}
function $eq_f64(x, y) {
    return x === y ? __true : __false;
}
function $eq_char(x, y) {
    return x === y ? __true : __false;
}
function $concat_string(s1, s2) {
    return s1.concat(s2);
}
function $append_char(c, s) {
    return s.concat(String.fromCodePoint(c));
}
function $eq_string(x, y) {
    return x === y ? __true : __false;
}
function $return_io(x) {
    return async function () {
        return x;
    };
}
function $print(s) {
    return async function () {
        process.stdout.write(s);
        return {
            tag: "MkUnit",
            args: [],
        };
    };
}
function $read_file(path) {
    return async function () {
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
function $readln(prompt) {
    return async function () {
        const rl = __readline.createInterface({
            input: process.stdin,
            output: process.stdout,
        });
        const answer = await rl.question(prompt);
        rl.close();
        return answer;
    };
}
function $args() {
    return async function () {
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
