import { readFileSync } from "node:fs";

export function add_i64(x, y) {
    return BigInt.asIntN(64, x + y);
}

export function sub_i64(x, y) {
    return BigInt.asIntN(64, x - y);
}

export function mul_i64(x, y) {
    return BigInt.asIntN(64, x * y);
}

export function div_i64(x, y) {
    return BigInt.asIntN(64, x / y);
}

export function add_f64(x, y) {
    return x + y;
}

export function sub_f64(x, y) {
    return x - y;
}

export function mul_f64(x, y) {
    return x * y;
}

export function div_f64(x, y) {
    return x / y;
}

export function concat(s1, s2) {
    s1.concat(s2)
}

export function append_char(c, s) {
    s.concat(String.fromCodePoint(c))
}

export function unit() {
    return void 0;
}

export function return_io(x) {
    return function() {
        return x;
    };
}

export function println(s) {
    return function() {
        console.log(s);
        return void 0;
    };
}

export function read_file(path) {
    return function() {
        try {
            const data = readFileSync(path, 'utf8');
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
