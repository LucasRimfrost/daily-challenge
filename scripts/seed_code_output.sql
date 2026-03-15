-- Seed: What's the Output? challenges
-- 2 months back (Jan 15) to 1 month forward (Apr 15) from March 15, 2026
-- Mix of Python, JavaScript, and Rust across easy/medium/hard

-- ── January 2026 ────────────────────────────────────────────────────────────

INSERT INTO code_output_challenges (title, description, language, code_snippet, expected_output, difficulty, hint, max_attempts, scheduled_date) VALUES

('The Classic Swap', 'What does this code print?', 'python',
'a, b = 1, 2
a, b = b, a
print(a, b)', '2 1', 'easy', 'Python allows tuple unpacking in assignments', 3, '2026-01-15'),

('String Multiplication', 'What does this code print?', 'python',
'print("ha" * 3)', 'hahaha', 'easy', 'Strings can be multiplied by integers in Python', 3, '2026-01-16'),

('Falsy Surprise', 'What does this code print?', 'javascript',
'console.log([] == false)', 'true', 'medium', 'JavaScript coerces both sides when using ==', 3, '2026-01-17'),

('Type Confusion', 'What does this code print?', 'javascript',
'console.log(typeof null)', 'object', 'medium', 'This is a famous JavaScript bug that was never fixed', 3, '2026-01-18'),

('List Identity', 'What does this code print?', 'python',
'a = [1, 2, 3]
b = a
b.append(4)
print(len(a))', '4', 'easy', 'Assignment copies the reference, not the list', 3, '2026-01-19'),

('NaN Madness', 'What does this code print?', 'javascript',
'console.log(NaN === NaN)', 'false', 'medium', 'NaN is not equal to anything, including itself', 3, '2026-01-20'),

('Slice and Dice', 'What does this code print?', 'python',
'x = "hello"
print(x[::-1])', 'olleh', 'easy', 'Negative step reverses the sequence', 3, '2026-01-21'),

('Plus Plus What', 'What does this code print?', 'javascript',
'console.log(1 + "2" + 3)', '123', 'medium', 'Once a string enters the +, everything becomes concatenation', 3, '2026-01-22'),

('Mutable Default', 'What does this code print?', 'python',
'def add(x, lst=[]):
    lst.append(x)
    return lst

add(1)
print(add(2))', '[1, 2]', 'hard', 'Default mutable arguments are shared across calls', 3, '2026-01-23'),

('Shadowed Variable', 'What does this code print?', 'rust',
'fn main() {
    let x = 5;
    let x = x + 1;
    let x = x * 2;
    println!("{x}");
}', '12', 'easy', 'Rust allows variable shadowing with let', 3, '2026-01-24'),

('Boolean Arithmetic', 'What does this code print?', 'python',
'print(True + True + False)', '2', 'easy', 'In Python, True is 1 and False is 0', 3, '2026-01-25'),

('Array Sort Surprise', 'What does this code print?', 'javascript',
'console.log([10, 9, 8].sort().join(","))', '10,8,9', 'hard', 'Array.sort() converts elements to strings by default', 3, '2026-01-26'),

('Chained Comparison', 'What does this code print?', 'python',
'print(1 < 2 < 3)', 'True', 'easy', 'Python supports chained comparisons', 3, '2026-01-27'),

('Emergency Exit', 'What does this code print?', 'python',
'for i in range(5):
    if i == 3:
        break
    print(i, end=" ")
print()', '0 1 2 ', 'easy', 'break exits the loop before printing 3', 3, '2026-01-28'),

('Tricky Ternary', 'What does this code print?', 'javascript',
'let x = 0;
console.log(x ? "truthy" : "falsy")', 'falsy', 'easy', '0 is falsy in JavaScript', 3, '2026-01-29'),

('Ownership Move', 'What does this code print?', 'rust',
'fn main() {
    let s1 = String::from("hello");
    let s2 = s1.clone();
    println!("{} {}", s1, s2);
}', 'hello hello', 'easy', 'clone() creates a deep copy so both variables are valid', 3, '2026-01-30'),

('Dict Merge', 'What does this code print?', 'python',
'a = {"x": 1, "y": 2}
b = {"y": 3, "z": 4}
print({**a, **b})', '{''x'': 1, ''y'': 3, ''z'': 4}', 'medium', 'Later keys overwrite earlier ones when merging dicts', 3, '2026-01-31'),

-- ── February 2026 ───────────────────────────────────────────────────────────

('Empty String Check', 'What does this code print?', 'javascript',
'console.log("" == false)', 'true', 'medium', 'Empty string is coerced to 0, and false is coerced to 0', 3, '2026-02-01'),

('Set Surprise', 'What does this code print?', 'python',
'print(len({1, 1, 2, 2, 3}))', '3', 'easy', 'Sets automatically deduplicate', 3, '2026-02-02'),

('Closure Trap', 'What does this code print?', 'javascript',
'const funcs = [];
for (var i = 0; i < 3; i++) {
    funcs.push(() => i);
}
console.log(funcs[0]())', '3', 'hard', 'var is function-scoped, so all closures share the same i', 3, '2026-02-03'),

('Integer Division', 'What does this code print?', 'python',
'print(7 // 2)', '3', 'easy', '// is floor division in Python', 3, '2026-02-04'),

('Template Literal', 'What does this code print?', 'javascript',
'const x = 10;
console.log(`${x > 5 ? "big" : "small"}`)', 'big', 'easy', 'Template literals can contain expressions', 3, '2026-02-05'),

('Tuple Indexing', 'What does this code print?', 'rust',
'fn main() {
    let t = (1, "hello", 3.14);
    println!("{}", t.1);
}', 'hello', 'easy', 'Tuple fields are accessed with .0, .1, .2, etc.', 3, '2026-02-06'),

('List Comprehension', 'What does this code print?', 'python',
'print([x**2 for x in range(5) if x % 2 == 0])', '[0, 4, 16]', 'medium', 'Filter runs before the transformation', 3, '2026-02-07'),

('Typeof Undefined', 'What does this code print?', 'javascript',
'let x;
console.log(typeof x)', 'undefined', 'easy', 'Declared but unassigned variables are undefined', 3, '2026-02-08'),

('Zip Magic', 'What does this code print?', 'python',
'a = [1, 2, 3]
b = ["a", "b", "c"]
print(list(zip(a, b)))', '[(1, ''a''), (2, ''b''), (3, ''c'')]', 'medium', 'zip pairs elements from both iterables', 3, '2026-02-09'),

('Spread Merge', 'What does this code print?', 'javascript',
'const a = [1, 2];
const b = [3, 4];
console.log([...a, ...b].length)', '4', 'easy', 'Spread operator flattens arrays', 3, '2026-02-10'),

('Match Expression', 'What does this code print?', 'rust',
'fn main() {
    let x = 42;
    let msg = match x {
        0..=9 => "small",
        10..=99 => "medium",
        _ => "big",
    };
    println!("{msg}");
}', 'medium', 'easy', 'match with ranges checks which range contains the value', 3, '2026-02-11'),

('String Join', 'What does this code print?', 'python',
'print("-".join(["a", "b", "c"]))', 'a-b-c', 'easy', 'join() places the separator between elements', 3, '2026-02-12'),

('Nullish Coalescing', 'What does this code print?', 'javascript',
'const x = null;
const y = 0;
console.log(x ?? "default", y ?? "default")', 'default 0', 'medium', '?? only checks for null/undefined, not falsy values', 3, '2026-02-13'),

('Enumerate', 'What does this code print?', 'python',
'for i, c in enumerate("abc"):
    print(i, end="")
print()', '012', 'easy', 'enumerate gives (index, value) pairs starting from 0', 3, '2026-02-14'),

('Destructuring', 'What does this code print?', 'javascript',
'const [a, , b] = [1, 2, 3, 4];
console.log(a + b)', '4', 'medium', 'The empty slot skips the second element', 3, '2026-02-15'),

('Vector Push', 'What does this code print?', 'rust',
'fn main() {
    let mut v = vec![1, 2, 3];
    v.push(4);
    v.push(5);
    println!("{}", v.len());
}', '5', 'easy', 'push adds elements to the end of the vector', 3, '2026-02-16'),

('Global vs Local', 'What does this code print?', 'python',
'x = 10
def foo():
    x = 20
    return x
foo()
print(x)', '10', 'medium', 'Assignment inside a function creates a local variable', 3, '2026-02-17'),

('Void Operator', 'What does this code print?', 'javascript',
'console.log(void 0 === undefined)', 'true', 'medium', 'void always evaluates to undefined', 3, '2026-02-18'),

('Iterator Sum', 'What does this code print?', 'rust',
'fn main() {
    let total: i32 = (1..=4).sum();
    println!("{total}");
}', '10', 'easy', '1..=4 is an inclusive range: 1+2+3+4', 3, '2026-02-19'),

('In Operator', 'What does this code print?', 'python',
'print("hell" in "hello world")', 'True', 'easy', 'in checks for substring membership in strings', 3, '2026-02-20'),

('Optional Chaining', 'What does this code print?', 'javascript',
'const obj = { a: { b: 42 } };
console.log(obj?.a?.b)', '42', 'easy', 'Optional chaining returns the value if the chain is valid', 3, '2026-02-21'),

('F-String Format', 'What does this code print?', 'python',
'pi = 3.14159
print(f"{pi:.2f}")', '3.14', 'medium', '.2f formats to 2 decimal places', 3, '2026-02-22'),

('Infinity Check', 'What does this code print?', 'javascript',
'console.log(1 / 0)', 'Infinity', 'medium', 'JavaScript returns Infinity instead of throwing an error', 3, '2026-02-23'),

('String Iter', 'What does this code print?', 'rust',
'fn main() {
    let s = "hi";
    println!("{}", s.chars().count());
}', '2', 'easy', 'chars().count() gives the number of characters', 3, '2026-02-24'),

('Walrus Operator', 'What does this code print?', 'python',
'if (n := 10) > 5:
    print(n)', '10', 'medium', ':= assigns and returns the value in one expression', 3, '2026-02-25'),

('Array From', 'What does this code print?', 'javascript',
'console.log(Array.from("hello").length)', '5', 'easy', 'Array.from converts an iterable to an array', 3, '2026-02-26'),

('Nested List', 'What does this code print?', 'python',
'matrix = [[1,2],[3,4],[5,6]]
flat = [x for row in matrix for x in row]
print(flat)', '[1, 2, 3, 4, 5, 6]', 'medium', 'Nested comprehension reads left to right like nested loops', 3, '2026-02-27'),

('Map Filter', 'What does this code print?', 'javascript',
'console.log([1,2,3,4,5].filter(x => x % 2).map(x => x * 10).join(","))', '10,30,50', 'medium', 'filter keeps odd numbers (truthy), then map multiplies by 10', 3, '2026-02-28'),

-- ── March 2026 ──────────────────────────────────────────────────────────────

('If Let', 'What does this code print?', 'rust',
'fn main() {
    let val: Option<i32> = Some(42);
    if let Some(x) = val {
        println!("{x}");
    }
}', '42', 'easy', 'if let destructures the Option and binds the inner value', 3, '2026-03-01'),

('Any All', 'What does this code print?', 'python',
'nums = [2, 4, 6, 8]
print(all(x % 2 == 0 for x in nums))', 'True', 'easy', 'all() returns True if every element satisfies the condition', 3, '2026-03-02'),

('Double Equals', 'What does this code print?', 'javascript',
'console.log(0 == "")', 'true', 'hard', 'Both are coerced to 0 with ==', 3, '2026-03-03'),

('Enumerate Start', 'What does this code print?', 'python',
'for i, x in enumerate(["a","b","c"], start=1):
    pass
print(i, x)', '3 c', 'medium', 'enumerate with start=1 begins counting from 1', 3, '2026-03-04'),

('Map Object', 'What does this code print?', 'javascript',
'const m = new Map();
m.set("a", 1);
m.set("b", 2);
m.set("a", 3);
console.log(m.size)', '2', 'medium', 'Map keys are unique — setting "a" again overwrites it', 3, '2026-03-05'),

('Collect Vector', 'What does this code print?', 'rust',
'fn main() {
    let v: Vec<i32> = (0..5).filter(|x| x % 2 == 0).collect();
    println!("{:?}", v);
}', '[0, 2, 4]', 'medium', 'filter keeps elements where the predicate is true', 3, '2026-03-06'),

('Dict Get', 'What does this code print?', 'python',
'd = {"a": 1, "b": 2}
print(d.get("c", 99))', '99', 'easy', 'get() returns the default value when the key is missing', 3, '2026-03-07'),

('Reduce Sum', 'What does this code print?', 'javascript',
'console.log([1,2,3,4].reduce((a,b) => a + b, 0))', '10', 'easy', 'reduce accumulates values starting from the initial value 0', 3, '2026-03-08'),

('Power Tower', 'What does this code print?', 'python',
'print(2 ** 3 ** 2)', '512', 'hard', 'Exponentiation is right-associative: 2 ** (3 ** 2) = 2 ** 9', 3, '2026-03-09'),

('String Repeat', 'What does this code print?', 'rust',
'fn main() {
    let s = "ab".repeat(3);
    println!("{s}");
}', 'ababab', 'easy', 'repeat() duplicates the string n times', 3, '2026-03-10'),

('Truthy Empty', 'What does this code print?', 'javascript',
'console.log(Boolean([]))', 'true', 'hard', 'Empty arrays are truthy in JavaScript — only [] == false uses coercion', 3, '2026-03-11'),

('Counter', 'What does this code print?', 'python',
'from collections import Counter
print(Counter("banana")["a"])', '3', 'easy', 'Counter counts occurrences of each element', 3, '2026-03-12'),

('Reverse Iterator', 'What does this code print?', 'rust',
'fn main() {
    let v = vec![1, 2, 3];
    let r: Vec<_> = v.iter().rev().collect();
    println!("{:?}", r);
}', '[3, 2, 1]', 'medium', 'rev() reverses the iterator order', 3, '2026-03-13'),

('String Contains', 'What does this code print?', 'javascript',
'console.log("foobar".includes("oob"))', 'true', 'easy', 'includes() checks for substring presence', 3, '2026-03-14'),

('Lambda Sort', 'What does this code print?', 'python',
'words = ["banana", "pie", "kiwi"]
words.sort(key=len)
print(words)', '[''pie'', ''kiwi'', ''banana'']', 'medium', 'key=len sorts by string length', 3, '2026-03-15'),

('Typeof Array', 'What does this code print?', 'javascript',
'console.log(typeof [])', 'object', 'medium', 'Arrays are objects in JavaScript — use Array.isArray() to check', 3, '2026-03-16'),

('Implicit Return', 'What does this code print?', 'rust',
'fn double(x: i32) -> i32 {
    x * 2
}
fn main() {
    println!("{}", double(21));
}', '42', 'easy', 'The last expression without a semicolon is the return value', 3, '2026-03-17'),

('Map vs List', 'What does this code print?', 'python',
'result = map(lambda x: x*2, [1,2,3])
print(type(result).__name__)', 'map', 'medium', 'map() returns a lazy iterator, not a list', 3, '2026-03-18'),

('Promise Value', 'What does this code print?', 'javascript',
'const x = "start";
setTimeout(() => console.log("timeout"), 0);
console.log(x)', 'start', 'hard', 'setTimeout callbacks run after the current call stack, even with 0ms delay', 3, '2026-03-19'),

('Struct Debug', 'What does this code print?', 'rust',
'#[derive(Debug)]
struct Point { x: i32, y: i32 }
fn main() {
    let p = Point { x: 3, y: 7 };
    println!("{:?}", p);
}', 'Point { x: 3, y: 7 }', 'easy', 'Debug trait prints the struct name and fields', 3, '2026-03-20'),

('Not Not', 'What does this code print?', 'python',
'print(not not [])', 'False', 'medium', 'not not converts to bool — empty list is falsy', 3, '2026-03-21'),

('Object Keys', 'What does this code print?', 'javascript',
'const obj = { a: 1, b: 2, c: 3 };
console.log(Object.keys(obj).length)', '3', 'easy', 'Object.keys() returns an array of the property names', 3, '2026-03-22'),

('Char Collect', 'What does this code print?', 'rust',
'fn main() {
    let s: String = "hello".chars().filter(|c| *c != ''l'').collect();
    println!("{s}");
}', 'heo', 'medium', 'filter removes matching characters, collect builds a new String', 3, '2026-03-23'),

('Floor Negative', 'What does this code print?', 'python',
'print(-7 // 2)', '-4', 'hard', 'Floor division rounds toward negative infinity, not zero', 3, '2026-03-24'),

('String Pad', 'What does this code print?', 'javascript',
'console.log("42".padStart(5, "0"))', '00042', 'easy', 'padStart adds characters to the beginning until the target length', 3, '2026-03-25'),

('Vec Dedup', 'What does this code print?', 'rust',
'fn main() {
    let mut v = vec![1, 1, 2, 3, 3, 3, 2];
    v.dedup();
    println!("{:?}", v);
}', '[1, 2, 3, 2]', 'hard', 'dedup only removes consecutive duplicates, not all duplicates', 3, '2026-03-26'),

('Unpacking Star', 'What does this code print?', 'python',
'first, *middle, last = [1, 2, 3, 4, 5]
print(middle)', '[2, 3, 4]', 'medium', 'The * collects everything between first and last', 3, '2026-03-27'),

('Comparison Chain', 'What does this code print?', 'javascript',
'console.log(3 > 2 > 1)', 'false', 'hard', '3 > 2 is true, then true > 1 is false (true coerces to 1, 1 > 1 is false)', 3, '2026-03-28'),

('HashMap Entry', 'What does this code print?', 'rust',
'use std::collections::HashMap;
fn main() {
    let mut m = HashMap::new();
    m.insert("a", 1);
    m.insert("a", 2);
    println!("{}", m["a"]);
}', '2', 'easy', 'insert overwrites the value for an existing key', 3, '2026-03-29'),

('Set Operations', 'What does this code print?', 'python',
'a = {1, 2, 3, 4}
b = {3, 4, 5, 6}
print(sorted(a & b))', '[3, 4]', 'medium', '& is set intersection — elements in both sets', 3, '2026-03-30'),

('Arrow Return', 'What does this code print?', 'javascript',
'const add = (a, b) => a + b;
console.log(add(3, 4))', '7', 'easy', 'Arrow functions with no braces implicitly return the expression', 3, '2026-03-31'),

-- ── April 2026 ──────────────────────────────────────────────────────────────

('Enum Match', 'What does this code print?', 'rust',
'enum Color { Red, Green, Blue }
fn main() {
    let c = Color::Green;
    match c {
        Color::Red => println!("R"),
        Color::Green => println!("G"),
        Color::Blue => println!("B"),
    }
}', 'G', 'easy', 'match checks which variant the enum is', 3, '2026-04-01'),

('Truthiness', 'What does this code print?', 'python',
'print(bool(""), bool(" "), bool("0"))', 'False True True', 'medium', 'Only empty string is falsy — " " and "0" have length > 0', 3, '2026-04-02'),

('Delete Property', 'What does this code print?', 'javascript',
'const obj = {a: 1, b: 2, c: 3};
delete obj.b;
console.log(Object.keys(obj).join(","))', 'a,c', 'easy', 'delete removes a property from an object', 3, '2026-04-03'),

('Closure Move', 'What does this code print?', 'rust',
'fn main() {
    let name = String::from("Rust");
    let greet = move || println!("Hello, {name}!");
    greet();
}', 'Hello, Rust!', 'medium', 'move captures variables by value into the closure', 3, '2026-04-04'),

('Generator Next', 'What does this code print?', 'python',
'def gen():
    yield 1
    yield 2
    yield 3
g = gen()
next(g)
print(next(g))', '2', 'medium', 'First next() yields 1, second next() yields 2', 3, '2026-04-05'),

('Short Circuit', 'What does this code print?', 'javascript',
'console.log(0 || "" || null || "hello" || "world")', 'hello', 'medium', '|| returns the first truthy value it finds', 3, '2026-04-06'),

('Reference Borrow', 'What does this code print?', 'rust',
'fn first(s: &str) -> &str {
    &s[..1]
}
fn main() {
    let word = String::from("hello");
    println!("{}", first(&word));
}', 'h', 'medium', '&s[..1] borrows the first byte/character of the string', 3, '2026-04-07'),

('Multiple Return', 'What does this code print?', 'python',
'def minmax(lst):
    return min(lst), max(lst)
lo, hi = minmax([5, 2, 8, 1])
print(lo, hi)', '1 8', 'easy', 'Python functions can return tuples which can be unpacked', 3, '2026-04-08'),

('Array Find', 'What does this code print?', 'javascript',
'const arr = [5, 12, 8, 130, 44];
console.log(arr.find(x => x > 10))', '12', 'easy', 'find() returns the first element that matches the predicate', 3, '2026-04-09'),

('Option Map', 'What does this code print?', 'rust',
'fn main() {
    let x: Option<i32> = Some(5);
    let y = x.map(|v| v * 2);
    println!("{:?}", y);
}', 'Some(10)', 'medium', 'map on Some applies the function to the inner value', 3, '2026-04-10'),

('Dict Comprehension', 'What does this code print?', 'python',
'd = {k: v for k, v in enumerate("abc")}
print(d)', '{0: ''a'', 1: ''b'', 2: ''c''}', 'medium', 'enumerate gives (index, value) pairs for the dict', 3, '2026-04-11'),

('Splice Return', 'What does this code print?', 'javascript',
'const arr = [1, 2, 3, 4, 5];
const removed = arr.splice(1, 2);
console.log(arr.join(","))', '1,4,5', 'medium', 'splice(1, 2) removes 2 elements starting at index 1', 3, '2026-04-12'),

('Result Unwrap', 'What does this code print?', 'rust',
'fn divide(a: f64, b: f64) -> Result<f64, String> {
    if b == 0.0 {
        Err("division by zero".into())
    } else {
        Ok(a / b)
    }
}
fn main() {
    println!("{}", divide(10.0, 4.0).unwrap());
}', '2.5', 'easy', 'unwrap() extracts the Ok value from a Result', 3, '2026-04-13'),

('Frozen Set', 'What does this code print?', 'python',
'a = frozenset([3, 1, 2, 1, 3])
print(sorted(a))', '[1, 2, 3]', 'easy', 'frozenset deduplicates like set, sorted() returns a list', 3, '2026-04-14'),

('Number Methods', 'What does this code print?', 'javascript',
'console.log(Number.isInteger(5.0))', 'true', 'medium', '5.0 has no fractional part, so JavaScript considers it an integer', 3, '2026-04-15');